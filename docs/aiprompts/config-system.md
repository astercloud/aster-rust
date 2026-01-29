# 配置系统

## 概述

配置系统管理 Aster 的各种设置，支持多层级配置和动态更新。

**核心路径**: `crates/aster/src/config/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `base.rs` | 核心配置结构 |
| `config_manager.rs` | 配置管理器 |
| `aster_mode.rs` | 运行模式 |
| `permission.rs` | 权限管理 |
| `extensions.rs` | 扩展配置 |
| `paths.rs` | 路径管理 |
| `agents_md_parser.rs` | AGENTS.md 解析 |
| `experiments.rs` | 实验功能 |

## 配置层级

```
┌─────────────────────────────────────┐
│         企业策略配置                 │  (最高优先级)
│    /etc/aster/policy.toml           │
├─────────────────────────────────────┤
│         用户全局配置                 │
│    ~/.aster/config.toml             │
├─────────────────────────────────────┤
│         项目配置                     │
│    .aster/config.toml               │
├─────────────────────────────────────┤
│         会话配置                     │  (最低优先级)
│    (运行时)                          │
└─────────────────────────────────────┘
```

## 核心配置

```rust
// crates/aster/src/config/base.rs
pub struct Config {
    // Provider 设置
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    
    // 运行模式
    pub mode: AsterMode,
    
    // 扩展
    pub extensions: Vec<ExtensionEntry>,
    
    // 权限
    pub permissions: PermissionConfig,
    
    // 实验功能
    pub experiments: ExperimentConfig,
}

impl Config {
    pub fn global() -> &'static Config;
    pub fn load() -> Result<Config>;
    pub fn get_aster_mode(&self) -> Option<AsterMode>;
}
```

## 运行模式

```rust
// crates/aster/src/config/aster_mode.rs
pub enum AsterMode {
    Auto,           // 自动执行
    SmartApprove,   // 智能审批
    Manual,         // 手动确认
}
```

## 配置管理器

```rust
// crates/aster/src/config/config_manager.rs
pub struct ConfigManager {
    options: ConfigManagerOptions,
}

impl ConfigManager {
    pub fn new(options: ConfigManagerOptions) -> Self;
    pub fn get<T>(&self, key: &str) -> Option<T>;
    pub fn set<T>(&self, key: &str, value: T) -> Result<()>;
    pub fn get_source(&self, key: &str) -> ConfigSourceInfo;
}

pub struct ConfigSourceInfo {
    pub source: ConfigSource,
    pub path: Option<PathBuf>,
}

pub enum ConfigSource {
    Default,
    EnvVar,
    GlobalConfig,
    ProjectConfig,
    SessionConfig,
    EnterprisePolicy,
}
```

## 扩展配置

```rust
// crates/aster/src/config/extensions.rs
pub struct ExtensionEntry {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub timeout: Option<Duration>,
    pub config: ExtensionConfig,
}

// 扩展管理函数
pub fn get_all_extensions() -> Vec<ExtensionEntry>;
pub fn get_enabled_extensions() -> Vec<ExtensionEntry>;
pub fn get_extension_by_name(name: &str) -> Option<ExtensionEntry>;
pub fn set_extension(entry: ExtensionEntry) -> Result<()>;
pub fn set_extension_enabled(name: &str, enabled: bool) -> Result<()>;
pub fn remove_extension(name: &str) -> Result<()>;
```

## 权限管理

```rust
// crates/aster/src/config/permission.rs
pub struct PermissionManager {
    mode: AsterMode,
    allowed_paths: HashSet<PathBuf>,
    denied_commands: HashSet<String>,
}

impl PermissionManager {
    pub fn check_file_access(&self, path: &Path) -> PermissionResult;
    pub fn check_command(&self, command: &str) -> PermissionResult;
    pub fn add_allowed_path(&mut self, path: PathBuf);
    pub fn add_denied_command(&mut self, command: String);
}
```

## AGENTS.md 解析

```rust
// crates/aster/src/config/agents_md_parser.rs
pub struct AgentsMdParser;

impl AgentsMdParser {
    pub fn parse(content: &str) -> Result<AgentsMdInfo>;
    pub fn parse_file(path: &Path) -> Result<AgentsMdInfo>;
}

pub struct AgentsMdInfo {
    pub instructions: Vec<String>,
    pub rules: Vec<String>,
    pub context: HashMap<String, String>,
}
```

## 配置命令

```rust
// crates/aster/src/config/config_command.rs
pub struct ConfigCommand {
    pub action: ConfigAction,
    pub key: Option<String>,
    pub value: Option<String>,
}

pub enum ConfigAction {
    Get,
    Set,
    List,
    Reset,
}

pub fn create_config_command(args: &[String]) -> Result<ConfigCommand>;
```

## 配置文件示例

```toml
# ~/.aster/config.toml

[provider]
type = "anthropic"
model = "claude-3-5-sonnet-20241022"
# api_key = "sk-..."  # 建议使用环境变量

[mode]
default = "smart_approve"

[permissions]
allowed_paths = ["/home/user/projects"]
denied_commands = ["rm -rf /", "sudo"]

[extensions]
enabled = ["filesystem", "github"]

[experiments]
parallel_agents = true
context_compression = true
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `ASTER_MODE` | 运行模式 |
| `ASTER_CONFIG_PATH` | 配置文件路径 |
| `OPENAI_API_KEY` | OpenAI API Key |
| `ANTHROPIC_API_KEY` | Anthropic API Key |
| `GOOGLE_API_KEY` | Google API Key |
