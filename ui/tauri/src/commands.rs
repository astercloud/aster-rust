//! Tauri 命令定义
//!
//! 提供前端调用的 Tauri 命令

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use aster::agents::extension::PLATFORM_EXTENSIONS;
use aster::config::extensions::{
    get_all_extensions, name_to_key, remove_extension, set_extension, set_extension_enabled,
    ExtensionEntry,
};
use aster::conversation::message::{Message as ConversationMessage, MessageContent};
use aster::conversation::Conversation;
use aster::session::{SessionManager, SessionType};
use chrono::{TimeZone, Utc};
use rmcp::model::Role;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::process::Command;
use tokio::time::sleep;
use uuid::Uuid;

use crate::state::{AppState, ServerStatus};

const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_SERVER_SECRET_KEY: &str = "test";
const SERVER_READY_RETRIES: usize = 30;
const SERVER_READY_DELAY_MS: u64 = 300;

/// 配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub working_dir: String,
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// Provider 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub models: Vec<String>,
}

/// 扩展信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

fn session_to_info(session: aster::session::Session) -> SessionInfo {
    SessionInfo {
        id: session.id,
        name: session.name,
        created_at: session.created_at.to_rfc3339(),
        working_dir: session.working_dir.to_string_lossy().to_string(),
    }
}

fn message_content_to_text(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(text) => text.text.clone(),
        other => other.to_string(),
    }
}

fn conversation_message_to_ui(message: &ConversationMessage) -> Message {
    let timestamp = Utc
        .timestamp_opt(message.created, 0)
        .single()
        .unwrap_or_else(Utc::now)
        .to_rfc3339();

    let role = match message.role {
        Role::User => "user",
        _ => "assistant",
    };

    Message {
        id: message
            .id
            .clone()
            .unwrap_or_else(|| format!("msg-{}", message.created)),
        role: role.to_string(),
        content: message
            .content
            .iter()
            .map(message_content_to_text)
            .collect::<Vec<_>>()
            .join("\n"),
        timestamp,
    }
}

fn extension_to_info(entry: ExtensionEntry) -> ExtensionInfo {
    let name = entry.config.name();
    let version = match &entry.config {
        aster::agents::ExtensionConfig::Platform { .. }
        | aster::agents::ExtensionConfig::Builtin { .. } => "builtin".to_string(),
        _ => "custom".to_string(),
    };

    ExtensionInfo {
        name,
        version,
        enabled: entry.enabled,
    }
}

fn build_platform_extension(name: &str) -> Result<ExtensionEntry, String> {
    let requested_key = name_to_key(name);
    let definition = PLATFORM_EXTENSIONS
        .values()
        .find(|item| item.name == name || name_to_key(item.name) == requested_key)
        .ok_or_else(|| format!("未找到内置扩展: {name}"))?;

    Ok(ExtensionEntry {
        enabled: true,
        config: aster::agents::ExtensionConfig::Platform {
            name: definition.name.to_string(),
            description: definition.description.to_string(),
            bundled: Some(true),
            available_tools: Vec::new(),
            deferred_loading: false,
            always_expose_tools: Vec::new(),
            allowed_caller: None,
        },
    })
}

fn resolve_repo_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("解析仓库根目录失败: {error}"))
}

