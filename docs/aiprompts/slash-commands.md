# Slash Commands

斜杠命令系统，将命令映射到 Recipe。

## 核心类型

```rust
pub struct SlashCommandMapping {
    pub command: String,
    pub recipe_path: String,
}
```

## 主要函数

```rust
pub fn list_commands() -> Vec<SlashCommandMapping>;

pub fn set_recipe_slash_command(
    recipe_path: PathBuf,
    command: Option<String>
) -> Result<()>;

pub fn get_recipe_for_command(command: &str) -> Option<PathBuf>;

pub fn resolve_slash_command(command: &str) -> Option<Recipe>;
```

## 使用示例

```
/review  -> 执行代码审查 Recipe
/test    -> 执行测试 Recipe
/deploy  -> 执行部署 Recipe
```


## 命令规范化

- 自动去除前导 `/`
- 转换为小写
- 空命令会删除映射

## 配置存储

存储在全局配置的 `slash_commands` 键下。

## 使用场景

- 快速执行常用 Recipe
- 自定义工作流快捷方式
- 团队共享命令

## 源码位置

`crates/aster/src/slash_commands.rs`
