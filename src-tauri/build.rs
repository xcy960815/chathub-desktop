fn main() {
    // 从 .env 文件加载环境变量到编译环境
    if let Ok(iter) = dotenvy::dotenv_iter() {
        for item in iter {
            if let Ok((key, val)) = item {
                println!("cargo:rustc-env={}={}", key, val);
            }
        }
    }

    tauri_build::build()
}
