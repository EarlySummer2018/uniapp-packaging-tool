//! iOS xcodebuild 环境解析与命令执行。

use std::path::{Path, PathBuf};

use tauri::Manager;

#[derive(Debug, Clone)]
pub(super) struct IosBuildEnvironment {
    pub(super) xcodebuild_bin: PathBuf,
    developer_dir: PathBuf,
}

pub(super) async fn run_xcodebuild(
    args: &[String],
    cwd: &Path,
    window: &tauri::Window,
    env: &IosBuildEnvironment,
    build_id: &str,
) -> Result<(), String> {
    let output = crate::utils::process::run_command_streaming_with_env_tagged(
        &env.xcodebuild_bin.to_string_lossy(),
        args,
        &cwd.to_string_lossy(),
        &ios_process_env(env),
        window.app_handle().clone(),
        "build-log",
        crate::utils::process::StreamLogMeta {
            build_id: build_id.to_string(),
            platform: "ios".to_string(),
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    if output.success {
        Ok(())
    } else {
        Err(format!("xcodebuild 失败，退出码: {:?}", output.exit_code))
    }
}

pub(super) fn resolve_ios_build_environment() -> Result<IosBuildEnvironment, String> {
    let xcodebuild_bin =
        crate::commands::shared::env::resolve_configured_tool_bin("xcode", "xcodebuild")?;
    let developer_dir = xcodebuild_bin
        .parent()
        .and_then(|bin| bin.parent())
        .and_then(|usr| usr.parent())
        .and_then(|developer| {
            (developer.file_name().and_then(|n| n.to_str()) == Some("Developer"))
                .then(|| developer.to_path_buf())
        })
        .or_else(|| {
            let configured =
                crate::commands::shared::env::require_configured_tool_path("xcode").ok()?;
            if configured.extension().and_then(|ext| ext.to_str()) == Some("app") {
                Some(configured.join("Contents/Developer"))
            } else {
                configured.parent().map(|p| p.to_path_buf())
            }
        })
        .ok_or_else(|| {
            format!(
                "无法从 xcodebuild 路径推导 DEVELOPER_DIR: {}",
                xcodebuild_bin.display()
            )
        })?;
    if !developer_dir.exists() {
        return Err(format!(
            "Xcode DEVELOPER_DIR 不存在: {}",
            developer_dir.display()
        ));
    }
    Ok(IosBuildEnvironment {
        xcodebuild_bin,
        developer_dir,
    })
}

fn ios_process_env(env: &IosBuildEnvironment) -> Vec<(String, String)> {
    vec![(
        "DEVELOPER_DIR".into(),
        env.developer_dir.to_string_lossy().to_string(),
    )]
}
