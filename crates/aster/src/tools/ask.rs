//! Ask Tool Implementation
//!
//! Provides user interaction capabilities for the agent to ask questions
//! and receive responses from the user.
//!
//! Requirements: 6.1, 6.2, 6.3, 6.4, 6.5

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::tools::base::{PermissionCheckResult, Tool};
use crate::tools::context::{ToolContext, ToolResult};
use crate::tools::error::ToolError;

/// Default timeout for user response (5 minutes)
pub const DEFAULT_ASK_TIMEOUT_SECS: u64 = 300;

/// Callback type for handling user questions
///
/// The callback receives the question and optional options, and returns
/// the user's response as a future.
pub type AskCallback = Arc<
    dyn Fn(String, Option<Vec<String>>) -> Pin<Box<dyn Future<Output = Option<String>> + Send>>
        + Send
        + Sync,
>;

/// A predefined option for the user to select
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskOption {
    /// The value to return if this option is selected
    pub value: String,
    /// Optional display label (defaults to value if not provided)
    pub label: Option<String>,
}

impl AskOption {
    /// Create a new option with just a value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
        }
    }

    /// Create a new option with a value and label
    pub fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
        }
    }


    /// Get the display text for this option
    pub fn display(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.value)
    }
}

/// Result of an ask operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskResult {
    /// The user's response
    pub response: String,
    /// Whether the response was from a predefined option
    pub from_option: bool,
    /// The index of the selected option (if applicable)
    pub option_index: Option<usize>,
}

impl AskResult {
    /// Create a new AskResult from free-form input
    pub fn from_input(response: String) -> Self {
        Self {
            response,
            from_option: false,
            option_index: None,
        }
    }

    /// Create a new AskResult from an option selection
    pub fn from_option(response: String, index: usize) -> Self {
        Self {
            response,
            from_option: true,
            option_index: Some(index),
        }
    }
}

/// Ask tool for user interaction
///
/// Allows the agent to ask questions to the user and receive responses.
/// Supports:
/// - Free-form text questions
/// - Predefined options for selection
/// - Configurable timeout
///
/// Requirements: 6.1, 6.2, 6.3, 6.4, 6.5
pub struct AskTool {
    /// Callback for handling user questions
    callback: Option<AskCallback>,
    /// Default timeout for user response
    timeout: Duration,
}

impl Default for AskTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AskTool {
    /// Create a new AskTool without a callback
    ///
    /// Note: Without a callback, the tool will return an error when executed.
    /// Use `with_callback` to set up the user interaction handler.
    pub fn new() -> Self {
        Self {
            callback: None,
            timeout: Duration::from_secs(DEFAULT_ASK_TIMEOUT_SECS),
        }
    }

    /// Set the callback for handling user questions
    pub fn with_callback(mut self, callback: AskCallback) -> Self {
        self.callback = Some(callback);
        self
    }

    /// Set the default timeout for user responses
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if a callback is configured
    pub fn has_callback(&self) -> bool {
        self.callback.is_some()
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }


    /// Ask a question to the user
    ///
    /// This method invokes the callback with the question and optional options,
    /// and waits for the user's response with timeout.
    ///
    /// # Arguments
    /// * `question` - The question to ask
    /// * `options` - Optional predefined options for the user to select from
    ///
    /// # Returns
    /// * `Ok(AskResult)` - The user's response
    /// * `Err(ToolError)` - If no callback is configured, timeout occurs, or user cancels
    pub async fn ask(
        &self,
        question: &str,
        options: Option<&[AskOption]>,
    ) -> Result<AskResult, ToolError> {
        let callback = self.callback.as_ref().ok_or_else(|| {
            ToolError::execution_failed("No callback configured for user interaction")
        })?;

        // Convert options to string labels for the callback
        let option_labels: Option<Vec<String>> = options.map(|opts| {
            opts.iter().map(|o| o.display().to_string()).collect()
        });

        // Call the callback with timeout
        let response = tokio::time::timeout(
            self.timeout,
            callback(question.to_string(), option_labels.clone()),
        )
        .await
        .map_err(|_| ToolError::timeout(self.timeout))?;

        // Handle the response
        match response {
            Some(response_text) => {
                // Check if response matches an option
                if let Some(opts) = options {
                    for (idx, opt) in opts.iter().enumerate() {
                        if response_text == opt.value || response_text == opt.display() {
                            return Ok(AskResult::from_option(opt.value.clone(), idx));
                        }
                    }
                }
                // Free-form response
                Ok(AskResult::from_input(response_text))
            }
            None => Err(ToolError::execution_failed("User cancelled the interaction")),
        }
    }
}

#[async_trait]
impl Tool for AskTool {
    fn name(&self) -> &str {
        "ask"
    }

