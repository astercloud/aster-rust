//! A2UI 标准组件目录
//!
//! 对应 A2UI 规范中的 standard_catalog.json

use serde::{Deserialize, Serialize};

use crate::common::{
    AccessibilityAttributes, Action, Checkable, ChildList, ComponentId, DynamicBoolean,
    DynamicNumber, DynamicString, DynamicStringList,
};

/// 标准组件目录 ID
pub const STANDARD_CATALOG_ID: &str = "https://a2ui.org/specification/v0_10/standard_catalog.json";

// ============================================================================
// 组件通用属性
// ============================================================================

/// 组件通用属性
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCommon {
    /// 组件唯一标识符
    pub id: ComponentId,
    /// 无障碍属性
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility: Option<AccessibilityAttributes>,
    /// 布局权重（仅在 Row/Column 子组件中有效）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

// ============================================================================
// 组件枚举
// ============================================================================

/// 所有标准组件的枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "component")]
pub enum Component {
    Text(TextComponent),
    Image(ImageComponent),
    Icon(IconComponent),
    Video(VideoComponent),
    AudioPlayer(AudioPlayerComponent),
    Row(RowComponent),
    Column(ColumnComponent),
    List(ListComponent),
    Card(CardComponent),
    Tabs(TabsComponent),
    Modal(ModalComponent),
    Divider(DividerComponent),
    Button(ButtonComponent),
    TextField(TextFieldComponent),
    CheckBox(CheckBoxComponent),
    ChoicePicker(ChoicePickerComponent),
    Slider(SliderComponent),
    DateTimeInput(DateTimeInputComponent),
}

impl Component {
    /// 获取组件 ID
    pub fn id(&self) -> &str {
        match self {
            Component::Text(c) => &c.common.id,
            Component::Image(c) => &c.common.id,
            Component::Icon(c) => &c.common.id,
            Component::Video(c) => &c.common.id,
            Component::AudioPlayer(c) => &c.common.id,
            Component::Row(c) => &c.common.id,
            Component::Column(c) => &c.common.id,
            Component::List(c) => &c.common.id,
            Component::Card(c) => &c.common.id,
            Component::Tabs(c) => &c.common.id,
            Component::Modal(c) => &c.common.id,
            Component::Divider(c) => &c.common.id,
            Component::Button(c) => &c.common.id,
            Component::TextField(c) => &c.common.id,
            Component::CheckBox(c) => &c.common.id,
            Component::ChoicePicker(c) => &c.common.id,
            Component::Slider(c) => &c.common.id,
            Component::DateTimeInput(c) => &c.common.id,
        }
    }
}

// ============================================================================
// 展示组件
// ============================================================================

/// 文本组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 文本内容（支持简单 Markdown）
    pub text: DynamicString,
    /// 文本样式变体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<TextVariant>,
}

/// 文本样式变体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TextVariant {
    H1,
    H2,
    H3,
    H4,
    H5,
    Caption,
    Body,
}

/// 图片组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 图片 URL
    pub url: DynamicString,
    /// 图片适应方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fit: Option<ImageFit>,
    /// 图片样式变体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ImageVariant>,
}

/// 图片适应方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ImageFit {
    Contain,
    Cover,
    Fill,
    None,
    ScaleDown,
}

/// 图片样式变体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ImageVariant {
    Icon,
    Avatar,
    SmallFeature,
    MediumFeature,
    LargeFeature,
    Header,
}

/// 图标组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IconComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 图标名称或自定义路径
    pub name: IconName,
}

/// 图标名称
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum IconName {
    /// 预定义图标
    Preset(PresetIcon),
    /// 自定义 SVG 路径
    Custom { path: String },
}

/// 预定义图标列表
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PresetIcon {
    AccountCircle,
    Add,
    ArrowBack,
    ArrowForward,
    AttachFile,
    CalendarToday,
    Call,
    Camera,
    Check,
    Close,
    Delete,
    Download,
    Edit,
    Event,
    Error,
    FastForward,
    Favorite,
    FavoriteOff,
    Folder,
    Help,
    Home,
    Info,
    LocationOn,
    Lock,
    LockOpen,
    Mail,
    Menu,
    MoreVert,
    MoreHoriz,
    NotificationsOff,
    Notifications,
    Pause,
    Payment,
    Person,
    Phone,
    Photo,
    Play,
    Print,
    Refresh,
    Rewind,
    Search,
    Send,
    Settings,
    Share,
    ShoppingCart,
    SkipNext,
    SkipPrevious,
    Star,
    StarHalf,
    StarOff,
    Stop,
    Upload,
    Visibility,
    VisibilityOff,
    VolumeDown,
    VolumeMute,
    VolumeOff,
    VolumeUp,
    Warning,
}

