use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use futures::FutureExt;
use rmcp::model::{Content, ErrorCode, ErrorData, Tool};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use crate::agents::subagent_handler::run_complete_subagent_task;
use crate::agents::subagent_task_config::TaskConfig;
use crate::agents::tool_execution::ToolCallResult;
use crate::providers;
use crate::recipe::build_recipe::build_recipe_from_template;
use crate::recipe::local_recipes::load_local_recipe_file;
use crate::recipe::{Recipe, SubRecipe};
use crate::session::{SessionManager, SubagentSessionMetadata};

pub const SUBAGENT_TOOL_NAME: &str = "subagent";
const SUBAGENT_TASK_SUMMARY_MAX_CHARS: usize = 160;

const SUMMARY_INSTRUCTIONS: &str = r#"
Important: Your parent agent will only receive your final message as a summary of your work.
Make sure your last message provides a comprehensive summary of:
- What you were asked to do
- What actions you took
- The results or outcomes
- Any important findings or recommendations

Be concise but complete.
"#;

#[derive(Debug, Deserialize)]
pub struct SubagentParams {
    pub instructions: Option<String>,
    pub subrecipe: Option<String>,
    pub role_hint: Option<String>,
    pub parameters: Option<HashMap<String, Value>>,
    pub extensions: Option<Vec<String>>,
    pub settings: Option<SubagentSettings>,
    #[serde(default = "default_summary")]
    pub summary: bool,
    pub images: Option<Vec<ImageData>>,
}

#[derive(Debug, Deserialize)]
pub struct ImageData {
    pub data: String,
    pub mime_type: String,
}

fn default_summary() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct SubagentSettings {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
}

pub fn create_subagent_tool(sub_recipes: &[SubRecipe]) -> Tool {
    let description = build_tool_description(sub_recipes);

    let schema = json!({
        "type": "object",
        "properties": {
            "instructions": {
                "type": "string",
                "description": "Instructions for the subagent. Required for ad-hoc tasks. For predefined tasks, adds additional context."
            },
            "subrecipe": {
                "type": "string",
                "description": "Name of a predefined subrecipe to run."
            },
            "role_hint": {
                "type": "string",
                "description": "Optional role or display label for the subagent, for example 'planner' or 'Image #1'."
            },
            "parameters": {
                "type": "object",
                "additionalProperties": true,
                "description": "Parameters for the subrecipe. Only valid when 'subrecipe' is specified."
            },
            "extensions": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Extensions to enable. Omit to inherit all, empty array for none."
            },
            "settings": {
                "type": "object",
                "properties": {
                    "provider": {"type": "string", "description": "Override LLM provider"},
                    "model": {"type": "string", "description": "Override model"},
                    "temperature": {"type": "number", "description": "Override temperature"}
                },
                "description": "Override model/provider settings."
            },
            "summary": {
                "type": "boolean",
                "default": true,
                "description": "If true (default), return only the subagent's final summary."
            },
            "images": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "data": {"type": "string", "description": "Base64 encoded image data"},
                        "mime_type": {"type": "string", "description": "MIME type of the image"}
                    },
                    "required": ["data", "mime_type"]
                },
                "description": "Images to include in the subagent task for multimodal analysis."
            }
        }
    });

    Tool::new(
        SUBAGENT_TOOL_NAME,
        description,
        schema.as_object().unwrap().clone(),
    )
}

