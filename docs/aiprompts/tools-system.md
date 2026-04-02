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
│  │  ┌────────┐ ┌────────────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ││
│  │  │ Grep   │ │ AskUserQuestion│ │ Config │ │ LSP    │ │ Skill  │ ││
│  │  └────────┘ └────────────────┘ └────────┘ └────────┘ └────────┘ ││
│  │  ┌────────┐ ┌────────┐                                           ││
│  │  │ Sleep  │ │ Task*  │                                           ││
│  │  └────────┘ └────────┘                                           ││
│  │  ┌────────┐ ┌────────┐ ┌────────────┐ ┌──────────────────┐ ││
│  │  │WebFetch│ │WebSearch│ │AnalyzeImage│ │ NotebookEdit    │ ││
│  │  └────────┘ └────────┘ └────────────┘ └──────────────────┘ ││
│  │  ┌──────────┐ ┌───────────────────────┐                    ││
│  │  │ToolSearch│ │List/Read MCP Resource │                    ││
│  │  └──────────┘ └───────────────────────┘                    ││
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
| AskUserQuestion | `ask.rs` | 用户交互 |
| Config | `config_tool.rs` | 读取/更新当前支持的运行时配置 |
| SendUserMessage | `send_user_message_tool.rs` | 发送一条用户可见但不会回流到 agent 上下文的进度/提醒消息 |
| Sleep | `sleep_tool.rs` | 按指定时长等待，可被取消 |
| PowerShell | `powershell_tool.rs` | 在运行环境可用时执行 PowerShell 命令 |
| lsp | `lsp.rs` | 代码智能 |
| Skill | `skills/` | 技能执行 |
| Workflow | `workflow_tool.rs` | 执行 `execution_mode=workflow` 的当前 workflow skill，并复用当前 session provider |
| TaskCreate / TaskList / TaskGet / TaskUpdate | `task_list_tools.rs` | 任务板管理 |
| TaskOutput | `task_output_tool.rs` | 任务输出 |
| TaskStop | `task_stop_tool.rs` | 终止后台任务 |
| NotebookEdit | `notebook_edit_tool.rs` | Notebook 编辑 |
| EnterWorktree | `worktree_tools.rs` | 创建隔离 git worktree 并切换当前 session 到该工作目录 |
| ExitWorktree | `worktree_tools.rs` | 退出当前 session 创建的 worktree，可保留或删除工作树 |
| ToolSearch | `tool_search_tool.rs` | 搜索并激活延迟工具 |
| ListMcpResourcesTool | `mcp_resource_tools.rs` | 列出 MCP 资源 |
| ReadMcpResourceTool | `mcp_resource_tools.rs` | 读取指定 MCP 资源 |
| CronCreate | `cron_tools.rs` | 创建 cron prompt 定时任务，支持 recurring / one-shot 与 durable / session-only |
| CronList | `cron_tools.rs` | 列出当前 cron 定时任务 |
| CronDelete | `cron_tools.rs` | 删除指定 cron 定时任务 |
| WebFetch | `web.rs` | 网页获取 |
| WebSearch | `web.rs` | 网页搜索 |
| analyze_image | `analyze_image.rs` | 图片分析 |
| EnterPlanMode | `plan_mode_tool.rs` | 进入计划模式 |
| ExitPlanMode | `plan_mode_tool.rs` | 退出计划模式 |

## 现代代理运行时工具

当宿主注入现代 agent runtime callback 时，会额外注册这组 current tools：

| 工具 | 模块 | 说明 |
|------|------|------|
| spawn_agent | `agent_control.rs` | 创建真实子代理会话并异步开始首条任务 |
| SendMessage | `agent_control.rs` | 向子代理发送消息；在活跃 team 中支持按成员名字或 `*` 广播路由 |
| wait_agent | `agent_control.rs` | 等待一个或多个子代理进入最终状态 |
| resume_agent | `agent_control.rs` | 恢复已关闭的子代理 |
| close_agent | `agent_control.rs` | 关闭子代理并回收其 team roster 成员 |

默认 native tools 还会注册 current `Workflow`：

| 工具 | 模块 | 说明 |
|------|------|------|
| Workflow | `workflow_tool.rs` | 执行 workflow skill；只接受 `execution_mode=workflow` 的 skill，不再暴露旧的 workflow 示例 surface |

当 `spawn_agent` 与 `SendMessage` 同时可用时，还会注册这组 team current tools：

| 工具 | 模块 | 说明 |
|------|------|------|
| TeamCreate | `team_tools.rs` | 创建共享 team 协作上下文，并把 team 名称作为共享 task list id |
| TeamDelete | `team_tools.rs` | 删除当前 team；若仍有已注册成员则拒绝删除 |
| ListPeers | `team_tools.rs` | 列出当前 team 中可通过 `SendMessage` 直接通信的成员 |

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

在 Agent 默认构造路径里，还会额外注册与扩展工具面相关的 current native tools：
- `Agent`（当前默认委派入口；旧的 `subagent` surface 不再暴露）
- `AskUserQuestion`（默认通过 elicitation 回调桥接到用户输入流程）
- `Config`
- `Sleep`
- `ToolSearch`
- `ListMcpResourcesTool`
- `ReadMcpResourceTool`
- `StructuredOutput`（存在输出 schema 时动态注入，用于返回最终结构化结果）

当 `Agent` 注入 scheduler 服务后，还会额外注册这组 current cron tools：
- `CronCreate`
- `CronList`
- `CronDelete`

当注入现代 agent runtime callback 时，任务板和 team 也会走同一份 session 扩展状态：
- `TeamCreate` 写入 `team_session.v0`
- 被注册进 team 的子代理会写入 `team_membership.v0`
- `TaskCreate / TaskList / TaskGet / TaskUpdate` 会优先读取 team 名称作为共享 task list id

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
