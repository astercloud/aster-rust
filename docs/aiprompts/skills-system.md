# 技能系统

## 概述

技能系统提供可复用的 Prompt/工作流，存储在 SKILL.md 文件中。

**核心路径**: `crates/aster/src/skills/`

## 目录结构

```
~/.claude/skills/          # 用户级技能
.claude/skills/            # 项目级技能
<plugin-cache>/skills/     # 插件提供的技能
```

## 模块结构

| 模块 | 说明 |
|------|------|
| `types.rs` | 技能类型定义 |
| `loader.rs` | 技能加载器 |
| `registry.rs` | 技能注册表 |
| `tool.rs` | SkillTool 实现 |

## Skill 结构

```rust
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub parameters: Vec<SkillParameter>,
    pub source: SkillSource,
}

pub struct SkillParameter {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
}

pub enum SkillSource {
    User,      // ~/.claude/skills/
    Project,   // .claude/skills/
    Plugin,    // 插件提供
}
```

## SKILL.md 格式

```markdown
---
name: code-review
description: 执行代码审查
parameters:
  - name: file_path
    description: 要审查的文件路径
    required: true
  - name: focus
    description: 审查重点
    default: "all"
---

# Code Review Skill

请对以下文件进行代码审查：

文件: {{file_path}}
重点: {{focus}}

## 审查要点

1. 代码风格和可读性
2. 潜在的 Bug
3. 性能问题
4. 安全漏洞
5. 测试覆盖
```

## SkillTool

```rust
// crates/aster/src/skills/tool.rs
pub struct SkillTool {
    registry: Arc<SkillRegistry>,
}

impl Tool for SkillTool {
    fn name(&self) -> &str { "Skill" }
    
    fn description(&self) -> &str {
        "Execute a skill by name with parameters"
    }
    
    async fn execute(
        &self,
        input: Value,
        context: &ToolContext,
    ) -> ToolResult<Value>;
}
```

## 技能加载

```rust
// crates/aster/src/skills/loader.rs
pub struct SkillLoader;

impl SkillLoader {
    // 从目录加载所有技能
    pub async fn load_from_directory(
        path: &Path
    ) -> Result<Vec<Skill>>;
    
    // 加载单个技能文件
    pub async fn load_skill(path: &Path) -> Result<Skill>;
    
    // 解析 SKILL.md 内容
    pub fn parse_skill_content(content: &str) -> Result<Skill>;
}
```

## 技能注册表

```rust
// crates/aster/src/skills/registry.rs
pub struct SkillRegistry {
    skills: RwLock<HashMap<String, Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self;
    
    // 注册技能
    pub async fn register(&self, skill: Skill);
    
    // 获取技能
    pub async fn get(&self, name: &str) -> Option<Skill>;
    
    // 列出所有技能
    pub async fn list(&self) -> Vec<Skill>;
    
    // 按来源过滤
    pub async fn list_by_source(
        &self, 
        source: SkillSource
    ) -> Vec<Skill>;
    
    // 刷新技能
    pub async fn refresh(&self) -> Result<()>;
}
```

## 使用示例

### CLI 调用

```bash
# 列出可用技能
aster skill list

# 执行技能
aster skill run code-review --file_path src/main.rs

# 带参数执行
aster skill run deploy --env production --dry-run true
```

### Agent 调用

```json
{
  "tool": "Skill",
  "input": {
    "name": "code-review",
    "parameters": {
      "file_path": "src/main.rs",
      "focus": "security"
    }
  }
}
```

## 技能发现优先级

```
1. 项目级 (.claude/skills/)     # 最高优先级
2. 用户级 (~/.claude/skills/)
3. 插件提供                      # 最低优先级
```

同名技能按优先级覆盖。
