//! iOS 离线 SDK 工程配置与 IPA 导出。
//!
//! 工程始终来自用户配置的 DCloud iOS 离线 SDK 自带 HBuilder-Hello*，
//! 本模块只复制该工程到 workspace 后修改副本。

use std::path::{Path, PathBuf};
use tauri::Emitter;

use super::build_env::{resolve_ios_build_environment, run_xcodebuild};
use super::icons::generate_app_icons;
use super::signing::{
    import_p12_certificate, install_mobileprovision, write_export_options, MobileProvisionInfo,
};

#[derive(Debug, Clone)]
struct IosWorkspace {
    config: crate::commands::project::ProjectConfig,
    app_version: String,
    workspace: PathBuf,
    project_root: PathBuf,
    project_file: PathBuf,
    scheme: String,
    profile: MobileProvisionInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IosRuntimeLayout {
    control_xml: PathBuf,
    apps_dir: PathBuf,
}

#[tauri::command]
pub async fn generate_ios_project(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
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
    )?;
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
    )?;
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
    run_xcodebuild(
        &[
            "-project".into(),
            workspace.project_file.to_string_lossy().to_string(),
            "-scheme".into(),
            workspace.scheme.clone(),
            "-configuration".into(),
            "Release".into(),
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
        ],
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
    })
}

fn configure_ios_workspace(
    project_id: &str,
    resource_path: &str,
    build_id: &str,
    supplied_manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    window: &tauri::Window,
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
    }

    let sdk_project = crate::commands::sdk::resolve_ios_sdk_project(&PathBuf::from(
        &sdk_config.dcloud_ios_sdk_path,
    ))?;
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
    let profile = install_mobileprovision(&config)?;
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

