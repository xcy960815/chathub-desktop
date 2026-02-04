use tauri::{
    menu::{Menu, MenuItem, Submenu, CheckMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_plugin_store::StoreExt;
use tauri_plugin_autostart::MacosLauncher;
use serde_json::json;

const CHATGPT_URL: &str = "https://chatgpt.com";
const DEEPSEEK_URL: &str = "https://chat.deepseek.com/";
const GROK_URL: &str = "https://grok.com/";
const GEMINI_URL: &str = "https://gemini.google.com/app";
const SETTINGS_FILENAME: &str = "settings.json";

fn save_last_url(app: &AppHandle, url: &str) {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    store.set("last_url", json!(url));
    let _ = store.save();
}

fn create_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    // Get current language setting
    let store = app.store(SETTINGS_FILENAME).unwrap();
    let current_lang = store.get("language")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "zh".to_string());
    
    let is_english = current_lang == "en";
    
    // Get text based on language
    let (quit_text, reload_text, open_browser_text, autostart_text, models_text, lang_text, proxy_text) = 
        if is_english {
            ("Quit", "Reload", "Open in Browser", "Launch at Login", "Models", "Language", "Proxy Settings")
        } else {
            ("退出", "重新加载", "在浏览器打开", "开机自启", "模型", "语言", "代理设置")
        };

    let quit_i = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;
    let reload_i = MenuItem::with_id(app, "reload", reload_text, true, None::<&str>)?;
    let open_browser_i = MenuItem::with_id(app, "open_browser", open_browser_text, true, None::<&str>)?;
    
    // Check if autostart is enabled
    use tauri_plugin_autostart::ManagerExt;
    let autostart_manager = app.autolaunch();
    let is_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
    let autostart_i = CheckMenuItem::with_id(app, "autostart", autostart_text, true, is_autostart_enabled, None::<&str>)?;
    
    // Proxy settings
    let proxy_i = MenuItem::with_id(app, "proxy", proxy_text, true, None::<&str>)?;
    
    // Models submenu - get current model from last_url
    let current_url = store.get("last_url")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| CHATGPT_URL.to_string());
    
    let chatgpt_i = CheckMenuItem::with_id(app, "chatgpt", "ChatGPT", true, current_url == CHATGPT_URL, None::<&str>)?;
    let deepseek_i = CheckMenuItem::with_id(app, "deepseek", "DeepSeek", true, current_url == DEEPSEEK_URL, None::<&str>)?;
    let grok_i = CheckMenuItem::with_id(app, "grok", "Grok", true, current_url == GROK_URL, None::<&str>)?;
    let gemini_i = CheckMenuItem::with_id(app, "gemini", "Gemini", true, current_url == GEMINI_URL, None::<&str>)?;

    let models_submenu = Submenu::with_items(
        app,
        models_text,
        true,
        &[&chatgpt_i, &deepseek_i, &grok_i, &gemini_i],
    )?;

    let lang_zh_i = CheckMenuItem::with_id(app, "lang_zh", "中文", true, !is_english, None::<&str>)?;
    let lang_en_i = CheckMenuItem::with_id(app, "lang_en", "English", true, is_english, None::<&str>)?;

    let language_submenu = Submenu::with_items(
        app,
        lang_text,
        true,
        &[&lang_zh_i, &lang_en_i],
    )?;

    Menu::with_items(app, &[
        &models_submenu,
        &language_submenu,
        &reload_i,
        &open_browser_i,
        &proxy_i,
        &autostart_i,
        &quit_i,
    ])
}

