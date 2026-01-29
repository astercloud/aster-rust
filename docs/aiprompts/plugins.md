# 插件系统

提供插件加载、管理、生命周期控制等功能。

## 模块结构

```
plugins/
├── context.rs   # 插件上下文
├── manager.rs   # 插件管理器
├── registry.rs  # 插件注册表
├── types.rs     # 类型定义
└── version.rs   # 版本检查
```

## 核心组件

### PluginManager
```rust
pub struct PluginManager;

impl PluginManager {
    pub fn discover() -> Vec<PluginMetadata>;
    pub fn load(name: &str) -> Result<Plugin>;
    pub fn unload(name: &str) -> Result<()>;
    pub fn emit(event: PluginEvent);
}
```

### PluginRegistry
```rust
pub struct PluginRegistry;

impl PluginRegistry {
    pub fn register_tool(def: ToolDefinition);
    pub fn register_command(def: CommandDefinition);
    pub fn register_skill(def: SkillDefinition);
    pub fn register_hook(def: HookDefinition);
}
```


## 插件定义

```rust
pub struct Plugin {
    pub metadata: PluginMetadata,
    pub state: PluginState,
    pub config: PluginConfig,
}

pub struct PluginMetadata {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<String>,
}

pub enum PluginState {
    Discovered,
    Loading,
    Active,
    Error(String),
    Unloaded,
}
```

## 插件上下文

```rust
pub struct PluginContext {
    pub logger: PluginLogger,
    pub config: PluginConfigAPI,
}
```

## 版本兼容性

```rust
pub struct VersionChecker;
pub struct Version { major, minor, patch };
```

## 源码位置

`crates/aster/src/plugins/`
