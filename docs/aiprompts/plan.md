# Plan 计划系统

提供计划持久化、版本控制和多方案对比功能。

## 模块结构

```
plan/
├── comparison.rs   # 方案对比
├── persistence.rs  # 持久化
└── types.rs        # 类型定义
```

## 功能

- 计划持久化存储
- 版本控制
- 多方案对比分析

## 使用场景

- 保存 Agent 生成的计划
- 比较不同实现方案
- 计划版本管理

## 源码位置

`crates/aster/src/plan/`
