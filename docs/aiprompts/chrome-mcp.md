# Chrome MCP 集成

与官方 Claude Code Chrome 扩展集成。

## 模块结构

```
chrome_mcp/
├── mcp_server.rs     # MCP 服务器
├── native_host.rs    # Native Host 管理
├── socket_client.rs  # Socket 客户端
├── socket_server.rs  # Socket 服务器
├── tools.rs          # MCP 工具定义
└── types.rs          # 类型定义
```

## 核心功能

### Native Host 管理
```rust
pub fn setup_chrome_native_host() -> SetupResult;
pub fn uninstall_chrome_native_host();
pub fn is_chrome_integration_configured() -> bool;
pub fn is_chrome_integration_supported() -> bool;
pub fn get_native_hosts_directory() -> PathBuf;
```


### Socket 通信
```rust
pub struct SocketClient;
pub struct SocketServer;

pub fn create_socket_client() -> SocketClient;
pub fn run_native_host();
```

### MCP 服务器
```rust
pub struct McpServer;
pub fn get_chrome_mcp_tools() -> Vec<McpTool>;
```

## 配置

```rust
pub struct ChromeIntegrationConfig {
    pub enabled: bool,
    pub socket_path: PathBuf,
}

pub struct McpServerConfig {
    pub name: String,
    pub version: String,
}
```

## 使用场景

- 浏览器自动化
- 网页内容提取
- Chrome 扩展集成

## 源码位置

`crates/aster/src/chrome_mcp/`
