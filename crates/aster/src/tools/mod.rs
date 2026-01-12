// =============================================================================
// Tool System Module
// =============================================================================
//
// This module provides a unified tool system for aster-rust, aligned with
// claude-code-open's tool architecture. It includes:
// - Tool trait and base types
// - Tool registry for managing native and MCP tools
// - Core tool implementations (Bash, File, Search, etc.)
// - Permission integration
// - Audit logging

// Core modules
pub mod base;
pub mod context;
pub mod error;
pub mod registry;
pub mod task;

// Tool implementations
pub mod ask;
pub mod bash;
pub mod file;
pub mod lsp;
pub mod search;

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

// Task management types
pub use task::{TaskManager, TaskState, TaskStatus, DEFAULT_MAX_CONCURRENT, DEFAULT_MAX_RUNTIME_SECS};

// Tool implementations
pub use bash::{BashTool, SafetyCheckResult, SandboxConfig, MAX_OUTPUT_LENGTH};

// File tools
pub use file::{
    create_shared_history, compute_content_hash,
    EditTool, FileReadHistory, FileReadRecord, ReadTool, SharedFileReadHistory, WriteTool,
};

// Search tools
pub use search::{
    GlobTool, GrepOutputMode, GrepTool, SearchResult,
    DEFAULT_MAX_RESULTS, DEFAULT_MAX_CONTEXT_LINES, MAX_OUTPUT_SIZE,
};

// Ask tool
pub use ask::{AskCallback, AskOption, AskResult, AskTool, DEFAULT_ASK_TIMEOUT_SECS};

// LSP tool
pub use lsp::{
    CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity, HoverInfo,
    Location, LspCallback, LspOperation, LspResult, LspTool, Position, Range,
};

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
}

impl std::fmt::Debug for ToolRegistrationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistrationConfig")
            .field("ask_callback", &self.ask_callback.as_ref().map(|_| "<callback>"))
            .field("lsp_callback", &self.lsp_callback.as_ref().map(|_| "<callback>"))
            .field("pdf_enabled", &self.pdf_enabled)
            .finish()
    }
}

impl Clone for ToolRegistrationConfig {
    fn clone(&self) -> Self {
        Self {
            ask_callback: self.ask_callback.clone(),
            lsp_callback: self.lsp_callback.clone(),
            pdf_enabled: self.pdf_enabled,
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
///
/// # Arguments
/// * `registry` - The ToolRegistry to register tools with
/// * `config` - Configuration for tool registration
///
/// # Returns
/// The shared file read history used by file tools
///
/// Requirements: 11.3
pub fn register_all_tools(
    registry: &mut ToolRegistry,
    config: ToolRegistrationConfig,
) -> SharedFileReadHistory {
    // Create shared file read history for file tools
    let shared_history = create_shared_history();

    // Register BashTool
    registry.register(Box::new(BashTool::new()));

    // Register file tools with shared history
    let read_tool = ReadTool::new(shared_history.clone())
        .with_pdf_enabled(config.pdf_enabled);
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

    shared_history
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
/// The shared file read history used by file tools
///
/// Requirements: 11.3
pub fn register_default_tools(registry: &mut ToolRegistry) -> SharedFileReadHistory {
    register_all_tools(registry, ToolRegistrationConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_register_default_tools() {
        let mut registry = ToolRegistry::new();
        let _history = register_default_tools(&mut registry);

        // Verify core tools are registered
        assert!(registry.contains("bash"));
        assert!(registry.contains("read"));
        assert!(registry.contains("write"));
        assert!(registry.contains("edit"));
        assert!(registry.contains("glob"));
        assert!(registry.contains("grep"));

        // AskTool and LSPTool should not be registered without callbacks
        assert!(!registry.contains("ask"));
        assert!(!registry.contains("lsp"));
    }

    #[test]
    fn test_register_all_tools_with_config() {
        use std::pin::Pin;
        use std::future::Future;
        use std::sync::Arc;

        let mut registry = ToolRegistry::new();

        // Create mock callbacks
        let ask_callback: AskCallback = Arc::new(|_question, _options| {
            Box::pin(async { Some("test response".to_string()) })
                as Pin<Box<dyn Future<Output = Option<String>> + Send>>
        });

        let lsp_callback: LspCallback = Arc::new(|_operation, _path: PathBuf, _position| {
            Box::pin(async { Ok(LspResult::Definition { locations: vec![] }) })
                as Pin<Box<dyn Future<Output = Result<LspResult, String>> + Send>>
        });

        let config = ToolRegistrationConfig::new()
            .with_ask_callback(ask_callback)
            .with_lsp_callback(lsp_callback)
            .with_pdf_enabled(true);

        let _history = register_all_tools(&mut registry, config);

        // Verify all tools are registered
        assert!(registry.contains("bash"));
        assert!(registry.contains("read"));
        assert!(registry.contains("write"));
        assert!(registry.contains("edit"));
        assert!(registry.contains("glob"));
        assert!(registry.contains("grep"));
        assert!(registry.contains("ask"));
        assert!(registry.contains("lsp"));
    }

    #[test]
    fn test_shared_history_is_shared() {
        let mut registry = ToolRegistry::new();
        let history = register_default_tools(&mut registry);

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
        assert!(history.read().unwrap().has_read(&std::path::PathBuf::from("/tmp/test.txt")));
    }

    #[test]
    fn test_tool_registration_config_builder() {
        let config = ToolRegistrationConfig::new()
            .with_pdf_enabled(true);

        assert!(config.pdf_enabled);
        assert!(config.ask_callback.is_none());
        assert!(config.lsp_callback.is_none());
    }
}
