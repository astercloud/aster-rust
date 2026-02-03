//! A2UI 标准函数定义
//!
//! 对应 A2UI 规范中 standard_catalog.json 的 functions 部分

use serde::{Deserialize, Serialize};

use crate::common::{DynamicBoolean, DynamicNumber, DynamicString, FunctionCall, ReturnType};

// ============================================================================
// 验证函数
// ============================================================================

/// 创建 required 函数调用
pub fn required(value: DynamicString) -> FunctionCall {
    FunctionCall {
        call: "required".to_string(),
        args: Some(serde_json::json!({ "value": value })),
        return_type: Some(ReturnType::Boolean),
    }
}

/// 创建 regex 函数调用
pub fn regex(value: DynamicString, pattern: &str) -> FunctionCall {
    FunctionCall {
        call: "regex".to_string(),
        args: Some(serde_json::json!({
            "value": value,
            "pattern": pattern
        })),
        return_type: Some(ReturnType::Boolean),
    }
}

/// 创建 length 函数调用
pub fn length(value: DynamicString, min: Option<u32>, max: Option<u32>) -> FunctionCall {
    let mut args = serde_json::json!({ "value": value });
    if let Some(min) = min {
        args["min"] = serde_json::json!(min);
    }
    if let Some(max) = max {
        args["max"] = serde_json::json!(max);
    }
    FunctionCall {
        call: "length".to_string(),
        args: Some(args),
        return_type: Some(ReturnType::Boolean),
    }
}

/// 创建 numeric 函数调用
pub fn numeric(value: DynamicNumber, min: Option<f64>, max: Option<f64>) -> FunctionCall {
    let mut args = serde_json::json!({ "value": value });
    if let Some(min) = min {
        args["min"] = serde_json::json!(min);
    }
    if let Some(max) = max {
        args["max"] = serde_json::json!(max);
    }
    FunctionCall {
        call: "numeric".to_string(),
        args: Some(args),
        return_type: Some(ReturnType::Boolean),
    }
}

/// 创建 email 函数调用
pub fn email(value: DynamicString) -> FunctionCall {
    FunctionCall {
        call: "email".to_string(),
        args: Some(serde_json::json!({ "value": value })),
        return_type: Some(ReturnType::Boolean),
    }
}

// ============================================================================
// 格式化函数
// ============================================================================

/// 创建 formatString 函数调用
pub fn format_string(value: DynamicString) -> FunctionCall {
    FunctionCall {
        call: "formatString".to_string(),
        args: Some(serde_json::json!({ "value": value })),
        return_type: Some(ReturnType::String),
    }
}

/// 创建 formatNumber 函数调用
pub fn format_number(
    value: DynamicNumber,
    decimals: Option<u32>,
    grouping: Option<bool>,
) -> FunctionCall {
    let mut args = serde_json::json!({ "value": value });
    if let Some(decimals) = decimals {
        args["decimals"] = serde_json::json!(decimals);
    }
    if let Some(grouping) = grouping {
        args["grouping"] = serde_json::json!(grouping);
    }
    FunctionCall {
        call: "formatNumber".to_string(),
        args: Some(args),
        return_type: Some(ReturnType::String),
    }
}

/// 创建 formatCurrency 函数调用
pub fn format_currency(
    value: DynamicNumber,
    currency: &str,
    decimals: Option<u32>,
    grouping: Option<bool>,
) -> FunctionCall {
    let mut args = serde_json::json!({
        "value": value,
        "currency": currency
    });
    if let Some(decimals) = decimals {
        args["decimals"] = serde_json::json!(decimals);
    }
    if let Some(grouping) = grouping {
        args["grouping"] = serde_json::json!(grouping);
    }
    FunctionCall {
        call: "formatCurrency".to_string(),
        args: Some(args),
        return_type: Some(ReturnType::String),
    }
}

/// 创建 formatDate 函数调用
///
/// format 参数使用 Unicode TR35 日期模式：
/// - 年: 'yy' (26), 'yyyy' (2026)
/// - 月: 'M' (1), 'MM' (01), 'MMM' (Jan), 'MMMM' (January)
/// - 日: 'd' (1), 'dd' (01), 'E' (Tue), 'EEEE' (Tuesday)
/// - 时(12h): 'h' (1-12), 'hh' (01-12)
/// - 时(24h): 'H' (0-23), 'HH' (00-23)
/// - 分: 'mm' (00-59)
/// - 秒: 'ss' (00-59)
/// - 上下午: 'a' (AM/PM)
pub fn format_date(value: DynamicString, format: &str) -> FunctionCall {
    FunctionCall {
        call: "formatDate".to_string(),
        args: Some(serde_json::json!({
            "value": value,
            "format": format
        })),
        return_type: Some(ReturnType::String),
    }
}

/// 复数形式参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluralizeArgs {
    pub zero: Option<DynamicString>,
    pub one: Option<DynamicString>,
    pub two: Option<DynamicString>,
    pub few: Option<DynamicString>,
    pub many: Option<DynamicString>,
    pub other: DynamicString,
}

impl PluralizeArgs {
    /// 创建新的复数形式参数
    pub fn new(other: impl Into<DynamicString>) -> Self {
        Self {
            zero: None,
            one: None,
            two: None,
            few: None,
            many: None,
            other: other.into(),
        }
    }
}

/// 创建 pluralize 函数调用
pub fn pluralize(value: DynamicNumber, args: PluralizeArgs) -> FunctionCall {
    let mut json_args = serde_json::json!({
        "value": value,
        "other": args.other
    });
    if let Some(zero) = args.zero {
        json_args["zero"] = serde_json::json!(zero);
    }
    if let Some(one) = args.one {
        json_args["one"] = serde_json::json!(one);
    }
    if let Some(two) = args.two {
        json_args["two"] = serde_json::json!(two);
    }
    if let Some(few) = args.few {
        json_args["few"] = serde_json::json!(few);
    }
    if let Some(many) = args.many {
        json_args["many"] = serde_json::json!(many);
    }
    FunctionCall {
        call: "pluralize".to_string(),
        args: Some(json_args),
        return_type: Some(ReturnType::String),
    }
}

// ============================================================================
// 逻辑函数
// ============================================================================

/// 创建 and 函数调用
pub fn and(values: Vec<DynamicBoolean>) -> FunctionCall {
    FunctionCall {
        call: "and".to_string(),
        args: Some(serde_json::json!({ "values": values })),
        return_type: Some(ReturnType::Boolean),
    }
}

/// 创建 or 函数调用
pub fn or(values: Vec<DynamicBoolean>) -> FunctionCall {
    FunctionCall {
        call: "or".to_string(),
        args: Some(serde_json::json!({ "values": values })),
        return_type: Some(ReturnType::Boolean),
    }
}

/// 创建 not 函数调用
pub fn not(value: DynamicBoolean) -> FunctionCall {
    FunctionCall {
        call: "not".to_string(),
        args: Some(serde_json::json!({ "value": value })),
        return_type: Some(ReturnType::Boolean),
    }
}

// ============================================================================
// 客户端动作函数
// ============================================================================

/// 创建 openUrl 函数调用
pub fn open_url(url: &str) -> FunctionCall {
    FunctionCall {
        call: "openUrl".to_string(),
        args: Some(serde_json::json!({ "url": url })),
        return_type: Some(ReturnType::Void),
    }
}
