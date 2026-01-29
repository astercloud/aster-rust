# 遥测系统

跟踪使用统计和事件，本地存储，支持批量上报。

## 模块结构

```
telemetry/
├── config.rs     # 遥测配置
├── sanitizer.rs  # 数据脱敏
├── tracker.rs    # 事件追踪器
└── types.rs      # 类型定义
```

## 核心功能

### TelemetryTracker
追踪和记录事件：
- 会话开始/结束
- 工具调用统计
- 错误事件
- 性能指标

### TelemetrySanitizer
数据脱敏处理：
- 移除敏感路径
- 脱敏用户信息
- 清理 API 密钥

## 配置选项

```rust
pub struct TelemetryConfig {
    pub enabled: bool,
    pub local_only: bool,
    pub batch_size: usize,
    pub flush_interval: Duration,
}
```


## 事件类型

- `SessionStart` - 会话开始
- `SessionEnd` - 会话结束
- `ToolCall` - 工具调用
- `Error` - 错误发生
- `TokenUsage` - Token 使用量

## 隐私保护

- 默认本地存储
- 可选上报功能
- 自动数据脱敏
- 用户可完全禁用

## 源码位置

`crates/aster/src/telemetry/`
