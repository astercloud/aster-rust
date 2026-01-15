//! Skill Loader
//!
//! Handles parsing and loading skills from SKILL.md files.

use super::types::{SkillDefinition, SkillFrontmatter, SkillSource};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Parse frontmatter from skill content
///
/// ```text
/// function NV(A) {
///   let Q = /^---\s*\n([\s\S]*?)---\s*\n?/;
///   let B = A.match(Q);
///   if (!B) return { frontmatter: {}, content: A };
///   ...
/// }
/// ```
pub fn parse_frontmatter(content: &str) -> (SkillFrontmatter, String) {
    // Match frontmatter block: ---\n...\n---
    let regex = regex::Regex::new(r"^---\s*\n([\s\S]*?)---\s*\n?").unwrap();

    if let Some(captures) = regex.captures(content) {
        let frontmatter_text = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let body_start = captures.get(0).map(|m| m.end()).unwrap_or(0);
        let body = content.get(body_start..).unwrap_or("").to_string();

        // Parse YAML-like frontmatter
        let mut frontmatter = SkillFrontmatter::default();
        let mut extra_fields: HashMap<String, String> = HashMap::new();

        for line in frontmatter_text.lines() {
            if let Some(colon_idx) = line.find(':') {
                let key = line.get(..colon_idx).unwrap_or("").trim();
                let value = line.get(colon_idx + 1..).unwrap_or("").trim();
                // Remove surrounding quotes
                let clean_value = value
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();

                match key {
                    "name" => frontmatter.name = Some(clean_value),
                    "description" => frontmatter.description = Some(clean_value),
                    "allowed-tools" => frontmatter.allowed_tools = Some(clean_value),
                    "argument-hint" => frontmatter.argument_hint = Some(clean_value),
                    "when-to-use" | "when_to_use" => frontmatter.when_to_use = Some(clean_value),
                    "version" => frontmatter.version = Some(clean_value),
                    "model" => frontmatter.model = Some(clean_value),
                    "user-invocable" => frontmatter.user_invocable = Some(clean_value),
                    "disable-model-invocation" => {
                        frontmatter.disable_model_invocation = Some(clean_value)
                    }
                    _ => {
                        extra_fields.insert(key.to_string(), clean_value);
                    }
                }
            }
        }

        (frontmatter, body)
    } else {
        (SkillFrontmatter::default(), content.to_string())
    }
}

/// Parse allowed-tools field into a list
pub fn parse_allowed_tools(value: Option<&str>) -> Option<Vec<String>> {
    value.and_then(|v| {
        if v.is_empty() {
            return None;
        }
        if v.contains(',') {
            Some(
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            )
        } else {
            Some(vec![v.trim().to_string()])
        }
    })
}

/// Parse boolean field
pub fn parse_boolean(value: Option<&str>, default: bool) -> bool {
    value
        .map(|v| {
            let lower = v.to_lowercase();
            matches!(lower.as_str(), "true" | "1" | "yes")
        })
        .unwrap_or(default)
}

/// Find supporting files in a skill directory
pub fn find_supporting_files(directory: &Path, skill_file: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path != skill_file {
                files.push(path);
            } else if path.is_dir() {
                // Recursively find files in subdirectories
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file() {
                            files.push(sub_path);
                        }
                    }
                }
            }
        }
    }

    files
}

/// Load a skill from a SKILL.md file
pub fn load_skill_from_file(
    skill_name: &str,
    file_path: &Path,
    source: SkillSource,
) -> Result<SkillDefinition, String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read skill file: {}", e))?;

    let (frontmatter, markdown_content) = parse_frontmatter(&content);

    let base_dir = file_path
        .parent()
        .ok_or("Skill file has no parent directory")?
        .to_path_buf();

    let supporting_files = find_supporting_files(&base_dir, file_path);

    let display_name = frontmatter
        .name
        .clone()
        .unwrap_or_else(|| skill_name.to_string());
    let description = frontmatter.description.clone().unwrap_or_default();
    let has_user_specified_description = frontmatter.description.is_some();

    let allowed_tools = parse_allowed_tools(frontmatter.allowed_tools.as_deref());
    let disable_model_invocation =
        parse_boolean(frontmatter.disable_model_invocation.as_deref(), false);
    let user_invocable = parse_boolean(frontmatter.user_invocable.as_deref(), true);

    Ok(SkillDefinition {
        skill_name: skill_name.to_string(),
        display_name,
        description,
        has_user_specified_description,
        markdown_content,
        allowed_tools,
        argument_hint: frontmatter.argument_hint,
        when_to_use: frontmatter.when_to_use,
        version: frontmatter.version,
        model: frontmatter.model,
        disable_model_invocation,
        user_invocable,
        source,
        base_dir,
        file_path: file_path.to_path_buf(),
        supporting_files,
    })
}

