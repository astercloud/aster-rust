//! Skill Types
//!
//! Core types for the skills system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Skill source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    /// User-level skill (~/.claude/skills/)
    User,
    /// Project-level skill (.claude/skills/)
    Project,
    /// Plugin-provided skill
    Plugin,
}

impl std::fmt::Display for SkillSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillSource::User => write!(f, "user"),
            SkillSource::Project => write!(f, "project"),
            SkillSource::Plugin => write!(f, "plugin"),
        }
    }
}

/// Skill frontmatter metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    /// Skill name (optional, defaults to directory name)
    pub name: Option<String>,
    /// Skill description
    pub description: Option<String>,
    /// Allowed tools (comma-separated or array)
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<String>,
    /// Argument hint for the skill
    #[serde(rename = "argument-hint")]
    pub argument_hint: Option<String>,
    /// When to use this skill
    #[serde(rename = "when-to-use", alias = "when_to_use")]
    pub when_to_use: Option<String>,
    /// Skill version
    pub version: Option<String>,
    /// Preferred model for this skill
    pub model: Option<String>,
    /// Whether the skill is user-invocable (default: true)
    #[serde(rename = "user-invocable")]
    pub user_invocable: Option<String>,
    /// Whether to disable model invocation (default: false)
    #[serde(rename = "disable-model-invocation")]
    pub disable_model_invocation: Option<String>,
}

/// Skill definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Skill name (with namespace, e.g., "user:my-skill")
    pub skill_name: String,
    /// Display name
    pub display_name: String,
    /// Description
    pub description: String,
    /// Whether description was specified in frontmatter
    pub has_user_specified_description: bool,
    /// Markdown content (body after frontmatter)
    pub markdown_content: String,
    /// Allowed tools list
    pub allowed_tools: Option<Vec<String>>,
    /// Argument hint
    pub argument_hint: Option<String>,
    /// When to use hint
    pub when_to_use: Option<String>,
    /// Version
    pub version: Option<String>,
    /// Preferred model
    pub model: Option<String>,
    /// Whether model invocation is disabled
    pub disable_model_invocation: bool,
    /// Whether user can invoke this skill
    pub user_invocable: bool,
    /// Source of the skill
    pub source: SkillSource,
    /// Base directory of the skill
    pub base_dir: PathBuf,
    /// File path of SKILL.md
    pub file_path: PathBuf,
    /// Supporting files in the skill directory
    pub supporting_files: Vec<PathBuf>,
}

impl SkillDefinition {
    /// Get the short name (without namespace)
    pub fn short_name(&self) -> &str {
        self.skill_name
            .rsplit(':')
            .next()
            .unwrap_or(&self.skill_name)
    }

    /// Get the namespace
    pub fn namespace(&self) -> Option<&str> {
        let parts: Vec<&str> = self.skill_name.split(':').collect();
        if parts.len() > 1 {
            Some(parts[0])
        } else {
            None
        }
    }
}

/// Invoked skill record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokedSkill {
    /// Skill name
    pub skill_name: String,
    /// Skill file path
    pub skill_path: PathBuf,
    /// Skill content that was invoked
    pub content: String,
    /// Timestamp when invoked
    pub invoked_at: u64,
}

/// Skill execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Output message
    pub output: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Command/skill name
    pub command_name: Option<String>,
    /// Allowed tools for this skill
    pub allowed_tools: Option<Vec<String>>,
    /// Preferred model
    pub model: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_source_display() {
        assert_eq!(SkillSource::User.to_string(), "user");
        assert_eq!(SkillSource::Project.to_string(), "project");
        assert_eq!(SkillSource::Plugin.to_string(), "plugin");
    }

    #[test]
    fn test_skill_definition_short_name() {
        let skill = SkillDefinition {
            skill_name: "user:my-skill".to_string(),
            display_name: "My Skill".to_string(),
            description: "Test".to_string(),
            has_user_specified_description: true,
            markdown_content: "# Content".to_string(),
            allowed_tools: None,
            argument_hint: None,
            when_to_use: None,
            version: None,
            model: None,
            disable_model_invocation: false,
            user_invocable: true,
            source: SkillSource::User,
            base_dir: PathBuf::from("/test"),
            file_path: PathBuf::from("/test/SKILL.md"),
            supporting_files: vec![],
        };

        assert_eq!(skill.short_name(), "my-skill");
        assert_eq!(skill.namespace(), Some("user"));
    }

    #[test]
    fn test_skill_definition_no_namespace() {
        let skill = SkillDefinition {
            skill_name: "simple-skill".to_string(),
            display_name: "Simple".to_string(),
            description: "Test".to_string(),
            has_user_specified_description: false,
            markdown_content: "".to_string(),
            allowed_tools: None,
            argument_hint: None,
            when_to_use: None,
            version: None,
            model: None,
            disable_model_invocation: false,
            user_invocable: true,
            source: SkillSource::Project,
            base_dir: PathBuf::from("/test"),
            file_path: PathBuf::from("/test/SKILL.md"),
            supporting_files: vec![],
        };

        assert_eq!(skill.short_name(), "simple-skill");
        assert_eq!(skill.namespace(), None);
    }

    #[test]
    fn test_skill_source_serialization() {
        let source = SkillSource::User;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"user\"");

        let deserialized: SkillSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SkillSource::User);
    }
}
