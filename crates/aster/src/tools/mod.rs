// =============================================================================
// Tool System Module
// =============================================================================
//
// This module provides a unified tool system for aster-rust, aligned with
// - Tool trait and base types
// - Tool registry for managing native and MCP tools
// - Core tool implementations (Bash, File, Search, etc.)
// - Permission integration
// - Audit logging

use std::sync::Arc;

// Core modules
pub mod base;
pub mod context;
pub mod error;
pub mod hooks;
pub mod registry;
pub mod task;

// Tool implementations
pub mod agent_control;
pub mod analyze_image;
pub mod ask;
pub mod bash;
pub mod file;
pub mod lsp;
pub mod mcp_resource_tools;
pub mod notebook_edit_tool;
pub mod plan_mode_tool;
pub mod search;
pub mod task_list_tools;
pub mod task_output_tool;
pub mod task_stop_tool;
pub mod web;
pub mod workflow_integration;

// Skills integration

// =============================================================================
// Core Type Exports
// =============================================================================

// Error types
pub use error::ToolError;

// Context and configuration types
pub use context::{ToolContext, ToolDefinition, ToolOptions, ToolResult};

// Base trait and permission types
pub use base::{PermissionBehavior, PermissionCheckResult, Tool};

// Registry types
pub use registry::{McpToolWrapper, PermissionRequestCallback, ToolRegistry};

// Hook system types
pub use hooks::{
    ErrorTrackingHook, FileOperationHook, HookContext, HookTrigger, LoggingHook, ToolHook,
    ToolHookManager,
};

// Task management types
pub use task::{
    TaskManager, TaskState, TaskStatus, DEFAULT_MAX_CONCURRENT, DEFAULT_MAX_RUNTIME_SECS,
};

// Tool implementations
pub use bash::{BashTool, SafetyCheckResult, SandboxConfig, MAX_OUTPUT_LENGTH};

// File tools
pub use file::{
    compute_content_hash, create_shared_history, EditTool, FileReadHistory, FileReadRecord,
    ReadTool, SharedFileReadHistory, WriteTool,
};

// Search tools
pub use search::{
    GlobTool, GrepOutputMode, GrepTool, SearchResult, DEFAULT_MAX_CONTEXT_LINES,
    DEFAULT_MAX_RESULTS, MAX_OUTPUT_SIZE,
};

// Ask tool
pub use agent_control::{
    register_agent_control_tools, AgentControlToolConfig, CloseAgentCallback, CloseAgentRequest,
    CloseAgentResponse, CloseAgentTool, ResumeAgentCallback, ResumeAgentRequest,
    ResumeAgentResponse, ResumeAgentTool, SendInputCallback, SendInputRequest, SendInputResponse,
    SendInputTool, SpawnAgentCallback, SpawnAgentRequest, SpawnAgentResponse, SpawnAgentTool,
    WaitAgentCallback, WaitAgentRequest, WaitAgentResponse, WaitAgentTool,
};
pub use ask::{AskCallback, AskOption, AskResult, AskTool, DEFAULT_ASK_TIMEOUT_SECS};

// LSP tool
pub use lsp::{
    CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity, HoverInfo, Location,
    LspCallback, LspOperation, LspResult, LspTool, Position, Range,
};
pub use mcp_resource_tools::{
    register_extension_resource_tools, ListMcpResourcesTool, ReadMcpResourceTool,
};

// Skill tool
pub use crate::skills::SkillTool;

// Task tools
pub use notebook_edit_tool::{NotebookCell, NotebookContent, NotebookEditInput, NotebookEditTool};
pub use plan_mode_tool::{EnterPlanModeTool, ExitPlanModeTool, PlanModeState, SavedPlan};
pub use task_list_tools::{
    TaskCreateInput, TaskCreateTool, TaskGetInput, TaskGetTool, TaskListInput, TaskListStorage,
    TaskListTool, TaskUpdateInput, TaskUpdateStatus, TaskUpdateTool,
};
pub use task_output_tool::TaskOutputTool;
pub use task_stop_tool::TaskStopTool;

