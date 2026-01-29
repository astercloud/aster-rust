# MCP 协议集成

## 概述

MCP (Model Context Protocol) 模块提供了与 MCP 服务器的完整集成能力。

**核心路径**: `crates/aster/src/mcp/`

## 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        Agent / CLI                               │
├─────────────────────────────────────────────────────────────────┤
│                     ExtensionManager                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ McpConnection   │  │ McpLifecycle    │  │ McpConfig       │  │
│  │ Manager         │  │ Manager         │  │ Manager         │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
│           │                    │                    │           │
│  ┌────────┴────────────────────┴────────────────────┴────────┐  │
│  │                     McpToolManager                         │  │
│  └────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                        Transport Layer                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │  Stdio   │  │   HTTP   │  │   SSE    │  │    WebSocket     │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. 连接管理 (ConnectionManager)

```rust
// crates/aster/src/mcp/connection_manager.rs
pub trait ConnectionManager: Send + Sync {
    async fn connect(&self, config: &McpServerConfig) -> McpResult<McpConnection>;
    async fn disconnect(&self, server_name: &str) -> McpResult<()>;
    async fn reconnect(&self, server_name: &str) -> McpResult<McpConnection>;
    async fn get_connection(&self, server_name: &str) -> Option<McpConnection>;
    async fn health_check(&self, server_name: &str) -> McpResult<HealthCheckResult>;
}
```

### 2. 生命周期管理 (LifecycleManager)

```rust
// crates/aster/src/mcp/lifecycle_manager.rs
pub trait LifecycleManager: Send + Sync {
    async fn start_server(&self, config: &McpServerConfig, options: StartOptions) 
        -> McpResult<ServerProcess>;
    async fn stop_server(&self, server_name: &str, options: StopOptions) -> McpResult<()>;
    async fn restart_server(&self, server_name: &str) -> McpResult<ServerProcess>;
    async fn get_server_state(&self, server_name: &str) -> Option<ServerState>;
}
```

### 3. 配置管理 (ConfigManager)

```rust
// crates/aster/src/mcp/config_manager.rs
pub trait ConfigManager: Send + Sync {
    async fn load_config(&self, scope: ConfigScope) -> McpResult<McpConfigFile>;
    async fn save_config(&self, scope: ConfigScope, config: &McpConfigFile) -> McpResult<()>;
    async fn validate_config(&self, config: &McpServerConfig) -> ValidationResult;
    fn watch_config(&self, callback: ConfigChangeCallback);
}
```

### 4. 工具管理 (ToolManager)

```rust
// crates/aster/src/mcp/tool_manager.rs
pub trait ToolManager: Send + Sync {
    async fn list_tools(&self, server_name: &str) -> McpResult<Vec<McpTool>>;
    async fn call_tool(&self, server_name: &str, call: ToolCall) -> McpResult<ToolCallResult>;
    async fn validate_args(&self, tool: &McpTool, args: &JsonObject) -> ArgValidationResult;
    async fn refresh_tools(&self, server_name: &str) -> McpResult<Vec<McpTool>>;
}
```

## 传输层

### 支持的传输类型

```rust
pub enum TransportType {
    Stdio,      // 标准输入输出
    Http,       // HTTP 请求
    Sse,        // Server-Sent Events
    WebSocket,  // WebSocket 连接
}
```

### 传输配置

```rust
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub timeout: Duration,
    pub retry_config: RetryConfig,
    pub headers: HashMap<String, String>,
}
```

## 服务器配置

```rust
pub struct McpServerConfig {
    pub transport_type: TransportType,
    pub command: Option<String>,        // Stdio 命令
    pub args: Option<Vec<String>>,      // 命令参数
    pub env: Option<HashMap<String, String>>,  // 环境变量
    pub url: Option<String>,            // HTTP/WS URL
    pub enabled: bool,
    pub auto_restart: bool,
    pub health_check_interval: Option<Duration>,
}
```

## 配置文件示例

```json
// .aster/mcp.json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"],
      "enabled": true
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

## 错误处理

```rust
pub enum McpError {
    ConnectionFailed(String),
    ServerNotFound(String),
    ToolNotFound(String),
    InvalidArguments(String),
    Timeout(Duration),
    TransportError(String),
    ProtocolError(String),
}
```

## 取消机制

```rust
// crates/aster/src/mcp/cancellation.rs
pub struct McpCancellationManager {
    // 支持请求级别的取消
    pub async fn cancel_request(&self, request_id: &str) -> CancellationResult;
    pub async fn cancel_all(&self, server_name: &str) -> CancellationResult;
}
```

## 通知系统

```rust
// crates/aster/src/mcp/notifications.rs
pub enum NotificationType {
    Progress(ProgressNotification),
    Log(McpLogEntry),
    ResourceChanged(String),
    ToolListChanged,
}
```
