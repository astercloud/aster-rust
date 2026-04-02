# Release v0.25.0

## 🎉 主要功能

### 工具协议收敛到结构化任务板与新版委派边界

本次更新把旧的 `TodoWrite`、`Task`、`KillShell` 等历史工具路径收敛到一组更稳定的结构化工具接口，方便宿主和 UI 对接现代任务流：

- **任务板工具替换旧 TODO 流程**：新增 `TaskCreate`、`TaskList`、`TaskGet`、`TaskUpdate`，并把会话扩展数据升级为 `task_list.v1` 结构化任务板
- **后台任务终止接口统一**：新增 `TaskStop`，标准化后台任务停止能力，并兼容旧 `shell_id` 参数名
- **委派运行时通过 callback 注入**：工具注册支持按需注入 `spawn_agent`、`send_input`、`wait_agent`、`resume_agent`、`close_agent`，为宿主接入现代 subagent/runtime 预留清晰边界

### Ask 与会话运行时升级为更现代的交互协议

围绕用户交互和运行时状态，本次也补齐了更结构化的协议表达，让 CLI、Server 与桌面端更容易消费同一套事件：

- **Ask 支持多问题结构化输入**：除了 legacy `question/options` 外，现在还支持 `questions` 数组、header、option description 与 multi-select 语义
- **Ask 返回标准化答案映射**：工具结果新增 `answers`、`raw_response` 与 `question_count` 等元数据，减少宿主自行解析负担
- **上下文压缩策略更可控**：当 turn context 明确关闭自动压缩时，runtime 会直接提示手动压缩或新建会话，不再静默触发 overflow recovery

## 🔧 改进

### Tauri 桌面调试与会话观测增强

桌面端这次主要补齐了本地调试闭环，方便在 GUI 中直接操作后端并观察远端会话活动：

- **侧边栏可直接启动 / 停止本地服务**：Tauri 状态中新增 `server_process` 句柄，桌面端可以在 GUI 中控制本地 `asterd`
- **聊天面板支持远端会话活动流**：新增活动时间线、运行时状态展示以及工具审批动作按钮，便于跟踪工具调用和用户批准流
- **协议与文档同步更新**：`tools-system.md`、提示模板和相关测试一起迁移到新版 Task*/agent-control 语义，降低新旧协议混用成本

## 🐛 Bug 修复

- 修复自动压缩被关闭时仍可能继续走 overflow recovery 的行为
- 修复任务状态在 session 扩展数据与旧 todo 快照之间不同步的问题
- 修复桌面端本地消息发送失败时缺少回退持久化路径的问题

---

**完整变更**: v0.24.0...v0.25.0