fn resolve_ios_manifest_info(
    config: &crate::commands::project::ProjectConfig,
    supplied: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<crate::commands::resource::UniappManifestInfo>, String> {
    if !config.local_path.trim().is_empty() {
        return crate::commands::shared::resource::read_uniapp_manifest_sync(&config.local_path)
            .map(Some)
            .map_err(|e| {
                format!(
                    "读取 iOS 本地项目 manifest.json 失败 ({}): {}",
                    config.local_path, e
                )
            });
    }
    Ok(supplied.cloned())
}

fn effective_app_name(
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> String {
    manifest_info
        .and_then(|info| info.app_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&config.app.name)
        .to_string()
}

fn effective_app_version(
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> String {
    manifest_info
        .and_then(|info| info.version_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&config.app.version)
        .to_string()
}

fn effective_app_version_code(
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> u32 {
    manifest_info
        .and_then(|info| info.version_code)
        .unwrap_or(config.app.version_code)
}

fn validate_ios_app_id(
    resource_app_id: &str,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let Some(manifest_app_id) = manifest_info
        .and_then(|info| info.app_id.as_deref())
        .filter(|app_id| !app_id.trim().is_empty())
    else {
        return Ok(());
    };
    if manifest_app_id == resource_app_id {
        return Ok(());
    }
    Err(format!(
        "iOS 本地 manifest AppId ({}) 与导入资源 AppId ({}) 不一致，无法安全配置 control.xml",
        manifest_app_id, resource_app_id
    ))
}

fn validate_ios_config(
    config: &crate::commands::project::ProjectConfig,
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<(), String> {
    if sdk_config.dcloud_ios_sdk_path.trim().is_empty() {
        return Err("请先在 SDK & 环境管理中配置 DCloud iOS 离线 SDK".to_string());
    }
    if config.ios.dcloud_app_key.trim().is_empty() {
        return Err("请先配置 iOS DCloud AppKey".to_string());
    }
    if config.ios.bundle_id.trim().is_empty() || config.ios.team_id.trim().is_empty() {
        return Err("请先配置 iOS Bundle ID 和 Team ID".to_string());
    }
    if config.ios.provisioning_profile.trim().is_empty() {
        return Err("请先选择 iOS 描述文件 mobileprovision".to_string());
    }
    if !config.ios.certificate.trim().is_empty() && !config.ios.has_certificate_password {
        return Err("导入 iOS P12 证书需要先保存证书密码".to_string());
    }
    Ok(())
}

fn patch_pbxproj(
    project_file: &Path,
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<bool, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let content = content.replace("io.dcloud.HBuilder", &config.ios.bundle_id);
    let content =
        set_pbx_build_setting(&content, "PRODUCT_BUNDLE_IDENTIFIER", &config.ios.bundle_id);
    let content = set_pbx_build_setting(&content, "DEVELOPMENT_TEAM", &config.ios.team_id);
    let content = set_pbx_build_setting(
        &content,
        "INFOPLIST_KEY_CFBundleDisplayName",
        &effective_app_name(config, manifest_info),
    );
    let content = set_pbx_build_setting(
        &content,
        "MARKETING_VERSION",
        &effective_app_version(config, manifest_info),
    );
    let content = set_pbx_build_setting(
        &content,
        "CURRENT_PROJECT_VERSION",
        &effective_app_version_code(config, manifest_info).to_string(),
    );
    let content = if classic_linker_available() {
        append_pbx_build_setting_flag(&content, "OTHER_LDFLAGS", "-ld_classic")
    } else {
        content
    };
    let uses_legacy_simulator_arch = legacy_simulator_x86_64_required(project_file);
    let content = if uses_legacy_simulator_arch {
        set_pbx_build_setting(&content, "\"ARCHS[sdk=iphonesimulator*]\"", "x86_64")
    } else {
        content
    };
    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(uses_legacy_simulator_arch)
}

fn legacy_simulator_x86_64_required(project_file: &Path) -> bool {
    project_file
        .parent()
        .and_then(Path::parent)
        .map(|workspace| {
            workspace
                .join("SDK/Libs/DCUniRecord.framework/DCUniRecord")
                .is_file()
        })
        .unwrap_or(false)
}

fn classic_linker_available() -> bool {
    std::process::Command::new("xcrun")
        .args(["--find", "ld-classic"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn append_pbx_build_setting_flag(content: &str, key: &str, flag: &str) -> String {
    let pattern = regex::Regex::new(&format!(
        r"(?m)^(\s*{}\s*=\s*)([^;]*)(;)",
        regex::escape(key)
    ))
    .expect("valid pbx setting regex");
    pattern
        .replace_all(content, |caps: &regex::Captures| {
            let value = caps.get(2).map(|value| value.as_str()).unwrap_or_default();
            if pbx_value_contains_flag(value, flag) {
                return caps
                    .get(0)
                    .map(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
            let trimmed = value.trim();
            let updated = if trimmed.starts_with('"') && trimmed.ends_with('"') {
                format!("\"{} {}\"", &trimmed[1..trimmed.len() - 1], flag)
            } else if trimmed.is_empty() {
                render_pbx_value(flag)
            } else {
                render_pbx_value(&format!("{} {}", trimmed, flag))
            };
            format!("{}{}{}", &caps[1], updated, &caps[3])
        })
        .into_owned()
}

fn pbx_value_contains_flag(value: &str, flag: &str) -> bool {
    value
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '"' | ',' | '(' | ')'))
        .any(|token| token == flag)
}

fn set_pbx_build_setting(content: &str, key: &str, value: &str) -> String {
    let rendered = render_pbx_value(value);
    let pattern = regex::Regex::new(&format!(r"(?m)^(\s*{}\s*=\s*)[^;]*;", regex::escape(key)))
        .expect("valid pbx setting regex");
    if pattern.is_match(content) {
        return pattern
            .replace_all(content, |caps: &regex::Captures| {
                format!(
                    "{}{};",
                    caps.get(1).map_or("", |value| value.as_str()),
                    rendered
                )
            })
            .into_owned();
    }

    let mut output = String::with_capacity(content.len() + key.len() + value.len() + 64);
    let mut in_build_settings = false;
    for line in content.lines() {
        if in_build_settings && line.trim() == "};" {
            output.push_str(&format!("\t\t\t\t{} = {};\n", key, rendered));
            in_build_settings = false;
        }
        output.push_str(line);
        output.push('\n');
        if line.contains("buildSettings = {") {
            in_build_settings = true;
        }
    }
    output
}

fn render_pbx_value(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '*'))
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}

fn patch_info_plist(
    project_root: &Path,
    project_file: &Path,
    config: &crate::commands::project::ProjectConfig,
    app_id: &str,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let plist_path = find_info_plist(project_root, project_file)
        .ok_or_else(|| "未找到主工程 Info.plist".to_string())?;
    let mut value =
        plist::Value::from_file(&plist_path).map_err(|e| format!("解析 Info.plist 失败: {}", e))?;
    let dict = value
        .as_dictionary_mut()
        .ok_or_else(|| "Info.plist 不是 dictionary".to_string())?;
    dict.insert(
        "dcloud_appkey".into(),
        plist::Value::String(config.ios.dcloud_app_key.clone()),
    );
    dict.insert(
        "CFBundleDisplayName".into(),
        plist::Value::String(effective_app_name(config, manifest_info)),
    );
    dict.insert(
        "CFBundleShortVersionString".into(),
        plist::Value::String(effective_app_version(config, manifest_info)),
    );
    dict.insert(
        "CFBundleVersion".into(),
        plist::Value::String(effective_app_version_code(config, manifest_info).to_string()),
    );
    dict.insert(
        "marketChannel".into(),
        plist::Value::String(format!("{}|{}||apple", config.ios.bundle_id, app_id)),
    );
    if find_file_named(project_root, "LaunchScreen.storyboard").is_some() {
        dict.insert(
            "UILaunchStoryboardName".into(),
            plist::Value::String("LaunchScreen".into()),
        );
    }
    set_dcloud_default_theme(dict);
    if let Some(info) = manifest_info {
        apply_ios_privacy_descriptions(dict, &info.ios_privacy_descriptions);
        if let Some(manifest) = info.manifest_value.as_ref() {
            apply_ios_manifest_plist(dict, manifest);
        }
    } else {
        cleanup_ios_privacy_descriptions(dict);
    }
    patch_info_plist_strings(project_root, &effective_app_name(config, manifest_info))?;
    value
        .to_file_xml(&plist_path)
        .map_err(|e| format!("写入 Info.plist 失败: {}", e))
}

fn apply_ios_privacy_descriptions(
    dict: &mut plist::Dictionary,
    descriptions: &std::collections::BTreeMap<String, String>,
) {
    cleanup_ios_privacy_descriptions(dict);
    for (key, description) in descriptions {
        let description = description.trim();
        if is_supported_ios_privacy_description_key(key) && !description.is_empty() {
            dict.insert(key.clone(), plist::Value::String(description.to_string()));
        }
    }
    cleanup_ios_privacy_descriptions(dict);
}

fn cleanup_ios_privacy_descriptions(dict: &mut plist::Dictionary) {
    promote_duplicate_ios_privacy_descriptions(dict);
    let keys = dict.keys().cloned().collect::<Vec<_>>();
    for key in keys {
        if is_duplicate_ios_privacy_key(&key) || is_empty_legacy_ios_privacy_value(dict, &key) {
            dict.remove(&key);
        }
    }
}

fn promote_duplicate_ios_privacy_descriptions(dict: &mut plist::Dictionary) {
    let keys = dict.keys().cloned().collect::<Vec<_>>();
    for key in keys {
        let Some(base) = duplicate_ios_privacy_base_key(&key) else {
            continue;
        };
        let Some(value) = dict
            .get(&key)
            .and_then(plist::Value::as_string)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        let base_is_empty = dict
            .get(&base)
            .and_then(plist::Value::as_string)
            .map(str::trim)
            .is_none_or(str::is_empty);
        if base_is_empty {
            dict.insert(base, plist::Value::String(value));
        }
    }
}

fn is_duplicate_ios_privacy_key(key: &str) -> bool {
    duplicate_ios_privacy_base_key(key).is_some()
}

fn duplicate_ios_privacy_base_key(key: &str) -> Option<String> {
    let Some((base, suffix)) = key.rsplit_once(" - ") else {
        return None;
    };
    if suffix.chars().all(|ch| ch.is_ascii_digit())
        && (is_ios_privacy_description_key(base) || is_legacy_ios_privacy_description_key(base))
    {
        Some(base.to_string())
    } else {
        None
    }
}

fn is_empty_legacy_ios_privacy_value(dict: &plist::Dictionary, key: &str) -> bool {
    if !is_legacy_ios_privacy_description_key(key) {
        return false;
    }
    dict.get(key)
        .and_then(plist::Value::as_string)
        .map(str::trim)
        .is_some_and(str::is_empty)
}

fn is_ios_privacy_description_key(key: &str) -> bool {
    is_supported_ios_privacy_description_key(key)
}

fn is_legacy_ios_privacy_description_key(key: &str) -> bool {
    key == "NSLocationWhenInUseDescription"
}

fn is_supported_ios_privacy_description_key(key: &str) -> bool {
    IOS_PRIVACY_DESCRIPTION_KEYS.contains(&key)
}

const IOS_PRIVACY_DESCRIPTION_KEYS: &[&str] = &[
    "NSPhotoLibraryUsageDescription",
    "NSPhotoLibraryAddUsageDescription",
    "NSCameraUsageDescription",
    "NSMicrophoneUsageDescription",
    "NSLocationWhenInUseUsageDescription",
    "NSLocationAlwaysUsageDescription",
    "NSLocationAlwaysAndWhenInUseUsageDescription",
    "NSCalendarsUsageDescription",
    "NSContactsUsageDescription",
    "NSBluetoothPeripheralUsageDescription",
    "NSBluetoothAlwaysUsageDescription",
    "NSSpeechRecognitionUsageDescription",
    "NSRemindersUsageDescription",
    "NSMotionUsageDescription",
    "NSHealthUpdateUsageDescription",
    "NSHealthShareUsageDescription",
    "NSAppleMusicUsageDescription",
    "NFCReaderUsageDescription",
    "NSHealthClinicalHealthRecordsShareUsageDescription",
    "NSHomeKitUsageDescription",
    "NSSiriUsageDescription",
    "NSFaceIDUsageDescription",
    "NSLocalNetworkUsageDescription",
    "NSUserTrackingUsageDescription",
];

fn patch_info_plist_strings(project_root: &Path, app_name: &str) -> Result<(), String> {
    let mut files = Vec::new();
    collect_files_named_skipping_bundles(project_root, "InfoPlist.strings", &mut files);
    for path in files {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取 InfoPlist.strings 失败 {}: {}", path.display(), e))?;
        let updated = set_info_plist_string_value(&content, "CFBundleDisplayName", app_name);
        std::fs::write(&path, updated)
            .map_err(|e| format!("写入 InfoPlist.strings 失败 {}: {}", path.display(), e))?;
    }
    Ok(())
}

fn set_info_plist_string_value(content: &str, key: &str, value: &str) -> String {
    let pattern = regex::Regex::new(&format!(
        r#"(?m)^(\s*(?:"{}"|{})\s*=\s*)"(?:\\.|[^"\\])*"(\s*;)"#,
        regex::escape(key),
        regex::escape(key)
    ))
    .expect("valid InfoPlist.strings regex");
    let escaped = escape_info_plist_strings_value(value);
    if pattern.is_match(content) {
        return pattern
            .replace_all(content, |caps: &regex::Captures| {
                format!(
                    "{}\"{}\"{}",
                    caps.get(1).map_or("", |value| value.as_str()),
                    escaped,
                    caps.get(2).map_or("", |value| value.as_str())
                )
            })
            .into_owned();
    }

    let mut updated = content.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&format!("\"{}\" = \"{}\";\n", key, escaped));
    updated
}

fn escape_info_plist_strings_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn apply_ios_manifest_plist(dict: &mut plist::Dictionary, manifest: &serde_json::Value) {
    let schemes = ios_manifest_url_schemes(manifest);
    if !schemes.is_empty() {
        dict.insert(
            "CFBundleURLTypes".into(),
            plist::Value::Array(
                schemes
                    .iter()
                    .map(|scheme| {
                        let mut entry = plist::Dictionary::new();
                        entry.insert(
                            "CFBundleURLName".into(),
                            plist::Value::String(format!("unipack.{}", scheme)),
                        );
                        entry.insert(
                            "CFBundleURLSchemes".into(),
                            plist::Value::Array(vec![plist::Value::String(scheme.clone())]),
                        );
                        plist::Value::Dictionary(entry)
                    })
                    .collect(),
            ),
        );
    }

    let query_schemes = ios_manifest_query_schemes(manifest);
    if !query_schemes.is_empty() {
        merge_plist_string_array(dict, "LSApplicationQueriesSchemes", query_schemes);
    }

    let background_modes = ios_manifest_background_modes(manifest);
    if !background_modes.is_empty() {
        merge_plist_string_array(dict, "UIBackgroundModes", background_modes);
    }

    if let Some(appid) = provider_value(manifest, "weixin", &["appid"]) {
        set_plist_dictionary_values(dict, "weixin", &[("appid", appid)]);
    }
    if let Some(link) = universal_links(manifest).into_iter().next() {
        dict.insert("UniversalLinks".into(), plist::Value::String(link));
    }
    let sina_appkey = provider_value(manifest, "sina", &["appkey"]);
    let sina_redirect = provider_value(manifest, "sina", &["redirect_uri", "redirectURI"]);
    if sina_appkey.is_some() || sina_redirect.is_some() {
        let mut values = Vec::new();
        if let Some(value) = sina_appkey {
            values.push(("appkey", value));
        }
        if let Some(value) = sina_redirect {
            values.push(("redirectURI", value));
        }
        set_plist_dictionary_values(dict, "sinaweibo", &values);
    }
    if let Some(value) = provider_value(manifest, "google", &["clientid", "clientId"]) {
        dict.insert("GIDClientID".into(), plist::Value::String(value));
    }
    if let Some(value) = provider_value(manifest, "facebook", &["appid"]) {
        dict.insert("FacebookAppID".into(), plist::Value::String(value));
    }
    if let Some(value) = provider_value(manifest, "facebook", &["client_token", "clientToken"]) {
        dict.insert("FacebookClientToken".into(), plist::Value::String(value));
    }
    if let Some(value) = provider_value(manifest, "amap", &["appkey_ios", "apikey_ios"]) {
        set_plist_dictionary_values(dict, "amap", &[("appkey", value.clone())]);
        dict.insert("AMapApiKey".into(), plist::Value::String(value));
    }
    if let Some(value) = provider_value(manifest, "baidu", &["appkey_ios", "apikey_ios"]) {
        set_plist_dictionary_values(dict, "baidu", &[("appkey", value.clone())]);
        dict.insert("BaiduMapApiKey".into(), plist::Value::String(value));
    }
    if let Some(speech) = manifest_provider(manifest, "baidu", Some("speech")) {
        let app_id = json_string_field(speech, &["appid"]);
        let api_key = json_string_field(speech, &["apikey", "apiKey"]);
        let secret_key = json_string_field(speech, &["secretkey", "secretKey"]);
        let mut values = Vec::new();
        if let Some(value) = app_id.clone() {
            values.push(("APP_ID", value.clone()));
            dict.insert("BDSpeechAPPID".into(), plist::Value::String(value));
        }
        if let Some(value) = api_key.clone() {
            values.push(("API_KEY", value.clone()));
            dict.insert("BDSpeechAPIKey".into(), plist::Value::String(value));
        }
        if let Some(value) = secret_key.clone() {
            values.push(("SECRET_KEY", value.clone()));
            dict.insert("BDSpeechSecretKey".into(), plist::Value::String(value));
        }
        set_plist_dictionary_values(dict, "baiduspeech", &values);
    }
    if let Some(umeng) = manifest_provider(manifest, "umeng", Some("statics")) {
        if let Some(appkey) = json_string_field(umeng, &["appkey_ios", "appkey"]) {
            set_plist_dictionary_values(dict, "umeng", &[("appkey", appkey.clone())]);
            dict.insert("UMENG_APPKEY".into(), plist::Value::String(appkey));
        }
        if let Some(channel) = json_string_field(umeng, &["channelid_ios", "channelid"]) {
            dict.insert("UMENG_CHANNEL".into(), plist::Value::String(channel));
        }
    }
}

fn set_plist_dictionary_values(dict: &mut plist::Dictionary, key: &str, values: &[(&str, String)]) {
    if values.is_empty() {
        return;
    }
    let mut nested = match dict.remove(key) {
        Some(plist::Value::Dictionary(value)) => value,
        _ => plist::Dictionary::new(),
    };
    for (name, value) in values {
        nested.insert((*name).to_string(), plist::Value::String(value.clone()));
    }
    dict.insert(key.to_string(), plist::Value::Dictionary(nested));
}

fn merge_plist_string_array(dict: &mut plist::Dictionary, key: &str, values: Vec<String>) {
    let mut merged = Vec::new();
    if let Some(existing) = dict.get(key) {
        collect_plist_strings(existing, &mut merged);
    }
    merged.extend(values);
    let merged = dedup_non_empty_strings(merged);
    if !merged.is_empty() {
        dict.insert(
            key.to_string(),
            plist::Value::Array(merged.into_iter().map(plist::Value::String).collect()),
        );
    }
}

fn collect_plist_strings(value: &plist::Value, output: &mut Vec<String>) {
    match value {
        plist::Value::String(value) => {
            output.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(String::from),
            );
        }
        plist::Value::Array(values) => {
            for value in values {
                collect_plist_strings(value, output);
            }
        }
        _ => {}
    }
}

fn ios_manifest_url_schemes(manifest: &serde_json::Value) -> Vec<String> {
    let mut schemes = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("urltypes"))
    {
        collect_json_strings(value, &mut schemes);
    }
    if let Some(value) = provider_value(manifest, "weixin", &["appid"]) {
        schemes.push(value);
    }
    if let Some(value) = provider_value(manifest, "qq", &["appid"]) {
        schemes.push(prefixed_scheme("tencent", &value));
    }
    if let Some(value) = provider_value(manifest, "sina", &["appkey"]) {
        schemes.push(prefixed_scheme("wb", &value));
    }
    if let Some(value) = provider_value(manifest, "facebook", &["appid"]) {
        schemes.push(prefixed_scheme("fb", &value));
    }
    dedup_non_empty_strings(schemes)
}

fn ios_manifest_query_schemes(manifest: &serde_json::Value) -> Vec<String> {
    let mut schemes = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("urlschemewhitelist"))
    {
        collect_json_strings(value, &mut schemes);
    }
    if provider_value(manifest, "weixin", &["appid"]).is_some() {
        schemes.extend(["weixin".into(), "weixinULAPI".into()]);
    }
    if provider_value(manifest, "qq", &["appid"]).is_some() {
        schemes.extend(
            [
                "mqq",
                "mqqapi",
                "mqqOpensdkSSoLogin",
                "mqqopensdkapiV2",
                "mqqopensdkapiV3",
                "mqqwpa",
                "mqzone",
            ]
            .into_iter()
            .map(String::from),
        );
    }
    if provider_value(manifest, "sina", &["appkey"]).is_some() {
        schemes.extend(
            ["sinaweibo", "sinaweibohd", "weibosdk", "weibosdk2.5"]
                .into_iter()
                .map(String::from),
        );
    }
    if provider_value(manifest, "facebook", &["appid"]).is_some() {
        schemes.extend(
            ["fb", "fbapi", "fb-messenger-share-api", "fbauth2"]
                .into_iter()
                .map(String::from),
        );
    }
    dedup_non_empty_strings(schemes)
}

fn ios_manifest_background_modes(manifest: &serde_json::Value) -> Vec<String> {
    let mut modes = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("UIBackgroundModes"))
    {
        collect_json_strings(value, &mut modes);
    }
    dedup_non_empty_strings(modes)
}

fn provider_value(manifest: &serde_json::Value, provider: &str, keys: &[&str]) -> Option<String> {
    [
        "oauth",
        "share",
        "payment",
        "geolocation",
        "maps",
        "speech",
        "statics",
    ]
    .into_iter()
    .find_map(|category| {
        manifest_provider(manifest, provider, Some(category))
            .and_then(|value| json_string_field(value, keys))
    })
}

fn manifest_provider<'a>(
    manifest: &'a serde_json::Value,
    provider: &str,
    category: Option<&str>,
) -> Option<&'a serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?;
    match category {
        Some(category) => sdk_configs.get(category)?.get(provider),
        None => sdk_configs.get(provider),
    }
}

