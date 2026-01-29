# Electron 桌面应用

## 概述

Aster Desktop 是基于 Electron 的桌面应用，提供图形化界面。

**路径**: `ui/desktop/`

## 目录结构

```
ui/desktop/
├── src/
│   ├── main/           # Electron 主进程
│   ├── preload/        # 预加载脚本
│   └── renderer/       # 渲染进程 (React)
├── resources/          # 静态资源
├── package.json
├── electron.vite.config.ts
└── tsconfig.json
```

## 技术栈

| 技术 | 用途 |
|------|------|
| Electron | 桌面框架 |
| React | UI 框架 |
| TypeScript | 类型安全 |
| Vite | 构建工具 |
| Tailwind CSS | 样式 |

## 与后端通信

桌面应用通过以下方式与 Rust 后端通信：

```
┌─────────────────────────────────────────┐
│           Electron App                   │
│  ┌─────────────────────────────────┐    │
│  │         Renderer Process         │    │
│  │         (React UI)               │    │
│  └──────────────┬──────────────────┘    │
│                 │ IPC                    │
│  ┌──────────────┴──────────────────┐    │
│  │         Main Process             │    │
│  └──────────────┬──────────────────┘    │
└─────────────────┼───────────────────────┘
                  │ HTTP/WebSocket
┌─────────────────┴───────────────────────┐
│           asterd (Rust Server)           │
└─────────────────────────────────────────┘
```

## 主要功能

### 1. 聊天界面
- 消息历史显示
- Markdown 渲染
- 代码高亮
- 流式响应

### 2. 会话管理
- 会话列表
- 创建/删除会话
- 会话搜索
- 会话导出

### 3. 设置面板
- Provider 配置
- 模型选择
- 扩展管理
- 主题设置

### 4. 工具面板
- 工具调用历史
- 文件浏览器
- 终端集成

## 开发命令

```bash
# 进入目录
cd ui/desktop

# 安装依赖
npm install

# 开发模式
npm run dev

# 构建
npm run build

# 打包
npm run package
```

## IPC 通信

```typescript
// 主进程 -> 渲染进程
ipcMain.handle('get-sessions', async () => {
  return await fetchSessions();
});

// 渲染进程调用
const sessions = await ipcRenderer.invoke('get-sessions');
```

## 配置文件

```json
// electron.vite.config.ts
{
  "main": {
    "build": {
      "outDir": "dist/main"
    }
  },
  "preload": {
    "build": {
      "outDir": "dist/preload"
    }
  },
  "renderer": {
    "build": {
      "outDir": "dist/renderer"
    }
  }
}
```

## 打包配置

```json
// package.json
{
  "build": {
    "appId": "com.aster.desktop",
    "productName": "Aster",
    "mac": {
      "target": ["dmg", "zip"]
    },
    "win": {
      "target": ["nsis", "portable"]
    },
    "linux": {
      "target": ["AppImage", "deb"]
    }
  }
}
```

## 与 CLI 的关系

```
┌─────────────────────────────────────────┐
│              用户界面                    │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │   CLI       │  │   Desktop App   │   │
│  │  (aster)    │  │   (Electron)    │   │
│  └──────┬──────┘  └────────┬────────┘   │
│         │                  │            │
│         └────────┬─────────┘            │
│                  │                      │
│         ┌────────┴────────┐             │
│         │     asterd      │             │
│         │  (Rust Server)  │             │
│         └─────────────────┘             │
└─────────────────────────────────────────┘
```

- CLI 直接调用核心库
- Desktop 通过 asterd 服务通信
- 两者共享相同的会话和配置


---

## Tauri 版本 (新增)

基于 Tauri 2.0 的轻量级桌面应用。

**路径**: `ui/tauri/`

### 目录结构

```
ui/tauri/
├── src/                # Rust 后端
│   ├── main.rs        # 入口点
│   ├── lib.rs         # 库定义
│   ├── commands.rs    # Tauri 命令
│   ├── state.rs       # 应用状态
│   └── tray.rs        # 系统托盘
├── src/                # 前端 (React)
│   ├── main.tsx       # React 入口
│   ├── App.tsx        # 主组件
│   └── components/    # UI 组件
├── tauri.conf.json    # Tauri 配置
├── Cargo.toml         # Rust 依赖
└── package.json       # 前端依赖
```

### Electron vs Tauri 对比

| 特性 | Electron | Tauri |
|------|----------|-------|
| 二进制大小 | ~150MB | ~10MB |
| 内存占用 | ~200MB | ~50MB |
| 后端语言 | Node.js | Rust |
| 渲染引擎 | Chromium | 系统 WebView |
| 启动速度 | 较慢 | 快 |

### 开发命令

```bash
cd ui/tauri
npm install
npm run tauri:dev    # 开发模式
npm run tauri:build  # 构建
```

### Tauri 命令示例

```rust
#[tauri::command]
async fn send_message(session_id: String, content: String) -> Result<Message, String> {
    // 调用 aster 核心库
    Ok(message)
}
```

## 源码位置

- Electron: `ui/desktop/`
- Tauri: `ui/tauri/`
