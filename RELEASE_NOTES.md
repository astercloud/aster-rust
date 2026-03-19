# Release v0.19.0

## 🎉 主要功能

### 共享运行时队列与启动引导

新增共享 `SessionRuntimeQueueService` 与启动辅助能力，让多入口场景可以在同一套 runtime store 之上协调执行：

- **共享初始化**：新增 `initialize_shared_thread_runtime_store`、`require_shared_thread_runtime_store` 与 session bootstrap 辅助逻辑
- **排队执行**：支持 `QueuedTurnRuntime` 持久化、按 session 排队、取出下一轮以及清理队列
- **跨进程一致性**：SQLite 与内存存储都支持 queued turn 生命周期管理

### 运行时项目投影增强

`Agent` 的运行时事件投影能力继续扩展，支持把更多结构化信息落到 runtime items：

- **计划块提取**：自动识别 `<proposed_plan>`，生成显式 plan runtime item
- **文件产物追踪**：从工具返回元数据中提取 artifact path / id，写入 file artifact runtime item
- **状态项更新**：补齐 runtime status item 的初始化、更新与完成链路

### TODO 状态结构化演进

TODO 扩展从单纯 markdown 兼容态继续演进到结构化清单：

- **优先使用 `todo.v1`**：运行时优先读取结构化 `TodoListState`
- **保留兼容回退**：仍支持 legacy `todo.v0` markdown 状态
- **工具链收口**：`todo_write_tool` 与 workflow 集成统一走结构化 TODO 解析/持久化

## 🔧 改进

### CLI / Server / Scheduler 对齐

- CLI 会话构建器与命令入口改为显式接入共享 runtime store
- Server 状态管理补齐 runtime queue 与共享 store 初始化
- Scheduler、ExecutionManager 与 SessionManager 的 runtime 协作链路进一步收口

### 工具系统精简

- 移除 `three_files_tool`
- 保留并加强 TODO / workflow 现役路径，减少平行实现

### Provider 与格式细节修正

- 调整 Google / OpenAI Responses 格式处理，改善与新的 runtime item 投影协作

## 📝 文档

- 更新 `docs/aiprompts/session-management.md`，补充共享 runtime store、队列与状态投影说明

## 🔄 Breaking Changes

- 依赖共享 runtime store 的入口现在要求先完成 bootstrap / initialize，再创建带必需 store 的 `Agent`
- `three_files_tool` 已移除，依赖该工具名的外部调用需要切换到现役工具路径
- TODO 扩展优先读取结构化状态，依赖 legacy markdown 直读的逻辑需要适配

## 🐛 Bug 修复

- 修复 `todo_extension` 测试作用域缺失 trait 导入导致的编译问题
- 修复 CLI session builder 中 fallible runtime store 初始化的错误处理
- 修复严格 clippy 下字符串切片、large enum variant 与 `manual_map` 报警

---

**完整变更**: v0.18.0...v0.19.0
