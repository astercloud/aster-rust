# Background 后台任务

提供任务队列、Shell 管理、超时处理和状态持久化功能。

## 模块结构

```
background/
├── persistence.rs    # 状态持久化
├── shell_manager.rs  # Shell 管理器
├── task_queue.rs     # 任务队列
├── timeout.rs        # 超时处理
└── types.rs          # 类型定义
```

## 任务队列

简单的后台任务队列实现。

## Shell 管理器

管理后台 Shell 进程：
- 创建/销毁 Shell
- 命令执行
- 输出捕获

## 超时处理

任务超时检测和处理。

## 状态持久化

后台任务状态的保存和恢复。

## 源码位置

`crates/aster/src/background/`
