use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{OnceCell, RwLock};
use utoipa::ToSchema;

use crate::config::paths::Paths;
use crate::session::session_manager::SESSIONS_FOLDER;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteRow};
use sqlx::{Pool, Row, Sqlite};

const RUNTIME_DB_NAME: &str = "runtime.db";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnContextOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = String)]
    pub cwd: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collaboration_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub output_schema: Option<Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[schema(value_type = Object)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStatus {
    #[default]
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TurnStatus {
    Queued,
    #[default]
    Running,
    Completed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    #[default]
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemRuntimePayload {
    UserMessage {
        content: String,
    },
    AgentMessage {
        text: String,
    },
    Reasoning {
        text: String,
    },
    ToolCall {
        tool_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(value_type = Object)]
        arguments: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(value_type = Object)]
        output: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        success: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(value_type = Object)]
        metadata: Option<Value>,
    },
    ApprovalRequest {
        request_id: String,
        action_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(value_type = Object)]
        arguments: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(value_type = Object)]
        response: Option<Value>,
    },
    RequestUserInput {
        request_id: String,
        action_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(value_type = Object)]
        requested_schema: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schema(value_type = Object)]
        response: Option<Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemRuntime {
    pub id: String,
    pub thread_id: String,
    pub turn_id: String,
    pub sequence: i64,
    pub status: ItemStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    #[serde(flatten)]
    pub payload: ItemRuntimePayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadRuntime {
    pub id: String,
    pub session_id: String,
    #[schema(value_type = String)]
    pub working_dir: PathBuf,
    pub status: ThreadStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[schema(value_type = Object)]
    pub metadata: HashMap<String, Value>,
}

impl ThreadRuntime {
    pub fn new(id: impl Into<String>, session_id: impl Into<String>, working_dir: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            session_id: session_id.into(),
            working_dir,
            status: ThreadStatus::Active,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TurnRuntime {
    pub id: String,
    pub session_id: String,
    pub thread_id: String,
    pub status: TurnStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_override: Option<TurnContextOverride>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl TurnRuntime {
    pub fn new(
        id: impl Into<String>,
        session_id: impl Into<String>,
        thread_id: impl Into<String>,
        input_text: Option<String>,
        context_override: Option<TurnContextOverride>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            session_id: session_id.into(),
            thread_id: thread_id.into(),
            status: TurnStatus::Running,
            input_text,
            error_message: None,
            context_override,
            created_at: now,
            started_at: Some(now),
            completed_at: None,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadRuntimeSnapshot {
    pub thread: ThreadRuntime,
    pub turns: Vec<TurnRuntime>,
    pub items: Vec<ItemRuntime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionRuntimeSnapshot {
    pub session_id: String,
    pub threads: Vec<ThreadRuntimeSnapshot>,
}

#[async_trait]
pub trait ThreadRuntimeStore: Send + Sync {
    async fn upsert_thread(&self, thread: ThreadRuntime) -> Result<ThreadRuntime>;
    async fn list_threads(&self, session_id: &str) -> Result<Vec<ThreadRuntime>>;
    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRuntime>>;
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    async fn create_turn(&self, turn: TurnRuntime) -> Result<TurnRuntime>;
    async fn update_turn(&self, turn: TurnRuntime) -> Result<TurnRuntime>;
    async fn get_turn(&self, turn_id: &str) -> Result<Option<TurnRuntime>>;
    async fn list_turns(&self, thread_id: &str) -> Result<Vec<TurnRuntime>>;
    async fn create_item(&self, item: ItemRuntime) -> Result<ItemRuntime>;
    async fn update_item(&self, item: ItemRuntime) -> Result<ItemRuntime>;
    async fn get_item(&self, item_id: &str) -> Result<Option<ItemRuntime>>;
    async fn list_items(&self, thread_id: &str) -> Result<Vec<ItemRuntime>>;
}

fn default_runtime_db_path() -> PathBuf {
    Paths::data_dir()
        .join(SESSIONS_FOLDER)
        .join(RUNTIME_DB_NAME)
}

pub fn shared_thread_runtime_store() -> Arc<dyn ThreadRuntimeStore> {
    static SHARED: OnceLock<Arc<SqliteThreadRuntimeStore>> = OnceLock::new();

    SHARED
        .get_or_init(|| Arc::new(SqliteThreadRuntimeStore::new(default_runtime_db_path())))
        .clone()
}

pub async fn delete_shared_thread_runtime_session(session_id: &str) -> Result<()> {
    shared_thread_runtime_store()
        .delete_session(session_id)
        .await
}

pub struct SqliteThreadRuntimeStore {
    db_path: PathBuf,
    pool: OnceCell<Pool<Sqlite>>,
}

impl SqliteThreadRuntimeStore {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            pool: OnceCell::const_new(),
        }
    }

    async fn pool(&self) -> Result<&Pool<Sqlite>> {
        let db_path = self.db_path.clone();
        self.pool
            .get_or_try_init(|| async move { Self::open_pool(&db_path).await })
            .await
    }

    async fn open_pool(db_path: &std::path::Path) -> Result<Pool<Sqlite>> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .busy_timeout(Duration::from_secs(5))
            .journal_mode(SqliteJournalMode::Wal);

        let pool = sqlx::SqlitePool::connect_with(options).await?;
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;
        Self::ensure_schema(&pool).await?;
        Ok(pool)
    }

    async fn ensure_schema(pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS thread_runtimes (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                working_dir TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL,
                updated_at TIMESTAMP NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}'
            )
        "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS turn_runtimes (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                thread_id TEXT NOT NULL REFERENCES thread_runtimes(id) ON DELETE CASCADE,
                status TEXT NOT NULL,
                input_text TEXT,
                error_message TEXT,
                context_override_json TEXT,
                created_at TIMESTAMP NOT NULL,
                started_at TIMESTAMP,
                completed_at TIMESTAMP,
                updated_at TIMESTAMP NOT NULL
            )
        "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS item_runtimes (
                id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL REFERENCES thread_runtimes(id) ON DELETE CASCADE,
                turn_id TEXT NOT NULL REFERENCES turn_runtimes(id) ON DELETE CASCADE,
                sequence INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at TIMESTAMP NOT NULL,
                completed_at TIMESTAMP,
                updated_at TIMESTAMP NOT NULL,
                payload_json TEXT NOT NULL
            )
        "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_thread_runtimes_session ON thread_runtimes(session_id, created_at ASC)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_turn_runtimes_thread ON turn_runtimes(thread_id, created_at ASC)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_turn_runtimes_session ON turn_runtimes(session_id, created_at ASC)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_item_runtimes_thread ON item_runtimes(thread_id, sequence ASC, started_at ASC)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_item_runtimes_turn ON item_runtimes(turn_id, sequence ASC)",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn write_thread(&self, mut thread: ThreadRuntime) -> Result<ThreadRuntime> {
        thread.updated_at = Utc::now();
        let pool = self.pool().await?;
        sqlx::query(
            r#"
            INSERT INTO thread_runtimes (
                id, session_id, working_dir, status, created_at, updated_at, metadata_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                session_id = excluded.session_id,
                working_dir = excluded.working_dir,
                status = excluded.status,
                updated_at = excluded.updated_at,
                metadata_json = excluded.metadata_json
        "#,
        )
        .bind(&thread.id)
        .bind(&thread.session_id)
        .bind(thread.working_dir.to_string_lossy().to_string())
        .bind(thread_status_as_str(thread.status))
        .bind(thread.created_at)
        .bind(thread.updated_at)
        .bind(serde_json::to_string(&thread.metadata)?)
        .execute(pool)
        .await?;
        Ok(thread)
    }

    async fn write_turn(&self, mut turn: TurnRuntime) -> Result<TurnRuntime> {
        turn.updated_at = Utc::now();
        let pool = self.pool().await?;
        sqlx::query(
            r#"
            INSERT INTO turn_runtimes (
                id, session_id, thread_id, status, input_text, error_message,
                context_override_json, created_at, started_at, completed_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                session_id = excluded.session_id,
                thread_id = excluded.thread_id,
                status = excluded.status,
                input_text = excluded.input_text,
                error_message = excluded.error_message,
                context_override_json = excluded.context_override_json,
                started_at = excluded.started_at,
                completed_at = excluded.completed_at,
                updated_at = excluded.updated_at
        "#,
        )
        .bind(&turn.id)
        .bind(&turn.session_id)
        .bind(&turn.thread_id)
        .bind(turn_status_as_str(turn.status))
        .bind(&turn.input_text)
        .bind(&turn.error_message)
        .bind(
            turn.context_override
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
        )
        .bind(turn.created_at)
        .bind(turn.started_at)
        .bind(turn.completed_at)
        .bind(turn.updated_at)
        .execute(pool)
        .await?;
        Ok(turn)
    }

    async fn write_item(&self, mut item: ItemRuntime) -> Result<ItemRuntime> {
        item.updated_at = Utc::now();
        let pool = self.pool().await?;
        sqlx::query(
            r#"
            INSERT INTO item_runtimes (
                id, thread_id, turn_id, sequence, status, started_at, completed_at, updated_at, payload_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                thread_id = excluded.thread_id,
                turn_id = excluded.turn_id,
                sequence = excluded.sequence,
                status = excluded.status,
                started_at = excluded.started_at,
                completed_at = excluded.completed_at,
                updated_at = excluded.updated_at,
                payload_json = excluded.payload_json
        "#,
        )
        .bind(&item.id)
        .bind(&item.thread_id)
        .bind(&item.turn_id)
        .bind(item.sequence)
        .bind(item_status_as_str(item.status))
        .bind(item.started_at)
        .bind(item.completed_at)
        .bind(item.updated_at)
        .bind(serde_json::to_string(&item.payload)?)
        .execute(pool)
        .await?;
        Ok(item)
    }

    fn decode_thread_row(row: SqliteRow) -> Result<ThreadRuntime> {
        let status_raw: String = row.try_get("status")?;
        let metadata_json: String = row.try_get("metadata_json")?;
        Ok(ThreadRuntime {
            id: row.try_get("id")?,
            session_id: row.try_get("session_id")?,
            working_dir: PathBuf::from(row.try_get::<String, _>("working_dir")?),
            status: parse_thread_status(&status_raw)?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            metadata: serde_json::from_str(&metadata_json)?,
        })
    }

    fn decode_turn_row(row: SqliteRow) -> Result<TurnRuntime> {
        let status_raw: String = row.try_get("status")?;
        let context_override_json: Option<String> = row.try_get("context_override_json")?;
        Ok(TurnRuntime {
            id: row.try_get("id")?,
            session_id: row.try_get("session_id")?,
            thread_id: row.try_get("thread_id")?,
            status: parse_turn_status(&status_raw)?,
            input_text: row.try_get("input_text")?,
            error_message: row.try_get("error_message")?,
            context_override: context_override_json
                .map(|json| serde_json::from_str(&json))
                .transpose()?,
            created_at: row.try_get("created_at")?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    fn decode_item_row(row: SqliteRow) -> Result<ItemRuntime> {
        let status_raw: String = row.try_get("status")?;
        let payload_json: String = row.try_get("payload_json")?;
        Ok(ItemRuntime {
            id: row.try_get("id")?,
            thread_id: row.try_get("thread_id")?,
            turn_id: row.try_get("turn_id")?,
            sequence: row.try_get("sequence")?,
            status: parse_item_status(&status_raw)?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            updated_at: row.try_get("updated_at")?,
            payload: serde_json::from_str(&payload_json)?,
        })
    }
}

fn thread_status_as_str(status: ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Active => "active",
        ThreadStatus::Archived => "archived",
    }
}

fn parse_thread_status(value: &str) -> Result<ThreadStatus> {
    match value {
        "active" => Ok(ThreadStatus::Active),
        "archived" => Ok(ThreadStatus::Archived),
        other => anyhow::bail!("Unknown thread status: {other}"),
    }
}

fn turn_status_as_str(status: TurnStatus) -> &'static str {
    match status {
        TurnStatus::Queued => "queued",
        TurnStatus::Running => "running",
        TurnStatus::Completed => "completed",
        TurnStatus::Failed => "failed",
        TurnStatus::Aborted => "aborted",
    }
}

fn parse_turn_status(value: &str) -> Result<TurnStatus> {
    match value {
        "queued" => Ok(TurnStatus::Queued),
        "running" => Ok(TurnStatus::Running),
        "completed" => Ok(TurnStatus::Completed),
        "failed" => Ok(TurnStatus::Failed),
        "aborted" => Ok(TurnStatus::Aborted),
        other => anyhow::bail!("Unknown turn status: {other}"),
    }
}

fn item_status_as_str(status: ItemStatus) -> &'static str {
    match status {
        ItemStatus::InProgress => "in_progress",
        ItemStatus::Completed => "completed",
        ItemStatus::Failed => "failed",
    }
}

fn parse_item_status(value: &str) -> Result<ItemStatus> {
    match value {
        "in_progress" => Ok(ItemStatus::InProgress),
        "completed" => Ok(ItemStatus::Completed),
        "failed" => Ok(ItemStatus::Failed),
        other => anyhow::bail!("Unknown item status: {other}"),
    }
}

#[async_trait]
impl ThreadRuntimeStore for SqliteThreadRuntimeStore {
    async fn upsert_thread(&self, thread: ThreadRuntime) -> Result<ThreadRuntime> {
        self.write_thread(thread).await
    }

    async fn list_threads(&self, session_id: &str) -> Result<Vec<ThreadRuntime>> {
        let pool = self.pool().await?;
        let rows = sqlx::query(
            r#"
            SELECT id, session_id, working_dir, status, created_at, updated_at, metadata_json
            FROM thread_runtimes
            WHERE session_id = ?
            ORDER BY created_at ASC, id ASC
        "#,
        )
        .bind(session_id)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::decode_thread_row).collect()
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRuntime>> {
        let pool = self.pool().await?;
        let row = sqlx::query(
            r#"
            SELECT id, session_id, working_dir, status, created_at, updated_at, metadata_json
            FROM thread_runtimes
            WHERE id = ?
        "#,
        )
        .bind(thread_id)
        .fetch_optional(pool)
        .await?;

        row.map(Self::decode_thread_row).transpose()
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let pool = self.pool().await?;
        sqlx::query("DELETE FROM thread_runtimes WHERE session_id = ?")
            .bind(session_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn create_turn(&self, turn: TurnRuntime) -> Result<TurnRuntime> {
        self.write_turn(turn).await
    }

    async fn update_turn(&self, turn: TurnRuntime) -> Result<TurnRuntime> {
        self.write_turn(turn).await
    }

    async fn get_turn(&self, turn_id: &str) -> Result<Option<TurnRuntime>> {
        let pool = self.pool().await?;
        let row = sqlx::query(
            r#"
            SELECT
                id, session_id, thread_id, status, input_text, error_message,
                context_override_json, created_at, started_at, completed_at, updated_at
            FROM turn_runtimes
            WHERE id = ?
        "#,
        )
        .bind(turn_id)
        .fetch_optional(pool)
        .await?;

        row.map(Self::decode_turn_row).transpose()
    }

    async fn list_turns(&self, thread_id: &str) -> Result<Vec<TurnRuntime>> {
        let pool = self.pool().await?;
        let rows = sqlx::query(
            r#"
            SELECT
                id, session_id, thread_id, status, input_text, error_message,
                context_override_json, created_at, started_at, completed_at, updated_at
            FROM turn_runtimes
            WHERE thread_id = ?
            ORDER BY created_at ASC, id ASC
        "#,
        )
        .bind(thread_id)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::decode_turn_row).collect()
    }

    async fn create_item(&self, item: ItemRuntime) -> Result<ItemRuntime> {
        self.write_item(item).await
    }

    async fn update_item(&self, item: ItemRuntime) -> Result<ItemRuntime> {
        self.write_item(item).await
    }

    async fn get_item(&self, item_id: &str) -> Result<Option<ItemRuntime>> {
        let pool = self.pool().await?;
        let row = sqlx::query(
            r#"
            SELECT
                id, thread_id, turn_id, sequence, status, started_at,
                completed_at, updated_at, payload_json
            FROM item_runtimes
            WHERE id = ?
        "#,
        )
        .bind(item_id)
        .fetch_optional(pool)
        .await?;

        row.map(Self::decode_item_row).transpose()
    }

    async fn list_items(&self, thread_id: &str) -> Result<Vec<ItemRuntime>> {
        let pool = self.pool().await?;
        let rows = sqlx::query(
            r#"
            SELECT
                i.id, i.thread_id, i.turn_id, i.sequence, i.status, i.started_at,
                i.completed_at, i.updated_at, i.payload_json
            FROM item_runtimes i
            LEFT JOIN turn_runtimes t ON t.id = i.turn_id
            WHERE i.thread_id = ?
            ORDER BY
                COALESCE(t.started_at, t.created_at, i.started_at) ASC,
                i.sequence ASC,
                i.turn_id ASC,
                i.started_at ASC,
                i.id ASC
        "#,
        )
        .bind(thread_id)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Self::decode_item_row).collect()
    }
}

#[derive(Default)]
pub struct NoopThreadRuntimeStore;

#[async_trait]
impl ThreadRuntimeStore for NoopThreadRuntimeStore {
    async fn upsert_thread(&self, thread: ThreadRuntime) -> Result<ThreadRuntime> {
        Ok(thread)
    }

    async fn list_threads(&self, _session_id: &str) -> Result<Vec<ThreadRuntime>> {
        Ok(Vec::new())
    }

    async fn get_thread(&self, _thread_id: &str) -> Result<Option<ThreadRuntime>> {
        Ok(None)
    }

    async fn delete_session(&self, _session_id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_turn(&self, turn: TurnRuntime) -> Result<TurnRuntime> {
        Ok(turn)
    }

    async fn update_turn(&self, turn: TurnRuntime) -> Result<TurnRuntime> {
        Ok(turn)
    }

    async fn get_turn(&self, _turn_id: &str) -> Result<Option<TurnRuntime>> {
        Ok(None)
    }

    async fn list_turns(&self, _thread_id: &str) -> Result<Vec<TurnRuntime>> {
        Ok(Vec::new())
    }

    async fn create_item(&self, item: ItemRuntime) -> Result<ItemRuntime> {
        Ok(item)
    }

    async fn update_item(&self, item: ItemRuntime) -> Result<ItemRuntime> {
        Ok(item)
    }

    async fn get_item(&self, _item_id: &str) -> Result<Option<ItemRuntime>> {
        Ok(None)
    }

    async fn list_items(&self, _thread_id: &str) -> Result<Vec<ItemRuntime>> {
        Ok(Vec::new())
    }
}

#[derive(Default)]
pub struct InMemoryThreadRuntimeStore {
    threads: RwLock<HashMap<String, ThreadRuntime>>,
    turns: RwLock<HashMap<String, TurnRuntime>>,
    items: RwLock<HashMap<String, ItemRuntime>>,
}

#[async_trait]
impl ThreadRuntimeStore for InMemoryThreadRuntimeStore {
    async fn upsert_thread(&self, mut thread: ThreadRuntime) -> Result<ThreadRuntime> {
        thread.updated_at = Utc::now();
        self.threads
            .write()
            .await
            .insert(thread.id.clone(), thread.clone());
        Ok(thread)
    }

    async fn list_threads(&self, session_id: &str) -> Result<Vec<ThreadRuntime>> {
        let mut threads = self
            .threads
            .read()
            .await
            .values()
            .filter(|thread| thread.session_id == session_id)
            .cloned()
            .collect::<Vec<_>>();
        threads.sort_by_key(|thread| thread.created_at);
        Ok(threads)
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRuntime>> {
        Ok(self.threads.read().await.get(thread_id).cloned())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let thread_ids = self
            .threads
            .read()
            .await
            .values()
            .filter(|thread| thread.session_id == session_id)
            .map(|thread| thread.id.clone())
            .collect::<Vec<_>>();

        if thread_ids.is_empty() {
            return Ok(());
        }

        self.threads
            .write()
            .await
            .retain(|_, thread| thread.session_id != session_id);
        self.turns
            .write()
            .await
            .retain(|_, turn| !thread_ids.contains(&turn.thread_id));
        self.items
            .write()
            .await
            .retain(|_, item| !thread_ids.contains(&item.thread_id));
        Ok(())
    }

    async fn create_turn(&self, mut turn: TurnRuntime) -> Result<TurnRuntime> {
        turn.updated_at = Utc::now();
        self.turns
            .write()
            .await
            .insert(turn.id.clone(), turn.clone());
        Ok(turn)
    }

    async fn update_turn(&self, mut turn: TurnRuntime) -> Result<TurnRuntime> {
        turn.updated_at = Utc::now();
        self.turns
            .write()
            .await
            .insert(turn.id.clone(), turn.clone());
        Ok(turn)
    }

    async fn get_turn(&self, turn_id: &str) -> Result<Option<TurnRuntime>> {
        Ok(self.turns.read().await.get(turn_id).cloned())
    }

    async fn list_turns(&self, thread_id: &str) -> Result<Vec<TurnRuntime>> {
        let mut turns = self
            .turns
            .read()
            .await
            .values()
            .filter(|turn| turn.thread_id == thread_id)
            .cloned()
            .collect::<Vec<_>>();
        turns.sort_by_key(|turn| turn.created_at);
        Ok(turns)
    }

    async fn create_item(&self, mut item: ItemRuntime) -> Result<ItemRuntime> {
        item.updated_at = Utc::now();
        self.items
            .write()
            .await
            .insert(item.id.clone(), item.clone());
        Ok(item)
    }

    async fn update_item(&self, mut item: ItemRuntime) -> Result<ItemRuntime> {
        item.updated_at = Utc::now();
        self.items
            .write()
            .await
            .insert(item.id.clone(), item.clone());
        Ok(item)
    }

    async fn get_item(&self, item_id: &str) -> Result<Option<ItemRuntime>> {
        Ok(self.items.read().await.get(item_id).cloned())
    }

    async fn list_items(&self, thread_id: &str) -> Result<Vec<ItemRuntime>> {
        let turn_started_at = self
            .turns
            .read()
            .await
            .values()
            .filter(|turn| turn.thread_id == thread_id)
            .map(|turn| (turn.id.clone(), turn.started_at.unwrap_or(turn.created_at)))
            .collect::<HashMap<_, _>>();
        let mut items = self
            .items
            .read()
            .await
            .values()
            .filter(|item| item.thread_id == thread_id)
            .cloned()
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            turn_started_at
                .get(&left.turn_id)
                .copied()
                .unwrap_or(left.started_at)
                .cmp(
                    &turn_started_at
                        .get(&right.turn_id)
                        .copied()
                        .unwrap_or(right.started_at),
                )
                .then(left.sequence.cmp(&right.sequence))
                .then(left.turn_id.cmp(&right.turn_id))
                .then(left.id.cmp(&right.id))
        });
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn in_memory_store_persists_threads_turns_and_items() {
        let store = InMemoryThreadRuntimeStore::default();
        let thread = ThreadRuntime::new("thread-1", "session-1", PathBuf::from("/tmp"));
        let turn = TurnRuntime::new(
            "turn-1",
            "session-1",
            "thread-1",
            Some("hello".to_string()),
            None,
        );
        let now = Utc::now();
        let item = ItemRuntime {
            id: "item-1".to_string(),
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            sequence: 1,
            status: ItemStatus::Completed,
            started_at: now,
            completed_at: Some(now),
            updated_at: now,
            payload: ItemRuntimePayload::AgentMessage {
                text: "hello".to_string(),
            },
        };

        store.upsert_thread(thread.clone()).await.unwrap();
        store.create_turn(turn.clone()).await.unwrap();
        store.create_item(item.clone()).await.unwrap();

        let stored_thread = store.get_thread("thread-1").await.unwrap().unwrap();
        assert_eq!(stored_thread.id, thread.id);
        assert_eq!(stored_thread.session_id, thread.session_id);
        assert_eq!(stored_thread.working_dir, thread.working_dir);
        assert_eq!(stored_thread.status, thread.status);

        let stored_turn = store.get_turn("turn-1").await.unwrap().unwrap();
        assert_eq!(stored_turn.id, turn.id);
        assert_eq!(stored_turn.session_id, turn.session_id);
        assert_eq!(stored_turn.thread_id, turn.thread_id);
        assert_eq!(stored_turn.status, turn.status);
        assert_eq!(stored_turn.input_text, turn.input_text);
        let stored_item = store.get_item("item-1").await.unwrap().unwrap();
        assert_eq!(stored_item.id, item.id);
        assert_eq!(stored_item.turn_id, item.turn_id);
        assert_eq!(stored_item.sequence, item.sequence);
        assert_eq!(stored_item.status, item.status);
        assert_eq!(store.list_turns("thread-1").await.unwrap().len(), 1);
        assert_eq!(store.list_items("thread-1").await.unwrap().len(), 1);
        assert_eq!(store.list_threads("session-1").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn sqlite_store_persists_threads_turns_and_items_across_reopen() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("runtime.db");

        let thread = ThreadRuntime::new("thread-1", "session-1", PathBuf::from("/tmp"));
        let turn = TurnRuntime::new(
            "turn-1",
            "session-1",
            "thread-1",
            Some("hello".to_string()),
            None,
        );
        let now = Utc::now();
        let item = ItemRuntime {
            id: "item-1".to_string(),
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            sequence: 1,
            status: ItemStatus::Completed,
            started_at: now,
            completed_at: Some(now),
            updated_at: now,
            payload: ItemRuntimePayload::AgentMessage {
                text: "hello".to_string(),
            },
        };

        let store = SqliteThreadRuntimeStore::new(db_path.clone());
        store.upsert_thread(thread.clone()).await.unwrap();
        store.create_turn(turn.clone()).await.unwrap();
        store.create_item(item.clone()).await.unwrap();

        let reopened = SqliteThreadRuntimeStore::new(db_path);
        let stored_thread = reopened.get_thread("thread-1").await.unwrap().unwrap();
        assert_eq!(stored_thread.id, thread.id);
        assert_eq!(stored_thread.session_id, thread.session_id);
        assert_eq!(stored_thread.working_dir, thread.working_dir);

        let stored_turn = reopened.get_turn("turn-1").await.unwrap().unwrap();
        assert_eq!(stored_turn.id, turn.id);
        assert_eq!(stored_turn.thread_id, turn.thread_id);
        assert_eq!(stored_turn.input_text, turn.input_text);

        let stored_item = reopened.get_item("item-1").await.unwrap().unwrap();
        assert_eq!(stored_item.id, item.id);
        assert_eq!(stored_item.turn_id, item.turn_id);
        assert_eq!(stored_item.sequence, item.sequence);
        assert_eq!(stored_item.status, item.status);
        assert_eq!(reopened.list_turns("thread-1").await.unwrap().len(), 1);
        assert_eq!(reopened.list_items("thread-1").await.unwrap().len(), 1);
        assert_eq!(reopened.list_threads("session-1").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn sqlite_store_delete_session_removes_threads_turns_and_items() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("runtime.db");

        let store = SqliteThreadRuntimeStore::new(db_path.clone());
        let thread = ThreadRuntime::new("thread-1", "session-1", PathBuf::from("/tmp"));
        let turn = TurnRuntime::new(
            "turn-1",
            "session-1",
            "thread-1",
            Some("hello".to_string()),
            None,
        );
        let now = Utc::now();
        let item = ItemRuntime {
            id: "item-1".to_string(),
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            sequence: 1,
            status: ItemStatus::Completed,
            started_at: now,
            completed_at: Some(now),
            updated_at: now,
            payload: ItemRuntimePayload::AgentMessage {
                text: "hello".to_string(),
            },
        };

        store.upsert_thread(thread).await.unwrap();
        store.create_turn(turn).await.unwrap();
        store.create_item(item).await.unwrap();
        store.delete_session("session-1").await.unwrap();

        let reopened = SqliteThreadRuntimeStore::new(db_path);
        assert!(reopened.list_threads("session-1").await.unwrap().is_empty());
        assert!(reopened.get_turn("turn-1").await.unwrap().is_none());
        assert!(reopened.get_item("item-1").await.unwrap().is_none());
    }
}
