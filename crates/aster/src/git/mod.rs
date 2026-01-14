//! Git 工具模块
//!
//! 提供 Git 状态检测、分支信息、安全检查等功能

mod core;
mod safety;

pub use core::{
    GitUtils, GitStatus, GitInfo, PushStatus,
    is_git_repository, get_git_status, get_git_info,
    get_current_branch, get_default_branch,
};
pub use safety::{
    GitSafety, SafetyCheckResult, SensitiveFilesCheck, is_dangerous_command,
};
