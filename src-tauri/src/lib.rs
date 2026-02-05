use tauri::{
    menu::{Menu, MenuItem, Submenu, CheckMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use tauri_plugin_store::StoreExt;
use tauri_plugin_autostart::MacosLauncher;
use serde_json::json;

const CHATGPT_URL: &str = "https://chatgpt.com";
const DEEPSEEK_URL: &str = "https://chat.deepseek.com/";
const GROK_URL: &str = "https://grok.com/";
const GEMINI_URL: &str = "https://gemini.google.com/app";
const SETTINGS_FILENAME: &str = "settings.json";
const DEFAULT_SHORTCUT: &str = "CommandOrControl+Shift+G";
// macOS Chrome 131 UA
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

// Google OAuth 配置
// 注意: 需要在 Google Cloud Console 创建 OAuth 2.0 Client ID
// 1. 访问 https://console.cloud.google.com/apis/credentials
// 2. 创建 OAuth 2.0 Client ID (类型: Desktop app)
// 3. 添加重定向 URI: chathub://oauth/callback
const GOOGLE_OAUTH_CLIENT_ID: &str = "908764850104-29trlc7ocji1g7q5gmgrs38b86icf8g4.apps.googleusercontent.com";
const GOOGLE_OAUTH_SCOPES: &str = "openid%20email%20profile";
const GOOGLE_OAUTH_REDIRECT_URI: &str = "chathub://oauth/callback";

/**
 * 保存最后使用的URL
 */
fn save_last_url(app: &AppHandle, url: &str) {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    store.set("last_url", json!(url));
    let _ = store.save();
}
/**
 * 创建托盘菜单
 */
fn create_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    // 获取当前语言设置
    let store = app.store(SETTINGS_FILENAME).unwrap();
    let current_lang = store.get("language")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "zh".to_string());
    
    let is_english = current_lang == "en";
    
    // 根据语言获取文本
        let (quit_text, reload_text, open_browser_text, autostart_text, models_text, lang_text, proxy_text, shortcut_text, google_login_text) = 
        if is_english {
            ("Quit", "Reload", "Open in Browser", "Launch at Login", "Models", "Language", "Proxy Settings", "Shortcut Settings", "Login with Google")
        } else {
            ("退出", "重新加载", "在浏览器打开", "开机自启", "模型", "语言", "代理设置", "快捷键设置", "登录 Google")
        };

    let quit_item = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;
    let reload_item = MenuItem::with_id(app, "reload", reload_text, true, None::<&str>)?;
    let open_browser_item = MenuItem::with_id(app, "open_browser", open_browser_text, true, None::<&str>)?;
    
    // 检查是否启用了开机自启
    use tauri_plugin_autostart::ManagerExt;
    let autostart_manager = app.autolaunch();
    let is_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
    let autostart_item = CheckMenuItem::with_id(app, "autostart", autostart_text, true, is_autostart_enabled, None::<&str>)?;
    
    // 代理设置
    let proxy_item = MenuItem::with_id(app, "proxy", proxy_text, true, None::<&str>)?;
    
    // 快捷键设置
    let shortcut_item = MenuItem::with_id(app, "shortcut", shortcut_text, true, None::<&str>)?;
    
    // Google 登录按钮
    let google_login_item = MenuItem::with_id(app, "google_login", google_login_text, true, None::<&str>)?;
    
    // 模型子菜单 - 从 last_url 获取当前模型
    let current_url = store.get("last_url")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| CHATGPT_URL.to_string());
    // chatgpt 子菜单
    let chatgpt_item = CheckMenuItem::with_id(app, "chatgpt", "ChatGPT", true, current_url == CHATGPT_URL, None::<&str>)?;
    // deepseek 子菜单
    let deepseek_item = CheckMenuItem::with_id(app, "deepseek", "DeepSeek", true, current_url == DEEPSEEK_URL, None::<&str>)?;
    // grok 子菜单
    let grok_item = CheckMenuItem::with_id(app, "grok", "Grok", true, current_url == GROK_URL, None::<&str>)?;
    // gemini 子菜单
    let gemini_item = CheckMenuItem::with_id(app, "gemini", "Gemini", true, current_url == GEMINI_URL, None::<&str>)?;
    // 模型菜单
    let models_submenu = Submenu::with_items(
        app,
        models_text,
        true,
        &[&chatgpt_item, &deepseek_item, &grok_item, &gemini_item],
    )?;
    // 语言子菜单
    let lang_zh_item = CheckMenuItem::with_id(app, "lang_zh", "中文", true, !is_english, None::<&str>)?;
    // 英文子菜单
    let lang_en_item = CheckMenuItem::with_id(app, "lang_en", "English", true, is_english, None::<&str>)?;
    // 语言菜单
    let language_submenu = Submenu::with_items(
        app,
        lang_text,
        true,
        &[&lang_zh_item, &lang_en_item],
    )?;

    Menu::with_items(app, &[
        &models_submenu,
        &language_submenu,
        &reload_item,
        &open_browser_item,
        &google_login_item,
        &shortcut_item,
        &proxy_item,
        &autostart_item,
        &quit_item,
    ])
}

