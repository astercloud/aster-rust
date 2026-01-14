//! 通知系统模块
//!
//! 提供桌面通知和终端通知功能

mod types;
mod manager;
mod desktop;

pub use types::{
    Notification, NotificationType, NotificationKind,
    NotificationAction, NotificationConfig,
};
pub use manager::NotificationManager;
pub use desktop::{send_desktop_notification, play_sound, bell};
