# Aster 项目架构概览

## 概述

Aster 是一个 Rust AI Agent 框架，支持 CLI 和 Electron 桌面界面。它提供了完整的 AI 代理能力，包括多 Provider 支持、MCP 协议集成、工具系统、会话管理等核心功能。

## 项目结构

```
aster-rust/
├── crates/
│   ├── aster/           # 核心逻辑库
│   ├── aster-cli/       # CLI 入口 (binary: aster)
│   ├── aster-server/    # 后端服务 (binary: asterd)
│   ├── aster-bench/     # 基准测试
│   ├── aster-mcp/       # MCP 扩展
│   └── aster-test/      # 测试工具
├── ui/
│   └── desktop/         # Electron 桌面应用
├── examples/            # 示例代码
└── docs/                # 文档
```

## 核心模块 (crates/aster/src/)

```
src/
├── agents/              # Agent 系统
│   ├── agent.rs         # 主 Agent 实现
│   ├── context/         # 上下文管理
│   ├── communication/   # Agent 间通信
│   ├── parallel/        # 并行执行
│   ├── monitor/         # 监控指标
│   ├── resume/          # 状态恢复
│   ├── specialized/     # 专用 Agent (Explore/Plan)
│   └── error_handling/  # 错误处理
├── providers/           # AI Provider 集成
├── mcp/                 # MCP 协议支持
├── tools/               # 工具系统
├── skills/              # 技能系统
├── config/              # 配置管理
├── session/             # 会话管理
├── context_mgmt/        # 上下文压缩
├── conversation/        # 对话管理
├── permission/          # 权限控制
└── memory/              # 记忆系统
```

## 入口点

| 入口 | 路径 | 说明 |
|------|------|------|
| CLI | `crates/aster-cli/src/main.rs` | 命令行工具入口 |
| Server | `crates/aster-server/src/main.rs` | 后端服务入口 |
| Agent | `crates/aster/src/agents/agent.rs` | 核心 Agent 实现 |

## 数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                     用户输入 (CLI/Desktop)                       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Agent                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Provider    │  │ Extension   │  │ Tool Registry           │  │
│  │ Manager     │  │ Manager     │  │ (Native + MCP)          │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                     │                 │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────────┴──────────┐     │
│  │ AI Provider │  │ MCP Server  │  │ Tool Execution      │     │
│  │ (多种)      │  │ (扩展)      │  │ (Bash/File/Search)  │     │
│  └─────────────┘  └─────────────┘  └─────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Session / Conversation                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Message     │  │ Context     │  │ Permission              │  │
│  │ History     │  │ Compaction  │  │ Management              │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 关键特性

### 1. 多 Provider 支持
- OpenAI, Anthropic, Google, Azure, Bedrock
- Ollama (本地), OpenRouter, LiteLLM
- 自定义 Provider 扩展

### 2. MCP 协议集成
- 多传输层: Stdio, HTTP, SSE, WebSocket
- 自动重连和健康检查
- 工具发现和缓存

### 3. 工具系统
- 原生工具: Bash, File, Search, Web
- MCP 工具: 动态加载
- 权限控制和审计

### 4. Agent 能力
- 子 Agent 支持
- 并行执行
- 状态恢复 (Checkpoint)
- 上下文压缩

## 配置层级

```
全局配置 (~/aster/config.toml)
    ↓
项目配置 (.aster/config.toml)
    ↓
会话配置 (运行时)
```

## 运行模式

| 模式 | 说明 |
|------|------|
| Auto | 自动执行，无需确认 |
| SmartApprove | 智能审批，危险操作需确认 |
| Manual | 手动模式，所有操作需确认 |


## 文档索引

### 核心系统
- [agent-architecture.md](agent-architecture.md) - Agent 系统架构
- [agent-context.md](agent-context.md) - Agent 上下文系统
- [providers.md](providers.md) - AI Provider 系统
- [tools-system.md](tools-system.md) - 工具系统
- [mcp-integration.md](mcp-integration.md) - MCP 协议集成

### Agent 能力
- [subagent.md](subagent.md) - 子 Agent 系统
- [parallel-execution.md](parallel-execution.md) - 并行执行
- [specialized-agents.md](specialized-agents.md) - 专用 Agent
- [error-handling.md](error-handling.md) - 错误处理
- [monitoring.md](monitoring.md) - 监控系统

### 会话与对话
- [session-management.md](session-management.md) - 会话管理
- [conversation.md](conversation.md) - 对话管理
- [context-management.md](context-management.md) - 上下文压缩
- [context.md](context.md) - 上下文管理
- [memory.md](memory.md) - 记忆系统


### 配置与权限
- [config-system.md](config-system.md) - 配置系统
- [permission.md](permission.md) - 权限系统
- [rules.md](rules.md) - 规则系统
- [hints.md](hints.md) - 提示文件系统

### 扩展系统
- [extension.md](extension.md) - 扩展系统
- [skills-system.md](skills-system.md) - 技能系统
- [plugins.md](plugins.md) - 插件系统
- [recipe.md](recipe.md) - Recipe 系统
- [hooks.md](hooks.md) - Hooks 系统

### 应用入口
- [cli-usage.md](cli-usage.md) - CLI 使用指南
- [desktop-app.md](desktop-app.md) - Electron 桌面应用
- [server.md](server.md) - Aster Server (asterd)
- [bench.md](bench.md) - 基准测试系统

### 开发工具
- [lsp.md](lsp.md) - LSP 服务器管理
- [parser.md](parser.md) - 代码解析
- [search.md](search.md) - 代码搜索
- [git-integration.md](git-integration.md) - Git 集成
- [github.md](github.md) - GitHub 集成


### 安全与监控
- [security.md](security.md) - 安全系统
- [sandbox.md](sandbox.md) - 沙箱系统
- [codesign.md](codesign.md) - 代码签名
- [diagnostics.md](diagnostics.md) - 诊断系统
- [telemetry.md](telemetry.md) - 遥测系统
- [tracing.md](tracing.md) - 追踪系统

### 状态管理
- [checkpoint.md](checkpoint.md) - 检查点系统
- [rewind.md](rewind.md) - 回退系统
- [execution.md](execution.md) - 执行管理
- [scheduler.md](scheduler.md) - 调度系统

### 网络与通信
- [streaming.md](streaming.md) - 流式处理
- [network.md](network.md) - 网络模块
- [oauth.md](oauth.md) - OAuth 认证
- [teleport.md](teleport.md) - 远程会话

### 其他模块
- [prompt.md](prompt.md) - 系统提示词
- [media.md](media.md) - 媒体处理
- [notifications.md](notifications.md) - 通知系统
- [ratelimit.md](ratelimit.md) - 速率限制
- [updater.md](updater.md) - 自动更新
- [map.md](map.md) - 代码本体图谱
- [blueprint.md](blueprint.md) - 蓝图系统
- [background.md](background.md) - 后台任务
- [chrome-mcp.md](chrome-mcp.md) - Chrome 集成
- [aster-apps.md](aster-apps.md) - Aster Apps
- [plan.md](plan.md) - 计划系统
- [token-counter.md](token-counter.md) - Token 计数
- [tool-monitor.md](tool-monitor.md) - 工具监控
- [slash-commands.md](slash-commands.md) - 斜杠命令
