# Recipe 系统

## 概述

Recipe 是预定义的 Agent 配置，包含指令、扩展、参数等，用于快速启动特定任务。

**核心路径**: `crates/aster/src/recipe/`

## 模块结构

| 模块 | 说明 |
|------|------|
| `mod.rs` | Recipe 核心定义 |
| `build_recipe.rs` | Recipe 构建 |
| `template_recipe.rs` | 模板渲染 |
| `validate_recipe.rs` | 验证逻辑 |
| `local_recipes.rs` | 本地 Recipe 管理 |
| `yaml_format_utils.rs` | YAML 格式化 |

## Recipe 结构

```rust
pub struct Recipe {
    // 必填字段
    pub version: String,
    pub title: String,
    pub description: String,
    
    // 可选字段 (至少需要 instructions 或 prompt)
    pub instructions: Option<String>,
    pub prompt: Option<String>,
    
    // 扩展配置
    pub extensions: Option<Vec<ExtensionConfig>>,
    
    // 设置
    pub settings: Option<Settings>,
    
    // 活动提示
    pub activities: Option<Vec<String>>,
    
    // 作者信息
    pub author: Option<Author>,
    
    // 参数定义
    pub parameters: Option<Vec<RecipeParameter>>,
    
    // 响应配置
    pub response: Option<Response>,
    
    // 子 Recipe
    pub sub_recipes: Option<Vec<SubRecipe>>,
    
    // 重试配置
    pub retry: Option<RetryConfig>,
}
```

## Recipe 参数

```rust
pub struct RecipeParameter {
    pub key: String,
    pub input_type: RecipeParameterInputType,
    pub requirement: RecipeParameterRequirement,
    pub description: String,
    pub default: Option<String>,
    pub options: Option<Vec<String>>,
}

pub enum RecipeParameterInputType {
    String,
    Number,
    Boolean,
    Date,
    File,    // 文件导入
    Select,  // 选择列表
}

pub enum RecipeParameterRequirement {
    Required,
    Optional,
    UserPrompt,  // 运行时提示用户
}
```

## Recipe 文件示例

```yaml
version: "1.0.0"
title: Code Review
description: 执行代码审查

instructions: |
  你是一个代码审查专家。
  请仔细审查提供的代码，关注：
  - 代码质量
  - 潜在 Bug
  - 性能问题
  - 安全漏洞

prompt: |
  请审查以下文件: {{file_path}}
  重点关注: {{focus}}

parameters:
  - key: file_path
    input_type: file
    requirement: required
    description: 要审查的文件

  - key: focus
    input_type: select
    requirement: optional
    description: 审查重点
    default: all
    options:
      - all
      - security
      - performance
      - style

extensions:
  - type: stdio
    name: developer
    cmd: aster
    args: [mcp, developer]

settings:
  aster_model: claude-3-5-sonnet

activities:
  - 分析代码结构
  - 检查潜在问题
  - 生成审查报告

author:
  contact: team@example.com
```

## 子 Recipe

```rust
pub struct SubRecipe {
    pub name: String,
    pub path: String,
    pub values: Option<HashMap<String, String>>,
    pub sequential_when_repeated: bool,
    pub description: Option<String>,
}
```

```yaml
sub_recipes:
  - name: lint
    path: ./lint.yaml
    values:
      strict: "true"
  
  - name: test
    path: ./test.yaml
    sequential_when_repeated: true
```

## 响应配置

```rust
pub struct Response {
    pub json_schema: Option<Value>,
}
```

```yaml
response:
  json_schema:
    type: object
    properties:
      summary:
        type: string
      issues:
        type: array
        items:
          type: object
          properties:
            severity: { type: string }
            message: { type: string }
    required: [summary]
```

## Recipe 加载

```rust
// 从文件加载
let recipe = Recipe::from_file_path(Path::new("recipe.yaml"))?;

// 从内容加载
let recipe = Recipe::from_content(yaml_content)?;

// 使用 Builder
let recipe = Recipe::builder()
    .title("My Recipe")
    .description("Description")
    .instructions("Do something")
    .build()?;
```

## 模板渲染

Recipe 支持 `{{parameter}}` 模板语法：

```rust
// template_recipe.rs
pub fn render_recipe(
    recipe: &Recipe,
    params: &HashMap<String, String>,
) -> Result<Recipe>;
```

## 安全检查

```rust
impl Recipe {
    pub fn check_for_security_warnings(&self) -> bool {
        // 检查 Unicode 标签注入等安全问题
    }
}
```

## CLI 使用

```bash
# 运行 Recipe
aster run --recipe code-review --params file_path=src/main.rs

# 查看 Recipe 详情
aster run --recipe code-review --explain

# 渲染 Recipe (不执行)
aster run --recipe code-review --render-recipe

# 验证 Recipe
aster recipe validate my-recipe.yaml

# 生成 Deeplink
aster recipe deeplink my-recipe -p key=value

# 列出可用 Recipe
aster recipe list
```
