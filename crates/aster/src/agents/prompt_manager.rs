//! 提示词管理器
//!
//! 管理 Agent 的系统提示词，支持分层组合：
//! 1. Identity（身份层）- 应用层可完全控制
//! 2. Capabilities（能力层）- 框架提供的 Extensions 等能力描述
//! 3. Context（上下文层）- 运行时注入的 hints 和额外指令

#[cfg(test)]
use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

use super::identity::AgentIdentity;
use crate::agents::extension::ExtensionInfo;
use crate::hints::load_hints::{load_hint_files, AGENTS_MD_FILENAME, ASTER_HINTS_FILENAME};
use crate::{
    config::{AsterMode, Config},
    prompt_template,
    utils::sanitize_unicode_tags,
};
use std::path::Path;

const MAX_EXTENSIONS: usize = 5;
const MAX_TOOLS: usize = 50;

pub struct PromptManager {
    /// 完全覆盖系统提示词（向后兼容）
    system_prompt_override: Option<String>,
    /// 额外指令（追加到末尾）
    system_prompt_extras: Vec<String>,
    /// 当前时间戳
    current_date_timestamp: String,
    /// Agent 身份配置（新增）
    identity: AgentIdentity,
    /// Session 级别的系统提示词
    session_prompt: Option<String>,
}

impl Default for PromptManager {
    fn default() -> Self {
        PromptManager::new()
    }
}

/// 身份提示词上下文
#[derive(Serialize)]
struct IdentityContext {
    agent_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language_preference: Option<String>,
}

/// 能力提示词上下文
#[derive(Serialize)]
struct SystemPromptContext {
    extensions: Vec<ExtensionInfo>,
    current_date_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extension_tool_limits: Option<(usize, usize)>,
    aster_mode: AsterMode,
    is_autonomous: bool,
    enable_subagents: bool,
    max_extensions: usize,
    max_tools: usize,
    code_execution_mode: bool,
}

pub struct SystemPromptBuilder<'a, M> {
    manager: &'a M,

    extensions_info: Vec<ExtensionInfo>,
    frontend_instructions: Option<String>,
    extension_tool_count: Option<(usize, usize)>,
    subagents_enabled: bool,
    hints: Option<String>,
    code_execution_mode: bool,
    session_prompt: Option<String>,
}

impl<'a> SystemPromptBuilder<'a, PromptManager> {
    pub fn with_extension(mut self, extension: ExtensionInfo) -> Self {
        self.extensions_info.push(extension);
        self
    }

    pub fn with_extensions(mut self, extensions: impl Iterator<Item = ExtensionInfo>) -> Self {
        for extension in extensions {
            self.extensions_info.push(extension);
        }
        self
    }

    pub fn with_frontend_instructions(mut self, frontend_instructions: Option<String>) -> Self {
        self.frontend_instructions = frontend_instructions;
        self
    }

    pub fn with_extension_and_tool_counts(
        mut self,
        extension_count: usize,
        tool_count: usize,
    ) -> Self {
        self.extension_tool_count = Some((extension_count, tool_count));
        self
    }

    pub fn with_code_execution_mode(mut self, enabled: bool) -> Self {
        self.code_execution_mode = enabled;
        self
    }

    pub fn with_hints(mut self, working_dir: &Path) -> Self {
        let config = Config::global();
        let hints_filenames = config
            .get_param::<Vec<String>>("CONTEXT_FILE_NAMES")
            .unwrap_or_else(|_| {
                vec![
                    ASTER_HINTS_FILENAME.to_string(),
                    AGENTS_MD_FILENAME.to_string(),
                ]
            });
        let ignore_patterns = {
            let builder = ignore::gitignore::GitignoreBuilder::new(working_dir);
            builder.build().unwrap_or_else(|_| {
                ignore::gitignore::GitignoreBuilder::new(working_dir)
                    .build()
                    .expect("Failed to build default gitignore")
            })
        };

        let hints = load_hint_files(working_dir, &hints_filenames, &ignore_patterns);

        if !hints.is_empty() {
            self.hints = Some(hints);
        }
        self
    }

    pub fn with_enable_subagents(mut self, subagents_enabled: bool) -> Self {
        self.subagents_enabled = subagents_enabled;
        self
    }

