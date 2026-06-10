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
    workspace: PathBuf,
    project_root: PathBuf,
    project_file: PathBuf,
    scheme: String,
    profile: MobileProvisionInfo,
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
    let dest = output_dir.join(format!(
        "{}-v{}.ipa",
        timestamp, workspace.config.app.version
    ));
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
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    window: &tauri::Window,
) -> Result<IosWorkspace, String> {
    let config = crate::commands::project::load_project_config_sync(project_id)?;
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    validate_ios_config(&config, &sdk_config)?;

    let resource_dir = PathBuf::from(resource_path);
    let scan = crate::commands::shared::resource_scan::scan_imported_resource(
        &resource_dir,
        &resource_dir,
        false,
    )?;
    let app_resource_dir = PathBuf::from(&scan.app_resource_path);
    emit_ios_log(
        window,
        build_id,
        "info",
        &format!("识别 iOS AppId: {}", scan.app_id),
        Some(10),
    );

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
    emit_ios_log(
        window,
        build_id,
        "success",
        "已复制 SDK 自带 HBuilder-Hello 到 workspace",
        Some(22),
    );

    let project_file = find_xcodeproj(&project_root)
        .ok_or_else(|| "复制后的 HBuilder-Hello 中未找到 .xcodeproj".to_string())?;
    patch_pbxproj(&project_file, &config)?;
    patch_info_plist(
        &project_root,
        &project_file,
        &config,
        &scan.app_id,
        manifest_info,
    )?;
    import_app_resource(&project_root, &app_resource_dir, &scan.app_id)?;
    patch_control_xml(&project_root, &scan.app_id)?;
    generate_app_icons(&project_root, &config, manifest_info)?;
    verify_privacy_manifest(&project_root, &project_file)?;
    let profile = install_mobileprovision(&config)?;
    import_p12_certificate(&config)?;
    emit_ios_log(window, build_id, "success", "iOS 工程配置完成", Some(55));

    let scheme = project_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("HBuilder-Hello")
        .to_string();
    Ok(IosWorkspace {
        config,
        workspace,
        project_root,
        project_file,
        scheme,
        profile,
    })
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
) -> Result<(), String> {
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let content = content.replace("io.dcloud.HBuilder", &config.ios.bundle_id);
    let content =
        set_pbx_build_setting(&content, "PRODUCT_BUNDLE_IDENTIFIER", &config.ios.bundle_id);
    let content = set_pbx_build_setting(&content, "DEVELOPMENT_TEAM", &config.ios.team_id);
    let content = set_pbx_build_setting(&content, "MARKETING_VERSION", &config.app.version);
    let content = set_pbx_build_setting(
        &content,
        "CURRENT_PROJECT_VERSION",
        &config.app.version_code.to_string(),
    );
    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))
}

fn set_pbx_build_setting(content: &str, key: &str, value: &str) -> String {
    let rendered = render_pbx_value(value);
    let pattern = regex::Regex::new(&format!(r"(?m)^(\s*{}\s*=\s*)[^;]*;", regex::escape(key)))
        .expect("valid pbx setting regex");
    if pattern.is_match(content) {
        return pattern
            .replace_all(content, format!("${{1}}{};", rendered))
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
        plist::Value::String(config.app.name.clone()),
    );
    dict.insert(
        "CFBundleShortVersionString".into(),
        plist::Value::String(config.app.version.clone()),
    );
    dict.insert(
        "CFBundleVersion".into(),
        plist::Value::String(config.app.version_code.to_string()),
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
        for (key, description) in &info.ios_privacy_descriptions {
            dict.insert(key.clone(), plist::Value::String(description.clone()));
        }
    }
    value
        .to_file_xml(&plist_path)
        .map_err(|e| format!("写入 Info.plist 失败: {}", e))
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

fn import_app_resource(
    project_root: &Path,
    resource_dir: &Path,
    app_id: &str,
) -> Result<(), String> {
    let apps_dir = project_root.join("Pandora/apps");
    if apps_dir.exists() {
        std::fs::remove_dir_all(&apps_dir)
            .map_err(|e| format!("清理旧 Pandora/apps 失败 {}: {}", apps_dir.display(), e))?;
    }
    crate::utils::fs::ensure_directory(&apps_dir).map_err(|e| e.to_string())?;
    crate::utils::fs::copy_recursive(resource_dir, &apps_dir.join(app_id))
        .map_err(|e| format!("复制 UniApp iOS 资源失败: {}", e))
}

fn patch_control_xml(project_root: &Path, app_id: &str) -> Result<(), String> {
    let control = project_root.join("Pandora/control.xml");
    if !control.exists() {
        return Err(format!("未找到 control.xml: {}", control.display()));
    }
    let content =
        std::fs::read_to_string(&control).map_err(|e| format!("读取 control.xml 失败: {}", e))?;
    let updated = crate::utils::xml::set_xml_attribute(&content, "/apps/app", "appid", app_id)
        .or_else(|_| crate::utils::xml::set_xml_attribute(&content, "/hbuilder", "appid", app_id))
        .map_err(|e| format!("写入 control.xml appid 失败: {}", e))?;
    std::fs::write(&control, updated).map_err(|e| format!("写入 control.xml 失败: {}", e))
}

fn verify_privacy_manifest(project_root: &Path, project_file: &Path) -> Result<(), String> {
    let privacy = find_file_with_ext(project_root, "xcprivacy").ok_or_else(|| {
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

fn find_xcodeproj(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("xcodeproj"))
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
}
