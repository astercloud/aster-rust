//! 沙箱模块
//!
//! 提供进程隔离、文件系统沙箱、网络沙箱等功能

mod config;
mod executor;
mod filesystem;
mod resource_limits;

pub use config::{
    SandboxConfig, SandboxPreset, ResourceLimits,
    SandboxConfigManager, SANDBOX_PRESETS,
};
pub use executor::{
    SandboxExecutor, ExecutorResult, ExecutorOptions,
    execute_in_sandbox, detect_best_sandbox, get_sandbox_capabilities,
};
pub use filesystem::{
    FilesystemSandbox, FilesystemPolicy, PathRule,
};
pub use resource_limits::{
    ResourceLimiter, ResourceUsage,
};
