# Streaming 流式处理

提供全面的流式处理支持。

## 模块结构

```
streaming/
├── message_stream.rs  # 消息流处理
├── sse.rs             # SSE 解析
└── stream_io.rs       # 流式 I/O
```

## SSE 解析

### SSEDecoder
```rust
pub struct SSEDecoder;

impl SSEDecoder {
    pub fn decode(data: &[u8]) -> Vec<SSEEvent>;
}

pub struct SSEEvent {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
}
```

## 消息流

### EnhancedMessageStream
```rust
pub struct EnhancedMessageStream {
    pub state: MessageState,
    pub callbacks: StreamCallbacks,
    pub options: StreamOptions,
}
```


### StreamOptions
```rust
pub struct StreamOptions {
    pub timeout: Duration,
    pub backpressure_limit: usize,
    pub buffer_size: usize,
}
```

## 流式 I/O

### StreamJsonReader/Writer
用于 CLI 通信的 JSON 流：
```rust
pub struct StreamJsonReader<R>;
pub struct StreamJsonWriter<W>;
pub struct StreamSession;
```

## 内容块类型

```rust
pub enum ContentBlock {
    Text(String),
    ToolUse { id: String, name: String, input: Value },
    ToolResult { id: String, content: String },
}
```

## 使用场景

- AI 响应流式输出
- CLI 与 Server 通信
- 实时进度显示

## 源码位置

`crates/aster/src/streaming/`
