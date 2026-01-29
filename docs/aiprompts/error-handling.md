# 错误处理系统

## 概述

统一错误处理系统提供错误记录、超时处理和重试机制。

**核心路径**: `crates/aster/src/agents/error_handling/`

## 错误类型

```rust
pub enum AgentErrorKind {
    // Provider 错误
    ProviderError,
    AuthenticationError,
    RateLimitError,
    
    // 工具错误
    ToolExecutionError,
    ToolNotFound,
    ToolTimeout,
    
    // 上下文错误
    ContextError,
    ContextLimitExceeded,
    
    // 配置错误
    ConfigurationError,
    InvalidInput,
    
    // 系统错误
    NetworkError,
    IoError,
    InternalError,
    
    // 执行错误
    Timeout,
    Cancelled,
    MaxRetriesExceeded,
}

pub struct AgentError {
    pub kind: AgentErrorKind,
    pub message: String,
    pub context: ErrorContext,
    pub source: Option<Box<dyn std::error::Error>>,
    pub recoverable: bool,
}
```

## 错误上下文

```rust
pub struct ErrorContext {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub phase: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub additional: HashMap<String, Value>,
}
```

## 错误记录

```rust
pub struct UnifiedErrorRecord {
    pub id: String,
    pub error: AgentError,
    pub stack_trace: Option<String>,
    pub retry_count: usize,
    pub resolved: bool,
    pub resolution: Option<String>,
}
```

## ErrorHandler

```rust
pub struct ErrorHandler {
    records: Vec<UnifiedErrorRecord>,
    max_records: usize,
}

impl ErrorHandler {
    pub fn new(max_records: usize) -> Self;
    
    // 记录错误
    pub fn record(&mut self, error: AgentError) -> String;
    
    // 标记已解决
    pub fn resolve(&mut self, id: &str, resolution: &str);
    
    // 获取未解决错误
    pub fn get_unresolved(&self) -> Vec<&UnifiedErrorRecord>;
    
    // 按类型过滤
    pub fn filter_by_kind(&self, kind: AgentErrorKind) -> Vec<&UnifiedErrorRecord>;
    
    // 清理旧记录
    pub fn cleanup(&mut self, before: DateTime<Utc>);
}
```

## 重试处理

```rust
pub struct UnifiedRetryConfig {
    pub max_retries: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<AgentErrorKind>,
}

pub enum RetryStrategy {
    Immediate,
    FixedDelay(Duration),
    ExponentialBackoff {
        initial: Duration,
        max: Duration,
        multiplier: f64,
    },
    Custom(Box<dyn Fn(usize) -> Duration>),
}

pub struct RetryHandler {
    config: UnifiedRetryConfig,
    strategy: RetryStrategy,
}

impl RetryHandler {
    // 判断是否应重试
    pub fn should_retry(&self, error: &AgentError, attempt: usize) -> bool;
    
    // 获取重试延迟
    pub fn get_delay(&self, attempt: usize) -> Duration;
    
    // 执行带重试
    pub async fn execute_with_retry<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: Fn() -> Future<Output = Result<T, E>>;
}

pub enum RetryResult {
    Success,
    Retried,
    Failed,
    MaxAttemptsReached,
}
```

## 超时处理

```rust
pub struct TimeoutConfig {
    pub default_timeout: Duration,
    pub tool_timeout: Duration,
    pub api_timeout: Duration,
    pub total_timeout: Option<Duration>,
}

pub enum TimeoutStatus {
    Active,
    Warning,
    Expired,
}

pub struct TimeoutHandler {
    config: TimeoutConfig,
    start_time: Instant,
}

impl TimeoutHandler {
    pub fn new(config: TimeoutConfig) -> Self;
    
    // 检查超时状态
    pub fn check_status(&self) -> TimeoutStatus;
    
    // 剩余时间
    pub fn remaining(&self) -> Option<Duration>;
    
    // 是否已超时
    pub fn is_expired(&self) -> bool;
    
    // 带超时执行
    pub async fn with_timeout<F, T>(&self, f: F) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>;
}

pub struct TimeoutEvent {
    pub event_type: String,
    pub duration: Duration,
    pub limit: Duration,
    pub timestamp: DateTime<Utc>,
}
```

## 使用示例

```rust
// 错误处理
let mut handler = ErrorHandler::new(1000);

let error = AgentError {
    kind: AgentErrorKind::ToolExecutionError,
    message: "Command failed".to_string(),
    context: ErrorContext::default(),
    source: None,
    recoverable: true,
};

let id = handler.record(error);

// 重试处理
let retry_config = UnifiedRetryConfig {
    max_retries: 3,
    initial_delay: Duration::from_secs(1),
    max_delay: Duration::from_secs(30),
    backoff_multiplier: 2.0,
    retryable_errors: vec![
        AgentErrorKind::RateLimitError,
        AgentErrorKind::NetworkError,
    ],
};

let retry_handler = RetryHandler::new(retry_config);

// 超时处理
let timeout_config = TimeoutConfig {
    default_timeout: Duration::from_secs(60),
    tool_timeout: Duration::from_secs(300),
    api_timeout: Duration::from_secs(30),
    total_timeout: Some(Duration::from_secs(3600)),
};

let timeout_handler = TimeoutHandler::new(timeout_config);
```
