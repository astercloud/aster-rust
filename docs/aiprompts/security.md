# Security 安全系统

提示词注入检测和安全扫描。

## 模块结构

```
security/
├── classification_client.rs  # ML 分类客户端
├── patterns.rs               # 模式匹配
├── scanner.rs                # 扫描器
└── security_inspector.rs     # 安全检查器
```

## SecurityManager

```rust
pub struct SecurityManager {
    scanner: OnceLock<PromptInjectionScanner>,
}

impl SecurityManager {
    pub fn new() -> Self;
    pub fn is_prompt_injection_detection_enabled() -> bool;
    pub async fn analyze_tool_requests(
        &self,
        tool_requests: &[ToolRequest],
        messages: &[Message],
    ) -> Result<Vec<SecurityResult>>;
}
```


## SecurityResult

```rust
pub struct SecurityResult {
    pub is_malicious: bool,
    pub confidence: f32,
    pub explanation: String,
    pub should_ask_user: bool,
    pub finding_id: String,
    pub tool_request_id: String,
}
```

## 检测模式

1. **模式匹配** - 基于规则的检测
2. **ML 检测** - 机器学习分类器（可选）

## 配置

```
SECURITY_PROMPT_ENABLED=true
SECURITY_PROMPT_CLASSIFIER_ENABLED=true
```

## 使用场景

- 检测恶意提示词注入
- 工具调用安全审计
- 阻止危险操作

## 源码位置

`crates/aster/src/security/`
