use std::path::Path;

fn main() {
    // Only attempt resource compilation if running on Windows and the icon file is available
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        if Path::new("app_icon.ico").exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("app_icon.ico");
            if let Err(e) = res.compile() {
                eprintln!("Warning: Resource compiler failed: {}", e);
            }
        } else {
            println!("cargo:warning=app_icon.ico not found, skipping icon embedding.");
        }
    }
}
