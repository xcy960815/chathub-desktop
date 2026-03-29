use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub const SETTINGS_FILENAME: &str = "settings.json";
pub const APP_SETTINGS_KEY: &str = "app_settings";
pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+G";
pub const HISTORY_LIMIT: usize = 10;

pub const CHATGPT_MODEL_ID: &str = "chatgpt";
pub const DEEPSEEK_MODEL_ID: &str = "deepseek";
pub const GROK_MODEL_ID: &str = "grok";
pub const GEMINI_MODEL_ID: &str = "gemini";
pub const QWEN_MODEL_ID: &str = "qwen";
pub const DOUBAO_MODEL_ID: &str = "doubao";

pub const CHATGPT_URL: &str = "https://chatgpt.com";
pub const DEEPSEEK_URL: &str = "https://chat.deepseek.com/";
pub const GROK_URL: &str = "https://grok.com/";
pub const GEMINI_URL: &str = "https://gemini.google.com/app";
pub const QWEN_URL: &str = "https://www.qianwen.com/chat";
pub const DOUBAO_URL: &str = "https://www.doubao.com/chat/";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelUrls {
    pub chatgpt: String,
    pub deepseek: String,
    pub grok: String,
    pub gemini: String,
    pub qwen: String,
    pub doubao: String,
}

impl Default for ModelUrls {
    fn default() -> Self {
        Self {
            chatgpt: CHATGPT_URL.to_string(),
            deepseek: DEEPSEEK_URL.to_string(),
            grok: GROK_URL.to_string(),
            gemini: GEMINI_URL.to_string(),
            qwen: QWEN_URL.to_string(),
            doubao: DOUBAO_URL.to_string(),
        }
    }
}

impl ModelUrls {
    pub fn get(&self, model: &str) -> &str {
        match normalize_model_id(model) {
            DEEPSEEK_MODEL_ID => &self.deepseek,
            GROK_MODEL_ID => &self.grok,
            GEMINI_MODEL_ID => &self.gemini,
            QWEN_MODEL_ID => &self.qwen,
            DOUBAO_MODEL_ID => &self.doubao,
            _ => &self.chatgpt,
        }
    }

