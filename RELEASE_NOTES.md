# Release v0.22.0

## 🎉 主要功能

### 会话执行运行时与结构化输出契约可观察

本次更新把会话执行期的 provider、model 与结构化输出能力从“配置推断”提升为“运行时可观测”，方便 CLI、Server 与 Desktop UI 基于真实执行状态做展示、诊断与计费：

- **运行时结构化输出透传**：turn runtime 现在会记录 `output_schema_runtime`，明确 provider、model、source 与 strategy 的真实决策结果
- **优先使用原生 structured output**：OpenAI / Codex 等支持原生结构化输出的 provider 会优先走原生协议，不支持时自动回退到 `final_output_tool`
- **跨端展示一致**：CLI、Server 与 Desktop UI 都可以看到真实执行模型以及结构化输出策略，而不是只依赖静态配置

### 子代理继承父任务上下文更完整

子代理创建流程现在会继承父任务的关键运行参数，同时显式清理不应继承的结构化输出契约与 continuation 元数据，减少上下文漂移：

- **关键上下文继承**：继承 `cwd`、`model`、`effort`、`approval_policy`、`sandbox_policy` 与 `collaboration_mode`
- **契约边界更清晰**：子任务会清理父任务的 `output_schema` 与 continuation metadata，避免父任务结果约束污染子任务执行
- **子代理行为更可预测**：父子任务之间的执行环境更一致，但结果契约仍保持隔离

## 🔧 改进

### UI 运行时模型与成本统计更准确

Desktop UI 现在会优先读取真实运行时 provider / model，减少模型切换、override 与回退策略下的展示偏差：

- **成本归因更准确**：`CostTracker` 会按真实运行时模型统计和归档累计成本
- **模型展示更贴近执行事实**：底部模型栏、输入区与会话流优先显示 runtime provider / model
- **会话状态同步更稳定**：模型切换或 recipe 覆盖后，前端展示与服务端执行结果保持一致

## 🐛 Bug 修复

- 修复运行时模型与静态配置模型不一致时，UI 展示与计费统计偏差的问题
- 修复 recipe 应用错误被吞没，导致调用链路诊断不清晰的问题
- 修复 model override / fast model 切换时 context limit 继承不正确的问题
- 修复 output schema 未正确透传到 OpenAI Responses 与 Codex App Server 的问题

---

**完整变更**: v0.21.0...v0.22.0
