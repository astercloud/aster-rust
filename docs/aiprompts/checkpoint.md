# Checkpoint 系统

文件检查点系统，在编辑会话期间保存和恢复文件状态。

## 核心功能

- 自动和手动检查点创建
- 增量 diff 存储
- Git 集成
- 检查点浏览和搜索
- 多文件恢复
- 压缩和存储优化

## 模块结构

```
checkpoint/
├── diff.rs      # Diff 计算和应用
├── session.rs   # 检查点会话管理
├── storage.rs   # 存储后端
└── types.rs     # 类型定义
```

## 关键类型

- `Checkpoint` - 检查点数据结构
- `CheckpointSession` - 会话管理器
- `CheckpointStorage` - 存储接口
- `FileDiff` - 文件差异

## 使用场景

1. Agent 编辑文件前自动创建检查点
2. 用户手动保存当前状态
3. 出错时回滚到之前状态
4. 浏览历史修改记录

## 源码位置

`crates/aster/src/checkpoint/`