    /// 设置 session 级别的系统提示词
    pub fn with_session_prompt(mut self, prompt: Option<String>) -> Self {
        self.session_prompt = prompt;
        self
    }

    pub fn build(self) -> String {
        let mut extensions_info = self.extensions_info;

        // Add frontend instructions to extensions_info to simplify json rendering
        if let Some(frontend_instructions) = self.frontend_instructions {
            extensions_info.push(ExtensionInfo::new(
                "frontend",
                &frontend_instructions,
                false,
            ));
        }
        // Stable tool ordering is important for multi session prompt caching.
        extensions_info.sort_by(|a, b| a.name.cmp(&b.name));

        let sanitized_extensions_info: Vec<ExtensionInfo> = extensions_info
            .into_iter()
            .map(|mut ext_info| {
                ext_info.instructions = sanitize_unicode_tags(&ext_info.instructions);
                ext_info
            })
            .collect();

        let config = Config::global();
        let aster_mode = config.get_aster_mode().unwrap_or(AsterMode::Auto);

        let extension_tool_limits = self
            .extension_tool_count
            .filter(|(extensions, tools)| *extensions > MAX_EXTENSIONS || *tools > MAX_TOOLS);

        let capabilities_context = SystemPromptContext {
            extensions: sanitized_extensions_info,
            current_date_time: self.manager.current_date_timestamp.clone(),
            extension_tool_limits,
            aster_mode,
            is_autonomous: aster_mode == AsterMode::Auto,
            enable_subagents: self.subagents_enabled,
            max_extensions: MAX_EXTENSIONS,
            max_tools: MAX_TOOLS,
            code_execution_mode: self.code_execution_mode,
        };

        // 构建提示词：优先使用 override，否则使用分层结构
        let base_prompt = if let Some(override_prompt) = &self.manager.system_prompt_override {
            // 向后兼容：完全覆盖模式
            let sanitized_override_prompt = sanitize_unicode_tags(override_prompt);
            prompt_template::render_inline_once(&sanitized_override_prompt, &capabilities_context)
                .unwrap_or_else(|_| override_prompt.clone())
        } else {
            // 新的分层模式：Identity + Session Context + Capabilities
            Self::build_layered_prompt_with_session(
                &self.manager.identity,
                &self.session_prompt,
                &capabilities_context,
            )
        };

        let mut system_prompt_extras = self.manager.system_prompt_extras.clone();

        // Add hints if provided
        if let Some(hints) = self.hints {
            system_prompt_extras.push(hints);
        }

        if aster_mode == AsterMode::Chat {
            system_prompt_extras.push(
                "Right now you are in the chat only mode, no access to any tool use and system."
                    .to_string(),
            );
        }

        let sanitized_system_prompt_extras: Vec<String> = system_prompt_extras
            .into_iter()
            .map(|extra| sanitize_unicode_tags(&extra))
            .collect();

        if sanitized_system_prompt_extras.is_empty() {
            base_prompt
        } else {
            format!(
                "{}\n\n# Additional Instructions:\n\n{}",
                base_prompt,
                sanitized_system_prompt_extras.join("\n\n")
            )
        }
    }

    /// 构建分层提示词：Identity + Capabilities（静态方法）
    fn build_layered_prompt_static(
        identity: &AgentIdentity,
        capabilities_context: &SystemPromptContext,
    ) -> String {
        // 1. 构建身份层
        let identity_prompt = if let Some(custom) = &identity.custom_prompt {
            // 使用完全自定义的身份提示词
            sanitize_unicode_tags(custom)
        } else {
            // 使用模板渲染身份
            let identity_context = IdentityContext {
                agent_name: identity.name.clone(),
                agent_creator: identity.creator.clone(),
                agent_description: identity.description.clone(),
                language_preference: identity.language.clone(),
            };
            prompt_template::render_global_file("identity.md", &identity_context)
                .unwrap_or_else(|_| format!("You are an AI agent called {}.", identity.name))
        };

        // 2. 构建能力层
        let capabilities_prompt =
            prompt_template::render_global_file("capabilities.md", capabilities_context)
                .unwrap_or_default();

        // 3. 组合
        if capabilities_prompt.is_empty() {
            identity_prompt
        } else {
            format!("{}\n\n{}", identity_prompt, capabilities_prompt)
        }
    }

