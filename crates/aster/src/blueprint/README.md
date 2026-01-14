# 蓝图系统 (Blueprint System)


## 概述

蓝图系统提供项目级别的需求管理和任务执行框架，支持：

1. **蓝图设计和管理** - 通过对话生成项目蓝图
2. **任务树生成和执行** - 从蓝图自动生成层级化任务结构
3. **TDD 驱动的开发循环** - 测试先行的开发方法
4. **主/子 Agent 协调** - 蜂王-蜜蜂协作模型
5. **检查点和时光倒流** - 支持回滚的快照系统
6. **边界检查器** - 模块边界验证和保护

## 核心概念

### Blueprint（蓝图）
需求调研后形成的目标业务流程、功能边界和系统架构草图。

### TaskTree（任务树）
由蓝图推导出的层级化任务结构，每个任务节点包含：
- 任务描述和优先级
- 依赖关系
- TDD 测试规格
- 验收测试（由蜂王生成）
- 检查点

### TDD Loop
每个 Agent 都在以下循环中工作：
1. 编写测试（红灯）
2. 运行测试确认失败
3. 编写实现代码
4. 运行测试确认通过（绿灯）
5. 重构优化

### 蜂王-蜜蜂模型
- **蜂王（Queen Agent）**：全局视野，负责任务分配、验收测试生成、协调
- **蜜蜂（Worker Agent）**：在各自的任务分支上工作，执行具体任务

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块入口，导出公共 API |
| `types.rs` | 类型定义（Blueprint, TaskTree, TaskNode 等） |
| `blueprint_manager.rs` | 蓝图管理器，CRUD 和状态流转 |
| `task_tree_manager.rs` | 任务树管理器，生成和执行 |
| `tdd_executor.rs` | TDD 执行器，循环管理和阶段转换 |
| `time_travel.rs` | 时光倒流管理器，检查点和回滚 |
| `boundary_checker.rs` | 边界检查器，模块边界验证 |
| `agent_coordinator.rs` | Agent 协调器，蜂王-蜜蜂模型 |
| `worker_executor.rs` | Worker 执行器，TDD 阶段执行逻辑 |
| `worker_sandbox.rs` | Worker 沙箱，文件隔离和锁机制 |
| `acceptance_test_generator.rs` | 验收测试生成器，由蜂王生成 |
| `acceptance_test_runner.rs` | 验收测试运行器，执行和验证 |
| `task_granularity.rs` | 任务粒度控制器，自动拆分/合并 |
| `blueprint_context.rs` | 蓝图上下文单例，工具边界检查桥梁 |
| `codebase_analyzer.rs` | 代码库分析器，逆向生成蓝图 |
| `requirement_dialog.rs` | 需求对话管理器，ERP 式需求收集 |
| `tests.rs` | 单元测试 |

## 使用示例

```rust
use aster::blueprint::{BlueprintManager, TaskTreeManager, Blueprint};

// 创建蓝图管理器
let bp_manager = BlueprintManager::default();

// 创建蓝图
let blueprint = bp_manager.create_blueprint(
    "我的项目".to_string(),
    "项目描述".to_string(),
).await?;

// 添加系统模块
bp_manager.add_module(&blueprint.id, SystemModule {
    name: "用户服务".to_string(),
    module_type: ModuleType::Backend,
    responsibilities: vec!["用户认证".to_string()],
    // ...
}).await?;

// 提交审核
bp_manager.submit_for_review(&blueprint.id).await?;

// 批准蓝图
bp_manager.approve_blueprint(&blueprint.id, Some("admin".to_string())).await?;

// 生成任务树
let tree_manager = TaskTreeManager::default();
let task_tree = tree_manager.generate_from_blueprint(&blueprint).await?;

// 获取可执行任务
let executable = tree_manager.get_executable_tasks(&task_tree.id).await;
```

## 状态流转

### 蓝图状态
```
Draft -> Review -> Approved -> Executing -> Completed
                     |            |
                     v            v
                  Rejected     Paused -> Modified
```

### 任务状态
```
Pending -> TestWriting -> Coding -> Testing -> Passed -> Approved
    |                        |         |
    v                        v         v
 Blocked                TestFailed  Rejected
```

## 与 TypeScript 版本的对应关系

| TypeScript | Rust | 状态 |
|------------|------|------|
| `blueprint-manager.ts` | `blueprint_manager.rs` | ✅ 完成 |
| `task-tree-manager.ts` | `task_tree_manager.rs` | ✅ 完成 |
| `types.ts` | `types.rs` | ✅ 完成 |
| `tdd-executor.ts` | `tdd_executor.rs` | ✅ 完成 |
| `agent-coordinator.ts` | `agent_coordinator.rs` | ✅ 完成 |
| `time-travel.ts` | `time_travel.rs` | ✅ 完成 |
| `boundary-checker.ts` | `boundary_checker.rs` | ✅ 完成 |
| `worker-executor.ts` | `worker_executor.rs` | ✅ 完成 |
| `worker-sandbox.ts` | `worker_sandbox.rs` | ✅ 完成 |
| `acceptance-test-generator.ts` | `acceptance_test_generator.rs` | ✅ 完成 |
| `acceptance-test-runner.ts` | `acceptance_test_runner.rs` | ✅ 完成 |
| `task-granularity.ts` | `task_granularity.rs` | ✅ 完成 |
| `blueprint-context.ts` | `blueprint_context.rs` | ✅ 完成 |
| `codebase-analyzer.ts` | `codebase_analyzer.rs` | ✅ 完成 |
| `requirement-dialog.ts` | `requirement_dialog.rs` | ✅ 完成 |

## 实现状态

所有核心模块已完成实现，包括：
- 蓝图管理和状态流转
- 任务树生成和执行
- TDD 驱动开发循环
- 蜂王-蜜蜂协调模型
- 时光倒流（检查点/回滚）
- 模块边界检查
- Worker 执行器和沙箱隔离
- 验收测试生成和运行
- 任务粒度自动控制
- 蓝图上下文单例（工具边界检查桥梁）
- 代码库分析器（逆向生成蓝图）
- 需求对话管理器（ERP 式需求收集）