// Web tools
pub use web::{clear_web_caches, get_web_cache_stats, WebCache, WebFetchTool, WebSearchTool};

// Image analysis tools
// Image analysis tools
pub use analyze_image::AnalyzeImageTool;
pub use analyze_image::{AnalyzeImageInput, AnalyzeImageResult, ImageDimensions};

// Workflow integration
pub use workflow_integration::{WorkflowIntegratedTool, WorkflowIntegratedToolBuilder};

// =============================================================================
// Tool Registration (Requirements: 11.3)
// =============================================================================

/// Configuration for tool registration
#[derive(Default)]
pub struct ToolRegistrationConfig {
    /// Callback for AskTool user interaction
    pub ask_callback: Option<AskCallback>,
    /// Callback for LSPTool operations
    pub lsp_callback: Option<LspCallback>,
    /// Whether to enable PDF reading in ReadTool
    pub pdf_enabled: bool,
    /// Whether to enable hook system
    pub hooks_enabled: bool,
    /// Optional modern delegation / agent runtime tools
    pub agent_control_tools: Option<AgentControlToolConfig>,
}

impl std::fmt::Debug for ToolRegistrationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistrationConfig")
            .field(
                "ask_callback",
                &self.ask_callback.as_ref().map(|_| "<callback>"),
            )
            .field(
                "lsp_callback",
                &self.lsp_callback.as_ref().map(|_| "<callback>"),
            )
            .field("pdf_enabled", &self.pdf_enabled)
            .field("hooks_enabled", &self.hooks_enabled)
            .field(
                "agent_control_tools",
                &self.agent_control_tools.as_ref().map(|_| "<callbacks>"),
            )
            .finish()
    }
}

impl Clone for ToolRegistrationConfig {
    fn clone(&self) -> Self {
        Self {
            ask_callback: self.ask_callback.clone(),
            lsp_callback: self.lsp_callback.clone(),
            pdf_enabled: self.pdf_enabled,
            hooks_enabled: self.hooks_enabled,
            agent_control_tools: self.agent_control_tools.clone(),
        }
    }
}

impl ToolRegistrationConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the AskTool callback
    pub fn with_ask_callback(mut self, callback: AskCallback) -> Self {
        self.ask_callback = Some(callback);
        self
    }

    /// Set the LSPTool callback
    pub fn with_lsp_callback(mut self, callback: LspCallback) -> Self {
        self.lsp_callback = Some(callback);
        self
    }

    /// Enable PDF reading
    pub fn with_pdf_enabled(mut self, enabled: bool) -> Self {
        self.pdf_enabled = enabled;
        self
    }

    /// Enable hook system
    pub fn with_hooks_enabled(mut self, enabled: bool) -> Self {
        self.hooks_enabled = enabled;
        self
    }

    /// Register modern delegation / agent runtime tools using callbacks
    pub fn with_agent_control_tools(mut self, config: AgentControlToolConfig) -> Self {
        self.agent_control_tools = Some(config);
        self
    }
}

