//! Ask Tool Implementation
//!
//! Provides user interaction capabilities for the agent to ask questions
//! and receive responses from the user.
//!
//! Requirements: 6.1, 6.2, 6.3, 6.4, 6.5

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::tools::base::{PermissionCheckResult, Tool};
use crate::tools::context::{ToolContext, ToolResult};
use crate::tools::error::ToolError;

/// Default timeout for user response (5 minutes)
pub const DEFAULT_ASK_TIMEOUT_SECS: u64 = 300;

/// A structured question payload aligned with modern ask_user style prompts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AskQuestion {
    /// The complete question text shown to the user
    pub question: String,
    /// Optional short chip/header label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// Optional predefined choices
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<AskOption>,
    /// Whether multiple options can be selected
    #[serde(default, alias = "multi_select")]
    pub multi_select: bool,
}

impl AskQuestion {
    /// Create a new free-form question
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            header: None,
            options: Vec::new(),
            multi_select: false,
        }
    }

    /// Create a question with predefined choices
    pub fn with_options(question: impl Into<String>, options: Vec<AskOption>) -> Self {
        Self {
            question: question.into(),
            header: None,
            options,
            multi_select: false,
        }
    }

    fn validate(&self) -> Result<(), ToolError> {
        if self.question.trim().is_empty() {
            return Err(ToolError::invalid_params(
                "Question text cannot be empty".to_string(),
            ));
        }

        if self.options.len() > 4 {
            return Err(ToolError::invalid_params(
                "Question options cannot exceed 4 choices".to_string(),
            ));
        }

        for option in &self.options {
            option.validate()?;
        }

        Ok(())
    }
}

/// A modern ask request that may contain one or more related questions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AskRequest {
    pub questions: Vec<AskQuestion>,
}

impl AskRequest {
    /// Build a legacy single-question request.
    pub fn from_legacy(question: impl Into<String>, options: Vec<AskOption>) -> Self {
        Self {
            questions: vec![AskQuestion::with_options(question, options)],
        }
    }

    fn validate(&self) -> Result<(), ToolError> {
        if self.questions.is_empty() {
            return Err(ToolError::invalid_params(
                "At least one question is required".to_string(),
            ));
        }

        if self.questions.len() > 4 {
            return Err(ToolError::invalid_params(
                "Questions cannot exceed 4 entries".to_string(),
            ));
        }

        for question in &self.questions {
            question.validate()?;
        }

        Ok(())
    }
}

/// Callback type for handling user questions
///
/// The callback receives the normalized ask request and returns the user's
/// structured response as JSON.
pub type AskCallback =
    Arc<dyn Fn(AskRequest) -> Pin<Box<dyn Future<Output = Option<Value>> + Send>> + Send + Sync>;

