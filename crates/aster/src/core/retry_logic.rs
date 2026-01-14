//! 上下文溢出自动恢复逻辑
//!
//! 解析上下文溢出错误，动态调整 max_tokens，自动重试

use regex::Regex;
use std::future::Future;
use thiserror::Error;

/// 最小输出 tokens
const MIN_OUTPUT_TOKENS: u64 = 3000;

/// 保留空间
const RESERVE_BUFFER: u64 = 1000;

/// 上下文溢出错误信息
#[derive(Debug, Clone)]
pub struct ContextOverflowError {
    /// 输入 tokens
    pub input_tokens: u64,
    /// 最大 tokens
    pub max_tokens: u64,
    /// 上下文限制
    pub context_limit: u64,
}

/// 溢出恢复错误
#[derive(Debug, Error)]
pub enum OverflowRecoveryError {
    #[error("Not a context overflow error")]
    NotOverflowError,
    #[error("Cannot recover: input={input_tokens}, limit={context_limit}")]
    CannotRecover {
        input_tokens: u64,
        context_limit: u64,
    },
    #[error("Max retries exceeded after {attempts} attempts")]
    MaxRetriesExceeded { attempts: u32 },
    #[error("Request failed: {0}")]
    RequestFailed(String),
}

/// 解析上下文溢出错误
///
/// 错误格式示例：
/// "input length and `max_tokens` exceed context limit: 195000 + 8192 > 200000"
pub fn parse_context_overflow_error(status: u16, message: &str) -> Option<ContextOverflowError> {
    // 检查是否为 400 错误
    if status != 400 {
        return None;
    }

    // 匹配错误消息模式
    let pattern =
        Regex::new(r"input length and `max_tokens` exceed context limit: (\d+) \+ (\d+) > (\d+)")
            .ok()?;

    let captures = pattern.captures(message)?;

    let input_tokens: u64 = captures.get(1)?.as_str().parse().ok()?;
    let max_tokens: u64 = captures.get(2)?.as_str().parse().ok()?;
    let context_limit: u64 = captures.get(3)?.as_str().parse().ok()?;

    Some(ContextOverflowError {
        input_tokens,
        max_tokens,
        context_limit,
    })
}

/// 计算调整后的 max_tokens
///
/// 策略：
/// 1. 计算可用空间 = contextLimit - inputTokens - reserve
/// 2. 如果可用空间 < MIN_OUTPUT_TOKENS，无法恢复
/// 3. 否则，返回 max(MIN_OUTPUT_TOKENS, available, thinkingTokens + 1)
pub fn calculate_adjusted_max_tokens(
    overflow: &ContextOverflowError,
    max_thinking_tokens: u64,
) -> Option<u64> {
    let available = overflow
        .context_limit
        .saturating_sub(overflow.input_tokens)
        .saturating_sub(RESERVE_BUFFER);

    // 如果可用空间不足最小要求，无法恢复
    if available < MIN_OUTPUT_TOKENS {
        return None;
    }

    // 计算调整后的值
    let thinking = max_thinking_tokens + 1;
    let adjusted = available.max(MIN_OUTPUT_TOKENS).max(thinking);

    Some(adjusted)
}

/// 处理上下文溢出错误
///
/// 返回调整后的 max_tokens，如果无法恢复则返回错误
pub fn handle_context_overflow(
    status: u16,
    message: &str,
    max_thinking_tokens: u64,
) -> Result<u64, OverflowRecoveryError> {
    let overflow = parse_context_overflow_error(status, message)
        .ok_or(OverflowRecoveryError::NotOverflowError)?;

    let adjusted = calculate_adjusted_max_tokens(&overflow, max_thinking_tokens).ok_or(
        OverflowRecoveryError::CannotRecover {
            input_tokens: overflow.input_tokens,
            context_limit: overflow.context_limit,
        },
    )?;

    tracing::warn!(
        "Context overflow detected. Adjusting max_tokens from {} to {}",
        overflow.max_tokens,
        adjusted
    );
    tracing::warn!(
        "  Input: {}, Limit: {}, Available: {}",
        overflow.input_tokens,
        overflow.context_limit,
        adjusted
    );

    Ok(adjusted)
}

/// 溢出恢复选项
#[derive(Debug, Clone)]
pub struct OverflowRecoveryOptions {
    /// 初始 max_tokens
    pub max_tokens: Option<u64>,
    /// 最大思考 tokens
    pub max_thinking_tokens: u64,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for OverflowRecoveryOptions {
    fn default() -> Self {
        Self {
            max_tokens: None,
            max_thinking_tokens: 0,
            max_retries: 3,
        }
    }
}

/// 请求错误信息
pub struct RequestError {
    pub status: u16,
    pub message: String,
}

/// 执行带溢出恢复的请求
pub async fn execute_with_overflow_recovery<T, E, F, Fut>(
    execute_request: F,
    options: OverflowRecoveryOptions,
    mut on_retry: Option<impl FnMut(u32, u64)>,
) -> Result<T, OverflowRecoveryError>
where
    F: Fn(Option<u64>) -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Into<RequestError>,
{
    let mut current_max_tokens = options.max_tokens;

    for attempt in 1..=options.max_retries {
        match execute_request(current_max_tokens).await {
            Ok(result) => return Ok(result),
            Err(error) => {
                let req_error: RequestError = error.into();

                let overflow =
                    match parse_context_overflow_error(req_error.status, &req_error.message) {
                        Some(o) => o,
                        None => {
                            return Err(OverflowRecoveryError::RequestFailed(req_error.message));
                        }
                    };

                if attempt >= options.max_retries {
                    tracing::error!(
                        "Context overflow recovery failed after {} attempts",
                        options.max_retries
                    );
                    return Err(OverflowRecoveryError::MaxRetriesExceeded { attempts: attempt });
                }

                let adjusted =
                    match calculate_adjusted_max_tokens(&overflow, options.max_thinking_tokens) {
                        Some(a) => a,
                        None => {
                            return Err(OverflowRecoveryError::CannotRecover {
                                input_tokens: overflow.input_tokens,
                                context_limit: overflow.context_limit,
                            });
                        }
                    };

                tracing::warn!(
                    "[Retry {}/{}] Context overflow detected. Adjusting max_tokens from {:?} to {}",
                    attempt,
                    options.max_retries,
                    current_max_tokens,
                    adjusted
                );

                current_max_tokens = Some(adjusted);

                if let Some(ref mut callback) = on_retry {
                    callback(attempt, adjusted);
                }
            }
        }
    }

    Err(OverflowRecoveryError::MaxRetriesExceeded {
        attempts: options.max_retries,
    })
}
