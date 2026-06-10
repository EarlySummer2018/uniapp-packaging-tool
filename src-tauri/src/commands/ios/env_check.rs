//! iOS 环境检测。

use crate::commands::shared::env::PlatformEnv;

#[tauri::command]
pub async fn check_ios_env() -> Result<PlatformEnv, String> {
    let xcode_select = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|path| !path.is_empty());

    let xcode_version = std::process::Command::new("xcodebuild")
        .arg("-version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|line| line.starts_with("Xcode "))
                .map(|line| line.trim_start_matches("Xcode ").trim().to_string())
        });

    let mut issues = Vec::new();
    if xcode_select.is_none() {
        issues.push("未配置 Xcode Command Line Tools".to_string());
    }
    if xcode_version.is_none() {
        issues.push("未检测到可用的 xcodebuild".to_string());
    }

    Ok(PlatformEnv {
        available: xcode_select.is_some() && xcode_version.is_some(),
        sdk_path: xcode_select,
        sdk_version: xcode_version,
        issues,
    })
}
