use std::env;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudPayloadMetadata {
    build_id: String,
    platform: String,
    project_id: String,
    android_sdk_url: String,
    ios_sdk_url: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("[error] {}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let payload_zip = env::args()
        .nth(1)
        .ok_or_else(|| "usage: unipack-cloud-build <payload.zip>".to_string())?;
    let payload_zip = PathBuf::from(payload_zip);
    if !payload_zip.is_file() {
        return Err(format!("payload 不存在: {}", payload_zip.display()));
    }
    let work_dir = env::current_dir()
        .map_err(|e| e.to_string())?
        .join(".unipack-cloud-work")
        .join(format!("{}", chrono::Utc::now().timestamp()));
    std::fs::create_dir_all(&work_dir).map_err(|e| e.to_string())?;
    unzip(&payload_zip, &work_dir)?;
    let metadata_path = work_dir.join("payload.json");
    let metadata: CloudPayloadMetadata = serde_json::from_str(
        &std::fs::read_to_string(&metadata_path)
            .map_err(|e| format!("读取 payload.json 失败: {}", e))?,
    )
    .map_err(|e| format!("解析 payload.json 失败: {}", e))?;
    println!("[info] build_id={}", metadata.build_id);
    println!("[info] platform={}", metadata.platform);
    println!("[info] project_id={}", metadata.project_id);
    if metadata.platform == "android" {
        println!("[info] android_sdk_url={}", empty_dash(&metadata.android_sdk_url));
    } else if metadata.platform == "ios" {
        println!("[info] ios_sdk_url={}", empty_dash(&metadata.ios_sdk_url));
    }
    Err("CI payload 已解析，但 headless 原生打包核心尚未接入；需要继续将现有 Android/iOS 构建流程从 tauri::Window 日志依赖中抽离后启用。".to_string())
}

fn empty_dash(value: &str) -> &str {
    if value.trim().is_empty() {
        "-"
    } else {
        value
    }
}

fn unzip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let enclosed = file
            .enclosed_name()
            .ok_or_else(|| format!("payload 包含不安全路径: {}", file.name()))?
            .to_path_buf();
        let out_path = dest_dir.join(enclosed);
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut output = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut output).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
