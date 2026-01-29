# GitHub 集成

提供 GitHub Actions 工作流设置、PR 管理等功能。

## 模块结构

```
github/
├── pr.rs        # PR 管理
└── workflow.rs  # 工作流设置
```

## PR 管理

```rust
pub struct PRInfo {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: String,
    pub author: String,
}

pub struct CreatePROptions {
    pub title: String,
    pub body: String,
    pub base: String,
    pub head: String,
}

pub async fn create_pr(options: CreatePROptions) -> Result<PRInfo>;
pub async fn get_pr_info(number: u64) -> Result<PRInfo>;
pub async fn add_pr_comment(number: u64, body: &str) -> Result<()>;
pub async fn get_pr_comments(number: u64) -> Result<Vec<PRComment>>;
```


## GitHub Actions 工作流

```rust
pub const CLAUDE_CODE_WORKFLOW: &str;  // 预定义工作流模板

pub fn check_github_cli() -> GitHubCLIStatus;
pub fn setup_github_workflow() -> Result<()>;
```

### GitHubCLIStatus
```rust
pub enum GitHubCLIStatus {
    Available,
    NotInstalled,
    NotAuthenticated,
}
```

## 使用场景

- 自动创建 PR
- 代码审查评论
- CI/CD 集成

## 源码位置

`crates/aster/src/github/`