fn update_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("tray") {
        if let Ok(menu) = create_tray_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn switch_model(app: &AppHandle, url: &str) {
    if let Some(window) = app.get_webview_window("main") {
        save_last_url(app, url);
        
        // Inject loading overlay into current page
        let loading_script = format!(r#"
            (function() {{
                // Create overlay
                var overlay = document.createElement('div');
                overlay.id = 'chathub-loading-overlay';
                overlay.innerHTML = `
                    <style>
                        #chathub-loading-overlay {{
                            position: fixed;
                            top: 0;
                            left: 0;
                            width: 100vw;
                            height: 100vh;
                            background: #f6f6f6;
                            display: flex;
                            flex-direction: column;
                            align-items: center;
                            justify-content: center;
                            z-index: 999999;
                            gap: 2rem;
                        }}
                        @media (prefers-color-scheme: dark) {{
                            #chathub-loading-overlay {{ background: #2f2f2f; }}
                            #chathub-loading-overlay .loading-text {{ color: #d1d5db; }}
                        }}
                        #chathub-loading-overlay .dots {{
                            display: flex;
                            align-items: flex-end;
                            gap: 8px;
                            height: 50px;
                        }}
                        #chathub-loading-overlay .dot {{
                            border-radius: 50%;
                            animation: chathub-bounce 0.6s ease-in-out infinite;
                        }}
                        #chathub-loading-overlay .dot-1 {{
                            width: 24px;
                            height: 24px;
                            background-color: #f87171;
                            animation-delay: 0s;
                        }}
                        #chathub-loading-overlay .dot-2 {{
                            width: 22px;
                            height: 22px;
                            background-color: #2dd4bf;
                            animation-delay: 0.1s;
                        }}
                        #chathub-loading-overlay .dot-3 {{
                            width: 18px;
                            height: 18px;
                            background-color: #7dd3fc;
                            animation-delay: 0.2s;
                        }}
                        @keyframes chathub-bounce {{
                            0%, 100% {{ transform: translateY(0); }}
                            50% {{ transform: translateY(-20px); }}
                        }}
                        #chathub-loading-overlay .loading-text {{
                            font-size: 1.5rem;
                            font-weight: 500;
                            color: #374151;
                            font-family: 'PingFang SC', 'Microsoft YaHei', sans-serif;
                        }}
                    </style>
                    <div class="dots">
                        <div class="dot dot-1"></div>
                        <div class="dot dot-2"></div>
                        <div class="dot dot-3"></div>
                    </div>
                    <p class="loading-text">模型加载中...</p>
                `;
                document.body.appendChild(overlay);
                
                // Navigate after animation shows
                setTimeout(function() {{
                    window.location.href = '{}';
                }}, 800);
            }})();
        "#, url);
        
        let _ = window.eval(&loading_script);
        let _ = window.show();
        let _ = window.set_focus();
    }
    // Update menu to reflect new model selection
    update_tray_menu(app);
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.move_window(Position::TrayCenter);
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

#[tauri::command]
fn save_proxy(app: AppHandle, proxy: String) {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    store.set("proxy", json!(proxy));
    let _ = store.save();
}

#[tauri::command]
fn close_proxy_window(app: AppHandle) {
    println!("Backend: Closing proxy window...");
    if let Some(win) = app.get_webview_window("proxy") {
        let _ = win.close();
        println!("Backend: Window closed successfully.");
    } else {
        println!("Backend: Proxy window not found!");
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
        .invoke_handler(tauri::generate_handler![get_last_model_url, save_proxy, close_proxy_window])
        .setup(|app| {
            // Hide dock icon on macOS
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Create initial menu
            let menu = create_tray_menu(app.handle())?;

            let _tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .show_menu_on_left_click(false)
                .on_menu_event(|app: &AppHandle, event| {
                    let id = event.id.as_ref();
                    match id {
                        "quit" => app.exit(0),
                        "reload" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.eval("window.location.reload()");
                            }
                        }
                        "open_browser" => {
                            let store = app.store(SETTINGS_FILENAME).unwrap();
                            let url = store.get("last_url")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| CHATGPT_URL.to_string());
                            let _ = tauri_plugin_opener::open_url(url, None::<&str>);
                        }
                        "autostart" => {
                            use tauri_plugin_autostart::ManagerExt;
                            let autostart_manager = app.autolaunch();
                            if autostart_manager.is_enabled().unwrap_or(false) {
                                let _ = autostart_manager.disable();
                            } else {
                                let _ = autostart_manager.enable();
                            }
                        }
                        "chatgpt" => switch_model(app, CHATGPT_URL),
                        "deepseek" => switch_model(app, DEEPSEEK_URL),
                        "grok" => switch_model(app, GROK_URL),
                        "gemini" => switch_model(app, GEMINI_URL),
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
                        "proxy" => {
                            // Get current proxy setting
                            let store = app.store(SETTINGS_FILENAME).unwrap();
                            let current_lang = store.get("language")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| "zh".to_string());
                            let current_proxy = store.get("proxy")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .unwrap_or_default();
                            
                            let is_english = current_lang == "en";
                            
                            // If window already exists, focus it
                            if let Some(proxy_win) = app.get_webview_window("proxy") {
                                let _ = proxy_win.set_focus();
                                return;
                            }

                            // Create a small separate window for proxy settings
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
                    // Only trigger on button Up to prevent double-triggering (Down + Up)
                    if let TrayIconEvent::Click { 
                        button: MouseButton::Left, 
                        button_state: MouseButtonState::Up, 
                        .. 
                    } = event {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Register Global Shortcut
            // Ensure capabilities allow this in tauri.conf.json / capabilities
            use tauri_plugin_global_shortcut::{ShortcutState};
            
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_shortcut("CommandOrControl+Shift+G")?
                    .with_handler(|app, _shortcut, event| {
                        if event.state() == ShortcutState::Pressed {
                             toggle_window(app);
                        }
                    })
                    .build(),
            )?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
