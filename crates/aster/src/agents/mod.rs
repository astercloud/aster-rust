mod agent;
pub(crate) mod chatrecall_extension;
pub(crate) mod code_execution_extension;
pub mod execute_commands;
pub mod extension;
pub mod extension_malware_check;
pub mod extension_manager;
pub mod extension_manager_extension;
pub mod final_output_tool;
mod large_response_handler;
pub mod mcp_client;
pub mod moim;
pub mod platform_tools;
pub mod prompt_manager;
mod reply_parts;
pub mod retry;
mod schedule_tool;
pub(crate) mod skills_extension;
pub mod subagent_execution_tool;
pub mod subagent_handler;
mod subagent_task_config;
pub mod subagent_tool;
pub(crate) mod todo_extension;
mod tool_execution;
pub mod types;

// ============================================================================
// ============================================================================

/// Agent context management module
///
/// Provides context creation, inheritance, compression, filtering,
/// persistence, and isolation capabilities for agents.
pub mod context;

/// Agent communication module
///
/// Provides inter-agent communication including message bus,
/// shared state management, and agent coordination.
pub mod communication;

/// Parallel execution module
///
/// Provides parallel agent execution with dependency management,
/// retry logic, and agent resource pooling.
pub mod parallel;

/// Agent monitoring module
///
/// Provides metrics collection, alert management, and
/// performance analysis for agent execution.
pub mod monitor;

/// Agent resume module
///
/// Provides state persistence, checkpoint management,
/// and agent resume capabilities.
pub mod resume;

/// Specialized agents module
///
/// Provides specialized agent implementations including
/// Explore agent and Plan agent.
pub mod specialized;

/// Unified error handling module
///
/// Provides comprehensive error handling including error recording,
/// timeout handling, and retry mechanisms.
pub mod error_handling;

// ============================================================================
// Core Agent Exports
// ============================================================================

pub use agent::{Agent, AgentEvent};
pub use execute_commands::COMPACT_TRIGGERS;
pub use extension::ExtensionConfig;
pub use extension_manager::ExtensionManager;
pub use prompt_manager::PromptManager;
pub use subagent_task_config::TaskConfig;
pub use types::{FrontendTool, RetryConfig, SessionConfig, SuccessCheck};

// ============================================================================
// Context Module Re-exports
// ============================================================================

pub use context::{
    // Core context types
    AgentContext,
    AgentContextError,
    AgentContextResult,
    ContextMetadata,
    FileContext,
    ToolExecutionResult,
    // Context manager
    AgentContextManager,
    // Context inheritance
    ContextInheritanceConfig,
    ContextInheritanceType,
    // Context operations
    CompressionResult,
    ContextFilter,
    ContextUpdate,
    // Context isolation
    ContextIsolation,
    SandboxedContext,
    SandboxRestrictions,
    SandboxState,
    ResourceUsage,
};

// ============================================================================
// Communication Module Re-exports
// ============================================================================

pub use communication::{
    // Message bus
    AgentMessage,
    AgentMessageBus,
    MessageBusError,
    MessageBusResult,
    MessageBusStats,
    MessagePriority,
    MessageSubscription,
    MessageTarget,
    // Shared state
    Lock,
    SharedStateError,
    SharedStateManager,
    SharedStateResult,
    SharedStateStats,
    StateEvent,
    WatchHandle,
    // Coordinator
    AgentCapabilities,
    AgentCoordinator,
    AgentStatus,
    AssignmentCriteria,
    CoordinatorError,
    CoordinatorEvent,
    CoordinatorResult,
    CoordinatorStats,
    DeadlockInfo,
    DependencyLink,
    LoadBalanceStrategy,
    Task,
    TaskResult,
    TaskStatus as CoordinatorTaskStatus,
};

// ============================================================================
// Parallel Module Re-exports
// ============================================================================

pub use parallel::{
    // Executor
    AgentResult,
    AgentTask,
    DependencyGraph,
    ExecutionProgress,
    ExecutorError,
    ExecutorResult,
    MergedResult,
    ParallelAgentConfig,
    ParallelAgentExecutor,
    ParallelExecutionResult,
    TaskExecutionInfo,
    TaskStatus as ExecutorTaskStatus,
    // Pool
    AgentPool,
    AgentWorker,
    PoolError,
    PoolResult,
    PoolStatus,
};

// ============================================================================
// Monitor Module Re-exports
// ============================================================================

pub use monitor::{
    // Metrics
    AgentMonitor,
    AggregatedStats,
    FullAgentMetrics,
    MonitorConfig,
    PerformanceMetrics,
    ToolCallMetric,
    // Alerts
    Alert,
    AlertManager,
    AlertSeverity,
    AlertType,
    AgentExecutionStatus,
    ErrorRecord,
    TokenUsage,
    // Analyzer
    AnalysisThresholds,
    Bottleneck,
    BottleneckCategory,
    PerformanceAnalyzer,
    PerformanceRating,
    PerformanceReport,
    PerformanceScores,
    Suggestion,
    SuggestionPriority,
};

// ============================================================================
// Resume Module Re-exports
// ============================================================================

pub use resume::{
    // State manager
    AgentState,
    AgentStateManager,
    AgentStateStatus,
    Checkpoint,
    StateManagerError,
    StateManagerResult,
    ToolCallRecord,
    // Resumer
    AgentResumer,
    ResumeOptions,
    ResumePoint,
    ResumePointInfo,
    ResumerError,
    ResumerResult,
};

// ============================================================================
// Specialized Module Re-exports
// ============================================================================

pub use specialized::{
    // Explore agent
    CodeSnippet,
    ExploreAgent,
    ExploreError,
    ExploreOptions,
    ExploreResult,
    ExploreResultData,
    ExploreStats,
    StructureAnalysis,
    ThoroughnessLevel,
    // Plan agent
    Alternative,
    ArchitecturalDecision,
    Complexity,
    CriticalFile,
    ModificationType,
    PlanAgent,
    PlanError,
    PlanOptions,
    PlanResult,
    PlanResultData,
    PlanStep,
    RequirementsAnalysis,
    Risk,
    RiskCategory,
    RiskSeverity,
    ScopeDefinition,
};

// ============================================================================
// Error Handling Module Re-exports
// ============================================================================

pub use error_handling::{
    // Error handler
    AgentError,
    AgentErrorKind,
    ErrorContext,
    ErrorHandler,
    UnifiedErrorRecord,
    // Timeout handler
    TimeoutConfig,
    TimeoutEvent,
    TimeoutHandler,
    TimeoutStatus,
    // Retry handler
    RetryHandler,
    RetryResult,
    RetryStrategy,
    UnifiedRetryConfig,
};
