# 工具系统

## 概述

工具系统提供了 Agent 执行各种操作的能力，包括原生工具和 MCP 工具。

**核心路径**: `crates/aster/src/tools/`

## 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        ToolRegistry                              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Native Tools                              ││
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐    ││
│  │  │ Bash   │ │ Read   │ │ Write  │ │ Edit   │ │ Glob   │    ││
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘    ││
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐    ││
│  │  │ Grep   │ │ Ask    │ │ LSP    │ │ Skill  │ │ Task   │    ││
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘    ││
│  │  ┌────────┐ ┌────────┐ ┌────────────┐ ┌──────────────────┐ ││
│  │  │WebFetch│ │WebSearch│ │AnalyzeImage│ │ NotebookEdit    │ ││
│  │  └────────┘ └────────┘ └────────────┘ └──────────────────┘ ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     MCP Tools                                ││
│  │  (动态从 MCP 服务器加载)                                      ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Tool Trait

```rust
// crates/aster/src/tools/base.rs
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: &ToolContext,
    ) -> ToolResult<serde_json::Value>;
    
    fn permission_behavior(&self) -> PermissionBehavior {
        PermissionBehavior::RequiresApproval
    }
}

pub enum PermissionBehavior {
    ReadOnly,           // 只读，无需审批
    RequiresApproval,   // 需要审批
    AlwaysAllow,        // 始终允许
}
```

## 原生工具列表

| 工具 | 模块 | 说明 |
|------|------|------|
| bash | `bash.rs` | Shell 命令执行 |
| read | `file.rs` | 文件读取 (文本/图片/PDF/Notebook) |
| write | `file.rs` | 文件写入 |
| edit | `file.rs` | 智能文件编辑 |
| glob | `search.rs` | Glob 模式文件搜索 |
| grep | `search.rs` | 正则内容搜索 |
| ask | `ask.rs` | 用户交互 |
| lsp | `lsp.rs` | 代码智能 |
| Skill | `skills/` | 技能执行 |
| Task | `task_tool.rs` | 后台任务 |
| TaskOutput | `task_output_tool.rs` | 任务输出 |
| KillShell | `kill_shell_tool.rs` | 终止 Shell |
| TodoWrite | `todo_write_tool.rs` | TODO 管理 |
| NotebookEdit | `notebook_edit_tool.rs` | Notebook 编辑 |
| WebFetch | `web.rs` | 网页获取 |
| WebSearch | `web.rs` | 网页搜索 |
| analyze_image | `analyze_image.rs` | 图片分析 |
| EnterPlanMode | `plan_mode_tool.rs` | 进入计划模式 |
| ExitPlanMode | `plan_mode_tool.rs` | 退出计划模式 |

## 工具注册

```rust
// 默认注册
let mut registry = ToolRegistry::new();
let (history, hook_manager) = register_default_tools(&mut registry);

// 自定义配置
let config = ToolRegistrationConfig::new()
    .with_ask_callback(ask_callback)
    .with_lsp_callback(lsp_callback)
    .with_pdf_enabled(true)
    .with_hooks_enabled(true);
let (history, hook_manager) = register_all_tools(&mut registry, config);
```

## 工具上下文

```rust
pub struct ToolContext {
    pub working_dir: PathBuf,
    pub session_id: String,
    pub cancellation_token: Option<CancellationToken>,
    pub permission_callback: Option<PermissionRequestCallback>,
}
```

## Bash 工具

```rust
pub struct BashTool {
    sandbox_config: SandboxConfig,
}

// 安全检查
pub enum SafetyCheckResult {
    Safe,
    RequiresConfirmation(String),
    Blocked(String),
}

// 输出限制
pub const MAX_OUTPUT_LENGTH: usize = 100_000;
```

## 文件工具

```rust
// 共享读取历史
pub type SharedFileReadHistory = Arc<RwLock<FileReadHistory>>;

pub struct FileReadHistory {
    records: HashMap<PathBuf, FileReadRecord>,
}

pub struct FileReadRecord {
    pub path: PathBuf,
    pub content_hash: String,
    pub size: u64,
    pub read_at: Instant,
}
```

## 搜索工具

```rust
// Glob 搜索
pub struct GlobTool;

// Grep 搜索
pub struct GrepTool {
    output_mode: GrepOutputMode,
}

pub enum GrepOutputMode {
    Default,
    Context(usize),  // 上下文行数
    FilesOnly,
}

pub const DEFAULT_MAX_RESULTS: usize = 100;
pub const DEFAULT_MAX_CONTEXT_LINES: usize = 2;
pub const MAX_OUTPUT_SIZE: usize = 50_000;
```

## Hook 系统

```rust
// crates/aster/src/tools/hooks.rs
pub trait ToolHook: Send + Sync {
    async fn before_execute(&self, context: &HookContext) -> Result<()>;
    async fn after_execute(&self, context: &HookContext, result: &ToolResult) -> Result<()>;
}

// 内置 Hook
pub struct LoggingHook;
pub struct ErrorTrackingHook;
pub struct FileOperationHook;
```

## MCP 工具包装

```rust
pub struct McpToolWrapper {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    server_name: String,
}

impl Tool for McpToolWrapper {
    // 委托给 MCP 服务器执行
}
```
