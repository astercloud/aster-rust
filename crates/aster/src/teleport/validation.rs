//! Teleport 仓库验证
//!
//! 确保远程会话在正确的 Git 仓库中运行

use super::types::{RepoValidationResult, RepoValidationStatus};
use tokio::process::Command;

/// 获取当前 Git 仓库远程 URL
pub async fn get_current_repo_url() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.is_empty() {
            None
        } else {
            Some(url)
        }
    } else {
        None
    }
}

/// 规范化仓库 URL
pub fn normalize_repo_url(url: &str) -> String {
    let mut normalized = url.trim().to_string();

    // 移除 .git 后缀
    if normalized.ends_with(".git") {
        normalized = normalized[..normalized.len() - 4].to_string();
    }

    // 转换 SSH 格式为 HTTPS
    if let Some(captures) = normalized.strip_prefix("git@") {
        if let Some((host, path)) = captures.split_once(':') {
            normalized = format!("https://{}/{}", host, path);
        }
    }

    // 移除尾部斜杠
    if normalized.ends_with('/') {
        normalized.pop();
    }

    normalized.to_lowercase()
}


/// 比较两个仓库 URL 是否相同
pub fn compare_repo_urls(url1: &str, url2: &str) -> bool {
    normalize_repo_url(url1) == normalize_repo_url(url2)
}

/// 验证会话仓库是否匹配当前仓库
pub async fn validate_session_repository(session_repo: Option<&str>) -> RepoValidationResult {
    // 如果会话没有仓库信息，不需要验证
    let Some(session_repo) = session_repo else {
        return RepoValidationResult {
            status: RepoValidationStatus::NoValidation,
            session_repo: None,
            current_repo: None,
            error_message: None,
        };
    };

    // 获取当前仓库
    let current_repo = match get_current_repo_url().await {
        Some(repo) => repo,
        None => {
            return RepoValidationResult {
                status: RepoValidationStatus::Error,
                session_repo: Some(session_repo.to_string()),
                current_repo: None,
                error_message: Some("当前目录不是 git 仓库".to_string()),
            };
        }
    };

    // 比较仓库
    if compare_repo_urls(session_repo, &current_repo) {
        RepoValidationResult {
            status: RepoValidationStatus::Match,
            session_repo: Some(session_repo.to_string()),
            current_repo: Some(current_repo),
            error_message: None,
        }
    } else {
        RepoValidationResult {
            status: RepoValidationStatus::Mismatch,
            session_repo: Some(session_repo.to_string()),
            current_repo: Some(current_repo),
            error_message: None,
        }
    }
}

/// 获取当前分支名
pub async fn get_current_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .await
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            None
        } else {
            Some(branch)
        }
    } else {
        None
    }
}

/// 检查工作目录是否干净
pub async fn is_working_directory_clean() -> bool {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().is_empty()
        }
        _ => false,
    }
}
