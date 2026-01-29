# Map 代码本体图谱

代码库分析和可视化系统。

## 模块结构

```
map/
├── analyzer.rs              # 代码分析器
├── call_graph_builder.rs    # 调用图构建
├── dependency_analyzer.rs   # 依赖分析
├── layer_classifier.rs      # 架构层分类
├── ontology_generator.rs    # 本体生成
├── semantic_generator.rs    # AI 语义生成
├── server.rs                # 可视化服务器
├── view_builder.rs          # 视图构建
└── ...
```

## 核心功能

### CodeMapAnalyzer
```rust
pub fn create_analyzer() -> CodeMapAnalyzer;
```

### 依赖分析
```rust
pub fn analyze_dependencies(path: &Path) -> DependencyStats;
```


### 调用图
```rust
pub fn build_call_graph(path: &Path) -> CallGraph;
```

### 架构层分类
```rust
pub fn classify_modules(modules: &[Module]) -> Vec<ClassificationResult>;
```

### 本体生成
```rust
pub fn generate_ontology(path: &Path) -> Ontology;
pub fn generate_enhanced_blueprint(path: &Path) -> EnhancedBlueprint;
```

## 可视化服务器

```rust
pub fn start_visualization_server(options: VisualizationServerOptions);
```

提供：
- 模块详情
- 搜索功能
- 入口点分析
- 代码阅读指南

## 使用场景

- 代码库理解
- 架构可视化
- 依赖分析

## 源码位置

`crates/aster/src/map/`
