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
use super::fs_utils::{clean_copied_project, find_scheme_name, find_xcodeproj, safe_file_name};
use super::logging::{emit_ios_log, emit_version_warning_if_needed};
use super::pbxproj::{enable_pbx_system_capability, patch_pbxproj};
use super::plist::patch_info_plist;
use super::pod::{integrate_ios_pods, IosPodContext};
use super::runtime::{
    import_app_resource, patch_control_xml, resolve_ios_runtime_layout,
    verify_pod_privacy_manifest, verify_privacy_manifest,
};
use super::sdk_support::{
    materialize_ios_sdk_support_for_pod, prepare_ios_sdk_support,
    repair_ios_sdk_support_alignment_for_project,
};
use super::splashscreen::apply_ios_splashscreen;
use super::IosPackagingMode;
use crate::commands::ios::modules::bluetooth::apply_ios_bluetooth_module;
use crate::commands::ios::modules::bluetooth::ios_bluetooth_background_enabled;
use crate::commands::ios::modules::facial_recognition_verify::{
    apply_ios_facial_recognition_verify_module, ios_facial_recognition_verify_enabled,
};
use crate::commands::ios::modules::geolocation::apply_ios_geolocation_module;
use crate::commands::ios::modules::ibeacon::apply_ios_ibeacon_module;
use crate::commands::ios::modules::ibeacon::ios_ibeacon_enabled;
use crate::commands::ios::modules::livepusher::apply_ios_livepusher_module;
use crate::commands::ios::modules::map::apply_ios_map_module;
use crate::commands::ios::modules::oauth::apply_ios_oauth_module;
use crate::commands::ios::modules::payment::apply_ios_payment_module;
use crate::commands::ios::modules::push::{apply_ios_push_module, ios_push_enabled};
use crate::commands::ios::modules::share::apply_ios_share_module;
use crate::commands::ios::modules::speech::apply_ios_speech_module;
use crate::commands::ios::modules::statistic::apply_ios_statistic_module;
use crate::commands::ios::modules::ui_webview::apply_ios_ui_webview_module;
use crate::commands::ios::modules::uts_plugins::{
    apply_ios_uts_base_module, apply_ios_uts_plugins,
};
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
    pub(super) workspace_file: Option<PathBuf>,
    pub(super) scheme: String,
    pub(super) profile: MobileProvisionInfo,
}

impl IosWorkspace {
    pub(super) fn archive_destination_args(&self) -> Vec<String> {
        if let Some(workspace_file) = &self.workspace_file {
            vec![
                "-workspace".into(),
                workspace_file.to_string_lossy().to_string(),
            ]
        } else {
            vec![
                "-project".into(),
                self.project_file.to_string_lossy().to_string(),
            ]
        }
    }
}

