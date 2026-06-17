use std::path::PathBuf;

use super::super::icons::generate_app_icons;
use super::super::signing::{
    import_p12_certificate, install_mobileprovision, MobileProvisionInfo,
    MobileProvisionValidationMode,
};
use super::config::{
    effective_app_name, effective_app_version, effective_app_version_code,
    resolve_ios_manifest_info, validate_ios_app_id, validate_ios_config,
};
use super::entitlements::patch_ios_entitlements;
use super::fs_utils::{
    clean_copied_project, find_scheme_name, find_xcodeproj, link_ios_sdk_support, safe_file_name,
};
use super::logging::{emit_ios_log, emit_version_warning_if_needed};
use super::pbxproj::patch_pbxproj;
use super::plist::patch_info_plist;
use super::runtime::{
    import_app_resource, patch_control_xml, resolve_ios_runtime_layout, verify_privacy_manifest,
};
use super::splashscreen::apply_ios_splashscreen;
use crate::commands::ios::modules::bluetooth::apply_ios_bluetooth_module;
use crate::commands::ios::modules::facial_recognition_verify::apply_ios_facial_recognition_verify_module;
use crate::commands::ios::modules::geolocation::apply_ios_geolocation_module;
use crate::commands::ios::modules::ibeacon::apply_ios_ibeacon_module;
use crate::commands::ios::modules::livepusher::apply_ios_livepusher_module;
use crate::commands::ios::modules::map::apply_ios_map_module;
use crate::commands::ios::modules::oauth::apply_ios_oauth_module;
use crate::commands::ios::modules::push::apply_ios_push_module;
use crate::commands::ios::modules::share::apply_ios_share_module;
use crate::commands::module::{
    manifest_push_unsupported_version, PUSH_UNSUPPORTED_VERSION_MESSAGE,
};

#[derive(Debug, Clone)]
pub(super) struct IosWorkspace {
    pub(super) config: crate::commands::project::ProjectConfig,
    pub(super) app_version: String,
    pub(super) workspace: PathBuf,
    pub(super) project_root: PathBuf,
    pub(super) project_file: PathBuf,
    pub(super) scheme: String,
    pub(super) profile: MobileProvisionInfo,
}

