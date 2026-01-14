//! 插件管理器
//!
//! 负责插件的发现、加载、卸载、依赖管理等

use super::types::*;
use super::context::PluginContext;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 插件管理器
pub struct PluginManager {
    /// 插件状态
    plugin_states: Arc<RwLock<HashMap<String, PluginState>>>,
    /// 插件配置
    plugin_configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
    /// 插件目录
    plugin_dirs: Vec<PathBuf>,
    /// 配置目录
    config_dir: PathBuf,
    /// Aster 版本
    aster_version: String,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(aster_version: &str) -> Self {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".aster");

        let plugin_dirs = vec![
            config_dir.join("plugins"),
            std::env::current_dir()
                .unwrap_or_default()
                .join(".aster")
                .join("plugins"),
        ];

        Self {
            plugin_states: Arc::new(RwLock::new(HashMap::new())),
            plugin_configs: Arc::new(RwLock::new(HashMap::new())),
            plugin_dirs,
            config_dir,
            aster_version: aster_version.to_string(),
        }
    }


    /// 添加插件目录
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        if !self.plugin_dirs.contains(&dir) {
            self.plugin_dirs.push(dir);
        }
    }

    /// 发现所有插件
    pub async fn discover(&self) -> Vec<PluginState> {
        let mut discovered = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                continue;
            }

            let entries = match tokio::fs::read_dir(dir).await {
                Ok(e) => e,
                Err(_) => continue,
            };

            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let package_path = path.join("package.json");
                if !package_path.exists() {
                    continue;
                }

                if let Ok(content) = tokio::fs::read_to_string(&package_path).await {
                    if let Ok(metadata) = serde_json::from_str::<PluginMetadata>(&content) {
                        let state = PluginState {
                            metadata: metadata.clone(),
                            path: path.clone(),
                            enabled: true,
                            loaded: false,
                            initialized: false,
                            activated: false,
                            error: None,
                            load_time: None,
                            dependencies: Vec::new(),
                            dependents: Vec::new(),
                        };

                        if let Ok(mut states) = self.plugin_states.write() {
                            states.insert(metadata.name.clone(), state.clone());
                        }
                        discovered.push(state);
                    }
                }
            }
        }

        discovered
    }


    /// 获取插件状态
    pub fn get_plugin_state(&self, name: &str) -> Option<PluginState> {
        self.plugin_states.read().ok()?.get(name).cloned()
    }

    /// 获取所有插件状态
    pub fn get_plugin_states(&self) -> Vec<PluginState> {
        self.plugin_states
            .read()
            .map(|s| s.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 设置插件启用状态
    pub fn set_enabled(&self, name: &str, enabled: bool) -> bool {
        if let Ok(mut states) = self.plugin_states.write() {
            if let Some(state) = states.get_mut(name) {
                state.enabled = enabled;
                return true;
            }
        }
        false
    }

    /// 获取已加载的插件数量
    pub fn loaded_count(&self) -> usize {
        self.plugin_states
            .read()
            .map(|s| s.values().filter(|p| p.loaded).count())
            .unwrap_or(0)
    }

    /// 获取已启用的插件数量
    pub fn enabled_count(&self) -> usize {
        self.plugin_states
            .read()
            .map(|s| s.values().filter(|p| p.enabled).count())
            .unwrap_or(0)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new("0.1.0")
    }
}
