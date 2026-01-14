//! Hooks 系统
//!
//! 支持在工具调用前后执行自定义脚本或回调

mod types;
mod executor;
mod registry;
mod loader;

pub use types::*;
pub use executor::*;
pub use registry::*;
pub use loader::*;

#[cfg(test)]
mod tests;
