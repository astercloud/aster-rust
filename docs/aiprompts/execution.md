# Execution 执行管理

统一的 Agent 生命周期管理，支持会话隔离。

## 模块结构

```
execution/
└── manager.rs  # 执行管理器
```

## 会话执行模式

```rust
pub enum SessionExecutionMode {
    Interactive,      // 交互式聊天
    Background,       // 后台/定时任务
    SubTask {         // 子任务
        parent_session: String,
    },
}
```

## 工厂方法

```rust
impl SessionExecutionMode {
    pub fn chat() -> Self;       // 交互模式
    pub fn scheduled() -> Self;  // 定时模式
    pub fn task(parent: String) -> Self;  // 子任务
}
```

## 使用场景

- 多会话并发执行
- 独立的 Agent 实例
- 父子任务关联

## 源码位置

`crates/aster/src/execution/`
