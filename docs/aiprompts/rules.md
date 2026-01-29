# Rules 规则系统

管理项目规则和 AGENTS.md 文件。

## 模块结构

```
rules/
├── applier.rs  # 规则应用
├── parser.rs   # 规则解析
└── types.rs    # 类型定义
```

## AGENTS.md

项目根目录的 `AGENTS.md` 文件定义 Agent 行为规则：

```markdown
# AGENTS Instructions

项目特定的指令...

## 结构
代码结构说明...

## 规则
- 规则 1
- 规则 2
```

## 核心 API

### 解析
```rust
pub fn find_agents_md(path: &Path) -> Option<PathBuf>;
pub fn parse_agents_md(content: &str) -> ProjectRules;
pub fn load_project_rules(path: &Path) -> ProjectRules;
```


### 应用
```rust
pub fn apply_rules(rules: &ProjectRules) -> RuleApplyResult;
pub fn generate_system_prompt_addition(rules: &ProjectRules) -> String;
pub fn init_agents_md(path: &Path) -> Result<()>;
```

## 类型定义

```rust
pub struct ProjectRules {
    pub sections: Vec<AgentsMdSection>,
    pub custom_rules: Vec<CustomRule>,
}

pub struct CustomRule {
    pub pattern: String,
    pub action: RuleAction,
    pub description: String,
}

pub enum RuleAction {
    Allow,
    Deny,
    Warn,
    Transform(String),
}
```

## 使用场景

- 定义项目编码规范
- 限制 Agent 行为
- 自定义提示词

## 源码位置

`crates/aster/src/rules/`
