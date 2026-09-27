fn main() {
    let mut loaded_keys = std::collections::HashSet::new();

    // 从 src-tauri/.env 加载环境变量到编译环境
    if let Ok(iter) = dotenvy::dotenv_iter() {
        for (key, val) in iter.flatten() {
            println!("cargo:rustc-env={key}={val}");
            loaded_keys.insert(key);
        }
    }

    // .env 变更时重新编译，避免凭据更新后仍打包旧值
    println!("cargo:rerun-if-changed=.env");

    // 未配置 .env 时使用占位值，保证本地开发可以编译启动；
    // CLIENT_SECRET 留空表示走纯 PKCE 流程
    set_default_if_missing(
        "GOOGLE_OAUTH_CLIENT_ID",
        "your_client_id_here",
        &loaded_keys,
    );
    set_default_if_missing("GOOGLE_OAUTH_CLIENT_SECRET", "", &loaded_keys);

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