pub(super) async fn configure_ios_workspace(
    project_id: &str,
    resource_path: &str,
    build_id: &str,
    supplied_manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    window: &tauri::Window,
    signing_validation_mode: MobileProvisionValidationMode,
    packaging_mode: IosPackagingMode,
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
    if let Some(support) = prepare_ios_sdk_support(&sdk_project, &workspace)? {
        for log in &support.logs {
            emit_ios_log(window, build_id, log.level, &log.message, Some(20));
        }
        let action = if support.copied_for_repair {
            format!(
                "已复制 iOS SDK 支持目录并完成静态库修复: {}",
                support.path.display()
            )
        } else {
            format!("已关联 iOS SDK 支持目录: {}", support.path.display())
        };
        emit_ios_log(window, build_id, "success", &action, Some(20));
        if !support.repaired_libraries.is_empty() {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已按 DCloud 教程修复 iOS 静态库 8-byte alignment: {}",
                    support.repaired_libraries.join("、")
                ),
                Some(21),
            );
        }
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
    let mut workspace_file = None;
    if packaging_mode == IosPackagingMode::AutoMigration {
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
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS 地图模块: {}，新增链接 {} 项，资源 {} 项",
                    map.summary(),
                    map.linked_count,
                    map.resource_count
                ),
                Some(29),
            );
        }
        if let Some(oauth) = apply_ios_oauth_module(&project_root, &project_file, manifest_info)? {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS Oauth 模块: {}，新增链接 {} 项，资源 {} 项",
                    oauth.summary(),
                    oauth.linked_count,
                    oauth.resource_count
                ),
                Some(29),
            );
        }
        if let Some(share) = apply_ios_share_module(&project_root, &project_file, manifest_info)? {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS 分享模块: {}，新增链接 {} 项，资源 {} 项",
                    share.summary(),
                    share.linked_count,
                    share.resource_count
                ),
                Some(29),
            );
        }
        if let Some(payment) =
            apply_ios_payment_module(&project_root, &project_file, manifest_info)?
        {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS 支付模块: {}，新增链接 {} 项，资源 {} 项",
                    payment.summary(),
                    payment.linked_count,
                    payment.resource_count
                ),
                Some(29),
            );
        }
        if let Some(speech) = apply_ios_speech_module(&project_root, &project_file, manifest_info)?
        {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS 语音输入模块: {}，新增链接 {} 项，资源 {} 项",
                    speech.summary(),
                    speech.linked_count,
                    speech.resource_count
                ),
                Some(29),
            );
        }
        if let Some(statistic) =
            apply_ios_statistic_module(&project_root, &project_file, manifest_info)?
        {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS 统计模块: {}，新增链接 {} 项，资源 {} 项",
                    statistic.summary(),
                    statistic.linked_count,
                    statistic.resource_count
                ),
                Some(29),
            );
        }
        let facial_recognition_verify_enabled =
            ios_facial_recognition_verify_enabled(manifest_info);
        let ios_uts_builtin_ext_api_required = scan
            .uts
            .builtin_modules
            .iter()
            .any(|module| module.ios_dir.is_some());
        if facial_recognition_verify_enabled || scan.uts.has_ios_uts_plugins {
            let uts_base = apply_ios_uts_base_module(
                &project_root,
                &project_file,
                ios_uts_builtin_ext_api_required,
            )?;
            emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已按文档接入 iOS UTS 基础模块: 新增链接 {} 项，Embed {} 项，ExtAPI {} 项，移除重复链接 {} 项",
                uts_base.linked_count,
                uts_base.embedded_count,
                uts_base.ext_api_count,
                uts_base.removed_duplicate_count
            ),
            Some(29),
        );
        }
        if let Some(facial) =
            apply_ios_facial_recognition_verify_module(&project_root, &project_file, manifest_info)?
        {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS 实人认证模块: 新增链接 {} 项，资源 {} 项",
                    facial.linked_count, facial.resource_count
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
            emit_ios_log(
            window,
            build_id,
            "warn",
            "LivePusher 依赖的 UPLiveSDKDll.framework 通常只包含真机架构；如在模拟器编译出现 AudioProcessor/RtcManager/UPAVPlayer 等 Undefined symbol，请改用真机或 Archive 构建，或关闭 LivePusher 模块",
            Some(29),
        );
        }
        if let Some(ui_webview) =
            apply_ios_ui_webview_module(&project_root, &project_file, manifest_info)?
        {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已自动接入 iOS UIWebview 模块: 新增链接 {} 项",
                    ui_webview.linked_count
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
        if let Some(uts) = apply_ios_uts_plugins(&project_root, &project_file, &scan.uts)? {
            emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已按文档复制 iOS UTS 插件 app-ios 目录 {} 个，并接入源码 {} 项、本地库 {} 项、静态库 {} 项、系统库 {} 项、资源 {} 项、plist {} 项；新增链接 {} 项、Embed {} 项、搜索路径 {} 项，deploymentTarget 更新 {} 项；未执行 Podfile 修改或 pod install",
                uts.copied_plugin_count,
                uts.source_count,
                uts.local_framework_count,
                uts.static_library_count,
                uts.system_framework_count,
                uts.resource_count,
                uts.plist_count,
                uts.linked_count,
                uts.embedded_count,
                uts.framework_search_path_count
                    + uts.library_search_path_count
                    + uts.header_search_path_count,
                uts.deployment_target_update_count
            ),
            Some(31),
        );
            if uts.pod_dependency_count > 0 {
                emit_ios_log(
                window,
                build_id,
                "warn",
                &format!(
                    "检测到 iOS UTS 插件声明 {} 个 Pod 依赖；按当前要求暂未执行 HBuilderX 5.13+ Pod 集成",
                    uts.pod_dependency_count
                ),
                Some(31),
            );
            }
        }
    } else {
        if let Some(support) = materialize_ios_sdk_support_for_pod(&sdk_project, &workspace)? {
            for log in &support.logs {
                emit_ios_log(window, build_id, log.level, &log.message, Some(40));
            }
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已按 HBuilderX Pod 示例工程目录结构准备 SDK 支持目录: {}",
                    support.path.display()
                ),
                Some(40),
            );
        }
        apply_ios_pod_capabilities(&project_file, manifest_info)?;
        let pod = integrate_ios_pods(IosPodContext {
            workspace: &workspace,
            project_root: &project_root,
            project_file: &project_file,
            sdk_root: &sdk_root,
            manifest_info,
            scan: &scan,
            window,
            build_id,
        })
        .await?;
        emit_ios_log(
            window,
            build_id,
            "success",
            &format!(
                "已按 HBuilderX 5.13+ 本地 Pod 流程接入 {} 个 subspec",
                pod.subspecs.len()
            ),
            Some(45),
        );
        workspace_file = Some(pod.workspace_file);
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
                Some(46),
            );
        }
    }
    let runtime_layout = resolve_ios_runtime_layout(&project_root)?;
    import_app_resource(&runtime_layout.apps_dir, &app_resource_dir, &scan.app_id)?;
    patch_control_xml(&runtime_layout.control_xml, &scan.app_id)?;
    generate_app_icons(&project_root, &config, manifest_info)?;
    if packaging_mode == IosPackagingMode::LocalPod {
        verify_pod_privacy_manifest(&workspace, &project_root)?;
    } else {
        verify_privacy_manifest(&workspace, &project_file)?;
    }
    if let Some(support) =
        repair_ios_sdk_support_alignment_for_project(&sdk_project, &workspace, &project_file)?
    {
        for log in &support.logs {
            emit_ios_log(window, build_id, log.level, &log.message, Some(52));
        }
        if support.copied_for_repair {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已使用 workspace SDK 副本完成静态库修复: {}",
                    support.path.display()
                ),
                Some(53),
            );
        }
        if !support.repaired_libraries.is_empty() {
            emit_ios_log(
                window,
                build_id,
                "success",
                &format!(
                    "已按 DCloud 教程修复 iOS 静态库 8-byte alignment: {}",
                    support.repaired_libraries.join("、")
                ),
                Some(54),
            );
        }
    }
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
        workspace_file,
        scheme,
        profile,
    })
}

fn apply_ios_pod_capabilities(
    project_file: &std::path::Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    if ios_bluetooth_background_enabled(manifest_info) || ios_ibeacon_enabled(manifest_info) {
        enable_pbx_system_capability(project_file, "com.apple.BackgroundModes")?;
    }
    if ios_push_enabled(manifest_info) {
        enable_pbx_system_capability(project_file, "com.apple.BackgroundModes")?;
        enable_pbx_system_capability(project_file, "com.apple.Push")?;
    }
    Ok(())
}
