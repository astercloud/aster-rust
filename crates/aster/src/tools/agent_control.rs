//! Agent control tools.
//!
//! 提供现代 delegation / subagent 运行时的通用工具抽象，允许宿主通过
//! callback 注入真正的 agent runtime，实现框架级的 spawn/send/wait/resume/close。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::tools::base::Tool;
use crate::tools::context::{ToolContext, ToolOptions, ToolResult};
use crate::tools::error::ToolError;
use crate::tools::registry::ToolRegistry;

type CallbackFuture<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;

pub type SpawnAgentCallback =
    Arc<dyn Fn(SpawnAgentRequest) -> CallbackFuture<SpawnAgentResponse> + Send + Sync>;
pub type SendInputCallback =
    Arc<dyn Fn(SendInputRequest) -> CallbackFuture<SendInputResponse> + Send + Sync>;
pub type WaitAgentCallback =
    Arc<dyn Fn(WaitAgentRequest) -> CallbackFuture<WaitAgentResponse> + Send + Sync>;
pub type ResumeAgentCallback =
    Arc<dyn Fn(ResumeAgentRequest) -> CallbackFuture<ResumeAgentResponse> + Send + Sync>;
pub type CloseAgentCallback =
    Arc<dyn Fn(CloseAgentRequest) -> CallbackFuture<CloseAgentResponse> + Send + Sync>;

#[derive(Clone, Default)]
pub struct AgentControlToolConfig {
    pub spawn_agent: Option<SpawnAgentCallback>,
    pub send_input: Option<SendInputCallback>,
    pub wait_agent: Option<WaitAgentCallback>,
    pub resume_agent: Option<ResumeAgentCallback>,
    pub close_agent: Option<CloseAgentCallback>,
}

impl AgentControlToolConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_spawn_agent_callback(mut self, callback: SpawnAgentCallback) -> Self {
        self.spawn_agent = Some(callback);
        self
    }

    pub fn with_send_input_callback(mut self, callback: SendInputCallback) -> Self {
        self.send_input = Some(callback);
        self
    }

    pub fn with_wait_agent_callback(mut self, callback: WaitAgentCallback) -> Self {
        self.wait_agent = Some(callback);
        self
    }

    pub fn with_resume_agent_callback(mut self, callback: ResumeAgentCallback) -> Self {
        self.resume_agent = Some(callback);
        self
    }

    pub fn with_close_agent_callback(mut self, callback: CloseAgentCallback) -> Self {
        self.close_agent = Some(callback);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.spawn_agent.is_none()
            && self.send_input.is_none()
            && self.wait_agent.is_none()
            && self.resume_agent.is_none()
            && self.close_agent.is_none()
    }
}

