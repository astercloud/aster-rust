//! Teleport 类型定义
//!
//! 远程会话连接的数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 远程会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportConfig {
    /// 会话 ID
    pub session_id: String,
    /// 远程服务器 URL (WebSocket)
    pub ingress_url: Option<String>,
    /// 认证令牌
    pub auth_token: Option<String>,
    /// 会话元数据
    pub metadata: Option<TeleportMetadata>,
}

/// 会话元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeleportMetadata {
    /// 仓库
    pub repo: Option<String>,
    /// 分支
    pub branch: Option<String>,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}


/// 仓库验证状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoValidationStatus {
    /// 仓库匹配
    Match,
    /// 仓库不匹配
    Mismatch,
    /// 不需要验证
    NoValidation,
    /// 验证错误
    Error,
}

/// 仓库验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoValidationResult {
    /// 验证状态
    pub status: RepoValidationStatus,
    /// 会话仓库
    pub session_repo: Option<String>,
    /// 当前仓库
    pub current_repo: Option<String>,
    /// 错误消息
    pub error_message: Option<String>,
}

/// 远程消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteMessageType {
    /// 同步请求
    SyncRequest,
    /// 同步响应
    SyncResponse,
    /// 用户消息
    Message,
    /// 助手消息
    AssistantMessage,
    /// 工具执行结果
    ToolResult,
    /// 心跳
    Heartbeat,
    /// 错误
    Error,
}

/// 远程消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMessage {
    /// 消息类型
    pub message_type: RemoteMessageType,
    /// 消息 ID
    pub id: Option<String>,
    /// 会话 ID
    pub session_id: String,
    /// 消息内容
    pub payload: serde_json::Value,
    /// 时间戳
    pub timestamp: String,
}


/// 同步状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncState {
    /// 是否正在同步
    pub syncing: bool,
    /// 最后同步时间
    pub last_sync_time: Option<String>,
    /// 同步的消息数量
    pub synced_messages: u32,
    /// 同步错误
    pub sync_error: Option<String>,
}

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    /// 未连接
    #[default]
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 同步中
    Syncing,
    /// 错误
    Error,
}

/// 远程会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSessionState {
    /// 连接状态
    pub connection_state: ConnectionState,
    /// 同步状态
    pub sync_state: SyncState,
    /// 会话配置
    pub config: TeleportConfig,
    /// 错误信息
    pub error: Option<String>,
}
