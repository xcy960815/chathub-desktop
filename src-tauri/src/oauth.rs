use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use log::{error, info, warn};
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

use crate::settings::SETTINGS_FILENAME;
use crate::{build_http_client, tray::update_tray_menu};

// Google OAuth 配置
// 注意: 需要在 Google Cloud Console 创建 OAuth 2.0 Client ID
// 1. 访问 https://console.cloud.google.com/apis/credentials
// 2. 创建 OAuth 2.0 Client ID (类型: Desktop app)
// 3. 在 src-tauri/.env 中配置 GOOGLE_OAUTH_CLIENT_ID；CLIENT_SECRET 可选，
//    留空时走纯 PKCE 流程（installed-app 场景下 secret 不具备机密性）
const GOOGLE_OAUTH_CLIENT_ID: &str = env!("GOOGLE_OAUTH_CLIENT_ID");
const GOOGLE_OAUTH_CLIENT_SECRET: &str = env!("GOOGLE_OAUTH_CLIENT_SECRET");
const GOOGLE_OAUTH_SCOPES: &str = "openid email profile";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_INFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const OAUTH_CALLBACK_TIMEOUT_SECS: u64 = 300;

// Token 存储于系统钥匙串（macOS Keychain / Windows Credential Manager / Secret Service）
const KEYCHAIN_SERVICE: &str = "com.opera.chathub-desktop";
const KEYCHAIN_ACCESS_TOKEN: &str = "oauth_access_token";
const KEYCHAIN_REFRESH_TOKEN: &str = "oauth_refresh_token";

pub struct OauthState {
    // state -> verifier 映射，支持并发授权并防止 CSRF
    pending: Mutex<HashMap<String, String>>,
}

impl Default for OauthState {
    fn default() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    token_type: String,
    scope: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
}

#[derive(Deserialize, serde::Serialize, Clone, Debug)]
struct UserInfo {
    id: String,
    email: String,
    #[serde(default)]
    verified_email: bool,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    locale: Option<String>,
}

fn generate_pkce_verifier() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn generate_pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

fn generate_random_state() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn keyring_entry(account: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYCHAIN_SERVICE, account).map_err(|e| format!("系统钥匙串不可用: {e}"))
}

fn save_token_to_keychain(account: &str, token: &str) -> Result<(), String> {
    keyring_entry(account)?
        .set_password(token)
        .map_err(|e| format!("写入系统钥匙串失败: {e}"))
}

fn delete_token_from_keychain(account: &str) {
    let entry = match keyring_entry(account) {
        Ok(entry) => entry,
        Err(e) => {
            warn!("[OAuth] {e}");
            return;
        }
    };

    if let Err(e) = entry.delete_credential() {
        if !matches!(e, keyring::Error::NoEntry) {
            warn!("[OAuth] 清除钥匙串中的 {account} 失败: {e}");
        }
    }
}

fn remove_pending_state(app: &AppHandle, state: &str) {
    if let Ok(mut pending) = app.state::<OauthState>().pending.lock() {
        pending.remove(state);
    }
}

fn emit_login_error(app: &AppHandle, msg: String) {
    error!("[OAuth] {msg}");
    let _ = app.emit("login_error", msg);
}

/// 旧版本将 Token 明文存放在 settings store 中，启动时迁移到钥匙串
pub fn migrate_plaintext_tokens(app: &AppHandle) {
    let Ok(store) = app.store(SETTINGS_FILENAME) else {
        return;
    };

    for account in [KEYCHAIN_ACCESS_TOKEN, KEYCHAIN_REFRESH_TOKEN] {
        let Some(token) = store
            .get(account)
            .and_then(|value| value.as_str().map(str::to_string))
        else {
            continue;
        };

        match save_token_to_keychain(account, &token) {
            Ok(()) => {
                store.delete(account);
                info!("[OAuth] 已将 {account} 迁移到系统钥匙串");
            }
            Err(e) => warn!("[OAuth] {account} 迁移失败，暂时保留原存储: {e}"),
        }
    }

    let _ = store.save();
}

