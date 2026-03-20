# Release v0.20.0

## 🎉 主要功能

### Assistant tool 调用保持原子消息结构

本次更新调整了 assistant 响应在工具调用前后的归一化策略，确保推理、正文与 tool request 保持在同一条 provider 回合消息中：

- **原子回合保留**：不再把 thinking 与 tool request 人工拆成多条 assistant 消息
- **请求归一化**：在分类工具请求时回写标准化后的 `ToolRequest`，保留 metadata 与 tool meta
- **多工具顺序稳定**：多个 tool request 会按原始顺序重新组装，减少后续 provider 回放歧义

### OpenAI / DeepSeek reasoning_content 往返增强

围绕 `reasoning_content` 的格式转换链路做了补强，改善带推理内容的 tool calling 兼容性：

- **完整推理拼接**：格式化消息时会保留多段 Thinking 内容并合并写入 `reasoning_content`
- **响应反序列化补齐**：从 OpenAI 风格响应恢复消息时，支持把 `reasoning_content` 还原成 Thinking 内容
- **工具调用协同**：带 reasoning 的 assistant tool-call 消息在下一轮发送时更完整

### Subagent 会话元数据落盘

新增 subagent session metadata 能力，让父子会话关系和展示语义更清晰：

- **父会话关联**：记录 `parent_session_id` 与来源工具
- **任务摘要**：自动生成 subrecipe / instructions 摘要，便于会话列表识别
- **角色提示**：支持显式 `role_hint`，用于展示更友好的子代理标签
- **来源 turn 追踪**：在上下文可用时记录创建该 subagent 的父 turn id

## 🔧 改进

### Session 元数据查询能力

- 导出 subagent session metadata 相关查询与列表方法
- 支持按父会话筛选并按更新时间倒序返回子会话

### 测试覆盖补强

- 为 tool request 归一化、reasoning_content 往返与 subagent metadata 新增针对性测试
- 补充多工具、多段推理与 parent turn 透传场景断言

## 🔄 Breaking Changes

- 依赖把 thinking 与 tool request 拆成独立 assistant 消息的下游逻辑，需要改为兼容新的原子消息结构
- Subagent 工具新增 `role_hint` 参数；如果有自定义 schema 校验，需要同步更新

## 🐛 Bug 修复

- 修复部分 provider 在 assistant 工具调用回合丢失推理内容，导致下一轮请求上下文不完整的问题
- 修复 tool request 标准化后 metadata / tool meta 可能未回写到原始响应结构的问题
- 修复 subagent 会话创建后缺少结构化元数据，导致父子关系与展示信息缺失的问题

---

**完整变更**: v0.19.0...v0.20.0
