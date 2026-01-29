# 通知系统

提供桌面通知和终端通知功能。

## 模块结构

```
notifications/
├── desktop.rs  # 桌面通知
├── manager.rs  # 通知管理器
└── types.rs    # 类型定义
```

## 核心功能

### 桌面通知
```rust
pub fn send_desktop_notification(
    title: &str,
    body: &str,
    kind: NotificationKind
) -> Result<()>;

pub fn bell();  // 终端响铃
pub fn play_sound(sound: &str);  // 播放声音
```

### NotificationManager
```rust
pub struct NotificationManager {
    config: NotificationConfig,
}

impl NotificationManager {
    pub fn notify(&self, notification: Notification);
    pub fn set_config(&mut self, config: NotificationConfig);
}
```


## 通知类型

```rust
pub struct Notification {
    pub title: String,
    pub body: String,
    pub kind: NotificationKind,
    pub actions: Vec<NotificationAction>,
}

pub enum NotificationKind {
    Info,
    Success,
    Warning,
    Error,
}

pub enum NotificationType {
    Desktop,
    Terminal,
    Sound,
}
```

## 配置

```rust
pub struct NotificationConfig {
    pub enabled: bool,
    pub desktop_enabled: bool,
    pub sound_enabled: bool,
    pub types: Vec<NotificationType>,
}
```

## 使用场景

- 任务完成通知
- 错误告警
- 需要用户确认时提醒

## 源码位置

`crates/aster/src/notifications/`
