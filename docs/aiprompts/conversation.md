# 对话管理系统

## 概述

对话系统管理 Agent 与用户之间的消息交互，包括消息验证、修复和格式化。

**核心路径**: `crates/aster/src/conversation/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `mod.rs` | Conversation 核心实现 |
| `message.rs` | Message 类型定义 |
| `tool_result_serde.rs` | 工具结果序列化 |

## Conversation 结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation(Vec<Message>);

impl Conversation {
    pub fn new<I>(messages: I) -> Result<Self, InvalidConversation>;
    pub fn new_unvalidated<I>(messages: I) -> Self;
    pub fn empty() -> Self;
    
    // 访问方法
    pub fn messages(&self) -> &Vec<Message>;
    pub fn first(&self) -> Option<&Message>;
    pub fn last(&self) -> Option<&Message>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    
    // 修改方法
    pub fn push(&mut self, message: Message);
    pub fn pop(&mut self) -> Option<Message>;
    pub fn extend<I>(&mut self, iter: I);
    pub fn truncate(&mut self, len: usize);
    pub fn clear(&mut self);
    
    // 过滤方法
    pub fn agent_visible_messages(&self) -> Vec<Message>;
    pub fn user_visible_messages(&self) -> Vec<Message>;
}
```

## Message 结构

```rust
pub struct Message {
    pub id: Option<String>,
    pub role: Role,
    pub content: Vec<MessageContent>,
    pub metadata: MessageMetadata,
}

pub enum Role {
    User,
    Assistant,
}

pub struct MessageMetadata {
    pub agent_visible: bool,
    pub user_visible: bool,
    pub timestamp: Option<DateTime<Utc>>,
}
```

## MessageContent 类型

```rust
pub enum MessageContent {
    Text(TextContent),
    Image(ImageContent),
    ToolRequest(ToolRequest),
    ToolResponse(ToolResponse),
    ToolConfirmationRequest(ToolConfirmationRequest),
    FrontendToolRequest(FrontendToolRequest),
    Thinking(ThinkingContent),
    RedactedThinking(RedactedThinkingContent),
    ActionRequired(ActionRequiredData),
}
```

## 对话修复

发送给 LLM 前自动修复对话：

```rust
pub fn fix_conversation(
    conversation: Conversation
) -> (Conversation, Vec<String>);
```

### 修复规则

1. **合并文本内容** - 合并连续的文本块
2. **修剪空白** - 移除 Assistant 消息尾部空白
3. **移除空消息** - 删除无内容的消息
4. **修复工具调用** - 确保请求/响应配对
5. **合并连续消息** - 合并相同角色的连续消息
6. **修复首尾** - 确保首尾是 User 消息
7. **填充空对话** - 空对话添加占位消息

## Message 构建器

```rust
// 创建用户消息
let msg = Message::user()
    .with_text("Hello")
    .with_image(data, media_type);

// 创建助手消息
let msg = Message::assistant()
    .with_text("I'll help you")
    .with_tool_request(id, tool_call)
    .with_thinking(text, signature);

// 工具响应
let msg = Message::user()
    .with_tool_response(id, result);
```

## 可见性过滤

```rust
// 获取 Agent 可见消息
let agent_msgs = conversation.agent_visible_messages();

// 获取用户可见消息  
let user_msgs = conversation.user_visible_messages();

// 自定义过滤
let filtered = conversation.filtered_messages(|meta| {
    meta.agent_visible && meta.timestamp.is_some()
});
```

## 有效角色

```rust
pub fn effective_role(message: &Message) -> String {
    if message.role == Role::User && has_tool_response(message) {
        "tool".to_string()
    } else {
        match message.role {
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
        }
    }
}
```

工具响应虽然在 User 消息中，但有效角色是 "tool"。
