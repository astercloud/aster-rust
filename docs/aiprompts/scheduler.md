# Scheduler 调度系统

基于 Cron 的任务调度系统。

## 核心类型

### Scheduler
```rust
pub struct Scheduler {
    tokio_scheduler: TokioJobScheduler,
    jobs: Arc<Mutex<JobsMap>>,
    storage_path: PathBuf,
    running_tasks: Arc<Mutex<RunningTasksMap>>,
}
```

### ScheduledJob
```rust
pub struct ScheduledJob {
    pub id: String,
    pub source: String,           // Recipe 路径
    pub cron: String,             // Cron 表达式
    pub last_run: Option<DateTime<Utc>>,
    pub currently_running: bool,
    pub paused: bool,
    pub current_session_id: Option<String>,
}
```


## 核心 API

```rust
impl Scheduler {
    pub async fn new(storage_path: PathBuf) -> Result<Arc<Self>>;
    pub async fn add_scheduled_job(job: ScheduledJob, make_copy: bool);
    pub async fn schedule_recipe(path: PathBuf, cron: Option<String>);
    pub async fn list_scheduled_jobs() -> Vec<ScheduledJob>;
    pub async fn remove_scheduled_job(id: &str, remove_recipe: bool);
    pub async fn run_now(sched_id: &str) -> Result<String>;
    pub async fn pause_schedule(sched_id: &str);
    pub async fn unpause_schedule(sched_id: &str);
    pub async fn update_schedule(sched_id: &str, new_cron: String);
    pub async fn kill_running_job(sched_id: &str);
}
```

## Cron 格式

支持 5 字段和 6 字段格式：
- 5 字段: `分 时 日 月 周`
- 6 字段: `秒 分 时 日 月 周`

## 持久化

调度任务存储在 `schedules.json`。

## 源码位置

`crates/aster/src/scheduler.rs`
