# AGENTS Instructions

Aster 是一个 Rust AI Agent 框架，支持 CLI 和 Electron 桌面界面。

文件超过 20 行，就要分批进行输出

## 结构
```
crates/
├── aster             # 核心逻辑
├── aster-bench       # 基准测试
├── aster-cli         # CLI 入口
├── aster-server      # 后端 (binary: asterd)
├── aster-mcp         # MCP 扩展
└── aster-test        # 测试工具
```

## 入口点
- CLI: crates/aster-cli/src/main.rs
- Server: crates/aster-server/src/main.rs
- Agent: crates/aster/src/agents/agent.rs