fn json_string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(String::from)
    })
}

fn collect_json_strings(value: &serde_json::Value, output: &mut Vec<String>) {
    match value {
        serde_json::Value::String(value) => {
            output.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(String::from),
            );
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_json_strings(value, output);
            }
        }
        _ => {}
    }
}

fn prefixed_scheme(prefix: &str, value: &str) -> String {
    if value.starts_with(prefix) {
        value.to_string()
    } else {
        format!("{}{}", prefix, value)
    }
}

fn dedup_non_empty_strings(values: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() && !result.iter().any(|existing| existing == value) {
            result.push(value.to_string());
        }
    }
    result
}

fn universal_links(manifest: &serde_json::Value) -> Vec<String> {
    let mut links = Vec::new();
    collect_values_for_key(manifest, "UniversalLinks", &mut links);
    dedup_non_empty_strings(links)
}

fn ios_manifest_associated_domains(manifest: &serde_json::Value) -> Vec<String> {
    let mut domains = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("capabilities"))
        .and_then(|value| value.get("entitlements"))
        .and_then(|value| value.get("com.apple.developer.associated-domains"))
    {
        collect_json_strings(value, &mut domains);
    }
    dedup_non_empty_strings(
        domains
            .into_iter()
            .filter_map(|domain| normalize_associated_domain(&domain))
            .collect(),
    )
}

fn normalize_associated_domain(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(domain) = universal_link_domain(value) {
        return Some(format!("applinks:{}", domain));
    }
    if value.contains(':') {
        Some(value.to_string())
    } else {
        Some(format!("applinks:{}", value))
    }
}

