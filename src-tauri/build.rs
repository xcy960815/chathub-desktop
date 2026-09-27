fn main() {
    let mut loaded_keys = std::collections::HashSet::new();

    // 从 src-tauri/.env 加载环境变量到编译环境
    if let Ok(iter) = dotenvy::dotenv_iter() {
        for item in iter {
            if let Ok((key, val)) = item {
                println!("cargo:rustc-env={key}={val}");
                loaded_keys.insert(key);
            }
        }
    }

    // 未配置 .env 时使用占位值，保证本地开发可以编译启动
    set_default_if_missing(
        "GOOGLE_OAUTH_CLIENT_ID",
        "your_client_id_here",
        &loaded_keys,
    );
    set_default_if_missing(
        "GOOGLE_OAUTH_CLIENT_SECRET",
        "your_client_secret_here",
        &loaded_keys,
    );

    tauri_build::build()
}

fn set_default_if_missing(
    key: &str,
    default: &str,
    loaded_keys: &std::collections::HashSet<String>,
) {
    if !loaded_keys.contains(key) {
        println!("cargo:rustc-env={key}={default}");
    }
}
