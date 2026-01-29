# CLI 使用指南

## 概述

Aster CLI 提供命令行交互界面，支持会话管理、任务执行、定时调度等功能。

**入口**: `crates/aster-cli/src/main.rs`

## 主要命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `session` | `s` | 会话管理 |
| `run` | - | 执行任务 |
| `project` | `p` | 打开项目 |
| `projects` | `ps` | 列出项目 |
| `recipe` | - | Recipe 工具 |
| `schedule` | `sched` | 定时任务 |
| `configure` | - | 配置设置 |
| `info` | - | 显示信息 |
| `mcp` | - | MCP 服务器 |
| `bench` | - | 基准测试 |
| `term` | - | 终端集成 |
| `update` | - | 更新 CLI |

## 会话命令

```bash
# 启动新会话
aster session

# 恢复上次会话
aster session --resume

# 指定会话名称
aster session --name my-project

# 恢复指定会话
aster session --resume --name my-project

# 列出会话
aster session list
aster session list --format json
aster session list --limit 10

# 删除会话
aster session remove --name old-session
aster session remove --regex "test-.*"

# 导出会话
aster session export --name my-session --output chat.md
aster session export --format json

# 诊断信息
aster session diagnostics --name my-session
```

## Run 命令

```bash
# 从文件执行
aster run --instructions task.txt

# 从 stdin 执行
echo "list files" | aster run --instructions -

# 直接文本输入
aster run --text "create a hello world script"

# 使用 Recipe
aster run --recipe code-review
aster run --recipe ./my-recipe.yaml

# 带参数
aster run --recipe deploy --params env=prod --params dry_run=true

# 查看 Recipe 详情
aster run --recipe code-review --explain

# 渲染 Recipe (不执行)
aster run --recipe deploy --render-recipe

# 静默模式
aster run --text "task" --quiet

# JSON 输出
aster run --text "task" --output-format json

# 指定模型
aster run --text "task" --provider anthropic --model claude-3-5-sonnet

# 交互模式
aster run --text "initial task" --interactive

# 无会话模式
aster run --text "quick task" --no-session
```

## 扩展选项

```bash
# 添加 stdio 扩展
aster session --with-extension "npx @mcp/server-filesystem /path"

# 添加 HTTP 扩展
aster session --with-streamable-http-extension "http://localhost:8080"

# 添加内置扩展
aster session --with-builtin developer
aster session --with-builtin developer,github
```

## Recipe 命令

```bash
# 验证 Recipe
aster recipe validate my-recipe.yaml

# 生成 Deeplink
aster recipe deeplink my-recipe -p key=value

# 在桌面应用打开
aster recipe open my-recipe

# 列出可用 Recipe
aster recipe list
aster recipe list --format json --verbose
```

## 定时任务

```bash
# 添加定时任务
aster schedule add \
  --schedule-id daily-report \
  --cron "0 9 * * *" \
  --recipe-source ./report.yaml

# 列出任务
aster schedule list

# 立即执行
aster schedule run-now --schedule-id daily-report

# 查看任务会话
aster schedule sessions --schedule-id daily-report

# 删除任务
aster schedule remove --schedule-id daily-report

# Cron 帮助
aster schedule cron-help
```

## 终端集成

```bash
# 初始化 (添加到 ~/.zshrc)
eval "$(aster term init zsh)"

# 运行命令
aster term run "list files"

# 使用别名 (初始化后)
@aster "create a script"
@g "quick question"

# 查看终端会话信息
aster term info

# 查看日志
aster term log
```

## MCP 服务器

```bash
# 运行内置 MCP 服务器
aster mcp developer
aster mcp memory
aster mcp tutorial
aster mcp computer-controller
aster mcp auto-visualiser
```

## 其他命令

```bash
# 配置
aster configure

# 显示信息
aster info
aster info --verbose

# 更新
aster update
aster update --canary

# Shell 补全
aster completion zsh > _aster
aster completion bash > aster.bash
```

## 会话选项

```bash
# 调试模式
aster session --debug

# 限制工具重复
aster session --max-tool-repetitions 5

# 限制轮次
aster session --max-turns 100
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `ASTER_PROVIDER` | 默认 Provider |
| `ASTER_MODEL` | 默认模型 |
| `ASTER_MODE` | 运行模式 |
| `ASTER_RECIPE_GITHUB_REPO` | Recipe GitHub 仓库 |
