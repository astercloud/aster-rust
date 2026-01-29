# 子 Agent 系统

## 概述

子 Agent 系统允许主 Agent 委托任务给独立运行的子 Agent。

**核心路径**: `crates/aster/src/agents/subagent_tool.rs`

## 工具定义

```rust
pub const SUBAGENT_TOOL_NAME: &str = "subagent";
```

## 使用模式

### 1. Ad-hoc 模式
提供 `instructions` 执行自定义任务：

```json
{
  "tool": "subagent",
  "input": {
    "instructions": "分析 src/ 目录下的代码结构"
  }
}
```

### 2. Predefined 模式
使用预定义的 `subrecipe`：

```json
{
  "tool": "subagent",
  "input": {
    "subrecipe": "code-review",
    "parameters": {
      "file_path": "src/main.rs"
    }
  }
}
```

### 3. Augmented 模式
结合 subrecipe 和额外指令：

```json
{
  "tool": "subagent",
  "input": {
    "subrecipe": "code-review",
    "instructions": "特别关注安全问题",
    "parameters": {
      "file_path": "src/auth.rs"
    }
  }
}
```

## 参数结构

```rust
pub struct SubagentParams {
    // 任务指令 (ad-hoc 必需)
    pub instructions: Option<String>,
    
    // 预定义 subrecipe 名称
    pub subrecipe: Option<String>,
    
    // subrecipe 参数
    pub parameters: Option<HashMap<String, Value>>,
    
    // 启用的扩展 (空数组=无扩展, 省略=继承全部)
    pub extensions: Option<Vec<String>>,
    
    // 模型设置覆盖
    pub settings: Option<SubagentSettings>,
    
    // 是否返回摘要 (默认 true)
    pub summary: bool,
    
    // 图片数据 (多模态)
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
主 Agent 调用 subagent 工具
    │
    ▼
┌─────────────────────────────┐
│  解析参数                    │
│  - 验证 instructions/subrecipe │
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

1. **子 Agent 不能创建子 Agent**
   ```rust
   if session.session_type == SessionType::SubAgent 
      && tool_call.name == SUBAGENT_TOOL_NAME {
       return Err("Subagents cannot create other subagents");
   }
   ```

2. **Gemini 模型不支持子 Agent**

3. **非自动模式不支持子 Agent**

## 并行执行

在同一消息中多次调用 `subagent` 可并行执行：

```json
// 主 Agent 的工具调用
[
  {"tool": "subagent", "input": {"subrecipe": "lint"}},
  {"tool": "subagent", "input": {"subrecipe": "test"}},
  {"tool": "subagent", "input": {"subrecipe": "build"}}
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
