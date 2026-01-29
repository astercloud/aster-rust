# LSP 服务器管理

提供 Language Server Protocol 服务器管理功能。

## 模块结构

```
lsp/
├── config.rs   # LSP 配置
├── manager.rs  # 服务器管理器
└── server.rs   # 服务器实例
```

## 核心类型

### LSPServerManager
```rust
pub struct LSPServerManager {
    servers: HashMap<String, LSPServer>,
}

impl LSPServerManager {
    pub fn initialize(options: InitializeLSPOptions);
    pub fn get_diagnostics(file: &Path) -> Vec<LSPDiagnostic>;
    pub fn shutdown();
}
```

### LSPServerConfig
```rust
pub struct LSPServerConfig {
    pub language: String,
    pub command: String,
    pub args: Vec<String>,
    pub root_patterns: Vec<String>,
}
```


### LSPDiagnostic
```rust
pub struct LSPDiagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: DiagnosticSeverity,
}
```

## 支持的语言服务器

通过 `default_lsp_configs()` 获取默认配置：
- TypeScript/JavaScript (tsserver)
- Rust (rust-analyzer)
- Python (pyright)
- Go (gopls)

## 使用场景

- Agent 编辑代码后获取诊断信息
- 自动修复 lint 错误
- 代码补全建议

## 源码位置

`crates/aster/src/lsp/`
