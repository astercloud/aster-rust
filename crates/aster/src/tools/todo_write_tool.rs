//! Todo Write Tool Implementation
//!
//! 此模块实现了 `TodoWriteTool`，用于任务管理和进度跟踪：
//! - 创建和管理结构化任务列表
//! - 跟踪任务状态（pending/in_progress/completed）
//! - 支持多 Agent 任务隔离
//! - 自动提醒机制
//! - 任务完成后自动清理
//!
//! Requirements: 基于 Claude Agent SDK todo.ts 中的 TodoWriteTool 实现

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::base::{PermissionCheckResult, Tool};
use super::context::{ToolContext, ToolOptions, ToolResult};
use super::error::ToolError;

/// Todo 项目状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    /// 待处理
    #[default]
    Pending,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
}

/// Todo 项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// 任务描述（命令式形式，如 "Run tests"）
    pub content: String,
    /// 任务状态
    pub status: TodoStatus,
    /// 进行时形式（如 "Running tests"）
    pub active_form: String,
}

impl TodoItem {
    /// 创建新的 Todo 项目
    pub fn new(content: impl Into<String>, active_form: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            status: TodoStatus::Pending,
            active_form: active_form.into(),
        }
    }

    /// 创建带状态的 Todo 项目
    pub fn with_status(
        content: impl Into<String>,
        active_form: impl Into<String>,
        status: TodoStatus,
    ) -> Self {
        Self {
            content: content.into(),
            status,
            active_form: active_form.into(),
        }
    }

    /// 检查是否为进行中状态
    pub fn is_in_progress(&self) -> bool {
        self.status == TodoStatus::InProgress
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        self.status == TodoStatus::Completed
    }
}

/// TodoWrite 工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoWriteInput {
    /// 更新后的 todo 列表
    pub todos: Vec<TodoItem>,
}

/// Todo 存储管理器
#[derive(Debug, Default)]
pub struct TodoStorage {
    /// 按 agent_id 分组的 todo 存储
    storage: RwLock<HashMap<String, Vec<TodoItem>>>,
}

impl TodoStorage {
    /// 创建新的 Todo 存储
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取指定 agent_id 的 todos
    pub fn get_todos(&self, agent_id: &str) -> Vec<TodoItem> {
        self.storage
            .read()
            .unwrap()
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// 设置指定 agent_id 的 todos
    pub fn set_todos(&self, agent_id: &str, todos: Vec<TodoItem>) {
        let mut storage = self.storage.write().unwrap();
        if todos.is_empty() {
            storage.remove(agent_id);
        } else {
            storage.insert(agent_id.to_string(), todos);
        }
    }

    /// 获取所有 agent 的 todo 统计
    pub fn get_stats(&self) -> HashMap<String, (usize, usize, usize)> {
        let storage = self.storage.read().unwrap();
        storage
            .iter()
            .map(|(agent_id, todos)| {
                let pending = todos
                    .iter()
                    .filter(|t| t.status == TodoStatus::Pending)
                    .count();
                let in_progress = todos
                    .iter()
                    .filter(|t| t.status == TodoStatus::InProgress)
                    .count();
                let completed = todos
                    .iter()
                    .filter(|t| t.status == TodoStatus::Completed)
                    .count();
                (agent_id.clone(), (pending, in_progress, completed))
            })
            .collect()
    }
}

/// Todo Write Tool for task management and progress tracking
///
/// 提供结构化的任务管理功能：
/// - 创建和管理任务列表
/// - 跟踪任务状态变化
/// - 支持多 Agent 隔离
/// - 自动完成清理
/// - 状态验证和约束
#[derive(Debug)]
pub struct TodoWriteTool {
    /// Todo 存储管理器
    storage: Arc<TodoStorage>,
    /// 默认 agent ID
    default_agent_id: String,
}

impl Default for TodoWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoWriteTool {
    /// Create a new TodoWriteTool with default settings
    pub fn new() -> Self {
        Self {
            storage: Arc::new(TodoStorage::new()),
            default_agent_id: "main".to_string(),
        }
    }

    /// Create a TodoWriteTool with custom storage
    pub fn with_storage(storage: Arc<TodoStorage>) -> Self {
        Self {
            storage,
            default_agent_id: "main".to_string(),
        }
    }

    /// Set default agent ID
    pub fn with_default_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.default_agent_id = agent_id.into();
        self
    }