pub(super) fn configure_ios_workspace(
    project_id: &str,
    resource_path: &str,
    build_id: &str,
    supplied_manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    window: &tauri::Window,
    signing_validation_mode: MobileProvisionValidationMode,
) -> Result<IosWorkspace, String> {
    let config = crate::commands::project::load_project_config_sync(project_id)?;
    let manifest_info = resolve_ios_manifest_info(&config, supplied_manifest_info)?;
    let manifest_info = manifest_info.as_ref();
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    validate_ios_config(&config, &sdk_config)?;

    let resource_dir = PathBuf::from(resource_path);
    let scan = crate::commands::shared::resource_scan::scan_imported_resource(
        &resource_dir,
        &resource_dir,
        false,
    )?;
    validate_ios_app_id(&scan.app_id, manifest_info)?;
    let app_resource_dir = PathBuf::from(&scan.app_resource_path);
    emit_ios_log(
        window,
        build_id,
        "info",
        &format!("识别 iOS AppId: {}", scan.app_id),
        Some(10),
    );
    if let Some(info) = manifest_info {
        emit_ios_log(
            window,
            build_id,
            "info",
            &format!("iOS 构建读取 manifest.json: {}", info.manifest_path),
            Some(11),
        );
        emit_ios_log(
            window,
            build_id,
            "info",
            &format!(
                "iOS manifest 配置: 名称 {}，版本 {} ({})，图标 {} 项，隐私描述 {} 项",
                effective_app_name(&config, manifest_info),
                effective_app_version(&config, manifest_info),
                effective_app_version_code(&config, manifest_info),
                info.ios_icons
                    .as_ref()
                    .map(|icons| icons.ios.len())
                    .unwrap_or_default(),
                info.ios_privacy_descriptions.len()
            ),
            Some(12),
        );
        if info
            .manifest_value
            .as_ref()
            .is_some_and(manifest_push_unsupported_version)
        {
            emit_ios_log(
                window,
                build_id,
                "warn",
                PUSH_UNSUPPORTED_VERSION_MESSAGE,
                Some(28),
            );
        }
    }

    let sdk_root = crate::commands::sdk::resolve_ios_sdk_root(&PathBuf::from(
        &sdk_config.dcloud_ios_sdk_path,
    ))?;
    let sdk_project = crate::commands::sdk::resolve_ios_sdk_project(&sdk_root)?;
    emit_version_warning_if_needed(window, build_id, &scan, &sdk_project);

    let workspace = crate::utils::fs::get_project_config_dir(project_id)
        .join("workspace")
        .join(safe_file_name(build_id));
    if workspace.exists() {
        std::fs::remove_dir_all(&workspace)
            .map_err(|e| format!("清理旧 iOS workspace 失败 {}: {}", workspace.display(), e))?;
    }
    crate::utils::fs::ensure_directory(&workspace).map_err(|e| e.to_string())?;

    let project_root = workspace.join(
        sdk_project
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("HBuilder-Hello"),
    );
    crate::utils::fs::copy_recursive(&sdk_project, &project_root)
        .map_err(|e| format!("复制 SDK 自带 HBuilder-Hello 失败: {}", e))?;
    clean_copied_project(&project_root)?;
    if let Some(support_dir) = link_ios_sdk_support(&sdk_project, &workspace)? {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!("已关联 iOS SDK 支持目录: {}", support_dir.display()),
            Some(20),
        );
    }
    emit_ios_log(
        window,
        build_id,
        "success",
        "已复制 SDK 自带 HBuilder-Hello 到 workspace",
        Some(22),
    );

    let project_file = find_xcodeproj(&project_root)
        .ok_or_else(|| "复制后的 HBuilder-Hello 中未找到 .xcodeproj".to_string())?;
    let uses_legacy_simulator_arch = patch_pbxproj(&project_file, &config, manifest_info)?;
    if uses_legacy_simulator_arch {
        emit_ios_log(
            window,
            build_id,
            "info",
            "检测到旧式 iOS framework，模拟器构建将使用 x86_64",
            Some(25),
        );
    }
    if let Some(resource_count) =
        apply_ios_splashscreen(&project_root, &project_file, manifest_info)?
    {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已应用 manifest iOS storyboard 启动界面，并注册 {} 个引用资源",
                resource_count
            ),
            Some(27),
        );
    }
    patch_info_plist(
        &project_root,
        &project_file,
        &config,
        &scan.app_id,
        manifest_info,
    )?;
    if let Some(geolocation) =
        apply_ios_geolocation_module(&project_root, &project_file, manifest_info)?
    {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已自动接入 iOS 定位模块: {}，新增链接 {} 项",
                geolocation.provider_summary(),
                geolocation.linked_count
            ),
            Some(29),
        );
    }
    if let Some(map) = apply_ios_map_module(&project_root, &project_file, manifest_info)? {
        let (level, message) = if map.local_pod {
            (
                "info",
                format!(
                    "已启用 iOS 地图模块本地 Pod 集成: {}，请确保使用 HBuilderX 5.13+ 导出的本地 APP 打包资源",
                    map.summary()
                ),
            )
        } else {
            (
                "success",
                format!(
                    "已自动接入 iOS 地图模块: {}，新增链接 {} 项，资源 {} 项",
                    map.summary(),
                    map.linked_count,
                    map.resource_count
                ),
            )
        };
        emit_ios_log(window, build_id, level, &message, Some(29));
    }
    if let Some(oauth) = apply_ios_oauth_module(&project_root, &project_file, manifest_info)? {
        let (level, message) = if oauth.local_pod {
            (
                "info",
                format!(
                    "已启用 iOS Oauth 模块本地 Pod 集成: {}，请确保使用 HBuilderX 5.13+ 导出的本地 APP 打包资源",
                    oauth.summary()
                ),
            )
        } else {
            (
                "success",
                format!(
                    "已自动接入 iOS Oauth 模块: {}，新增链接 {} 项，资源 {} 项",
                    oauth.summary(),
                    oauth.linked_count,
                    oauth.resource_count
                ),
            )
        };
        emit_ios_log(window, build_id, level, &message, Some(29));
    }
    if let Some(share) = apply_ios_share_module(&project_root, &project_file, manifest_info)? {
        let (level, message) = if share.local_pod {
            (
                "info",
                format!(
                    "已启用 iOS 分享模块本地 Pod 集成: {}，请确保使用 HBuilderX 5.13+ 导出的本地 APP 打包资源",
                    share.summary()
                ),
            )
        } else {
            (
                "success",
                format!(
                    "已自动接入 iOS 分享模块: {}，新增链接 {} 项，资源 {} 项",
                    share.summary(),
                    share.linked_count,
                    share.resource_count
                ),
            )
        };
        emit_ios_log(window, build_id, level, &message, Some(29));
    }
    if let Some(facial) =
        apply_ios_facial_recognition_verify_module(&project_root, &project_file, manifest_info)?
    {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已自动接入 iOS 实人认证模块: 新增链接 {} 项，Embed {} 项，资源 {} 项，移除 UTS 重复链接 {} 项",
                facial.linked_count,
                facial.embedded_count,
                facial.resource_count,
                facial.removed_duplicate_count
            ),
            Some(29),
        );
    }
    if let Some(livepusher) = apply_ios_livepusher_module(
        &project_root,
        &project_file,
        manifest_info,
        &config.ios_module_config,
    )? {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已自动接入 iOS LivePusher 模块: 新增链接 {} 项，Embed {} 项",
                livepusher.linked_count, livepusher.embedded_count
            ),
            Some(29),
        );
    }
    if let Some(bluetooth) = apply_ios_bluetooth_module(&project_file, manifest_info)? {
        if bluetooth.background_enabled {
            emit_ios_log(
                window,
                build_id,
                "success",
                "已开启 iOS 蓝牙后台模式能力",
                Some(30),
            );
        }
    }
    if let Some(push) = apply_ios_push_module(&project_root, &project_file, manifest_info)? {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已自动接入 iOS Push 模块: 后台模式 {}，新增链接 {} 项",
                push.background_modes.join("、"),
                push.linked_count
            ),
            Some(30),
        );
    }
    if let Some(ibeacon) = apply_ios_ibeacon_module(&project_file, manifest_info)? {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已开启 iOS iBeacon 后台模式能力: {}",
                ibeacon.background_modes.join("、")
            ),
            Some(30),
        );
    }
    let associated_domain_count =
        patch_ios_entitlements(&project_root, &project_file, manifest_info)?;
    if associated_domain_count > 0 {
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已从 manifest 配置 {} 个 iOS Associated Domains",
                associated_domain_count
            ),
            Some(30),
        );
    }
    let runtime_layout = resolve_ios_runtime_layout(&project_root)?;
    import_app_resource(&runtime_layout.apps_dir, &app_resource_dir, &scan.app_id)?;
    patch_control_xml(&runtime_layout.control_xml, &scan.app_id)?;
    generate_app_icons(&project_root, &config, manifest_info)?;
    verify_privacy_manifest(&workspace, &project_file)?;
    let profile = install_mobileprovision(&config, signing_validation_mode)?;
    import_p12_certificate(&config)?;
    emit_ios_log(window, build_id, "success", "iOS 工程配置完成", Some(55));

    let scheme = find_scheme_name(&project_file).unwrap_or_else(|| {
        project_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("HBuilder-Hello")
            .to_string()
    });
    Ok(IosWorkspace {
        app_version: effective_app_version(&config, manifest_info),
        config,
        workspace,
        project_root,
        project_file,
        scheme,
        profile,
    })
}