    fn description(&self) -> &str {
        "Ask a question to the user and wait for their response. \
         Supports free-form text input or selection from predefined options. \
         Use this tool when you need clarification, confirmation, or user input \
         to proceed with a task."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "description": "The question to ask the user"
                },
                "options": {
                    "type": "array",
                    "description": "Optional predefined options for the user to select from",
                    "items": {
                        "type": "object",
                        "properties": {
                            "value": {
                                "type": "string",
                                "description": "The value to return if this option is selected"
                            },
                            "label": {
                                "type": "string",
                                "description": "Optional display label (defaults to value)"
                            }
                        },
                        "required": ["value"]
                    }
                }
            },
            "required": ["question"]
        })
    }


    async fn execute(
        &self,
        params: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        // Parse question
        let question = params
            .get("question")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("Missing required parameter: question"))?;

        // Parse options if provided
        let options: Option<Vec<AskOption>> = params
            .get("options")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        // Ask the question
        let result = self.ask(question, options.as_deref()).await?;

        // Format the response
        let output = if result.from_option {
            format!(
                "User selected option {}: {}",
                result.option_index.unwrap_or(0) + 1,
                result.response
            )
        } else {
            format!("User response: {}", result.response)
        };

        Ok(ToolResult::success(output)
            .with_metadata("response", serde_json::json!(result.response))
            .with_metadata("from_option", serde_json::json!(result.from_option))
            .with_metadata("option_index", serde_json::json!(result.option_index)))
    }

    async fn check_permissions(
        &self,
        _params: &serde_json::Value,
        _context: &ToolContext,
    ) -> PermissionCheckResult {
        // Ask tool always requires user interaction, so it's always allowed
        // The actual permission is implicit in the user's response
        PermissionCheckResult::allow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Create a mock callback that returns a fixed response
    fn mock_callback(response: Option<String>) -> AskCallback {
        Arc::new(move |_question, _options| {
            let resp = response.clone();
            Box::pin(async move { resp })
        })
    }

    /// Create a mock callback that delays before responding
    fn mock_callback_delayed(response: Option<String>, delay_ms: u64) -> AskCallback {
        Arc::new(move |_question, _options| {
            let resp = response.clone();
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                resp
            })
        })
    }

    #[test]
    fn test_ask_option_new() {
        let opt = AskOption::new("yes");
        assert_eq!(opt.value, "yes");
        assert!(opt.label.is_none());
        assert_eq!(opt.display(), "yes");
    }

    #[test]
    fn test_ask_option_with_label() {
        let opt = AskOption::with_label("y", "Yes, proceed");
        assert_eq!(opt.value, "y");
        assert_eq!(opt.label, Some("Yes, proceed".to_string()));
        assert_eq!(opt.display(), "Yes, proceed");
    }

    #[test]
    fn test_ask_result_from_input() {
        let result = AskResult::from_input("hello".to_string());
        assert_eq!(result.response, "hello");
        assert!(!result.from_option);
        assert!(result.option_index.is_none());
    }

    #[test]
    fn test_ask_result_from_option() {
        let result = AskResult::from_option("yes".to_string(), 0);
        assert_eq!(result.response, "yes");
        assert!(result.from_option);
        assert_eq!(result.option_index, Some(0));
    }


    #[test]
    fn test_ask_tool_new() {
        let tool = AskTool::new();
        assert!(!tool.has_callback());
        assert_eq!(tool.timeout(), Duration::from_secs(DEFAULT_ASK_TIMEOUT_SECS));
    }

    #[test]
    fn test_ask_tool_with_callback() {
        let callback = mock_callback(Some("test".to_string()));
        let tool = AskTool::new().with_callback(callback);
        assert!(tool.has_callback());
    }

    #[test]
    fn test_ask_tool_with_timeout() {
        let tool = AskTool::new().with_timeout(Duration::from_secs(60));
        assert_eq!(tool.timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_ask_tool_default() {
        let tool = AskTool::default();
        assert!(!tool.has_callback());
        assert_eq!(tool.timeout(), Duration::from_secs(DEFAULT_ASK_TIMEOUT_SECS));
    }

    #[tokio::test]
    async fn test_ask_without_callback() {
        let tool = AskTool::new();
        let result = tool.ask("What is your name?", None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::ExecutionFailed(_)));
    }

    #[tokio::test]
    async fn test_ask_free_form_response() {
        let callback = mock_callback(Some("John".to_string()));
        let tool = AskTool::new().with_callback(callback);

        let result = tool.ask("What is your name?", None).await.unwrap();
        assert_eq!(result.response, "John");
        assert!(!result.from_option);
        assert!(result.option_index.is_none());
    }

    #[tokio::test]
    async fn test_ask_with_options_select_by_value() {
        let callback = mock_callback(Some("yes".to_string()));
        let tool = AskTool::new().with_callback(callback);

        let options = vec![
            AskOption::new("yes"),
            AskOption::new("no"),
        ];

        let result = tool.ask("Continue?", Some(&options)).await.unwrap();
        assert_eq!(result.response, "yes");
        assert!(result.from_option);
        assert_eq!(result.option_index, Some(0));
    }

    #[tokio::test]
    async fn test_ask_with_options_select_by_label() {
        let callback = mock_callback(Some("Yes, proceed".to_string()));
        let tool = AskTool::new().with_callback(callback);

        let options = vec![
            AskOption::with_label("y", "Yes, proceed"),
            AskOption::with_label("n", "No, cancel"),
        ];

        let result = tool.ask("Continue?", Some(&options)).await.unwrap();
        assert_eq!(result.response, "y");
        assert!(result.from_option);
        assert_eq!(result.option_index, Some(0));
    }

    #[tokio::test]
    async fn test_ask_with_options_free_form() {
        let callback = mock_callback(Some("maybe".to_string()));
        let tool = AskTool::new().with_callback(callback);

        let options = vec![
            AskOption::new("yes"),
            AskOption::new("no"),
        ];

        let result = tool.ask("Continue?", Some(&options)).await.unwrap();
        assert_eq!(result.response, "maybe");
        assert!(!result.from_option);
        assert!(result.option_index.is_none());
    }

    #[tokio::test]
    async fn test_ask_user_cancels() {
        let callback = mock_callback(None);
        let tool = AskTool::new().with_callback(callback);

        let result = tool.ask("What is your name?", None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::ExecutionFailed(_)));
    }

    #[tokio::test]
    async fn test_ask_timeout() {
        let callback = mock_callback_delayed(Some("response".to_string()), 200);
        let tool = AskTool::new()
            .with_callback(callback)
            .with_timeout(Duration::from_millis(50));

        let result = tool.ask("What is your name?", None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::Timeout(_)));
    }

    #[tokio::test]
    async fn test_ask_tool_trait_name() {
        let tool = AskTool::new();
        assert_eq!(tool.name(), "ask");
    }

    #[tokio::test]
    async fn test_ask_tool_trait_description() {
        let tool = AskTool::new();
        assert!(tool.description().contains("Ask a question"));
    }

    #[tokio::test]
    async fn test_ask_tool_trait_input_schema() {
        let tool = AskTool::new();
        let schema = tool.input_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["question"].is_object());
        assert!(schema["properties"]["options"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&serde_json::json!("question")));
    }

    #[tokio::test]
    async fn test_ask_tool_execute_success() {
        let callback = mock_callback(Some("John".to_string()));
        let tool = AskTool::new().with_callback(callback);
        let context = ToolContext::new(PathBuf::from("/tmp"));

        let params = serde_json::json!({
            "question": "What is your name?"
        });

        let result = tool.execute(params, &context).await.unwrap();
        assert!(result.is_success());
        assert!(result.output.unwrap().contains("John"));
        assert_eq!(result.metadata.get("response"), Some(&serde_json::json!("John")));
        assert_eq!(result.metadata.get("from_option"), Some(&serde_json::json!(false)));
    }

    #[tokio::test]
    async fn test_ask_tool_execute_with_options() {
        let callback = mock_callback(Some("yes".to_string()));
        let tool = AskTool::new().with_callback(callback);
        let context = ToolContext::new(PathBuf::from("/tmp"));

        let params = serde_json::json!({
            "question": "Continue?",
            "options": [
                { "value": "yes", "label": "Yes" },
                { "value": "no", "label": "No" }
            ]
        });

        let result = tool.execute(params, &context).await.unwrap();
        assert!(result.is_success());
        assert!(result.output.unwrap().contains("selected option"));
        assert_eq!(result.metadata.get("from_option"), Some(&serde_json::json!(true)));
        assert_eq!(result.metadata.get("option_index"), Some(&serde_json::json!(0)));
    }

    #[tokio::test]
    async fn test_ask_tool_execute_missing_question() {
        let callback = mock_callback(Some("test".to_string()));
        let tool = AskTool::new().with_callback(callback);
        let context = ToolContext::new(PathBuf::from("/tmp"));

        let params = serde_json::json!({});

        let result = tool.execute(params, &context).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidParams(_)));
    }

    #[tokio::test]
    async fn test_ask_tool_check_permissions() {
        let tool = AskTool::new();
        let context = ToolContext::new(PathBuf::from("/tmp"));
        let params = serde_json::json!({"question": "test"});

        let result = tool.check_permissions(&params, &context).await;
        assert!(result.is_allowed());
    }

    #[test]
    fn test_ask_option_serialization() {
        let opt = AskOption::with_label("y", "Yes");
        let json = serde_json::to_string(&opt).unwrap();
        let deserialized: AskOption = serde_json::from_str(&json).unwrap();

        assert_eq!(opt.value, deserialized.value);
        assert_eq!(opt.label, deserialized.label);
    }

    #[test]
    fn test_ask_result_serialization() {
        let result = AskResult::from_option("yes".to_string(), 0);
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AskResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.response, deserialized.response);
        assert_eq!(result.from_option, deserialized.from_option);
        assert_eq!(result.option_index, deserialized.option_index);
    }
}
