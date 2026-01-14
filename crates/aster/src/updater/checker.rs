//! 版本检查器
//!
//! 提供版本检查和比较功能

use serde::{Deserialize, Serialize};

/// 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub release_date: String,
    pub changelog: Option<String>,
    pub download_url: Option<String>,
    pub description: Option<String>,
}

/// 更新检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub version_info: Option<VersionInfo>,
    pub changelog: Option<Vec<String>>,
}


/// 比较版本号
/// 返回: 1 表示 v1 > v2, -1 表示 v1 < v2, 0 表示相等
pub fn compare_versions(v1: &str, v2: &str) -> i32 {
    let parse_version = |v: &str| -> Vec<u32> {
        v.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>()
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let parts1 = parse_version(v1);
    let parts2 = parse_version(v2);

    let max_len = parts1.len().max(parts2.len());

    for i in 0..max_len {
        let p1 = parts1.get(i).copied().unwrap_or(0);
        let p2 = parts2.get(i).copied().unwrap_or(0);

        if p1 > p2 { return 1; }
        if p1 < p2 { return -1; }
    }

    0
}

/// 检查更新（简化实现）
pub async fn check_for_updates(current_version: &str) -> Result<UpdateCheckResult, String> {
    // 实际实现需要从远程获取最新版本
    // 这里返回一个模拟结果
    Ok(UpdateCheckResult {
        has_update: false,
        current_version: current_version.to_string(),
        latest_version: current_version.to_string(),
        version_info: None,
        changelog: None,
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert_eq!(compare_versions("1.0.1", "1.0.0"), 1);
        assert_eq!(compare_versions("1.0.0", "1.0.1"), -1);
        assert_eq!(compare_versions("2.0.0", "1.9.9"), 1);
        assert_eq!(compare_versions("1.10.0", "1.9.0"), 1);
        assert_eq!(compare_versions("v1.0.0", "1.0.0"), 0);
    }
}