/**
 * 更新托盘菜单
 */
fn update_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("tray") {
        if let Ok(menu) = create_tray_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

/**
 * 切换模型
 */
fn switch_model(app: &AppHandle, url: &str) {
    if let Some(window) = app.get_webview_window("main") {
        save_last_url(app, url);
        
        // 向当前页面注入加载遮罩层
        let loading_script = include_str!("loading_overlay.ts")
            .replace("__TARGET_URL__", url);
        
        let _ = window.eval(&loading_script);
        let _ = window.show();
        let _ = window.set_focus();
    }
    // 更新菜单以反映新的模型选择
    update_tray_menu(app);
}

/**
 * 切换窗口显示状态
 */
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[tauri::command]
fn get_last_model_url(app: AppHandle) -> String {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    if let Some(url) = store.get("last_url") {
        return url.as_str().unwrap_or(CHATGPT_URL).to_string();
    }
    CHATGPT_URL.to_string()
}


fn log_debug(msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/opera/Documents/self/chathub-desktop/tauri_debug.log")
        .unwrap();
    let _ = writeln!(file, "{}", msg);
}

#[tauri::command]
fn save_proxy(app: AppHandle, proxy: String) {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    store.set("proxy", json!(proxy));
    let _ = store.save();
}

#[tauri::command]
fn close_proxy_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("proxy") {
        let _ = win.close();
    }
}

#[tauri::command]
fn save_shortcut(app: AppHandle, shortcut: String) {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    store.set("shortcut", json!(shortcut));
    let _ = store.save();
}

