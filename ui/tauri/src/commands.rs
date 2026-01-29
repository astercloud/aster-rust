//! Tauri 命令定义
//!
//! 提供前端调用的 Tauri 命令

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::{AppState, ServerStatus};

/// 配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub working_dir: String,
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}


/// Provider 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub models: Vec<String>,
}

/// 扩展信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

// ============================================================================
// 配置命令
// ============================================================================

#[tauri::command]
pub async fn get_config(key: String) -> Result<serde_json::Value, String> {
    // TODO: 调用 aster 核心库获取配置
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn set_config(key: String, value: serde_json::Value) -> Result<(), String> {
    // TODO: 调用 aster 核心库设置配置
    Ok(())
}


// ============================================================================
// 会话命令
// ============================================================================

#[tauri::command]
pub async fn start_session(
    state: State<'_, AppState>,
    name: String,
    working_dir: String,
) -> Result<SessionInfo, String> {
    // TODO: 调用 aster 核心库创建会话
    Ok(SessionInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        created_at: chrono::Utc::now().to_rfc3339(),
        working_dir,
    })
}

#[tauri::command]
pub async fn stop_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    // TODO: 调用 aster 核心库停止会话
    Ok(())
}

#[tauri::command]
pub async fn send_message(session_id: String, content: String) -> Result<Message, String> {
    // TODO: 调用 aster 核心库发送消息
    Ok(Message {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}


#[tauri::command]
pub async fn get_sessions() -> Result<Vec<SessionInfo>, String> {
    // TODO: 调用 aster 核心库获取会话列表
    Ok(vec![])
}

#[tauri::command]
pub async fn get_session_messages(session_id: String) -> Result<Vec<Message>, String> {
    // TODO: 调用 aster 核心库获取会话消息
    Ok(vec![])
}

// ============================================================================
// Provider 命令
// ============================================================================

#[tauri::command]
pub async fn get_providers() -> Result<Vec<ProviderInfo>, String> {
    // TODO: 调用 aster 核心库获取 Provider 列表
    Ok(vec![
        ProviderInfo {
            name: "anthropic".to_string(),
            display_name: "Anthropic".to_string(),
            models: vec!["claude-sonnet-4-20250514".to_string()],
        },
        ProviderInfo {
            name: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            models: vec!["gpt-4o".to_string()],
        },
    ])
}


// ============================================================================
// 扩展命令
// ============================================================================

#[tauri::command]
pub async fn get_extensions() -> Result<Vec<ExtensionInfo>, String> {
    // TODO: 调用 aster 核心库获取扩展列表
    Ok(vec![])
}

#[tauri::command]
pub async fn install_extension(name: String) -> Result<ExtensionInfo, String> {
    // TODO: 调用 aster 核心库安装扩展
    Ok(ExtensionInfo {
        name,
        version: "1.0.0".to_string(),
        enabled: true,
    })
}

#[tauri::command]
pub async fn uninstall_extension(name: String) -> Result<(), String> {
    // TODO: 调用 aster 核心库卸载扩展
    Ok(())
}


// ============================================================================
// 服务器命令
// ============================================================================

#[tauri::command]
pub async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    let status = state.server_status.read().await;
    Ok(status.clone())
}

#[tauri::command]
pub async fn start_server(state: State<'_, AppState>, port: Option<u16>) -> Result<(), String> {
    let port = port.unwrap_or(3000);
    
    {
        let mut status = state.server_status.write().await;
        *status = ServerStatus::Starting;
    }
    
    // TODO: 启动 asterd 服务器
    // 可以通过 tauri_plugin_shell 启动子进程
    
    {
        let mut status = state.server_status.write().await;
        *status = ServerStatus::Running;
        let mut server_port = state.server_port.write().await;
        *server_port = port;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    // TODO: 停止 asterd 服务器
    
    let mut status = state.server_status.write().await;
    *status = ServerStatus::Stopped;
    
    Ok(())
}
