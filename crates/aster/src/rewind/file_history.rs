//! 文件历史跟踪系统
//!
//! 提供文件修改跟踪、快照创建、状态恢复功能

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;

/// 文件备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBackup {
    /// 备份文件名
    pub backup_file_name: Option<String>,
    /// 原始文件的最后修改时间
    pub mtime: u64,
    /// 版本号
    pub version: u32,
    /// 文件哈希
    pub hash: Option<String>,
}


/// 快照数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    /// 关联的消息 ID
    pub message_id: String,
    /// 快照创建时间
    pub timestamp: i64,
    /// 被跟踪文件的备份信息
    pub tracked_file_backups: HashMap<String, FileBackup>,
}

/// Rewind 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewindResult {
    pub success: bool,
    pub files_changed: Vec<String>,
    pub insertions: u32,
    pub deletions: u32,
    pub error: Option<String>,
}

impl RewindResult {
    pub fn success(files_changed: Vec<String>, insertions: u32, deletions: u32) -> Self {
        Self { success: true, files_changed, insertions, deletions, error: None }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self { success: false, files_changed: vec![], insertions: 0, deletions: 0, error: Some(msg.into()) }
    }
}


/// 文件历史管理器
pub struct FileHistoryManager {
    session_id: String,
    tracked_files: HashSet<String>,
    snapshots: Vec<FileSnapshot>,
    backup_dir: PathBuf,
    enabled: bool,
}

impl FileHistoryManager {
    /// 创建新的文件历史管理器
    pub fn new(session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        let backup_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("aster")
            .join("file-history")
            .join(&session_id);

        // 确保备份目录存在
        let _ = fs::create_dir_all(&backup_dir);

        Self {
            session_id,
            tracked_files: HashSet::new(),
            snapshots: Vec::new(),
            backup_dir,
            enabled: true,
        }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 启用/禁用文件历史
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }


    /// 开始跟踪文件
    pub fn track_file(&mut self, file_path: impl AsRef<Path>) {
        if !self.enabled { return; }
        let path = self.normalize_path(file_path.as_ref());
        self.tracked_files.insert(path);
    }

    /// 检查文件是否被跟踪
    pub fn is_tracked(&self, file_path: impl AsRef<Path>) -> bool {
        let path = self.normalize_path(file_path.as_ref());
        self.tracked_files.contains(&path)
    }

    /// 在文件修改前创建备份
    pub fn backup_file_before_change(&mut self, file_path: impl AsRef<Path>) -> Option<FileBackup> {
        if !self.enabled { return None; }

        let path = file_path.as_ref();
        let normalized = self.normalize_path(path);

        // 如果文件不存在，返回空备份
        if !path.exists() {
            return Some(FileBackup {
                backup_file_name: None,
                mtime: 0,
                version: 1,
                hash: None,
            });
        }

        // 读取文件内容并计算哈希
        let content = fs::read(path).ok()?;
        let hash = self.compute_hash(&content);
        let mtime = fs::metadata(path).ok()?.modified().ok()?
            .duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();

        // 生成备份文件名
        let backup_file_name = self.generate_backup_file_name(path, &hash);
        let backup_path = self.backup_dir.join(&backup_file_name);

        // 如果备份不存在，创建它
        if !backup_path.exists() {
            let _ = fs::write(&backup_path, &content);
        }

        // 开始跟踪这个文件
        self.tracked_files.insert(normalized);

        Some(FileBackup {
            backup_file_name: Some(backup_file_name),
            mtime,
            version: 1,
            hash: Some(hash),
        })
    }


    /// 创建快照
    pub fn create_snapshot(&mut self, message_id: impl Into<String>) {
        if !self.enabled { return; }

        let mut tracked_file_backups = HashMap::new();

        for file_path in self.tracked_files.clone() {
            if let Some(backup) = self.backup_file_before_change(&file_path) {
                tracked_file_backups.insert(file_path, backup);
            }
        }

        self.snapshots.push(FileSnapshot {
            message_id: message_id.into(),
            timestamp: chrono::Utc::now().timestamp(),
            tracked_file_backups,
        });
    }