/// Register all native tools with the registry
///
/// This function registers all built-in tools:
/// - BashTool: Shell command execution
/// - ReadTool: File reading (text, images, PDF, notebooks)
/// - WriteTool: File writing with validation
/// - EditTool: Smart file editing
/// - GlobTool: File search with glob patterns
/// - GrepTool: Content search with regex
/// - AskTool: User interaction (if callback provided)
/// - LSPTool: Code intelligence (if callback provided)
/// - SkillTool: Skill execution and management
///
/// # Arguments
/// * `registry` - The ToolRegistry to register tools with
/// * `config` - Configuration for tool registration
///
/// # Returns
/// A tuple containing (shared file read history, hook manager)
///
/// Requirements: 11.3
pub fn register_all_tools(
    registry: &mut ToolRegistry,
    config: ToolRegistrationConfig,
) -> (SharedFileReadHistory, Option<ToolHookManager>) {
    // Create shared file read history for file tools
    let shared_history = create_shared_history();

    // Initialize hook manager if enabled
    let hook_manager = if config.hooks_enabled {
        let manager = ToolHookManager::new(true);
        // Register default hooks in a blocking context
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                manager.register_default_hooks().await;
            })
        });
        Some(manager)
    } else {
        None
    };

    let shared_task_manager = Arc::new(TaskManager::new());
    let shared_task_list_storage = Arc::new(TaskListStorage::new());

    // Register BashTool
    registry.register(Box::new(BashTool::with_task_manager(
        shared_task_manager.clone(),
    )));

    // Register file tools with shared history
    let read_tool = ReadTool::new(shared_history.clone()).with_pdf_enabled(config.pdf_enabled);
    registry.register(Box::new(read_tool));

    let write_tool = WriteTool::new(shared_history.clone());
    registry.register(Box::new(write_tool));

    let edit_tool = EditTool::new(shared_history.clone());
    registry.register(Box::new(edit_tool));

    // Register search tools
    registry.register(Box::new(GlobTool::new()));
    registry.register(Box::new(GrepTool::new()));

    // Register AskTool if callback is provided
    if let Some(callback) = config.ask_callback {
        let ask_tool = AskTool::new().with_callback(callback);
        registry.register(Box::new(ask_tool));
    }

    // Register LSPTool if callback is provided
    if let Some(callback) = config.lsp_callback {
        let lsp_tool = LspTool::new().with_callback(callback);
        registry.register(Box::new(lsp_tool));
    }

    // Register SkillTool
    registry.register(Box::new(SkillTool::new()));

    // Register background execution and structured task board tools
    registry.register(Box::new(TaskCreateTool::with_storage(
        shared_task_list_storage.clone(),
    )));
    registry.register(Box::new(TaskListTool::with_storage(
        shared_task_list_storage.clone(),
    )));
    registry.register(Box::new(TaskGetTool::with_storage(
        shared_task_list_storage.clone(),
    )));
    registry.register(Box::new(TaskUpdateTool::with_storage(
        shared_task_list_storage,
    )));
    registry.register(Box::new(TaskOutputTool::with_manager(
        shared_task_manager.clone(),
    )));
    registry.register(Box::new(TaskStopTool::with_task_manager(
        shared_task_manager,
    )));
    registry.register(Box::new(NotebookEditTool::new()));

    // Register Plan Mode tools
    registry.register(Box::new(EnterPlanModeTool::new()));
    registry.register(Box::new(ExitPlanModeTool::new()));

    if let Some(agent_control_tools) = config.agent_control_tools.as_ref() {
        register_agent_control_tools(registry, agent_control_tools);
    }

    // Register Web tools
    registry.register(Box::new(WebFetchTool::new()));
    registry.register(Box::new(WebSearchTool::new()));

    // Register Image Analysis tools
    registry.register(Box::new(AnalyzeImageTool::new()));

    (shared_history, hook_manager)
}