fn collect_values_for_key(value: &serde_json::Value, key: &str, output: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(values) => {
            for (name, value) in values {
                if name.eq_ignore_ascii_case(key) {
                    collect_json_strings(value, output);
                } else {
                    collect_values_for_key(value, key, output);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_values_for_key(value, key, output);
            }
        }
        _ => {}
    }
}

fn patch_ios_entitlements(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<usize, String> {
    let Some(manifest) = manifest_info.and_then(|info| info.manifest_value.as_ref()) else {
        return Ok(0);
    };
    let mut domains = ios_manifest_associated_domains(manifest);
    domains.extend(
        universal_links(manifest)
            .into_iter()
            .filter_map(|link| universal_link_domain(&link))
            .map(|domain| format!("applinks:{}", domain)),
    );
    let domains = dedup_non_empty_strings(domains);
    if domains.is_empty() {
        return Ok(0);
    }
    let entitlements = find_entitlements(project_root, project_file).ok_or_else(|| {
        "manifest 配置了 UniversalLinks，但 iOS 工程中未找到 entitlements 文件".to_string()
    })?;
    let mut value = plist::Value::from_file(&entitlements).map_err(|e| {
        format!(
            "解析 iOS entitlements 失败 {}: {}",
            entitlements.display(),
            e
        )
    })?;
    let dict = value.as_dictionary_mut().ok_or_else(|| {
        format!(
            "iOS entitlements 不是 dictionary: {}",
            entitlements.display()
        )
    })?;
    dict.insert(
        "com.apple.developer.associated-domains".into(),
        plist::Value::Array(domains.iter().cloned().map(plist::Value::String).collect()),
    );
    value.to_file_xml(&entitlements).map_err(|e| {
        format!(
            "写入 iOS entitlements 失败 {}: {}",
            entitlements.display(),
            e
        )
    })?;
    Ok(domains.len())
}

fn universal_link_domain(link: &str) -> Option<String> {
    let value = link
        .trim()
        .strip_prefix("https://")
        .or_else(|| link.trim().strip_prefix("http://"))?;
    value
        .split('/')
        .next()
        .map(|domain| domain.split(':').next().unwrap_or(domain).trim())
        .filter(|domain| !domain.is_empty())
        .map(String::from)
}

fn find_entitlements(project_root: &Path, project_file: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(project_file.join("project.pbxproj")).ok()?;
    let pattern = regex::Regex::new(r#"CODE_SIGN_ENTITLEMENTS = "?([^";]+)"?;"#).ok()?;
    for capture in pattern.captures_iter(&content) {
        let relative = capture.get(1)?.as_str().trim_matches('"');
        let candidate = project_root.join(relative);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    find_file_with_ext_skipping_bundles(project_root, "entitlements")
}

fn apply_ios_splashscreen(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<usize>, String> {
    let Some(splashscreen) = manifest_info.and_then(|info| info.splashscreen.as_ref()) else {
        return Ok(None);
    };
    if splashscreen.ios_style.as_deref() != Some("storyboard") {
        return Ok(None);
    }
    let zip_path = splashscreen
        .ios_storyboard
        .as_deref()
        .map(PathBuf::from)
        .ok_or_else(|| {
            "manifest 已配置 iOS storyboard 启动界面，但未配置 app-plus.distribute.splashscreen.ios.storyboard"
                .to_string()
        })?;
    if !zip_path.is_file() {
        return Err(format!(
            "manifest 配置的 iOS storyboard zip 不存在: {}",
            zip_path.display()
        ));
    }

    let launch_screen = find_launch_screen_storyboard(project_root, project_file)
        .ok_or_else(|| "SDK 自带 iOS 工程中未找到 LaunchScreen.storyboard".to_string())?;
    let target_dir = launch_screen
        .parent()
        .ok_or_else(|| format!("启动界面目录异常: {}", launch_screen.display()))?;
    let file = std::fs::File::open(&zip_path)
        .map_err(|e| format!("打开 iOS storyboard zip 失败 {}: {}", zip_path.display(), e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("解析 iOS storyboard zip 失败: {}", e))?;
    let mut storyboard_candidates = Vec::new();
    let mut resource_names = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("读取 iOS storyboard zip 条目失败: {}", e))?;
        if entry.is_dir() {
            continue;
        }
        let Some(enclosed) = entry.enclosed_name() else {
            return Err(format!(
                "iOS storyboard zip 包含不安全路径: {}",
                entry.name()
            ));
        };
        if enclosed
            .components()
            .any(|component| component.as_os_str() == "__MACOSX")
        {
            continue;
        }
        let Some(file_name) = enclosed.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with('.') || file_name.contains("/*") || file_name.contains("*/") {
            continue;
        }
        if enclosed.extension().and_then(|ext| ext.to_str()) == Some("storyboard") {
            let mut content = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut content)
                .map_err(|e| format!("读取 iOS storyboard 失败: {}", e))?;
            let priority = if file_name.eq_ignore_ascii_case("LaunchScreen.storyboard") {
                0
            } else {
                1
            };
            storyboard_candidates.push((priority, file_name.to_string(), content));
            continue;
        }

        let output = target_dir.join(file_name);
        let mut output_file = std::fs::File::create(&output)
            .map_err(|e| format!("创建启动界面资源失败 {}: {}", output.display(), e))?;
        std::io::copy(&mut entry, &mut output_file)
            .map_err(|e| format!("复制启动界面资源失败 {}: {}", output.display(), e))?;
        resource_names.push(file_name.to_string());
    }

    storyboard_candidates.sort_by_key(|candidate| candidate.0);
    let (_, source_name, storyboard) =
        storyboard_candidates.into_iter().next().ok_or_else(|| {
            format!(
                "iOS storyboard zip 中未找到 .storyboard 文件: {}",
                zip_path.display()
            )
        })?;
    std::fs::write(&launch_screen, storyboard).map_err(|e| {
        format!(
            "写入 manifest 启动界面 {} -> {} 失败: {}",
            source_name,
            launch_screen.display(),
            e
        )
    })?;

    resource_names.sort();
    resource_names.dedup();
    register_pbx_resources(project_file, &resource_names)?;
    Ok(Some(resource_names.len()))
}

fn find_launch_screen_storyboard(project_root: &Path, project_file: &Path) -> Option<PathBuf> {
    if let Some(info_plist) = find_info_plist(project_root, project_file) {
        let candidate = info_plist.parent()?.join("LaunchScreen.storyboard");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    find_file_named_skipping_bundles(project_root, "LaunchScreen.storyboard")
}

fn register_pbx_resources(project_file: &Path, resource_names: &[String]) -> Result<(), String> {
    if resource_names.is_empty() {
        return Ok(());
    }
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    for name in resource_names {
        if content.contains(&format!("/* {} in Resources */", name)) {
            continue;
        }
        let file_ref = pbx_object_id();
        let build_ref = pbx_object_id();
        let file_type = pbx_resource_file_type(name);
        let path = render_pbx_value(name);
        let build_line = format!(
            "\t\t{} /* {} in Resources */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; }};\n",
            build_ref, name, file_ref, name
        );
        let file_line = format!(
            "\t\t{} /* {} */ = {{isa = PBXFileReference; lastKnownFileType = {}; path = {}; sourceTree = \"<group>\"; }};\n",
            file_ref, name, file_type, path
        );
        content = insert_after_marker(
            &content,
            "/* Begin PBXBuildFile section */\n",
            &build_line,
            "PBXBuildFile section",
        )?;
        content = insert_after_marker(
            &content,
            "/* Begin PBXFileReference section */\n",
            &file_line,
            "PBXFileReference section",
        )?;
        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Supporting Files \*/ = \{\s*isa = PBXGroup;\s*children = \(\n)",
            &format!("\t\t\t\t{} /* {} */,\n", file_ref, name),
            "Supporting Files group",
        )?;
        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Resources \*/ = \{\s*isa = PBXResourcesBuildPhase;.*?files = \(\n)",
            &format!("\t\t\t\t{} /* {} in Resources */,\n", build_ref, name),
            "PBXResourcesBuildPhase",
        )?;
    }
    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))
}

fn insert_after_marker(
    content: &str,
    marker: &str,
    value: &str,
    description: &str,
) -> Result<String, String> {
    let index = content
        .find(marker)
        .ok_or_else(|| format!("project.pbxproj 缺少 {}", description))?
        + marker.len();
    let mut result = String::with_capacity(content.len() + value.len());
    result.push_str(&content[..index]);
    result.push_str(value);
    result.push_str(&content[index..]);
    Ok(result)
}

fn insert_into_pbx_list(
    content: &str,
    pattern: &str,
    value: &str,
    description: &str,
) -> Result<String, String> {
    let regex = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
    let matched = regex
        .find(content)
        .ok_or_else(|| format!("project.pbxproj 缺少 {}", description))?;
    let mut result = String::with_capacity(content.len() + value.len());
    result.push_str(&content[..matched.end()]);
    result.push_str(value);
    result.push_str(&content[matched.end()..]);
    Ok(result)
}

fn pbx_object_id() -> String {
    uuid::Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(24)
        .collect::<String>()
        .to_uppercase()
}

fn pbx_resource_file_type(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image.png",
        Some("jpg" | "jpeg") => "image.jpeg",
        Some("pdf") => "image.pdf",
        Some("json") => "text.json",
        _ => "file",
    }
}