    /// 构建分层提示词（包含 session_prompt）：Identity + Session Context + Capabilities
    fn build_layered_prompt_with_session(
        identity: &AgentIdentity,
        session_prompt: &Option<String>,
        capabilities_context: &SystemPromptContext,
    ) -> String {
        // 1. 构建身份层
        let identity_prompt = if let Some(custom) = &identity.custom_prompt {
            sanitize_unicode_tags(custom)
        } else {
            let identity_context = IdentityContext {
                agent_name: identity.name.clone(),
                agent_creator: identity.creator.clone(),
                agent_description: identity.description.clone(),
                language_preference: identity.language.clone(),
            };
            prompt_template::render_global_file("identity.md", &identity_context)
                .unwrap_or_else(|_| format!("You are an AI agent called {}.", identity.name))
        };

        // 2. Session Context 层（如果有）
        let session_section = if let Some(prompt) = session_prompt {
            let sanitized = sanitize_unicode_tags(prompt);
            format!("\n\n## Session Context\n\n{}", sanitized)
        } else {
            String::new()
        };

        // 3. 构建能力层
        let capabilities_prompt =
            prompt_template::render_global_file("capabilities.md", capabilities_context)
                .unwrap_or_default();

        // 4. 组合：Identity + Session Context + Capabilities
        if capabilities_prompt.is_empty() {
            format!("{}{}", identity_prompt, session_section)
        } else {
            format!(
                "{}{}\n\n{}",
                identity_prompt, session_section, capabilities_prompt
            )
        }
    }
}

impl PromptManager {
    pub fn new() -> Self {
        PromptManager {
            system_prompt_override: None,
            system_prompt_extras: Vec::new(),
            current_date_timestamp: Utc::now().format("%Y-%m-%d %H:00").to_string(),
            identity: AgentIdentity::default(),
            session_prompt: None,
        }
    }

    /// 创建带自定义身份的 PromptManager
    pub fn with_identity(identity: AgentIdentity) -> Self {
        PromptManager {
            system_prompt_override: None,
            system_prompt_extras: Vec::new(),
            current_date_timestamp: Utc::now().format("%Y-%m-%d %H:00").to_string(),
            identity,
            session_prompt: None,
        }
    }

    #[cfg(test)]
    pub fn with_timestamp(dt: DateTime<Utc>) -> Self {
        PromptManager {
            system_prompt_override: None,
            system_prompt_extras: Vec::new(),
            current_date_timestamp: dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            identity: AgentIdentity::default(),
            session_prompt: None,
        }
    }

    /// 设置 Agent 身份
    pub fn set_identity(&mut self, identity: AgentIdentity) {
        self.identity = identity;
    }

    /// 获取当前身份配置
    pub fn identity(&self) -> &AgentIdentity {
        &self.identity
    }

    /// 设置 session 级别的系统提示词
    pub fn set_session_prompt(&mut self, prompt: Option<String>) {
        self.session_prompt = prompt;
    }

    /// 获取当前 session 提示词
    pub fn session_prompt(&self) -> Option<&String> {
        self.session_prompt.as_ref()
    }

    /// 清除 session 提示词
    pub fn clear_session_prompt(&mut self) {
        self.session_prompt = None;
    }

    /// Add an additional instruction to the system prompt
    pub fn add_system_prompt_extra(&mut self, instruction: String) {
        self.system_prompt_extras.push(instruction);
    }

    /// Override the system prompt with custom text (向后兼容)
    pub fn set_system_prompt_override(&mut self, template: String) {
        self.system_prompt_override = Some(template);
    }

