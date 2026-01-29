# Context 上下文管理

全面的上下文管理功能。

## 模块结构

```
context/
├── agents_md_parser.rs  # AGENTS.md 解析
├── cache_controller.rs  # 缓存控制
├── compressor.rs        # 消息压缩
├── file_mention.rs      # 文件引用解析
├── manager.rs           # 上下文管理器
├── priority_sorter.rs   # 优先级排序
├── summarizer.rs        # 消息摘要
├── token_estimator.rs   # Token 估算
├── types.rs             # 类型定义
└── window_manager.rs    # 窗口管理
```

## 核心功能

- Token 估算
- 动态上下文窗口管理
- 智能消息摘要
- 消息压缩
- 提示词缓存
- 消息优先级排序
- 文件引用解析
- AGENTS.md 解析


## EnhancedContextManager

```rust
let mut manager = EnhancedContextManager::new(ContextConfig::default());
manager.set_system_prompt("You are a helpful assistant.");
manager.add_turn(user_message, assistant_message, Some(usage));
let messages = manager.get_messages();
```

## Token 估算

```rust
pub struct TokenEstimator;

// 不同内容类型的估算常量
pub const CHARS_PER_TOKEN_DEFAULT: f32;
pub const CHARS_PER_TOKEN_ASIAN: f32;
pub const CHARS_PER_TOKEN_CODE: f32;
```

## 消息压缩

```rust
pub struct MessageCompressor;

pub fn compress_code_block(code: &str, max_lines: usize);
pub fn compress_message(msg: &Message, config: &CompressionConfig);
```

## 源码位置

`crates/aster/src/context/`