/**
 * 发起 Google OAuth 登录：系统浏览器完成授权，本地随机端口接收回调
 */
pub fn start_login(app: &AppHandle) {
    let verifier = generate_pkce_verifier();
    let challenge = generate_pkce_challenge(&verifier);
    let state_param = generate_random_state();

    let listener = match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            error!("[OAuth] 绑定本地端口失败: {e}");
            return;
        }
    };
    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => {
            error!("[OAuth] 读取回调端口失败: {e}");
            return;
        }
    };
    let _ = listener.set_nonblocking(true);
    let redirect_uri = format!("http://localhost:{port}");

    let oauth_state = app.state::<OauthState>();
    if let Err(e) = oauth_state
        .pending
        .lock()
        .map(|mut pending| pending.insert(state_param.clone(), verifier))
    {
        error!("[OAuth] 注册授权状态失败: {e}");
        return;
    }

    let oauth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&access_type=offline&state={}",
        GOOGLE_OAUTH_CLIENT_ID,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(GOOGLE_OAUTH_SCOPES),
        challenge,
        urlencoding::encode(&state_param)
    );

    info!("[Google OAuth] 打开授权页面，回调监听端口: {port}");
    let _ = tauri_plugin_opener::open_url(oauth_url, None::<&str>);

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let tokio_listener = match tokio::net::TcpListener::from_std(listener) {
            Ok(l) => l,
            Err(e) => {
                error!("[OAuth] 转换 tokio listener 失败: {e}");
                return;
            }
        };

        let accept_result = tokio::time::timeout(
            std::time::Duration::from_secs(OAUTH_CALLBACK_TIMEOUT_SECS),
            tokio_listener.accept(),
        )
        .await;

        match accept_result {
            Ok(Ok((mut stream, _))) => {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                let (mut code, mut cb_state) = (None, None);
                if let Some(request_line) = request.lines().next() {
                    if let Some(path) = request_line.split_whitespace().nth(1) {
                        let full_url = format!("http://localhost{path}");
                        if let Ok(parsed) = url::Url::parse(&full_url) {
                            code = parsed
                                .query_pairs()
                                .find(|(key, _)| key == "code")
                                .map(|(_, value)| value.into_owned());
                            cb_state = parsed
                                .query_pairs()
                                .find(|(key, _)| key == "state")
                                .map(|(_, value)| value.into_owned());
                        }
                    }
                }

                reply_callback_page(&mut stream).await;
                handle_callback(&app_handle, &redirect_uri, code, cb_state).await;
            }
            Ok(Err(e)) => {
                emit_login_error(&app_handle, format!("接受回调连接失败: {e}"));
            }
            Err(_) => {
                remove_pending_state(&app_handle, &state_param);
                emit_login_error(&app_handle, "登录超时，请重试".to_string());
            }
        }
    });
}

