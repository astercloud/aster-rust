# Aster-Rust 改进分析：从 OpenClaw 学习

> 本文档分析 OpenClaw 的架构优势，对比 Aster-Rust 现有能力，提出改进建议。

## 一、能力对比总览

| 能力领域 | OpenClaw | Aster-Rust | 差距分析 |
|---------|----------|------------|---------|
| **Agent Loop** | ✅ 完整 | ✅ 完整 | 基本持平 |
| **Tool Execution** | ✅ 完整 | ✅ 完整 | 基本持平 |
| **Tool Policy** | ✅ 多层策略 | ⚠️ 基础权限 | **需要增强** |
| **Hooks System** | ✅ 事件驱动 | ⚠️ 工具钩子 | **需要增强** |
| **Sandbox** | ✅ Docker 沙箱 | ✅ 进程沙箱 | 基本持平 |
| **Memory** | ✅ 完整 | ✅ 完整 | 基本持平 |
| **Context** | ✅ 完整 | ✅ 完整 | 基本持平 |
| **Session** | ✅ 完整 | ✅ 完整 | 基本持平 |
| **Scheduler** | ✅ 丰富调度 | ⚠️ 基础 Cron | **需要增强** |
| **Channel Adapter** | ✅ 多渠道 | ❌ 无 | **平台层能力** |
| **Gateway Server** | ✅ WebSocket | ❌ 无 | **平台层能力** |
| **Session Router** | ✅ 智能路由 | ❌ 无 | **平台层能力** |
| **Auto-Reply** | ✅ 触发机制 | ❌ 无 | **需要引入** |
| **Plugin System** | ✅ 完整 | ✅ 完整 | 基本持平 |

---

## 二、需要改进的核心能力

### 2.1 Tool Policy 系统（P0 优先级）

**OpenClaw 实现特点**：

```typescript
// OpenClaw 的多层 Tool Policy
const TOOL_PROFILES = {
  minimal: { allow: ["session_status"] },
  coding: { allow: ["group:fs", "group:runtime", "group:sessions"] },
  messaging: { allow: ["group:messaging", "sessions_list"] },
  full: {},  // 允许所有
};

const TOOL_GROUPS = {
  "group:fs": ["read", "write", "edit", "apply_patch"],
  "group:runtime": ["exec", "process"],
  "group:sessions": ["sessions_list", "sessions_history", ...],
  "group:web": ["web_search", "web_fetch"],
  "group:memory": ["memory_search", "memory_get"],
};
```

**OpenClaw 的策略层级**：
1. `profilePolicy` - 预设配置文件策略
2. `globalPolicy` - 全局策略
3. `agentPolicy` - Agent 级别策略
4. `groupPolicy` - 群组/渠道策略
5. `sandboxPolicy` - 沙箱策略
6. `subagentPolicy` - 子 Agent 策略

**Aster-Rust 现状**：
- 有 `permission/` 模块，支持工具权限检查
- 但缺少**预设配置文件**（Profile）概念
- 缺少**工具分组**（Tool Groups）概念
- 缺少**多层策略合并**机制


**改进建议**：

```rust
// 建议在 Aster-Rust 中添加 Tool Policy 系统

/// 工具配置文件预设
pub enum ToolProfile {
    Minimal,    // 最小权限：仅状态查询
    Coding,     // 编码模式：文件操作 + 执行
    Messaging,  // 消息模式：会话管理
    Full,       // 完整权限
    Custom(String),  // 自定义配置
}

/// 工具分组定义
pub struct ToolGroups {
    groups: HashMap<String, Vec<String>>,
}

impl Default for ToolGroups {
    fn default() -> Self {
        let mut groups = HashMap::new();
        groups.insert("group:fs".into(), vec!["read", "write", "edit"]);
        groups.insert("group:runtime".into(), vec!["exec", "process"]);
        groups.insert("group:memory".into(), vec!["memory_search", "memory_get"]);
        groups.insert("group:web".into(), vec!["web_search", "web_fetch"]);
        Self { groups }
    }
}

/// 多层策略合并器
pub struct ToolPolicyMerger {
    profile_policy: Option<ToolPolicy>,
    global_policy: Option<ToolPolicy>,
    agent_policy: Option<ToolPolicy>,
    session_policy: Option<ToolPolicy>,
}
```

---

### 2.2 Hooks 事件系统（P1 优先级）

