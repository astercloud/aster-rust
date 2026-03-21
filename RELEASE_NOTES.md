# Release v0.20.1

## 🎉 主要功能

### OpenAI Responses continuation 支持 previous_response_id

本次更新为 OpenAI Responses API 增加基于 `previous_response_id` 的增量续写能力，减少需要整段历史重放的场景：

- **请求选项抽象**：新增 `ResponsesRequestOptions`，统一承载 `previous_response_id` 与 `store` 之类的请求附加参数
- **续写边界裁剪**：命中历史响应 id 时，只发送边界之后的增量消息，避免重复回放整段上下文
- **安全降级**：如果历史边界缺失或 continuation 元数据不完整，会自动退回完整历史重放

### Turn context 透传到 provider 执行期

本次同时把 turn 级上下文继续往 provider 执行链路透传，使 provider 可以在运行时读取 continuation 元数据：

- **流式场景补齐**：`scope_stream` 现在会携带 `turn_context`，避免流式 provider 分支拿不到当前 turn 元数据
- **Provider 侧读取统一**：OpenAI provider 直接从 `session_context` 读取 `provider_continuation`，不再依赖外部手动拼接
- **兼容现有模型路由**：仅在 Responses API / continuation 条件满足时启用，不影响其他 provider 路径

## 🔧 改进

### Responses API 请求构造更清晰

- `create_responses_request` 现在显式接收请求选项对象，减少后续再加 provider 特定参数时的函数签名震荡
- 为 `previous_response_id` 增加针对性单测，覆盖 `store = true` 与请求载荷断言

## 🐛 Bug 修复

- 修复 OpenAI Responses continuation 在 provider 执行期拿不到 `turn_context`，导致无法命中历史 response id 的问题
- 修复 continuation 历史边界命中失败时缺少明确降级路径的问题
- 修复相关请求构造测试仍假定当前版本即可产生升级提示的基线漂移问题

---

**完整变更**: v0.20.0...v0.20.1
