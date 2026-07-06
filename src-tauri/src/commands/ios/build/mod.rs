//! iOS 离线 SDK 工程配置与 IPA 导出。
//!
//! 工程始终来自用户配置的 DCloud iOS 离线 SDK 自带 HBuilder-Hello*，
//! 本模块只复制该工程到 workspace 后修改副本。

mod config;
mod entitlements;
pub(crate) mod fs_utils;
mod logging;
pub(crate) mod pbxproj;
mod plist;
mod pod;
mod pod_config;
mod pod_subspecs;
mod runtime;
mod splashscreen;
mod workspace;

use super::build_env::{resolve_ios_build_environment, run_xcodebuild};
use super::signing::{write_export_options, MobileProvisionValidationMode};
use config::ensure_macos;
use fs_utils::{expand_home, find_file_with_ext};
use logging::emit_ios_log;
use workspace::configure_ios_workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum IosPackagingMode {
    AutoMigration,
    LocalPod,
}

impl Default for IosPackagingMode {
    fn default() -> Self {
        Self::AutoMigration
    }
}

#[tauri::command]
pub async fn generate_ios_project(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    ios_packaging_mode: Option<IosPackagingMode>,
    window: tauri::Window,
) -> Result<String, String> {
    ensure_macos("iOS 工程生成")?;
    let build_id = build_id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("ios-gen-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    emit_ios_log(&window, &build_id, "info", "开始生成 iOS 工程", Some(2));
    let _env = resolve_ios_build_environment()?;
    let workspace = configure_ios_workspace(
        &project_id,
        &resource_path,
        &build_id,
        manifest_info.as_ref(),
        &window,
        MobileProvisionValidationMode::ProjectGeneration,
        ios_packaging_mode.unwrap_or_default(),
    )
    .await?;
    emit_ios_log(
        &window,
        &build_id,
        "success",
        &format!("iOS 工程已生成: {}", workspace.project_root.display()),
        Some(100),
    );
    Ok(workspace.project_root.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn build_ios_ipa(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    ios_packaging_mode: Option<IosPackagingMode>,
    window: tauri::Window,
) -> Result<crate::commands::android::BuildArtifact, String> {
    ensure_macos("iOS 打包")?;
    let build_id = build_id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("ios-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    emit_ios_log(&window, &build_id, "info", "开始 iOS IPA 构建", Some(2));
    let env = resolve_ios_build_environment()?;
    let workspace = configure_ios_workspace(
        &project_id,
        &resource_path,
        &build_id,
        manifest_info.as_ref(),
        &window,
        MobileProvisionValidationMode::IpaExport,
        ios_packaging_mode.unwrap_or_default(),
    )
    .await?;
    let archive_path = workspace.workspace.join("build/output.xcarchive");
    let export_options = workspace.workspace.join("ExportOptions.plist");
    let export_path = workspace.workspace.join("build/export");

    write_export_options(&export_options, &workspace.config, &workspace.profile)?;
    emit_ios_log(
        &window,
        &build_id,
        "info",
        "执行 xcodebuild archive",
        Some(65),
    );
    let mut archive_args = workspace.archive_destination_args();
    archive_args.extend([
        "-scheme".into(),
        workspace.scheme.clone(),
        "-configuration".into(),
        "Release".into(),
        "-quiet".into(),
        "-destination".into(),
        "generic/platform=iOS".into(),
        "-archivePath".into(),
        archive_path.to_string_lossy().to_string(),
        "archive".into(),
        format!("DEVELOPMENT_TEAM={}", workspace.config.ios.team_id),
        format!(
            "PRODUCT_BUNDLE_IDENTIFIER={}",
            workspace.config.ios.bundle_id
        ),
        format!(
            "PROVISIONING_PROFILE_SPECIFIER={}",
            workspace.profile.specifier()
        ),
        "CODE_SIGN_STYLE=Manual".into(),
    ]);
    run_xcodebuild(
        &archive_args,
        &workspace.project_root,
        &window,
        &env,
        &build_id,
    )
    .await?;

    emit_ios_log(
        &window,
        &build_id,
        "info",
        "执行 xcodebuild exportArchive",
        Some(85),
    );
    run_xcodebuild(
        &[
            "-exportArchive".into(),
            "-quiet".into(),
            "-archivePath".into(),
            archive_path.to_string_lossy().to_string(),
            "-exportPath".into(),
            export_path.to_string_lossy().to_string(),
            "-exportOptionsPlist".into(),
            export_options.to_string_lossy().to_string(),
        ],
        &workspace.project_root,
        &window,
        &env,
        &build_id,
    )
    .await?;

    let ipa = find_file_with_ext(&export_path, "ipa")
        .ok_or_else(|| "导出成功后未找到 IPA 文件".to_string())?;
    let output_dir = expand_home(&workspace.config.output_dir);
    crate::utils::fs::ensure_directory(&output_dir).map_err(|e| e.to_string())?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dest = output_dir.join(format!("{}-v{}.ipa", timestamp, workspace.app_version));
    std::fs::copy(&ipa, &dest).map_err(|e| format!("复制 IPA 失败: {}", e))?;
    let size_bytes = std::fs::metadata(&dest)
        .map(|m| m.len())
        .unwrap_or_default();
    emit_ios_log(
        &window,
        &build_id,
        "success",
        &format!("iOS 打包完成: {}", dest.display()),
        Some(100),
    );
    Ok(crate::commands::android::BuildArtifact {
        platform: "ios".to_string(),
        path: dest.to_string_lossy().to_string(),
        file_name: dest
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app.ipa")
            .to_string(),
        size_bytes,
        build_id,
        cloud_run_url: None,
    })
}

#[cfg(test)]
mod tests;
