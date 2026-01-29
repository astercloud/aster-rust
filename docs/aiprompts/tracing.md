# Tracing 追踪系统

分布式追踪和可观测性。

## 模块结构

```
tracing/
├── langfuse_layer.rs     # Langfuse 集成
├── observation_layer.rs  # 观测层
├── otlp_layer.rs         # OTLP 导出
└── rate_limiter.rs       # 速率限制
```

## Langfuse 集成

```rust
pub fn create_langfuse_observer() -> LangfuseBatchManager;
```

用于 LLM 调用追踪和分析。

## OTLP 导出

```rust
pub struct OtlpConfig {
    pub endpoint: String,
    pub service_name: String,
}

pub fn init_otlp_tracing(config: OtlpConfig);
pub fn init_otlp_metrics(config: OtlpConfig);
pub fn shutdown_otlp();
```


## 观测层

```rust
pub struct ObservationLayer;
pub struct SpanTracker;
pub struct BatchManager;

pub fn flatten_metadata(meta: &Metadata) -> HashMap<String, Value>;
pub fn map_level(level: Level) -> String;
```

## 速率限制

```rust
pub struct RateLimitedTelemetrySender;

pub enum TelemetryEvent {
    Span(SpanData),
    Metric(MetricData),
}
```

防止遥测数据过载。

## 使用场景

- LLM 调用追踪
- 性能监控
- 错误诊断

## 源码位置

`crates/aster/src/tracing/`
