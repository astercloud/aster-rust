//! A2UI 协议消息类型
//!
//! 定义服务端到客户端和客户端到服务端的消息格式

use serde::{Deserialize, Serialize};

use crate::catalog::Component;

/// A2UI 协议版本
pub const PROTOCOL_VERSION: &str = "v0.10";

// ============================================================================
// 服务端到客户端消息
// ============================================================================

/// 服务端到客户端的消息信封
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerMessage {
    /// 协议版本
    pub version: String,
    /// 消息内容（四选一）
    #[serde(flatten)]
    pub content: ServerMessageContent,
}

/// 服务端消息内容
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessageContent {
    /// 创建 Surface
    CreateSurface(CreateSurface),
    /// 更新组件
    UpdateComponents(UpdateComponents),
    /// 更新数据模型
    UpdateDataModel(UpdateDataModel),
    /// 删除 Surface
    DeleteSurface(DeleteSurface),
}

/// 创建 Surface 消息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurface {
    /// Surface 唯一标识符
    pub surface_id: String,
    /// 组件目录 ID
    pub catalog_id: String,
    /// 主题配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    /// 是否发送数据模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_data_model: Option<bool>,
}

/// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    /// 主色调
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_color: Option<String>,
    /// 图标 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Agent 显示名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_display_name: Option<String>,
}

/// 更新组件消息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponents {
    /// Surface ID
    pub surface_id: String,
    /// 组件列表
    pub components: Vec<Component>,
}

/// 更新数据模型消息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModel {
    /// Surface ID
    pub surface_id: String,
    /// JSON Pointer 路径（默认为 "/"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 新值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// 删除 Surface 消息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    /// Surface ID
    pub surface_id: String,
}

// ============================================================================
// 客户端到服务端消息
// ============================================================================

/// 客户端到服务端的消息信封
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientMessage {
    /// 协议版本
    pub version: String,
    /// 消息内容
    #[serde(flatten)]
    pub content: ClientMessageContent,
}

/// 客户端消息内容
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessageContent {
    /// 动作事件
    Action(ActionMessage),
    /// 错误消息
    Error(ErrorMessage),
}

/// 动作消息（用户交互触发）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionMessage {
    /// 事件名称
    pub name: String,
    /// Surface ID
    pub surface_id: String,
    /// 触发事件的组件 ID
    pub source_component_id: String,
    /// ISO 8601 时间戳
    pub timestamp: String,
    /// 事件上下文
    pub context: serde_json::Map<String, serde_json::Value>,
}

/// 错误消息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    /// 错误代码
    pub code: ErrorCode,
    /// Surface ID
    pub surface_id: String,
    /// 错误消息
    pub message: String,
    /// JSON Pointer 路径（仅 VALIDATION_FAILED）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// 错误代码
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCode {
    /// 验证失败
    #[serde(rename = "VALIDATION_FAILED")]
    ValidationFailed,
    /// 其他错误
    #[serde(other)]
    Other,
}

/// 客户端能力声明（通过 Transport metadata 发送）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// 支持的组件目录 ID 列表
    pub supported_catalog_ids: Vec<String>,
    /// 内联目录定义（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_catalogs: Option<Vec<Catalog>>,
}

/// 目录定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    /// 目录唯一标识符
    pub catalog_id: String,
    /// 组件定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<serde_json::Map<String, serde_json::Value>>,
    /// 函数定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDefinition>>,
    /// 主题定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<serde_json::Map<String, serde_json::Value>>,
}

/// 函数定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDefinition {
    /// 函数名称
    pub name: String,
    /// 函数描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 参数 JSON Schema
    pub parameters: serde_json::Value,
    /// 返回类型
    pub return_type: String,
}

/// 客户端数据模型（通过 Transport metadata 发送）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientDataModel {
    /// 协议版本
    pub version: String,
    /// Surface ID 到数据模型的映射
    pub surfaces: std::collections::HashMap<String, serde_json::Value>,
}

// ============================================================================
// 构建器模式
// ============================================================================

impl ServerMessage {
    /// 创建新的服务端消息
    pub fn new(content: ServerMessageContent) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            content,
        }
    }

    /// 创建 CreateSurface 消息
    pub fn create_surface(surface_id: &str, catalog_id: &str) -> Self {
        Self::new(ServerMessageContent::CreateSurface(CreateSurface {
            surface_id: surface_id.to_string(),
            catalog_id: catalog_id.to_string(),
            theme: None,
            send_data_model: None,
        }))
    }

    /// 创建 UpdateComponents 消息
    pub fn update_components(surface_id: &str, components: Vec<Component>) -> Self {
        Self::new(ServerMessageContent::UpdateComponents(UpdateComponents {
            surface_id: surface_id.to_string(),
            components,
        }))
    }

    /// 创建 UpdateDataModel 消息
    pub fn update_data_model(surface_id: &str, value: serde_json::Value) -> Self {
        Self::new(ServerMessageContent::UpdateDataModel(UpdateDataModel {
            surface_id: surface_id.to_string(),
            path: None,
            value: Some(value),
        }))
    }

    /// 创建 DeleteSurface 消息
    pub fn delete_surface(surface_id: &str) -> Self {
        Self::new(ServerMessageContent::DeleteSurface(DeleteSurface {
            surface_id: surface_id.to_string(),
        }))
    }
}

impl ClientMessage {
    /// 创建新的客户端消息
    pub fn new(content: ClientMessageContent) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            content,
        }
    }

    /// 创建动作消息
    pub fn action(
        surface_id: &str,
        name: &str,
        source_component_id: &str,
        context: serde_json::Map<String, serde_json::Value>,
    ) -> Self {
        Self::new(ClientMessageContent::Action(ActionMessage {
            name: name.to_string(),
            surface_id: surface_id.to_string(),
            source_component_id: source_component_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            context,
        }))
    }

    /// 创建验证失败错误消息
    pub fn validation_error(surface_id: &str, path: &str, message: &str) -> Self {
        Self::new(ClientMessageContent::Error(ErrorMessage {
            code: ErrorCode::ValidationFailed,
            surface_id: surface_id.to_string(),
            message: message.to_string(),
            path: Some(path.to_string()),
        }))
    }

    /// 创建通用错误消息
    pub fn error(surface_id: &str, message: &str) -> Self {
        Self::new(ClientMessageContent::Error(ErrorMessage {
            code: ErrorCode::Other,
            surface_id: surface_id.to_string(),
            message: message.to_string(),
            path: None,
        }))
    }
}

impl ClientCapabilities {
    /// 创建新的客户端能力声明
    pub fn new(supported_catalog_ids: Vec<String>) -> Self {
        Self {
            supported_catalog_ids,
            inline_catalogs: None,
        }
    }

    /// 添加内联目录
    pub fn with_inline_catalog(mut self, catalog: Catalog) -> Self {
        self.inline_catalogs
            .get_or_insert_with(Vec::new)
            .push(catalog);
        self
    }
}

impl ClientDataModel {
    /// 创建新的客户端数据模型
    pub fn new() -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            surfaces: std::collections::HashMap::new(),
        }
    }

    /// 添加 Surface 数据模型
    pub fn with_surface(mut self, surface_id: &str, data: serde_json::Value) -> Self {
        self.surfaces.insert(surface_id.to_string(), data);
        self
    }
}

impl Default for ClientDataModel {
    fn default() -> Self {
        Self::new()
    }
}
