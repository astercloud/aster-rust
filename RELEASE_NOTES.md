# Release v0.18.0

## 🎉 主要功能

### 会话运行时状态追踪系统

新增完整的运行时状态追踪机制，支持细粒度的会话执行监控：

- **ThreadRuntimeStore**: 持久化会话线程运行时状态
- **TurnRuntime**: 追踪每个对话轮次的执行状态
- **ItemRuntime**: 记录消息、工具调用等细粒度执行项
- **状态快照**: 支持导出完整的会话运行时快照

### 增强的 Agent 事件系统

扩展 `AgentEvent` 枚举，新增以下事件类型：

- `TurnStarted`: 对话轮次开始
- `ItemStarted`: 执行项开始
- `ItemUpdated`: 执行项更新
- `ItemCompleted`: 执行项完成

支持更精细的事件驱动架构和实时状态监控。

### ActionRequired 作用域管理

改进 `ActionRequiredManager`，支持：

- **作用域隔离**: 通过 `ActionRequiredScope` 区分不同上下文的请求
- **消息队列**: 使用 `VecDeque` 管理待处理的 action required 消息
- **异步等待**: `request_and_wait_scoped` 方法支持作用域级别的请求处理

## 🔧 改进

### CLI 命令优化

- 更新 `acp`、`configure`、`web` 命令以支持新的运行时状态
- 改进会话构建器 (`session/builder.rs`) 集成运行时存储

### Server 路由增强

- `routes/agent.rs`: 集成运行时状态追踪
- `routes/reply.rs`: 支持细粒度的回复状态管理
- `routes/session.rs`: 增强会话生命周期管理
- `routes/action_required.rs`: 支持作用域化的 action required 处理

### 核心库改进

- **Scheduler**: 优化子 agent 调度逻辑
- **ExecutionManager**: 改进执行管理器与运行时状态的集成
- **SessionContext**: 扩展会话上下文以支持运行时元数据
- **Message**: 增强消息结构以支持 ActionRequired 数据

## 📝 文档

- 更新 `docs/aiprompts/session-management.md`，补充运行时状态管理说明

## 🔄 Breaking Changes

- `AgentEvent` 枚举新增多个变体，可能影响模式匹配代码
- `ActionRequiredManager` API 变更，新增 `request_and_wait_scoped` 方法
- 会话管理相关接口扩展，需要适配新的运行时存储机制

## 📦 依赖更新

- 保持与 v0.17.1 相同的依赖版本

## 🐛 Bug 修复

- 修复会话状态追踪中的竞态条件
- 改进 action required 消息的并发处理

---

**完整变更**: v0.17.1...v0.18.0
