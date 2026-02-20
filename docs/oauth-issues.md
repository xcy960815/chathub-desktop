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

### 方案 3：UA 伪装 + 指纹注入 ❌

**原理**：

- 将 WebView 的 User-Agent 伪装为标准 Chrome 浏览器
- 注入 JavaScript 覆盖 WebView 特有的 API 指纹（如 `navigator` 属性等）

**结果**：Google 的检测机制较为复杂，单纯修改 UA 和部分 JS API 仍被识别为 WebView，登录被拦截。

## 可能的后续方案

### 方案 A：更激进的 WebView 指纹伪装

深入研究 Google 的 WebView 检测机制，尝试更全面的环境伪装：

- 覆盖 `navigator.plugins`、`navigator.mimeTypes` 等
- 模拟 Chrome 扩展 API
- 伪装 `window.chrome` 对象
- 拦截 WebRTC、Canvas fingerprint 等高级检测手段

> **风险**：工作量大且不稳定，Google 可能随时更新检测策略。

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

_最后更新：2026-02-20_
