# Sandbox 沙箱系统

提供进程隔离、文件系统沙箱、网络沙箱等安全功能。

## 模块结构

```
sandbox/
├── config.rs          # 沙箱配置
├── executor.rs        # 沙箱执行器
├── filesystem.rs      # 文件系统沙箱
└── resource_limits.rs # 资源限制
```

## 核心类型

### SandboxConfig
```rust
pub struct SandboxConfig {
    pub preset: SandboxPreset,
    pub resource_limits: ResourceLimits,
    pub filesystem_policy: FilesystemPolicy,
}
```

### SandboxPreset
预定义的沙箱配置级别：
- `Minimal` - 最小限制
- `Standard` - 标准限制
- `Strict` - 严格限制
- `Custom` - 自定义

## 关键功能

- `execute_in_sandbox()` - 在沙箱中执行命令
- `detect_best_sandbox()` - 检测最佳沙箱方案
- `get_sandbox_capabilities()` - 获取沙箱能力

## 文件系统策略

```rust
pub struct FilesystemPolicy {
    pub rules: Vec<PathRule>,
    pub default_action: Action,
}

pub struct PathRule {
    pub path: PathBuf,
    pub action: Action,  // Allow/Deny/ReadOnly
}
```

## 资源限制

- CPU 时间限制
- 内存使用限制
- 文件描述符限制
- 进程数限制

## 源码位置

`crates/aster/src/sandbox/`
