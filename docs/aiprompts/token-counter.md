# Token Counter

基于 tiktoken 的 Token 计数器。

## 核心类型

```rust
pub struct TokenCounter {
    tokenizer: Arc<CoreBPE>,
    token_cache: Arc<DashMap<u64, usize>>,
}
```

## 主要方法

```rust
impl TokenCounter {
    pub async fn new() -> Result<Self, String>;
    pub fn count_tokens(&self, text: &str) -> usize;
    pub fn count_tokens_for_tools(&self, tools: &[Tool]) -> usize;
    pub fn count_chat_tokens(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> usize;
    pub fn clear_cache(&self);
    pub fn cache_size(&self) -> usize;
}
```

## 工厂函数

```rust
pub async fn create_token_counter() -> Result<TokenCounter, String>;
```


## 特性

- 使用 o200k_base tokenizer
- 带缓存的 token 计数
- 支持工具定义计数
- 线程安全

## 缓存管理

- 最大缓存大小: 10,000 条
- 自动 LRU 淘汰
- 可手动清除

## 使用场景

- 上下文窗口管理
- API 成本估算
- 消息截断决策

## 源码位置

`crates/aster/src/token_counter.rs`
