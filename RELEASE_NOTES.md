# Release v0.27.0

## 🎉 主要功能

### 会话线程运行时复用更稳定

这次版本重点收口在 Agent 运行时和会话状态一致性上，减少同一 thread/turn 生命周期中的重复加载与错配风险：

- **已有 thread 直接复用 turn runtime**：当运行时线程已存在时，`ensure_runtime_turn_initialized` 会直接基于 `session_config.id` 初始化 turn runtime，不再额外回读 session
- **工具派发按需加载最新会话**：只有在 `agent` 工具需要 current surface 语义时才回读最新 session，降低普通工具调用的额外存储访问
- **结构化最终输出更早分流**：`final_output` 工具改为优先处理，减少工具分发表中的歧义路径

### 会话命名与记忆检索更稳健

围绕用户可感知的会话体验，这次也补上了两个常见稳定性问题：

- **自动命名只作用于占位标题**：只有 `新对话`、`New Session`、`Untitled` 等占位标题且消息数仍在阈值内时，系统才会自动生成会话名，避免覆盖已有有效标题
- **Memory FTS 查询先做安全归一化**：`@mention`、邮箱、括号和纯符号输入会先转换为可搜索词元；纯符号查询直接返回空结果，不再把脏输入交给 FTS
- **补齐针对性回归测试**：新增会话命名判定与 `@bot`/符号类 memory 查询测试，覆盖这次修复边界

## 🔧 改进

### 子代理能力暴露与版本元数据同步

- **subagent 开关跟随真实可见工具**：reply 构造逻辑现在根据当前工具列表里是否真正暴露 `agent` 工具来决定是否启用 subagent 能力，避免提示层和实际能力不一致
- **工作区版本统一升级到 `0.27.0`**：同步更新 Rust workspace、crate 间显式版本依赖、桌面端 `package.json` 和 OpenAPI 版本号
- **`Cargo.lock` 随版本一起收敛**：工作区包版本元数据全部刷新到 `0.27.0`

## 🐛 Bug 修复

- 修复已有 thread 存在时 turn runtime 初始化仍依赖旧 session 读取路径的问题
- 修复 `agent` 工具在 subagent/current-surface 场景中的会话获取过度问题
- 修复自动命名可能覆盖非占位会话标题的问题
- 修复 memory 全文检索对 `@mention`、邮箱和纯符号查询不稳的问题

---

**完整变更**: v0.26.0...v0.27.0
