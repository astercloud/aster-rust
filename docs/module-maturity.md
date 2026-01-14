# 模块成熟度指南

本文档描述 aster 核心库各模块的成熟度级别，帮助用户和贡献者了解哪些功能可以放心使用，哪些还在开发中。

## 成熟度级别

| 级别 | 标识 | 含义 |
|------|------|------|
| 🟢 稳定 | Stable | 生产可用，API 稳定，有完整测试 |
| 🟡 实验性 | Experimental | 功能可用，API 可能调整，测试覆盖中等 |
| 🔴 草稿 | Draft | 结构已有，实现待完善，仅供参考 |

---

## 🟢 稳定 (Stable)

这些模块经过充分设计和测试，可用于生产环境。

| 模块 | 路径 | 描述 |
|------|------|------|
| `agents` | `src/agents/` | Agent 核心实现，事件驱动架构 |
| `tools` | `src/tools/` | 工具系统，Registry 模式 |
| `mcp` | `src/mcp/` | MCP 协议支持，工具发现与调用 |
| `streaming` | `src/streaming/` | 流式输出，SSE/WebSocket 支持 |
| `memory` | `src/memory/` | 上下文管理，多层记忆系统 |
| `context` | `src/context/` | 上下文构建，Token 管理 |
| `session` | `src/session/` | 会话持久化，事件溯源 |
| `config` | `src/config/` | 配置管理，多来源合并 |
| `core` | `src/core/` | 基础类型定义 |
| `rewind` | `src/rewind/` | 状态回退，20 个测试用例 |
| `diagnostics` | `src/diagnostics/` | 诊断信息收集，31 个测试用例 |
| `plugins` | `src/plugins/` | 插件系统，35 个测试用例 |
| `search` | `src/search/` | 代码搜索，40 个测试用例 |
| `updater` | `src/updater/` | 自动更新，39 个测试用例 |
| `teleport` | `src/teleport/` | 远程会话，35 个测试用例 |
| `blueprint` | `src/blueprint/` | 任务规划与分解，102 个测试用例 |
| `checkpoint` | `src/checkpoint/` | 状态快照与恢复，47 个测试用例 |

---

## 🟡 实验性 (Experimental)

这些模块功能基本可用，但 API 可能在后续版本中调整。

| 模块 | 路径 | 描述 | 备注 |
|------|------|------|------|
| `git` | `src/git/` | Git 操作封装 | 功能完整 |
| `github` | `src/github/` | GitHub API 集成 | PR/Issue/Actions |
| `hooks` | `src/hooks/` | 事件钩子系统 | 触发机制稳定中 |
| `lsp` | `src/lsp/` | 语言服务协议 | 基础功能可用 |
| `plan` | `src/plan/` | 执行计划管理 | 与 blueprint 配合 |
| `prompt` | `src/prompt/` | 提示词模板管理 | 模板语法稳定 |
| `rules` | `src/rules/` | 规则系统 | 规则格式稳定中 |
| `skills` | `src/skills/` | 技能系统 | 技能定义格式稳定中 |

---

## 🔴 草稿 (Draft)

这些模块结构已建立，但实现尚不完整，仅供参考和贡献。

| 模块 | 路径 | 描述 | 状态 |
|------|------|------|------|
| `sandbox` | `src/sandbox/` | 沙箱执行环境 | 多后端支持，配置复杂 |
| `codesign` | `src/codesign/` | 代码签名验证 | CLI 场景为主 |
| `notifications` | `src/notifications/` | 系统通知 | 跨平台支持待完善 |
| `chrome` | `src/chrome/` | 浏览器集成 | 场景有限 |
| `background` | `src/background/` | 后台任务 | 调度策略待定 |
| `map` | `src/map/` | 代码地图 | 基础实现 |

---

## 设计参考

aster-rust 的模块设计参考了 [claude-code-open](https://github.com/anthropics/claude-code) 项目，并根据 Rust 生态和桌面/CLI 双场景需求进行了调整。

### 与 claude-code-open 的关系

- **学习借鉴**：模块划分和设计思路参考了 claude-code-open
- **语言差异**：使用 Rust 重新实现，利用 Rust 的性能和安全特性
- **场景扩展**：同时支持 CLI 和桌面应用场景
- **独立演进**：后续会根据 aster 自身需求独立发展

### 与 Claude Agent SDK 的关系

Anthropic 推出了官方的 Agent SDK，aster-rust 定位为：
- Rust 生态的 Agent 框架替代方案
- 学习和实验 Agent 架构的参考实现
- 可组合的模块化设计，适配不同场景

---

## 贡献指南

### 如何提升模块成熟度

1. **草稿 → 实验性**
   - 完成核心功能实现
   - 添加基础单元测试
   - 编写 README 文档

2. **实验性 → 稳定**
   - API 设计评审通过
   - 测试覆盖率 > 80%
   - 有实际使用案例
   - 文档完整

### 优先贡献方向

- 🔴 草稿模块的核心功能实现
- 🟡 实验性模块的测试补充
- 🟢 稳定模块的性能优化

---

## 更新记录

| 日期 | 变更 |
|------|------|
| 2026-01-14 | checkpoint 模块提升到 🟢 稳定（47 个测试用例） |
| 2026-01-14 | blueprint 模块提升到 🟢 稳定（102 个测试用例） |
| 2026-01-14 | teleport 模块提升到 🟢 稳定（35 个测试用例） |
| 2026-01-14 | updater 模块提升到 🟢 稳定（39 个测试用例） |
| 2026-01-14 | search 模块提升到 🟢 稳定（40 个测试用例） |
| 2026-01-14 | plugins 模块提升到 🟢 稳定（35 个测试用例） |
| 2026-01-14 | diagnostics 模块提升到 🟢 稳定（31 个测试用例） |
| 2026-01-14 | rewind 模块提升到 🟢 稳定（20 个测试用例） |
| 2026-01-14 | 对齐 plugins/diagnostics/teleport/rewind/updater/search 模块 |
| 2026-01-14 | 初始版本，建立成熟度分级 |
