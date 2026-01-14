//! 自动更新系统
//!
//! 提供版本检查、下载、安装和回滚功能

mod checker;
mod installer;
mod manager;

pub use checker::{
    VersionInfo, UpdateCheckResult, check_for_updates as check_version, compare_versions,
};
pub use installer::{
    Installer, InstallOptions, InstallResult, DownloadProgress, DownloadPhase,
};
pub use manager::{
    UpdateManager, UpdateConfig, UpdateChannel, UpdateStatus, UpdateOptions, UpdateEvent,
    check_for_updates, perform_update, rollback_version, list_versions,
};
