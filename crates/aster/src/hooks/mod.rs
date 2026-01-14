//! Hooks 系统
//!
//! 支持在工具调用前后执行自定义脚本或回调

mod executor;
mod loader;
mod registry;
mod types;

pub use executor::*;
pub use loader::*;
pub use registry::*;
pub use types::*;

#[cfg(test)]
mod tests;