/// Register all native tools with default configuration
///
/// This is a convenience function that registers all tools with default settings.
/// AskTool and LSPTool are not registered since they require callbacks.
///
/// # Arguments
/// * `registry` - The ToolRegistry to register tools with
///
/// # Returns
/// A tuple containing (shared file read history, hook manager)
///
/// Requirements: 11.3
pub fn register_default_tools(
    registry: &mut ToolRegistry,
) -> (SharedFileReadHistory, Option<ToolHookManager>) {
    register_all_tools(registry, ToolRegistrationConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_register_default_tools() {
        let mut registry = ToolRegistry::new();
        let (_history, _hook_manager) = register_default_tools(&mut registry);

        // Verify core tools are registered
        assert!(registry.contains("bash"));
        assert!(registry.contains("read"));
        assert!(registry.contains("write"));
        assert!(registry.contains("edit"));
        assert!(registry.contains("glob"));
        assert!(registry.contains("grep"));
        assert!(registry.contains("Skill"));
        assert!(registry.contains("TaskCreate"));
        assert!(registry.contains("TaskList"));
        assert!(registry.contains("TaskGet"));
        assert!(registry.contains("TaskUpdate"));
        assert!(registry.contains("TaskOutput"));
        assert!(registry.contains("TaskStop"));
        assert!(registry.contains("NotebookEdit"));
        assert!(registry.contains("EnterPlanMode"));
        assert!(registry.contains("ExitPlanMode"));
        assert!(registry.contains("WebFetch"));
        assert!(registry.contains("WebSearch"));
        assert!(registry.contains("analyze_image"));
        assert!(!registry.contains("spawn_agent"));
        assert!(!registry.contains("send_input"));
        assert!(!registry.contains("wait_agent"));
        assert!(!registry.contains("resume_agent"));
        assert!(!registry.contains("close_agent"));
        // AskTool and LSPTool should not be registered without callbacks
        assert!(!registry.contains("ask"));
        assert!(!registry.contains("lsp"));
    }

    #[test]
    fn test_register_all_tools_with_config() {
        use std::future::Future;
        use std::pin::Pin;
        use std::sync::Arc;

        let mut registry = ToolRegistry::new();

        // Create mock callbacks
        let ask_callback: AskCallback = Arc::new(|_request| {
            Box::pin(async { Some(serde_json::json!("test response")) })
                as Pin<Box<dyn Future<Output = Option<serde_json::Value>> + Send>>
        });
        let spawn_agent_callback: SpawnAgentCallback = Arc::new(|request| {
            Box::pin(async move {
                Ok(SpawnAgentResponse {
                    agent_id: request.parent_session_id,
                    nickname: Some("delegate".to_string()),
                    extra: std::collections::BTreeMap::new(),
                })
            })
        });

        let lsp_callback: LspCallback = Arc::new(|_operation, _path: PathBuf, _position| {
            Box::pin(async { Ok(LspResult::Definition { locations: vec![] }) })
                as Pin<Box<dyn Future<Output = Result<LspResult, String>> + Send>>
        });

        let config = ToolRegistrationConfig::new()
            .with_ask_callback(ask_callback)
            .with_lsp_callback(lsp_callback)
            .with_pdf_enabled(true)
            .with_agent_control_tools(
                AgentControlToolConfig::new().with_spawn_agent_callback(spawn_agent_callback),
            );

        let (_history, _hook_manager) = register_all_tools(&mut registry, config);

        // Verify all tools are registered
        assert!(registry.contains("bash"));
        assert!(registry.contains("read"));
        assert!(registry.contains("write"));
        assert!(registry.contains("edit"));
        assert!(registry.contains("glob"));
        assert!(registry.contains("grep"));
        assert!(registry.contains("ask"));
        assert!(registry.contains("lsp"));
        assert!(registry.contains("Skill"));
        assert!(registry.contains("TaskCreate"));
        assert!(registry.contains("TaskList"));
        assert!(registry.contains("TaskGet"));
        assert!(registry.contains("TaskUpdate"));
        assert!(registry.contains("TaskOutput"));
        assert!(registry.contains("TaskStop"));
        assert!(registry.contains("NotebookEdit"));
        assert!(registry.contains("EnterPlanMode"));
        assert!(registry.contains("ExitPlanMode"));
        assert!(registry.contains("WebFetch"));
        assert!(registry.contains("WebSearch"));
        assert!(registry.contains("analyze_image"));
        assert!(registry.contains("spawn_agent"));
        assert!(!registry.contains("send_input"));
    }

    #[test]
    fn test_shared_history_is_shared() {
        let mut registry = ToolRegistry::new();
        let (history, _hook_manager) = register_default_tools(&mut registry);

        // The history should be empty initially
        assert!(history.read().unwrap().is_empty());

        // We can write to it
        {
            let mut write_guard = history.write().unwrap();
            write_guard.record_read(FileReadRecord::new(
                std::path::PathBuf::from("/tmp/test.txt"),
                "hash123".to_string(),
                100,
            ));
        }

        // And read from it
        assert!(history
            .read()
            .unwrap()
            .has_read(&std::path::PathBuf::from("/tmp/test.txt")));
    }

    #[test]
    fn test_tool_registration_config_builder() {
        let config = ToolRegistrationConfig::new().with_pdf_enabled(true);

        assert!(config.pdf_enabled);
        assert!(config.ask_callback.is_none());
        assert!(config.lsp_callback.is_none());
        assert!(config.agent_control_tools.is_none());
    }
}
