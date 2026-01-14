//! 可视化服务器模块
//!
//! 提供代码本体图谱的交互式可视化 Web 服务器
//!
//! ## 模块结构
//! - `types`: 可视化相关类型定义
//! - `server`: HTTP 服务器实现
//! - `routes`: API 路由处理
//! - `services`: 业务逻辑服务

pub mod types;
pub mod server;
pub mod routes;
pub mod services;

// 类型导出
pub use types::*;

// 服务器导出
pub use server::{VisualizationServer, VisualizationServerOptions, start_visualization_server};

// 服务导出
pub use services::{
    architecture::{build_architecture_map, get_module_detail, get_symbol_refs, get_dir},
    dependency::{detect_entry_points, build_dependency_tree},
};
