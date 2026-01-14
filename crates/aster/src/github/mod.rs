//! GitHub 集成模块
//!
//! 提供 GitHub Actions 工作流设置、PR 管理等功能

mod workflow;
mod pr;

pub use workflow::{
    setup_github_workflow, check_github_cli, GitHubCLIStatus,
    CLAUDE_CODE_WORKFLOW,
};
pub use pr::{
    PRInfo, PRComment, get_pr_info, get_pr_comments,
    add_pr_comment, create_pr, CreatePROptions,
};