/// Load skills from a directory
///
/// 1. Check for SKILL.md in root (single skill mode)
/// 2. Otherwise, scan subdirectories for SKILL.md files
pub fn load_skills_from_directory(dir_path: &Path, source: SkillSource) -> Vec<SkillDefinition> {
    let mut results = Vec::new();

    if !dir_path.exists() {
        return results;
    }

    // 1. Check for SKILL.md in root directory (single skill mode)
    let root_skill_file = dir_path.join("SKILL.md");
    if root_skill_file.exists() {
        let skill_name = format!(
            "{}:{}",
            source,
            dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
        );

        if let Ok(skill) = load_skill_from_file(&skill_name, &root_skill_file, source) {
            results.push(skill);
        }
        return results;
    }

    // 2. Scan subdirectories for SKILL.md files
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                let skill_name = format!(
                    "{}:{}",
                    source,
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                );

                if let Ok(skill) = load_skill_from_file(&skill_name, &skill_file, source) {
                    results.push(skill);
                }
            }
        }
    }

    results
}

/// Get enabled plugins from settings
pub fn get_enabled_plugins() -> std::collections::HashSet<String> {
    let mut enabled = std::collections::HashSet::new();

    if let Some(home) = dirs::home_dir() {
        let settings_path = home.join(".claude/settings.json");
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(plugins) = settings.get("enabledPlugins").and_then(|v| v.as_object()) {
                    for (plugin_id, is_enabled) in plugins {
                        if is_enabled.as_bool().unwrap_or(false) {
                            enabled.insert(plugin_id.clone());
                        }
                    }
                }
            }
        }
    }

    enabled
}

