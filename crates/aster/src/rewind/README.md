# Rewind 功能模块

提供对话和文件状态的回退功能。

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块导出 |
| `file_history.rs` | 文件历史跟踪：备份、快照、恢复 |
| `manager.rs` | Rewind 管理器：协调文件和对话回退 |

## 核心功能

### FileHistoryManager
- 文件修改跟踪
- 快照创建和管理
- 文件状态恢复
- 差异计算

### RewindManager
- 用户消息记录
- 文件修改记录
- 回退操作执行
- 回退预览

## 使用示例

```rust
use aster::rewind::{RewindManager, RewindOption};

let mut manager = RewindManager::new("session-123");

// 记录用户消息
manager.record_user_message("msg-1");

// 记录文件修改
manager.record_file_change("src/main.rs");

// 预览回退
let preview = manager.preview_rewind("msg-1", RewindOption::Both);
println!("将修改 {} 个文件", preview.files_will_change.len());

// 执行回退
let result = manager.rewind("msg-1", RewindOption::Code);
if result.success {
    println!("回退成功");
}
```