    pub fn builder<'a>(&'a self) -> SystemPromptBuilder<'a, Self> {
        SystemPromptBuilder {
            manager: self,

            extensions_info: vec![],
            frontend_instructions: None,
            extension_tool_count: None,
            subagents_enabled: false,
            hints: None,
            code_execution_mode: false,
            session_prompt: None,
        }
    }

    pub async fn get_recipe_prompt(&self) -> String {
        let context: HashMap<&str, Value> = HashMap::new();
        prompt_template::render_global_file("recipe.md", &context)
            .unwrap_or_else(|_| "The recipe prompt is busted. Tell the user.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_build_system_prompt_sanitizes_override() {
        let mut manager = PromptManager::new();
        let malicious_override = "System prompt\u{E0041}\u{E0042}\u{E0043}with hidden text";
        manager.set_system_prompt_override(malicious_override.to_string());

        let result = manager.builder().build();

        assert!(!result.contains('\u{E0041}'));
        assert!(!result.contains('\u{E0042}'));
        assert!(!result.contains('\u{E0043}'));
        assert!(result.contains("System prompt"));
        assert!(result.contains("with hidden text"));
    }

    #[test]
    fn test_build_system_prompt_sanitizes_extras() {
        let mut manager = PromptManager::new();
        let malicious_extra = "Extra instruction\u{E0041}\u{E0042}\u{E0043}hidden";
        manager.add_system_prompt_extra(malicious_extra.to_string());

        let result = manager.builder().build();

        assert!(!result.contains('\u{E0041}'));
        assert!(!result.contains('\u{E0042}'));
        assert!(!result.contains('\u{E0043}'));
        assert!(result.contains("Extra instruction"));
        assert!(result.contains("hidden"));
    }

    #[test]
    fn test_build_system_prompt_sanitizes_multiple_extras() {
        let mut manager = PromptManager::new();
        manager.add_system_prompt_extra("First\u{E0041}instruction".to_string());
        manager.add_system_prompt_extra("Second\u{E0042}instruction".to_string());
        manager.add_system_prompt_extra("Third\u{E0043}instruction".to_string());

        let result = manager.builder().build();

        assert!(!result.contains('\u{E0041}'));
        assert!(!result.contains('\u{E0042}'));
        assert!(!result.contains('\u{E0043}'));
        assert!(result.contains("Firstinstruction"));
        assert!(result.contains("Secondinstruction"));
        assert!(result.contains("Thirdinstruction"));
    }

    #[test]
    fn test_build_system_prompt_preserves_legitimate_unicode_in_extras() {
        let mut manager = PromptManager::new();
        let legitimate_unicode = "Instruction with 世界 and 🌍 emojis";
        manager.add_system_prompt_extra(legitimate_unicode.to_string());

        let result = manager.builder().build();

        assert!(result.contains("世界"));
        assert!(result.contains("🌍"));
        assert!(result.contains("Instruction with"));
        assert!(result.contains("emojis"));
    }

    #[test]
    fn test_build_system_prompt_sanitizes_extension_instructions() {
        let manager = PromptManager::new();
        let malicious_extension_info = ExtensionInfo::new(
            "test_extension",
            "Extension help\u{E0041}\u{E0042}\u{E0043}hidden instructions",
            false,
        );

        let result = manager
            .builder()
            .with_extension(malicious_extension_info)
            .build();

        assert!(!result.contains('\u{E0041}'));
        assert!(!result.contains('\u{E0042}'));
        assert!(!result.contains('\u{E0043}'));
        assert!(result.contains("Extension help"));
        assert!(result.contains("hidden instructions"));
    }

    #[test]
    fn test_basic() {
        let manager = PromptManager::with_timestamp(DateTime::<Utc>::from_timestamp(0, 0).unwrap());

        let system_prompt = manager.builder().build();

        assert_snapshot!(system_prompt)
    }

    #[test]
    fn test_one_extension() {
        let manager = PromptManager::with_timestamp(DateTime::<Utc>::from_timestamp(0, 0).unwrap());

        let system_prompt = manager
            .builder()
            .with_extension(ExtensionInfo::new(
                "test",
                "how to use this extension",
                true,
            ))
            .build();

        assert_snapshot!(system_prompt)
    }

    #[test]
    fn test_typical_setup() {
        let manager = PromptManager::with_timestamp(DateTime::<Utc>::from_timestamp(0, 0).unwrap());

        let system_prompt = manager
            .builder()
            .with_extension(ExtensionInfo::new(
                "extension_A",
                "<instructions on how to use extension A>",
                true,
            ))
            .with_extension(ExtensionInfo::new(
                "extension_B",
                "<instructions on how to use extension B (no resources)>",
                false,
            ))
            .with_extension_and_tool_counts(MAX_EXTENSIONS + 1, MAX_TOOLS + 1)
            .build();

        assert_snapshot!(system_prompt)
    }
}