/// Load skills from plugin cache
///
pub fn load_skills_from_plugin_cache() -> Vec<SkillDefinition> {
    let mut results = Vec::new();

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return results,
    };

    let plugins_cache_dir = home.join(".claude/plugins/cache");
    if !plugins_cache_dir.exists() {
        return results;
    }

    let enabled_plugins = get_enabled_plugins();

    // Traverse marketplace directories
    let marketplaces = match fs::read_dir(&plugins_cache_dir) {
        Ok(entries) => entries,
        Err(_) => return results,
    };

    for marketplace_entry in marketplaces.flatten() {
        if !marketplace_entry.path().is_dir() {
            continue;
        }

        let marketplace_name = marketplace_entry.file_name();
        let marketplace_path = marketplace_entry.path();

        let plugins = match fs::read_dir(&marketplace_path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for plugin_entry in plugins.flatten() {
            if !plugin_entry.path().is_dir() {
                continue;
            }

            let plugin_name = plugin_entry.file_name();
            let plugin_id = format!(
                "{}@{}",
                plugin_name.to_string_lossy(),
                marketplace_name.to_string_lossy()
            );

            // Check if plugin is enabled
            if !enabled_plugins.contains(&plugin_id) {
                continue;
            }

            let plugin_path = plugin_entry.path();
            let versions = match fs::read_dir(&plugin_path) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for version_entry in versions.flatten() {
                if !version_entry.path().is_dir() {
                    continue;
                }

                let skills_path = version_entry.path().join("skills");
                if !skills_path.exists() {
                    continue;
                }

                let skill_dirs = match fs::read_dir(&skills_path) {
                    Ok(entries) => entries,
                    Err(_) => continue,
                };

                for skill_dir_entry in skill_dirs.flatten() {
                    if !skill_dir_entry.path().is_dir() {
                        continue;
                    }

                    let skill_md_path = skill_dir_entry.path().join("SKILL.md");
                    if !skill_md_path.exists() {
                        continue;
                    }

                    let skill_name = format!(
                        "{}:{}",
                        plugin_name.to_string_lossy(),
                        skill_dir_entry.file_name().to_string_lossy()
                    );

                    if let Ok(skill) =
                        load_skill_from_file(&skill_name, &skill_md_path, SkillSource::Plugin)
                    {
                        results.push(skill);
                    }
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = r#"---
name: test-skill
description: A test skill
---

# Test Skill

This is the body.
"#;
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.name, Some("test-skill".to_string()));
        assert_eq!(fm.description, Some("A test skill".to_string()));
        assert!(body.contains("# Test Skill"));
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Just content\nNo frontmatter here.";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.name.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_with_quotes() {
        let content = r#"---
name: "quoted-name"
description: 'single quoted'
---
Body
"#;
        let (fm, _) = parse_frontmatter(content);
        assert_eq!(fm.name, Some("quoted-name".to_string()));
        assert_eq!(fm.description, Some("single quoted".to_string()));
    }

    #[test]
    fn test_parse_allowed_tools() {
        assert_eq!(parse_allowed_tools(None), None);
        assert_eq!(parse_allowed_tools(Some("")), None);
        assert_eq!(
            parse_allowed_tools(Some("tool1")),
            Some(vec!["tool1".to_string()])
        );
        assert_eq!(
            parse_allowed_tools(Some("tool1, tool2, tool3")),
            Some(vec![
                "tool1".to_string(),
                "tool2".to_string(),
                "tool3".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_boolean() {
        assert!(!parse_boolean(None, false));
        assert!(parse_boolean(None, true));
        assert!(parse_boolean(Some("true"), false));
        assert!(parse_boolean(Some("TRUE"), false));
        assert!(parse_boolean(Some("1"), false));
        assert!(parse_boolean(Some("yes"), false));
        assert!(!parse_boolean(Some("false"), true));
        assert!(!parse_boolean(Some("no"), true));
    }

    #[test]
    fn test_load_skill_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("my-skill");
        fs::create_dir(&skill_dir).unwrap();

        let skill_file = skill_dir.join("SKILL.md");
        fs::write(
            &skill_file,
            r#"---
name: my-skill
description: Test skill description
allowed-tools: tool1, tool2
version: 1.0.0
---

# My Skill

Instructions here.
"#,
        )
        .unwrap();

        // Add supporting file
        fs::write(skill_dir.join("helper.py"), "print('hello')").unwrap();

        let skill = load_skill_from_file("user:my-skill", &skill_file, SkillSource::User).unwrap();

        assert_eq!(skill.skill_name, "user:my-skill");
        assert_eq!(skill.display_name, "my-skill");
        assert_eq!(skill.description, "Test skill description");
        assert!(skill.has_user_specified_description);
        assert_eq!(
            skill.allowed_tools,
            Some(vec!["tool1".to_string(), "tool2".to_string()])
        );
        assert_eq!(skill.version, Some("1.0.0".to_string()));
        assert!(skill.user_invocable);
        assert!(!skill.disable_model_invocation);
        assert_eq!(skill.supporting_files.len(), 1);
    }

    #[test]
    fn test_load_skills_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir(&skills_dir).unwrap();

        // Create skill 1
        let skill1_dir = skills_dir.join("skill-one");
        fs::create_dir(&skill1_dir).unwrap();
        fs::write(
            skill1_dir.join("SKILL.md"),
            r#"---
name: skill-one
description: First skill
---
Content 1
"#,
        )
        .unwrap();

        // Create skill 2
        let skill2_dir = skills_dir.join("skill-two");
        fs::create_dir(&skill2_dir).unwrap();
        fs::write(
            skill2_dir.join("SKILL.md"),
            r#"---
name: skill-two
description: Second skill
---
Content 2
"#,
        )
        .unwrap();

        let skills = load_skills_from_directory(&skills_dir, SkillSource::User);

        assert_eq!(skills.len(), 2);
        let names: Vec<_> = skills.iter().map(|s| s.short_name()).collect();
        assert!(names.contains(&"skill-one"));
        assert!(names.contains(&"skill-two"));
    }

    #[test]
    fn test_load_skills_single_skill_mode() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("single-skill");
        fs::create_dir(&skill_dir).unwrap();

        // SKILL.md in root = single skill mode
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: single
description: Single skill
---
Content
"#,
        )
        .unwrap();

        let skills = load_skills_from_directory(&skill_dir, SkillSource::Project);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].display_name, "single");
    }

    #[test]
    fn test_find_supporting_files() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path();

        let skill_file = skill_dir.join("SKILL.md");
        fs::write(&skill_file, "content").unwrap();
        fs::write(skill_dir.join("helper.py"), "code").unwrap();
        fs::write(skill_dir.join("config.json"), "{}").unwrap();

        let sub_dir = skill_dir.join("templates");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("template.txt"), "template").unwrap();

        let files = find_supporting_files(skill_dir, &skill_file);

        assert_eq!(files.len(), 3);
    }
}
