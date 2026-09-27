use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::Shortcut;

use crate::settings::load_app_settings;

/**
 * 打开代理设置弹窗（语言等文案通过 URL 参数传入）
 */
pub fn open_proxy_dialog(app: &AppHandle) {
    let settings = load_app_settings(app);
    let current_lang = settings.menu_language;
    let current_proxy = settings.proxy.unwrap_or_default();

    let is_english = current_lang == "en";

    if let Some(proxy_win) = app.get_webview_window("proxy") {
        let _ = proxy_win.set_focus();
        return;
    }

    let title = if is_english {
        "Proxy Settings"
    } else {
        "代理设置"
    };
    let ok_text = if is_english { "Save" } else { "保存" };
    let cancel_text = if is_english { "Cancel" } else { "取消" };
    let hint_text = if is_english {
        "Enter proxy address (e.g. socks5://127.0.0.1:7897)"
    } else {
        "输入代理地址（如 socks5://127.0.0.1:7897）"
    };
    let placeholder_text = if is_english {
        "Leave empty to disable proxy"
    } else {
        "留空则禁用代理"
    };
    let history_text = if is_english {
        "Recent Proxies"
    } else {
        "最近使用"
    };
    let empty_history_text = if is_english {
        "No proxy history yet"
    } else {
        "暂无历史记录"
    };
    let delete_text = if is_english { "Delete" } else { "删除" };
    let clear_text = if is_english { "Clear" } else { "清空" };

    let query_params = format!(
        "?hint={hint_text}&current={current_proxy}&placeholder={placeholder_text}&cancelText={cancel_text}&okText={ok_text}&historyLabel={historyLabel}&emptyHistory={emptyHistory}&deleteText={deleteText}&clearText={clearText}",
        hint_text = urlencoding::encode(&hint_text),
        current_proxy = urlencoding::encode(&current_proxy),
        placeholder_text = urlencoding::encode(&placeholder_text),
        cancel_text = urlencoding::encode(&cancel_text),
        ok_text = urlencoding::encode(&ok_text),
        historyLabel = urlencoding::encode(&history_text),
        emptyHistory = urlencoding::encode(&empty_history_text),
        deleteText = urlencoding::encode(&delete_text),
        clearText = urlencoding::encode(&clear_text)
    );

    let _ = tauri::webview::WebviewWindowBuilder::new(
        app,
        "proxy",
        tauri::WebviewUrl::App(format!("proxy.html{}", query_params).into()),
    )
    .title(title)
    .inner_size(460.0, 360.0)
    .resizable(false)
    .minimizable(false)
    .always_on_top(true)
    .center()
    .build();
}

/**
 * 打开快捷键设置弹窗；打开期间注销全局快捷键，窗口销毁时恢复
 */
pub fn open_shortcut_dialog(app: &AppHandle) {
    let settings = load_app_settings(app);
    let current_lang = settings.menu_language;
    let current_shortcut = settings.toggle_shortcut;

    let is_english = current_lang == "en";

    if let Some(shortcut_win) = app.get_webview_window("shortcut") {
        let _ = shortcut_win.set_focus();
        return;
    }

    let title = if is_english {
        "Shortcut Settings"
    } else {
        "快捷键设置"
    };
    let ok_text = if is_english { "Save" } else { "保存" };
    let cancel_text = if is_english { "Cancel" } else { "取消" };
    let hint_text = if is_english {
        "Press keys to set new shortcut"
    } else {
        "按下按键组合以设置快捷键"
    };
    let history_text = if is_english {
        "Recent Shortcuts"
    } else {
        "最近使用"
    };
    let empty_history_text = if is_english {
        "No shortcut history yet"
    } else {
        "暂无历史记录"
    };
    let reset_text = if is_english {
        "Reset Default"
    } else {
        "恢复默认"
    };
    let delete_text = if is_english { "Delete" } else { "删除" };

    let query_params = format!(
        "?title={title}&hint={hint}&current={current}&cancelText={cancelText}&okText={okText}&historyLabel={historyLabel}&emptyHistory={emptyHistory}&resetText={resetText}&deleteText={deleteText}",
        title = urlencoding::encode(&title),
        hint = urlencoding::encode(&hint_text),
        current = urlencoding::encode(&current_shortcut),
        cancelText = urlencoding::encode(&cancel_text),
        okText = urlencoding::encode(&ok_text),
        historyLabel = urlencoding::encode(&history_text),
        emptyHistory = urlencoding::encode(&empty_history_text),
        resetText = urlencoding::encode(&reset_text),
        deleteText = urlencoding::encode(&delete_text)
    );

    // 在打开设置前取消注册当前快捷键
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    if let Ok(current_s) = current_shortcut.parse::<Shortcut>() {
        let _ = app.global_shortcut().unregister(current_s);
    }

    let window = tauri::webview::WebviewWindowBuilder::new(
        app,
        "shortcut",
        tauri::WebviewUrl::App(format!("shortcut.html{}", query_params).into()),
    )
    .title(title)
    .inner_size(460.0, 360.0)
    .resizable(false)
    .minimizable(false)
    .always_on_top(true)
    .center()
    .build()
    .unwrap();

    let app_handle = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            let shortcut_str = load_app_settings(&app_handle).toggle_shortcut;

            if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                println!("重新启用全局快捷键: {}", shortcut_str);
                let _ = app_handle.global_shortcut().register(shortcut);
            }
        }
    });
}
