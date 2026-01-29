# Updater 自动更新

提供版本检查、下载、安装和回滚功能。

## 模块结构

```
updater/
├── checker.rs    # 版本检查
├── installer.rs  # 安装器
└── manager.rs    # 更新管理器
```

## 核心组件

### UpdateManager
```rust
pub struct UpdateManager;

impl UpdateManager {
    pub async fn check_for_updates() -> UpdateCheckResult;
    pub async fn perform_update(options: UpdateOptions) -> UpdateStatus;
    pub async fn rollback_version(version: &str) -> Result<()>;
    pub fn list_versions() -> Vec<VersionInfo>;
}
```

### UpdateConfig
```rust
pub struct UpdateConfig {
    pub channel: UpdateChannel,
    pub auto_check: bool,
    pub auto_install: bool,
}

pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
}
```


### Installer
```rust
pub struct Installer;

impl Installer {
    pub async fn download(version: &str) -> DownloadProgress;
    pub async fn install(options: InstallOptions) -> InstallResult;
}

pub struct DownloadProgress {
    pub phase: DownloadPhase,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}
```

## 版本比较

```rust
pub fn compare_versions(a: &str, b: &str) -> Ordering;
pub fn check_version() -> UpdateCheckResult;
```

## 使用场景

- 自动检查更新
- 手动更新安装
- 版本回滚

## 源码位置

`crates/aster/src/updater/`
