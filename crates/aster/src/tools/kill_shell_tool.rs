//! Kill Shell Tool Implementation
//!
//! 此模块实现了 `KillShellTool`，用于终止正在运行的后台任务：
//! - 支持通过 task_id 终止特定任务
//! - 与 TaskManager 集成
//! - 提供安全的任务终止机制
//! - 兼容 Claude Agent SDK 的 KillShellTool 接口
//!
//! Requirements: 基于 Claude Agent SDK bash.ts 中的 KillShellTool 实现

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::base::{PermissionCheckResult, Tool};
use super::context::{ToolContext, ToolOptions, ToolResult};
use super::error::ToolError;
use super::task::TaskManager;

/// KillShell 工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillShellInput {
    /// 要终止的任务 ID（支持 shell_id 和 task_id 两种格式）
    #[serde(alias = "task_id")]
    pub shell_id: String,
}

/// Kill Shell Tool for terminating background tasks
///
/// 提供安全的后台任务终止功能：
/// - 通过 task_id 或 shell_id 终止任务
/// - 与现有 TaskManager 集成
/// - 支持向后兼容的参数名称
/// - 提供详细的终止状态反馈
#[derive(Debug)]
pub struct KillShellTool {
    /// Task manager for background task management
    task_manager: Arc<TaskManager>,
}

impl Default for KillShellTool {
    fn default() -> Self {
        Self::new()
    }
}

impl KillShellTool {
    /// Create a new KillShellTool with default TaskManager
    pub fn new() -> Self {
        Self {
            task_manager: Arc::new(TaskManager::new()),
        }
    }

    /// Create a KillShellTool with custom TaskManager
    pub fn with_task_manager(task_manager: Arc<TaskManager>) -> Self {
        Self { task_manager }
    }

    /// Get the task manager
    pub fn task_manager(&self) -> &Arc<TaskManager> {
        &self.task_manager
    }
}

#[async_trait]
impl Tool for KillShellTool {
    /// Returns the tool name
    fn name(&self) -> &str {
        "KillShell"
    }

    /// Returns the tool description
    fn description(&self) -> &str {
        "Kills a running background bash shell by its ID. \
         Takes a shell_id parameter identifying the shell to kill. \
         Returns a success or failure status. \
         Use this tool when you need to terminate a long-running shell. \
         Shell IDs can be found using the TaskOutput tool or from background task execution results."
    }

