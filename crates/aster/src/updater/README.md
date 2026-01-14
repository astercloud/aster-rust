# 自动更新系统

提供版本检查和更新功能。

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块导出 |
| `checker.rs` | 版本检查器：版本比较、更新检查 |
| `manager.rs` | 更新管理器：下载、安装、状态管理 |

## 核心功能

### VersionInfo / UpdateCheckResult
- 版本信息结构
- 更新检查结果

### UpdateManager
- 更新状态管理
- 版本检查
- 下载和安装
- 支持 dry-run 模式

## 使用示例

```rust
use aster::updater::{UpdateManager, UpdateConfig, UpdateOptions};

let manager = UpdateManager::new(UpdateConfig::default());

// 检查更新
let result = manager.check_for_updates().await?;
if result.has_update {
    println!("发现新版本: {}", result.latest_version);
    
    // 下载并安装
    manager.download(None, &UpdateOptions::default()).await?;
    manager.install(None, &UpdateOptions::default()).await?;
}
```


