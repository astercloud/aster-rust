# Tauri 桌面应用

基于 Tauri 2.0 的轻量级桌面应用。

## 概述

Tauri 版本提供与 Electron 版本相同的功能，但具有更小的体积和更低的资源占用。

**路径**: `ui/tauri/`

## 技术栈

| 技术 | 用途 |
|------|------|
| Tauri 2.0 | 桌面框架 |
| Rust | 后端逻辑 |
| React | UI 框架 |
| TypeScript | 类型安全 |
| Vite | 构建工具 |
| Tailwind CSS | 样式 |

## Rust 后端结构

```
ui/tauri/src/
├── main.rs      # 入口点
├── lib.rs       # 库定义和插件初始化
├── commands.rs  # Tauri 命令
├── state.rs     # 应用状态管理
└── tray.rs      # 系统托盘
```


## Tauri 命令

```rust
// 配置命令
#[tauri::command]
async fn get_config(key: String) -> Result<Value, String>;

#[tauri::command]
async fn set_config(key: String, value: Value) -> Result<(), String>;

// 会话命令
#[tauri::command]
async fn start_session(name: String, working_dir: String) -> Result<SessionInfo, String>;

#[tauri::command]
async fn send_message(session_id: String, content: String) -> Result<Message, String>;

// 服务器命令
#[tauri::command]
async fn start_server(port: Option<u16>) -> Result<(), String>;

#[tauri::command]
async fn stop_server() -> Result<(), String>;
```

## 插件配置

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_os::init())
    .plugin(tauri_plugin_opener::init())
```

## 前端调用

```typescript
import { invoke } from "@tauri-apps/api/core";

// 调用 Rust 命令
const sessions = await invoke<SessionInfo[]>("get_sessions");
const message = await invoke<Message>("send_message", {
  sessionId: "xxx",
  content: "Hello",
});
```

## 开发命令

```bash
npm run tauri:dev    # 开发模式
npm run tauri:build  # 构建发布版
```

## 源码位置

`ui/tauri/`