    /// Returns the JSON Schema for input parameters
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "shell_id": {
                    "type": "string",
                    "description": "The ID of the background shell/task to kill"
                }
            },
            "required": ["shell_id"]
        })
    }

    /// Execute the kill shell command
    async fn execute(
        &self,
        params: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        // Extract shell_id parameter (also accept task_id for compatibility)
        let shell_id = params
            .get("shell_id")
            .or_else(|| params.get("task_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("Missing required parameter: shell_id"))?;

        // Attempt to kill the task
        match self.task_manager.kill(shell_id).await {
            Ok(()) => {
                let success_message = format!("Successfully killed shell: {}", shell_id);
                Ok(ToolResult::success(success_message)
                    .with_metadata("shell_id", serde_json::json!(shell_id))
                    .with_metadata("killed", serde_json::json!(true)))
            }
            Err(ToolError::NotFound(_)) => {
                let error_message = format!("No shell found with ID: {}", shell_id);
                Ok(ToolResult::error(error_message)
                    .with_metadata("shell_id", serde_json::json!(shell_id))
                    .with_metadata("killed", serde_json::json!(false)))
            }
            Err(e) => {
                let error_message = format!("Failed to kill shell {}: {}", shell_id, e);
                Ok(ToolResult::error(error_message)
                    .with_metadata("shell_id", serde_json::json!(shell_id))
                    .with_metadata("killed", serde_json::json!(false)))
            }
        }
    }

    /// Check permissions before execution
    async fn check_permissions(
        &self,
        params: &serde_json::Value,
        _context: &ToolContext,
    ) -> PermissionCheckResult {
        // Extract shell_id for validation
        let shell_id = match params
            .get("shell_id")
            .or_else(|| params.get("task_id"))
            .and_then(|v| v.as_str())
        {
            Some(id) => id,
            None => return PermissionCheckResult::deny("Missing shell_id parameter"),
        };

        // Basic validation - ensure shell_id is not empty
        if shell_id.trim().is_empty() {
            return PermissionCheckResult::deny("shell_id cannot be empty");
        }

        // Allow the operation - killing tasks is generally safe
        PermissionCheckResult::allow()
    }

    /// Get tool options
    fn options(&self) -> ToolOptions {
        ToolOptions::new()
            .with_max_retries(0) // Don't retry kill operations
            .with_base_timeout(std::time::Duration::from_secs(10)) // Quick timeout for kill operations
            .with_dynamic_timeout(false)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_context() -> ToolContext {
        ToolContext::new(PathBuf::from("/tmp"))
            .with_session_id("test-session")
            .with_user("test-user")
    }

    fn create_test_manager() -> Arc<TaskManager> {
        let temp_dir = TempDir::new().unwrap();
        Arc::new(TaskManager::new().with_output_directory(temp_dir.path().to_path_buf()))
    }

    #[test]
    fn test_tool_name() {
        let tool = KillShellTool::new();
        assert_eq!(tool.name(), "KillShell");
    }

    #[test]
    fn test_tool_description() {
        let tool = KillShellTool::new();
        assert!(!tool.description().is_empty());
        assert!(tool.description().contains("kill"));
        assert!(tool.description().contains("shell"));
    }

    #[test]
    fn test_tool_input_schema() {
        let tool = KillShellTool::new();
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["shell_id"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("shell_id")));
    }

    #[test]
    fn test_tool_options() {
        let tool = KillShellTool::new();
        let options = tool.options();
        assert_eq!(options.max_retries, 0);
        assert_eq!(options.base_timeout, std::time::Duration::from_secs(10));
        assert!(!options.enable_dynamic_timeout);
    }

    #[test]
    fn test_builder_with_task_manager() {
        let task_manager = create_test_manager();
        let tool = KillShellTool::with_task_manager(task_manager.clone());
        assert!(Arc::ptr_eq(&tool.task_manager, &task_manager));
    }

    // Permission Check Tests

    #[tokio::test]
    async fn test_check_permissions_valid_shell_id() {
        let tool = KillShellTool::new();
        let context = create_test_context();
        let params = serde_json::json!({"shell_id": "test-task-123"});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_permissions_task_id_alias() {
        let tool = KillShellTool::new();
        let context = create_test_context();
        let params = serde_json::json!({"task_id": "test-task-123"});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_permissions_missing_shell_id() {
        let tool = KillShellTool::new();
        let context = create_test_context();
        let params = serde_json::json!({});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_denied());
    }

    #[tokio::test]
    async fn test_check_permissions_empty_shell_id() {
        let tool = KillShellTool::new();
        let context = create_test_context();
        let params = serde_json::json!({"shell_id": ""});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_denied());
    }

    // Execution Tests

    #[tokio::test]
    async fn test_execute_nonexistent_task() {
        let task_manager = create_test_manager();
        let tool = KillShellTool::with_task_manager(task_manager);
        let context = create_test_context();
        let params = serde_json::json!({"shell_id": "nonexistent-task"});

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.is_error());
        // 错误信息在 error 字段中，不是 output 字段
        assert!(tool_result.error.unwrap().contains("No shell found"));
    }

    #[tokio::test]
    async fn test_execute_missing_shell_id() {
        let tool = KillShellTool::new();
        let context = create_test_context();
        let params = serde_json::json!({});

        let result = tool.execute(params, &context).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidParams(_)));
    }

    #[tokio::test]
    async fn test_execute_with_task_id_alias() {
        let task_manager = create_test_manager();
        let tool = KillShellTool::with_task_manager(task_manager);
        let context = create_test_context();
        let params = serde_json::json!({"task_id": "nonexistent-task"});

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.is_error());
        // 错误信息在 error 字段中，不是 output 字段
        assert!(tool_result.error.unwrap().contains("No shell found"));
    }

    #[tokio::test]
    async fn test_execute_kill_running_task() {
        let task_manager = create_test_manager();
        let tool = KillShellTool::with_task_manager(task_manager.clone());
        let context = create_test_context();

        // Start a long-running task
        let command = if cfg!(target_os = "windows") {
            "timeout /t 30"
        } else {
            "sleep 30"
        };
        let task_id = task_manager.start(command, &context).await.unwrap();

        // Kill the task
        let params = serde_json::json!({"shell_id": task_id});
        let result = tool.execute(params, &context).await;

        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.is_success());
        assert!(tool_result.output.unwrap().contains("Successfully killed"));
    }
}
