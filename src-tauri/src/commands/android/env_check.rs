//! Android 平台环境检测

use crate::commands::shared::env::check_tool;
use crate::commands::shared::env::PlatformEnv;

/// 检测 Android 平台环境是否就绪（ANDROID_HOME / Java）
async fn check_android_platform() -> PlatformEnv {
    let android_home = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_ROOT"))
        .ok();
    let mut issues = Vec::new();

    let (sdk_path, sdk_version) = match &android_home {
        Some(path) => {
            let p = std::path::Path::new(path);
            if !p.exists() {
                issues.push(format!(
                    "ANDROID_HOME points to non-existent path: {}",
                    path
                ));
            }
            let version = p.join("build-tools").read_dir().ok().and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .max()
            });
            (Some(path.clone()), version)
        }
        None => {
            issues.push("ANDROID_HOME environment variable is not set".to_string());
            (None, None)
        }
    };

    if !check_tool("java", "-version").installed {
        issues.push("Java/JDK is not installed or not in PATH".to_string());
    }

    PlatformEnv {
        available: android_home.is_some(),
        sdk_path,
        sdk_version,
        issues,
    }
}

#[tauri::command]
pub async fn check_android_env() -> Result<PlatformEnv, String> {
    Ok(check_android_platform().await)
}
