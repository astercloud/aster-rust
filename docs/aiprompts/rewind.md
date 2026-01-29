# Rewind 回退系统

提供对话和文件状态的回退功能。

## 模块结构

```
rewind/
├── file_history.rs  # 文件历史追踪
└── manager.rs       # 回退管理器
```

## 核心类型

### RewindManager
```rust
pub struct RewindManager;

impl RewindManager {
    pub fn create_snapshot() -> SnapshotDetails;
    pub fn preview_rewind(option: RewindOption) -> RewindPreview;
    pub fn execute_rewind(option: RewindOption) -> RewindOperationResult;
}
```

### FileHistoryManager
```rust
pub struct FileHistoryManager;

impl FileHistoryManager {
    pub fn track_file(path: &Path);
    pub fn create_snapshot(path: &Path) -> FileSnapshot;
    pub fn restore(snapshot: &FileSnapshot) -> RewindResult;
}
```


## 全局实例管理

```rust
pub fn get_rewind_manager() -> &'static RewindManager;
pub fn cleanup_rewind_manager();
pub fn cleanup_all_rewind_managers();
```

## 回退选项

```rust
pub enum RewindOption {
    ToMessage(MessageId),
    ToSnapshot(SnapshotId),
    Steps(usize),
}

pub struct RewindPreview {
    pub messages_to_remove: Vec<RewindableMessage>,
    pub files_to_restore: Vec<FileBackup>,
}
```

## 使用场景

- 撤销 Agent 的错误操作
- 回退到之前的对话状态
- 恢复被误删的文件

## 源码位置

`crates/aster/src/rewind/`
