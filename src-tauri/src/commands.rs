use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::settings::{
    load_app_settings, normalize_proxy, remove_history, save_app_settings, upsert_history,
    DEFAULT_SHORTCUT,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyDialogData {
    pub current: String,
    pub history: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutDialogData {
    pub current: String,
    pub history: Vec<String>,
    pub default_shortcut: String,
}

#[tauri::command]
pub fn save_proxy(app: AppHandle, proxy: String) -> Result<(), String> {
    let normalized = normalize_proxy(&proxy)?;
    let mut settings = load_app_settings(&app);
    settings.proxy = normalized.clone();

    if let Some(proxy_value) = normalized.as_deref() {
        upsert_history(&mut settings.proxy_history, proxy_value);
    }

    save_app_settings(&app, &settings);
    Ok(())
}

#[tauri::command]
pub fn close_proxy_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("proxy") {
        let _ = win.close();
    }
}

#[tauri::command]
pub fn get_proxy_dialog_data(app: AppHandle) -> ProxyDialogData {
    let settings = load_app_settings(&app);
    ProxyDialogData {
        current: settings.proxy.unwrap_or_default(),
        history: settings.proxy_history,
    }
}

#[tauri::command]
pub fn remove_proxy_history(app: AppHandle, proxy: String) {
    let mut settings = load_app_settings(&app);
    remove_history(&mut settings.proxy_history, &proxy);
    save_app_settings(&app, &settings);
}

#[tauri::command]
pub fn save_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    let normalized = if shortcut.trim().is_empty() {
        DEFAULT_SHORTCUT.to_string()
    } else {
        shortcut.trim().to_string()
    };

    use tauri_plugin_global_shortcut::Shortcut;
    normalized
        .parse::<Shortcut>()
        .map_err(|_| "快捷键格式无效，请重新录入".to_string())?;

    let mut settings = load_app_settings(&app);
    settings.toggle_shortcut = normalized.clone();
    upsert_history(&mut settings.shortcut_history, &normalized);
    save_app_settings(&app, &settings);
    Ok(())
}

#[tauri::command]
pub fn close_shortcut_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("shortcut") {
        let _ = win.close();
    }
}

#[tauri::command]
pub fn get_shortcut_dialog_data(app: AppHandle) -> ShortcutDialogData {
    let settings = load_app_settings(&app);
    ShortcutDialogData {
        current: settings.toggle_shortcut,
        history: settings.shortcut_history,
        default_shortcut: DEFAULT_SHORTCUT.to_string(),
    }
}

#[tauri::command]
pub fn remove_shortcut_history(app: AppHandle, shortcut: String) {
    let mut settings = load_app_settings(&app);
    remove_history(&mut settings.shortcut_history, &shortcut);
    save_app_settings(&app, &settings);
}
