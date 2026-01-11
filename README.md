# Aster

Aster 是一个 Rust AI Agent 框架，基于 goose 项目 fork 并改造。

## 特性

- MCP (Model Context Protocol) 支持
- Skills 系统
- 多 Provider 支持
- CLI 和 Electron 桌面界面

## 结构

```
crates/
├── aster             # 核心逻辑
├── aster-bench       # 基准测试
├── aster-cli         # CLI 入口
├── aster-server      # 后端 (binary: asterd)
├── aster-mcp         # MCP 扩展
└── aster-test        # 测试工具

ui/desktop/           # Electron 桌面应用
```

## 构建

```bash
cargo build
cargo build --release
```

## 运行

```bash
# CLI
cargo run -p aster-cli

# Server
cargo run -p aster-server
```

## License

Apache-2.0