    pub fn set(&mut self, model: &str, url: String) {
        match normalize_model_id(model) {
            DEEPSEEK_MODEL_ID => self.deepseek = url,
            GROK_MODEL_ID => self.grok = url,
            GEMINI_MODEL_ID => self.gemini = url,
            QWEN_MODEL_ID => self.qwen = url,
            DOUBAO_MODEL_ID => self.doubao = url,
            _ => self.chatgpt = url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub model: String,
    pub urls: ModelUrls,
    pub toggle_shortcut: String,
    pub auto_launch_on_startup: bool,
    pub always_on_top: bool,
    pub menu_language: String,
    pub proxy: Option<String>,
    pub proxy_history: Vec<String>,
    pub shortcut_history: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct LegacyModelUrls {
    #[serde(rename = "ChatGPT")]
    chatgpt: Option<String>,
    #[serde(rename = "DeepSeek")]
    deepseek: Option<String>,
    #[serde(rename = "Grok")]
    grok: Option<String>,
    #[serde(rename = "Gemini")]
    gemini: Option<String>,
    #[serde(rename = "Qwen")]
    qwen: Option<String>,
    #[serde(rename = "Doubao")]
    doubao: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct LegacyAppSettings {
    model: Option<String>,
    #[serde(rename = "lastVisitedUrl")]
    last_visited_url: Option<String>,
    urls: Option<LegacyModelUrls>,
    #[serde(rename = "toggleShortcut")]
    toggle_shortcut: Option<String>,
    #[serde(rename = "autoLaunchOnStartup")]
    auto_launch_on_startup: Option<bool>,
    #[serde(rename = "alwaysOnTop")]
    always_on_top: Option<bool>,
    #[serde(rename = "menuLanguage")]
    menu_language: Option<String>,
    proxy: Option<String>,
    #[serde(rename = "proxyHistory")]
    proxy_history: Option<Vec<String>>,
    #[serde(rename = "shortcutHistory")]
    shortcut_history: Option<Vec<String>>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            model: CHATGPT_MODEL_ID.to_string(),
            urls: ModelUrls::default(),
            toggle_shortcut: DEFAULT_SHORTCUT.to_string(),
            auto_launch_on_startup: false,
            always_on_top: false,
            menu_language: "zh".to_string(),
            proxy: None,
            proxy_history: Vec::new(),
            shortcut_history: Vec::new(),
        }
    }
}

impl AppSettings {
    pub fn current_url(&self) -> String {
        self.urls.get(&self.model).to_string()
    }

    pub fn set_model(&mut self, model: &str) {
        self.model = normalize_model_id(model).to_string();
    }

    pub fn set_current_url(&mut self, url: String) {
        let current_model = self.model.clone();
        self.urls.set(&current_model, url);
    }

    pub fn reset_urls(&mut self) {
        self.urls = ModelUrls::default();
    }
}

pub fn normalize_model_id(raw: &str) -> &'static str {
    match raw.to_ascii_lowercase().as_str() {
        DEEPSEEK_MODEL_ID => DEEPSEEK_MODEL_ID,
        GROK_MODEL_ID => GROK_MODEL_ID,
        GEMINI_MODEL_ID => GEMINI_MODEL_ID,
        QWEN_MODEL_ID => QWEN_MODEL_ID,
        DOUBAO_MODEL_ID => DOUBAO_MODEL_ID,
        "chatgpt" | "chatgpt.com" | "chatgpt_url" => CHATGPT_MODEL_ID,
        _ => CHATGPT_MODEL_ID,
    }
}

pub fn infer_model_from_url(url: &str) -> &'static str {
    let lowered = url.to_ascii_lowercase();
    if lowered.contains("deepseek.com") {
        DEEPSEEK_MODEL_ID
    } else if lowered.contains("grok.com") {
        GROK_MODEL_ID
    } else if lowered.contains("gemini.google.com") {
        GEMINI_MODEL_ID
    } else if lowered.contains("qianwen.com") {
        QWEN_MODEL_ID
    } else if lowered.contains("doubao.com") {
        DOUBAO_MODEL_ID
    } else {
        CHATGPT_MODEL_ID
    }
}

fn trim_non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn dedupe_history(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();

    for value in values {
        if let Some(value) = trim_non_empty(&value) {
            if deduped.iter().any(|existing| existing == &value) {
                continue;
            }

            deduped.push(value);
            if deduped.len() >= HISTORY_LIMIT {
                break;
            }
        }
    }

    deduped
}

fn legacy_settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")?;
        return Some(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("chathub-desktop")
                .join("config")
                .join("settings.json"),
        );
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = env::var_os("APPDATA")?;
        return Some(
            PathBuf::from(app_data)
                .join("chathub-desktop")
                .join("config")
                .join("settings.json"),
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let base = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;

        Some(
            base.join("chathub-desktop")
                .join("config")
                .join("settings.json"),
        )
    }
}

fn load_legacy_app_settings() -> Option<LegacyAppSettings> {
    let path = legacy_settings_path()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<LegacyAppSettings>(&raw).ok()
}

fn normalize_legacy_proxy(proxy: &str) -> Option<String> {
    match normalize_proxy(proxy) {
        Ok(Some(value)) => Some(value),
        Ok(None) => None,
        Err(_) => trim_non_empty(proxy),
    }
}

fn apply_legacy_urls(settings: &mut AppSettings, urls: LegacyModelUrls) {
    if let Some(url) = urls.chatgpt.and_then(|value| trim_non_empty(&value)) {
        settings.urls.chatgpt = url;
    }
    if let Some(url) = urls.deepseek.and_then(|value| trim_non_empty(&value)) {
        settings.urls.deepseek = url;
    }
    if let Some(url) = urls.grok.and_then(|value| trim_non_empty(&value)) {
        settings.urls.grok = url;
    }
    if let Some(url) = urls.gemini.and_then(|value| trim_non_empty(&value)) {
        settings.urls.gemini = url;
    }
    if let Some(url) = urls.qwen.and_then(|value| trim_non_empty(&value)) {
        settings.urls.qwen = url;
    }
    if let Some(url) = urls.doubao.and_then(|value| trim_non_empty(&value)) {
        settings.urls.doubao = url;
    }
}

