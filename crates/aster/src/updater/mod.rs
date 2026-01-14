//! 自动更新系统
//!
//! 提供版本检查和更新功能

mod checker;
mod manager;

pub use checker::{
    VersionInfo, UpdateCheckResult, check_for_updates, compare_versions,
};
pub use manager::{
    UpdateManager, UpdateConfig, UpdateStatus, UpdateOptions,
};
