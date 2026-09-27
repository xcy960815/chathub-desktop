mod commands;
mod dialog_windows;
mod models;
mod oauth;
mod settings;
mod tray;
mod updater;

use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;

use settings::{
    load_app_settings, save_app_settings, CHATGPT_MODEL_ID, DEEPSEEK_MODEL_ID, DOUBAO_MODEL_ID,
    GEMINI_MODEL_ID, GROK_MODEL_ID, QWEN_MODEL_ID,
};

// HTTP 请求使用的 UA（与 WebView 分离，避免 Cloudflare 检测到不一致指纹）
#[cfg(target_os = "macos")]
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.4 Safari/605.1.15";
#[cfg(not(target_os = "macos"))]
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

#[cfg(debug_assertions)]
const APP_SHELL_URL: &str = "http://localhost:1420/";
#[cfg(not(debug_assertions))]
const APP_SHELL_URL: &str = "tauri://localhost/";

fn build_http_client(app: &AppHandle) -> reqwest::Client {
    let settings = load_app_settings(app);
    let mut builder = reqwest::Client::builder().user_agent(USER_AGENT);

    if let Some(proxy) = settings.proxy.as_deref() {
        if let Ok(proxy_value) = reqwest::Proxy::all(proxy)
            .or_else(|_| reqwest::Proxy::http(proxy))
            .or_else(|_| reqwest::Proxy::https(proxy))
        {
            builder = builder.proxy(proxy_value);
        }
    }

    builder.build().unwrap_or_else(|_| reqwest::Client::new())
}

fn save_current_model_url(app: &AppHandle, url: &str) {
    let mut settings = load_app_settings(app);
    if settings.set_current_url(url.to_string()) {
        save_app_settings(app, &settings);
    }
}

fn show_js_alert(app: &AppHandle, message: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let payload = serde_json::to_string(message)
            .unwrap_or_else(|_| "\"操作失败，请稍后重试\"".to_string());
        let _ = window.eval(&format!("window.alert({payload});"));
    }
}

fn toggle_always_on_top(app: &AppHandle) {
    let mut settings = load_app_settings(app);
    settings.always_on_top = !settings.always_on_top;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(settings.always_on_top);
    }

    save_app_settings(app, &settings);
    tray::update_tray_menu(app);
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

fn build_initialization_script() -> String {
    // macOS WKWebView 上注入指纹伪装会破坏 Cloudflare Turnstile，仅保留 URL 追踪脚本
    #[cfg(target_os = "macos")]
    {
        include_str!("navigation_bridge.js").to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        format!(
            "window.__CHATHUB_USER_AGENT__ = {ua:?};\n{stealth}\n{navigation}",
            ua = USER_AGENT,
            stealth = include_str!("stealth_bridge.js"),
            navigation = include_str!("navigation_bridge.js")
        )
    }
}

/**
 * 是否为应用自身页面（外壳或弹窗），其 URL 不应持久化为模型地址
 */
fn is_app_owned_url(url: &tauri::Url) -> bool {
    if url.scheme() == "tauri" {
        return true;
    }

    matches!(url.scheme(), "http" | "https")
        && matches!(
            url.host_str(),
            Some("localhost") | Some("tauri.localhost") | Some("127.0.0.1")
        )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(oauth::OauthState::default())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
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
            models::get_model_launch_plan,
            models::open_model_in_browser,
            commands::save_proxy,
            commands::close_proxy_window,
            commands::get_proxy_dialog_data,
            commands::remove_proxy_history,
            commands::save_shortcut,
            commands::close_shortcut_window,
            commands::get_shortcut_dialog_data,
            commands::remove_shortcut_history
        ])
        .setup(|app| {
            // 创建主窗口
            let navigation_app = app.handle().clone();
            let initialization_script = build_initialization_script();
            let main_window = {
                let builder = tauri::webview::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::default(),
                )
                .title("ChatHub Desktop")
                .inner_size(900.0, 600.0)
                .visible(false)
                .initialization_script(&initialization_script);

                #[cfg(not(target_os = "macos"))]
                let builder = builder.user_agent(USER_AGENT);

                builder
            }
            .on_navigation(move |url| {
                if url.scheme() == "chathub" && url.host_str() == Some("url-change") {
                    if let Some((_, tracked_url)) =
                        url.query_pairs().find(|(key, _)| key == "value")
                    {
                        save_current_model_url(&navigation_app, tracked_url.as_ref());
                    }
                    return false;
                }

                if is_app_owned_url(url) {
                    return true;
                }

                save_current_model_url(&navigation_app, &url.to_string());
                true
            })
            .build()?;

            let app_handle = app.handle().clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
            });

            let settings = load_app_settings(app.handle());
            let _ = main_window.set_always_on_top(settings.always_on_top);

            println!("[调试] 开始创建托盘菜单...");
            let menu = match tray::create_tray_menu(app.handle()) {
                Ok(m) => {
                    println!("[调试] 托盘菜单创建成功");
                    m
                }
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
                        "always_on_top" => toggle_always_on_top(app),
                        // 打开浏览器
                        "open_browser" => {
                            let url = load_app_settings(app).current_url();
                            models::open_model_in_system_browser(&url);
                        }
                        "check_updates" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                updater::check_for_updates(app_handle).await;
                            });
                        }
                        // Google OAuth 登录（localhost 回调方案）
                        "google_login" => oauth::start_login(app),
                        // 退出登录
                        "logout" => oauth::logout(app),
                        // 开机自启
                        "autostart" => {
                            use tauri_plugin_autostart::ManagerExt;
                            let autostart_manager = app.autolaunch();
                            if autostart_manager.is_enabled().unwrap_or(false) {
                                let _ = autostart_manager.disable();
                            } else {
                                let _ = autostart_manager.enable();
                            }
                            let mut settings = load_app_settings(app);
                            settings.auto_launch_on_startup =
                                autostart_manager.is_enabled().unwrap_or(false);
                            save_app_settings(app, &settings);
                            tray::update_tray_menu(app);
                        }
                        CHATGPT_MODEL_ID => models::switch_model(app, CHATGPT_MODEL_ID),
                        DEEPSEEK_MODEL_ID => models::switch_model(app, DEEPSEEK_MODEL_ID),
                        GROK_MODEL_ID => models::switch_model(app, GROK_MODEL_ID),
                        GEMINI_MODEL_ID => models::switch_model(app, GEMINI_MODEL_ID),
                        QWEN_MODEL_ID => models::switch_model(app, QWEN_MODEL_ID),
                        DOUBAO_MODEL_ID => models::switch_model(app, DOUBAO_MODEL_ID),
                        // 语言切换
                        "lang_zh" => {
                            let mut settings = load_app_settings(app);
                            settings.menu_language = "zh".to_string();
                            save_app_settings(app, &settings);
                            tray::update_tray_menu(app);
                        }
                        "lang_en" => {
                            let mut settings = load_app_settings(app);
                            settings.menu_language = "en".to_string();
                            save_app_settings(app, &settings);
                            tray::update_tray_menu(app);
                        }
                        // 快捷键设置
                        "shortcut" => dialog_windows::open_shortcut_dialog(app),
                        // 代理设置
                        "proxy" => dialog_windows::open_proxy_dialog(app),
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
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            let shortcut_str = load_app_settings(app.handle()).toggle_shortcut;

            if !shortcut_str.is_empty() {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                    println!("注册初始快捷键: {}", shortcut_str);
                    let _ = app.global_shortcut().register(shortcut);
                }
            }

            // 启动时显示主窗口
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