/// 视频组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 视频 URL
    pub url: DynamicString,
}

/// 音频播放器组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioPlayerComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 音频 URL
    pub url: DynamicString,
    /// 音频描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<DynamicString>,
}

// ============================================================================
// 布局组件
// ============================================================================

/// 水平布局组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RowComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 子组件列表
    pub children: ChildList,
    /// 主轴对齐方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<JustifyContent>,
    /// 交叉轴对齐方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<AlignItems>,
}

/// 垂直布局组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ColumnComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 子组件列表
    pub children: ChildList,
    /// 主轴对齐方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<JustifyContent>,
    /// 交叉轴对齐方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<AlignItems>,
}

/// 主轴对齐方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

/// 交叉轴对齐方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

/// 列表组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 子组件列表
    pub children: ChildList,
    /// 列表方向
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<ListDirection>,
    /// 交叉轴对齐方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<AlignItems>,
}

/// 列表方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ListDirection {
    Vertical,
    Horizontal,
}

/// 卡片组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 子组件 ID
    pub child: ComponentId,
}

/// 标签页组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabsComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 标签页列表
    pub tabs: Vec<TabItem>,
}

/// 标签页项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabItem {
    /// 标签标题
    pub title: DynamicString,
    /// 标签内容组件 ID
    pub child: ComponentId,
}

/// 模态框组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModalComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 触发器组件 ID
    pub trigger: ComponentId,
    /// 内容组件 ID
    pub content: ComponentId,
}

/// 分隔线组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DividerComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 分隔线方向
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<DividerAxis>,
}

/// 分隔线方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DividerAxis {
    Horizontal,
    Vertical,
}

// ============================================================================
// 交互组件
// ============================================================================

/// 按钮组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ButtonComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 按钮内容组件 ID
    pub child: ComponentId,
    /// 按钮动作
    pub action: Action,
    /// 按钮样式变体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ButtonVariant>,
    /// 验证规则
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub checkable: Option<Checkable>,
}

/// 按钮样式变体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ButtonVariant {
    Primary,
    Borderless,
}

/// 文本输入框组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextFieldComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 输入框标签
    pub label: DynamicString,
    /// 输入框值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<DynamicString>,
    /// 输入框类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<TextFieldVariant>,
    /// 验证规则
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub checkable: Option<Checkable>,
}

/// 文本输入框类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TextFieldVariant {
    ShortText,
    LongText,
    Number,
    Obscured,
}

/// 复选框组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CheckBoxComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 复选框标签
    pub label: DynamicString,
    /// 复选框值
    pub value: DynamicBoolean,
    /// 验证规则
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub checkable: Option<Checkable>,
}

/// 选择器组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChoicePickerComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 选择器标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<DynamicString>,
    /// 选项列表
    pub options: Vec<ChoiceOption>,
    /// 当前选中值
    pub value: DynamicStringList,
    /// 选择器类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ChoicePickerVariant>,
    /// 验证规则
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub checkable: Option<Checkable>,
}

/// 选择器选项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceOption {
    /// 选项显示文本
    pub label: DynamicString,
    /// 选项值
    pub value: String,
}

/// 选择器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChoicePickerVariant {
    MultipleSelection,
    MutuallyExclusive,
}

/// 滑块组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SliderComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 滑块标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<DynamicString>,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 当前值
    pub value: DynamicNumber,
    /// 验证规则
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub checkable: Option<Checkable>,
}

/// 日期时间输入组件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DateTimeInputComponent {
    #[serde(flatten)]
    pub common: ComponentCommon,
    /// 输入框标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<DynamicString>,
    /// 当前值（ISO 8601 格式）
    pub value: DynamicString,
    /// 是否启用日期选择
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_date: Option<bool>,
    /// 是否启用时间选择
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_time: Option<bool>,
    /// 最小日期时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<DynamicString>,
    /// 最大日期时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<DynamicString>,
    /// 验证规则
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub checkable: Option<Checkable>,
}
