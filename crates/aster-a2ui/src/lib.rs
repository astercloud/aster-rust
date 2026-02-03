//! # aster-a2ui
//!
//! A2UI (Agent-to-User Interface) 协议实现
//!
//! 基于 Google A2UI v0.10 规范，提供：
//! - 协议消息类型定义
//! - 组件目录（Standard Catalog）
//! - 客户端函数定义
//! - JSON Schema 验证
//! - 流式消息构建器
//!
//! ## 快速开始
//!
//! ```rust
//! use aster_a2ui::prelude::*;
//!
//! // 创建一个 Surface
//! let msg = ServerMessage::create_surface(
//!     "contact_form",
//!     STANDARD_CATALOG_ID,
//! );
//! ```

pub mod catalog;
pub mod common;
pub mod functions;
pub mod protocol;
pub mod validation;

pub mod prelude {
    //! 常用类型的便捷导入
    pub use crate::catalog::*;
    pub use crate::common::*;
    pub use crate::functions::*;
    pub use crate::protocol::*;
}
