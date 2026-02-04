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

const MASKING_SCRIPT: &str = r#"
(function() {
  if (window.__TAURI_MASKING_APPLIED__) return;
  window.__TAURI_MASKING_APPLIED__ = true;

  const UA = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36';

  // 1. Mock UserAgentData
  const userAgentData = {
    brands: [
      { brand: 'Not(A:Brand', version: '99' },
      { brand: 'Google Chrome', version: '132' },
      { brand: 'Chromium', version: '132' }
    ],
    mobile: false,
    platform: 'macOS',
    getHighEntropyValues: function(hints) {
      return Promise.resolve({
        brands: this.brands,
        mobile: this.mobile,
        platform: this.platform,
        architecture: 'arm',
        bitness: '64',
        model: '',
        platformVersion: '14.3.1',
        uaFullVersion: '132.0.6834.110'
      });
    }
  };

  // 2. Define Navigator Overrides
  const overrides = {
    userAgent: UA,
    appVersion: UA.replace('Mozilla/', ''),
    userAgentData: userAgentData,
    webdriver: false,
    languages: ['zh-CN', 'zh', 'en-US', 'en'],
    language: 'zh-CN',
    vendor: 'Google Inc.',
    productSub: '20030107',
    deviceMemory: 8,
    hardwareConcurrency: 8,
    maxTouchPoints: 0,
    pdfViewerEnabled: true,
  };

  // 3. Hijack Navigator using Proxy for robustness
  const rawNavigator = window.navigator;
  const proxiedNavigator = new Proxy(rawNavigator, {
    get: (target, prop) => {
      if (prop in overrides) return overrides[prop];
      let val = target[prop];
      if (typeof val === 'function') val = val.bind(target);
      return val;
    }
  });

  Object.defineProperty(window, 'navigator', {
    value: proxiedNavigator,
    configurable: false,
    enumerable: true,
    writable: false
  });

  // 4. Mock window.chrome
  window.chrome = {
    runtime: {},
    loadTimes: function() {},
    csi: function() {},
    app: {}
  };

  // 5. WebGL Masking
  const getParameter = WebGLRenderingContext.prototype.getParameter;
  WebGLRenderingContext.prototype.getParameter = function(parameter) {
    if (parameter === 37445) return 'Intel Inc.';
    if (parameter === 37446) return 'Intel(R) Iris(R) Plus Graphics 640';
    return getParameter.apply(this, arguments);
  };

  // 6. Permissions Mock
  if (navigator.permissions && navigator.permissions.query) {
    const originalQuery = navigator.permissions.query;
    navigator.permissions.query = function(descriptor) {
      if (descriptor.name === 'notifications') {
        return Promise.resolve({ state: 'default', onchange: null });
      }
      return originalQuery.apply(this, arguments);
    };
  }

  // 7. Clean up automation signatures
  for (const prop in window) {
    if (prop.startsWith('cdc_') || prop.startsWith('__playwright')) {
      try { delete window[prop]; } catch(e) {}
    }
  }
})();
"#;

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
    let (quit_text, reload_text, open_browser_text, autostart_text, models_text, lang_text, proxy_text, shortcut_text) = 
        if is_english {
            ("Quit", "Reload", "Open in Browser", "Launch at Login", "Models", "Language", "Proxy Settings", "Shortcut Settings")
        } else {
            ("退出", "重新加载", "在浏览器打开", "开机自启", "模型", "语言", "代理设置", "快捷键设置")
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
    
    // Shortcut settings
    let shortcut_i = MenuItem::with_id(app, "shortcut", shortcut_text, true, None::<&str>)?;
    
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
        &shortcut_i,
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
        let loading_script = include_str!("loading_overlay.ts")
            .replace("__TARGET_URL__", url);
        
        let _ = window.eval(&loading_script);
        let _ = window.show();
        let _ = window.set_focus();
    }
    // Update menu to reflect new model selection
    update_tray_menu(app);
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        log_debug(&format!("Backend: Window visibility before toggle: {}", is_visible));
        if is_visible {
            let _ = window.hide();
            log_debug("Backend: Hiding window");
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            log_debug("Backend: Showing and focusing window");
        }
    } else {
        log_debug("Backend: Could not find main window during toggle");
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
    log_debug(&format!("Backend: Saving new shortcut to store: {}", shortcut));
    
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
            // Hide dock icon on macOS
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Create main window with identity masking
            let _main_window = tauri::webview::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::default(),
            )
            .title("ChatHub Desktop")
            .inner_size(900.0, 600.0)
            .visible(false)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")
            .initialization_script(MASKING_SCRIPT)
            .build()?;

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

                            // Unregister current shortcut before opening settings
                            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                            if let Ok(current_s) = current_shortcut.parse::<Shortcut>() {
                                log_debug("Backend: Suppressing global shortcut for recording");
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
                                        log_debug(&format!("Backend: Re-enabling global shortcut: {}", shortcut_str));
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

            let store = app.handle().store(SETTINGS_FILENAME).unwrap();
            let shortcut_str = store.get("shortcut")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string());

            if !shortcut_str.is_empty() {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                    log_debug(&format!("Backend: Registering initial shortcut: {}", shortcut_str));
                    let _ = app.global_shortcut().register(shortcut);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
