//! Rewind 功能模块
//!
//! 提供对话和文件状态的回退功能

mod file_history;
mod manager;

pub use file_history::{
    FileHistoryManager, FileBackup, FileSnapshot, RewindResult,
};
pub use manager::{
    RewindManager, RewindOption, RewindableMessage, RewindOperationResult,
};
