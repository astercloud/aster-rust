# 代码搜索

提供 ripgrep 集成的代码搜索功能。

## 模块结构

```
search/
└── ripgrep.rs  # Ripgrep 集成
```

## 核心功能

### 搜索函数
```rust
pub async fn search(
    pattern: &str,
    options: RipgrepOptions
) -> RipgrepResult;

pub fn search_sync(
    pattern: &str,
    options: RipgrepOptions
) -> RipgrepResult;

pub async fn list_files(
    path: &Path,
    options: RipgrepOptions
) -> Vec<PathBuf>;
```

### RipgrepOptions
```rust
pub struct RipgrepOptions {
    pub path: PathBuf,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub max_results: Option<usize>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}
```


### RipgrepMatch
```rust
pub struct RipgrepMatch {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub text: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}
```

## Ripgrep 管理

```rust
pub fn ensure_ripgrep_available() -> Result<PathBuf>;
pub fn get_rg_path() -> Option<PathBuf>;
pub fn download_vendored_rg() -> Result<PathBuf>;
pub fn is_ripgrep_available() -> bool;
```

## 使用场景

- Agent 搜索代码库
- 查找符号定义
- 批量替换

## 源码位置

`crates/aster/src/search/`
