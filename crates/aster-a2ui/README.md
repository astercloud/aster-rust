# aster-a2ui

A2UI (Agent-to-User Interface) 协议的 Rust 实现，基于 Google A2UI v0.10 规范。

## 功能特性

- 完整的协议消息类型定义（服务端/客户端）
- 标准组件目录（18 种组件）
- 标准函数库（14 种函数）
- 客户端能力声明和数据模型同步
- JSON Pointer 路径解析
- 流式消息构建器

## 快速开始

```rust
use aster_a2ui::prelude::*;

// 创建 Surface
let msg = ServerMessage::create_surface(
    "contact_form",
    STANDARD_CATALOG_ID,
);

// 创建组件
let text = Component::Text(TextComponent {
    common: ComponentCommon {
        id: "title".to_string(),
        ..Default::default()
    },
    text: "请填写联系信息".into(),
    variant: Some(TextVariant::H2),
});

// 更新组件
let update = ServerMessage::update_components("contact_form", vec![text]);
```

## 模块结构

| 模块 | 说明 |
|------|------|
| `protocol` | 协议消息类型（服务端/客户端） |
| `catalog` | 标准组件定义 |
| `common` | 通用类型（DynamicValue, Action 等） |
| `functions` | 标准函数构建器 |
| `validation` | JSON Pointer 工具 |

## 组件列表

| 类别 | 组件 |
|------|------|
| 展示 | Text, Image, Icon, Video, AudioPlayer |
| 布局 | Row, Column, List, Card, Tabs, Modal, Divider |
| 交互 | Button, TextField, CheckBox, ChoicePicker, Slider, DateTimeInput |

## 函数列表

| 类别 | 函数 |
|------|------|
| 验证 | required, regex, length, numeric, email |
| 格式化 | formatString, formatNumber, formatCurrency, formatDate, pluralize |
| 逻辑 | and, or, not |
| 动作 | openUrl |

## 协议消息

### 服务端到客户端
- `createSurface` - 创建 UI Surface
- `updateComponents` - 更新组件树
- `updateDataModel` - 更新数据模型
- `deleteSurface` - 删除 Surface

### 客户端到服务端
- `action` - 用户交互事件
- `error` - 客户端错误报告

### Transport Metadata
- `ClientCapabilities` - 客户端能力声明
- `ClientDataModel` - 客户端数据模型同步

## 许可证

Apache-2.0
