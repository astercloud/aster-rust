# Parser 代码解析

基于 LSP 的代码解析模块。

## 模块结构

```
parser/
├── lsp_client.rs       # LSP 客户端
├── lsp_manager.rs      # LSP 管理器
├── symbol_extractor.rs # 符号提取
└── types.rs            # 类型定义
```

## 功能

- LSP 客户端管理
- 符号提取（函数、类、方法等）
- 引用查找
- 跳转到定义
- 代码折叠区域检测

## 核心类型

### LspClient
```rust
pub struct LspClient {
    config: LspClientConfig,
    state: LspServerState,
}
```

### LspManager
```rust
pub struct LspManager;
pub static LSP_SERVERS: &[LspServerInfo];
```


### 符号提取
```rust
pub struct LspSymbolExtractor;

pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
}

pub enum SymbolKind {
    Function,
    Class,
    Method,
    Variable,
    // ...
}

pub struct Reference {
    pub file: PathBuf,
    pub range: Range,
}
```

## 使用场景

- 代码导航
- 符号搜索
- 重构支持

## 源码位置

`crates/aster/src/parser/`
