# 会话管理系统

## 概述

会话系统管理用户与 Agent 的交互历史、状态持久化和恢复。

**核心路径**: `crates/aster/src/session/`

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
    pub enabled_extensions: EnabledExtensionsState,
    pub todo_state: Option<TodoState>,
    pub custom: HashMap<String, Value>,
}

pub struct EnabledExtensionsState {
    pub extensions: Vec<ExtensionState>,
}

pub struct ExtensionState {
    pub name: String,
    pub config: ExtensionConfig,
    pub enabled: bool,
}
```
