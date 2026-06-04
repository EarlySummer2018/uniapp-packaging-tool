//! iOS 平台环境检测

use crate::commands::shared::env::PlatformEnv;

/// 检测 iOS 平台环境是否就绪（Xcode / Command Line Tools）
async fn check_ios_platform() -> PlatformEnv {
    let mut issues = Vec::new();
    let xcode_path = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let xcode_version = std::process::Command::new("xcodebuild")
        .arg("-version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find(|l| l.contains("Xcode"))
                .map(|s| {
                    s.trim()
                        .replace("Xcode ", "")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string()
                })
        });

    match &xcode_path {
        Some(p) if p.is_empty() => {
            issues.push("Xcode Command Line Tools are not configured".to_string());
        }
        None => {
            issues.push("Xcode is not installed".to_string());
        }
        _ => {}
    }

    PlatformEnv {
        available: xcode_path.is_some() && xcode_path.as_deref().unwrap_or("").is_empty() == false,
        sdk_path: xcode_path,
        sdk_version: xcode_version.or(Some("detected".to_string())),
        issues,
    }
}

#[tauri::command]
pub async fn check_ios_env() -> Result<PlatformEnv, String> {
    Ok(check_ios_platform().await)
}