    /// 检查是否有指定消息的快照
    pub fn has_snapshot(&self, message_id: &str) -> bool {
        self.snapshots.iter().any(|s| s.message_id == message_id)
    }

    /// 获取快照列表
    pub fn get_snapshots(&self) -> &[FileSnapshot] {
        &self.snapshots
    }

    /// 回退到指定消息的状态
    pub fn rewind_to_message(&self, message_id: &str, dry_run: bool) -> RewindResult {
        if !self.enabled {
            return RewindResult::error("文件历史已禁用");
        }

        // 查找快照
        let snapshot = self.snapshots.iter().rev().find(|s| s.message_id == message_id);
        let snapshot = match snapshot {
            Some(s) => s,
            None => return RewindResult::error(format!("未找到消息 {} 的快照", message_id)),
        };

        self.apply_snapshot(snapshot, dry_run)
    }


    /// 应用快照
    fn apply_snapshot(&self, snapshot: &FileSnapshot, dry_run: bool) -> RewindResult {
        let mut files_changed = Vec::new();
        let mut insertions = 0u32;
        let mut deletions = 0u32;

        for file_path in &self.tracked_files {
            let backup = match snapshot.tracked_file_backups.get(file_path) {
                Some(b) => b,
                None => continue,
            };

            let path = Path::new(file_path);

            if backup.backup_file_name.is_none() {
                // 文件在快照时不存在，应该删除
                if path.exists() {
                    deletions += self.count_lines(path);
                    if !dry_run {
                        let _ = fs::remove_file(path);
                    }
                    files_changed.push(file_path.clone());
                }
            } else if let Some(ref backup_name) = backup.backup_file_name {
                // 恢复文件内容
                let backup_path = self.backup_dir.join(backup_name);
                if !backup_path.exists() {
                    continue;
                }

                let (ins, del) = self.calculate_diff(path, &backup_path);
                insertions += ins;
                deletions += del;

                if ins > 0 || del > 0 {
                    if !dry_run {
                        if let Ok(content) = fs::read(&backup_path) {
                            if let Some(parent) = path.parent() {
                                let _ = fs::create_dir_all(parent);
                            }
                            let _ = fs::write(path, content);
                        }
                    }
                    files_changed.push(file_path.clone());
                }
            }
        }

        RewindResult::success(files_changed, insertions, deletions)
    }


    /// 计算文件差异
    fn calculate_diff(&self, current: &Path, backup: &Path) -> (u32, u32) {
        let current_lines = self.count_lines(current);
        let backup_lines = self.count_lines(backup);

        let insertions = backup_lines.saturating_sub(current_lines);
        let deletions = current_lines.saturating_sub(backup_lines);

        (insertions, deletions)
    }

    /// 计算文件行数
    fn count_lines(&self, path: &Path) -> u32 {
        fs::read_to_string(path)
            .map(|s| s.lines().count() as u32)
            .unwrap_or(0)
    }

    /// 生成备份文件名
    fn generate_backup_file_name(&self, file_path: &Path, hash: &str) -> String {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let name = file_path.file_stem().and_then(|n| n.to_str()).unwrap_or("file");

        if ext.is_empty() {
            format!("{}_{}", name, &hash[..8])
        } else {
            format!("{}_{}.{}", name, &hash[..8], ext)
        }
    }

    /// 计算文件内容的哈希
    fn compute_hash(&self, content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// 规范化文件路径
    fn normalize_path(&self, path: &Path) -> String {
        if path.is_absolute() {
            path.display().to_string()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path).display().to_string())
                .unwrap_or_else(|_| path.display().to_string())
        }
    }

    /// 清理备份文件
    pub fn cleanup(&self) {
        let _ = fs::remove_dir_all(&self.backup_dir);
    }

    /// 获取被跟踪的文件数量
    pub fn get_tracked_files_count(&self) -> usize {
        self.tracked_files.len()
    }

    /// 获取快照数量
    pub fn get_snapshots_count(&self) -> usize {
        self.snapshots.len()
    }
}
