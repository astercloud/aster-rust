//! Skills System
//!
//! Provides skill discovery, loading, and execution functionality.
//! Skills are reusable prompts/workflows stored in SKILL.md files.
//!
//! Directory structure:
//! - `~/.claude/skills/` - User-level skills
//! - `.claude/skills/` - Project-level skills
//! - Plugin cache - Plugin-provided skills

mod types;
mod loader;
mod registry;
mod tool;

pub use types::*;
pub use loader::*;
pub use registry::*;
pub use tool::*;
