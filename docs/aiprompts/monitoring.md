# 监控系统

## 概述

监控系统跟踪 Agent 执行指标，包括 Token 使用、API 调用、工具调用、成本和错误。

**核心路径**: `crates/aster/src/agents/monitor/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `metrics.rs` | 指标收集 |
| `alerts.rs` | 告警管理 |
| `analyzer.rs` | 性能分析 |

## FullAgentMetrics

```rust
pub struct FullAgentMetrics {
    pub agent_id: String,
    pub agent_type: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub status: AgentExecutionStatus,
    pub tokens_used: TokenUsage,
    pub api_calls: usize,
    pub api_calls_successful: usize,
    pub tool_calls: Vec<ToolCallMetric>,
    pub cost: f64,
    pub errors: Vec<ErrorRecord>,
    pub performance: PerformanceMetrics,
    pub timeout: Option<Duration>,
}
```

## TokenUsage

```rust
pub struct TokenUsage {
    pub input: usize,
    pub output: usize,
    pub total: usize,
}
```

## ToolCallMetric

```rust
pub struct ToolCallMetric {
    pub id: String,
    pub tool_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub success: bool,
    pub error: Option<String>,
    pub input_size: Option<usize>,
    pub output_size: Option<usize>,
}
```

## PerformanceMetrics

```rust
pub struct PerformanceMetrics {
    pub avg_api_latency: Option<Duration>,
    pub avg_tool_duration: Option<Duration>,
    pub tokens_per_second: Option<f64>,
    pub api_calls_per_minute: Option<f64>,
}
```

## AgentExecutionStatus

```rust
pub enum AgentExecutionStatus {
    Running,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}
```

## AgentMonitor

```rust
pub struct AgentMonitor {
    config: MonitorConfig,
    metrics: HashMap<String, FullAgentMetrics>,
    active_tool_calls: HashMap<String, (String, ToolCallMetric)>,
    metrics_dir: PathBuf,
}

impl AgentMonitor {
    // 开始跟踪
    pub fn start_tracking(&mut self, agent_id: &str, agent_type: &str, desc: Option<&str>);
    
    // 记录 Token
    pub fn record_tokens(&mut self, agent_id: &str, input: usize, output: usize);
    
    // 记录 API 调用
    pub fn record_api_call(&mut self, agent_id: &str, success: bool, latency: Option<Duration>);
    
    // 记录成本
    pub fn record_cost(&mut self, agent_id: &str, cost: f64);
    
    // 记录错误
    pub fn record_error(&mut self, agent_id: &str, error: &str, phase: Option<&str>);
    
    // 开始工具调用
    pub fn start_tool_call(&mut self, agent_id: &str, tool_name: &str, input_size: Option<usize>) -> String;
    
    // 结束工具调用
    pub fn end_tool_call(&mut self, agent_id: &str, tool_call_id: &str, success: bool, error: Option<&str>, output_size: Option<usize>);
    
    // 停止跟踪
    pub fn stop_tracking(&mut self, agent_id: &str, status: AgentExecutionStatus);
    
    // 获取聚合统计
    pub fn get_aggregated_stats(&self) -> AggregatedStats;
    
    // 持久化指标
    pub fn persist_metrics(&self, agent_id: &str) -> std::io::Result<()>;
}
```

## AggregatedStats

```rust
pub struct AggregatedStats {
    pub total_agents: usize,
    pub completed_agents: usize,
    pub failed_agents: usize,
    pub running_agents: usize,
    pub total_tokens: usize,
    pub total_api_calls: usize,
    pub total_tool_calls: usize,
    pub total_cost: f64,
    pub total_errors: usize,
    pub avg_duration: Option<Duration>,
    pub avg_tokens_per_agent: f64,
    pub overall_error_rate: f32,
}
```

## MonitorConfig

```rust
pub struct MonitorConfig {
    pub track_tool_calls: bool,      // 跟踪工具调用
    pub track_api_latencies: bool,   // 跟踪 API 延迟
    pub auto_persist: bool,          // 自动持久化
    pub max_metrics_in_memory: usize,// 内存最大指标数
    pub metrics_dir: Option<PathBuf>,// 指标目录
}
```

## 使用示例

```rust
let mut monitor = AgentMonitor::new(None);

// 开始跟踪
monitor.start_tracking("agent-1", "explore", Some("代码分析"));

// 记录指标
monitor.record_tokens("agent-1", 1000, 500);
monitor.record_api_call("agent-1", true, Some(Duration::from_millis(200)));
monitor.record_cost("agent-1", 0.05);

// 跟踪工具调用
let tool_id = monitor.start_tool_call("agent-1", "bash", Some(100));
// ... 执行工具 ...
monitor.end_tool_call("agent-1", &tool_id, true, None, Some(500));

// 停止跟踪
monitor.stop_tracking("agent-1", AgentExecutionStatus::Completed);

// 获取统计
let stats = monitor.get_aggregated_stats();
println!("Total tokens: {}", stats.total_tokens);
```

## 指标持久化

指标保存为 JSON 文件：

```
.aster/metrics/
├── agent-1.json
├── agent-2.json
└── agent-3.json
```
