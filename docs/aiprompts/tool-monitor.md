# Tool Monitor

工具调用监控和重复检测。

## RepetitionInspector

```rust
pub struct RepetitionInspector {
    max_repetitions: Option<u32>,
    last_call: Option<InternalToolCall>,
    repeat_count: u32,
    call_counts: HashMap<String, u32>,
}

impl RepetitionInspector {
    pub fn new(max_repetitions: Option<u32>) -> Self;
    pub fn check_tool_call(&mut self, tool_call: CallToolRequestParam) -> bool;
    pub fn reset(&mut self);
}
```

## ToolInspector Trait

```rust
#[async_trait]
impl ToolInspector for RepetitionInspector {
    fn name(&self) -> &'static str;
    async fn inspect(
        &self,
        tool_requests: &[ToolRequest],
        messages: &[Message],
    ) -> Result<Vec<InspectionResult>>;
}
```


## InspectionResult

```rust
pub struct InspectionResult {
    pub tool_request_id: String,
    pub action: InspectionAction,
    pub reason: String,
    pub confidence: f32,
    pub inspector_name: String,
    pub finding_id: Option<String>,
}

pub enum InspectionAction {
    Allow,
    Deny,
    Warn,
}
```

## 使用场景

- 防止工具调用死循环
- 检测重复操作
- 限制调用频率

## 源码位置

`crates/aster/src/tool_monitor.rs`
