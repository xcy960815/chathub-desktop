use serde::Deserialize;
use tauri::AppHandle;

use crate::{build_http_client, show_js_alert};

const RELEASES_URL: &str = "https://github.com/xcy960815/chathub-desktop/releases";
const RELEASES_API_URL: &str =
    "https://api.github.com/repos/xcy960815/chathub-desktop/releases/latest";

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: Option<String>,
}

fn is_newer_version(version1: &str, version2: &str) -> bool {
    let left = version1
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0));
    let right = version2
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0));

    let left_parts: Vec<u32> = left.collect();
    let right_parts: Vec<u32> = right.collect();
    let max_len = left_parts.len().max(right_parts.len());

    for index in 0..max_len {
        let left_part = *left_parts.get(index).unwrap_or(&0);
        let right_part = *right_parts.get(index).unwrap_or(&0);

        if left_part > right_part {
            return true;
        }
        if left_part < right_part {
            return false;
        }
    }

    false
}

pub async fn check_for_updates(app: AppHandle) {
    let client = build_http_client(&app);
    let response = match client
        .get(RELEASES_API_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            show_js_alert(&app, &format!("检查更新失败：{error}"));
            return;
        }
    };

    if !response.status().is_success() {
        show_js_alert(
            &app,
            &format!("检查更新失败：GitHub API 返回 {}", response.status()),
        );
        return;
    }

    let release = match response.json::<GitHubRelease>().await {
        Ok(release) => release,
        Err(error) => {
            show_js_alert(&app, &format!("解析更新信息失败：{error}"));
            return;
        }
    };

    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version = app.package_info().version.to_string();

    if is_newer_version(latest_version, &current_version) {
        let download_url = release.html_url.unwrap_or_else(|| RELEASES_URL.to_string());
        let _ = tauri_plugin_opener::open_url(download_url, None::<&str>);
        show_js_alert(
            &app,
            &format!(
                "发现新版本 {latest_version}，已为你打开下载页面。当前版本：{current_version}"
            ),
        );
    } else {
        show_js_alert(&app, "当前已经是最新版本。");
    }
}
