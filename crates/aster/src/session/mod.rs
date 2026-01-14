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
