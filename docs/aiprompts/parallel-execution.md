# 并行执行系统

## 概述

并行执行系统支持多个 Agent 任务的并发执行，包括依赖管理和重试逻辑。

**核心路径**: `crates/aster/src/agents/parallel/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `executor.rs` | 并行执行器 |
| `pool.rs` | Agent 资源池 |

## 配置

```rust
pub struct ParallelAgentConfig {
    pub max_concurrency: usize,      // 最大并发数 (默认 4)
    pub timeout: Duration,           // 默认超时 (默认 5 分钟)
    pub retry_on_failure: bool,      // 失败重试 (默认 true)
    pub stop_on_first_error: bool,   // 首错停止 (默认 false)
    pub max_retries: usize,          // 最大重试次数 (默认 3)
    pub retry_delay: Duration,       // 重试延迟 (默认 1 秒)
}
```

## AgentTask

```rust
pub struct AgentTask {
    pub id: String,
    pub task_type: String,
    pub prompt: String,
    pub description: Option<String>,
    pub options: Option<HashMap<String, Value>>,
    pub priority: Option<u8>,        // 优先级 (越高越先执行)
    pub dependencies: Option<Vec<String>>,
    pub timeout: Option<Duration>,
}
```

## 任务状态

```rust
pub enum TaskStatus {
    Pending,                // 等待执行
    WaitingForDependencies, // 等待依赖
    Running,                // 执行中
    Completed,              // 完成
    Failed,                 // 失败
    Cancelled,              // 取消
    Skipped,                // 跳过 (依赖失败)
}
```

## 执行结果

```rust
pub struct AgentResult {
    pub task_id: String,
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub duration: Duration,
    pub retries: usize,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

pub struct ParallelExecutionResult {
    pub success: bool,
    pub results: Vec<AgentResult>,
    pub total_duration: Duration,
    pub successful_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub merged_result: Option<MergedResult>,
}
```

## 依赖图

```rust
pub struct DependencyGraph {
    dependencies: HashMap<String, HashSet<String>>,
    dependents: HashMap<String, HashSet<String>>,
    task_ids: HashSet<String>,
}

impl DependencyGraph {
    pub fn add_task(&mut self, task_id: &str);
    pub fn add_dependency(&mut self, task_id: &str, dep_id: &str);
    pub fn get_dependencies(&self, task_id: &str) -> HashSet<String>;
    pub fn get_ready_tasks(&self, completed: &HashSet<String>) -> Vec<String>;
    pub fn has_unmet_dependencies(&self, task_id: &str, completed: &HashSet<String>) -> bool;
}
```

## ParallelAgentExecutor

```rust
pub struct ParallelAgentExecutor {
    config: ParallelAgentConfig,
    tasks: Arc<Mutex<HashMap<String, TaskExecutionInfo>>>,
    running: Arc<Mutex<bool>>,
    cancelled: Arc<Mutex<bool>>,
}

impl ParallelAgentExecutor {
    // 无依赖并行执行
    pub async fn execute(&mut self, tasks: Vec<AgentTask>) 
        -> ExecutorResult<ParallelExecutionResult>;
    
    // 带依赖执行
    pub async fn execute_with_dependencies(&mut self, tasks: Vec<AgentTask>)
        -> ExecutorResult<ParallelExecutionResult>;
}
```

## 使用示例

```rust
let config = ParallelAgentConfig {
    max_concurrency: 4,
    stop_on_first_error: false,
    ..Default::default()
};

let mut executor = ParallelAgentExecutor::with_config(config);

let tasks = vec![
    AgentTask::new("task-1", "explore", "分析代码结构"),
    AgentTask::new("task-2", "plan", "制定实现计划")
        .with_dependencies(vec!["task-1".to_string()]),
    AgentTask::new("task-3", "execute", "执行实现")
        .with_dependencies(vec!["task-2".to_string()]),
];

let result = executor.execute_with_dependencies(tasks).await?;
```

## 执行流程

```
任务列表
    │
    ▼
┌─────────────────────────────┐
│  验证依赖 (检测循环)         │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  构建依赖图                  │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  按优先级排序                │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  并行执行就绪任务            │
│  (受 max_concurrency 限制)  │
└───────────┬─────────────────┘
            │
            ▼
┌─────────────────────────────┐
│  等待完成，解锁依赖任务      │
└───────────┬─────────────────┘
            │
            ▼
合并结果
```

## 错误类型

```rust
pub enum ExecutorError {
    TaskNotFound(String),
    TaskTimeout(String),
    TaskFailed { task_id: String, error: String },
    CircularDependency(Vec<String>),
    InvalidDependency { task_id: String, dependency: String },
    Cancelled,
    RetriesExhausted(String),
    DependencyFailed { task_id: String, dependency: String },
}
```
