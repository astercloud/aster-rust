//! 遥测系统
//!
//! 跟踪使用统计和事件（本地存储，支持批量上报）

mod types;
mod tracker;
mod config;
mod sanitizer;

pub use types::*;
pub use tracker::*;
pub use config::*;
pub use sanitizer::*;

#[cfg(test)]
mod tests;
