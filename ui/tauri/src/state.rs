//! 应用状态管理

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

/// 应用状态
pub struct AppState {
    /// 服务器状态
    pub server_status: Arc<RwLock<ServerStatus>>,
    /// 当前会话 ID
    pub current_session: Arc<RwLock<Option<String>>>,
    /// 服务器端口
    pub server_port: Arc<RwLock<u16>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            server_status: Arc::new(RwLock::new(ServerStatus::Stopped)),
            current_session: Arc::new(RwLock::new(None)),
            server_port: Arc::new(RwLock::new(3000)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
