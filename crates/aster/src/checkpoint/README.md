# 文件检查点系统

在编辑会话期间保存和恢复文件状态。

## 模块概览

### 1. 类型定义 (types.rs)

核心类型定义：
- `FileCheckpoint` - 文件检查点
- `CheckpointSearchOptions` - 搜索选项
- `CheckpointRestoreOptions` - 恢复选项
- `CheckpointStats` - 统计信息
- `CheckpointHistory` - 历史记录

### 2. 会话管理 (session.rs)

检查点会话和管理器：
- `CheckpointSession` - 会话状态
- `CheckpointManager` - 主要管理器
  - `init()` - 初始化系统
  - `create_checkpoint()` - 创建检查点
  - `restore_checkpoint()` - 恢复检查点
  - `undo()` / `redo()` - 撤销/重做
  - `get_checkpoint_history()` - 获取历史


### 3. 存储管理 (storage.rs)

磁盘存储操作：
- `CheckpointStorage` - 存储管理器
  - `save_checkpoint()` - 保存检查点
  - `load_session()` - 加载会话
  - `cleanup_old_checkpoints()` - 清理旧数据
  - `compress_content()` / `decompress_content()` - 压缩/解压

### 4. Diff 引擎 (diff.rs)

文件差异计算：
- `DiffEngine` - Diff 引擎
  - `calculate_diff()` - 计算差异
  - `apply_diff()` - 应用差异
  - LCS (最长公共子序列) 算法

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块入口和导出 |
| `types.rs` | 类型定义 |
| `session.rs` | 会话管理 |
| `storage.rs` | 存储管理 |
| `diff.rs` | Diff 引擎 |

## 使用示例

```rust
use aster::checkpoint::CheckpointManager;

// 创建管理器
let manager = CheckpointManager::new();

// 初始化
manager.init(None, 5).await?;

// 创建检查点
manager.create_checkpoint("src/main.rs", None).await;

// 恢复检查点
manager.restore_checkpoint("src/main.rs", Some(0), None).await;

// Undo/Redo
manager.undo("src/main.rs").await;
manager.redo("src/main.rs").await;
```

## 功能特性

- 自动检查点（每 N 次编辑）
- 增量 diff 存储
- Git 集成
- 压缩存储
- 会话持久化
- 过期自动清理
