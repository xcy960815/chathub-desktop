# Google OAuth 登录 — 当前问题总结

## 已完成 ✅

| 功能           | 状态 | 说明                                                    |
| -------------- | ---- | ------------------------------------------------------- |
| OAuth 授权流程 | ✅   | PKCE + state 防 CSRF，localhost 随机端口回调            |
| Token 获取     | ✅   | 成功获取 access_token、refresh_token                    |
| 用户信息获取   | ✅   | 成功获取 id、email、name、头像等                        |
| 数据持久化     | ✅   | Token 和用户信息保存到 tauri-plugin-store               |
| Secrets 安全   | ✅   | Client ID/Secret 通过 `.env` 环境变量注入，不提交到仓库 |

## 未解决的核心问题 ❌

### WebView 无法登录 Google

**现象**：OAuth 流程在系统浏览器中完成后，主窗口的 WebView（Gemini/ChatGPT 页面）仍是未登录状态。用户需要在 WebView 内再次登录 Google 才能使用 Gemini 等服务。

**根本原因**：OAuth 获取的 `access_token` 是 API 令牌，无法直接转换为 WebView 的浏览器会话 Cookie（如 `SID`、`HSID`、`SSID` 等）。

## 已尝试的方案

### 方案 1：OAuthLogin + MergeSession ❌

**原理**：用 access_token 调用 Google 的 `OAuthLogin` 接口获取 `uberauth` token，再通过 `MergeSession` URL 在 WebView 中建立会话。

```
GET https://accounts.google.com/OAuthLogin?source=ChromiumBrowser&issueuberauth=1
Authorization: Bearer {access_token}
```

**结果**：返回 `403 Forbidden`。Google 已将此 API 限制为 Chrome 浏览器内部使用。

### 方案 2：WebView 内直接登录 Google ❌

**原理**：用户直接在 WebView 中点击 Gemini 的"登录"按钮，完成 Google 账号登录。

**结果**：Google 检测到嵌入式 WebView 环境（macOS 上是 WKWebView），显示 **"此浏览器或应用可能不安全"** 错误，拒绝登录。

### 方案 3：UA 伪装 + 指纹注入 ⚠️ 已完善，效果待验证

**原理**：

- 将 WebView 的 User-Agent 伪装为标准 Chrome 浏览器
- 注入 JavaScript 覆盖 WebView 特有的 API 指纹（如 `navigator` 属性等）

**已实现（2026-05）**：

- Rust 层与 JS 层共用同一 UA 常量，避免 HTTP 头与 `navigator.userAgent` 不一致
- 完整的 Chrome `plugins` / `mimeTypes` 模拟（PDF Plugin、PDF Viewer、Native Client）
- `window.chrome` 对象（含 `runtime`、`webstore`、`loadTimes`、`csi`）
- `navigator.userAgentData` Client Hints 模拟
- `navigator.webdriver = false`
- WebGL 渲染器/vendor 伪装
- Canvas 指纹噪声
- WebRTC 本地 IP 候选过滤
- Google/OpenAI 页面隐藏 `window.webkit`、`ApplePaySession` 等 WebKit 特征
- 调试快照：`window.__CHATHUB_STEALTH_SNAPSHOT__()`

**注意**：Google OAuth 仍可能因 `disallowed_useragent` 策略拦截嵌入式 WebView；此方案主要改善 Gemini 等页面内的 Google 账号登录体验，不保证 100% 绕过。

> **2026-09 更新**：实测在 macOS 上注入伪装脚本会破坏 Cloudflare Turnstile 人机验证（chatgpt.com 加载挑战页时被干扰），已在 macOS 上**完全禁用** stealth 注入（`build_initialization_script` 仅保留 `navigation_bridge.js`，UA 也不再伪装），伪装脚本仅在非 macOS 平台启用。

### ChatGPT 的 Cloudflare 人机验证无法通过 ✅ 已解决（系统浏览器外开）

**现象**：macOS WKWebView 打开 chatgpt.com 时被 Cloudflare Turnstile 拦截，无法进入对话页面。

**方案**：macOS 上切换到 ChatGPT 时不再用内置 WebView 加载，改为调用系统浏览器打开，主窗口导航回应用外壳并显示"已在系统浏览器中打开"的提示卡片（支持"再次打开"）。相关逻辑：

- Rust：`get_model_launch_plan` 统一返回启动计划并负责打开浏览器；`switch_model` 只负责把 WebView 导航回外壳页面，避免重复打开
- 前端：`App.vue` 的 `launchModel` 根据计划决定跳转 URL 还是展示外开提示
- 持久化：`chatgpt.com/auth/...` 等错误页 URL 视为瞬态地址，自动回退到默认 URL

## 可能的后续方案

### 方案 A：更激进的 WebView 指纹伪装 ✅ 已实现（仅非 macOS 平台启用）

已在 `src-tauri/src/stealth_bridge.js` 中实现，包括：

- 覆盖 `navigator.plugins`、`navigator.mimeTypes` 等
- 模拟 Chrome 扩展 API（`window.chrome.runtime` 等）
- 伪装 `window.chrome` 对象
- Canvas fingerprint 噪声、WebRTC 本地 IP 过滤

> **风险**：Google 可能随时更新检测策略，需实际测试验证。

### 方案 B：Cookie 共享（macOS 限定）

利用 macOS 上 WKWebView 的 `WKWebsiteDataStore` 共享 Safari 的 Cookie。如果用户已在 Safari 中登录 Google，WebView 可直接复用该会话。

> **限制**：仅限 macOS，且需要 Tauri 底层支持切换 data store，可行性待验证。

### 方案 C：迁移到 Electron

Electron 基于 Chromium，Google 对 Chromium 内核更友好：

- 可通过 `session.partition` 管理 Cookie
- UA 与标准 Chrome 更接近，更难被检测
- 社区有成熟的 Google 登录解决方案

> **代价**：需要从 Tauri 迁移到 Electron，应用体积会从 ~10MB 增加到 ~100MB。

### 方案 D：接受现状

- OAuth 登录用于获取用户基本信息（用于应用内功能）
- Gemini 等服务的登录，引导用户在 WebView 内手动完成（如果 Google 允许）
- 或者引导用户在系统浏览器中使用 Gemini

---

_最后更新：2026-09-27_