async fn reply_callback_page(stream: &mut (impl tokio::io::AsyncWrite + Unpin + Send)) {
    use tokio::io::AsyncWriteExt;
    let html_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>授权成功</title><style>body{display:flex;justify-content:center;align-items:center;min-height:100vh;font-family:system-ui;background:#f0f2f5;margin:0}div{text-align:center;padding:2rem;background:white;border-radius:12px;box-shadow:0 2px 8px rgba(0,0,0,0.1)}h1{color:#1a73e8}p{color:#666}</style></head><body><div><h1>✅ 授权成功</h1><p>您可以关闭此页面并返回应用。</p></div></body></html>";
    let _ = stream.write_all(html_response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

async fn handle_callback(
    app: &AppHandle,
    redirect_uri: &str,
    code: Option<String>,
    state: Option<String>,
) {
    let (Some(auth_code), Some(recv_state)) = (code.as_deref(), state.as_deref()) else {
        if let Some(recv_state) = state.as_deref() {
            remove_pending_state(app, recv_state);
        }
        emit_login_error(app, "回调缺少 code 或 state 参数".to_string());
        return;
    };

    // 校验 state 防 CSRF，并取回对应的 PKCE verifier
    let verifier = match app.state::<OauthState>().pending.lock() {
        Ok(pending) => pending.get(recv_state).cloned(),
        Err(e) => {
            emit_login_error(app, format!("读取授权状态失败: {e}"));
            return;
        }
    };
    let Some(verifier) = verifier else {
        emit_login_error(app, "授权验证失败: state 参数不匹配或已过期".to_string());
        return;
    };

    info!("[OAuth] 使用 verifier 换取 token...");
    let client = build_http_client(app);

    let mut params = vec![
        ("client_id", GOOGLE_OAUTH_CLIENT_ID.to_string()),
        ("code", auth_code.to_string()),
        ("code_verifier", verifier),
        ("grant_type", "authorization_code".to_string()),
        ("redirect_uri", redirect_uri.to_string()),
    ];
    if !GOOGLE_OAUTH_CLIENT_SECRET.is_empty() {
        params.push(("client_secret", GOOGLE_OAUTH_CLIENT_SECRET.to_string()));
    }

    let response = match client.post(GOOGLE_TOKEN_URL).form(&params).send().await {
        Ok(response) => response,
        Err(e) => {
            emit_login_error(app, format!("请求 Token 失败: {e}"));
            return;
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        emit_login_error(app, format!("换取 Token 失败 ({status}): {detail}"));
        return;
    }

    let token = match response.json::<TokenResponse>().await {
        Ok(token) => token,
        Err(e) => {
            emit_login_error(app, format!("解析 Token 响应失败: {e}"));
            return;
        }
    };

    // 授权流程结束，清理 pending 记录
    remove_pending_state(app, recv_state);

    if let Err(e) = save_token_to_keychain(KEYCHAIN_ACCESS_TOKEN, &token.access_token) {
        emit_login_error(app, e);
        return;
    }
    if let Some(refresh_token) = token.refresh_token.as_deref() {
        if let Err(e) = save_token_to_keychain(KEYCHAIN_REFRESH_TOKEN, refresh_token) {
            emit_login_error(app, e);
            return;
        }
    }

    // 用户信息（非敏感）保留在 settings store 中
    let user_response = match client
        .get(GOOGLE_USER_INFO_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            emit_login_error(app, format!("获取用户信息请求失败: {e}"));
            return;
        }
    };

    if !user_response.status().is_success() {
        emit_login_error(
            app,
            format!("获取用户信息失败，状态码: {}", user_response.status()),
        );
        return;
    }

    let user_info = match user_response.json::<UserInfo>().await {
        Ok(info) => info,
        Err(e) => {
            emit_login_error(app, format!("解析用户信息失败: {e}"));
            return;
        }
    };

    match app.store(SETTINGS_FILENAME) {
        Ok(store) => match serde_json::to_value(&user_info) {
            Ok(value) => {
                store.set("oauth_user_info", value);
                let _ = store.save();
            }
            Err(e) => {
                emit_login_error(app, format!("序列化用户信息失败: {e}"));
                return;
            }
        },
        Err(e) => {
            emit_login_error(app, format!("保存用户信息失败: {e}"));
            return;
        }
    }

    info!("[OAuth] 登录成功: {}", user_info.email);
    let _ = app.emit("login_success", user_info);
    update_tray_menu(app);
}

/**
 * 退出登录：清除钥匙串中的 Token 与本地用户信息
 */
pub fn logout(app: &AppHandle) {
    delete_token_from_keychain(KEYCHAIN_ACCESS_TOKEN);
    delete_token_from_keychain(KEYCHAIN_REFRESH_TOKEN);

    if let Ok(store) = app.store(SETTINGS_FILENAME) {
        store.delete("oauth_user_info");
        // 兼容清理旧版本明文存储的 Token
        store.delete(KEYCHAIN_ACCESS_TOKEN);
        store.delete(KEYCHAIN_REFRESH_TOKEN);
        let _ = store.save();
    }

    info!("[OAuth] 已退出登录");
    update_tray_menu(app);
}
