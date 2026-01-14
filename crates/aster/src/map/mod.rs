//! 代码本体图谱模块
//!

pub mod types;
pub mod types_enhanced;
pub mod types_chunked;
pub mod analyzer;
pub mod dependency_analyzer;
pub mod call_graph_builder;
pub mod incremental_cache;
pub mod layer_classifier;
pub mod view_builder;
pub mod ontology_generator;
pub mod enhanced_generator;
pub mod chunked_generator;
pub mod incremental_updater;
pub mod sync_manager;
pub mod symbol_reference_analyzer;
pub mod type_reference_analyzer;
pub mod semantic_generator;
pub mod server;

#[cfg(test)]
mod tests;

// 基础类型
pub use types::*;

// 增强类型
pub use types_enhanced::*;

// 分块类型
pub use types_chunked::*;

// 分析器
pub use analyzer::{CodeMapAnalyzer, create_analyzer};

// 依赖分析
pub use dependency_analyzer::{DependencyAnalyzer, analyze_dependencies, DependencyStats};

// 调用图
pub use call_graph_builder::{CallGraphBuilder, build_call_graph};

// 增量缓存
pub use incremental_cache::{IncrementalCache, create_cache, CacheStats, FileCheckResult};

// 架构层分类
pub use layer_classifier::{LayerClassifier, ClassificationResult, classify_module, classify_modules};

// 视图构建
pub use view_builder::{ViewBuilder, build_views, build_directory_tree, build_architecture_layers, count_tree_nodes, get_tree_depth};

// 本体生成
pub use ontology_generator::{OntologyGenerator, generate_ontology, generate_and_save_ontology};

// 增强版生成
pub use enhanced_generator::{EnhancedOntologyGenerator, generate_enhanced_blueprint, generate_and_save_enhanced_blueprint};

// 分块生成
pub use chunked_generator::ChunkedBlueprintGenerator;

// 增量更新
pub use incremental_updater::{IncrementalBlueprintUpdater, UpdateOptions, UpdateResult, update_blueprint};

// 双向同步
pub use sync_manager::{
    BlueprintCodeSyncManager, SyncOptions, SyncResult, Conflict, ConflictType,
    ConflictResolution, CodeGenerationResult, sync_code_to_blueprint, sync_blueprint_to_code,
};

// 符号引用分析
pub use symbol_reference_analyzer::{
    SymbolReferenceAnalyzer, SymbolReferenceResult, CallType, analyze_symbol_references,
};

// 类型引用分析
pub use type_reference_analyzer::{
    TypeReferenceAnalyzer, TypeUsageAnalyzer, TypeUsage, TypeUsageKind, TypeUsageLocation,
    analyze_type_references, analyze_type_usages,
};

// AI 语义生成
pub use semantic_generator::{
    SemanticGenerator, SemanticGeneratorOptions, generate_module_semantic,
    batch_generate_semantics, generate_project_semantic,
};

// 可视化服务器
pub use server::{
    VisualizationServer, VisualizationServerOptions, start_visualization_server,
    // 服务器类型
    ModuleDetailInfo, ModuleSymbols, SymbolInfo, SymbolLocation,
    SymbolRefInfo, CallerInfo, TypeRefInfo, LineLocation,
    DependencyTreeNode, LogicBlock, LogicBlockType, ArchitectureMap,
    FlowchartNode, FlowchartNodeType, FlowchartEdge, FlowchartEdgeType, Flowchart,
    ScenarioInfo, BeginnerGuide, GuideCard, GuideCardFile, FileImportance,
    StoryGuide, BusinessStory, StoryChapter, StoryKeyFile, CodeSnippet,
    CodeReadingGuide, ReadingPath, ReadingStep, ReadingDifficulty, LineRange,
    KnowledgeSnapshot, KnowledgeSnapshotSummary,
    EntryPointsResponse, SearchResultItem, SearchResponse,
};
