# Teleport 远程会话

提供远程会话连接、同步、仓库验证等功能。

## 模块结构

```
teleport/
├── connection.rs  # WebSocket 连接
├── session.rs     # 远程会话
├── types.rs       # 类型定义
└── validation.rs  # 仓库验证
```

## 核心功能

### 连接管理
```rust
pub struct WebSocketManager;
pub struct ConnectionConfig {
    pub url: String,
    pub reconnect: bool,
    pub heartbeat_interval: Duration,
}

pub async fn connect_to_remote_session(config: ConnectionConfig);
pub fn can_teleport_to_session(session_id: &str) -> bool;
```

### 远程会话
```rust
pub struct RemoteSession;
pub async fn create_remote_session() -> RemoteSession;
```


## 仓库验证

```rust
pub fn validate_session_repository() -> RepoValidationResult;
pub fn get_current_repo_url() -> Option<String>;
pub fn get_current_branch() -> Option<String>;
pub fn is_working_directory_clean() -> bool;
pub fn normalize_repo_url(url: &str) -> String;
pub fn compare_repo_urls(a: &str, b: &str) -> bool;
```

## 状态类型

```rust
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

pub enum SyncState {
    Synced,
    Syncing,
    OutOfSync,
}
```

## 使用场景

- 多设备会话同步
- 远程协作
- 会话恢复

## 源码位置

`crates/aster/src/teleport/`
