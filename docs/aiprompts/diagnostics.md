# 诊断系统

提供系统健康检查、故障排除功能。

## 模块结构

```
diagnostics/
├── checker.rs  # 诊断检查器
├── health.rs   # 健康检查
├── network.rs  # 网络检查
├── report.rs   # 报告生成
└── system.rs   # 系统检查
```

## 检查类型

### 环境检查
- Git 可用性
- Ripgrep 可用性
- 必要工具检测

### 系统资源检查
- CPU 使用率
- 内存使用率
- 磁盘空间

### 网络检查
- API 连通性
- 代理配置
- DNS 解析


### 配置检查
- MCP 服务器配置
- 会话目录权限
- 缓存状态

## 核心 API

```rust
pub fn run_diagnostics(options: DiagnosticOptions) -> DiagnosticReport;
pub fn quick_health_check() -> HealthStatus;
pub fn get_system_health_summary() -> HealthSummary;
```

## 自动修复

```rust
pub struct AutoFixer;

impl AutoFixer {
    pub fn fix(check: &DiagnosticCheck) -> AutoFixResult;
}
```

## 健康状态

```rust
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

## 使用场景

- 启动时环境检查
- 故障排除
- 性能监控

## 源码位置

`crates/aster/src/diagnostics/`