async fn is_server_healthy(port: u16) -> bool {
    let url = format!("http://{DEFAULT_SERVER_HOST}:{port}/status");
    reqwest::Client::new()
        .get(url)
        .timeout(Duration::from_millis(700))
        .send()
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

fn build_server_command(port: u16) -> Result<Command, String> {
    let repo_root = resolve_repo_root()?;
    let server_bin = std::env::var("ASTER_SERVER_BIN").ok();

    let mut command = if let Some(bin) = server_bin {
        let mut command = Command::new(bin);
        command.arg("agent");
        command
    } else {
        let mut command = Command::new("cargo");
        command
            .arg("run")
            .arg("--manifest-path")
            .arg(repo_root.join("Cargo.toml"))
            .arg("-p")
            .arg("aster-server")
            .arg("--")
            .arg("agent");
        command
    };

    command
        .current_dir(repo_root)
        .env("ASTER_PORT", port.to_string())
        .env("ASTER_SERVER__SECRET_KEY", DEFAULT_SERVER_SECRET_KEY)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    Ok(command)
}

fn server_status_label(status: &ServerStatus) -> String {
    match status {
        ServerStatus::Stopped => "Stopped".to_string(),
        ServerStatus::Starting => "Starting".to_string(),
        ServerStatus::Running => "Running".to_string(),
        ServerStatus::Error(message) => format!("Error: {message}"),
    }
}

async fn wait_for_server_ready(child: &mut tokio::process::Child, port: u16) -> Result<(), String> {
    for _ in 0..SERVER_READY_RETRIES {
        if is_server_healthy(port).await {
            return Ok(());
        }

        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("检查 asterd 进程状态失败: {error}"))?
        {
            return Err(format!("asterd 进程提前退出: {status}"));
        }

        sleep(Duration::from_millis(SERVER_READY_DELAY_MS)).await;
    }

    Err("等待 asterd 启动超时".to_string())
}

// ============================================================================
// 配置命令
// ============================================================================

#[tauri::command]
pub async fn get_config(_key: String) -> Result<serde_json::Value, String> {
    // TODO: 调用 aster 核心库获取配置
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn set_config(_key: String, _value: serde_json::Value) -> Result<(), String> {
    // TODO: 调用 aster 核心库设置配置
    Ok(())
}

// ============================================================================
// 会话命令
// ============================================================================

#[tauri::command]
pub async fn start_session(
    state: State<'_, AppState>,
    name: String,
    working_dir: String,
) -> Result<SessionInfo, String> {
    let session =
        SessionManager::create_session(PathBuf::from(&working_dir), name, SessionType::User)
            .await
            .map_err(|error| format!("创建会话失败: {error}"))?;

    {
        let mut current_session = state.current_session.write().await;
        *current_session = Some(session.id.clone());
    }

    Ok(session_to_info(session))
}

#[tauri::command]
pub async fn stop_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    SessionManager::delete_session(&session_id)
        .await
        .map_err(|error| format!("停止会话失败: {error}"))?;

    let mut current_session = state.current_session.write().await;
    if current_session.as_deref() == Some(session_id.as_str()) {
        *current_session = None;
    }
    Ok(())
}

#[tauri::command]
pub async fn send_message(session_id: String, content: String) -> Result<Message, String> {
    let session = SessionManager::get_session(&session_id, true)
        .await
        .map_err(|error| format!("获取会话失败: {error}"))?;

    let mut conversation = session.conversation.unwrap_or_else(Conversation::empty);
    let user_message = ConversationMessage::user()
        .with_id(Uuid::new_v4().to_string())
        .with_text(content.clone());
    conversation.push(user_message.clone());

    SessionManager::replace_conversation(&session_id, &conversation)
        .await
        .map_err(|error| format!("写入消息失败: {error}"))?;

    Ok(conversation_message_to_ui(&user_message))
}

#[tauri::command]
pub async fn get_sessions() -> Result<Vec<SessionInfo>, String> {
    let mut sessions = SessionManager::list_sessions()
        .await
        .map_err(|error| format!("获取会话列表失败: {error}"))?;

    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    Ok(sessions
        .into_iter()
        .filter(|session| session.session_type != SessionType::Hidden)
        .map(session_to_info)
        .collect())
}

#[tauri::command]
pub async fn get_session_messages(session_id: String) -> Result<Vec<Message>, String> {
    let session = SessionManager::get_session(&session_id, true)
        .await
        .map_err(|error| format!("获取会话消息失败: {error}"))?;

    Ok(session
        .conversation
        .map(|conversation| {
            conversation
                .user_visible_messages()
                .iter()
                .map(conversation_message_to_ui)
                .collect()
        })
        .unwrap_or_default())
}

// ============================================================================
// Provider 命令
// ============================================================================