**OpenClaw 实现特点**：

```typescript
// OpenClaw 的事件类型
type HookEventType = "command" | "session" | "agent" | "gateway";

// 事件结构
interface HookEvent {
  type: HookEventType;
  action: string;           // 如 "new", "reset", "stop"
  sessionKey: string;
  context: Record<string, unknown>;
  timestamp: Date;
  messages: string[];       // 钩子可以向用户推送消息
}

// 注册钩子
registerHook("command:new", async (event) => {
  await saveSessionToMemory(event);
});

registerHook("agent:bootstrap", async (event) => {
  await loadWorkspaceFiles(event);
});
```

**Aster-Rust 现状**：
- 有 `hooks/` 模块，但主要是**工具调用前后的钩子**
- 缺少**Agent 生命周期事件**（启动、停止、错误）
- 缺少**Session 事件**（创建、恢复、结束）
- 缺少**命令事件**（用户命令解析）


**改进建议**：

```rust
// 建议扩展 Aster-Rust 的 Hooks 系统

/// 事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum HookEventType {
    Agent,      // Agent 生命周期
    Session,    // 会话事件
    Tool,       // 工具调用（现有）
    Command,    // 用户命令
    Gateway,    // 网关事件（平台层）
}

/// 事件动作
pub enum HookAction {
    // Agent 事件
    AgentStart,
    AgentStop,
    AgentError,
    AgentBootstrap,
    
    // Session 事件
    SessionCreate,
    SessionResume,
    SessionEnd,
    SessionCompact,
    
    // Tool 事件（现有）
    ToolBefore,
    ToolAfter,
    ToolError,
    
    // Command 事件
    CommandNew,
    CommandReset,
    CommandStatus,
}

/// 钩子事件
pub struct HookEvent {
    pub event_type: HookEventType,
    pub action: HookAction,
    pub session_id: Option<String>,
    pub context: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<String>,  // 钩子可以推送消息
}

/// 钩子注册表
pub struct HookRegistry {
    handlers: HashMap<String, Vec<Box<dyn HookHandler>>>,
}

impl HookRegistry {
    /// 注册钩子
    pub fn register(&mut self, event_key: &str, handler: impl HookHandler) {
        // event_key 格式: "agent:start", "session:create", "tool:before"
    }
    
    /// 触发钩子
    pub async fn trigger(&self, event: HookEvent) -> Result<()> {
        // 1. 触发通用类型钩子 (如 "agent")
        // 2. 触发具体动作钩子 (如 "agent:start")
    }
}
```

---

### 2.3 Auto-Reply 触发机制（P2 优先级）

**OpenClaw 实现特点**：

```typescript
// OpenClaw 的自动回复层
// 1. 入站处理 - 白名单检查、群组激活检测
// 2. 命令解析 - /new /think /status 等
// 3. 触发条件 - 关键词、@提及、私聊

// 配置示例
autoReply: {
  triggers: [
    { type: "mention", enabled: true },
    { type: "keyword", patterns: ["help", "问题"] },
    { type: "dm", enabled: true },
  ],
  whitelist: ["user1", "user2"],
  groupActivation: {
    requireMention: true,
    cooldownSeconds: 60,
  }
}
```

**Aster-Rust 现状**：
- 无自动回复机制
- 所有消息都需要显式调用 Agent

**改进建议**：

```rust
/// 自动回复触发器
pub struct AutoReplyTrigger {
    pub trigger_type: TriggerType,
    pub enabled: bool,
    pub config: TriggerConfig,
}

pub enum TriggerType {
    Mention,        // @提及
    Keyword,        // 关键词匹配
    DirectMessage,  // 私聊
    Schedule,       // 定时触发
    Webhook,        // 外部触发
}

/// 自动回复管理器
pub struct AutoReplyManager {
    triggers: Vec<AutoReplyTrigger>,
    whitelist: HashSet<String>,
    cooldown: Duration,
}

impl AutoReplyManager {
    /// 检查消息是否应该触发自动回复
    pub fn should_reply(&self, message: &IncomingMessage) -> bool {
        // 1. 检查白名单
        // 2. 检查冷却时间
        // 3. 匹配触发条件
    }
}
```

---

### 2.4 Cron/Scheduler 系统增强（P1 优先级）

**OpenClaw 实现特点**：