fn set_dcloud_default_theme(dict: &mut plist::Dictionary) {
    let existing = dict.remove("DCloudConfig");
    let mut dcloud = match existing {
        Some(plist::Value::Dictionary(value)) => value,
        _ => plist::Dictionary::new(),
    };
    dcloud.insert("defaultTheme".into(), plist::Value::String("auto".into()));
    dict.insert("DCloudConfig".into(), plist::Value::Dictionary(dcloud));
}

fn import_app_resource(apps_dir: &Path, resource_dir: &Path, app_id: &str) -> Result<(), String> {
    if apps_dir.exists() {
        std::fs::remove_dir_all(apps_dir)
            .map_err(|e| format!("清理旧 Pandora/apps 失败 {}: {}", apps_dir.display(), e))?;
    }
    crate::utils::fs::ensure_directory(apps_dir).map_err(|e| e.to_string())?;
    crate::utils::fs::copy_recursive(resource_dir, &apps_dir.join(app_id))
        .map_err(|e| format!("复制 UniApp iOS 资源失败: {}", e))
}

fn patch_control_xml(control_xml: &Path, app_id: &str) -> Result<(), String> {
    if !control_xml.exists() {
        return Err(format!("未找到 control.xml: {}", control_xml.display()));
    }
    let content = std::fs::read_to_string(control_xml)
        .map_err(|e| format!("读取 control.xml 失败: {}", e))?;
    let updated = crate::utils::xml::set_xml_attribute(&content, "/apps/app", "appid", app_id)
        .or_else(|_| crate::utils::xml::set_xml_attribute(&content, "/hbuilder", "appid", app_id))
        .map_err(|e| format!("写入 control.xml appid 失败: {}", e))?;
    std::fs::write(control_xml, updated).map_err(|e| format!("写入 control.xml 失败: {}", e))
}

fn resolve_ios_runtime_layout(project_root: &Path) -> Result<IosRuntimeLayout, String> {
    let mut controls = Vec::new();
    collect_files_named_skipping_bundles(project_root, "control.xml", &mut controls);
    controls.sort();

    for control_xml in &controls {
        let Some(parent) = control_xml.parent() else {
            continue;
        };
        let pandora = parent.join("Pandora");
        if pandora.is_dir() {
            return Ok(IosRuntimeLayout {
                control_xml: control_xml.clone(),
                apps_dir: pandora.join("apps"),
            });
        }
        if parent.file_name().and_then(|name| name.to_str()) == Some("Pandora") {
            return Ok(IosRuntimeLayout {
                control_xml: control_xml.clone(),
                apps_dir: parent.join("apps"),
            });
        }
    }

    let checked = if controls.is_empty() {
        project_root.display().to_string()
    } else {
        controls
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    Err(format!(
        "未找到有效的 iOS control.xml 与 Pandora/apps 布局。已检查: {}",
        checked
    ))
}

fn verify_privacy_manifest(workspace: &Path, project_file: &Path) -> Result<(), String> {
    let sdk_privacy = workspace.join("SDK/PrivacyInfo.xcprivacy");
    let privacy = sdk_privacy
        .is_file()
        .then_some(sdk_privacy)
        .or_else(|| find_file_named_skipping_bundles(workspace, "PrivacyInfo.xcprivacy"))
        .or_else(|| find_file_with_ext_skipping_bundles(workspace, "xcprivacy"))
        .ok_or_else(|| {
            "iOS SDK 工程缺少 .xcprivacy 隐私清单，请确认使用 HBuilderX 5.0+ 对应的 iOS 离线 SDK"
                .to_string()
        })?;
    let name = privacy
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("隐私清单文件名异常: {}", privacy.display()))?;
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    if !content.contains(name) {
        return Err(format!(
            "隐私清单 {} 未纳入 Xcode 工程，请检查 iOS SDK 自带 HBuilder-Hello",
            name
        ));
    }
    Ok(())
}

fn emit_ios_log(
    window: &tauri::Window,
    build_id: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    let event = crate::commands::android::BuildLogEvent {
        build_id: Some(build_id.to_string()),
        platform: "ios".to_string(),
        level: level.to_string(),
        message: message.to_string(),
        progress,
    };
    let _ = window.emit("build-log", event);
}

fn emit_version_warning_if_needed(
    window: &tauri::Window,
    build_id: &str,
    scan: &crate::commands::shared::resource_scan::ResourceScanResult,
    sdk_project: &Path,
) {
    let Some(resource_version) = scan.hbuilderx_version.as_deref() else {
        return;
    };
    let Some(sdk_version) = detect_version_from_path(sdk_project) else {
        emit_ios_log(
            window,
            build_id,
            "warn",
            "无法从 iOS SDK 路径识别版本，请确认与 HBuilderX 导出资源版本一致",
            Some(12),
        );
        return;
    };
    if sdk_version != resource_version {
        emit_ios_log(
            window,
            build_id,
            "warn",
            &format!(
                "资源 HBuilderX 版本 ({}) 与 iOS SDK 路径版本 ({}) 不一致，请确认 SDK 选择正确",
                resource_version, sdk_version
            ),
            Some(12),
        );
    }
}

fn detect_version_from_path(path: &Path) -> Option<String> {
    let re = regex::Regex::new(r"\d+\.\d+(?:\.\d+)?").ok()?;
    path.ancestors().find_map(|ancestor| {
        ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| re.find(name).map(|m| m.as_str().to_string()))
    })
}

fn clean_copied_project(project_root: &Path) -> Result<(), String> {
    for path in [
        project_root.join("build"),
        project_root.join("DerivedData"),
        project_root.join(".build"),
    ] {
        if path.exists() {
            std::fs::remove_dir_all(&path)
                .map_err(|e| format!("清理 iOS 工程旧构建产物失败 {}: {}", path.display(), e))?;
        }
    }
    Ok(())
}

fn link_ios_sdk_support(sdk_project: &Path, workspace: &Path) -> Result<Option<PathBuf>, String> {
    let Some(sdk_root) = sdk_project.parent() else {
        return Ok(None);
    };
    let support_source = sdk_root.join("SDK");
    if !support_source.is_dir() {
        return Ok(None);
    }

    let support_dest = workspace.join("SDK");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&support_source, &support_dest).map_err(|e| {
        format!(
            "关联 iOS SDK 支持目录失败 {} -> {}: {}",
            support_dest.display(),
            support_source.display(),
            e
        )
    })?;
    #[cfg(not(unix))]
    crate::utils::fs::copy_recursive(&support_source, &support_dest).map_err(|e| {
        format!(
            "复制 iOS SDK 支持目录失败 {} -> {}: {}",
            support_source.display(),
            support_dest.display(),
            e
        )
    })?;

    Ok(Some(support_dest))
}

fn find_xcodeproj(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("xcodeproj"))
}

