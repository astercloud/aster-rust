//! Rewind 管理器
//!
//! 协调文件历史和对话状态的回退

use serde::{Deserialize, Serialize};
use super::file_history::{FileHistoryManager, RewindResult};

/// Rewind 选项
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewindOption {
    Code,
    Conversation,
    Both,
    Nevermind,
}

/// 可回退的消息信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewindableMessage {
    pub uuid: String,
    pub index: usize,
    pub preview: String,
    pub timestamp: Option<i64>,
    pub has_file_changes: bool,
}


/// Rewind 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewindOperationResult {
    pub success: bool,
    pub option: RewindOption,
    pub code_result: Option<RewindResult>,
    pub conversation_result: Option<ConversationRewindResult>,
    pub error: Option<String>,
}

/// 对话回退结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRewindResult {
    pub messages_removed: i32,
    pub new_message_count: usize,
}

/// Rewind 管理器
pub struct RewindManager {
    file_history: FileHistoryManager,
    message_count: usize,
}


impl RewindManager {
    /// 创建新的 Rewind 管理器
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            file_history: FileHistoryManager::new(session_id),
            message_count: 0,
        }
    }

    /// 获取文件历史管理器
    pub fn get_file_history_manager(&self) -> &FileHistoryManager {
        &self.file_history
    }

    /// 获取可变文件历史管理器
    pub fn get_file_history_manager_mut(&mut self) -> &mut FileHistoryManager {
        &mut self.file_history
    }

    /// 记录用户消息（创建快照点）
    pub fn record_user_message(&mut self, message_id: impl Into<String>) {
        self.file_history.create_snapshot(message_id);
        self.message_count += 1;
    }

    /// 记录文件修改
    pub fn record_file_change(&mut self, file_path: impl AsRef<std::path::Path>) {
        self.file_history.backup_file_before_change(file_path.as_ref());
        self.file_history.track_file(file_path);
    }


    /// 执行回退操作
    pub fn rewind(&mut self, message_id: &str, option: RewindOption) -> RewindOperationResult {
        if option == RewindOption::Nevermind {
            return RewindOperationResult {
                success: true,
                option,
                code_result: None,
                conversation_result: None,
                error: None,
            };
        }

        let mut result = RewindOperationResult {
            success: true,
            option,
            code_result: None,
            conversation_result: None,
            error: None,
        };

        // 回退代码
        if option == RewindOption::Code || option == RewindOption::Both {
            let code_result = self.file_history.rewind_to_message(message_id, false);
            if !code_result.success {
                result.success = false;
                result.error = code_result.error.clone();
            }
            result.code_result = Some(code_result);
        }

        // 回退对话（简化实现，实际需要与消息存储集成）
        if option == RewindOption::Conversation || option == RewindOption::Both {
            result.conversation_result = Some(ConversationRewindResult {
                messages_removed: 0,
                new_message_count: self.message_count,
            });
        }

        result
    }


    /// 预览回退操作
    pub fn preview_rewind(&self, message_id: &str, option: RewindOption) -> RewindPreview {
        let mut preview = RewindPreview::default();

        if option == RewindOption::Code || option == RewindOption::Both {
            let result = self.file_history.rewind_to_message(message_id, true);
            preview.files_will_change = result.files_changed;
            preview.insertions = result.insertions;
            preview.deletions = result.deletions;
        }

        preview
    }

    /// 检查是否可以回退
    pub fn can_rewind(&self) -> bool {
        self.file_history.get_snapshots_count() > 0
    }

    /// 清理
    pub fn cleanup(&self) {
        self.file_history.cleanup();
    }
}

/// 回退预览
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RewindPreview {
    pub files_will_change: Vec<String>,
    pub messages_will_remove: usize,
    pub insertions: u32,
    pub deletions: u32,
}
