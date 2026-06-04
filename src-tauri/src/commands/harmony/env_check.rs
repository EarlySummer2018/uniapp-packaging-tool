//! 鸿蒙(HarmonyOS) 平台环境检测

use crate::commands::shared::env::which_tool;
use crate::commands::shared::env::PlatformEnv;

/// 检测鸿蒙平台环境是否就绪（ohpm 或 hvigorw）
async fn check_harmony_platform() -> PlatformEnv {
    let mut issues = Vec::new();

    let ohpm_installed = std::process::Command::new("ohpm")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let hvigorw_path = which_tool("hvigorw");

    if !ohpm_installed && hvigorw_path.is_none() {
        issues
            .push("HarmonyOS SDK is not installed or DevEco Studio is not configured".to_string());
    }

    PlatformEnv {
        available: ohpm_installed || hvigorw_path.is_some(),
        sdk_path: hvigorw_path,
        sdk_version: if ohpm_installed {
            Some("detected".to_string())
        } else {
            None
        },
        issues,
    }
}

#[tauri::command]
pub async fn check_harmony_env() -> Result<PlatformEnv, String> {
    Ok(check_harmony_platform().await)
}