```typescript
// OpenClaw 的 Cron 调度类型
type CronSchedule =
  | { kind: "at"; atMs: number }           // 一次性定时
  | { kind: "every"; everyMs: number }     // 固定间隔
  | { kind: "cron"; expr: string; tz?: string };  // Cron 表达式

// OpenClaw 的任务载荷类型
type CronPayload =
  | { kind: "systemEvent"; text: string }  // 系统事件
  | { kind: "agentTurn";                   // Agent 执行
      message: string;
      model?: string;           // 模型覆盖
      thinking?: string;        // 思考级别
      timeoutSeconds?: number;  // 超时
      deliver?: boolean;        // 是否投递结果
      channel?: string;         // 投递渠道
      to?: string;              // 投递目标
    };

// OpenClaw 的会话隔离
type CronIsolation = {
  postToMainPrefix?: string;
  postToMainMode?: "summary" | "full";  // 结果回传模式
  postToMainMaxChars?: number;
};
```

**OpenClaw Cron 的高级特性**：
1. **多种调度类型** - 一次性、固定间隔、Cron 表达式
2. **时区支持** - 可指定时区
3. **会话隔离** - isolated session 执行，结果回传 main session
4. **结果投递** - 执行结果可投递到指定渠道
5. **模型覆盖** - 每个任务可指定不同模型
6. **安全边界** - 外部 Hook 内容的安全包装

**Aster-Rust 现状**：
- ✅ 有 `scheduler.rs`，支持 Cron 表达式
- ✅ 支持任务暂停/恢复/删除
- ✅ 支持立即执行（run_now）
- ❌ 缺少**一次性定时**（at）和**固定间隔**（every）
- ❌ 缺少**会话隔离**执行
- ❌ 缺少**结果投递**机制
- ❌ 缺少**时区配置**

**改进建议**：

```rust
/// 扩展调度类型
pub enum ScheduleType {
    /// 一次性定时执行
    At { at_ms: i64 },
    /// 固定间隔执行
    Every { every_ms: u64, anchor_ms: Option<u64> },
    /// Cron 表达式
    Cron { expr: String, timezone: Option<String> },
}

/// 任务载荷类型
pub enum CronPayload {
    /// 系统事件（简单文本）
    SystemEvent { text: String },
    /// Agent 执行
    AgentTurn {
        message: String,
        model_override: Option<String>,
        thinking_level: Option<String>,
        timeout_seconds: Option<u64>,
        deliver: Option<DeliveryConfig>,
    },
}

/// 结果投递配置
pub struct DeliveryConfig {
    pub enabled: bool,
    pub channel: Option<String>,
    pub to: Option<String>,
    pub best_effort: bool,
}

/// 会话隔离配置
pub struct IsolationConfig {
    pub enabled: bool,
    pub post_to_main_mode: PostToMainMode,
    pub post_to_main_max_chars: usize,
}

pub enum PostToMainMode {
    Summary,  // 摘要
    Full,     // 完整输出
}

/// 增强的 ScheduledJob
pub struct ScheduledJob {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub delete_after_run: bool,  // 一次性任务
    pub schedule: ScheduleType,
    pub payload: CronPayload,
    pub isolation: Option<IsolationConfig>,
    pub state: JobState,
}
```

---

## 三、平台层能力（Aster Platform 需要实现）

以下能力不属于框架层，应该在未来的 Aster Platform 项目中实现：

### 3.1 Channel Adapter（渠道适配器）

**OpenClaw 实现**：
- `src/telegram/` - Telegram 适配
- `src/discord/` - Discord 适配
- `src/slack/` - Slack 适配
- `src/whatsapp/` - WhatsApp 适配
- `src/signal/` - Signal 适配

**平台层设计建议**：

```rust
/// 渠道适配器 trait
pub trait ChannelAdapter: Send + Sync {
    /// 渠道标识
    fn channel_id(&self) -> &str;
    
    /// 接收消息
    async fn receive(&self) -> Result<IncomingMessage>;
    
    /// 发送消息
    async fn send(&self, message: OutgoingMessage) -> Result<()>;
    
    /// 消息格式化
    fn format_message(&self, content: &str) -> String;
}

/// 消息标准化
pub struct StandardMessage {
    pub channel: String,
    pub sender_id: String,
    pub content: String,
    pub attachments: Vec<Attachment>,
    pub metadata: MessageMetadata,
}
```


### 3.2 Gateway Server（网关服务器）