#[tauri::command]
fn close_shortcut_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("shortcut") {
        let _ = win.close();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state() == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_last_model_url, 
            save_proxy, 
            close_proxy_window,
            save_shortcut,
            close_shortcut_window
        ])
        .setup(|app| {
            // 注册深度链接回调处理
            #[cfg(desktop)]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let app_handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    let urls = event.urls();
                    for url in urls {
                        let url_str = url.to_string();
                        println!("[深度链接] 收到回调: {}", url_str);
                        
                        // 处理 OAuth 回调: chathub://oauth/callback?code=xxx
                        if url_str.starts_with("chathub://oauth/callback") {
                            // 解析 URL 参数
                            if let Some(query) = url.query() {
                                println!("[深度链接] OAuth 回调参数: {}", query);
                                // TODO: 解析 code 并交换 token
                                // 这里需要后端服务支持
                            }
                            
                            // 显示窗口并刷新页面
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.eval("window.location.reload()");
                            }
                        }
                    }
                });
            }
            
            // 在 macOS 上隐藏 Dock 图标
            // #[cfg(target_os = "macos")]
            // app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // 创建主窗口
            let _main_window = tauri::webview::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::default(),
            )
            .title("ChatHub Desktop")
            .inner_size(900.0, 600.0)
            .visible(false)
            .user_agent(USER_AGENT)
            .on_navigation(|url| {
                let url_str = url.to_string();
                
                // 允许本地和内部协议
                if url_str.starts_with("tauri://") || url_str.starts_with("http://localhost") {
                    return true;
                }

                // Google 登录必须在外部浏览器中打开
                // Google 自 2023 年 2 月起阻止在嵌入式 WebView 中进行 OAuth 认证
                if url_str.contains("accounts.google.com") {
                    let _ = tauri_plugin_opener::open_url(url_str, None::<&str>);
                    return false;
                }

                // 应保留在应用内的域名白名单
                let whitelist = [
                    "chatgpt.com",
                    "openai.com",
                    "deepseek.com",
                    "grok.com",
                    "gemini.google.com",
                    "googleusercontent.com",
                    "gstatic.com",
                    "challenges.cloudflare.com", // Cloudflare 验证
                    "accounts.youtube.com",      // Google 登录相关
                ];

                for domain in whitelist {
                    if url_str.contains(domain) {
                        return true;
                    }
                }

                // 仅在默认浏览器中打开真正的外部链接
                let _ = tauri_plugin_opener::open_url(url_str, None::<&str>);
                false
            })
            .build()?;

            // 创建初始菜单
            println!("[调试] 开始创建托盘菜单...");
            let menu = match create_tray_menu(app.handle()) {
                Ok(m) => {
                    println!("[调试] 托盘菜单创建成功");
                    m
                },
                Err(e) => {
                    println!("[错误] 托盘菜单创建失败: {}", e);
                    return Err(e.into());
                }
            };

            println!("[调试] 开始构建托盘图标...");
            let _tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .show_menu_on_left_click(false)
                .on_menu_event(|app: &AppHandle, event| {
                    let id = event.id.as_ref();
                    match id {
                        // 退出
                        "quit" => app.exit(0),
                        // 重载
                        "reload" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.eval("window.location.reload()");
                            }
                        }
                        // 打开浏览器
                        "open_browser" => {
                            let store = app.store(SETTINGS_FILENAME).unwrap();
                            let url = store.get("last_url")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| CHATGPT_URL.to_string());
                            let _ = tauri_plugin_opener::open_url(url, None::<&str>);
                        }
                        // Google OAuth 登录
                        "google_login" => {
                            // 构建 Google OAuth 授权 URL
                            let oauth_url = format!(
                                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline",
                                GOOGLE_OAUTH_CLIENT_ID,
                                GOOGLE_OAUTH_REDIRECT_URI,
                                GOOGLE_OAUTH_SCOPES
                            );
                            println!("[Google OAuth] 打开授权页面: {}", oauth_url);
                            let _ = tauri_plugin_opener::open_url(oauth_url, None::<&str>);
                        }
                        // 开机自启
                        "autostart" => {
                            use tauri_plugin_autostart::ManagerExt;
                            let autostart_manager = app.autolaunch();
                            if autostart_manager.is_enabled().unwrap_or(false) {
                                let _ = autostart_manager.disable();
                            } else {
                                let _ = autostart_manager.enable();
                            }
                        }
                        // 模型切换
                        "chatgpt" => switch_model(app, CHATGPT_URL),
                        "deepseek" => switch_model(app, DEEPSEEK_URL),
                        "grok" => switch_model(app, GROK_URL),
                        "gemini" => switch_model(app, GEMINI_URL),
                        // 语言切换
                        "lang_zh" => {
                            let store = app.store(SETTINGS_FILENAME).unwrap();
                            store.set("language", json!("zh"));
                            let _ = store.save();
                            update_tray_menu(app);
                        }
                        "lang_en" => {
                            let store = app.store(SETTINGS_FILENAME).unwrap();
                            store.set("language", json!("en"));
                            let _ = store.save();
                            update_tray_menu(app);
                        }
                        // 快捷键设置
                        "shortcut" => {
                            let store = app.store(SETTINGS_FILENAME).unwrap();
                            let current_lang = store.get("language")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| "zh".to_string());
                            let current_shortcut = store.get("shortcut")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string());
                            
                            let is_english = current_lang == "en";
                            
                            if let Some(shortcut_win) = app.get_webview_window("shortcut") {
                                let _ = shortcut_win.set_focus();
                                return;
                            }

                            let title = if is_english { "Shortcut Settings" } else { "快捷键设置" };
                            let ok_text = if is_english { "Save" } else { "保存" };
                            let cancel_text = if is_english { "Cancel" } else { "取消" };
                            let hint_text = if is_english { "Press keys to set new shortcut" } else { "按下按键组合以设置快捷键" };

                            let query_params = format!(
                                "?title={title}&hint={hint}&current={current}&cancelText={cancelText}&okText={okText}",
                                title = urlencoding::encode(&title),
                                hint = urlencoding::encode(&hint_text),
                                current = urlencoding::encode(&current_shortcut),
                                cancelText = urlencoding::encode(&cancel_text),
                                okText = urlencoding::encode(&ok_text)
                            );

                            // 在打开设置前取消注册当前快捷键
                            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                            if let Ok(current_s) = current_shortcut.parse::<Shortcut>() {
                                let _ = app.global_shortcut().unregister(current_s);
                            }

                            let window = tauri::webview::WebviewWindowBuilder::new(
                                app,
                                "shortcut",
                                tauri::WebviewUrl::App(format!("shortcut.html{}", query_params).into())
                            )
                            .title(title)
                            .inner_size(420.0, 200.0)
                            .resizable(false)
                            .minimizable(false)
                            .always_on_top(true)
                            .center()
                            .build()
                            .unwrap();

                            let app_handle = app.clone();
                            window.on_window_event(move |event| {
                                if let tauri::WindowEvent::Destroyed = event {
                                    let store = app_handle.store(SETTINGS_FILENAME).unwrap();
                                    let shortcut_str = store.get("shortcut")
                                        .and_then(|v: serde_json::Value| v.as_str().map(|s| s.to_string()))
                                        .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string());
                                    
                                    if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                                        log_debug(&format!("重新启用全局快捷键: {}", shortcut_str));
                                        let _ = app_handle.global_shortcut().register(shortcut);
                                    }
                                }
                            });
                        }
                        "proxy" => {
                            let store = app.store(SETTINGS_FILENAME).unwrap();
                            let current_lang = store.get("language")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| "zh".to_string());
                            let current_proxy = store.get("proxy")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_default();
                            
                            let is_english = current_lang == "en";
                            
                            if let Some(proxy_win) = app.get_webview_window("proxy") {
                                let _ = proxy_win.set_focus();
                                return;
                            }

                            let title = if is_english { "Proxy Settings" } else { "代理设置" };
                            let ok_text = if is_english { "Save" } else { "保存" };
                            let cancel_text = if is_english { "Cancel" } else { "取消" };
                            let hint_text = if is_english { "Enter proxy address (e.g. socks5://127.0.0.1:7897)" } else { "输入代理地址（如 socks5://127.0.0.1:7897）" };
                            let placeholder_text = if is_english { "Leave empty to disable proxy" } else { "留空则禁用代理" };

                            let query_params = format!(
                                "?hint={hint_text}&current={current_proxy}&placeholder={placeholder_text}&cancelText={cancel_text}&okText={ok_text}",
                                hint_text = urlencoding::encode(&hint_text),
                                current_proxy = urlencoding::encode(&current_proxy),
                                placeholder_text = urlencoding::encode(&placeholder_text),
                                cancel_text = urlencoding::encode(&cancel_text),
                                ok_text = urlencoding::encode(&ok_text)
                            );

                            let _ = tauri::webview::WebviewWindowBuilder::new(
                                app,
                                "proxy",
                                tauri::WebviewUrl::App(format!("proxy.html{}", query_params).into())
                            )
                            .title(title)
                            .inner_size(420.0, 200.0)
                            .resizable(false)
                            .minimizable(false)
                            .always_on_top(true)
                            .center()
                            .build();
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
                    // 仅在按钮抬起时触发，防止重复触发（按下 + 抬起）
                    if let TrayIconEvent::Click { 
                        button: MouseButton::Left, 
                        button_state: MouseButtonState::Up, 
                        .. 
                    } = event {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            let store = app.handle().store(SETTINGS_FILENAME).unwrap();
            let shortcut_str = store.get("shortcut")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string());

            if !shortcut_str.is_empty() {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                    log_debug(&format!("注册初始快捷键: {}", shortcut_str));
                    let _ = app.global_shortcut().register(shortcut);
                }
            }

            // 调试：启动时强制显示窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}
