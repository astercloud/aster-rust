# Aster Tauri Desktop App

Tauri 版本的 Aster 桌面应用，提供轻量级、高性能的桌面体验。

## 特性

- 🚀 更小的二进制体积（相比 Electron）
- 💾 更低的内存占用
- 🔒 更好的安全性（Rust 后端）
- 🖥️ 原生系统集成

## 开发

### 前置要求

- Rust 1.70+
- Node.js 20+
- Tauri CLI

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装 Tauri CLI
cargo install tauri-cli
```

### 开发模式

```bash
npm run tauri:dev
```

### 构建

```bash
npm run tauri:build
```


## 项目结构

```
ui/tauri/
├── src/                    # Rust 后端
│   ├── main.rs            # 入口点
│   ├── lib.rs             # 库定义
│   ├── commands.rs        # Tauri 命令
│   ├── state.rs           # 应用状态
│   └── tray.rs            # 系统托盘
├── src/                    # 前端 (React)
│   ├── main.tsx           # React 入口
│   ├── App.tsx            # 主组件
│   └── components/        # UI 组件
├── tauri.conf.json        # Tauri 配置
├── Cargo.toml             # Rust 依赖
└── package.json           # 前端依赖
```

## 与 Electron 版本的区别

| 特性 | Tauri | Electron |
|------|-------|----------|
| 二进制大小 | ~10MB | ~150MB |
| 内存占用 | ~50MB | ~200MB |
| 后端语言 | Rust | Node.js |
| 渲染引擎 | 系统 WebView | Chromium |

## 许可证

Apache-2.0