fn find_scheme_name(project_file: &Path) -> Option<String> {
    let content = std::fs::read_to_string(project_file.join("project.pbxproj")).ok()?;
    let pattern =
        regex::Regex::new(r#"(?s)isa = PBXNativeTarget;.*?\n\s*name = "?([^";]+)"?;"#).ok()?;
    pattern
        .captures(&content)
        .and_then(|captures| captures.get(1))
        .map(|name| name.as_str().trim().to_string())
        .filter(|name| !name.is_empty())
}

fn find_info_plist(project_root: &Path, project_file: &Path) -> Option<PathBuf> {
    let pbxproj = project_file.join("project.pbxproj");
    if let Ok(content) = std::fs::read_to_string(pbxproj) {
        let re = regex::Regex::new(r#"INFOPLIST_FILE = "?([^";]+)"?;"#).ok()?;
        for cap in re.captures_iter(&content) {
            let rel = cap
                .get(1)?
                .as_str()
                .replace("$(SRCROOT)/", "")
                .replace("${SRCROOT}/", "");
            let candidate = project_root.join(rel.trim_matches('"'));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    find_file_named_skipping_bundles(project_root, "Info.plist")
}

fn find_file_named(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_named(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

fn find_file_named_skipping_bundles(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_xcode_package_dir(&path) {
                continue;
            }
            if let Some(found) = find_file_named_skipping_bundles(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

fn collect_files_named_skipping_bundles(dir: &Path, name: &str, output: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if !is_xcode_package_dir(&path) {
                collect_files_named_skipping_bundles(&path, name, output);
            }
        } else if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            output.push(path);
        }
    }
}

fn find_file_with_ext_skipping_bundles(dir: &Path, ext: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_xcode_package_dir(&path) {
                continue;
            }
            if let Some(found) = find_file_with_ext_skipping_bundles(&path, ext) {
                return Some(found);
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some(ext) {
            return Some(path);
        }
    }
    None
}

fn find_file_with_ext(dir: &Path, ext: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_with_ext(&path, ext) {
                return Some(found);
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some(ext) {
            return Some(path);
        }
    }
    None
}

fn is_xcode_package_dir(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("framework" | "xcframework" | "bundle" | "xcodeproj" | "xcworkspace")
    )
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    PathBuf::from(path)
}

fn safe_file_name(value: &str) -> String {
    let cleaned = value.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    if cleaned.trim().is_empty() {
        "ios-build".to_string()
    } else {
        cleaned
    }
}

fn ensure_macos(action: &str) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        Ok(())
    } else {
        Err(format!("{} 仅支持 macOS", action))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_build_reloads_manifest_from_configured_local_project() {
        let root = std::env::temp_dir().join(format!(
            "unipack-ios-local-manifest-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(root.join("unpackage/res/icons")).unwrap();
        std::fs::write(root.join("unpackage/res/icons/1024.png"), "icon").unwrap();
        std::fs::write(
            root.join("manifest.json"),
            r#"{
                "name": "Manifest App",
                "appid": "__UNI__MANIFEST",
                "versionName": "2.3.4",
                "versionCode": "234",
                "app-plus": {
                    "distribute": {
                        "ios": {
                            "privacyDescription": {
                                "NSCameraUsageDescription": "用于扫码"
                            }
                        },
                        "icons": {
                            "ios": {
                                "appstore": "unpackage/res/icons/1024.png"
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();
        let mut config = crate::commands::project::ProjectConfig::default();
        config.local_path = root.to_string_lossy().to_string();
        config.app.name = "Config App".into();
        config.app.version = "1.0.0".into();
        config.app.version_code = 1;

        let info = resolve_ios_manifest_info(&config, None).unwrap().unwrap();

        assert_eq!(effective_app_name(&config, Some(&info)), "Manifest App");
        assert_eq!(effective_app_version(&config, Some(&info)), "2.3.4");
        assert_eq!(effective_app_version_code(&config, Some(&info)), 234);
        assert_eq!(
            info.ios_privacy_descriptions
                .get("NSCameraUsageDescription")
                .map(String::as_str),
            Some("用于扫码")
        );
        let expected_icon = root
            .join("unpackage/res/icons/1024.png")
            .to_string_lossy()
            .to_string();
        assert_eq!(
            info.ios_icons
                .as_ref()
                .and_then(|icons| icons.ios.get("appstore"))
                .map(String::as_str),
            Some(expected_icon.as_str())
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn ios_build_rejects_manifest_and_resource_app_id_mismatch() {
        let manifest = serde_json::json!({ "appid": "__UNI__MANIFEST" });
        let root = std::env::temp_dir().join(format!("unipack-ios-appid-{}", uuid::Uuid::new_v4()));
        let info = crate::commands::shared::resource::parse_uniapp_manifest(
            &manifest,
            &root.join("manifest.json"),
            &root,
            None,
        );

        let error = validate_ios_app_id("__UNI__RESOURCE", Some(&info)).unwrap_err();

        assert!(error.contains("__UNI__MANIFEST"));
        assert!(error.contains("__UNI__RESOURCE"));
    }

    #[test]
    fn ios_manifest_basic_info_and_privacy_are_written_to_xcode_project() {
        let root = std::env::temp_dir().join(format!(
            "unipack-ios-manifest-apply-{}",
            uuid::Uuid::new_v4()
        ));
        let project_file = root.join("HBuilder-Hello.xcodeproj");
        let plist_path = root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
        std::fs::create_dir_all(&project_file).unwrap();
        std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
        let localized_plist = root.join("HBuilder-Hello/en.lproj/InfoPlist.strings");
        std::fs::create_dir_all(localized_plist.parent().unwrap()).unwrap();
        std::fs::write(
            &localized_plist,
            "/* Localized */\nCFBundleDisplayName=\"HBuilder Hello\";\n",
        )
        .unwrap();
        plist::Value::Dictionary(plist::Dictionary::new())
            .to_file_xml(&plist_path)
            .unwrap();
        std::fs::write(
            project_file.join("project.pbxproj"),
            r#"buildSettings = {
				CURRENT_PROJECT_VERSION = 1;
				DEVELOPMENT_TEAM = OLDTEAM;
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
				MARKETING_VERSION = 1.0.0;
				PRODUCT_BUNDLE_IDENTIFIER = io.dcloud.HBuilder;
			};
"#,
        )
        .unwrap();
        let manifest = serde_json::json!({
            "name": "Manifest App",
            "versionName": "3.4.5",
            "versionCode": 345,
            "app-plus": {
                "distribute": {
                    "ios": {
                        "urltypes": ["manifest-app"],
                        "urlschemewhitelist": "alipays,alipay,wechat",
                        "UIBackgroundModes": "audio",
                        "privacyDescription": {
                            "NSPhotoLibraryUsageDescription": "用于选择图片"
                        },
                        "capabilities": {
                            "entitlements": {
                                "com.apple.developer.associated-domains": [
                                    "applinks:www.hubeijianmeishiye.cn"
                                ]
                            }
                        }
                    },
                    "sdkConfigs": {
                        "oauth": {
                            "google": {
                                "clientid": "google-client-id"
                            }
                        },
                        "share": {
                            "weixin": {
                                "appid": "wx-manifest",
                                "UniversalLinks": "https://example.com/app/"
                            }
                        },
                        "geolocation": {
                            "amap": {
                                "appkey_ios": "amap-ios-key"
                            }
                        },
                        "statics": {
                            "umeng": {
                                "appkey_ios": "umeng-ios-key",
                                "channelid_ios": "App Store"
                            }
                        }
                    }
                }
            }
        });
        let info = crate::commands::shared::resource::parse_uniapp_manifest(
            &manifest,
            &root.join("manifest.json"),
            &root,
            None,
        );
        let mut config = crate::commands::project::ProjectConfig::default();
        config.app.name = "Config App".into();
        config.app.version = "1.0.0".into();
        config.app.version_code = 1;
        config.ios.bundle_id = "com.example.manifest".into();
        config.ios.team_id = "TEAM123".into();
        config.ios.dcloud_app_key = "app-key".into();

        patch_pbxproj(&project_file, &config, Some(&info)).unwrap();
        patch_info_plist(
            &root,
            &project_file,
            &config,
            "__UNI__MANIFEST",
            Some(&info),
        )
        .unwrap();

        let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
        assert!(pbxproj.contains("MARKETING_VERSION = 3.4.5;"));
        assert!(pbxproj.contains("CURRENT_PROJECT_VERSION = 345;"));
        assert!(pbxproj.contains("INFOPLIST_KEY_CFBundleDisplayName = \"Manifest App\";"));
        let plist = plist::Value::from_file(&plist_path).unwrap();
        let dict = plist.as_dictionary().unwrap();
        assert_eq!(
            dict.get("CFBundleDisplayName")
                .and_then(plist::Value::as_string),
            Some("Manifest App")
        );
        assert_eq!(
            dict.get("CFBundleShortVersionString")
                .and_then(plist::Value::as_string),
            Some("3.4.5")
        );
        assert_eq!(
            dict.get("CFBundleVersion")
                .and_then(plist::Value::as_string),
            Some("345")
        );
        assert_eq!(
            dict.get("NSPhotoLibraryUsageDescription")
                .and_then(plist::Value::as_string),
            Some("用于选择图片")
        );
        assert_eq!(
            dict.get("GIDClientID").and_then(plist::Value::as_string),
            Some("google-client-id")
        );
        assert_eq!(
            dict.get("AMapApiKey").and_then(plist::Value::as_string),
            Some("amap-ios-key")
        );
        assert_eq!(
            dict.get("UMENG_APPKEY").and_then(plist::Value::as_string),
            Some("umeng-ios-key")
        );
        let url_schemes = dict
            .get("CFBundleURLTypes")
            .and_then(plist::Value::as_array)
            .unwrap()
            .iter()
            .filter_map(plist::Value::as_dictionary)
            .filter_map(|entry| entry.get("CFBundleURLSchemes"))
            .filter_map(plist::Value::as_array)
            .flatten()
            .filter_map(plist::Value::as_string)
            .collect::<Vec<_>>();
        assert!(url_schemes.contains(&"manifest-app"));
        assert!(url_schemes.contains(&"wx-manifest"));
        let query_schemes = dict
            .get("LSApplicationQueriesSchemes")
            .and_then(plist::Value::as_array)
            .unwrap()
            .iter()
            .filter_map(plist::Value::as_string)
            .collect::<Vec<_>>();
        assert!(query_schemes.contains(&"alipays"));
        assert!(query_schemes.contains(&"alipay"));
        assert!(query_schemes.contains(&"wechat"));
        let background_modes = dict
            .get("UIBackgroundModes")
            .and_then(plist::Value::as_array)
            .unwrap()
            .iter()
            .filter_map(plist::Value::as_string)
            .collect::<Vec<_>>();
        assert_eq!(background_modes, vec!["audio"]);
        assert!(std::fs::read_to_string(&localized_plist)
            .unwrap()
            .contains("CFBundleDisplayName=\"Manifest App\";"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn info_plist_strings_value_is_replaced_and_escaped() {
        let content = "/* Localized */\nCFBundleDisplayName=\"HBuilder Hello\";\nOther=\"keep\";\n";

        let updated =
            set_info_plist_string_value(content, "CFBundleDisplayName", "My \"$App\" \\ Name");

        assert!(updated.contains("CFBundleDisplayName=\"My \\\"$App\\\" \\\\ Name\";"));
        assert!(updated.contains("Other=\"keep\";"));
    }

    #[test]
    fn info_plist_strings_supports_quoted_keys_and_missing_keys() {
        let content =
            "/* Localized */\n\"CFBundleDisplayName\" = \"HBuilder Hello\";\nOther=\"keep\";\n";

        let updated = set_info_plist_string_value(content, "CFBundleDisplayName", "Manifest App");
        let appended = set_info_plist_string_value("Other=\"keep\";", "CFBundleDisplayName", "App");

        assert!(updated.contains("\"CFBundleDisplayName\" = \"Manifest App\";"));
        assert!(updated.contains("Other=\"keep\";"));
        assert!(appended.contains("Other=\"keep\";\n\"CFBundleDisplayName\" = \"App\";"));
    }

    #[test]
    fn ios_privacy_descriptions_overlay_manifest_values_on_sdk_defaults() {
        let mut dict = plist::Dictionary::new();
        dict.insert(
            "NSCameraUsageDescription".into(),
            plist::Value::String("模板相机说明".into()),
        );
        dict.insert(
            "NSMicrophoneUsageDescription".into(),
            plist::Value::String("模板麦克风说明".into()),
        );
        dict.insert(
            "NSLocationWhenInUseUsageDescription".into(),
            plist::Value::String("".into()),
        );
        dict.insert(
            "NSLocationWhenInUseUsageDescription - 2".into(),
            plist::Value::String("用户使用时期定位".into()),
        );
        dict.insert(
            "NSLocationWhenInUseDescription".into(),
            plist::Value::String("".into()),
        );
        let descriptions = std::collections::BTreeMap::from([
            (
                "NSPhotoLibraryUsageDescription".to_string(),
                "在上传头像或发布内容时，开启相册权限便于您保存图片或选择图片上传".to_string(),
            ),
            (
                "NSPhotoLibraryAddUsageDescription".to_string(),
                "该应用需要读取您的相册，以便您使用应用生成海报时保存到相册".to_string(),
            ),
            (
                "NSCameraUsageDescription".to_string(),
                "在上传头像或发布内容时，开启相机权限便于您拍照上传图片".to_string(),
            ),
            (
                "NSLocalNetworkUsageDescription".to_string(),
                "请允许访问本地网络，以便更好的体验应用".to_string(),
            ),
            (
                "NSContactsUsageDescription".to_string(),
                "用于拨通客服热线".to_string(),
            ),
        ]);

        apply_ios_privacy_descriptions(&mut dict, &descriptions);

        assert_eq!(
            dict.get("NSCameraUsageDescription")
                .and_then(plist::Value::as_string),
            Some("在上传头像或发布内容时，开启相机权限便于您拍照上传图片")
        );
        assert_eq!(
            dict.get("NSPhotoLibraryAddUsageDescription")
                .and_then(plist::Value::as_string),
            Some("该应用需要读取您的相册，以便您使用应用生成海报时保存到相册")
        );
        assert_eq!(
            dict.get("NSMicrophoneUsageDescription")
                .and_then(plist::Value::as_string),
            Some("模板麦克风说明")
        );
        assert_eq!(
            dict.get("NSLocationWhenInUseUsageDescription")
                .and_then(plist::Value::as_string),
            Some("用户使用时期定位")
        );
        assert_eq!(
            dict.get("NSLocalNetworkUsageDescription")
                .and_then(plist::Value::as_string),
            Some("请允许访问本地网络，以便更好的体验应用")
        );
        assert_eq!(
            dict.get("NSContactsUsageDescription")
                .and_then(plist::Value::as_string),
            Some("用于拨通客服热线")
        );
        assert!(!dict.contains_key("NSLocationWhenInUseUsageDescription - 2"));
        assert!(!dict.contains_key("NSLocationWhenInUseDescription"));
    }

    #[test]
    fn ios_manifest_universal_links_are_written_to_entitlements() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-entitlements-{}", uuid::Uuid::new_v4()));
        let project_file = root.join("HBuilder-Hello.xcodeproj");
        let entitlements = root.join("HBuilder/HBuilder.entitlements");
        std::fs::create_dir_all(&project_file).unwrap();
        std::fs::create_dir_all(entitlements.parent().unwrap()).unwrap();
        plist::Value::Dictionary(plist::Dictionary::new())
            .to_file_xml(&entitlements)
            .unwrap();
        std::fs::write(
            project_file.join("project.pbxproj"),
            "CODE_SIGN_ENTITLEMENTS = HBuilder/HBuilder.entitlements;",
        )
        .unwrap();
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "ios": {
                        "capabilities": {
                            "entitlements": {
                                "com.apple.developer.associated-domains": [
                                    "applinks:www.hubeijianmeishiye.cn"
                                ]
                            }
                        }
                    },
                    "sdkConfigs": {
                        "share": {
                            "weixin": {
                                "UniversalLinks": "https://example.com/apple-app-site-association/"
                            }
                        },
                        "payment": {
                            "weixin": {
                                "UniversalLinks": "https://example.com/pay/"
                            }
                        }
                    }
                }
            }
        });
        let info = crate::commands::shared::resource::parse_uniapp_manifest(
            &manifest,
            &root.join("manifest.json"),
            &root,
            None,
        );

        assert_eq!(
            patch_ios_entitlements(&root, &project_file, Some(&info)).unwrap(),
            2
        );
        let value = plist::Value::from_file(&entitlements).unwrap();
        let domains = value
            .as_dictionary()
            .unwrap()
            .get("com.apple.developer.associated-domains")
            .and_then(plist::Value::as_array)
            .unwrap();
        assert!(domains.contains(&plist::Value::String(
            "applinks:www.hubeijianmeishiye.cn".into()
        )));
        assert!(domains.contains(&plist::Value::String("applinks:example.com".into())));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn ios_storyboard_zip_replaces_launch_screen_and_registers_resources() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-storyboard-{}", uuid::Uuid::new_v4()));
        let project_file = root.join("HBuilder-Hello.xcodeproj");
        let source_dir = root.join("HBuilder-Hello");
        let launch_screen = source_dir.join("LaunchScreen.storyboard");
        let zip_path = root.join("storyboard.zip");
        std::fs::create_dir_all(&project_file).unwrap();
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(&launch_screen, "old storyboard").unwrap();
        std::fs::write(source_dir.join("HBuilder-Hello-Info.plist"), "<plist/>").unwrap();
        std::fs::write(
            project_file.join("project.pbxproj"),
            r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
		AAA /* Supporting Files */ = {
			isa = PBXGroup;
			children = (
			);
		};
		BBB /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			files = (
			);
		};
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
"#,
        )
        .unwrap();
        let zip_file = std::fs::File::create(&zip_path).unwrap();
        let mut writer = zip::ZipWriter::new(zip_file);
        let options = zip::write::SimpleFileOptions::default();
        writer
            .start_file("LaunchScreen.storyboard", options)
            .unwrap();
        std::io::Write::write_all(&mut writer, b"new storyboard").unwrap();
        writer
            .start_file("images/background@2x.png", options)
            .unwrap();
        std::io::Write::write_all(&mut writer, b"image").unwrap();
        writer.finish().unwrap();
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "splashscreen": {
                        "iosStyle": "storyboard",
                        "ios": {
                            "storyboard": zip_path.to_string_lossy()
                        }
                    }
                }
            }
        });
        let info = crate::commands::shared::resource::parse_uniapp_manifest(
            &manifest,
            &root.join("manifest.json"),
            &root,
            None,
        );

        assert_eq!(
            apply_ios_splashscreen(&root, &project_file, Some(&info)).unwrap(),
            Some(1)
        );
        assert_eq!(
            std::fs::read_to_string(&launch_screen).unwrap(),
            "new storyboard"
        );
        assert!(source_dir.join("background@2x.png").is_file());
        let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
        assert!(pbxproj.contains("background@2x.png in Resources"));
        assert!(pbxproj.contains("lastKnownFileType = image.png"));

        apply_ios_splashscreen(&root, &project_file, Some(&info)).unwrap();
        let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
        assert_eq!(pbxproj.matches("background@2x.png in Resources").count(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn configured_ios_sdk_project_accepts_local_manifest_when_requested() {
        let Ok(sdk_project) = std::env::var("UNIPACK_TEST_IOS_SDK_PROJECT") else {
            return;
        };
        let Ok(local_project) = std::env::var("UNIPACK_TEST_UNIAPP_PROJECT") else {
            return;
        };
        let root =
            std::env::temp_dir().join(format!("unipack-ios-real-config-{}", uuid::Uuid::new_v4()));
        let project_root = root.join("HBuilder-Hello");
        crate::utils::fs::copy_recursive(Path::new(&sdk_project), &project_root).unwrap();
        let project_file = find_xcodeproj(&project_root).unwrap();
        let mut config = crate::commands::project::ProjectConfig::default();
        config.local_path = local_project;
        config.ios.bundle_id = "com.example.unipack.verify".into();
        config.ios.team_id = "TEAM123".into();
        config.ios.dcloud_app_key = "verify-app-key".into();
        let info = resolve_ios_manifest_info(&config, None).unwrap().unwrap();

        patch_pbxproj(&project_file, &config, Some(&info)).unwrap();
        apply_ios_splashscreen(&project_root, &project_file, Some(&info)).unwrap();
        patch_info_plist(
            &project_root,
            &project_file,
            &config,
            info.app_id.as_deref().unwrap_or("__UNI__VERIFY"),
            Some(&info),
        )
        .unwrap();
        assert_eq!(
            patch_ios_entitlements(&project_root, &project_file, Some(&info)).unwrap(),
            1
        );
        generate_app_icons(&project_root, &config, Some(&info)).unwrap();

        let output = std::process::Command::new("xcodebuild")
            .args(["-list", "-project"])
            .arg(&project_file)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
        assert!(pbxproj.contains("dc_launchscreen_portrait_background@2x.png in Resources"));
        assert!(project_root
            .join("HBuilder-Hello/dc_launchscreen_portrait_background@2x.png")
            .is_file());
        let plist =
            plist::Value::from_file(project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist"))
                .unwrap();
        let plist = plist.as_dictionary().unwrap();
        assert_eq!(
            plist
                .get("CFBundleDisplayName")
                .and_then(plist::Value::as_string),
            Some("ccc222")
        );
        assert_eq!(
            plist.get("AMapApiKey").and_then(plist::Value::as_string),
            Some("e58f1b2f4c1e3d8a9b7c6d5e4f3a2b1c")
        );
        let entitlements =
            plist::Value::from_file(project_root.join("HBuilder/HBuilder.entitlements")).unwrap();
        assert!(entitlements
            .as_dictionary()
            .unwrap()
            .contains_key("com.apple.developer.associated-domains"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn pbx_setting_replaces_existing_value() {
        let content = "\t\t\t\tPRODUCT_BUNDLE_IDENTIFIER = io.dcloud.HBuilder;\n";
        let updated =
            set_pbx_build_setting(content, "PRODUCT_BUNDLE_IDENTIFIER", "com.example.app");
        assert!(updated.contains("PRODUCT_BUNDLE_IDENTIFIER = com.example.app;"));
    }

    #[test]
    fn pbx_setting_inserts_into_build_settings_block() {
        let content = "buildSettings = {\n\tOTHER = value;\n};\n";
        let updated = set_pbx_build_setting(content, "DEVELOPMENT_TEAM", "TEAM123");
        assert!(updated.contains("DEVELOPMENT_TEAM = TEAM123;"));
    }

    #[test]
    fn pbx_linker_flag_preserves_existing_flags_and_is_idempotent() {
        let content = "\t\t\t\tOTHER_LDFLAGS = \"-ObjC\";\n";
        let updated = append_pbx_build_setting_flag(content, "OTHER_LDFLAGS", "-ld_classic");
        let updated_again = append_pbx_build_setting_flag(&updated, "OTHER_LDFLAGS", "-ld_classic");

        assert!(updated.contains("OTHER_LDFLAGS = \"-ObjC -ld_classic\";"));
        assert_eq!(updated, updated_again);
    }

    #[test]
    fn pbx_conditional_simulator_arch_is_quoted() {
        let content = "buildSettings = {\n\tOTHER = value;\n};\n";
        let updated = set_pbx_build_setting(content, "\"ARCHS[sdk=iphonesimulator*]\"", "x86_64");

        assert!(updated.contains("\"ARCHS[sdk=iphonesimulator*]\" = x86_64;"));
    }

    #[test]
    fn legacy_framework_requires_x86_64_simulator_compatibility() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-legacy-sim-{}", uuid::Uuid::new_v4()));
        let project = root.join("HBuilder-Hello/HBuilder-Hello.xcodeproj");
        let framework = root.join("SDK/Libs/DCUniRecord.framework");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&framework).unwrap();
        std::fs::write(framework.join("DCUniRecord"), "legacy").unwrap();

        assert!(legacy_simulator_x86_64_required(&project));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn scheme_name_uses_native_target_instead_of_project_file_name() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-scheme-{}", uuid::Uuid::new_v4()));
        let project = root.join("HBuilder-Hello.xcodeproj");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(
            project.join("project.pbxproj"),
            r#"
				isa = PBXNativeTarget;
				buildConfigurationList = ABC;
				name = HBuilder;
				productName = "HBuilder-Hello";
			};
"#,
        )
        .unwrap();

        assert_eq!(find_scheme_name(&project).as_deref(), Some("HBuilder"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_layout_supports_nested_hbuilder_source_directory() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-layout-{}", uuid::Uuid::new_v4()));
        let source = root.join("HBuilder-Hello");
        let resource = root.join("resource");
        std::fs::create_dir_all(source.join("Pandora/apps")).unwrap();
        std::fs::create_dir_all(&resource).unwrap();
        std::fs::write(
            source.join("control.xml"),
            r#"<HBuilder><apps><app appid="__UNI__OLD"/></apps></HBuilder>"#,
        )
        .unwrap();
        std::fs::write(resource.join("manifest.json"), "{}").unwrap();

        let layout = resolve_ios_runtime_layout(&root).unwrap();
        import_app_resource(&layout.apps_dir, &resource, "__UNI__NEW").unwrap();
        patch_control_xml(&layout.control_xml, "__UNI__NEW").unwrap();

        assert_eq!(layout.control_xml, source.join("control.xml"));
        assert_eq!(layout.apps_dir, source.join("Pandora/apps"));
        assert!(source
            .join("Pandora/apps/__UNI__NEW/manifest.json")
            .is_file());
        assert!(!root.join("Pandora").exists());
        assert!(std::fs::read_to_string(source.join("control.xml"))
            .unwrap()
            .contains(r#"appid="__UNI__NEW""#));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_layout_supports_control_inside_pandora() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-layout-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("Pandora/apps")).unwrap();
        std::fs::write(root.join("Pandora/control.xml"), "<HBuilder />").unwrap();

        let layout = resolve_ios_runtime_layout(&root).unwrap();

        assert_eq!(layout.control_xml, root.join("Pandora/control.xml"));
        assert_eq!(layout.apps_dir, root.join("Pandora/apps"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_links_sibling_sdk_support_directory() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-support-{}", uuid::Uuid::new_v4()));
        let sdk_project = root.join("package/HBuilder-Hello");
        let support = root.join("package/SDK");
        let workspace = root.join("workspace");
        std::fs::create_dir_all(&sdk_project).unwrap();
        std::fs::create_dir_all(&support).unwrap();
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(support.join("PrivacyInfo.xcprivacy"), "privacy").unwrap();

        let linked = link_ios_sdk_support(&sdk_project, &workspace)
            .unwrap()
            .unwrap();

        assert_eq!(linked, workspace.join("SDK"));
        assert!(linked.join("PrivacyInfo.xcprivacy").is_file());
        assert!(std::fs::symlink_metadata(&linked)
            .unwrap()
            .file_type()
            .is_symlink());
        std::fs::remove_dir_all(&workspace).unwrap();
        assert!(support.join("PrivacyInfo.xcprivacy").is_file());
        let _ = std::fs::remove_dir_all(root);
    }
}