pub fn register_agent_control_tools(registry: &mut ToolRegistry, config: &AgentControlToolConfig) {
    if let Some(callback) = config.spawn_agent.clone() {
        registry.register(Box::new(SpawnAgentTool::new(callback)));
    }

    if let Some(callback) = config.send_input.clone() {
        registry.register(Box::new(SendInputTool::new(callback)));
    }

    if let Some(callback) = config.wait_agent.clone() {
        registry.register(Box::new(WaitAgentTool::new(callback)));
    }

    if let Some(callback) = config.resume_agent.clone() {
        registry.register(Box::new(ResumeAgentTool::new(callback)));
    }

    if let Some(callback) = config.close_agent.clone() {
        registry.register(Box::new(CloseAgentTool::new(callback)));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpawnAgentRequest {
    pub parent_session_id: String,
    pub message: String,
    #[serde(alias = "agent_type")]
    pub agent_type: Option<String>,
    pub model: Option<String>,
    #[serde(alias = "reasoning_effort")]
    pub reasoning_effort: Option<String>,
    #[serde(alias = "fork_context")]
    pub fork_context: bool,
    #[serde(alias = "blueprint_role_id")]
    pub blueprint_role_id: Option<String>,
    #[serde(alias = "blueprint_role_label")]
    pub blueprint_role_label: Option<String>,
    #[serde(alias = "profile_id")]
    pub profile_id: Option<String>,
    #[serde(alias = "profile_name")]
    pub profile_name: Option<String>,
    #[serde(alias = "role_key")]
    pub role_key: Option<String>,
    #[serde(default, alias = "skill_ids")]
    pub skill_ids: Vec<String>,
    #[serde(default, alias = "skill_directories")]
    pub skill_directories: Vec<String>,
    #[serde(alias = "team_preset_id")]
    pub team_preset_id: Option<String>,
    pub theme: Option<String>,
    #[serde(alias = "system_overlay")]
    pub system_overlay: Option<String>,
    #[serde(alias = "output_contract")]
    pub output_contract: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpawnAgentResponse {
    pub agent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SendInputRequest {
    pub id: String,
    pub message: String,
    #[serde(default)]
    pub interrupt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendInputResponse {
    pub submission_id: String,
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WaitAgentRequest {
    pub ids: Vec<String>,
    #[serde(default, alias = "timeout_ms")]
    pub timeout_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WaitAgentResponse {
    #[serde(default)]
    pub timed_out: bool,
    #[serde(default)]
    pub status: BTreeMap<String, Value>,
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResumeAgentRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResumeAgentResponse {
    #[serde(default)]
    pub changed_session_ids: Vec<String>,
    #[serde(default)]
    pub status: Value,
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CloseAgentRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloseAgentResponse {
    #[serde(default)]
    pub changed_session_ids: Vec<String>,
    #[serde(default)]
    pub previous_status: Value,
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

fn normalize_required_text(value: &str, field_name: &str) -> Result<String, ToolError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ToolError::invalid_params(format!("{field_name} 不能为空")));
    }

    Ok(trimmed.to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpawnAgentToolInput {
    message: String,
    #[serde(alias = "agent_type")]
    agent_type: Option<String>,
    model: Option<String>,
    #[serde(alias = "reasoning_effort")]
    reasoning_effort: Option<String>,
    #[serde(default, alias = "fork_context")]
    fork_context: bool,
    #[serde(alias = "blueprint_role_id")]
    blueprint_role_id: Option<String>,
    #[serde(alias = "blueprint_role_label")]
    blueprint_role_label: Option<String>,
    #[serde(alias = "profile_id")]
    profile_id: Option<String>,
    #[serde(alias = "profile_name")]
    profile_name: Option<String>,
    #[serde(alias = "role_key")]
    role_key: Option<String>,
    #[serde(default, alias = "skill_ids")]
    skill_ids: Vec<String>,
    #[serde(default, alias = "skill_directories")]
    skill_directories: Vec<String>,
    #[serde(alias = "team_preset_id")]
    team_preset_id: Option<String>,
    theme: Option<String>,
    #[serde(alias = "system_overlay")]
    system_overlay: Option<String>,
    #[serde(alias = "output_contract")]
    output_contract: Option<String>,
}

#[derive(Clone)]
pub struct SpawnAgentTool {
    callback: SpawnAgentCallback,
}

impl SpawnAgentTool {
    pub fn new(callback: SpawnAgentCallback) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl Tool for SpawnAgentTool {
    fn name(&self) -> &str {
        "spawn_agent"
    }

    fn description(&self) -> &str {
        "仅在任务需要拆成多个独立子范围、并行评审/验证，或用户明确要求多代理时使用。先判断当前关键路径：如果下一步立即依赖结果，不要把阻塞工作委派出去；优先把可并行推进的 sidecar 子任务交给子代理，同时主线程继续做不重叠的工作。创建真实子代理会话，并异步开始执行首条任务。不要对简单任务创建子代理；多个子代理必须分工明确，避免修改同一片文件；当前 team runtime 默认不允许子代理继续创建新的子代理。"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string", "description": "发送给子代理的首条任务消息。应是边界清晰、可独立完成、不会与其他并发子代理写入范围重叠的子任务。" },
                "agentType": { "type": "string", "description": "子代理角色提示，例如 explorer/planner/executor，也可以是 Image #1 这类展示标签" },
                "model": { "type": "string", "description": "可选模型覆盖" },
                "reasoningEffort": { "type": "string", "description": "保留字段，当前仅记录到 metadata" },
                "forkContext": { "type": "boolean", "description": "保留字段，当前仅记录到 metadata" },
                "blueprintRoleId": { "type": "string", "description": "可选当前 Team 蓝图角色 id；当 GUI 已提前准备协作分工时，优先传入对应角色 id，便于真实成员接管画布泳道" },
                "blueprintRoleLabel": { "type": "string", "description": "可选当前 Team 蓝图角色标签，例如 分析 / 执行 / 验证" },
                "profileId": { "type": "string", "description": "可选内置 profile id，例如 code-explorer / code-executor / code-verifier" },
                "profileName": { "type": "string", "description": "可选 profile 展示名称，用于 Team Workspace 与子代理 prompt" },
                "roleKey": { "type": "string", "description": "可选角色键，例如 explorer / executor / verifier / researcher" },
                "skillIds": { "type": "array", "items": { "type": "string" }, "description": "可选 builtin skill id 列表，用于附加子代理技能提示" },
                "skillDirectories": { "type": "array", "items": { "type": "string" }, "description": "可选本地已安装 skill 目录名；会读取对应 SKILL.md 注入子代理 prompt" },
                "teamPresetId": { "type": "string", "description": "可选 team preset id，例如 code-triage-team / research-team / content-creation-team" },
                "theme": { "type": "string", "description": "可选子代理主题标签，用于 GUI 展示与 prompt 约束" },
                "systemOverlay": { "type": "string", "description": "附加给该子代理的额外系统约束" },
                "outputContract": { "type": "string", "description": "要求子代理遵循的输出契约" }
            },
            "required": ["message"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> Result<ToolResult, ToolError> {
        let input: SpawnAgentToolInput = serde_json::from_value(params)
            .map_err(|error| ToolError::invalid_params(format!("spawn_agent 参数无效: {error}")))?;
        let request = SpawnAgentRequest {
            parent_session_id: normalize_required_text(&context.session_id, "session_id")?,
            message: normalize_required_text(&input.message, "message")?,
            agent_type: input.agent_type,
            model: input.model,
            reasoning_effort: input.reasoning_effort,
            fork_context: input.fork_context,
            blueprint_role_id: input.blueprint_role_id,
            blueprint_role_label: input.blueprint_role_label,
            profile_id: input.profile_id,
            profile_name: input.profile_name,
            role_key: input.role_key,
            skill_ids: input.skill_ids,
            skill_directories: input.skill_directories,
            team_preset_id: input.team_preset_id,
            theme: input.theme,
            system_overlay: input.system_overlay,
            output_contract: input.output_contract,
        };
        let response = (self.callback)(request)
            .await
            .map_err(ToolError::execution_failed)?;

        Ok(
            ToolResult::success(format!("子代理已创建: {}", response.agent_id)).with_metadata(
                "spawn_agent",
                serde_json::to_value(&response).unwrap_or_default(),
            ),
        )
    }
}

#[derive(Clone)]
pub struct SendInputTool {
    callback: SendInputCallback,
}

impl SendInputTool {
    pub fn new(callback: SendInputCallback) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl Tool for SendInputTool {
    fn name(&self) -> &str {
        "send_input"
    }

    fn description(&self) -> &str {
        "向已存在的子代理追加输入。对强依赖既有上下文的后续任务，优先复用已有子代理而不是重复 spawn；interrupt=true 时会先中断当前执行并清空旧队列。"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "子代理 session id" },
                "message": { "type": "string", "description": "要发送给子代理的输入" },
                "interrupt": { "type": "boolean", "description": "是否先中断当前执行" }
            },
            "required": ["id", "message"],
            "additionalProperties": false
        })
    }

    async fn execute(
        &self,
        params: Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let request: SendInputRequest = serde_json::from_value(params)
            .map_err(|error| ToolError::invalid_params(format!("send_input 参数无效: {error}")))?;
        let response = (self.callback)(request)
            .await
            .map_err(ToolError::execution_failed)?;

        Ok(
            ToolResult::success(format!("子代理输入已提交: {}", response.submission_id))
                .with_metadata(
                    "send_input",
                    serde_json::to_value(&response).unwrap_or_default(),
                ),
        )
    }
}

#[derive(Clone)]
pub struct WaitAgentTool {
    callback: WaitAgentCallback,
}

impl WaitAgentTool {
    pub fn new(callback: WaitAgentCallback) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl Tool for WaitAgentTool {
    fn name(&self) -> &str {
        "wait_agent"
    }

    fn description(&self) -> &str {
        "等待一个或多个子代理进入最终状态。只有在主线程确实被结果阻塞、下一步必须依赖这些结果时才调用；可以同时等待多个 id，任一子代理先完成就会返回。不要反复机械 wait，优先在等待前继续做不重叠的本地工作；timeout_ms 应与任务规模匹配，避免过短轮询。"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "ids": { "type": "array", "items": { "type": "string" }, "description": "要等待的子代理 session id 列表" },
                "timeoutMs": { "type": "integer", "minimum": 1, "description": "最长等待时间（毫秒）" }
            },
            "required": ["ids"],
            "additionalProperties": false
        })
    }

    fn options(&self) -> ToolOptions {
        ToolOptions::new()
            .with_max_retries(0)
            .with_base_timeout(Duration::from_secs(310))
            .with_dynamic_timeout(false)
    }

    async fn execute(
        &self,
        params: Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let request: WaitAgentRequest = serde_json::from_value(params)
            .map_err(|error| ToolError::invalid_params(format!("wait_agent 参数无效: {error}")))?;
        let response = (self.callback)(request)
            .await
            .map_err(ToolError::execution_failed)?;
        let summary = if response.timed_out {
            "wait_agent 超时，未观测到最终状态".to_string()
        } else {
            format!("已观测到 {} 个子代理进入最终状态", response.status.len())
        };

        Ok(ToolResult::success(summary).with_metadata(
            "wait_agent",
            serde_json::to_value(&response).unwrap_or_default(),
        ))
    }
}

#[derive(Clone)]
pub struct ResumeAgentTool {
    callback: ResumeAgentCallback,
}

impl ResumeAgentTool {
    pub fn new(callback: ResumeAgentCallback) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl Tool for ResumeAgentTool {
    fn name(&self) -> &str {
        "resume_agent"
    }

    fn description(&self) -> &str {
        "恢复之前关闭的子代理；若子代理未关闭则返回当前状态"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "子代理 session id" }
            },
            "required": ["id"],
            "additionalProperties": false
        })
    }

    async fn execute(
        &self,
        params: Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let request: ResumeAgentRequest = serde_json::from_value(params).map_err(|error| {
            ToolError::invalid_params(format!("resume_agent 参数无效: {error}"))
        })?;
        let response = (self.callback)(request)
            .await
            .map_err(ToolError::execution_failed)?;
        let changed_count = response.changed_session_ids.len();
        let success_message = if changed_count > 1 {
            format!("子代理已恢复，并级联恢复 {changed_count} 个会话")
        } else if changed_count == 1 {
            "子代理已恢复".to_string()
        } else {
            format!("子代理当前状态: {}", response.status)
        };

        Ok(ToolResult::success(success_message).with_metadata(
            "resume_agent",
            serde_json::to_value(&response).unwrap_or_default(),
        ))
    }
}

#[derive(Clone)]
pub struct CloseAgentTool {
    callback: CloseAgentCallback,
}

impl CloseAgentTool {
    pub fn new(callback: CloseAgentCallback) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl Tool for CloseAgentTool {
    fn name(&self) -> &str {
        "close_agent"
    }

    fn description(&self) -> &str {
        "关闭子代理并级联关闭其子树；历史保留，可后续恢复"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "子代理 session id" }
            },
            "required": ["id"],
            "additionalProperties": false
        })
    }

    async fn execute(
        &self,
        params: Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let request: CloseAgentRequest = serde_json::from_value(params)
            .map_err(|error| ToolError::invalid_params(format!("close_agent 参数无效: {error}")))?;
        let response = (self.callback)(request)
            .await
            .map_err(ToolError::execution_failed)?;
        let changed_count = response.changed_session_ids.len();
        let success_message = if changed_count > 1 {
            format!(
                "子代理已关闭，并级联关闭 {changed_count} 个会话；关闭前状态: {}",
                response.previous_status
            )
        } else {
            format!("子代理已关闭，关闭前状态: {}", response.previous_status)
        };

        Ok(ToolResult::success(success_message).with_metadata(
            "close_agent",
            serde_json::to_value(&response).unwrap_or_default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Tool;
    use std::path::PathBuf;
    use std::sync::Mutex;

    fn create_test_context() -> ToolContext {
        ToolContext::new(PathBuf::from("/tmp")).with_session_id("parent-session")
    }

    #[test]
    fn test_register_agent_control_tools_registers_only_configured_callbacks() {
        let mut registry = ToolRegistry::new();
        let config = AgentControlToolConfig::new().with_spawn_agent_callback(Arc::new(|request| {
            Box::pin(async move {
                Ok(SpawnAgentResponse {
                    agent_id: request.parent_session_id,
                    nickname: None,
                    extra: BTreeMap::new(),
                })
            })
        }));

        register_agent_control_tools(&mut registry, &config);

        assert!(registry.contains("spawn_agent"));
        assert!(!registry.contains("send_input"));
        assert!(!registry.contains("wait_agent"));
    }

    #[tokio::test]
    async fn test_spawn_agent_tool_accepts_snake_case_aliases() {
        let captured = Arc::new(Mutex::new(None::<SpawnAgentRequest>));
        let captured_clone = captured.clone();
        let tool = SpawnAgentTool::new(Arc::new(move |request| {
            *captured_clone.lock().unwrap() = Some(request.clone());
            Box::pin(async move {
                Ok(SpawnAgentResponse {
                    agent_id: "agent-1".to_string(),
                    nickname: Some("Explorer".to_string()),
                    extra: BTreeMap::new(),
                })
            })
        }));

        let result = tool
            .execute(
                serde_json::json!({
                    "message": "检查变更",
                    "agent_type": "explorer",
                    "reasoning_effort": "high",
                    "fork_context": true,
                    "skill_ids": ["skill-a"]
                }),
                &create_test_context(),
            )
            .await
            .unwrap();

        let request = captured.lock().unwrap().clone().unwrap();
        assert_eq!(request.parent_session_id, "parent-session");
        assert_eq!(request.agent_type.as_deref(), Some("explorer"));
        assert_eq!(request.reasoning_effort.as_deref(), Some("high"));
        assert!(request.fork_context);
        assert_eq!(request.skill_ids, vec!["skill-a"]);
        assert_eq!(result.metadata["spawn_agent"]["agent_id"], "agent-1");
    }

    #[tokio::test]
    async fn test_wait_agent_tool_uses_timeout_alias_and_metadata() {
        let tool = WaitAgentTool::new(Arc::new(|request| {
            Box::pin(async move {
                assert_eq!(request.timeout_ms, Some(1200));
                let mut status = BTreeMap::new();
                status.insert(
                    "agent-1".to_string(),
                    serde_json::json!({"kind": "completed"}),
                );
                Ok(WaitAgentResponse {
                    timed_out: false,
                    status,
                    extra: BTreeMap::new(),
                })
            })
        }));

        let result = tool
            .execute(
                serde_json::json!({
                    "ids": ["agent-1"],
                    "timeout_ms": 1200
                }),
                &create_test_context(),
            )
            .await
            .unwrap();

        assert_eq!(
            result.output.as_deref(),
            Some("已观测到 1 个子代理进入最终状态")
        );
        assert_eq!(
            result.metadata["wait_agent"]["status"]["agent-1"]["kind"],
            "completed"
        );
    }
}
