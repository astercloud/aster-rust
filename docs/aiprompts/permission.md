# 权限系统

## 概述

权限系统控制工具执行的安全性，包括权限检查、审计日志和模板管理。

**核心路径**: `crates/aster/src/permission/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `types.rs` | 核心类型定义 |
| `manager.rs` | 权限管理器 |
| `audit.rs` | 审计日志 |
| `condition.rs` | 条件评估 |
| `pattern.rs` | 模式匹配 |
| `restriction.rs` | 参数限制 |
| `templates.rs` | 权限模板 |
| `merger.rs` | 权限合并 |
| `integration.rs` | 系统集成 |
| `migration.rs` | 迁移工具 |

## 核心类型

### ToolPermission

```rust
pub struct ToolPermission {
    pub tool_pattern: String,
    pub scope: PermissionScope,
    pub restrictions: Vec<ParameterRestriction>,
    pub conditions: Vec<PermissionCondition>,
    pub inheritance: PermissionInheritance,
    pub expires_at: Option<DateTime<Utc>>,
}

pub enum PermissionScope {
    Allow,
    Deny,
    Ask,
}
```

### ParameterRestriction

```rust
pub struct ParameterRestriction {
    pub parameter: String,
    pub restriction_type: RestrictionType,
    pub value: Value,
}

pub enum RestrictionType {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    Regex,
    Range,
    OneOf,
    PathPrefix,
    MaxLength,
}
```

### PermissionCondition

```rust
pub struct PermissionCondition {
    pub condition_type: ConditionType,
    pub operator: ConditionOperator,
    pub value: Value,
}

pub enum ConditionType {
    WorkingDirectory,
    TimeOfDay,
    DayOfWeek,
    Environment,
    SessionType,
    Custom(String),
}
```

## 权限管理器

```rust
pub struct ToolPermissionManager {
    config: PermissionConfig,
    permissions: RwLock<Vec<ToolPermission>>,
    audit_logger: AuditLogger,
}

impl ToolPermissionManager {
    pub fn new(config: PermissionConfig) -> Self;
    
    // 检查权限
    pub async fn check_permission(
        &self,
        tool_name: &str,
        parameters: &Value,
        context: &PermissionContext,
    ) -> PermissionResult;
    
    // 添加权限
    pub async fn add_permission(&self, permission: ToolPermission);
    
    // 移除权限
    pub async fn remove_permission(&self, tool_pattern: &str);
    
    // 列出权限
    pub async fn list_permissions(&self) -> Vec<ToolPermission>;
}
```

## 审计日志

```rust
pub struct AuditLogger {
    level: AuditLogLevel,
    entries: RwLock<Vec<AuditLogEntry>>,
}

pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub parameters: Value,
    pub result: PermissionResult,
    pub context: PermissionContext,
}

pub enum AuditLogLevel {
    None,
    Denied,
    All,
}
```

## 权限模板

```rust
pub struct PermissionTemplates;

impl PermissionTemplates {
    // 只读文件访问
    pub fn read_only_files() -> ToolPermission;
    
    // 安全 Shell 命令
    pub fn safe_shell_commands() -> ToolPermission;
    
    // 项目目录限制
    pub fn project_directory_only(path: &Path) -> ToolPermission;
    
    // 工作时间限制
    pub fn working_hours_only() -> ToolPermission;
}
```

## 模式匹配

```rust
// 检查是否包含通配符
pub fn has_wildcards(pattern: &str) -> bool;

// 匹配模式
pub fn match_pattern(pattern: &str, value: &str) -> bool;

// 转换为正则
pub fn pattern_to_regex(pattern: &str) -> Regex;
```

支持的通配符：
- `*` - 匹配任意字符
- `?` - 匹配单个字符
- `[abc]` - 字符集

## 权限合并

```rust
pub enum MergeStrategy {
    MostRestrictive,  // 最严格
    LeastRestrictive, // 最宽松
    Override,         // 覆盖
    Combine,          // 组合
}

pub fn merge_permissions(
    permissions: &[ToolPermission],
    strategy: MergeStrategy,
) -> ToolPermission;
```

## 权限检查流程

```
工具调用
    │
    ▼
┌─────────────────────────┐
│  模式匹配 (tool_pattern) │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│  参数限制检查            │
│  (ParameterRestriction) │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│  条件评估                │
│  (PermissionCondition)  │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│  返回结果 + 审计日志     │
└─────────────────────────┘
```

## 权限结果

```rust
pub enum PermissionResult {
    Allowed,
    Denied { reason: String },
    Ask { prompt: String },
    Expired,
}
```
