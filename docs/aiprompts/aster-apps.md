# Aster Apps

UI 资源模块，用于 MCP 服务器或原生应用渲染。

## 模块结构

```
aster_apps/
└── resource.rs  # 资源定义
```

## 核心类型

```rust
pub struct McpAppResource {
    pub metadata: ResourceMetadata,
    pub ui: UiMetadata,
    pub csp: CspMetadata,
}

pub struct ResourceMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
}

pub struct UiMetadata {
    pub entry_point: String,
    pub assets: Vec<String>,
}

pub struct CspMetadata {
    pub directives: HashMap<String, String>,
}
```

## 使用场景

- MCP 服务器 UI 资源
- 原生应用集成
- 混合应用开发

## 源码位置

`crates/aster/src/aster_apps/`
