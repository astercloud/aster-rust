# Hints 提示文件系统

加载项目提示文件（AGENTS.md, .asterhints 等）。

## 模块结构

```
hints/
├── import_files.rs  # 文件导入
└── load_hints.rs    # 提示加载
```

## 支持的文件

- `AGENTS.md` - 项目 Agent 指令
- `.asterhints` - Aster 特定提示
- `.goosehints` - 兼容 Goose 格式

## 核心 API

```rust
pub const AGENTS_MD_FILENAME: &str = "AGENTS.md";
pub const ASTER_HINTS_FILENAME: &str = ".asterhints";

pub fn load_hint_files(path: &Path) -> Vec<HintFile>;
```

## 使用场景

- 启动时加载项目规则
- 为 Agent 提供上下文
- 自定义行为指令

## 源码位置

`crates/aster/src/hints/`
