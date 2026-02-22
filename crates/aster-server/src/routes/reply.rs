use crate::state::AppState;
use aster::agents::{AgentEvent, SessionConfig};
use aster::context::ContextTraceStep;
use aster::conversation::message::{Message, MessageContent, TokenState};
use aster::conversation::Conversation;
use aster::session::SessionManager;
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{self, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use bytes::Bytes;
use futures::{stream::StreamExt, Stream};
use rmcp::model::ServerNotification;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

fn track_tool_telemetry(content: &MessageContent, all_messages: &[Message]) {
    match content {
        MessageContent::ToolRequest(tool_request) => {
            if let Ok(tool_call) = &tool_request.tool_call {
                tracing::info!(monotonic_counter.aster.tool_calls = 1,
                    tool_name = %tool_call.name,
                    "Tool call started"
                );
            }
        }
        MessageContent::ToolResponse(tool_response) => {
            let tool_name = all_messages
                .iter()
                .rev()
                .find_map(|msg| {
                    msg.content.iter().find_map(|c| {
                        if let MessageContent::ToolRequest(req) = c {
                            if req.id == tool_response.id {
                                if let Ok(tool_call) = &req.tool_call {
                                    Some(tool_call.name.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or_else(|| "unknown".to_string().into());

            let success = tool_response.tool_result.is_ok();
            let result_status = if success { "success" } else { "error" };

            tracing::info!(
                counter.aster.tool_completions = 1,
                tool_name = %tool_name,
                result = %result_status,
                "Tool call completed"
            );
        }
        _ => {}
    }
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ChatRequest {
    user_message: Message,
    #[serde(default)]
    conversation_so_far: Option<Vec<Message>>,
    session_id: String,
    recipe_name: Option<String>,
    recipe_version: Option<String>,
    #[serde(default)]
    include_context_trace: Option<bool>,
    #[serde(default)]
    context_trace_level: Option<ContextTraceLevel>,
    #[serde(default)]
    context_trace_redact: Option<bool>,
}

pub struct SseResponse {
    rx: ReceiverStream<String>,
}

impl SseResponse {
    fn new(rx: ReceiverStream<String>) -> Self {
        Self { rx }
    }
}

impl Stream for SseResponse {
    type Item = Result<Bytes, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx)
            .poll_next(cx)
            .map(|opt| opt.map(|s| Ok(Bytes::from(s))))
    }
}

impl IntoResponse for SseResponse {
    fn into_response(self) -> axum::response::Response {
        let stream = self;
        let body = axum::body::Body::from_stream(stream);

        http::Response::builder()
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .body(body)
            .unwrap()
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(tag = "type")]
pub enum MessageEvent {
    Message {
        message: Message,
        token_state: TokenState,
    },
    Error {
        error: String,
    },
    Finish {
        reason: String,
        token_state: TokenState,
    },
    ModelChange {
        model: String,
        mode: String,
    },
    Notification {
        request_id: String,
        #[schema(value_type = Object)]
        message: ServerNotification,
    },
    UpdateConversation {
        conversation: Conversation,
    },
    ContextTrace {
        steps: Vec<ContextTraceStepResponse>,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextTraceStepResponse {
    pub stage: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, utoipa::ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContextTraceLevel {
    #[default]
    Basic,
    Verbose,
}

impl From<ContextTraceStep> for ContextTraceStepResponse {
    fn from(value: ContextTraceStep) -> Self {
        Self {
            stage: value.stage,
            detail: value.detail,
        }
    }
}

fn transform_context_trace_steps(
    steps: Vec<ContextTraceStep>,
    level: ContextTraceLevel,
    redact: bool,
) -> Vec<ContextTraceStepResponse> {
    steps
        .into_iter()
        .map(|step| {
            let mut detail = match level {
                ContextTraceLevel::Basic => simplify_trace_detail(&step.stage, &step.detail),
                ContextTraceLevel::Verbose => step.detail,
            };

            if redact {
                detail = redact_trace_detail(&detail);
            }

            ContextTraceStepResponse {
                stage: step.stage,
                detail,
            }
        })
        .collect()
}

fn simplify_trace_detail(stage: &str, detail: &str) -> String {
    match stage {
        "session" => "session initialized".to_string(),
        "conversation_input" => {
            if let Some(count) = extract_key_value(detail, "messages=") {
                format!("messages={count}")
            } else {
                "conversation captured".to_string()
            }
        }
        "conversation_fixed" => {
            let messages =
                extract_key_value(detail, "messages=").unwrap_or_else(|| "?".to_string());
            let issues = extract_key_value(detail, "issues=").unwrap_or_else(|| "?".to_string());
            format!("messages={messages}, issues={issues}")
        }
        "tools_ready" => {
            let tools = extract_key_value(detail, "tools=").unwrap_or_else(|| "?".to_string());
            let toolshim =
                extract_key_value(detail, "toolshim_tools=").unwrap_or_else(|| "?".to_string());
            format!("tools={tools}, toolshim_tools={toolshim}")
        }
        "mode" => detail.to_string(),
        _ => "step completed".to_string(),
    }
}

fn extract_key_value(detail: &str, key: &str) -> Option<String> {
    detail
        .split(',')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(key).map(ToString::to_string))
}

fn redact_trace_detail(detail: &str) -> String {
    let with_session_redacted = redact_key_value(detail, "session_id=");
    redact_unix_paths(&with_session_redacted)
}

fn redact_key_value(input: &str, key: &str) -> String {
    // 使用 collect() 和 chars() 更安全地处理
    let chars: Vec<char> = input.chars().collect();
    let key_chars: Vec<char> = key.chars().collect();

    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while cursor < chars.len() {
        // 查找关键字
        let mut matches = false;
        let mut match_end = cursor;

        for (i, &key_ch) in key_chars.iter().enumerate() {
            if cursor + i >= chars.len() {
                break;
            }
            if chars[cursor + i] != key_ch {
                break;
            }
            if i == key_chars.len() - 1 {
                matches = true;
                match_end = cursor + i + 1;
            }
        }

        if matches {
            // 添加之前的内容
            for i in cursor..cursor {
                output.push(chars[i]);
            }
            output.push_str(key);
            output.push_str("<redacted>");

            // 跳过值
            cursor = match_end;
            while cursor < chars.len() {
                let ch = chars[cursor];
                if ch == ',' || ch.is_whitespace() {
                    break;
                }
                cursor += 1;
            }
        } else {
            cursor += 1;
        }
    }

    // 添加剩余字符
    for i in cursor..chars.len() {
        output.push(chars[i]);
    }
    output
}

fn redact_unix_paths(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0usize;

    while index < chars.len() {
        let current = chars[index];
        let prev = if index == 0 {
            None
        } else {
            Some(chars[index - 1])
        };
        let is_path_start = current == '/'
            && prev
                .map(|ch| ch.is_whitespace() || ch == ',' || ch == '=' || ch == ':' || ch == '(')
                .unwrap_or(true);

        if is_path_start {
            let mut end = index + 1;
            while end < chars.len() && !chars[end].is_whitespace() && chars[end] != ',' {
                end += 1;
            }
            output.push_str("<path>");
            index = end;
            continue;
        }

        output.push(current);
        index += 1;
    }

    output
}

async fn get_token_state(session_id: &str) -> TokenState {
    SessionManager::get_session(session_id, false)
        .await
        .map(|session| TokenState {
            input_tokens: session.input_tokens.unwrap_or(0),
            output_tokens: session.output_tokens.unwrap_or(0),
            total_tokens: session.total_tokens.unwrap_or(0),
            accumulated_input_tokens: session.accumulated_input_tokens.unwrap_or(0),
            accumulated_output_tokens: session.accumulated_output_tokens.unwrap_or(0),
            accumulated_total_tokens: session.accumulated_total_tokens.unwrap_or(0),
        })
        .inspect_err(|e| {
            tracing::warn!(
                "Failed to fetch session token state for {}: {}",
                session_id,
                e
            );
        })
        .unwrap_or_default()
}

async fn stream_event(
    event: MessageEvent,
    tx: &mpsc::Sender<String>,
    cancel_token: &CancellationToken,
) {
    let json = serde_json::to_string(&event).unwrap_or_else(|e| {
        format!(
            r#"{{"type":"Error","error":"Failed to serialize event: {}"}}"#,
            e
        )
    });

    if tx.send(format!("data: {}\n\n", json)).await.is_err() {
        tracing::info!("client hung up");
        cancel_token.cancel();
    }
}

#[allow(clippy::too_many_lines)]
#[utoipa::path(
    post,
    path = "/reply",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Streaming response initiated",
         body = MessageEvent,
         content_type = "text/event-stream"),
        (status = 424, description = "Agent not initialized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn reply(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<SseResponse, StatusCode> {
    let session_start = std::time::Instant::now();

    tracing::info!(
        counter.aster.session_starts = 1,
        session_type = "app",
        interface = "ui",
        "Session started"
    );

    let session_id = request.session_id.clone();
    let include_context_trace = request.include_context_trace.unwrap_or(false);
    let context_trace_level = request.context_trace_level.unwrap_or_default();
    let context_trace_redact = request.context_trace_redact.unwrap_or(true);

    if let Some(recipe_name) = request.recipe_name.clone() {
        if state.mark_recipe_run_if_absent(&session_id).await {
            let recipe_version = request
                .recipe_version
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            tracing::info!(
                counter.aster.recipe_runs = 1,
                recipe_name = %recipe_name,
                recipe_version = %recipe_version,
                session_type = "app",
                interface = "ui",
                "Recipe execution started"
            );
        }
    }

    let (tx, rx) = mpsc::channel(100);
    let stream = ReceiverStream::new(rx);
    let cancel_token = CancellationToken::new();

    let user_message = request.user_message;
    let conversation_so_far = request.conversation_so_far;

    let task_cancel = cancel_token.clone();
    let task_tx = tx.clone();

    drop(tokio::spawn(async move {
        let agent = match state.get_agent(session_id.clone()).await {
            Ok(agent) => agent,
            Err(e) => {
                tracing::error!("Failed to get session agent: {}", e);
                let _ = stream_event(
                    MessageEvent::Error {
                        error: format!("Failed to get session agent: {}", e),
                    },
                    &task_tx,
                    &task_cancel,
                )
                .await;
                return;
            }
        };

        let session = match SessionManager::get_session(&session_id, true).await {
            Ok(metadata) => metadata,
            Err(e) => {
                tracing::error!("Failed to read session for {}: {}", session_id, e);
                let _ = stream_event(
                    MessageEvent::Error {
                        error: format!("Failed to read session: {}", e),
                    },
                    &task_tx,
                    &cancel_token,
                )
                .await;
                return;
            }
        };

        let session_config = SessionConfig {
            id: session_id.clone(),
            schedule_id: session.schedule_id.clone(),
            max_turns: None,
            retry_config: None,
            system_prompt: None,
            include_context_trace: Some(include_context_trace),
        };

        let mut all_messages = match conversation_so_far {
            Some(history) => {
                let conv = Conversation::new_unvalidated(history);
                if let Err(e) = SessionManager::replace_conversation(&session_id, &conv).await {
                    tracing::warn!(
                        "Failed to replace session conversation for {}: {}",
                        session_id,
                        e
                    );
                }
                conv
            }
            None => session.conversation.unwrap_or_default(),
        };
        all_messages.push(user_message.clone());

        let mut stream = match agent
            .reply(
                user_message.clone(),
                session_config,
                Some(task_cancel.clone()),
            )
            .await
        {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!("Failed to start reply stream: {:?}", e);
                stream_event(
                    MessageEvent::Error {
                        error: e.to_string(),
                    },
                    &task_tx,
                    &cancel_token,
                )
                .await;
                return;
            }
        };

        let mut heartbeat_interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            tokio::select! {
                _ = task_cancel.cancelled() => {
                    tracing::info!("Agent task cancelled");
                    break;
                }
                _ = heartbeat_interval.tick() => {
                    stream_event(MessageEvent::Ping, &tx, &cancel_token).await;
                }
                response = timeout(Duration::from_millis(500), stream.next()) => {
                    match response {
                        Ok(Some(Ok(AgentEvent::Message(message)))) => {
                            for content in &message.content {
                                track_tool_telemetry(content, all_messages.messages());
                            }

                            all_messages.push(message.clone());

                            let token_state = get_token_state(&session_id).await;

                            stream_event(MessageEvent::Message { message, token_state }, &tx, &cancel_token).await;
                        }
                        Ok(Some(Ok(AgentEvent::HistoryReplaced(new_messages)))) => {
                            all_messages = new_messages.clone();
                            stream_event(MessageEvent::UpdateConversation {conversation: new_messages}, &tx, &cancel_token).await;

                        }
                        Ok(Some(Ok(AgentEvent::ModelChange { model, mode }))) => {
                            stream_event(MessageEvent::ModelChange { model, mode }, &tx, &cancel_token).await;
                        }
                        Ok(Some(Ok(AgentEvent::ContextTrace { steps }))) => {
                            stream_event(
                                MessageEvent::ContextTrace {
                                    steps: transform_context_trace_steps(
                                        steps,
                                        context_trace_level,
                                        context_trace_redact,
                                    ),
                                },
                                &tx,
                                &cancel_token,
                            )
                            .await;
                        }
                        Ok(Some(Ok(AgentEvent::McpNotification((request_id, n))))) => {
                            stream_event(MessageEvent::Notification{
                                request_id: request_id.clone(),
                                message: n,
                            }, &tx, &cancel_token).await;
                        }

                        Ok(Some(Err(e))) => {
                            tracing::error!("Error processing message: {}", e);
                            stream_event(
                                MessageEvent::Error {
                                    error: e.to_string(),
                                },
                                &tx,
                                &cancel_token,
                            ).await;
                            break;
                        }
                        Ok(None) => {
                            break;
                        }
                        Err(_) => {
                            if tx.is_closed() {
                                break;
                            }
                            continue;
                        }
                    }
                }
            }
        }

        let session_duration = session_start.elapsed();

        if let Ok(session) = SessionManager::get_session(&session_id, true).await {
            let total_tokens = session.total_tokens.unwrap_or(0);
            tracing::info!(
                counter.aster.session_completions = 1,
                session_type = "app",
                interface = "ui",
                exit_type = "normal",
                duration_ms = session_duration.as_millis() as u64,
                total_tokens = total_tokens,
                message_count = session.message_count,
                "Session completed"
            );

            tracing::info!(
                counter.aster.session_duration_ms = session_duration.as_millis() as u64,
                session_type = "app",
                interface = "ui",
                "Session duration"
            );

            if total_tokens > 0 {
                tracing::info!(
                    counter.aster.session_tokens = total_tokens,
                    session_type = "app",
                    interface = "ui",
                    "Session tokens"
                );
            }
        } else {
            tracing::info!(
                counter.aster.session_completions = 1,
                session_type = "app",
                interface = "ui",
                exit_type = "normal",
                duration_ms = session_duration.as_millis() as u64,
                total_tokens = 0u64,
                message_count = all_messages.len(),
                "Session completed"
            );

            tracing::info!(
                counter.aster.session_duration_ms = session_duration.as_millis() as u64,
                session_type = "app",
                interface = "ui",
                "Session duration"
            );
        }

        let final_token_state = get_token_state(&session_id).await;

        let _ = stream_event(
            MessageEvent::Finish {
                reason: "stop".to_string(),
                token_state: final_token_state,
            },
            &task_tx,
            &cancel_token,
        )
        .await;
    }));
    Ok(SseResponse::new(stream))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/reply",
            post(reply).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_context_trace_basic_and_redacted() {
        let steps = vec![
            ContextTraceStep {
                stage: "session".to_string(),
                detail: "session_id=test-session-123".to_string(),
            },
            ContextTraceStep {
                stage: "tools_ready".to_string(),
                detail: "tools=4, toolshim_tools=1, system_prompt_chars=1200".to_string(),
            },
        ];

        let transformed = transform_context_trace_steps(steps, ContextTraceLevel::Basic, true);

        assert_eq!(transformed.len(), 2);
        assert_eq!(transformed[0].detail, "session initialized");
        assert_eq!(transformed[1].detail, "tools=4, toolshim_tools=1");
    }

    #[test]
    fn test_transform_context_trace_verbose_redacts_sensitive_values() {
        let steps = vec![ContextTraceStep {
            stage: "session".to_string(),
            detail:
                "session_id=test-session-123, cwd=/Users/coso/Documents/dev/ai/astercloud/aster-rust"
                    .to_string(),
        }];

        let transformed = transform_context_trace_steps(steps, ContextTraceLevel::Verbose, true);

        assert_eq!(transformed.len(), 1);
        assert!(transformed[0].detail.contains("session_id=<redacted>"));
        assert!(transformed[0].detail.contains("cwd=<path>"));
    }

    mod integration_tests {
        use super::*;
        use aster::conversation::message::Message;
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        #[tokio::test(flavor = "multi_thread")]
        async fn test_reply_endpoint() {
            let state = AppState::new().await.unwrap();

            let app = routes(state);

            let request = Request::builder()
                .uri("/reply")
                .method("POST")
                .header("content-type", "application/json")
                .header("x-secret-key", "test-secret")
                .body(Body::from(
                    serde_json::to_string(&ChatRequest {
                        user_message: Message::user().with_text("test message"),
                        conversation_so_far: None,
                        session_id: "test-session".to_string(),
                        recipe_name: None,
                        recipe_version: None,
                        include_context_trace: None,
                        context_trace_level: None,
                        context_trace_redact: None,
                    })
                    .unwrap(),
                ))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}
