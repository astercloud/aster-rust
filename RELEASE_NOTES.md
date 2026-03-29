# Release v0.23.0

## 🎉 主要功能

### Codex reasoning 摘要与原始内容双通道接入

本次更新补齐了 Codex App Server 新版 reasoning 事件模型，turn runtime 现在既能保留完整 reasoning 文本，也能按段落输出可读摘要，方便 CLI、Server 与 UI 做更稳定的展示与调试：

- **reasoning 摘要分段持久化**：runtime item 新增 `summary` 字段，保留按空行拆分的可读摘要片段
- **兼容新旧事件协议**：同时支持 `summaryPartAdded`、`summaryTextDelta`、`textDelta` 与 legacy `item/reasoning/delta`
- **原始内容仍可追踪**：reasoning item 可以同时承载 `summary` 与 `content`，既利于展示，也保留原始推理流

### Codex turn 启动策略跟随运行时上下文

Codex provider 在新建与恢复 thread 时，会读取当前 turn context 中的审批与 sandbox 策略，不再把启动参数固定写死为默认值：

- **审批策略按 turn 透传**：`approvalPolicy` 会跟随当前 turn context，而不是始终固定为 `never`
- **sandbox 模式标准化映射**：`read-only`、`workspace-write`、`danger-full-access` 会映射成 App Server 约定的协议值
- **恢复会话行为更一致**：恢复旧会话失败后重新建会话，也会继续沿用当前运行时策略

## 🔧 改进

### Runtime payload 与测试覆盖更完整

围绕 reasoning 与 turn start 策略，本次还补齐了运行时数据结构与回归测试：

- **payload 表达更完整**：`ItemRuntimePayload::Reasoning` 在有摘要时会显式携带 `summary`
- **事件解析更稳定**：`parse_event` 覆盖新旧 reasoning 增量事件，减少协议升级带来的展示偏差
- **单测覆盖补齐**：新增 reasoning summary 分段、turn start policy 透传与新事件解析的测试

## 🐛 Bug 修复

- 修复 Codex 新版 reasoning 事件无法被完整消费，导致摘要展示丢失的问题
- 修复恢复 thread 或创建新 thread 时审批 / sandbox 策略被错误固定的问题
- 修复 reasoning runtime item 只能保存拼接文本、无法稳定保留摘要段落的问题

---

**完整变更**: v0.22.0...v0.23.0
