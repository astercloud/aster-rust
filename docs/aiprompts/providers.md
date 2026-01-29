# AI Provider 系统

## 概述

Provider 系统负责与各种 AI 服务进行通信，提供统一的接口抽象。

**核心路径**: `crates/aster/src/providers/`

## 支持的 Provider

| Provider | 模块 | 说明 |
|----------|------|------|
| OpenAI | `openai.rs` | GPT 系列模型 |
| Anthropic | `anthropic.rs` | Claude 系列模型 |
| Google | `google.rs` | Gemini 系列模型 |
| Azure | `azure.rs` | Azure OpenAI |
| Bedrock | `bedrock.rs` | AWS Bedrock |
| GCP Vertex AI | `gcpvertexai.rs` | Google Cloud |
| Ollama | `ollama.rs` | 本地模型 |
| OpenRouter | `openrouter.rs` | 多模型路由 |
| LiteLLM | `litellm.rs` | 统一代理 |
| xAI | `xai.rs` | Grok 模型 |
| Venice | `venice.rs` | Venice AI |
| Databricks | `databricks.rs` | Databricks |
| Snowflake | `snowflake.rs` | Snowflake |
| SageMaker | `sagemaker_tgi.rs` | AWS SageMaker |

## Provider 接口

```rust
// crates/aster/src/providers/base.rs
pub trait Provider: Send + Sync {
    fn get_active_model_name(&self) -> String;
    
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<Tool>,
        system_prompt: &str,
    ) -> Result<CompletionResponse>;
    
    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<Tool>,
        system_prompt: &str,
    ) -> Result<BoxStream<CompletionChunk>>;
}
```

## 创建 Provider

```rust
use crate::providers::{create, create_with_named_model};

// 自动检测
let provider = create().await?;

// 指定模型
let provider = create_with_named_model("claude-3-5-sonnet").await?;

// 刷新自定义 Provider
refresh_custom_providers().await;
```

## 配置示例

### OpenAI

```toml
[provider]
type = "openai"
api_key = "sk-..."
model = "gpt-4o"
base_url = "https://api.openai.com/v1"  # 可选
```

### Anthropic

```toml
[provider]
type = "anthropic"
api_key = "sk-ant-..."
model = "claude-3-5-sonnet-20241022"
```

### Ollama (本地)

```toml
[provider]
type = "ollama"
model = "llama3.2"
base_url = "http://localhost:11434"
```

### Azure OpenAI

```toml
[provider]
type = "azure"
api_key = "..."
deployment = "gpt-4o"
endpoint = "https://xxx.openai.azure.com"
api_version = "2024-02-15-preview"
```

## 重试机制

```rust
// crates/aster/src/providers/retry.rs
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

// 使用重试
let result = retry_operation(|| async {
    provider.complete(messages, tools, system).await
}, RetryConfig::default()).await?;
```

## 错误处理

```rust
// crates/aster/src/providers/errors.rs
pub enum ProviderError {
    AuthenticationError(String),
    RateLimitError { retry_after: Option<Duration> },
    ModelNotFound(String),
    InvalidRequest(String),
    NetworkError(String),
    InternalError(String),
}
```

## 自动检测

```rust
// crates/aster/src/providers/auto_detect.rs
// 按优先级检测可用的 Provider:
// 1. 环境变量 (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)
// 2. 配置文件
// 3. 本地 Ollama
```

## 使用估算

```rust
// crates/aster/src/providers/usage_estimator.rs
pub struct UsageEstimator {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub estimated_cost: f64,
}
```
