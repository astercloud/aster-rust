//! A2UI 通用类型定义
//!
//! 对应 A2UI 规范中的 common_types.json

use serde::{Deserialize, Serialize};

/// 组件唯一标识符
pub type ComponentId = String;

/// 数据绑定 - 引用数据模型中的值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataBinding {
    /// JSON Pointer 路径
    pub path: String,
}

/// 函数调用
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    /// 函数名称
    pub call: String,
    /// 函数参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
    /// 返回类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<ReturnType>,
}

/// 函数返回类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReturnType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Any,
    Void,
}

/// 动态值 - 可以是字面量、数据绑定或函数调用
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DynamicValue {
    /// 字符串字面量
    String(String),
    /// 数字字面量
    Number(f64),
    /// 布尔字面量
    Boolean(bool),
    /// 数组字面量
    Array(Vec<serde_json::Value>),
    /// 数据绑定
    Binding(DataBinding),
    /// 函数调用
    Function(FunctionCall),
}

/// 动态字符串
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DynamicString {
    Literal(String),
    Binding(DataBinding),
    Function(FunctionCall),
}

impl From<&str> for DynamicString {
    fn from(s: &str) -> Self {
        DynamicString::Literal(s.to_string())
    }
}

impl From<String> for DynamicString {
    fn from(s: String) -> Self {
        DynamicString::Literal(s)
    }
}

/// 动态数字
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DynamicNumber {
    Literal(f64),
    Binding(DataBinding),
    Function(FunctionCall),
}

impl From<f64> for DynamicNumber {
    fn from(n: f64) -> Self {
        DynamicNumber::Literal(n)
    }
}

impl From<i32> for DynamicNumber {
    fn from(n: i32) -> Self {
        DynamicNumber::Literal(n as f64)
    }
}

/// 动态布尔值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DynamicBoolean {
    Literal(bool),
    Binding(DataBinding),
    Function(FunctionCall),
}

impl From<bool> for DynamicBoolean {
    fn from(b: bool) -> Self {
        DynamicBoolean::Literal(b)
    }
}

/// 动态字符串列表
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DynamicStringList {
    Literal(Vec<String>),
    Binding(DataBinding),
    Function(FunctionCall),
}

/// 子组件列表 - 静态数组或动态模板
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ChildList {
    /// 静态子组件 ID 列表
    Static(Vec<ComponentId>),
    /// 动态模板（从数据模型生成）
    Template(ChildTemplate),
}

/// 子组件模板
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChildTemplate {
    /// 模板组件 ID
    pub component_id: ComponentId,
    /// 数据模型中的列表路径
    pub path: String,
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckRule {
    /// 验证条件（必须返回布尔值）
    pub condition: DynamicBoolean,
    /// 验证失败时的错误消息
    pub message: String,
}

/// 可验证组件的属性
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Checkable {
    /// 验证规则列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Vec<CheckRule>>,
}

/// 无障碍属性
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AccessibilityAttributes {
    /// 无障碍标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<DynamicString>,
    /// 无障碍描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<DynamicString>,
}

/// 动作定义 - 服务端事件或客户端函数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Action {
    /// 服务端事件
    Event(EventAction),
    /// 客户端函数调用
    Function(FunctionAction),
}

/// 服务端事件动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventAction {
    /// 事件定义
    pub event: EventDefinition,
}

/// 事件定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDefinition {
    /// 事件名称
    pub name: String,
    /// 事件上下文（键值对，值可以是动态的）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Map<String, serde_json::Value>>,
}

/// 客户端函数动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionAction {
    /// 函数调用
    pub function_call: FunctionCall,
}
