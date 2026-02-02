//! Session 模块
//!
//! 提供 session 管理功能，包括：
//! - `SessionStore` trait: 可插拔的存储抽象
//! - `SessionManager`: 向后兼容的静态方法（使用全局 store）
//! - SQLite 默认实现
//!
//! ## 使用方式
//!
//! ### 方式 1: 使用默认 SQLite 存储（向后兼容）
//! ```ignore
//! use aster::session::SessionManager;
//! let session = SessionManager::create_session(dir, name, session_type).await?;
//! ```
//!
//! ### 方式 2: 注入自定义存储（推荐）
//! ```ignore
//! use aster::session::{SessionStore, NoopSessionStore};
//! use aster::agents::Agent;
//!
//! let store = Arc::new(MyCustomStore::new());
//! let agent = Agent::new().with_session_store(store);
//! ```

mod archive;
mod chat_history_search;
mod cleanup;
mod diagnostics;
mod export;
pub mod extension_data;
mod fork;
mod legacy;
pub mod resume;
pub mod session_manager;
mod statistics;
mod store;

// 导出存储抽象
pub use store::{
    get_global_session_store, is_global_session_store_set, set_global_session_store,
    ChatHistoryMatch, NoopSessionStore, SessionStore,
};

// 导出现有功能（向后兼容）
pub use archive::{
    archive_and_delete_session, archive_session, bulk_archive_sessions, delete_archived_session,
    list_archived_sessions, restore_archived_session, BulkArchiveResult,
};
pub use cleanup::{
    cleanup_expired_data, force_cleanup, get_cutoff_date, schedule_cleanup, CleanupStats,
    DEFAULT_CLEANUP_PERIOD_DAYS,
};
pub use diagnostics::generate_diagnostics;
pub use export::{
    bulk_export_sessions, export_session, export_session_to_file, ExportFormat, ExportOptions,
};
pub use extension_data::{EnabledExtensionsState, ExtensionData, ExtensionState, TodoState};
pub use fork::{
    fork_session, get_session_branch_tree, merge_sessions, ForkMetadata, ForkOptions, MergeOptions,
    MergeStrategy, MetadataStrategy, SessionBranchTree,
};
pub use resume::{
    build_resume_message, delete_summary, has_summary, list_summaries, load_summary,
    load_summary_data, save_summary, SummaryCacheData,
};
pub use session_manager::{Session, SessionInsights, SessionManager, SessionType};
pub use statistics::{
    calculate_statistics, generate_report, get_all_statistics, SessionStatistics, SessionSummary,
};
