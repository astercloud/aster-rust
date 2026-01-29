# 专用 Agent

## 概述

专用 Agent 提供特定任务的优化实现，包括 Explore Agent 和 Plan Agent。

**核心路径**: `crates/aster/src/agents/specialized/`

## Explore Agent

用于代码库探索和分析。

### ExploreOptions

```rust
pub struct ExploreOptions {
    pub max_depth: usize,           // 最大探索深度
    pub include_tests: bool,        // 包含测试文件
    pub include_docs: bool,         // 包含文档
    pub file_patterns: Vec<String>, // 文件模式
    pub exclude_patterns: Vec<String>,
    pub thoroughness: ThoroughnessLevel,
}

pub enum ThoroughnessLevel {
    Quick,      // 快速扫描
    Normal,     // 正常分析
    Thorough,   // 深入分析
}
```

### ExploreResult

```rust
pub struct ExploreResult {
    pub success: bool,
    pub data: ExploreResultData,
    pub stats: ExploreStats,
    pub duration: Duration,
}

pub struct ExploreResultData {
    pub structure: StructureAnalysis,
    pub critical_files: Vec<CriticalFile>,
    pub code_snippets: Vec<CodeSnippet>,
    pub summary: String,
}

pub struct StructureAnalysis {
    pub directories: Vec<String>,
    pub file_count: usize,
    pub language_breakdown: HashMap<String, usize>,
    pub entry_points: Vec<String>,
}

pub struct CriticalFile {
    pub path: PathBuf,
    pub importance: f32,
    pub reason: String,
    pub symbols: Vec<SymbolInfo>,
}

pub struct CodeSnippet {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub relevance: f32,
}
```

### ExploreAgent

```rust
pub struct ExploreAgent {
    options: ExploreOptions,
}

impl ExploreAgent {
    pub fn new(options: ExploreOptions) -> Self;
    
    pub async fn explore(&self, query: &str, working_dir: &Path) 
        -> ExploreResult<ExploreResult>;
}
```

## Plan Agent

用于任务规划和方案设计。

### PlanOptions

```rust
pub struct PlanOptions {
    pub max_steps: usize,
    pub include_alternatives: bool,
    pub risk_analysis: bool,
    pub complexity_threshold: Complexity,
}

pub enum Complexity {
    Simple,
    Medium,
    Complex,
}
```

### PlanResult

```rust
pub struct PlanResult {
    pub success: bool,
    pub data: PlanResultData,
    pub duration: Duration,
}

pub struct PlanResultData {
    pub steps: Vec<PlanStep>,
    pub requirements: RequirementsAnalysis,
    pub scope: ScopeDefinition,
    pub risks: Vec<Risk>,
    pub alternatives: Vec<Alternative>,
    pub decisions: Vec<ArchitecturalDecision>,
}

pub struct PlanStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub estimated_effort: Option<String>,
    pub files_affected: Vec<String>,
    pub modification_type: ModificationType,
}

pub enum ModificationType {
    Create,
    Modify,
    Delete,
    Refactor,
}

pub struct Risk {
    pub id: String,
    pub description: String,
    pub severity: RiskSeverity,
    pub category: RiskCategory,
    pub mitigation: Option<String>,
}

pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub enum RiskCategory {
    Technical,
    Security,
    Performance,
    Compatibility,
    Maintenance,
}
```

### PlanAgent

```rust
pub struct PlanAgent {
    options: PlanOptions,
}

impl PlanAgent {
    pub fn new(options: PlanOptions) -> Self;
    
    pub async fn plan(&self, task: &str, context: &AgentContext) 
        -> PlanResult<PlanResult>;
}
```

## 使用示例

### Explore Agent

```rust
let options = ExploreOptions {
    max_depth: 5,
    include_tests: false,
    thoroughness: ThoroughnessLevel::Normal,
    ..Default::default()
};

let agent = ExploreAgent::new(options);
let result = agent.explore(
    "找到用户认证相关的代码",
    Path::new("/project")
).await?;

for file in result.data.critical_files {
    println!("{}: {}", file.path.display(), file.reason);
}
```

### Plan Agent

```rust
let options = PlanOptions {
    max_steps: 10,
    include_alternatives: true,
    risk_analysis: true,
    complexity_threshold: Complexity::Medium,
};

let agent = PlanAgent::new(options);
let result = agent.plan(
    "添加用户注册功能",
    &context
).await?;

for step in result.data.steps {
    println!("{}. {}", step.id, step.title);
}
```
