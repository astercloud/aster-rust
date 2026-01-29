# OAuth 认证

MCP 服务器 OAuth 认证流程。

## 核心功能

### oauth_flow
```rust
pub async fn oauth_flow(
    mcp_server_url: &String,
    name: &String,
) -> Result<AuthorizationManager>;
```

## 认证流程

1. 检查已存储的凭证
2. 尝试刷新 token
3. 如果失败，启动完整 OAuth 流程
4. 启动本地回调服务器
5. 打开浏览器进行授权
6. 接收回调并交换 token
7. 存储凭证

## 凭证存储

```rust
struct AsterCredentialStore {
    name: String,
}

impl CredentialStore for AsterCredentialStore {
    async fn save(credentials: StoredCredentials);
    async fn load() -> Option<StoredCredentials>;
    async fn clear();
}
```


## 回调服务器

- 监听 `127.0.0.1` 随机端口
- 路由: `/oauth_callback`
- 接收 `code` 和 `state` 参数

## 使用场景

- MCP 服务器需要 OAuth 认证
- 第三方 API 集成

## 源码位置

`crates/aster/src/oauth/`