    /// Get the todo storage
    pub fn storage(&self) -> &Arc<TodoStorage> {
        &self.storage
    }

    /// 验证 todo 列表的有效性
    fn validate_todos(&self, todos: &[TodoItem]) -> Result<(), String> {
        // 检查只能有一个 in_progress 任务
        let in_progress_count = todos.iter().filter(|t| t.is_in_progress()).count();
        if in_progress_count > 1 {
            return Err("Only one task can be in_progress at a time".to_string());
        }

        // 检查任务内容不能为空
        for todo in todos {
            if todo.content.trim().is_empty() {
                return Err("Task content cannot be empty".to_string());
            }
            if todo.active_form.trim().is_empty() {
                return Err("Task active_form cannot be empty".to_string());
            }
        }

        Ok(())
    }

    /// 获取 agent ID（从上下文或使用默认值）
    fn get_agent_id(&self, context: &ToolContext) -> String {
        // 尝试从环境变量或会话 ID 中获取 agent ID
        context
            .environment
            .get("AGENT_ID")
            .cloned()
            .unwrap_or_else(|| {
                if context.session_id.is_empty() {
                    self.default_agent_id.clone()
                } else {
                    context.session_id.clone()
                }
            })
    }
}

#[async_trait]
impl Tool for TodoWriteTool {
    /// Returns the tool name
    fn name(&self) -> &str {
        "TodoWrite"
    }

    /// Returns the tool description
    fn description(&self) -> &str {
        "Use this tool to create and manage a structured task list for your current coding session. \
         This helps you track progress, organize complex tasks, and demonstrate thoroughness to the user. \
         It also helps the user understand the progress of the task and overall progress of their requests.\n\n\
         ## When to Use This Tool\n\
         Use this tool proactively in these scenarios:\n\
         1. Complex multi-step tasks - When a task requires 3 or more distinct steps or actions\n\
         2. Non-trivial and complex tasks - Tasks that require careful planning or multiple operations\n\
         3. User explicitly requests todo list - When the user directly asks you to use the todo list\n\
         4. User provides multiple tasks - When users provide a list of things to be done\n\
         5. After receiving new instructions - Immediately capture user requirements as todos\n\
         6. When you start working on a task - Mark it as in_progress BEFORE beginning work\n\
         7. After completing a task - Mark it as completed and add any new follow-up tasks\n\n\
         ## Task States and Management\n\
         1. **Task States**: Use these states to track progress:\n\
            - pending: Task not yet started\n\
            - in_progress: Currently working on (limit to ONE task at a time)\n\
            - completed: Task finished successfully\n\
         2. **Task Management**:\n\
            - Update task status in real-time as you work\n\
            - Mark tasks complete IMMEDIATELY after finishing\n\
            - Exactly ONE task must be in_progress at any time\n\
            - Complete current tasks before starting new ones\n\
            - Remove tasks that are no longer relevant from the list entirely"
    }

