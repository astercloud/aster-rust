# Prompt 系统提示词

系统提示词构建和管理。

## 模块结构

```
prompt/
├── attachments.rs  # 附件管理
├── builder.rs      # 提示词构建器
├── cache.rs        # 缓存系统
├── templates.rs    # 模板常量
└── types.rs        # 类型定义
```

## 核心组件

### SystemPromptBuilder
```rust
pub struct SystemPromptBuilder;

impl SystemPromptBuilder {
    pub fn new() -> Self;
    pub fn with_context(ctx: PromptContext) -> Self;
    pub fn build() -> BuildResult;
}
```

### PromptCache
```rust
pub struct PromptCache;

impl PromptCache {
    pub fn get(key: &str) -> Option<String>;
    pub fn set(key: &str, value: String);
    pub fn stats() -> CacheStats;
}
```


## 模板常量

```rust
pub const CORE_IDENTITY: &str;      // 核心身份
pub const CODING_GUIDELINES: &str;  // 编码指南
pub const GIT_GUIDELINES: &str;     // Git 指南
pub const OUTPUT_STYLE: &str;       // 输出风格
pub const TOOL_GUIDELINES: &str;    // 工具指南
pub const TASK_MANAGEMENT: &str;    // 任务管理
pub const SUBAGENT_SYSTEM: &str;    // 子 Agent 系统
```

## 上下文信息

```rust
pub fn get_environment_info() -> EnvironmentInfo;
pub fn get_git_status_info() -> GitStatusInfo;
pub fn get_diagnostics_info() -> DiagnosticInfo;
pub fn get_memory_info() -> String;
pub fn get_todo_list_info() -> String;
```

## 附件类型

```rust
pub enum AttachmentType {
    File,
    Image,
    Code,
    Url,
}
```

## 源码位置

`crates/aster/src/prompt/`
