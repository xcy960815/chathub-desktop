use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, Submenu},
    AppHandle,
};

use tauri_plugin_autostart::ManagerExt;

use crate::settings::{
    load_app_settings, normalize_model_id, CHATGPT_MODEL_ID, DEEPSEEK_MODEL_ID, DOUBAO_MODEL_ID,
    GEMINI_MODEL_ID, GROK_MODEL_ID, QWEN_MODEL_ID,
};

/**
 * 创建托盘菜单
 */
pub fn create_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let settings = load_app_settings(app);
    let current_lang = settings.menu_language.clone();
    let is_english = current_lang == "en";

    let (
        quit_text,
        reload_text,
        open_browser_text,
        autostart_text,
        always_on_top_text,
        models_text,
        lang_text,
        proxy_text,
        shortcut_text,
        check_updates_text,
    ) = if is_english {
        (
            "Quit",
            "Reload",
            "Open in Browser",
            "Launch at Login",
            "Always on Top",
            "Models",
            "Language",
            "Proxy Settings",
            "Shortcut Settings",
            "Check for Updates",
        )
    } else {
        (
            "退出",
            "重新加载",
            "在浏览器打开",
            "开机自启",
            "窗口置顶",
            "模型",
            "语言",
            "代理设置",
            "快捷键设置",
            "检查更新",
        )
    };

    let quit_item = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;
    let reload_item = MenuItem::with_id(app, "reload", reload_text, true, None::<&str>)?;
    let open_browser_item =
        MenuItem::with_id(app, "open_browser", open_browser_text, true, None::<&str>)?;
    let check_updates_item =
        MenuItem::with_id(app, "check_updates", check_updates_text, true, None::<&str>)?;

    let autostart_manager = app.autolaunch();
    let is_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);
    let autostart_item = CheckMenuItem::with_id(
        app,
        "autostart",
        autostart_text,
        true,
        is_autostart_enabled,
        None::<&str>,
    )?;
    let always_on_top_item = CheckMenuItem::with_id(
        app,
        "always_on_top",
        always_on_top_text,
        true,
        settings.always_on_top,
        None::<&str>,
    )?;

    let proxy_item = MenuItem::with_id(app, "proxy", proxy_text, true, None::<&str>)?;
    let shortcut_item = MenuItem::with_id(app, "shortcut", shortcut_text, true, None::<&str>)?;

    let current_model = normalize_model_id(&settings.model);
    let chatgpt_item = CheckMenuItem::with_id(
        app,
        CHATGPT_MODEL_ID,
        "ChatGPT",
        true,
        current_model == CHATGPT_MODEL_ID,
        None::<&str>,
    )?;
    let grok_item = CheckMenuItem::with_id(
        app,
        GROK_MODEL_ID,
        "Grok",
        true,
        current_model == GROK_MODEL_ID,
        None::<&str>,
    )?;
    let gemini_item = CheckMenuItem::with_id(
        app,
        GEMINI_MODEL_ID,
        "Gemini",
        true,
        current_model == GEMINI_MODEL_ID,
        None::<&str>,
    )?;
    let deepseek_item = CheckMenuItem::with_id(
        app,
        DEEPSEEK_MODEL_ID,
        "DeepSeek",
        true,
        current_model == DEEPSEEK_MODEL_ID,
        None::<&str>,
    )?;
    let qwen_item = CheckMenuItem::with_id(
        app,
        QWEN_MODEL_ID,
        "Qwen",
        true,
        current_model == QWEN_MODEL_ID,
        None::<&str>,
    )?;
    let doubao_item = CheckMenuItem::with_id(
        app,
        DOUBAO_MODEL_ID,
        "Doubao",
        true,
        current_model == DOUBAO_MODEL_ID,
        None::<&str>,
    )?;

    let models_submenu = Submenu::with_items(
        app,
        models_text,
        true,
        &[
            &chatgpt_item,
            &grok_item,
            &gemini_item,
            &deepseek_item,
            &qwen_item,
            &doubao_item,
        ],
    )?;

    let lang_zh_item =
        CheckMenuItem::with_id(app, "lang_zh", "中文", true, !is_english, None::<&str>)?;
    let lang_en_item =
        CheckMenuItem::with_id(app, "lang_en", "English", true, is_english, None::<&str>)?;
    let language_submenu =
        Submenu::with_items(app, lang_text, true, &[&lang_zh_item, &lang_en_item])?;

    Menu::with_items(
        app,
        &[
            &models_submenu,
            &always_on_top_item,
            &shortcut_item,
            &proxy_item,
            &autostart_item,
            &language_submenu,
            &reload_item,
            &open_browser_item,
            &check_updates_item,
            &quit_item,
        ],
    )
}

/**
 * 更新托盘菜单
 */
pub fn update_tray_menu(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("tray") {
        if let Ok(menu) = create_tray_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
