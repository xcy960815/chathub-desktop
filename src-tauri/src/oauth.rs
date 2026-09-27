use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
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
// 3. 在 src-tauri/.env 中配置 GOOGLE_OAUTH_CLIENT_ID 和 GOOGLE_OAUTH_CLIENT_SECRET
const GOOGLE_OAUTH_CLIENT_ID: &str = env!("GOOGLE_OAUTH_CLIENT_ID");
const GOOGLE_OAUTH_SCOPES: &str = "openid email profile";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_INFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const GOOGLE_OAUTH_CLIENT_SECRET: &str = env!("GOOGLE_OAUTH_CLIENT_SECRET");

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

/**
 * 发起 Google OAuth 登录（localhost 回调方案）
 */
pub fn start_login(app: &AppHandle) {
    // 生成 PKCE verifier 和 challenge
    let verifier = generate_pkce_verifier();
    let challenge = generate_pkce_challenge(&verifier);
    let state_param = generate_random_state();

    // 绑定本地随机端口
    let listener = match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            println!("[OAuth] 绑定本地端口失败: {}", e);
            return;
        }
    };
    let port = listener.local_addr().unwrap().port();
    let _ = listener.set_nonblocking(true);
    let redirect_uri = format!("http://localhost:{}", port);

    // 保存 state -> verifier 映射
    let oauth_state = app.state::<OauthState>();
    if let Ok(mut pending) = oauth_state.pending.lock() {
        pending.insert(state_param.clone(), verifier);
    }

    // 构建 Google OAuth 授权 URL
    let oauth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&access_type=offline&state={}",
        GOOGLE_OAUTH_CLIENT_ID,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(GOOGLE_OAUTH_SCOPES),
        challenge,
        urlencoding::encode(&state_param)
    );

    println!("[Google OAuth] 打开授权页面: {}", oauth_url);
    println!("[Google OAuth] 回调监听端口: {}", port);
    let _ = tauri_plugin_opener::open_url(oauth_url, None::<&str>);

    // 异步等待 OAuth 回调
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let tokio_listener = match tokio::net::TcpListener::from_std(listener) {
            Ok(l) => l,
            Err(e) => {
                println!("[OAuth] 转换 tokio listener 失败: {}", e);
                return;
            }
        };

        // 等待回调（5分钟超时）
        let accept_result =
            tokio::time::timeout(std::time::Duration::from_secs(300), tokio_listener.accept())
                .await;

        match accept_result {
            Ok(Ok((mut stream, _))) => {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                // 解析 HTTP 请求中的 URL 参数
                let mut code: Option<String> = None;
                let mut cb_state: Option<String> = None;

                if let Some(request_line) = request.lines().next() {
                    if let Some(path) = request_line.split_whitespace().nth(1) {
                        let full_url = format!("http://localhost{}", path);
                        if let Ok(parsed) = url::Url::parse(&full_url) {
                            code = parsed
                                .query_pairs()
                                .find(|(k, _)| k == "code")
                                .map(|(_, v)| v.into_owned());
                            cb_state = parsed
                                .query_pairs()
                                .find(|(k, _)| k == "state")
                                .map(|(_, v)| v.into_owned());
                        }
                    }
                }

                // 返回成功页面给浏览器
                let html_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>授权成功</title><style>body{display:flex;justify-content:center;align-items:center;min-height:100vh;font-family:system-ui;background:#f0f2f5;margin:0}div{text-align:center;padding:2rem;background:white;border-radius:12px;box-shadow:0 2px 8px rgba(0,0,0,0.1)}h1{color:#1a73e8}p{color:#666}</style></head><body><div><h1>✅ 授权成功</h1><p>您可以关闭此页面并返回应用。</p></div></body></html>";
                let _ = stream.write_all(html_response.as_bytes()).await;
                let _ = stream.shutdown().await;

                // 处理 Token 交换
                match (code, cb_state) {
                    (Some(auth_code), Some(recv_state)) => {
                        // 验证 state 防 CSRF
                        let verifier = {
                            let oauth_state = app_handle.state::<OauthState>();
                            let pending = oauth_state.pending.lock().unwrap();
                            pending.get(&recv_state).cloned()
                        };

                        if let Some(verifier) = verifier {
                            println!("[OAuth] 使用 verifier 换取 token...");
                            let client = build_http_client(&app_handle);
                            let redirect = redirect_uri.clone();

                            let params = [
                                ("client_id", GOOGLE_OAUTH_CLIENT_ID),
                                ("client_secret", GOOGLE_OAUTH_CLIENT_SECRET),
                                ("code", auth_code.as_str()),
                                ("code_verifier", verifier.as_str()),
                                ("grant_type", "authorization_code"),
                                ("redirect_uri", redirect.as_str()),
                            ];

                            match client.post(GOOGLE_TOKEN_URL).form(&params).send().await {
                                Ok(res) => {
                                    if res.status().is_success() {
                                        match res.json::<TokenResponse>().await {
                                            Ok(token_res) => {
                                                println!("[OAuth] 获取 Token 成功");

                                                // 移除 pending 记录
                                                {
                                                    let oauth_state =
                                                        app_handle.state::<OauthState>();
                                                    let mut pending =
                                                        oauth_state.pending.lock().unwrap();
                                                    pending.remove(&recv_state);
                                                }

                                                // 持久化 Token
                                                {
                                                    let store = app_handle
                                                        .store(SETTINGS_FILENAME)
                                                        .unwrap();
                                                    store.set(
                                                        "oauth_access_token",
                                                        json!(&token_res.access_token),
                                                    );
                                                    if let Some(ref rt) = token_res.refresh_token {
                                                        store.set("oauth_refresh_token", json!(rt));
                                                    }
                                                    let _ = store.save();
                                                }

                                                // 获取用户信息
                                                match client
                                                    .get(GOOGLE_USER_INFO_URL)
                                                    .bearer_auth(&token_res.access_token)
                                                    .send()
                                                    .await
                                                {
                                                    Ok(user_res) => {
                                                        if user_res.status().is_success() {
                                                            match user_res.json::<UserInfo>().await
                                                            {
                                                                Ok(user_info) => {
                                                                    println!(
                                                                        "[OAuth] 用户信息: {:?}",
                                                                        user_info
                                                                    );
                                                                    {
                                                                        let store = app_handle
                                                                            .store(
                                                                                SETTINGS_FILENAME,
                                                                            )
                                                                            .unwrap();
                                                                        store.set(
                                                                            "oauth_user_info",
                                                                            serde_json::to_value(
                                                                                &user_info,
                                                                            )
                                                                            .unwrap(),
                                                                        );
                                                                        let _ = store.save();
                                                                    }
                                                                    let _ = app_handle.emit(
                                                                        "login_success",
                                                                        user_info,
                                                                    );
                                                                    // 刷新托盘菜单显示用户信息
                                                                    update_tray_menu(&app_handle);
                                                                }
                                                                Err(e) => {
                                                                    let msg = format!(
                                                                        "解析用户信息失败: {}",
                                                                        e
                                                                    );
                                                                    println!("[OAuth] {}", msg);
                                                                    let _ = app_handle
                                                                        .emit("login_error", msg);
                                                                }
                                                            }
                                                        } else {
                                                            let msg = format!(
                                                                "获取用户信息失败，状态码: {}",
                                                                user_res.status()
                                                            );
                                                            println!("[OAuth] {}", msg);
                                                            let _ =
                                                                app_handle.emit("login_error", msg);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let msg =
                                                            format!("获取用户信息请求失败: {}", e);
                                                        println!("[OAuth] {}", msg);
                                                        let _ = app_handle.emit("login_error", msg);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                let msg = format!("解析 Token 响应失败: {}", e);
                                                println!("[OAuth] {}", msg);
                                                let _ = app_handle.emit("login_error", msg);
                                            }
                                        }
                                    } else {
                                        let status = res.status();
                                        let detail = res.text().await.unwrap_or_default();
                                        let msg =
                                            format!("换取 Token 失败 ({}): {}", status, detail);
                                        println!("[OAuth] {}", msg);
                                        let _ = app_handle.emit("login_error", msg);
                                    }
                                }
                                Err(e) => {
                                    let msg = format!("请求 Token 失败: {}", e);
                                    println!("[OAuth] {}", msg);
                                    let _ = app_handle.emit("login_error", msg);
                                }
                            }
                        } else {
                            let msg = "授权验证失败: state 参数不匹配或已过期".to_string();
                            println!("[OAuth] {}", msg);
                            let _ = app_handle.emit("login_error", msg);
                        }
                    }
                    _ => {
                        let msg = "回调缺少 code 或 state 参数".to_string();
                        println!("[OAuth] {}", msg);
                        let _ = app_handle.emit("login_error", msg);
                    }
                }
            }
            Ok(Err(e)) => {
                let msg = format!("接受回调连接失败: {}", e);
                println!("[OAuth] {}", msg);
                let _ = app_handle.emit("login_error", msg);
            }
            Err(_) => {
                println!("[OAuth] 等待回调超时（5分钟）");
                let _ = app_handle.emit("login_error", "登录超时，请重试".to_string());
            }
        }
    });
}

/**
 * 退出登录，清除保存的 OAuth 凭据
 */
pub fn logout(app: &AppHandle) {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    store.delete("oauth_access_token");
    store.delete("oauth_refresh_token");
    store.delete("oauth_user_info");
    let _ = store.save();
    println!("[OAuth] 已退出登录");
    update_tray_menu(app);
}