/// A predefined option for the user to select
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AskOption {
    /// The value to return if this option is selected
    pub value: String,
    /// Optional display label (defaults to value if not provided)
    pub label: Option<String>,
    /// Optional explanation for the option
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional preview payload for richer UIs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

impl AskOption {
    /// Create a new option with just a value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: None,
            preview: None,
        }
    }

    /// Create a new option with a value and label
    pub fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
            description: None,
            preview: None,
        }
    }

    /// Get the display text for this option
    pub fn display(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.value)
    }

    fn validate(&self) -> Result<(), ToolError> {
        if self.value.trim().is_empty() && self.display().trim().is_empty() {
            return Err(ToolError::invalid_params(
                "Option value/label cannot both be empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Result of an ask operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskResult {
    /// Raw user data returned from the ask bridge
    pub response: Value,
    /// Normalized answers keyed by question text
    pub answers: BTreeMap<String, String>,
    /// Whether the response was from a predefined option
    pub from_option: bool,
    /// The index of the selected option (if applicable)
    pub option_index: Option<usize>,
}

impl AskResult {
    fn new(
        response: Value,
        answers: BTreeMap<String, String>,
        from_option: bool,
        option_index: Option<usize>,
    ) -> Self {
        Self {
            response,
            answers,
            from_option,
            option_index,
        }
    }

    pub fn primary_response(&self) -> Option<&str> {
        self.answers.values().next().map(String::as_str)
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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum AskOptionInput {
    String(String),
    Object(AskOptionObject),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AskOptionObject {
    value: Option<String>,
    label: Option<String>,
    description: Option<String>,
    preview: Option<String>,
}

impl TryFrom<AskOptionInput> for AskOption {
    type Error = ToolError;

    fn try_from(value: AskOptionInput) -> Result<Self, Self::Error> {
        match value {
            AskOptionInput::String(value) => {
                let option = AskOption::new(value);
                option.validate()?;
                Ok(option)
            }
            AskOptionInput::Object(object) => {
                let value = object
                    .value
                    .or_else(|| object.label.clone())
                    .unwrap_or_default();
                let option = AskOption {
                    value,
                    label: object.label,
                    description: object.description,
                    preview: object.preview,
                };
                option.validate()?;
                Ok(option)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AskQuestionInput {
    question: String,
    header: Option<String>,
    options: Option<Vec<AskOptionInput>>,
    #[serde(default, alias = "multi_select")]
    multi_select: bool,
}

impl TryFrom<AskQuestionInput> for AskQuestion {
    type Error = ToolError;

    fn try_from(value: AskQuestionInput) -> Result<Self, Self::Error> {
        let options = value
            .options
            .unwrap_or_default()
            .into_iter()
            .map(AskOption::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let question = AskQuestion {
            question: value.question,
            header: value.header,
            options,
            multi_select: value.multi_select,
        };
        question.validate()?;
        Ok(question)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AskToolInput {
    question: Option<String>,
    header: Option<String>,
    options: Option<Vec<AskOptionInput>>,
    #[serde(default, alias = "multi_select")]
    multi_select: bool,
    questions: Option<Vec<AskQuestionInput>>,
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

    fn parse_request(&self, params: Value) -> Result<AskRequest, ToolError> {
        let input: AskToolInput = serde_json::from_value(params)
            .map_err(|e| ToolError::invalid_params(format!("Failed to parse ask input: {e}")))?;

        let request = if let Some(questions) = input.questions {
            AskRequest {
                questions: questions
                    .into_iter()
                    .map(AskQuestion::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            }
        } else {
            let question = input.question.ok_or_else(|| {
                ToolError::invalid_params(
                    "Missing required parameter: question or questions".to_string(),
                )
            })?;
            let options = input
                .options
                .unwrap_or_default()
                .into_iter()
                .map(AskOption::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            AskRequest {
                questions: vec![AskQuestion {
                    question,
                    header: input.header,
                    options,
                    multi_select: input.multi_select,
                }],
            }
        };

        request.validate()?;
        Ok(request)
    }

    fn normalize_answer_value(value: &Value) -> Option<String> {
        match value {
            Value::String(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            Value::Array(items) => {
                let parts = items
                    .iter()
                    .filter_map(Self::normalize_answer_value)
                    .collect::<Vec<_>>();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(", "))
                }
            }
            Value::Number(number) => Some(number.to_string()),
            Value::Bool(value) => Some(value.to_string()),
            _ => None,
        }
    }

    fn question_field_key(question: &AskQuestion, index: usize, total: usize) -> String {
        if total == 1 {
            return "answer".to_string();
        }

        if let Some(header) = question.header.as_deref() {
            let normalized = header
                .trim()
                .to_lowercase()
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                        ch
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
                .trim_matches('_')
                .to_string();

            if !normalized.is_empty() {
                return normalized;
            }
        }

        format!("question_{}", index + 1)
    }

    fn resolve_answers(&self, request: &AskRequest, response: &Value) -> BTreeMap<String, String> {
        let mut answers = BTreeMap::new();
        let total = request.questions.len();

        match response {
            Value::String(_) | Value::Array(_) | Value::Number(_) | Value::Bool(_) => {
                if let Some(first_question) = request.questions.first() {
                    if let Some(answer) = Self::normalize_answer_value(response) {
                        answers.insert(first_question.question.clone(), answer);
                    }
                }
                return answers;
            }
            Value::Object(map) => {
                if let Some(Value::Object(answer_map)) = map.get("answers") {
                    for question in &request.questions {
                        if let Some(value) = answer_map.get(&question.question) {
                            if let Some(answer) = Self::normalize_answer_value(value) {
                                answers.insert(question.question.clone(), answer);
                            }
                        }
                    }
                }

                for (index, question) in request.questions.iter().enumerate() {
                    if answers.contains_key(&question.question) {
                        continue;
                    }

                    for key in [
                        question.question.clone(),
                        question.header.clone().unwrap_or_default(),
                        Self::question_field_key(question, index, total),
                    ] {
                        if key.is_empty() {
                            continue;
                        }

                        if let Some(value) = map.get(&key) {
                            if let Some(answer) = Self::normalize_answer_value(value) {
                                answers.insert(question.question.clone(), answer);
                                break;
                            }
                        }
                    }
                }

                if answers.is_empty() && total == 1 {
                    let candidate = map
                        .get("other")
                        .or_else(|| map.get("answer"))
                        .and_then(Self::normalize_answer_value);
                    if let (Some(question), Some(answer)) = (request.questions.first(), candidate) {
                        answers.insert(question.question.clone(), answer);
                    }
                }

                return answers;
            }
            _ => {}
        }

        answers
    }

    fn resolve_option_match(question: &AskQuestion, answer: Option<&str>) -> (bool, Option<usize>) {
        let Some(answer) = answer.map(str::trim).filter(|value| !value.is_empty()) else {
            return (false, None);
        };

        for (index, option) in question.options.iter().enumerate() {
            if answer == option.value || answer == option.display() {
                return (true, Some(index));
            }
        }

        (false, None)
    }

    fn normalize_result(
        &self,
        request: &AskRequest,
        response: Value,
    ) -> Result<AskResult, ToolError> {
        let mut answers = self.resolve_answers(request, &response);
        if answers.is_empty() {
            return Err(ToolError::execution_failed(
                "User response was empty or could not be normalized",
            ));
        }

        let (from_option, option_index) = if request.questions.len() == 1 {
            let question = &request.questions[0];
            let answer = answers.get(&question.question).map(String::as_str);
            let (from_option, option_index) = Self::resolve_option_match(question, answer);
            if let Some(index) = option_index {
                if let Some(option) = question.options.get(index) {
                    answers.insert(question.question.clone(), option.value.clone());
                }
            }
            (from_option, option_index)
        } else {
            (false, None)
        };

        Ok(AskResult::new(response, answers, from_option, option_index))
    }

    /// Ask one or more questions to the user and wait for their response.
    pub async fn ask(&self, request: &AskRequest) -> Result<AskResult, ToolError> {
        let callback = self.callback.as_ref().ok_or_else(|| {
            ToolError::execution_failed("No callback configured for user interaction")
        })?;

        // Call the callback with timeout
        let response = tokio::time::timeout(self.timeout, callback(request.clone()))
            .await
            .map_err(|_| ToolError::timeout(self.timeout))?;

        // Handle the response
        match response {
            Some(response_data) => self.normalize_result(request, response_data),
            None => Err(ToolError::execution_failed(
                "User cancelled the interaction",
            )),
        }
    }
}

#[async_trait]
impl Tool for AskTool {
    fn name(&self) -> &str {
        "ask"
    }

    fn description(&self) -> &str {
        "Ask one or more focused questions to the user and wait for their response. \
         Supports both the legacy `question/options` format and the modern \
         `questions` array format with headers, descriptions, and multi-select \
         choices. Use this tool when you need clarification, confirmation, or \
         user input to proceed with a task."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "description": "Legacy single-question field. Use `questions` for the richer modern protocol."
                },
                "header": {
                    "type": "string",
                    "description": "Optional short chip/header for the legacy single-question format"
                },
                "options": {
                    "type": "array",
                    "description": "Optional predefined options for the legacy single-question format",
                    "items": {
                        "oneOf": [
                            {
                                "type": "string"
                            },
                            {
                                "type": "object",
                                "properties": {
                                    "value": {
                                        "type": "string",
                                        "description": "The value to return if this option is selected"
                                    },
                                    "label": {
                                        "type": "string",
                                        "description": "Optional display label (defaults to value)"
                                    },
                                    "description": {
                                        "type": "string",
                                        "description": "Optional explanation of this choice"
                                    },
                                    "preview": {
                                        "type": "string",
                                        "description": "Optional preview payload for richer UIs"
                                    }
                                }
                            }
                        ]
                    }
                },
                "multiSelect": {
                    "type": "boolean",
                    "description": "Whether the legacy single-question format allows selecting multiple options"
                },
                "questions": {
                    "type": "array",
                    "description": "Modern ask protocol. Prefer this over the legacy single-question fields.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The full question text shown to the user"
                            },
                            "header": {
                                "type": "string",
                                "description": "Optional short chip/header label"
                            },
                            "options": {
                                "type": "array",
                                "items": {
                                    "oneOf": [
                                        {
                                            "type": "string"
                                        },
                                        {
                                            "type": "object",
                                            "properties": {
                                                "value": {
                                                    "type": "string"
                                                },
                                                "label": {
                                                    "type": "string"
                                                },
                                                "description": {
                                                    "type": "string"
                                                },
                                                "preview": {
                                                    "type": "string"
                                                }
                                            }
                                        }
                                    ]
                                }
                            },
                            "multiSelect": {
                                "type": "boolean",
                                "description": "Whether multiple options can be selected"
                            }
                        },
                        "required": ["question"]
                    }
                }
            },
            "oneOf": [
                { "required": ["question"] },
                { "required": ["questions"] }
            ]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let request = self.parse_request(params)?;
        let result = self.ask(&request).await?;
        let primary_response = result.primary_response().unwrap_or_default().to_string();

        // Format the response
        let output = if result.answers.len() > 1 {
            let lines = result
                .answers
                .iter()
                .map(|(question, answer)| format!("- {question}: {answer}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("User answered multiple questions:\n{lines}")
        } else if result.from_option {
            format!(
                "User selected option {}: {}",
                result.option_index.unwrap_or(0) + 1,
                primary_response
            )
        } else {
            format!("User response: {}", primary_response)
        };

        Ok(ToolResult::success(output)
            .with_metadata("response", serde_json::json!(primary_response))
            .with_metadata("answers", serde_json::json!(result.answers))
            .with_metadata("raw_response", result.response.clone())
            .with_metadata("question_count", serde_json::json!(request.questions.len()))
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
    fn mock_callback(response: Option<Value>) -> AskCallback {
        Arc::new(move |_request| {
            let resp = response.clone();
            Box::pin(async move { resp })
        })
    }

    /// Create a mock callback that delays before responding
    fn mock_callback_delayed(response: Option<Value>, delay_ms: u64) -> AskCallback {
        Arc::new(move |_request| {
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
    fn test_ask_request_from_legacy() {
        let request = AskRequest::from_legacy("Continue?", vec![AskOption::new("yes")]);
        assert_eq!(request.questions.len(), 1);
        assert_eq!(request.questions[0].question, "Continue?");
        assert_eq!(request.questions[0].options[0].value, "yes");
    }

    #[test]
    fn test_ask_result_primary_response() {
        let mut answers = BTreeMap::new();
        answers.insert("Question".to_string(), "hello".to_string());
        let result = AskResult::new(serde_json::json!("hello"), answers, false, None);
        assert_eq!(result.primary_response(), Some("hello"));
        assert!(!result.from_option);
        assert!(result.option_index.is_none());
    }

    #[test]
    fn test_ask_result_option_metadata() {
        let mut answers = BTreeMap::new();
        answers.insert("Continue?".to_string(), "yes".to_string());
        let result = AskResult::new(serde_json::json!("yes"), answers, true, Some(0));
        assert_eq!(result.primary_response(), Some("yes"));
        assert!(result.from_option);
        assert_eq!(result.option_index, Some(0));
    }

    #[test]
    fn test_ask_tool_new() {
        let tool = AskTool::new();
        assert!(!tool.has_callback());
        assert_eq!(
            tool.timeout(),
            Duration::from_secs(DEFAULT_ASK_TIMEOUT_SECS)
        );
    }

    #[test]
    fn test_ask_tool_with_callback() {
        let callback = mock_callback(Some(serde_json::json!("test")));
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
        assert_eq!(
            tool.timeout(),
            Duration::from_secs(DEFAULT_ASK_TIMEOUT_SECS)
        );
    }

    #[tokio::test]
    async fn test_ask_without_callback() {
        let tool = AskTool::new();
        let result = tool
            .ask(&AskRequest::from_legacy("What is your name?", vec![]))
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::ExecutionFailed(_)));
    }

    #[tokio::test]
    async fn test_ask_free_form_response() {
        let callback = mock_callback(Some(serde_json::json!("John")));
        let tool = AskTool::new().with_callback(callback);

        let result = tool
            .ask(&AskRequest::from_legacy("What is your name?", vec![]))
            .await
            .unwrap();
        assert_eq!(result.primary_response(), Some("John"));
        assert_eq!(
            result.answers.get("What is your name?").map(String::as_str),
            Some("John")
        );
        assert!(!result.from_option);
        assert!(result.option_index.is_none());
    }

    #[tokio::test]
    async fn test_ask_with_options_select_by_value() {
        let callback = mock_callback(Some(serde_json::json!("yes")));
        let tool = AskTool::new().with_callback(callback);

        let options = vec![AskOption::new("yes"), AskOption::new("no")];

        let result = tool
            .ask(&AskRequest::from_legacy("Continue?", options))
            .await
            .unwrap();
        assert_eq!(result.primary_response(), Some("yes"));
        assert!(result.from_option);
        assert_eq!(result.option_index, Some(0));
    }

    #[tokio::test]
    async fn test_ask_with_options_select_by_label() {
        let callback = mock_callback(Some(serde_json::json!("Yes, proceed")));
        let tool = AskTool::new().with_callback(callback);

        let options = vec![
            AskOption::with_label("y", "Yes, proceed"),
            AskOption::with_label("n", "No, cancel"),
        ];

        let result = tool
            .ask(&AskRequest::from_legacy("Continue?", options))
            .await
            .unwrap();
        assert_eq!(result.primary_response(), Some("y"));
        assert!(result.from_option);
        assert_eq!(result.option_index, Some(0));
    }

    #[tokio::test]
    async fn test_ask_with_options_free_form() {
        let callback = mock_callback(Some(serde_json::json!("maybe")));
        let tool = AskTool::new().with_callback(callback);

        let options = vec![AskOption::new("yes"), AskOption::new("no")];

        let result = tool
            .ask(&AskRequest::from_legacy("Continue?", options))
            .await
            .unwrap();
        assert_eq!(result.primary_response(), Some("maybe"));
        assert!(!result.from_option);
        assert!(result.option_index.is_none());
    }

    #[tokio::test]
    async fn test_ask_with_modern_questions_payload() {
        let callback = mock_callback(Some(serde_json::json!({
            "answers": {
                "Choose a theme": "Cyber green"
            }
        })));
        let tool = AskTool::new().with_callback(callback);

        let request = AskRequest {
            questions: vec![AskQuestion {
                question: "Choose a theme".to_string(),
                header: Some("Theme".to_string()),
                options: vec![
                    AskOption {
                        value: "Network matrix".to_string(),
                        label: Some("Network matrix".to_string()),
                        description: Some("Dense and technical".to_string()),
                        preview: None,
                    },
                    AskOption {
                        value: "Cyber green".to_string(),
                        label: Some("Cyber green".to_string()),
                        description: Some("Bright and futuristic".to_string()),
                        preview: None,
                    },
                ],
                multi_select: false,
            }],
        };

        let result = tool.ask(&request).await.unwrap();
        assert_eq!(result.primary_response(), Some("Cyber green"));
        assert_eq!(
            result.answers.get("Choose a theme").map(String::as_str),
            Some("Cyber green")
        );
    }

    #[tokio::test]
    async fn test_ask_with_multiple_questions_payload() {
        let callback = mock_callback(Some(serde_json::json!({
            "answers": {
                "Primary goal?": "Ship quickly",
                "Need tests?": "Yes"
            }
        })));
        let tool = AskTool::new().with_callback(callback);

        let request = AskRequest {
            questions: vec![
                AskQuestion::new("Primary goal?"),
                AskQuestion::new("Need tests?"),
            ],
        };

        let result = tool.ask(&request).await.unwrap();
        assert_eq!(result.answers.len(), 2);
        assert_eq!(
            result.answers.get("Primary goal?").map(String::as_str),
            Some("Ship quickly")
        );
        assert_eq!(
            result.answers.get("Need tests?").map(String::as_str),
            Some("Yes")
        );
    }

    #[tokio::test]
    async fn test_ask_user_cancels() {
        let callback = mock_callback(None);
        let tool = AskTool::new().with_callback(callback);

        let result = tool
            .ask(&AskRequest::from_legacy("What is your name?", vec![]))
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::ExecutionFailed(_)));
    }

    #[tokio::test]
    async fn test_ask_timeout() {
        let callback = mock_callback_delayed(Some(serde_json::json!("response")), 200);
        let tool = AskTool::new()
            .with_callback(callback)
            .with_timeout(Duration::from_millis(50));

        let result = tool
            .ask(&AskRequest::from_legacy("What is your name?", vec![]))
            .await;
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
        assert!(tool
            .description()
            .contains("Ask one or more focused questions"));
    }

    #[tokio::test]
    async fn test_ask_tool_trait_input_schema() {
        let tool = AskTool::new();
        let schema = tool.input_schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["question"].is_object());
        assert!(schema["properties"]["questions"].is_object());
        assert!(schema["properties"]["options"].is_object());
        assert!(schema["oneOf"].is_array());
    }

    #[tokio::test]
    async fn test_ask_tool_execute_success() {
        let callback = mock_callback(Some(serde_json::json!("John")));
        let tool = AskTool::new().with_callback(callback);
        let context = ToolContext::new(PathBuf::from("/tmp"));

        let params = serde_json::json!({
            "question": "What is your name?"
        });

        let result = tool.execute(params, &context).await.unwrap();
        assert!(result.is_success());
        assert!(result.output.unwrap().contains("John"));
        assert_eq!(
            result.metadata.get("response"),
            Some(&serde_json::json!("John"))
        );
        assert_eq!(
            result.metadata.get("question_count"),
            Some(&serde_json::json!(1))
        );
        assert_eq!(
            result.metadata.get("from_option"),
            Some(&serde_json::json!(false))
        );
    }

    #[tokio::test]
    async fn test_ask_tool_execute_with_options() {
        let callback = mock_callback(Some(serde_json::json!("yes")));
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
        assert_eq!(
            result.metadata.get("from_option"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.metadata.get("option_index"),
            Some(&serde_json::json!(0))
        );
    }

    #[tokio::test]
    async fn test_ask_tool_execute_with_modern_questions() {
        let callback = mock_callback(Some(serde_json::json!({
            "answers": {
                "Which mode?": "Fast"
            }
        })));
        let tool = AskTool::new().with_callback(callback);
        let context = ToolContext::new(PathBuf::from("/tmp"));

        let params = serde_json::json!({
            "questions": [
                {
                    "question": "Which mode?",
                    "header": "Mode",
                    "options": [
                        { "label": "Fast", "description": "Optimized for speed" },
                        { "label": "Thorough", "description": "Optimized for depth" }
                    ]
                }
            ]
        });

        let result = tool.execute(params, &context).await.unwrap();
        assert!(result.is_success());
        assert_eq!(
            result.metadata.get("answers"),
            Some(&serde_json::json!({
                "Which mode?": "Fast"
            }))
        );
    }

    #[tokio::test]
    async fn test_ask_tool_execute_missing_question() {
        let callback = mock_callback(Some(serde_json::json!("test")));
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
        let mut answers = BTreeMap::new();
        answers.insert("Continue?".to_string(), "yes".to_string());
        let result = AskResult::new(serde_json::json!("yes"), answers, true, Some(0));
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AskResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.response, deserialized.response);
        assert_eq!(result.answers, deserialized.answers);
        assert_eq!(result.from_option, deserialized.from_option);
        assert_eq!(result.option_index, deserialized.option_index);
    }
}
