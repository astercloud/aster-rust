# 会话管理系统

## 概述

会话系统管理用户与 Agent 的交互历史、状态持久化和恢复。

**核心路径**: `crates/aster/src/session/`

## 当前事实源

- `SessionManager` 负责会话与对话消息的持久化。
- `Thread / Turn / Item runtime` 统一收口到 `ThreadRuntimeStore`。
- `aster::session` 只保留 `initialize_*` / `require_*` 这组 current API；旧的 `shared_thread_runtime_store()` / `shared_session_runtime_queue_service()` 隐式 fallback 已从框架面删除。
- 初始化 shared thread runtime store 时会同步固定 shared runtime queue service，避免 `require_shared_session_runtime_queue_service()` 在热路径隐式补初始化。
- `aster-cli` 会在 `main()` 启动阶段先初始化 shared SQLite runtime store，CLI 构造 Agent 时优先消费 `Agent::new_with_required_shared_thread_runtime_store()`。
- `aster-server` 现在会在 `AppState::new()` 中先初始化 shared SQLite runtime store，再创建 `AgentManager`。
- `AgentManager::instance()` 与 scheduler 执行路径现在都依赖已初始化的 shared runtime store；它们不再承担隐式 fallback 初始化职责。
- 宿主默认 Agent 构造应优先使用 `Agent::new_with_required_shared_thread_runtime_store()`，不要在 CLI / scheduler / server 里重复手搓 `Agent::new() + with_thread_runtime_store(...)`。
- 运行时热路径优先使用 `require_shared_thread_runtime_store()` / `require_shared_session_runtime_queue_service()`，避免隐式 fallback。
- `SessionManager::delete_session()` 会同步清理 runtime 残留；若宿主未初始化 shared runtime store，则只记录告警，不再隐式回退到默认路径。

如果某个入口需要测试隔离，应显式注入 `InMemoryThreadRuntimeStore`，而不是让生产入口继续回退到内存态。

## 模块结构

| 模块 | 说明 |
|------|------|
| `session_manager.rs` | 会话管理器核心 |
| `archive.rs` | 会话归档 |
| `cleanup.rs` | 过期数据清理 |
| `export.rs` | 会话导出 |
| `fork.rs` | 会话分支/合并 |
| `resume.rs` | 会话恢复 |
| `statistics.rs` | 统计信息 |
| `diagnostics.rs` | 诊断工具 |
| `extension_data.rs` | 扩展数据存储 |

## Session 结构

```rust
pub struct Session {
    pub id: String,
    pub name: String,
    pub working_dir: PathBuf,
    pub session_type: SessionType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub extension_data: ExtensionData,
}

pub enum SessionType {
    User,       // 用户会话
    SubAgent,   // 子 Agent 会话
    Scheduled,  // 定时任务会话
}
```

## SessionManager API

```rust
impl SessionManager {
    // 创建会话
    pub async fn create_session(
        working_dir: PathBuf,
        name: String,
        session_type: SessionType,
    ) -> Result<Session>;
    
    // 获取会话
    pub async fn get_session(id: &str, include_messages: bool) 
        -> Result<Session>;
    
    // 列出会话
    pub async fn list_sessions() -> Result<Vec<Session>>;
    
    // 更新会话
    pub fn update_session(id: &str) -> SessionUpdateBuilder;
    
    // 添加消息
    pub async fn add_message(id: &str, message: &Message) -> Result<()>;
    
    // 删除会话
    pub async fn delete_session(id: &str) -> Result<()>;
}
```

删除语义说明：

- 删除 `Session` / `messages`
- best-effort 删除关联 `thread / turn / item runtime`
- 调用方只负责业务层清理，例如取消任务、移除 UI 投影；不要重复实现 runtime 删除

## 会话归档

```rust
// 归档单个会话
pub async fn archive_session(session_id: &str) -> Result<()>;

// 批量归档
pub async fn bulk_archive_sessions(
    session_ids: &[String]
) -> BulkArchiveResult;

// 恢复归档
pub async fn restore_archived_session(session_id: &str) -> Result<()>;

// 列出归档
pub async fn list_archived_sessions() -> Result<Vec<ArchivedSession>>;
```

## 会话导出

```rust
pub enum ExportFormat {
    Markdown,
    Json,
    Yaml,
}

pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_tool_calls: bool,
}

// 导出会话
pub async fn export_session(
    session_id: &str,
    options: ExportOptions,
) -> Result<String>;

// 导出到文件
pub async fn export_session_to_file(
    session_id: &str,
    path: &Path,
    options: ExportOptions,
) -> Result<()>;
```

## 会话分支

```rust
pub struct ForkOptions {
    pub from_message_index: Option<usize>,
    pub new_name: Option<String>,
    pub metadata_strategy: MetadataStrategy,
}

// 分支会话
pub async fn fork_session(
    session_id: &str,
    options: ForkOptions,
) -> Result<Session>;

// 合并会话
pub async fn merge_sessions(
    source_id: &str,
    target_id: &str,
    options: MergeOptions,
) -> Result<()>;

// 获取分支树
pub async fn get_session_branch_tree(
    session_id: &str
) -> Result<SessionBranchTree>;
```

## 会话恢复

```rust
// 构建恢复消息
pub async fn build_resume_message(session_id: &str) -> Result<Message>;

// 保存摘要
pub async fn save_summary(
    session_id: &str,
    summary: &SummaryCacheData,
) -> Result<()>;

// 加载摘要
pub async fn load_summary(session_id: &str) -> Result<SummaryCacheData>;
```

## 统计信息

```rust
pub struct SessionStatistics {
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub tool_calls: usize,
    pub total_tokens: u64,
    pub duration: Duration,
}

pub async fn calculate_statistics(
    session_id: &str
) -> Result<SessionStatistics>;

pub async fn generate_report(
    session_id: &str
) -> Result<SessionSummary>;
```

## 清理机制

```rust
pub const DEFAULT_CLEANUP_PERIOD_DAYS: u64 = 30;

pub struct CleanupStats {
    pub sessions_cleaned: usize,
    pub bytes_freed: u64,
}

// 清理过期数据
pub async fn cleanup_expired_data() -> Result<CleanupStats>;

// 强制清理
pub async fn force_cleanup(before_date: DateTime<Utc>) -> Result<CleanupStats>;

// 定时清理
pub fn schedule_cleanup(interval: Duration);
```

## 扩展数据

```rust
pub struct ExtensionData {
    pub extension_states: HashMap<String, Value>,
}

pub struct EnabledExtensionsState {
    pub extensions: Vec<ExtensionConfig>,
}

// TODO 使用结构化 `todo.v1`，legacy `todo.v0` 仅保留在兼容读取边界
```
