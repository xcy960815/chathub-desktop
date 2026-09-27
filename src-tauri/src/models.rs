use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::settings::{load_app_settings, normalize_model_id, save_app_settings, CHATGPT_MODEL_ID};
use crate::APP_SHELL_URL;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLaunchPlan {
    pub url: String,
    pub open_in_system_browser: bool,
    pub title: Option<String>,
    pub notice: Option<String>,
}

/**
 * 切换模型
 */
pub fn switch_model(app: &AppHandle, url: &str) {
    let target_model = normalize_model_id(url);
    let mut settings = load_app_settings(app);
    settings.set_model(target_model);
    let target_url = settings.current_url();
    let loading_text = if settings.menu_language == "en" {
        "Loading model..."
    } else {
        "模型加载中..."
    };
    save_app_settings(app, &settings);

    if should_open_in_system_browser(target_model) {
        // 只需回到外壳页面：外壳挂载时会调用 get_model_launch_plan 统一打开系统浏览器，
        // 这里若直接打开，外壳重载后还会再打开一次
        navigate_main_webview_to_app_shell(app);
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
        crate::tray::update_tray_menu(app);
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        let loading_script = include_str!("loading_overlay.js")
            .replace("__TARGET_URL__", &target_url)
            .replace("__LOADING_TEXT__", loading_text);

        let _ = window.eval(&loading_script);
        let _ = window.show();
        let _ = window.set_focus();
    }
    crate::tray::update_tray_menu(app);
}

pub fn should_open_in_system_browser(model_id: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        normalize_model_id(model_id) == CHATGPT_MODEL_ID
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = model_id;
        false
    }
}

pub fn open_model_in_system_browser(url: &str) {
    let _ = tauri_plugin_opener::open_url(url, None::<&str>);
}

fn navigate_main_webview_to_app_shell(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let script = format!(
            "window.location.replace({});",
            serde_json::to_string(APP_SHELL_URL).unwrap_or_else(|_| "\"/\"".to_string())
        );
        let _ = window.eval(&script);
    }
}

#[tauri::command]
pub fn get_model_launch_plan(app: AppHandle) -> ModelLaunchPlan {
    let settings = load_app_settings(&app);
    let model = normalize_model_id(&settings.model);
    let url = settings.current_url();

    if should_open_in_system_browser(model) {
        open_model_in_system_browser(&url);
        let (title, notice) = if settings.menu_language == "en" {
            (
                "ChatGPT opened in your browser",
                "The built-in WebView on macOS cannot reliably pass Cloudflare's human verification. Please continue in Safari or Chrome.",
            )
        } else {
            (
                "ChatGPT 已在系统浏览器中打开",
                "macOS 内置 WebView 无法可靠通过 Cloudflare 人机验证，请使用 Safari 或 Chrome 继续。",
            )
        };
        return ModelLaunchPlan {
            url,
            open_in_system_browser: true,
            title: Some(title.to_string()),
            notice: Some(notice.to_string()),
        };
    }

    ModelLaunchPlan {
        url,
        open_in_system_browser: false,
        title: None,
        notice: None,
    }
}

#[tauri::command]
pub fn open_model_in_browser(url: String) {
    open_model_in_system_browser(&url);
}
