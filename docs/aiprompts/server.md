# Aster Server (asterd)

## 概述

asterd 是 Aster 的后端服务，为桌面应用和 Web 界面提供 API。

**入口**: `crates/aster-server/src/main.rs`

## 命令

```bash
# 运行 Agent 服务器
asterd agent

# 运行 MCP 服务器
asterd mcp developer
asterd mcp memory
asterd mcp tutorial
asterd mcp computer-controller
asterd mcp auto-visualiser
```

## 模块结构

```
crates/aster-server/src/
├── main.rs           # 入口点
├── commands/         # 命令处理
│   └── agent.rs      # Agent 服务器
├── routes/           # API 路由
├── state.rs          # 应用状态
├── configuration.rs  # 配置
├── error.rs          # 错误处理
├── logging.rs        # 日志
├── openapi.rs        # OpenAPI 文档
└── tunnel.rs         # 隧道支持
```

## Agent 服务器

启动 HTTP API 服务：

```bash
asterd agent
```

默认监听 `127.0.0.1:3000`

## API 端点

### 会话管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/sessions` | 列出会话 |
| POST | `/sessions` | 创建会话 |
| GET | `/sessions/:id` | 获取会话 |
| DELETE | `/sessions/:id` | 删除会话 |
| POST | `/sessions/:id/messages` | 发送消息 |

### 配置

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/config` | 获取配置 |
| PUT | `/config` | 更新配置 |

### 扩展

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/extensions` | 列出扩展 |
| POST | `/extensions` | 添加扩展 |
| DELETE | `/extensions/:name` | 移除扩展 |

## 内置 MCP 服务器

### Developer

开发者工具，提供文件操作、Shell 命令等：

```bash
asterd mcp developer
```

### Memory

记忆服务，提供长期记忆存储：

```bash
asterd mcp memory
```

### Tutorial

教程服务：

```bash
asterd mcp tutorial
```

### Computer Controller

计算机控制，提供屏幕截图、鼠标键盘操作：

```bash
asterd mcp computer-controller
```

### Auto Visualiser

自动可视化：

```bash
asterd mcp auto-visualiser
```

## 配置

```toml
# 服务器配置
[server]
host = "127.0.0.1"
port = 3000

# 认证
[auth]
token = "your-secret-token"

# 日志
[logging]
level = "info"
```

## 与 CLI 的关系

```
┌─────────────────────────────────────────┐
│              用户界面                    │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │   CLI       │  │   Desktop App   │   │
│  │  (aster)    │  │   (Electron)    │   │
│  └──────┬──────┘  └────────┬────────┘   │
│         │                  │            │
│    直接调用            HTTP API         │
│         │                  │            │
│  ┌──────┴──────┐  ┌────────┴────────┐   │
│  │ aster crate │  │     asterd      │   │
│  │  (核心库)   │  │   (HTTP 服务)   │   │
│  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────┘
```

- CLI 直接使用 aster 核心库
- Desktop 通过 asterd HTTP API 通信
- 两者共享相同的核心逻辑