    /// Returns the JSON Schema for input parameters
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "description": "The updated todo list",
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "minLength": 1,
                                "description": "Task description (imperative form, e.g., 'Run tests')"
                            },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed"],
                                "description": "Task status"
                            },
                            "active_form": {
                                "type": "string",
                                "minLength": 1,
                                "description": "Present continuous form (e.g., 'Running tests')"
                            }
                        },
                        "required": ["content", "status", "active_form"]
                    }
                }
            },
            "required": ["todos"]
        })
    }

    /// Execute the todo write command
    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        // Extract todos parameter
        let input: TodoWriteInput = serde_json::from_value(params)
            .map_err(|e| ToolError::invalid_params(format!("Invalid input format: {}", e)))?;

        // Validate todos
        if let Err(error) = self.validate_todos(&input.todos) {
            return Ok(ToolResult::error(error));
        }

        // Get agent ID
        let agent_id = self.get_agent_id(context);

        // Get old todos for comparison
        let old_todos = self.storage.get_todos(&agent_id);

        // Check if all tasks are completed - if so, auto-clear the list
        let new_todos = if input.todos.iter().all(|t| t.is_completed()) {
            Vec::new()
        } else {
            input.todos.clone()
        };

        // Save the new todos
        self.storage.set_todos(&agent_id, new_todos.clone());

        // Create success message
        let message = if new_todos.is_empty() && !input.todos.is_empty() {
            "All tasks completed! Todo list has been automatically cleared. \
             Ensure that you continue to use the todo list to track your progress for future tasks."
        } else {
            "Todos have been modified successfully. \
             Ensure that you continue to use the todo list to track your progress. \
             Please proceed with the current tasks if applicable."
        };

        // Return result with old and new todos data
        Ok(ToolResult::success(message)
            .with_metadata("agent_id", serde_json::json!(agent_id))
            .with_metadata("old_todos", serde_json::json!(old_todos))
            .with_metadata("new_todos", serde_json::json!(input.todos))
            .with_metadata(
                "auto_cleared",
                serde_json::json!(new_todos.is_empty() && !input.todos.is_empty()),
            ))
    }

    /// Check permissions before execution
    async fn check_permissions(
        &self,
        params: &serde_json::Value,
        _context: &ToolContext,
    ) -> PermissionCheckResult {
        // Validate input format
        match serde_json::from_value::<TodoWriteInput>(params.clone()) {
            Ok(input) => {
                // Validate todos
                if let Err(error) = self.validate_todos(&input.todos) {
                    return PermissionCheckResult::deny(format!("Invalid todos: {}", error));
                }
                PermissionCheckResult::allow()
            }
            Err(e) => PermissionCheckResult::deny(format!("Invalid input format: {}", e)),
        }
    }

    /// Get tool options
    fn options(&self) -> ToolOptions {
        ToolOptions::new()
            .with_max_retries(0) // Don't retry todo operations
            .with_base_timeout(std::time::Duration::from_secs(5)) // Quick timeout for todo operations
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

    fn create_test_context() -> ToolContext {
        ToolContext::new(PathBuf::from("/tmp"))
            .with_session_id("test-session")
            .with_user("test-user")
    }

    fn create_test_storage() -> Arc<TodoStorage> {
        Arc::new(TodoStorage::new())
    }

    #[test]
    fn test_todo_item_creation() {
        let todo = TodoItem::new("Run tests", "Running tests");
        assert_eq!(todo.content, "Run tests");
        assert_eq!(todo.active_form, "Running tests");
        assert_eq!(todo.status, TodoStatus::Pending);
        assert!(!todo.is_in_progress());
        assert!(!todo.is_completed());
    }

    #[test]
    fn test_todo_item_with_status() {
        let todo =
            TodoItem::with_status("Build project", "Building project", TodoStatus::InProgress);
        assert_eq!(todo.content, "Build project");
        assert_eq!(todo.active_form, "Building project");
        assert_eq!(todo.status, TodoStatus::InProgress);
        assert!(todo.is_in_progress());
        assert!(!todo.is_completed());
    }

    #[test]
    fn test_todo_storage_basic_operations() {
        let storage = TodoStorage::new();
        let agent_id = "test-agent";

        // Initially empty
        assert!(storage.get_todos(agent_id).is_empty());

        // Add todos
        let todos = vec![
            TodoItem::new("Task 1", "Doing task 1"),
            TodoItem::with_status("Task 2", "Doing task 2", TodoStatus::InProgress),
        ];
        storage.set_todos(agent_id, todos.clone());

        // Retrieve todos
        let retrieved = storage.get_todos(agent_id);
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].content, "Task 1");
        assert_eq!(retrieved[1].content, "Task 2");
        assert_eq!(retrieved[1].status, TodoStatus::InProgress);

        // Clear todos
        storage.set_todos(agent_id, vec![]);
        assert!(storage.get_todos(agent_id).is_empty());
    }

    #[test]
    fn test_todo_storage_multi_agent() {
        let storage = TodoStorage::new();
        let agent1 = "agent-1";
        let agent2 = "agent-2";

        // Add todos for different agents
        storage.set_todos(
            agent1,
            vec![TodoItem::new("Agent 1 Task", "Doing agent 1 task")],
        );
        storage.set_todos(
            agent2,
            vec![TodoItem::new("Agent 2 Task", "Doing agent 2 task")],
        );

        // Verify isolation
        let todos1 = storage.get_todos(agent1);
        let todos2 = storage.get_todos(agent2);

        assert_eq!(todos1.len(), 1);
        assert_eq!(todos2.len(), 1);
        assert_eq!(todos1[0].content, "Agent 1 Task");
        assert_eq!(todos2[0].content, "Agent 2 Task");
    }

    #[test]
    fn test_todo_storage_stats() {
        let storage = TodoStorage::new();
        let agent_id = "test-agent";

        let todos = vec![
            TodoItem::new("Pending task", "Doing pending task"),
            TodoItem::with_status(
                "In progress task",
                "Doing in progress task",
                TodoStatus::InProgress,
            ),
            TodoItem::with_status(
                "Completed task",
                "Doing completed task",
                TodoStatus::Completed,
            ),
        ];
        storage.set_todos(agent_id, todos);

        let stats = storage.get_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[agent_id], (1, 1, 1)); // (pending, in_progress, completed)
    }

    #[test]
    fn test_tool_name() {
        let tool = TodoWriteTool::new();
        assert_eq!(tool.name(), "TodoWrite");
    }

    #[test]
    fn test_tool_description() {
        let tool = TodoWriteTool::new();
        assert!(!tool.description().is_empty());
        assert!(tool.description().contains("task list"));
        assert!(tool.description().contains("progress"));
    }

    #[test]
    fn test_tool_input_schema() {
        let tool = TodoWriteTool::new();
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["todos"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("todos")));
    }

    #[test]
    fn test_tool_options() {
        let tool = TodoWriteTool::new();
        let options = tool.options();
        assert_eq!(options.max_retries, 0);
        assert_eq!(options.base_timeout, std::time::Duration::from_secs(5));
        assert!(!options.enable_dynamic_timeout);
    }

    #[test]
    fn test_builder_with_storage() {
        let storage = create_test_storage();
        let tool = TodoWriteTool::with_storage(storage.clone());
        assert!(Arc::ptr_eq(&tool.storage, &storage));
    }

    #[test]
    fn test_builder_with_default_agent_id() {
        let tool = TodoWriteTool::new().with_default_agent_id("custom-agent");
        assert_eq!(tool.default_agent_id, "custom-agent");
    }

    #[test]
    fn test_validate_todos_success() {
        let tool = TodoWriteTool::new();
        let todos = vec![
            TodoItem::new("Task 1", "Doing task 1"),
            TodoItem::with_status("Task 2", "Doing task 2", TodoStatus::InProgress),
        ];
        assert!(tool.validate_todos(&todos).is_ok());
    }

    #[test]
    fn test_validate_todos_multiple_in_progress() {
        let tool = TodoWriteTool::new();
        let todos = vec![
            TodoItem::with_status("Task 1", "Doing task 1", TodoStatus::InProgress),
            TodoItem::with_status("Task 2", "Doing task 2", TodoStatus::InProgress),
        ];
        let result = tool.validate_todos(&todos);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Only one task can be in_progress"));
    }

    #[test]
    fn test_validate_todos_empty_content() {
        let tool = TodoWriteTool::new();
        let todos = vec![TodoItem::new("", "Doing something")];
        let result = tool.validate_todos(&todos);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Task content cannot be empty"));
    }

    #[test]
    fn test_validate_todos_empty_active_form() {
        let tool = TodoWriteTool::new();
        let todos = vec![TodoItem::new("Do something", "")];
        let result = tool.validate_todos(&todos);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Task active_form cannot be empty"));
    }

    #[test]
    fn test_get_agent_id_from_environment() {
        let tool = TodoWriteTool::new();
        let context = create_test_context().with_env_var("AGENT_ID", "env-agent");
        let agent_id = tool.get_agent_id(&context);
        assert_eq!(agent_id, "env-agent");
    }

    #[test]
    fn test_get_agent_id_from_session() {
        let tool = TodoWriteTool::new();
        let context = create_test_context();
        let agent_id = tool.get_agent_id(&context);
        assert_eq!(agent_id, "test-session");
    }

    #[test]
    fn test_get_agent_id_default() {
        let tool = TodoWriteTool::new();
        let context = ToolContext::new(PathBuf::from("/tmp"));
        let agent_id = tool.get_agent_id(&context);
        assert_eq!(agent_id, "main");
    }

    // Permission Check Tests

    #[tokio::test]
    async fn test_check_permissions_valid_input() {
        let tool = TodoWriteTool::new();
        let context = create_test_context();
        let params = serde_json::json!({
            "todos": [
                {
                    "content": "Test task",
                    "status": "pending",
                    "active_form": "Testing task"
                }
            ]
        });

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_permissions_invalid_format() {
        let tool = TodoWriteTool::new();
        let context = create_test_context();
        let params = serde_json::json!({"invalid": "format"});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_denied());
    }

    #[tokio::test]
    async fn test_check_permissions_multiple_in_progress() {
        let tool = TodoWriteTool::new();
        let context = create_test_context();
        let params = serde_json::json!({
            "todos": [
                {
                    "content": "Task 1",
                    "status": "in_progress",
                    "active_form": "Doing task 1"
                },
                {
                    "content": "Task 2",
                    "status": "in_progress",
                    "active_form": "Doing task 2"
                }
            ]
        });

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_denied());
    }

    // Execution Tests

    #[tokio::test]
    async fn test_execute_simple_todos() {
        let storage = create_test_storage();
        let tool = TodoWriteTool::with_storage(storage.clone());
        let context = create_test_context();
        let params = serde_json::json!({
            "todos": [
                {
                    "content": "Run tests",
                    "status": "pending",
                    "active_form": "Running tests"
                },
                {
                    "content": "Build project",
                    "status": "in_progress",
                    "active_form": "Building project"
                }
            ]
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.is_success());
        assert!(tool_result
            .output
            .unwrap()
            .contains("modified successfully"));

        // Verify todos were saved
        let saved_todos = storage.get_todos("test-session");
        assert_eq!(saved_todos.len(), 2);
        assert_eq!(saved_todos[0].content, "Run tests");
        assert_eq!(saved_todos[1].content, "Build project");
        assert_eq!(saved_todos[1].status, TodoStatus::InProgress);
    }

    #[tokio::test]
    async fn test_execute_auto_clear_completed() {
        let storage = create_test_storage();
        let tool = TodoWriteTool::with_storage(storage.clone());
        let context = create_test_context();
        let params = serde_json::json!({
            "todos": [
                {
                    "content": "Task 1",
                    "status": "completed",
                    "active_form": "Doing task 1"
                },
                {
                    "content": "Task 2",
                    "status": "completed",
                    "active_form": "Doing task 2"
                }
            ]
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.is_success());
        assert!(tool_result.output.unwrap().contains("All tasks completed"));

        // Verify todos were cleared
        let saved_todos = storage.get_todos("test-session");
        assert!(saved_todos.is_empty());

        // Check metadata
        assert_eq!(
            tool_result.metadata.get("auto_cleared"),
            Some(&serde_json::json!(true))
        );
    }

    #[tokio::test]
    async fn test_execute_invalid_todos() {
        let tool = TodoWriteTool::new();
        let context = create_test_context();
        let params = serde_json::json!({
            "todos": [
                {
                    "content": "Task 1",
                    "status": "in_progress",
                    "active_form": "Doing task 1"
                },
                {
                    "content": "Task 2",
                    "status": "in_progress",
                    "active_form": "Doing task 2"
                }
            ]
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.is_error());
        assert!(tool_result
            .error
            .unwrap()
            .contains("Only one task can be in_progress"));
    }

    #[tokio::test]
    async fn test_execute_invalid_input_format() {
        let tool = TodoWriteTool::new();
        let context = create_test_context();
        let params = serde_json::json!({"invalid": "format"});

        let result = tool.execute(params, &context).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidParams(_)));
    }

    #[tokio::test]
    async fn test_execute_with_metadata() {
        let storage = create_test_storage();
        let tool = TodoWriteTool::with_storage(storage.clone());
        let context = create_test_context();

        // First, add some todos
        let initial_todos = vec![TodoItem::new("Old task", "Doing old task")];
        storage.set_todos("test-session", initial_todos.clone());

        let params = serde_json::json!({
            "todos": [
                {
                    "content": "New task",
                    "status": "pending",
                    "active_form": "Doing new task"
                }
            ]
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());
        let tool_result = result.unwrap();
        assert!(tool_result.is_success());

        // Check metadata
        assert_eq!(
            tool_result.metadata.get("agent_id"),
            Some(&serde_json::json!("test-session"))
        );
        assert!(tool_result.metadata.contains_key("old_todos"));
        assert!(tool_result.metadata.contains_key("new_todos"));
        assert_eq!(
            tool_result.metadata.get("auto_cleared"),
            Some(&serde_json::json!(false))
        );
    }
}