**OpenClaw 实现**：
- WebSocket RPC 控制中心
- 连接管理
- 消息路由

**平台层设计建议**：

```rust
/// 网关服务器
pub struct GatewayServer {
    channels: HashMap<String, Box<dyn ChannelAdapter>>,
    router: SessionRouter,
    agent_pool: AgentPool,
}

impl GatewayServer {
    /// 启动网关
    pub async fn start(&self, addr: SocketAddr) -> Result<()>;
    
    /// 处理入站消息
    async fn handle_inbound(&self, msg: StandardMessage) -> Result<()> {
        // 1. 路由到正确的 Session
        // 2. 排队控制
        // 3. 调用 Agent
        // 4. 返回响应
    }
}
```

### 3.3 Session Router（会话路由器）

**OpenClaw 实现**：
- 根据 channel/peer/guild 路由
- 会话粘性
- 负载均衡

**平台层设计建议**：

```rust
/// 会话路由器
pub struct SessionRouter {
    bindings: Vec<RouteBinding>,
    dm_scope: DmScope,
}

pub struct RouteBinding {
    pub match_rule: MatchRule,
    pub agent_id: String,
}

pub enum MatchRule {
    Channel(String),
    Peer { kind: PeerKind, id: String },
    Guild(String),
    Team(String),
    Account(String),
}

impl SessionRouter {
    /// 解析路由
    pub fn resolve(&self, input: &RouteInput) -> ResolvedRoute {
        // 1. 匹配 peer
        // 2. 匹配 guild/team
        // 3. 匹配 account
        // 4. 匹配 channel
        // 5. 默认路由
    }
}
```

---

## 四、改进优先级和实施计划

### P0 - 必须改进（框架核心能力）

| 改进项 | 现有模块 | 改进内容 | 工作量 |
|-------|---------|---------|-------|
| Tool Policy 增强 | `permission/` | 添加 Profile、Groups、多层合并 | 2 周 |

### P1 - 建议改进（增强框架能力）

| 改进项 | 现有模块 | 改进内容 | 工作量 |
|-------|---------|---------|-------|
| Hooks 事件扩展 | `hooks/` | 添加 Agent/Session/Command 事件 | 1 周 |
| Cron/Scheduler 增强 | `scheduler.rs` | 添加 at/every 调度、会话隔离、结果投递 | 1.5 周 |

### P2 - 可选改进（扩展框架能力）

| 改进项 | 现有模块 | 改进内容 | 工作量 |
|-------|---------|---------|-------|
| Auto-Reply | 无 | 新增自动回复触发机制 | 1 周 |

### 平台层（未来新项目）

| 能力 | 说明 | 依赖 |
|-----|------|-----|
| Channel Adapter | 多渠道消息适配 | Aster-Rust |
| Gateway Server | WebSocket 网关 | Aster-Rust |
| Session Router | 会话路由 | Aster-Rust |
| User Management | 用户管理 | Aster-Rust |

---

## 五、总结

### Aster-Rust 框架层改进重点

1. **Tool Policy 系统**（P0）
   - 引入 Profile 预设概念
   - 引入 Tool Groups 分组
   - 实现多层策略合并

2. **Hooks 事件系统**（P1）
   - 扩展事件类型（Agent/Session/Command）
   - 支持事件消息推送
   - 支持异步钩子处理

3. **Cron/Scheduler 增强**（P1）
   - 添加一次性定时（at）和固定间隔（every）调度
   - 实现会话隔离执行
   - 支持结果投递到指定渠道
   - 添加时区配置支持

4. **Auto-Reply 机制**（P2）
   - 触发条件配置
   - 白名单/冷却控制
   - 与 Scheduler 集成

### 不属于框架层的能力

以下能力应该在 **Aster Platform** 平台层实现：
- Channel Adapter（多渠道适配）
- Gateway Server（网关服务）
- Session Router（会话路由）
- User Management（用户管理）

### 与 OpenClaw 的定位差异

| 维度 | OpenClaw | Aster-Rust |
|-----|----------|-----------|
| 定位 | 完整的多渠道 AI 助手 | AI Agent 框架 |
| 架构 | 单体应用 | 框架 + 平台 + 应用 分层 |
| 渠道 | 内置多渠道支持 | 框架不含渠道，平台层实现 |
| 部署 | CLI + Gateway | 框架库 + 独立平台服务 |
