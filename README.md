# ChatHub Desktop

<p align="center">
  <img src="src-tauri/icons/128x128.png" width="128" alt="ChatHub Icon" />
</p>

<p align="center">
  <strong>一款基于 Tauri v2 构建的轻量级、极速多模型 AI 桌面客户端</strong>
</p>

---

ChatHub Desktop 是一个聚合了顶级 AI 模型（ChatGPT、DeepSeek、Grok、Gemini、Qwen、Doubao）的桌面端应用。它旨在提供比网页端更流畅的交互体验，并深度集成系统原生功能（如托盘运行、全局快捷键、代理配置等）。

## ✨ 核心特性

- 🚀 **多模型集成**: 一键切换 ChatGPT、DeepSeek、Grok、Gemini、Qwen、Doubao。
- 💾 **按模型记忆会话**: 每个模型分别保存最近访问 URL，切换回来时自动恢复。
- 🎨 **极致设计**:
  - 精致的 **macOS 原生风格** 交互体验。
  - 深度适配 **深色模式 (Dark Mode)**。
  - 极简的加载动效，提供无缝的无感切换。
- 🌍 **代理与快捷键历史**: 代理设置、快捷键设置支持最近记录与快速复用。
- ⏲️ **轻量常驻**:
  - 菜单栏/系统托盘运行，不占桌面空间。
  - 点击图标即开即用，关闭主窗口时自动隐藏到托盘。
- 🛠️ **系统深度集成**:
  - **全局快捷键**: 支持通过快捷键快速呼出。
  - **窗口置顶**: 可从托盘菜单直接切换。
  - **开机自启**: 可配置跟随系统自动启动。
  - **检查更新**: 托盘菜单可手动检查 GitHub Release 更新。

## 🛠️ 技术栈

- **Core**: [Tauri v2](https://tauri.app/) (Rust 驱动)
- **Frontend**: [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/)
- **Build Tool**: [Vite](https://vitejs.dev/)
- **Styling**: Vanilla CSS (Modern CSS Variables)
- **Persistence**: `tauri-plugin-store`

## 📦 快速开始

### 环境依赖

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) & [pnpm](https://pnpm.io/)
- 对应系统的构建工具 (macOS: Xcode Command Line Tools)

### 运行开发版本

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev
```

### 构建生产版本

```bash
pnpm tauri build
```

## ⌨️ 快捷操作

- `Cmd/Ctrl + G`: 快速显示/隐藏窗口（默认）
- `Esc`: 关闭当前弹出窗口（如代理设置）
- `Tray Click`: 切换主程序可见性

## 🤝 贡献说明

欢迎提交 Issue 或 Pull Request。在提交 PR 前，请确保代码符合项目的 ESLint 和 Rust 代码规范。

---

**ChatHub Desktop** - 让 AI 触手可及。
