# 记忆系统

## 概述

记忆系统提供对话记忆、压缩和管理功能，支持长期记忆存储。

**核心路径**: `crates/aster/src/memory/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `types.rs` | 类型定义 |
| `chat_memory.rs` | 对话记忆 |
| `compressor.rs` | 记忆压缩 |
| `memory_manager.rs` | 记忆管理器 |

## 核心类型

### MemoryEntry

```rust
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub importance: MemoryImportance,
    pub scope: MemoryScope,
    pub timestamp: Timestamp,
    pub emotion: Option<MemoryEmotion>,
    pub links: Vec<MemoryLink>,
}

pub enum MemoryImportance {
    Low,
    Medium,
    High,
    Critical,
}

pub enum MemoryScope {
    Session,    // 会话级
    Project,    // 项目级
    Global,     // 全局
}
```

### ConversationChunk

```rust
pub struct ConversationChunk {
    pub id: String,
    pub messages: Vec<ChunkMessage>,
    pub summary: Option<ConversationSummary>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

pub struct ChunkMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Timestamp,
}

pub enum MessageRole {
    User,
    Assistant,
    System,
}
```

### ConversationSummary

```rust
pub struct ConversationSummary {
    pub text: String,
    pub key_points: Vec<String>,
    pub topics: Vec<String>,
    pub created_at: Timestamp,
}
```

## ChatMemory

```rust
pub struct ChatMemory {
    chunks: Vec<ConversationChunk>,
    stats: ChatMemoryStats,
}

impl ChatMemory {
    pub fn new() -> Self;
    
    // 添加消息
    pub fn add_message(&mut self, role: MessageRole, content: &str);
    
    // 获取最近消息
    pub fn recent_messages(&self, count: usize) -> Vec<&ChunkMessage>;
    
    // 搜索记忆
    pub fn search(&self, query: &str) -> Vec<MemoryRecallResult>;
    
    // 获取统计
    pub fn stats(&self) -> &ChatMemoryStats;
}
```

## MemoryCompressor

```rust
pub struct MemoryCompressor {
    config: CompressorConfig,
}

pub struct CompressorConfig {
    pub max_chunk_size: usize,
    pub compression_threshold: usize,
    pub summary_model: Option<String>,
}

impl MemoryCompressor {
    pub fn new(config: CompressorConfig) -> Self;
    
    // 压缩对话块
    pub async fn compress_chunk(
        &self,
        chunk: &ConversationChunk,
    ) -> Result<CompressionResult>;
    
    // 按时间段压缩
    pub async fn compress_by_period(
        &self,
        chunks: &[ConversationChunk],
        period: Period,
    ) -> Result<Vec<CompressionResult>>;
}

pub enum Period {
    Hour,
    Day,
    Week,
    Month,
}

pub struct CompressionResult {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub summary: ConversationSummary,
}
```

## MemoryManager

```rust
pub struct MemoryManager {
    chat_memory: ChatMemory,
    compressor: MemoryCompressor,
}

impl MemoryManager {
    pub fn new() -> Self;
    
    // 记录消息
    pub fn record(&mut self, role: MessageRole, content: &str);
    
    // 回忆相关记忆
    pub fn recall(&self, query: &str, limit: usize) 
        -> Vec<MemoryRecallResult>;
    
    // 触发压缩
    pub async fn compact(&mut self) -> Result<()>;
    
    // 导出记忆
    pub fn export(&self) -> Vec<MemoryEntry>;
    
    // 导入记忆
    pub fn import(&mut self, entries: Vec<MemoryEntry>);
}
```

## 记忆存储

```rust
pub trait SimpleMemoryStore: Send + Sync {
    fn save(&self, entry: &MemoryEntry) -> Result<()>;
    fn load(&self, id: &str) -> Result<Option<MemoryEntry>>;
    fn search(&self, query: &str) -> Result<Vec<MemoryEntry>>;
    fn delete(&self, id: &str) -> Result<()>;
}

pub trait ChatMemoryStore: Send + Sync {
    fn save_chunk(&self, chunk: &ConversationChunk) -> Result<()>;
    fn load_chunks(&self, session_id: &str) -> Result<Vec<ConversationChunk>>;
}
```

## 记忆链接

```rust
pub struct MemoryLink {
    pub target_id: String,
    pub link_type: String,
    pub strength: f32,
}
```

支持记忆之间的关联，用于上下文检索。

## 用户画像

```rust
pub struct UserProfile {
    pub preferences: HashMap<String, String>,
    pub communication_style: CommunicationStyle,
    pub topics_of_interest: Vec<String>,
}

pub enum CommunicationStyle {
    Formal,
    Casual,
    Technical,
    Concise,
}
```

## 统计信息

```rust
pub struct ChatMemoryStats {
    pub total_messages: usize,
    pub total_chunks: usize,
    pub compressed_chunks: usize,
    pub total_tokens: usize,
}

pub struct MemoryStats {
    pub total_entries: usize,
    pub by_scope: HashMap<MemoryScope, usize>,
    pub by_importance: HashMap<MemoryImportance, usize>,
}
```
