//! 插件系统模块
//!
//! 提供插件加载、管理、生命周期控制等功能

mod types;
mod context;
mod manager;

pub use types::{
    PluginMetadata, PluginState, PluginConfig,
    CommandDefinition, SkillDefinition, HookDefinition,
    PluginHookType, Plugin,
};
pub use context::{PluginContext, PluginLogger, PluginConfigAPI};
pub use manager::PluginManager;
