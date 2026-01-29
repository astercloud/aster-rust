# Blueprint 蓝图系统

需求驱动的开发系统，支持 TDD 和多 Agent 协调。

## 核心概念

- **Blueprint** - 目标业务流程、功能边界和系统架构草图
- **TaskTree** - 层级化任务结构
- **TDD Loop** - 任务→测试→编码→验证循环
- **Checkpoint** - 时光倒流快照系统

## 模块结构

```
blueprint/
├── blueprint_manager.rs      # 蓝图管理
├── task_tree_manager.rs      # 任务树管理
├── tdd_executor.rs           # TDD 执行器
├── agent_coordinator.rs      # Agent 协调器
├── time_travel.rs            # 时光倒流
├── boundary_checker.rs       # 边界检查
├── worker_executor.rs        # Worker 执行
└── ...
```


## 核心组件

### BlueprintManager
```rust
pub struct BlueprintManager;

impl BlueprintManager {
    pub async fn create_blueprint(name: String, desc: String) -> Blueprint;
    pub fn generate_blueprint_summary(bp: &Blueprint) -> String;
}
```

### TaskTreeManager
```rust
pub struct TaskTreeManager;

impl TaskTreeManager {
    pub async fn generate_from_blueprint(bp: &Blueprint) -> TaskTree;
}
```

### TddExecutor
```rust
pub struct TddExecutor {
    config: TddConfig,
    state: TddLoopState,
}
```

### AgentCoordinator (蜂王-蜜蜂模型)
```rust
pub struct AgentCoordinator {
    config: CoordinatorConfig,
    strategy: ModelStrategy,
}
```

## 源码位置

`crates/aster/src/blueprint/`
