# Agent 上下文系统

## 概述

Agent 上下文系统管理执行状态，包括对话历史、文件上下文、工具结果等。

**核心路径**: `crates/aster/src/agents/context/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `types.rs` | 核心类型定义 |
| `manager.rs` | 上下文管理器 |
| `isolation.rs` | 沙箱隔离 |

## AgentContext 结构

```rust
pub struct AgentContext {
    pub context_id: String,
    pub agent_id: Option<String>,
    pub parent_context_id: Option<String>,
    pub conversation_history: Vec<Message>,
    pub conversation_summary: Option<String>,
    pub file_context: Vec<FileContext>,
    pub tool_results: Vec<ToolExecutionResult>,
    pub system_prompt: Option<String>,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub metadata: ContextMetadata,
}
```

## 上下文元数据

```rust
pub struct ContextMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub token_count: usize,
    pub is_compressed: bool,
    pub compression_ratio: Option<f64>,
    pub tags: Vec<String>,
    pub custom: HashMap<String, Value>,
}
```

## 文件上下文

```rust
pub struct FileContext {
    pub path: PathBuf,
    pub content: String,
    pub original_size: usize,
    pub is_truncated: bool,
    pub language: Option<String>,
    pub line_range: Option<(usize, usize)>,
    pub last_modified: Option<DateTime<Utc>>,
}
```

## 工具执行结果

```rust
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub call_id: String,
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub executed_at: DateTime<Utc>,
    pub input: Option<Value>,
    pub is_truncated: bool,
}
```

## 上下文继承

```rust
pub enum ContextInheritanceType {
    Full,       // 完整继承
    Shallow,    // 浅继承 (仅引用)
    Selective,  // 选择性继承
    None,       // 不继承
}

pub struct ContextInheritanceConfig {
    pub inherit_conversation: bool,
    pub inherit_files: bool,
    pub inherit_tool_results: bool,
    pub inherit_environment: bool,
    pub max_history_length: Option<usize>,
    pub max_file_contexts: Option<usize>,
    pub max_tool_results: Option<usize>,
    pub filter_sensitive: bool,
    pub compress_context: bool,
    pub target_tokens: Option<usize>,
}
```

## 上下文过滤

```rust
pub struct ContextFilter {
    pub sensitive_patterns: Vec<String>,
    pub excluded_env_keys: Vec<String>,
    pub excluded_tools: Vec<String>,
    pub excluded_file_patterns: Vec<String>,
}

// 默认敏感模式
impl ContextFilter {
    pub fn with_defaults() -> Self {
        Self {
            sensitive_patterns: vec![
                r"(?i)api[_-]?key",
                r"(?i)password",
                r"(?i)secret",
                r"(?i)token",
            ],
            excluded_env_keys: vec![
                "API_KEY", "SECRET", "PASSWORD", "TOKEN",
            ],
            ...
        }
    }
}
```

## 压缩结果

```rust
pub struct CompressionResult {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub ratio: f64,
    pub messages_summarized: usize,
    pub files_removed: usize,
    pub tool_results_removed: usize,
}
```

## 上下文更新

```rust
pub struct ContextUpdate {
    pub add_messages: Option<Vec<Message>>,
    pub add_files: Option<Vec<FileContext>>,
    pub add_tool_results: Option<Vec<ToolExecutionResult>>,
    pub set_environment: Option<HashMap<String, String>>,
    pub set_system_prompt: Option<String>,
    pub set_working_directory: Option<PathBuf>,
    pub add_tags: Option<Vec<String>>,
    pub set_custom_metadata: Option<HashMap<String, Value>>,
}
```

## 使用示例

```rust
// 创建上下文
let mut ctx = AgentContext::new()
    .with_agent_id("agent-1")
    .with_system_prompt("You are a helpful assistant")
    .with_working_directory("/project");

// 添加消息
ctx.add_message(Message::user().with_text("Hello"));

// 添加文件上下文
ctx.add_file_context(FileContext::new("src/main.rs", content));

// 添加工具结果
ctx.add_tool_result(ToolExecutionResult::success(
    "bash", "call-1", "output", 100
));

// 设置环境变量
ctx.set_env("PROJECT_NAME", "my-project");
```
