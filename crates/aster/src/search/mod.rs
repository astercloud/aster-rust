//! 代码搜索模块
//!
//! 提供 ripgrep 集成的代码搜索功能

mod ripgrep;

pub use ripgrep::{
    RipgrepOptions, RipgrepMatch, RipgrepResult,
    get_rg_path, get_system_rg_path, get_vendored_rg_path,
    is_ripgrep_available, get_ripgrep_version,
    search, search_sync, list_files,
    download_vendored_rg, ensure_ripgrep_available,
    RG_VERSION,
};
