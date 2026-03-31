//! 应用状态管理

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::{Mutex, RwLock};

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
    /// 本地启动的 asterd 进程句柄
    pub server_process: Arc<Mutex<Option<Child>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            server_status: Arc::new(RwLock::new(ServerStatus::Stopped)),
            current_session: Arc::new(RwLock::new(None)),
            server_port: Arc::new(RwLock::new(3000)),
            server_process: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
