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