fn build_tool_description(sub_recipes: &[SubRecipe]) -> String {
    let mut desc = String::from(
        "Delegate a task to a subagent that runs independently with its own context.\n\n\
         Modes:\n\
         1. Ad-hoc: Provide `instructions` for a custom task\n\
         2. Predefined: Provide `subrecipe` name to run a predefined task\n\
         3. Augmented: Provide both `subrecipe` and `instructions` to add context\n\n\
         The subagent has access to the same tools as you by default. \
         Use `extensions` to limit which extensions the subagent can use.\n\n\
         For parallel execution, make multiple `subagent` tool calls in the same message.",
    );

    if !sub_recipes.is_empty() {
        desc.push_str("\n\nAvailable subrecipes:");
        for sr in sub_recipes {
            let params_info = get_subrecipe_params_description(sr);
            let sequential_hint = if sr.sequential_when_repeated {
                " [run sequentially, not in parallel]"
            } else {
                ""
            };
            desc.push_str(&format!(
                "\n• {}{} - {}{}",
                sr.name,
                sequential_hint,
                sr.description.as_deref().unwrap_or("No description"),
                if params_info.is_empty() {
                    String::new()
                } else {
                    format!(" (params: {})", params_info)
                }
            ));
        }
    }

    desc
}

fn get_subrecipe_params_description(sub_recipe: &SubRecipe) -> String {
    match load_local_recipe_file(&sub_recipe.path) {
        Ok(recipe_file) => match Recipe::from_content(&recipe_file.content) {
            Ok(recipe) => {
                if let Some(params) = recipe.parameters {
                    params
                        .iter()
                        .filter(|p| {
                            sub_recipe
                                .values
                                .as_ref()
                                .map(|v| !v.contains_key(&p.key))
                                .unwrap_or(true)
                        })
                        .map(|p| {
                            let req = match p.requirement {
                                crate::recipe::RecipeParameterRequirement::Required => "[required]",
                                _ => "[optional]",
                            };
                            format!("{} {}", p.key, req)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    }
}

/// Note: SubRecipe.sequential_when_repeated is surfaced as a hint in the tool description
/// (e.g., "[run sequentially, not in parallel]") but not enforced. The LLM controls
/// sequencing by making sequential vs parallel tool calls.
pub fn handle_subagent_tool(
    params: Value,
    task_config: TaskConfig,
    sub_recipes: HashMap<String, SubRecipe>,
    working_dir: PathBuf,
    cancellation_token: Option<CancellationToken>,
) -> ToolCallResult {
    let parsed_params: SubagentParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return ToolCallResult::from(Err(ErrorData {
                code: ErrorCode::INVALID_PARAMS,
                message: Cow::from(format!("Invalid parameters: {}", e)),
                data: None,
            }));
        }
    };

    if parsed_params.instructions.is_none() && parsed_params.subrecipe.is_none() {
        return ToolCallResult::from(Err(ErrorData {
            code: ErrorCode::INVALID_PARAMS,
            message: Cow::from("Must provide 'instructions' or 'subrecipe' (or both)"),
            data: None,
        }));
    }

    if parsed_params.parameters.is_some() && parsed_params.subrecipe.is_none() {
        return ToolCallResult::from(Err(ErrorData {
            code: ErrorCode::INVALID_PARAMS,
            message: Cow::from("'parameters' can only be used with 'subrecipe'"),
            data: None,
        }));
    }

    let recipe = match build_recipe(&parsed_params, &sub_recipes) {
        Ok(r) => r,
        Err(e) => {
            return ToolCallResult::from(Err(ErrorData {
                code: ErrorCode::INVALID_PARAMS,
                message: Cow::from(e.to_string()),
                data: None,
            }));
        }
    };

    ToolCallResult {
        notification_stream: None,
        result: Box::new(
            execute_subagent(
                recipe,
                task_config,
                parsed_params,
                working_dir,
                cancellation_token,
            )
            .boxed(),
        ),
    }
}

async fn execute_subagent(
    recipe: Recipe,
    task_config: TaskConfig,
    params: SubagentParams,
    working_dir: PathBuf,
    cancellation_token: Option<CancellationToken>,
) -> Result<rmcp::model::CallToolResult, ErrorData> {
    let task_config = apply_settings_overrides(task_config, &params)
        .await
        .map_err(|e| ErrorData {
            code: ErrorCode::INVALID_PARAMS,
            message: Cow::from(e.to_string()),
            data: None,
        })?;

    let session = SessionManager::create_session(
        working_dir,
        build_subagent_session_name(&params, &recipe),
        crate::session::session_manager::SessionType::SubAgent,
    )
    .await
    .map_err(|e| ErrorData {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::from(format!("Failed to create session: {}", e)),
        data: None,
    })?;

    persist_subagent_session_metadata(
        &session.id,
        &session,
        build_subagent_session_metadata(&task_config, &params, &recipe),
    )
    .await
    .map_err(|e| ErrorData {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::from(format!(
            "Failed to persist subagent session metadata: {}",
            e
        )),
        data: None,
    })?;

    let result = run_complete_subagent_task(
        recipe,
        task_config,
        params.summary,
        session.id,
        params.images,
        cancellation_token,
    )
    .await;

    match result {
        Ok(text) => Ok(rmcp::model::CallToolResult {
            content: vec![Content::text(text)],
            structured_content: None,
            is_error: Some(false),
            meta: None,
        }),
        Err(e) => Err(ErrorData {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(e.to_string()),
            data: None,
        }),
    }
}

fn build_subagent_session_metadata(
    task_config: &TaskConfig,
    params: &SubagentParams,
    recipe: &Recipe,
) -> SubagentSessionMetadata {
    SubagentSessionMetadata::new(task_config.parent_session_id.clone())
        .with_task_summary(build_subagent_task_summary(params, recipe))
        .with_role_hint(build_subagent_role_hint(params))
        .with_created_from_turn_id(resolve_parent_turn_id(&task_config.parent_session_id))
}

fn build_subagent_task_summary(params: &SubagentParams, recipe: &Recipe) -> Option<String> {
    let subrecipe_name = params
        .subrecipe
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let instruction_preview = params
        .instructions
        .as_deref()
        .map(normalize_whitespace)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_chars(&value, SUBAGENT_TASK_SUMMARY_MAX_CHARS));

    match (subrecipe_name, instruction_preview) {
        (Some(subrecipe), Some(instruction)) => Some(truncate_chars(
            &format!("Run subrecipe `{}`: {}", subrecipe, instruction),
            SUBAGENT_TASK_SUMMARY_MAX_CHARS,
        )),
        (Some(subrecipe), None) => Some(truncate_chars(
            &format!("Run subrecipe `{}`", subrecipe),
            SUBAGENT_TASK_SUMMARY_MAX_CHARS,
        )),
        (None, Some(instruction)) => Some(instruction),
        (None, None) => {
            let title = recipe.title.trim();
            if title.is_empty() {
                None
            } else {
                Some(truncate_chars(title, SUBAGENT_TASK_SUMMARY_MAX_CHARS))
            }
        }
    }
}

fn build_subagent_role_hint(params: &SubagentParams) -> Option<String> {
    normalize_subagent_label(params.role_hint.as_deref())
        .or_else(|| normalize_subagent_label(params.subrecipe.as_deref()))
}

fn resolve_parent_turn_id(parent_session_id: &str) -> Option<String> {
    let scope = crate::session_context::current_action_scope()?;
    if scope.session_id.as_deref() != Some(parent_session_id) {
        return None;
    }

    normalize_optional_identifier(scope.turn_id)
}

fn build_subagent_session_name(params: &SubagentParams, recipe: &Recipe) -> String {
    build_subagent_role_hint(params)
        .or_else(|| {
            build_subagent_task_summary(params, recipe)
                .map(|summary| truncate_chars(&summary, SUBAGENT_TASK_SUMMARY_MAX_CHARS))
        })
        .unwrap_or_else(|| "Subagent task".to_string())
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_subagent_label(value: Option<&str>) -> Option<String> {
    let normalized = value
        .map(normalize_whitespace)
        .unwrap_or_default()
        .trim()
        .to_string();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_optional_identifier(value: Option<String>) -> Option<String> {
    let normalized = value?.trim().to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    if max_chars <= 3 {
        return value.chars().take(max_chars).collect();
    }

    let truncated: String = value.chars().take(max_chars - 3).collect();
    format!("{}...", truncated)
}

async fn persist_subagent_session_metadata(
    session_id: &str,
    session: &crate::session::Session,
    metadata: SubagentSessionMetadata,
) -> Result<()> {
    let extension_data = metadata.into_updated_extension_data(session)?;
    SessionManager::update_session(session_id)
        .extension_data(extension_data)
        .apply()
        .await
}

fn build_recipe(
    params: &SubagentParams,
    sub_recipes: &HashMap<String, SubRecipe>,
) -> Result<Recipe> {
    let mut recipe = if let Some(subrecipe_name) = &params.subrecipe {
        build_subrecipe(subrecipe_name, params, sub_recipes)?
    } else {
        build_adhoc_recipe(params)?
    };

    if params.summary {
        let current = recipe.instructions.unwrap_or_default();
        recipe.instructions = Some(format!("{}\n{}", current, SUMMARY_INSTRUCTIONS));
    }

    Ok(recipe)
}

fn build_subrecipe(
    subrecipe_name: &str,
    params: &SubagentParams,
    sub_recipes: &HashMap<String, SubRecipe>,
) -> Result<Recipe> {
    let sub_recipe = sub_recipes.get(subrecipe_name).ok_or_else(|| {
        let available: Vec<_> = sub_recipes.keys().cloned().collect();
        anyhow!(
            "Unknown subrecipe '{}'. Available: {}",
            subrecipe_name,
            available.join(", ")
        )
    })?;

    let recipe_file = load_local_recipe_file(&sub_recipe.path)
        .map_err(|e| anyhow!("Failed to load subrecipe '{}': {}", subrecipe_name, e))?;

    let mut param_values: Vec<(String, String)> = Vec::new();

    if let Some(values) = &sub_recipe.values {
        for (k, v) in values {
            param_values.push((k.clone(), v.clone()));
        }
    }

    if let Some(provided_params) = &params.parameters {
        for (k, v) in provided_params {
            let value_str = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            param_values.push((k.clone(), value_str));
        }
    }

    let mut recipe = build_recipe_from_template(
        recipe_file.content,
        &recipe_file.parent_dir,
        param_values,
        None::<fn(&str, &str) -> Result<String, anyhow::Error>>,
    )
    .map_err(|e| anyhow!("Failed to build subrecipe: {}", e))?;

    if let Some(extra) = &params.instructions {
        let mut current = recipe.instructions.take().unwrap_or_default();
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(extra);
        recipe.instructions = Some(current);
    }

    Ok(recipe)
}

fn build_adhoc_recipe(params: &SubagentParams) -> Result<Recipe> {
    let instructions = params
        .instructions
        .as_ref()
        .ok_or_else(|| anyhow!("Instructions required for ad-hoc task"))?;

    let recipe = Recipe::builder()
        .version("1.0.0")
        .title("Subagent Task")
        .description("Ad-hoc subagent task")
        .instructions(instructions)
        .build()
        .map_err(|e| anyhow!("Failed to build recipe: {}", e))?;

    if recipe.check_for_security_warnings() {
        return Err(anyhow!("Recipe contains potentially harmful content"));
    }

    Ok(recipe)
}

async fn apply_settings_overrides(
    mut task_config: TaskConfig,
    params: &SubagentParams,
) -> Result<TaskConfig> {
    if let Some(settings) = &params.settings {
        let current_model_config = task_config.provider.get_model_config();
        let provider_override = settings
            .provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let model_override = settings
            .model
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        if provider_override.is_some() || model_override.is_some() || settings.temperature.is_some()
        {
            let provider_name = provider_override
                .clone()
                .unwrap_or_else(|| task_config.provider.get_name().to_string());
            let resolved_model_name = if let Some(model) = model_override.as_deref() {
                model.to_string()
            } else if provider_override.is_some() {
                providers::create_with_default_model(&provider_name)
                    .await
                    .map_err(|e| {
                        anyhow!(
                            "Failed to resolve default model for provider '{}': {}",
                            provider_name,
                            e
                        )
                    })?
                    .get_model_config()
                    .model_name
            } else {
                current_model_config.model_name.clone()
            };

            let mut model_config = current_model_config
                .rebuild_with_model_name(&resolved_model_name)
                .map_err(|e| {
                    anyhow!(
                        "Failed to rebuild model config for model '{}': {}",
                        resolved_model_name,
                        e
                    )
                })?;

            if let Some(temp) = settings.temperature {
                model_config = model_config.with_temperature(Some(temp));
            }

            task_config.provider = providers::create(&provider_name, model_config)
                .await
                .map_err(|e| anyhow!("Failed to create provider '{}': {}", provider_name, e))?;

            if provider_override.is_some() || model_override.is_some() {
                let turn_context = task_config
                    .turn_context
                    .get_or_insert_with(crate::session::TurnContextOverride::default);
                turn_context.model = Some(task_config.provider.get_model_config().model_name);
            }
        }
    }

    if let Some(extension_names) = &params.extensions {
        if extension_names.is_empty() {
            task_config.extensions = Vec::new();
        } else {
            task_config
                .extensions
                .retain(|ext| extension_names.contains(&ext.name()));
        }
    }

    Ok(task_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::declarative_providers::{DeclarativeProviderConfig, ProviderEngine};
    use crate::conversation::message::ActionRequiredScope;
    use crate::providers::base::{ModelInfo, Provider};
    use crate::session::TurnContextOverride;
    use std::sync::Arc;

    #[test]
    fn test_tool_name() {
        assert_eq!(SUBAGENT_TOOL_NAME, "subagent");
    }

    #[test]
    fn test_create_tool_without_subrecipes() {
        let tool = create_subagent_tool(&[]);
        assert_eq!(tool.name, "subagent");
        assert!(tool.description.as_ref().unwrap().contains("Ad-hoc"));
        assert!(!tool
            .description
            .as_ref()
            .unwrap()
            .contains("Available subrecipes"));
    }

    #[test]
    fn test_create_tool_with_subrecipes() {
        let sub_recipes = vec![SubRecipe {
            name: "test_recipe".to_string(),
            path: "test.yaml".to_string(),
            values: None,
            sequential_when_repeated: false,
            description: Some("A test recipe".to_string()),
        }];

        let tool = create_subagent_tool(&sub_recipes);
        assert!(tool
            .description
            .as_ref()
            .unwrap()
            .contains("Available subrecipes"));
        assert!(tool.description.as_ref().unwrap().contains("test_recipe"));
    }

    #[test]
    fn test_sequential_hint_in_description() {
        let sub_recipes = vec![
            SubRecipe {
                name: "parallel_ok".to_string(),
                path: "test.yaml".to_string(),
                values: None,
                sequential_when_repeated: false,
                description: Some("Can run in parallel".to_string()),
            },
            SubRecipe {
                name: "sequential_only".to_string(),
                path: "test.yaml".to_string(),
                values: None,
                sequential_when_repeated: true,
                description: Some("Must run sequentially".to_string()),
            },
        ];

        let tool = create_subagent_tool(&sub_recipes);
        let desc = tool.description.as_ref().unwrap();

        assert!(desc.contains("parallel_ok"));
        assert!(!desc.contains("parallel_ok [run sequentially"));

        assert!(desc.contains("sequential_only [run sequentially, not in parallel]"));
    }

    #[test]
    fn test_params_deserialization_full() {
        let params: SubagentParams = serde_json::from_value(json!({
            "instructions": "Extra context",
            "subrecipe": "my_recipe",
            "role_hint": "Image #1",
            "parameters": {"key": "value"},
            "extensions": ["developer"],
            "settings": {"model": "gpt-4"},
            "summary": false
        }))
        .unwrap();

        assert_eq!(params.instructions, Some("Extra context".to_string()));
        assert_eq!(params.subrecipe, Some("my_recipe".to_string()));
        assert_eq!(params.role_hint, Some("Image #1".to_string()));
        assert!(params.parameters.is_some());
        assert_eq!(params.extensions, Some(vec!["developer".to_string()]));
        assert!(!params.summary);
    }

    #[test]
    fn test_build_subagent_task_summary_prefers_subrecipe_and_instruction_preview() {
        let params = SubagentParams {
            instructions: Some("Investigate   the    failing \n integration test".to_string()),
            subrecipe: Some("debug_failure".to_string()),
            role_hint: None,
            parameters: None,
            extensions: None,
            settings: None,
            summary: true,
            images: None,
        };

        let recipe = Recipe::builder()
            .version("1.0.0")
            .title("Fallback title")
            .description("Fallback description")
            .instructions("Unused")
            .build()
            .unwrap();

        assert_eq!(
            build_subagent_task_summary(&params, &recipe),
            Some(
                "Run subrecipe `debug_failure`: Investigate the failing integration test"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_build_subagent_task_summary_falls_back_to_recipe_title() {
        let params = SubagentParams {
            instructions: None,
            subrecipe: None,
            role_hint: None,
            parameters: None,
            extensions: None,
            settings: None,
            summary: true,
            images: None,
        };

        let recipe = Recipe::builder()
            .version("1.0.0")
            .title("Subagent Task")
            .description("Ad-hoc task")
            .instructions("Inspect the repository")
            .build()
            .unwrap();

        assert_eq!(
            build_subagent_task_summary(&params, &recipe),
            Some("Subagent Task".to_string())
        );
    }

    #[test]
    fn test_build_subagent_role_hint_prefers_explicit_role_hint() {
        let params = SubagentParams {
            instructions: None,
            subrecipe: Some("planner_recipe".to_string()),
            role_hint: Some("Image #1".to_string()),
            parameters: None,
            extensions: None,
            settings: None,
            summary: true,
            images: None,
        };

        assert_eq!(
            build_subagent_role_hint(&params),
            Some("Image #1".to_string())
        );
    }

    #[test]
    fn test_build_subagent_session_name_prefers_role_hint() {
        let params = SubagentParams {
            instructions: Some("处理图片风格统一".to_string()),
            subrecipe: Some("image_pipeline".to_string()),
            role_hint: Some("Image #1".to_string()),
            parameters: None,
            extensions: None,
            settings: None,
            summary: true,
            images: None,
        };

        let recipe = Recipe::builder()
            .version("1.0.0")
            .title("Fallback title")
            .description("Fallback description")
            .instructions("Unused")
            .build()
            .unwrap();

        assert_eq!(build_subagent_session_name(&params, &recipe), "Image #1");
    }

    #[tokio::test]
    async fn test_build_subagent_session_metadata_uses_current_parent_turn_id() {
        let params = SubagentParams {
            instructions: Some("处理图片风格统一".to_string()),
            subrecipe: Some("image_pipeline".to_string()),
            role_hint: Some("Image #1".to_string()),
            parameters: None,
            extensions: None,
            settings: None,
            summary: true,
            images: None,
        };
        let recipe = Recipe::builder()
            .version("1.0.0")
            .title("Fallback title")
            .description("Fallback description")
            .instructions("Unused")
            .build()
            .unwrap();
        let task_config = TaskConfig {
            provider: std::sync::Arc::new(
                crate::providers::testprovider::TestProvider::new_replaying(
                    "/tmp/aster-subagent-tool-metadata.json",
                )
                .expect("provider"),
            ),
            parent_session_id: "parent-session-1".to_string(),
            parent_working_dir: PathBuf::from("/tmp/workspace-parent"),
            extensions: Vec::new(),
            max_turns: Some(3),
            turn_context: None,
        };
        let scope = ActionRequiredScope {
            session_id: Some("parent-session-1".to_string()),
            thread_id: Some("thread-1".to_string()),
            turn_id: Some("turn-1".to_string()),
        };

        let metadata = crate::session_context::with_action_scope(scope, async move {
            build_subagent_session_metadata(&task_config, &params, &recipe)
        })
        .await;

        assert_eq!(metadata.created_from_turn_id.as_deref(), Some("turn-1"));
    }

    fn build_test_ollama_provider(model: &str) -> Arc<dyn Provider> {
        Arc::new(
            crate::providers::ollama::OllamaProvider::from_custom_config(
                crate::model::ModelConfig::new_or_fail(model),
                DeclarativeProviderConfig {
                    name: "ollama".to_string(),
                    engine: ProviderEngine::Ollama,
                    display_name: "Test Ollama".to_string(),
                    description: Some("Test-only Ollama provider".to_string()),
                    api_key_env: "IGNORED".to_string(),
                    base_url: "http://localhost:11434".to_string(),
                    models: vec![ModelInfo::new(model, 128_000)],
                    headers: None,
                    timeout_seconds: Some(1),
                    supports_streaming: Some(true),
                },
            )
            .expect("provider"),
        )
    }

    #[tokio::test]
    async fn test_apply_settings_overrides_syncs_explicit_model_into_turn_context() {
        let task_config = TaskConfig {
            provider: build_test_ollama_provider("qwen3"),
            parent_session_id: "parent-session-1".to_string(),
            parent_working_dir: PathBuf::from("/tmp/workspace-parent"),
            extensions: Vec::new(),
            max_turns: Some(3),
            turn_context: Some(TurnContextOverride {
                model: Some("parent-model".to_string()),
                effort: Some("high".to_string()),
                ..TurnContextOverride::default()
            }),
        };
        let params = SubagentParams {
            instructions: Some("分析仓库结构".to_string()),
            subrecipe: None,
            role_hint: None,
            parameters: None,
            extensions: None,
            settings: Some(SubagentSettings {
                provider: None,
                model: Some("qwen3-coder:30b".to_string()),
                temperature: Some(0.2),
            }),
            summary: true,
            images: None,
        };

        let updated = apply_settings_overrides(task_config, &params)
            .await
            .expect("settings should apply");

        assert_eq!(
            updated.provider.get_model_config().model_name,
            "qwen3-coder:30b"
        );
        assert_eq!(updated.provider.get_model_config().temperature, Some(0.2));
        assert_eq!(
            updated
                .turn_context
                .as_ref()
                .and_then(|context| context.model.as_deref()),
            Some("qwen3-coder:30b")
        );
        assert_eq!(
            updated
                .turn_context
                .as_ref()
                .and_then(|context| context.effort.as_deref()),
            Some("high")
        );
    }

    #[tokio::test]
    async fn test_apply_settings_overrides_provider_override_resets_to_provider_default_model() {
        let task_config = TaskConfig {
            provider: build_test_ollama_provider("qwen3-coder:30b"),
            parent_session_id: "parent-session-1".to_string(),
            parent_working_dir: PathBuf::from("/tmp/workspace-parent"),
            extensions: Vec::new(),
            max_turns: Some(3),
            turn_context: Some(TurnContextOverride {
                model: Some("qwen3-coder:30b".to_string()),
                ..TurnContextOverride::default()
            }),
        };
        let params = SubagentParams {
            instructions: Some("分析仓库结构".to_string()),
            subrecipe: None,
            role_hint: None,
            parameters: None,
            extensions: None,
            settings: Some(SubagentSettings {
                provider: Some("ollama".to_string()),
                model: None,
                temperature: None,
            }),
            summary: true,
            images: None,
        };

        let updated = apply_settings_overrides(task_config, &params)
            .await
            .expect("provider override should apply");

        assert_eq!(
            updated.provider.get_model_config().model_name,
            crate::providers::ollama::OLLAMA_DEFAULT_MODEL
        );
        assert_eq!(
            updated
                .turn_context
                .as_ref()
                .and_then(|context| context.model.as_deref()),
            Some(crate::providers::ollama::OLLAMA_DEFAULT_MODEL)
        );
    }
}
