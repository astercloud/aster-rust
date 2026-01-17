//! Task Tool - 后台任务管理工具
//!
//! 基于 TaskManager 实现的任务启动和管理工具，对齐 Claude Agent SDK

use super::base::{PermissionCheckResult, Tool};
use super::context::{ToolContext, ToolResult};
use super::error::ToolError;
use super::task::TaskManager;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// TaskTool 输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    /// 任务命令
    pub command: String,
    /// 任务描述（可选）
    pub description: Option<String>,
    /// 是否在后台运行
    pub run_in_background: Option<bool>,
}

/// TaskTool - 启动后台任务
///
/// 对齐 Claude Agent SDK 的 TaskTool 功能，用于启动和管理后台任务
pub struct TaskTool {
    /// 任务管理器
    task_manager: Arc<TaskManager>,
}

impl TaskTool {
    /// 创建新的 TaskTool
    pub fn new() -> Self {
        Self {
            task_manager: Arc::new(TaskManager::new()),
        }
    }

    /// 使用自定义 TaskManager 创建 TaskTool
    pub fn with_manager(task_manager: Arc<TaskManager>) -> Self {
        Self { task_manager }
    }
}

impl Default for TaskTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TaskTool {
    fn name(&self) -> &str {
        "Task"
    }

    fn description(&self) -> &str {
        r#"启动后台任务执行命令

用于启动长时间运行的命令或需要并行执行的任务。支持：
- 后台执行命令
- 任务状态跟踪
- 输出文件持久化
- 并发任务限制

参数：
- command: 要执行的命令
- description: 任务描述（可选）
- run_in_background: 是否后台运行（默认 true）

返回任务 ID，可用于后续查询任务状态和输出。"#
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "要执行的命令"
                },
                "description": {
                    "type": "string",
                    "description": "任务描述（可选）"
                },
                "run_in_background": {
                    "type": "boolean",
                    "description": "是否在后台运行（默认 true）"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let input: TaskInput = serde_json::from_value(params)
            .map_err(|e| ToolError::invalid_params(format!("参数解析失败: {}", e)))?;

        let run_in_background = input.run_in_background.unwrap_or(true);

        // 启动任务
        let task_id = self.task_manager.start(&input.command, context).await?;

        let description = input.description.unwrap_or_else(|| {
            // 截取命令的前50个字符作为描述，安全处理 UTF-8
            let cmd = &input.command;
            if cmd.chars().count() > 50 {
                let truncated: String = cmd.chars().take(47).collect();
                format!("{}...", truncated)
            } else {
                cmd.to_string()
            }
        });

        if run_in_background {
            // 后台运行 - 立即返回任务 ID
            Ok(ToolResult::success(format!(
                "任务已启动（后台运行）\n任务 ID: {}\n描述: {}\n命令: {}\n\n使用 TaskOutput 工具查询任务状态和输出。",
                task_id, description, input.command
            )).with_metadata("task_id", serde_json::json!(task_id)))
        } else {
            // 前台运行 - 等待完成
            // 等待任务完成（最多等待30秒）
            let timeout = Duration::from_secs(30);
            let start_time = std::time::Instant::now();

            loop {
                if let Some(state) = self.task_manager.get_status(&task_id).await {
                    if state.status.is_terminal() {
                        // 任务已完成，获取输出
                        let output = self
                            .task_manager
                            .get_output(&task_id, None)
                            .await
                            .unwrap_or_else(|_| "无法获取任务输出".to_string());

                        let duration = state.duration().as_secs_f64();

                        return Ok(ToolResult::success(format!(
                            "任务已完成\n任务 ID: {}\n描述: {}\n状态: {}\n执行时间: {:.2}秒\n\n=== 输出 ===\n{}",
                            task_id, description, state.status, duration, output
                        )).with_metadata("task_id", serde_json::json!(task_id))
                          .with_metadata("status", serde_json::json!(state.status.to_string()))
                          .with_metadata("duration", serde_json::json!(duration)));
                    }
                }

                // 检查超时
                if start_time.elapsed() > timeout {
                    return Ok(ToolResult::success(format!(
                        "任务启动成功但执行时间超过 {}秒，已转为后台运行\n任务 ID: {}\n描述: {}\n\n使用 TaskOutput 工具查询任务状态和输出。",
                        timeout.as_secs(), task_id, description
                    )).with_metadata("task_id", serde_json::json!(task_id)));
                }

                // 等待100ms后重新检查
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    async fn check_permissions(
        &self,
        _params: &serde_json::Value,
        _context: &ToolContext,
    ) -> PermissionCheckResult {
        // 任务启动需要执行权限
        PermissionCheckResult::ask("执行后台任务")
    }
}

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

    #[tokio::test]
    async fn test_task_tool_new() {
        let tool = TaskTool::new();
        assert_eq!(tool.name(), "Task");
    }

    #[tokio::test]
    async fn test_task_tool_input_schema() {
        let tool = TaskTool::new();
        let schema = tool.input_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["command"].is_object());
        assert_eq!(schema["required"], serde_json::json!(["command"]));
    }

    #[tokio::test]
    async fn test_task_tool_execute_background() {
        let temp_dir = TempDir::new().unwrap();
        let task_manager = Arc::new(
            TaskManager::new()
                .with_output_directory(temp_dir.path().to_path_buf())
                .with_max_concurrent(5),
        );
        let tool = TaskTool::with_manager(task_manager);
        let context = create_test_context();

        let params = serde_json::json!({
            "command": "echo hello",
            "description": "测试任务",
            "run_in_background": true
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert!(tool_result.success);
        assert!(tool_result.output.as_ref().unwrap().contains("任务已启动"));
        assert!(tool_result.metadata.contains_key("task_id"));
    }

    #[tokio::test]
    async fn test_task_tool_execute_foreground() {
        let temp_dir = TempDir::new().unwrap();
        let task_manager = Arc::new(
            TaskManager::new()
                .with_output_directory(temp_dir.path().to_path_buf())
                .with_max_concurrent(5),
        );
        let tool = TaskTool::with_manager(task_manager);
        let context = create_test_context();

        let params = serde_json::json!({
            "command": "echo hello world",
            "run_in_background": false
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert!(tool_result.success);
        // 应该包含任务输出
        let output = tool_result.output.as_ref().unwrap();
        assert!(output.contains("hello world") || output.contains("任务已完成"));
    }

    #[tokio::test]
    async fn test_task_tool_invalid_params() {
        let tool = TaskTool::new();
        let context = create_test_context();

        let params = serde_json::json!({
            "invalid": "params"
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_task_tool_check_permissions() {
        let tool = TaskTool::new();
        let context = create_test_context();
        let params = serde_json::json!({"command": "echo test"});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.requires_confirmation());
    }
}
