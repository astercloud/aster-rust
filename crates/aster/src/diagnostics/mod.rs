//! 诊断和健康检查系统
//!
//! 提供系统健康检查、故障排除功能

mod checker;
mod report;

pub use checker::{
    DiagnosticChecker, DiagnosticCheck, CheckStatus,
    run_diagnostics, quick_health_check,
};
pub use report::{
    DiagnosticReport, DiagnosticOptions, SystemInfo,
    format_diagnostic_report,
};
