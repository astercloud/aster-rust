# Agent 系统架构

## 概述

Agent 是 Aster 的核心组件，负责协调 AI Provider、工具执行、会话管理等功能。

**核心文件**: `crates/aster/src/agents/agent.rs`

## Agent 结构

```rust
pub struct Agent {
    // Provider 管理
    provider: SharedProvider,
    
    // 扩展管理
    extension_manager: Arc<ExtensionManager>,
    
    // 子 Recipe 支持
    sub_recipes: Mutex<HashMap<String, SubRecipe>>,
    
    // 最终输出工具
    final_output_tool: Arc<Mutex<Option<FinalOutputTool>>>,
    
    // 前端工具
    frontend_tools: Mutex<HashMap<String, FrontendTool>>,
    
    // Prompt 管理
    prompt_manager: Mutex<PromptManager>,
    
    // 确认通道
    confirmation_tx: mpsc::Sender<(String, PermissionConfirmation)>,
    confirmation_rx: Mutex<mpsc::Receiver<(String, PermissionConfirmation)>>,
    
    // 工具结果通道
    tool_result_tx: mpsc::Sender<(String, ToolResult<CallToolResult>)>,
    tool_result_rx: ToolResultReceiver,
    
    // 调度服务
    scheduler_service: Mutex<Option<Arc<dyn SchedulerTrait>>>,
    
    // 重试管理
    retry_manager: RetryManager,
    
    // 工具检查
    tool_inspection_manager: ToolInspectionManager,
    
    // 工具注册表
    tool_registry: Arc<RwLock<ToolRegistry>>,
    
    // 文件读取历史
    file_read_history: SharedFileReadHistory,
}
```

## 核心方法

### 1. 创建 Agent

```rust
// 默认创建
let agent = Agent::new();

// 自定义工具配置
let config = ToolRegistrationConfig::new()
    .with_pdf_enabled(true)
    .with_hooks_enabled(true);
let agent = Agent::with_tool_config(config);
```

### 2. 回复处理

```rust
pub async fn reply(
    &self,
    user_message: Message,
    session_config: SessionConfig,
    cancel_token: Option<CancellationToken>,
) -> Result<BoxStream<'_, Result<AgentEvent>>>
```

### 3. 工具调度

```rust
pub async fn dispatch_tool_call(
    &self,
    tool_call: CallToolRequestParam,
    request_id: String,
    cancellation_token: Option<CancellationToken>,
    session: &Session,
) -> (String, Result<ToolCallResult, ErrorData>)
```

## Agent 事件

```rust
pub enum AgentEvent {
    Message(Message),
    McpNotification((String, ServerNotification)),
    ModelChange { model: String, mode: String },
    HistoryReplaced(Conversation),
}
```

## 工具检查流程

```
工具调用请求
    │
    ▼
┌─────────────────────────────────────┐
│     ToolInspectionManager           │
│  ┌─────────────────────────────┐    │
│  │ SecurityInspector (最高优先) │    │
│  └─────────────────────────────┘    │
│  ┌─────────────────────────────┐    │
│  │ PermissionInspector         │    │
│  └─────────────────────────────┘    │
│  ┌─────────────────────────────┐    │
│  │ RepetitionInspector         │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
    │
    ▼
执行或拒绝
```

## 子模块

| 模块 | 路径 | 说明 |
|------|------|------|
| context | `agents/context/` | 上下文管理、继承、隔离 |
| communication | `agents/communication/` | Agent 间通信、消息总线 |
| parallel | `agents/parallel/` | 并行执行、依赖管理 |
| monitor | `agents/monitor/` | 指标收集、告警 |
| resume | `agents/resume/` | 状态持久化、检查点 |
| specialized | `agents/specialized/` | Explore/Plan Agent |
| error_handling | `agents/error_handling/` | 统一错误处理 |

## 扩展管理

```rust
// 添加扩展
agent.add_extension(ExtensionConfig::Mcp { ... }).await?;

// 列出扩展
let extensions = agent.list_extensions().await;

// 移除扩展
agent.remove_extension("extension_name").await?;
```

## 会话配置

```rust
pub struct SessionConfig {
    pub id: String,
    pub working_dir: PathBuf,
    pub max_turns: Option<u32>,
    pub retry_config: Option<RetryConfig>,
    pub success_checks: Vec<SuccessCheck>,
}
```
