# 基准测试系统

## 概述

基准测试系统用于评估 Agent 在各种任务上的表现。

**核心路径**: `crates/aster-bench/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `bench_config.rs` | 配置定义 |
| `bench_session.rs` | 测试会话 |
| `bench_work_dir.rs` | 工作目录管理 |
| `error_capture.rs` | 错误捕获 |
| `eval_suites/` | 评估套件 |
| `reporting/` | 报告生成 |
| `runners/` | 运行器 |
| `utilities/` | 工具函数 |

## CLI 命令

### 初始化配置

```bash
aster bench init-config --name my-bench.yaml
```

### 运行基准测试

```bash
aster bench run --config my-bench.yaml
```

### 列出选择器

```bash
aster bench selectors
aster bench selectors --config my-bench.yaml
```

### 评估模型

```bash
aster bench eval-model --config model-config.json
```

### 执行单个评估

```bash
aster bench exec-eval --config eval-config.json
```

### 生成排行榜

```bash
aster bench generate-leaderboard --benchmark-dir ./results
```

## 配置结构

```yaml
# my-bench.yaml
name: "My Benchmark"
description: "Benchmark description"

# 模型配置
models:
  - provider: anthropic
    model: claude-3-5-sonnet
  - provider: openai
    model: gpt-4o

# 评估套件
eval_suites:
  - name: coding
    tasks:
      - file_operations
      - code_generation
      - bug_fixing
  - name: reasoning
    tasks:
      - math_problems
      - logic_puzzles

# 运行配置
run_config:
  max_turns: 50
  timeout_seconds: 300
  parallel_runs: 4
```

## 运行器

### BenchRunner

主运行器，协调整个基准测试：

```rust
pub struct BenchRunner {
    config: BenchRunConfig,
}

impl BenchRunner {
    pub async fn run(&self) -> Result<BenchResults>;
}
```

### ModelRunner

单个模型的运行器：

```rust
pub struct ModelRunner {
    provider: Arc<dyn Provider>,
    model_config: ModelConfig,
}
```

### EvalRunner

单个评估的运行器：

```rust
pub struct EvalRunner {
    task: EvalTask,
    session: BenchSession,
}
```

### MetricAggregator

指标聚合器：

```rust
pub struct MetricAggregator {
    results: Vec<EvalResult>,
}

impl MetricAggregator {
    pub fn aggregate(&self) -> AggregatedMetrics;
    pub fn to_csv(&self) -> String;
}
```

## 评估套件

预定义的评估任务集合：

- **coding** - 编码任务
- **reasoning** - 推理任务
- **tool_use** - 工具使用
- **multi_step** - 多步骤任务

## 报告格式

```json
{
  "benchmark_name": "My Benchmark",
  "timestamp": "2025-01-29T10:00:00Z",
  "models": [
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet",
      "results": {
        "coding": {
          "score": 0.85,
          "tasks_completed": 17,
          "tasks_total": 20
        }
      }
    }
  ],
  "summary": {
    "best_model": "claude-3-5-sonnet",
    "average_score": 0.82
  }
}
```

## 工作目录

每个测试在隔离的工作目录中运行：

```rust
pub struct BenchWorkDir {
    path: PathBuf,
    cleanup_on_drop: bool,
}

impl BenchWorkDir {
    pub fn new() -> Result<Self>;
    pub fn path(&self) -> &Path;
}
```

## 错误捕获

```rust
pub struct ErrorCapture {
    errors: Vec<CapturedError>,
}

pub struct CapturedError {
    pub task: String,
    pub error: String,
    pub timestamp: DateTime<Utc>,
}
```
