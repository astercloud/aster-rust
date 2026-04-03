# 子 Agent 系统

## 概述

子 Agent 系统允许主 Agent 委托任务给独立运行的子 Agent。

**核心路径**: `crates/aster/src/agents/subagent_tool.rs`

## 工具定义

```rust
pub const AGENT_TOOL_NAME: &str = "Agent";
```

## 使用模式

### 1. Ad-hoc 模式
省略 `subagent_type` 时，使用通用委派 Agent：

```json
{
  "tool": "Agent",
  "input": {
    "description": "分析代码结构",
    "prompt": "分析 src/ 目录下的代码结构"
  }
}
```

### 2. Specialized 模式
使用 `subagent_type` 命中本地 specialized recipe：

```json
{
  "tool": "Agent",
  "input": {
    "description": "代码审查",
    "subagent_type": "code-review",
    "prompt": "检查 src/main.rs 的实现质量与潜在风险"
  }
}
```

### 3. Augmented 模式
结合 specialized type、命名和模型覆盖：

```json
{
  "tool": "Agent",
  "input": {
    "description": "鉴权审查",
    "subagent_type": "code-review",
    "name": "auth-review",
    "model": "gpt-5.4",
    "prompt": "检查 src/auth.rs，特别关注安全问题"
  }
}
```

## 模型可见参数结构

```rust
struct AgentToolParams {
    pub description: String,
    pub prompt: String,
    pub subagent_type: Option<String>,
    pub model: Option<String>,
    pub run_in_background: bool,
    pub name: Option<String>,
    pub team_name: Option<String>,
    pub mode: Option<String>,
    pub isolation: Option<String>,
    pub cwd: Option<String>,
    pub images: Option<Vec<ImageData>>,
}
```

当前 runtime 的收敛规则：

- `description` + `prompt` 是必填
- `subagent_type` 命中本地 `SubRecipe` 时走 specialized recipe
- `subagent_type` 未命中时，不报错，而是作为 role hint 注入 prompt
- 未注入 callback-backed agent runtime 时，`run_in_background` / `team_name` / `mode` / `isolation` 会直接报参错；`cwd` 仅接受绝对目录
- 注入 callback-backed agent runtime 后，`run_in_background` / `name` / `team_name` / `cwd` 会切到现代子代理会话主链
- team 子代理虽然保留 `Agent` current surface，但只允许继续创建同步子代理；不允许再启动后台 agent，也不允许再派生 teammate

## 内部映射结构

```rust
pub struct SubagentParams {
    pub instructions: Option<String>,
    pub subrecipe: Option<String>,
    pub role_hint: Option<String>,
    pub parameters: Option<HashMap<String, Value>>,
    pub extensions: Option<Vec<String>>,
    pub settings: Option<SubagentSettings>,
    pub summary: bool,
    pub images: Option<Vec<ImageData>>,
}

pub struct SubagentSettings {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
}
```

## SubRecipe 定义

```rust
pub struct SubRecipe {
    pub name: String,
    pub path: String,
    pub values: Option<HashMap<String, String>>,
    pub sequential_when_repeated: bool,
    pub description: Option<String>,
}
```

在主 Recipe 中定义：

```yaml
sub_recipes:
  - name: lint
    path: ./lint.yaml
    description: 执行代码检查
    values:
      strict: "true"
    sequential_when_repeated: false
    
  - name: deploy
    path: ./deploy.yaml
    description: 部署应用
    sequential_when_repeated: true  # 不能并行
```

## 执行流程

```
主 Agent 调用 Agent 工具
    │
    ▼
┌─────────────────────────────┐
│  解析参数                    │
│  - 验证 description/prompt   │
│  - 构建 Recipe               │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  创建子会话                  │
│  SessionType::SubAgent      │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  应用设置覆盖                │
│  - Provider/Model           │
│  - Extensions 过滤          │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  执行子 Agent 任务           │
└───────────┬─────────────────┘
            │
            ▼
返回结果/摘要给主 Agent
```

## 摘要指令

当 `summary: true` 时，自动添加：

```rust
const SUMMARY_INSTRUCTIONS: &str = r#"
Important: Your parent agent will only receive your final message...
Make sure your last message provides a comprehensive summary of:
- What you were asked to do
- What actions you took
- The results or outcomes
- Any important findings or recommendations
"#;
```

## 限制

1. **普通子 Agent 不能递归创建新的 Agent**
   ```rust
   if session.session_type == SessionType::SubAgent && tool_call.name == AGENT_TOOL_NAME {
       if !session_allows_subagent_teammate_tools(session) {
           return Err("Agents cannot create other agents");
       }
   }
   ```

2. **team 子代理只允许同步再派生子代理**
   ```rust
   if team_subagent && request.run_in_background {
       return Err("Team subagents cannot spawn background agents in the current runtime");
   }
   if team_subagent && (name.is_some() || team_name.is_some()) {
       return Err("Team subagents cannot spawn teammates in the current runtime; omit name and team_name");
   }
   ```

3. **Gemini 模型不支持子 Agent**

4. **非自动模式不支持子 Agent**

## 并行执行

在同一消息中多次调用 `Agent` 可并行执行：

```json
// 主 Agent 的工具调用
[
  {"tool": "Agent", "input": {"description": "代码检查", "subagent_type": "lint", "prompt": "运行 lint 并汇总结果"}},
  {"tool": "Agent", "input": {"description": "测试执行", "subagent_type": "test", "prompt": "运行测试并汇总失败项"}},
  {"tool": "Agent", "input": {"description": "构建验证", "subagent_type": "build", "prompt": "执行构建并报告结果"}}
]
```

除非 `sequential_when_repeated: true`。

## 扩展继承

```json
// 继承所有扩展 (省略 extensions)
{"instructions": "..."}

// 不使用任何扩展
{"instructions": "...", "extensions": []}

// 只使用指定扩展
{"instructions": "...", "extensions": ["developer"]}
```

## 运行时继承与覆盖

子 Agent 会从父 turn 继承执行语义相关的运行时上下文：

- `cwd`
- `model`
- `effort`
- `approval_policy`
- `sandbox_policy`
- `collaboration_mode`

以下字段不会默认继承：

- `output_schema`
- `output_schema_source`
- `metadata`

原因：

- `output_schema` 是当前 agent 的最终输出契约，不能默认泄漏到子 agent
- `metadata` 可能包含 provider continuation 等仅对当前 turn 有效的运行时状态

`settings` 的覆盖规则遵循单一事实源原则：

- `settings.provider` 单独出现时，会切到该 provider 的默认模型
- `settings.model` 不仅会重建 provider 的 `ModelConfig`
- 还会同步写回子 agent 的 `turn_context.model`

这样 `provider` 配置与 `turn` 级执行 override 始终保持一致，避免 reply 阶段再次被旧继承模型覆盖。

模型切换不是简单改字符串：

- 框架会按新模型重新构建 `ModelConfig`
- 会同步刷新该模型对应的 `context_limit`
- 已存在的通用调优项（如 `temperature` / `max_tokens` / `toolshim`）继续保留