#[tauri::command]
pub async fn get_providers() -> Result<Vec<ProviderInfo>, String> {
    // TODO: 调用 aster 核心库获取 Provider 列表
    Ok(vec![
        ProviderInfo {
            name: "anthropic".to_string(),
            display_name: "Anthropic".to_string(),
            models: vec!["claude-sonnet-4-20250514".to_string()],
        },
        ProviderInfo {
            name: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            models: vec!["gpt-4o".to_string()],
        },
    ])
}

// ============================================================================
// 扩展命令
// ============================================================================

#[tauri::command]
pub async fn get_extensions() -> Result<Vec<ExtensionInfo>, String> {
    let mut extensions = get_all_extensions();
    extensions.sort_by(|left, right| left.config.name().cmp(&right.config.name()));

    Ok(extensions.into_iter().map(extension_to_info).collect())
}

#[tauri::command]
pub async fn install_extension(name: String) -> Result<ExtensionInfo, String> {
    let entry = build_platform_extension(&name)?;
    let info = extension_to_info(entry.clone());
    set_extension(entry);
    Ok(info)
}

#[tauri::command]
pub async fn uninstall_extension(name: String) -> Result<(), String> {
    let key = name_to_key(&name);
    let installed = get_all_extensions()
        .into_iter()
        .find(|entry| entry.config.key() == key || entry.config.name() == name);

    let entry = installed.ok_or_else(|| format!("扩展不存在: {name}"))?;

    let key = entry.config.key();
    let should_disable = matches!(
        &entry.config,
        aster::agents::ExtensionConfig::Platform { .. }
            | aster::agents::ExtensionConfig::Builtin { .. }
    );

    if should_disable {
        set_extension_enabled(&key, false);
    } else {
        remove_extension(&key);
    }

    Ok(())
}

// ============================================================================
// 服务器命令
// ============================================================================

#[tauri::command]
pub async fn get_server_status(state: State<'_, AppState>) -> Result<String, String> {
    let port = *state.server_port.read().await;

    if is_server_healthy(port).await {
        let mut status = state.server_status.write().await;
        *status = ServerStatus::Running;
        return Ok(server_status_label(&status));
    }

    {
        let mut process_guard = state.server_process.lock().await;
        if let Some(child) = process_guard.as_mut() {
            if child
                .try_wait()
                .map_err(|error| format!("检查 asterd 进程失败: {error}"))?
                .is_some()
            {
                *process_guard = None;
            }
        }
    }

    let mut status = state.server_status.write().await;
    *status = ServerStatus::Stopped;
    Ok(server_status_label(&status))
}

#[tauri::command]
pub async fn start_server(state: State<'_, AppState>, port: Option<u16>) -> Result<(), String> {
    let port = port.unwrap_or(3000);

    if is_server_healthy(port).await {
        let mut status = state.server_status.write().await;
        *status = ServerStatus::Running;
        let mut server_port = state.server_port.write().await;
        *server_port = port;
        return Ok(());
    }

    {
        let mut status = state.server_status.write().await;
        *status = ServerStatus::Starting;
    }

    let mut command = build_server_command(port)?;
    let mut child = command
        .spawn()
        .map_err(|error| format!("启动 asterd 失败: {error}"))?;

    if let Err(error) = wait_for_server_ready(&mut child, port).await {
        let _ = child.kill().await;
        let mut status = state.server_status.write().await;
        *status = ServerStatus::Error(error.clone());
        return Err(error);
    }

    {
        let mut process = state.server_process.lock().await;
        *process = Some(child);
    }

    {
        let mut status = state.server_status.write().await;
        *status = ServerStatus::Running;
        let mut server_port = state.server_port.write().await;
        *server_port = port;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut process_guard = state.server_process.lock().await;
        if let Some(child) = process_guard.as_mut() {
            child
                .kill()
                .await
                .map_err(|error| format!("停止 asterd 失败: {error}"))?;
        }
        *process_guard = None;
    }

    let mut status = state.server_status.write().await;
    *status = ServerStatus::Stopped;

    Ok(())
}