fn apply_legacy_settings(settings: &mut AppSettings, legacy: LegacyAppSettings) {
    if let Some(urls) = legacy.urls {
        apply_legacy_urls(settings, urls);
    }

    if let Some(model) = legacy.model.as_deref() {
        settings.model = normalize_model_id(model).to_string();
    }

    if let Some(shortcut) = legacy
        .toggle_shortcut
        .and_then(|value| trim_non_empty(&value))
    {
        settings.toggle_shortcut = shortcut;
    }

    if let Some(auto_launch_on_startup) = legacy.auto_launch_on_startup {
        settings.auto_launch_on_startup = auto_launch_on_startup;
    }

    if let Some(always_on_top) = legacy.always_on_top {
        settings.always_on_top = always_on_top;
    }

    if let Some(menu_language) = legacy.menu_language.as_deref() {
        settings.menu_language = if menu_language.eq_ignore_ascii_case("en") {
            "en".to_string()
        } else {
            "zh".to_string()
        };
    }

    if let Some(proxy) = legacy.proxy.as_deref().and_then(normalize_legacy_proxy) {
        settings.proxy = Some(proxy);
    }

    if let Some(proxy_history) = legacy.proxy_history {
        settings.proxy_history = dedupe_history(proxy_history);
    }

    if let Some(shortcut_history) = legacy.shortcut_history {
        settings.shortcut_history = dedupe_history(shortcut_history);
    }

    if let Some(last_visited_url) = legacy.last_visited_url.as_deref().and_then(trim_non_empty) {
        let model = infer_model_from_url(&last_visited_url);
        settings.model = model.to_string();
        settings.urls.set(model, last_visited_url);
    }
}

pub fn load_app_settings(app: &AppHandle) -> AppSettings {
    let store = app.store(SETTINGS_FILENAME).unwrap();

    if let Some(value) = store.get(APP_SETTINGS_KEY) {
        if let Ok(settings) = serde_json::from_value::<AppSettings>(value) {
            return settings;
        }
    }

    let mut settings = AppSettings::default();

    if let Some(legacy_settings) = load_legacy_app_settings() {
        apply_legacy_settings(&mut settings, legacy_settings);
    }

    if let Some(language) = store
        .get("language")
        .and_then(|value| value.as_str().map(str::to_string))
    {
        settings.menu_language = language;
    }

    if let Some(shortcut) = store
        .get("shortcut")
        .and_then(|value| value.as_str().map(str::to_string))
    {
        settings.toggle_shortcut = shortcut;
    }

    if let Some(proxy) = store
        .get("proxy")
        .and_then(|value| value.as_str().map(str::to_string))
    {
        settings.proxy = Some(proxy);
    }

    if let Some(last_url) = store
        .get("last_url")
        .and_then(|value| value.as_str().map(str::to_string))
    {
        let model = infer_model_from_url(&last_url);
        settings.model = model.to_string();
        settings.urls.set(model, last_url);
    }

    save_app_settings(app, &settings);
    settings
}

pub fn save_app_settings(app: &AppHandle, settings: &AppSettings) {
    let store = app.store(SETTINGS_FILENAME).unwrap();
    let value = serde_json::to_value(settings).unwrap_or_default();
    store.set(APP_SETTINGS_KEY, value);
    let _ = store.save();
}

pub fn upsert_history(history: &mut Vec<String>, value: &str) {
    if value.is_empty() {
        return;
    }

    history.retain(|item| item != value);
    history.insert(0, value.to_string());
    history.truncate(HISTORY_LIMIT);
}

pub fn remove_history(history: &mut Vec<String>, value: &str) {
    history.retain(|item| item != value);
}

pub fn normalize_proxy(raw: &str) -> Result<Option<String>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    };

    let parsed = url::Url::parse(&candidate)
        .map_err(|_| "代理地址格式无效，请输入类似 socks5://127.0.0.1:7897 的地址".to_string())?;

    if parsed.host_str().is_none() || parsed.port().is_none() {
        return Err("代理地址格式无效，请补充主机和端口".to_string());
    }

    Ok(Some(candidate))
}
