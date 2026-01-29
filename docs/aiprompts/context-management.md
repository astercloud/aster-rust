# 上下文管理

## 概述

上下文管理系统负责对话历史的压缩和优化，防止超出模型上下文限制。

**核心路径**: `crates/aster/src/context_mgmt/`

## 核心常量

```rust
pub const DEFAULT_COMPACTION_THRESHOLD: f64 = 0.8;
```

当 Token 使用率超过 80% 时触发自动压缩。

## 压缩检查

```rust
pub async fn check_if_compaction_needed(
    provider: &dyn Provider,
    conversation: &Conversation,
    threshold_override: Option<f64>,
    session: &Session,
) -> Result<bool>;
```

检查流程：
1. 获取模型上下文限制
2. 计算当前 Token 使用量
3. 比较使用率与阈值

## 消息压缩

```rust
pub async fn compact_messages(
    provider: &dyn Provider,
    conversation: &Conversation,
    manual_compact: bool,
) -> Result<(Conversation, ProviderUsage)>;
```

### 压缩流程

```
原始对话
    │
    ▼
┌─────────────────────────────┐
│  过滤 Agent 可见消息         │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  渐进式移除工具响应          │
│  (0% → 10% → 20% → 50%)     │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  调用 LLM 生成摘要           │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  更新消息可见性元数据        │
│  - 原消息: user_visible only │
│  - 摘要: agent_visible only  │
└───────────┬─────────────────┘
            │
            ▼
压缩后对话
```

### 可见性处理

压缩后消息的可见性：

| 消息类型 | agent_visible | user_visible |
|----------|---------------|--------------|
| 原始消息 | ❌ | ✅ |
| 摘要消息 | ✅ | ❌ |
| 继续消息 | ✅ | ❌ |

## 继续提示

根据压缩场景使用不同的继续提示：

```rust
// 对话继续
const CONVERSATION_CONTINUATION_TEXT: &str =
    "The previous message contains a summary...
     Just continue the conversation naturally...";

// 工具循环继续
const TOOL_LOOP_CONTINUATION_TEXT: &str =
    "The previous message contains a summary...
     Continue calling tools as necessary...";

// 手动压缩
const MANUAL_COMPACT_CONTINUATION_TEXT: &str =
    "The previous message contains a summary...
     at the user's request...";
```

## 消息格式化

压缩时将消息格式化为文本：

```rust
fn format_message_for_compacting(msg: &Message) -> String {
    // Text → 原文
    // Image → [image: mime_type]
    // ToolRequest → tool_request(name): args
    // ToolResponse → tool_response: content
    // Thinking → thinking: text
}
```

## 渐进式移除

当上下文仍然过大时，渐进式移除工具响应：

```rust
let removal_percentages = [0, 10, 20, 50, 100];

for remove_percent in removal_percentages {
    let filtered = filter_tool_responses(&messages, remove_percent);
    // 尝试压缩...
}
```

从中间开始移除，保留首尾的工具响应。

## 配置

```toml
# 环境变量或配置文件
ASTER_AUTO_COMPACT_THRESHOLD = 0.8  # 0-1, 0 禁用
```

## 手动触发

```bash
# CLI 触发压缩
aster session compact --name my-session
```

或在对话中使用 `/compact` 命令。
