# Git 集成

提供 Git 状态检测、分支信息、安全检查等功能。

## 模块结构

```
git/
├── core.rs    # 核心 Git 操作
└── safety.rs  # 安全检查
```

## 核心功能

### GitUtils
```rust
pub struct GitUtils;

impl GitUtils {
    pub fn get_git_info() -> GitInfo;
    pub fn get_git_status() -> GitStatus;
    pub fn get_current_branch() -> String;
    pub fn get_default_branch() -> String;
    pub fn is_git_repository() -> bool;
}
```

### GitInfo
```rust
pub struct GitInfo {
    pub is_repo: bool,
    pub branch: String,
    pub remote: Option<String>,
    pub has_changes: bool,
}
```

### GitStatus
```rust
pub struct GitStatus {
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
}
```

## 安全检查

### GitSafety
```rust
pub struct GitSafety;

impl GitSafety {
    pub fn is_dangerous_command(cmd: &str) -> bool;
    pub fn check_sensitive_files() -> SensitiveFilesCheck;
}
```

检测危险命令：
- `git push --force`
- `git reset --hard`
- `git clean -fd`

## 使用场景

- Agent 执行前检查 Git 状态
- 防止危险 Git 操作
- 自动提交变更

## 源码位置

`crates/aster/src/git/`
