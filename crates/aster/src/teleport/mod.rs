//! Teleport 模块
//!
//! 提供远程会话连接、同步、仓库验证等功能

mod types;
mod session;
mod validation;

pub use types::{
    TeleportConfig, RepoValidationResult, RepoValidationStatus,
    RemoteMessage, RemoteMessageType, RemoteSessionState,
    ConnectionState, SyncState,
};
pub use session::{RemoteSession, create_remote_session};
pub use validation::{
    validate_session_repository, get_current_repo_url,
    normalize_repo_url, compare_repo_urls,
    get_current_branch, is_working_directory_clean,
};
