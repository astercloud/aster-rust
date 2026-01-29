# 速率限制系统

处理 API 速率限制和自动重试。

## 模块结构

```
ratelimit/
├── budget.rs   # 预算管理
├── limiter.rs  # 速率限制器
└── retry.rs    # 重试策略
```

## 核心组件

### RateLimiter
```rust
pub struct RateLimiter {
    config: RateLimitConfig,
    state: RateLimitState,
}

impl RateLimiter {
    pub fn check(&self) -> bool;
    pub fn record_request(&mut self);
    pub fn wait_if_needed(&self) -> Duration;
}
```

### RateLimitConfig
```rust
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub burst_limit: u32,
}
```


## 预算管理

### BudgetManager
```rust
pub struct BudgetManager {
    pub daily_limit: u64,
    pub monthly_limit: u64,
}

pub struct CostTracker {
    pub total_cost: f64,
    pub daily_cost: f64,
}
```

## 重试策略

```rust
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

pub fn retry_with_backoff<F, T>(f: F, policy: RetryPolicy) -> Result<T>;
pub fn is_retryable_error(error: &Error) -> bool;
pub fn parse_retry_after(headers: &Headers) -> Option<Duration>;
```

## 使用场景

- 防止 API 超限
- 自动重试失败请求
- 成本控制

## 源码位置

`crates/aster/src/ratelimit/`
