//! iOS IPA 构建模块

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosBuildOptions {
    pub project_path: String,
    pub scheme: Option<String>,
    pub configuration: Option<String>,
    pub clean: Option<bool>,
    pub destination: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub logs: Vec<String>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

fn emit_ios_log(
    window: &tauri::Window,
    build_id: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    let event = crate::commands::build_android::BuildLogEvent {
        build_id: Some(build_id.to_string()),
        platform: "ios".to_string(),
        level: level.to_string(),
        message: message.to_string(),
        progress,
    };
    let _ = window.emit("build-log", event);
}

#[derive(Debug, Clone)]
struct IosBuildEnvironment {
    xcodebuild_bin: PathBuf,
    developer_dir: PathBuf,
}

#[tauri::command]
pub async fn prepare_ios_build(options: IosBuildOptions) -> Result<BuildResult, String> {
    let env = resolve_ios_build_environment()?;
    let project_dir = std::path::Path::new(&options.project_path);
    if !project_dir.exists() {
        return Err(format!(
            "Project path does not exist: {}",
            options.project_path
        ));
    }

    fn has_xcode_project(dir: &std::path::Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".xcodeproj") || name_str.ends_with(".xcworkspace") {
                    return true;
                }
            }
        }
        false
    }

    let has_xcodeproj = has_xcode_project(project_dir);
    let has_podfile = project_dir.join("Podfile").exists();

    Ok(BuildResult {
        success: true,
        output_path: None,
        logs: vec![
            "[prepare] iOS build environment checking...".to_string(),
            format!("[prepare] xcodebuild: {}", env.xcodebuild_bin.display()),
            format!("[prepare] DEVELOPER_DIR: {}", env.developer_dir.display()),
            format!("[prepare] Xcode project: {}", has_xcodeproj),
            format!("[prepare] CocoaPods: {}", has_podfile),
            format!(
                "[prepare] Scheme: {}",
                options.scheme.as_deref().unwrap_or("default")
            ),
        ],
        duration_ms: 0,
        error: None,
    })
}

#[tauri::command]
pub async fn run_ios_build(
    options: IosBuildOptions,
    app_handle: tauri::AppHandle,
) -> Result<BuildResult, String> {
    let start = std::time::Instant::now();

    let scheme = options.scheme.unwrap_or_else(|| "default".to_string());
    let configuration = options.configuration.unwrap_or_else(|| "Debug".to_string());

    let project_dir = std::path::Path::new(&options.project_path);
    let ios_env = resolve_ios_build_environment()?;

    let workspace_or_project = if project_dir.join("*.xcworkspace").exists() {
        format!("*.xcworkspace -scheme {}", scheme)
    } else if project_dir.join("*.xcodeproj").exists() {
        format!("*.xcodeproj -scheme {}", scheme)
    } else {
        return Err("No .xcodeproj or .xcworkspace found".to_string());
    };

    let mut args = vec![
        "build".to_string(),
        workspace_or_project,
        format!("-configuration {}", configuration),
    ];

    if options.clean.unwrap_or(false) {
        args.push("clean".to_string());
    }

    if let Some(dest) = options.destination {
        args.push(format!("-destination \"{}\"", dest));
    }

    args.push("-allowProvisioningUpdates".to_string());
    args.push("-verbose".to_string());

    let output = crate::utils::process::run_command_streaming_with_env(
        &ios_env.xcodebuild_bin.to_string_lossy(),
        &args,
        &project_dir.to_string_lossy(),
        &ios_process_env(&ios_env),
        app_handle,
        "ios-build",
    )
    .await
    .map_err(|e| e.to_string())?;

    let elapsed = start.elapsed().as_millis() as u64;

    Ok(BuildResult {
        success: output.success,
        output_path: None,
        logs: output.logs,
        duration_ms: elapsed,
        error: if output.success {
            None
        } else {
            Some("iOS build failed".to_string())
        },
    })
}

#[tauri::command]
pub async fn build_ios_ipa(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    window: tauri::Window,
) -> Result<crate::commands::build_android::BuildArtifact, String> {
    if !cfg!(target_os = "macos") {
        return Err("iOS 打包仅支持 macOS".to_string());
    }

    let build_id = build_id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("ios-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    emit_ios_log(&window, &build_id, "info", "开始 iOS IPA 构建流程", Some(2));
    let config = crate::commands::project::load_project_config_sync(&project_id)?;
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    validate_ios_config(&config, &sdk_config)?;
    let ios_env = resolve_ios_build_environment()?;

    let resource_dir = PathBuf::from(&resource_path);
    let scan = crate::commands::shared::resource_scan::scan_imported_resource(
        &resource_dir,
        &resource_dir,
        false,
    )?;
    let app_resource_dir = PathBuf::from(&scan.app_resource_path);
    let workspace = crate::utils::fs::get_project_config_dir(&project_id)
        .join("workspace")
        .join(safe_file_name(&build_id));
    crate::utils::fs::ensure_directory(&workspace).map_err(|e| e.to_string())?;

    let sdk_project = crate::commands::sdk::resolve_ios_sdk_project(&PathBuf::from(
        &sdk_config.dcloud_ios_sdk_path,
    ))?;
    if !sdk_project.exists() {
        return Err(format!("未找到 iOS SDK 工程: {}", sdk_project.display()));
    }
    let project_root = workspace.join(
        sdk_project
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("HBuilder-Hello"),
    );
    crate::utils::fs::copy_recursive(&sdk_project, &project_root)
        .map_err(|e| format!("复制 HBuilder-Hello 失败: {}", e))?;
    emit_ios_log(
        &window,
        &build_id,
        "success",
        "HBuilder-Hello 已复制到工作区",
        Some(12),
    );

    patch_pbxproj(&project_root, &config.ios.bundle_id)?;
    patch_info_plist(&project_root, &config)?;
    import_ios_resource(&project_root, &app_resource_dir, &scan.app_id)?;
    patch_ios_control(&project_root, &scan.app_id)?;
    copy_uts_ios_frameworks(&project_root, &scan)?;
    generate_ios_icons(&project_root, &config, manifest_info.as_ref())?;
    install_mobileprovision(&config.ios.provisioning_profile)?;
    import_p12_certificate(&config)?;
    emit_ios_log(&window, &build_id, "success", "iOS 工程配置已完成", Some(45));

    let archive_path = workspace.join("build/output.xcarchive");
    let project_file = find_xcodeproj(&project_root)
        .ok_or_else(|| "HBuilder-Hello 中未找到 .xcodeproj".to_string())?;
    let scheme = project_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("HBuilder-Hello")
        .to_string();

    let archive_args = vec![
        "-project".to_string(),
        project_file.to_string_lossy().to_string(),
        "-scheme".to_string(),
        scheme,
        "-configuration".to_string(),
        "Release".to_string(),
        "-archivePath".to_string(),
        archive_path.to_string_lossy().to_string(),
        "archive".to_string(),
        format!("DEVELOPMENT_TEAM={}", config.ios.team_id),
        format!("PRODUCT_BUNDLE_IDENTIFIER={}", config.ios.bundle_id),
        "CODE_SIGN_STYLE=Manual".to_string(),
    ];
    emit_ios_log(&window, &build_id, "info", "执行 xcodebuild archive", Some(55));
    run_xcodebuild(&archive_args, &project_root, &window, &ios_env, &build_id).await?;

    let export_options = workspace.join("ExportOptions.plist");
    write_export_options(&export_options, &config)?;
    let export_path = workspace.join("build/export");
    let export_args = vec![
        "-exportArchive".to_string(),
        "-archivePath".to_string(),
        archive_path.to_string_lossy().to_string(),
        "-exportPath".to_string(),
        export_path.to_string_lossy().to_string(),
        "-exportOptionsPlist".to_string(),
        export_options.to_string_lossy().to_string(),
    ];
    emit_ios_log(&window, &build_id, "info", "执行 xcodebuild exportArchive", Some(80));
    run_xcodebuild(&export_args, &project_root, &window, &ios_env, &build_id).await?;

    let ipa = find_file_with_ext(&export_path, "ipa")
        .ok_or_else(|| "导出成功后未找到 IPA 文件".to_string())?;
    let output_dir = expand_home(&config.output_dir);
    crate::utils::fs::ensure_directory(&output_dir).map_err(|e| e.to_string())?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let dest = output_dir.join(format!("{}-v{}.ipa", ts, config.app.version));
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

    Ok(crate::commands::build_android::BuildArtifact {
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

fn validate_ios_config(
    config: &crate::commands::project::ProjectConfig,
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<(), String> {
    if sdk_config.dcloud_ios_sdk_path.trim().is_empty()
        || config.ios.bundle_id.trim().is_empty()
        || config.ios.team_id.trim().is_empty()
        || config.ios.provisioning_profile.trim().is_empty()
    {
        return Err(
            "请先在 SDK & 环境管理中配置 DCloud iOS 离线 SDK，并配置 Bundle ID、Team ID 和描述文件"
                .to_string(),
        );
    }
    if !config.ios.certificate.trim().is_empty() && !config.ios.has_certificate_password {
        return Err("导入 iOS P12 证书需要先保存证书密码".to_string());
    }
    if config.ios.dcloud_app_key.trim().is_empty() {
        return Err("请先配置 iOS DCloud AppKey".to_string());
    }
    Ok(())
}

fn find_xcodeproj(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().map(|e| e == "xcodeproj").unwrap_or(false))
}

fn patch_pbxproj(project_root: &Path, bundle_id: &str) -> Result<(), String> {
    let Some(xcodeproj) = find_xcodeproj(project_root) else {
        return Ok(());
    };
    let pbxproj = xcodeproj.join("project.pbxproj");
    if !pbxproj.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&pbxproj).map_err(|e| e.to_string())?;
    let updated = content.replace("io.dcloud.HBuilder", bundle_id).replace(
        "PRODUCT_BUNDLE_IDENTIFIER = HBuilder;",
        &format!("PRODUCT_BUNDLE_IDENTIFIER = {};", bundle_id),
    );
    std::fs::write(&pbxproj, updated).map_err(|e| e.to_string())
}

fn patch_pbxproj_framework_references(
    project_root: &Path,
    frameworks: &[String],
) -> Result<(), String> {
    if frameworks.is_empty() {
        return Ok(());
    }
    let Some(xcodeproj) = find_xcodeproj(project_root) else {
        return Ok(());
    };
    let pbxproj = xcodeproj.join("project.pbxproj");
    if !pbxproj.exists() {
        return Ok(());
    }
    let mut content = std::fs::read_to_string(&pbxproj).map_err(|e| e.to_string())?;
    let mut build_files = String::new();
    let mut file_refs = String::new();
    let mut framework_phase_entries = String::new();
    let mut embed_phase_entries = String::new();
    for framework in frameworks {
        let Some(name) = Path::new(framework).file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if content.contains(name) || file_refs.contains(name) {
            continue;
        }
        let file_id = xcode_id(&format!("unipack-file-{}", name));
        let framework_build_id = xcode_id(&format!("unipack-framework-{}", name));
        let embed_build_id = xcode_id(&format!("unipack-embed-{}", name));
        let file_type = if name.ends_with(".xcframework") {
            "wrapper.xcframework"
        } else {
            "wrapper.framework"
        };
        build_files.push_str(&format!(
            "\t\t{} /* {} in Frameworks */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; }};\n",
            framework_build_id, name, file_id, name
        ));
        build_files.push_str(&format!(
            "\t\t{} /* {} in Embed Frameworks */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; settings = {{ATTRIBUTES = (CodeSignOnCopy, RemoveHeadersOnCopy, ); }}; }};\n",
            embed_build_id, name, file_id, name
        ));
        file_refs.push_str(&format!(
            "\t\t{} /* {} */ = {{isa = PBXFileReference; lastKnownFileType = {}; name = {}; path = UTSFrameworks/{}; sourceTree = \"<group>\"; }};\n",
            file_id, name, file_type, name, name
        ));
        framework_phase_entries.push_str(&format!(
            "\t\t\t\t{} /* {} in Frameworks */,\n",
            framework_build_id, name
        ));
        embed_phase_entries.push_str(&format!(
            "\t\t\t\t{} /* {} in Embed Frameworks */,\n",
            embed_build_id, name
        ));
    }
    if build_files.is_empty() {
        return Ok(());
    }
    content = insert_into_pbx_section(&content, "PBXBuildFile", &build_files)?;
    content = insert_into_pbx_section(&content, "PBXFileReference", &file_refs)?;
    content = insert_into_first_phase_files(
        &content,
        "PBXFrameworksBuildPhase",
        &framework_phase_entries,
    )?;
    content = insert_embed_framework_entries(&content, &embed_phase_entries)?;
    std::fs::write(&pbxproj, content).map_err(|e| e.to_string())
}

fn insert_into_pbx_section(
    content: &str,
    section: &str,
    insertion: &str,
) -> Result<String, String> {
    let end_marker = format!("/* End {} section */", section);
    let Some(pos) = content.find(&end_marker) else {
        return Err(format!("project.pbxproj 缺少 {} section", section));
    };
    let mut updated = String::with_capacity(content.len() + insertion.len());
    updated.push_str(&content[..pos]);
    updated.push_str(insertion);
    updated.push_str(&content[pos..]);
    Ok(updated)
}

fn insert_into_first_phase_files(
    content: &str,
    phase_isa: &str,
    entries: &str,
) -> Result<String, String> {
    let Some(phase_pos) = content.find(&format!("isa = {};", phase_isa)) else {
        return Err(format!("project.pbxproj 缺少 {}", phase_isa));
    };
    let Some(files_rel) = content[phase_pos..].find("files = (") else {
        return Err(format!("{} 缺少 files 列表", phase_isa));
    };
    let files_pos = phase_pos + files_rel;
    let Some(insert_rel) = content[files_pos..].find('\n') else {
        return Err(format!("{} files 列表格式异常", phase_isa));
    };
    let insert_pos = files_pos + insert_rel + 1;
    let mut updated = String::with_capacity(content.len() + entries.len());
    updated.push_str(&content[..insert_pos]);
    updated.push_str(entries);
    updated.push_str(&content[insert_pos..]);
    Ok(updated)
}

fn insert_embed_framework_entries(content: &str, entries: &str) -> Result<String, String> {
    if let Some(copy_pos) = content.find("dstSubfolderSpec = 10;") {
        let Some(files_rel) = content[copy_pos..].find("files = (") else {
            return Err("Embed Frameworks build phase 缺少 files 列表".to_string());
        };
        let files_pos = copy_pos + files_rel;
        let Some(insert_rel) = content[files_pos..].find('\n') else {
            return Err("Embed Frameworks files 列表格式异常".to_string());
        };
        let insert_pos = files_pos + insert_rel + 1;
        let mut updated = String::with_capacity(content.len() + entries.len());
        updated.push_str(&content[..insert_pos]);
        updated.push_str(entries);
        updated.push_str(&content[insert_pos..]);
        return Ok(updated);
    }

    let phase_id = xcode_id("unipack-embed-frameworks-phase");
    let phase = format!(
        "\t\t{} /* Embed Frameworks */ = {{\n\t\t\tisa = PBXCopyFilesBuildPhase;\n\t\t\tbuildActionMask = 2147483647;\n\t\t\tdstPath = \"\";\n\t\t\tdstSubfolderSpec = 10;\n\t\t\tfiles = (\n{}\t\t\t);\n\t\t\tname = \"Embed Frameworks\";\n\t\t\trunOnlyForDeploymentPostprocessing = 0;\n\t\t}};\n",
        phase_id, entries
    );
    let content = if content.contains("/* End PBXCopyFilesBuildPhase section */") {
        insert_into_pbx_section(content, "PBXCopyFilesBuildPhase", &phase)?
    } else {
        let Some(anchor) = content.find("/* Begin PBXFrameworksBuildPhase section */") else {
            return Err("project.pbxproj 缺少可插入 PBXCopyFilesBuildPhase 的位置".to_string());
        };
        let section = format!(
            "/* Begin PBXCopyFilesBuildPhase section */\n{}/* End PBXCopyFilesBuildPhase section */\n\n",
            phase
        );
        let mut updated = String::with_capacity(content.len() + section.len());
        updated.push_str(&content[..anchor]);
        updated.push_str(&section);
        updated.push_str(&content[anchor..]);
        updated
    };
    insert_into_first_native_target_build_phases(
        &content,
        &format!("\t\t\t\t{} /* Embed Frameworks */,\n", phase_id),
    )
}

fn insert_into_first_native_target_build_phases(
    content: &str,
    entry: &str,
) -> Result<String, String> {
    let Some(target_pos) = content.find("isa = PBXNativeTarget;") else {
        return Err("project.pbxproj 缺少 PBXNativeTarget".to_string());
    };
    let Some(phases_rel) = content[target_pos..].find("buildPhases = (") else {
        return Err("PBXNativeTarget 缺少 buildPhases".to_string());
    };
    let phases_pos = target_pos + phases_rel;
    let Some(insert_rel) = content[phases_pos..].find('\n') else {
        return Err("buildPhases 列表格式异常".to_string());
    };
    let insert_pos = phases_pos + insert_rel + 1;
    let mut updated = String::with_capacity(content.len() + entry.len());
    updated.push_str(&content[..insert_pos]);
    updated.push_str(entry);
    updated.push_str(&content[insert_pos..]);
    Ok(updated)
}

fn xcode_id(seed: &str) -> String {
    let mut hash: u128 = 0xCBF29CE484222325;
    for byte in seed.as_bytes() {
        hash ^= *byte as u128;
        hash = hash.wrapping_mul(0x100000001B3);
    }
    format!("{:024X}", hash & 0xFFFFFFFFFFFFFFFFFFFFFFFF)
}

fn patch_info_plist(
    project_root: &Path,
    config: &crate::commands::project::ProjectConfig,
) -> Result<(), String> {
    let plist_path = find_file_named(project_root, "Info.plist")
        .ok_or_else(|| "未找到 Info.plist".to_string())?;
    let mut value = plist::Value::from_file(&plist_path).map_err(|e| e.to_string())?;
    let dict = value
        .as_dictionary_mut()
        .ok_or_else(|| "Info.plist 不是 dictionary".to_string())?;
    dict.insert(
        "dcloud_appkey".to_string(),
        plist::Value::String(config.ios.dcloud_app_key.clone()),
    );
    dict.insert(
        "CFBundleDisplayName".to_string(),
        plist::Value::String(config.app.name.clone()),
    );
    dict.insert(
        "CFBundleShortVersionString".to_string(),
        plist::Value::String(config.app.version.clone()),
    );
    dict.insert(
        "CFBundleVersion".to_string(),
        plist::Value::String(config.app.version_code.to_string()),
    );
    value.to_file_xml(&plist_path).map_err(|e| e.to_string())
}

fn import_ios_resource(
    project_root: &Path,
    resource_dir: &Path,
    app_id: &str,
) -> Result<(), String> {
    let dest = project_root.join("Pandora/apps").join(app_id);
    crate::utils::fs::copy_recursive(resource_dir, &dest)
        .map_err(|e| format!("复制 iOS 资源失败: {}", e))
}

fn patch_ios_control(project_root: &Path, app_id: &str) -> Result<(), String> {
    let control = project_root.join("Pandora/control.xml");
    if !control.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&control).map_err(|e| e.to_string())?;
    let updated = crate::utils::xml::set_xml_attribute(&content, "/apps/app", "appid", app_id)
        .or_else(|_| crate::utils::xml::set_xml_attribute(&content, "/hbuilder", "appid", app_id))
        .map_err(|e| e.to_string())?;
    std::fs::write(&control, updated).map_err(|e| e.to_string())
}

fn copy_uts_ios_frameworks(
    project_root: &Path,
    scan: &crate::commands::resource::ResourceScanResult,
) -> Result<(), String> {
    if !scan.uts.has_uts_plugins {
        return Ok(());
    }
    let dest = project_root.join("UTSFrameworks");
    crate::utils::fs::ensure_directory(&dest).map_err(|e| e.to_string())?;
    let mut copied_frameworks = Vec::new();
    for plugin in &scan.uts.custom_plugins {
        for fw in &plugin.ios_frameworks {
            let src = PathBuf::from(fw);
            let Some(name) = src.file_name() else {
                continue;
            };
            let target = dest.join(name);
            if src.is_dir() {
                crate::utils::fs::copy_recursive(&src, &target).map_err(|e| e.to_string())?;
            } else {
                crate::utils::fs::copy_file(&src, &target).map_err(|e| e.to_string())?;
            }
            copied_frameworks.push(target.to_string_lossy().to_string());
        }
    }
    copy_sdk_uts_frameworks(project_root, &dest, scan, &mut copied_frameworks)?;
    patch_pbxproj_framework_references(project_root, &copied_frameworks)?;
    Ok(())
}

fn copy_sdk_uts_frameworks(
    project_root: &Path,
    dest: &Path,
    scan: &crate::commands::resource::ResourceScanResult,
    copied_frameworks: &mut Vec<String>,
) -> Result<(), String> {
    let sdk_candidates = [
        project_root.join("SDK"),
        project_root.join("Libs"),
        project_root.join("libs"),
        project_root.to_path_buf(),
    ];
    let required = ["DCUniBase.framework", "DCloudUTSFoundation.framework"];
    for framework_name in required {
        if let Some(src) = find_file_or_dir_named_any(&sdk_candidates, framework_name) {
            let target = dest.join(framework_name);
            if src.is_dir() {
                crate::utils::fs::copy_recursive(&src, &target).map_err(|e| e.to_string())?;
            } else {
                crate::utils::fs::copy_file(&src, &target).map_err(|e| e.to_string())?;
            }
            copied_frameworks.push(target.to_string_lossy().to_string());
        }
    }
    if !scan.uts.builtin_modules.is_empty() {
        if let Some(src) = find_file_or_dir_named_any(&sdk_candidates, "DCloudUTSExtAPI.framework")
        {
            let target = dest.join("DCloudUTSExtAPI.framework");
            if src.is_dir() {
                crate::utils::fs::copy_recursive(&src, &target).map_err(|e| e.to_string())?;
            } else {
                crate::utils::fs::copy_file(&src, &target).map_err(|e| e.to_string())?;
            }
            copied_frameworks.push(target.to_string_lossy().to_string());
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct IosAppIconSlot {
    manifest_key: &'static str,
    filename: &'static str,
    idiom: &'static str,
    size: &'static str,
    scale: &'static str,
    pixels: u32,
}

const IOS_APP_ICON_SLOTS: &[IosAppIconSlot] = &[
    IosAppIconSlot {
        manifest_key: "iphone.notification@2x",
        filename: "Icon-iphone-20@2x.png",
        idiom: "iphone",
        size: "20x20",
        scale: "2x",
        pixels: 40,
    },
    IosAppIconSlot {
        manifest_key: "iphone.notification@3x",
        filename: "Icon-iphone-20@3x.png",
        idiom: "iphone",
        size: "20x20",
        scale: "3x",
        pixels: 60,
    },
    IosAppIconSlot {
        manifest_key: "iphone.settings@2x",
        filename: "Icon-iphone-29@2x.png",
        idiom: "iphone",
        size: "29x29",
        scale: "2x",
        pixels: 58,
    },
    IosAppIconSlot {
        manifest_key: "iphone.settings@3x",
        filename: "Icon-iphone-29@3x.png",
        idiom: "iphone",
        size: "29x29",
        scale: "3x",
        pixels: 87,
    },
    IosAppIconSlot {
        manifest_key: "iphone.spotlight@2x",
        filename: "Icon-iphone-40@2x.png",
        idiom: "iphone",
        size: "40x40",
        scale: "2x",
        pixels: 80,
    },
    IosAppIconSlot {
        manifest_key: "iphone.spotlight@3x",
        filename: "Icon-iphone-40@3x.png",
        idiom: "iphone",
        size: "40x40",
        scale: "3x",
        pixels: 120,
    },
    IosAppIconSlot {
        manifest_key: "iphone.app@2x",
        filename: "Icon-iphone-60@2x.png",
        idiom: "iphone",
        size: "60x60",
        scale: "2x",
        pixels: 120,
    },
    IosAppIconSlot {
        manifest_key: "iphone.app@3x",
        filename: "Icon-iphone-60@3x.png",
        idiom: "iphone",
        size: "60x60",
        scale: "3x",
        pixels: 180,
    },
    IosAppIconSlot {
        manifest_key: "ipad.notification",
        filename: "Icon-ipad-20.png",
        idiom: "ipad",
        size: "20x20",
        scale: "1x",
        pixels: 20,
    },
    IosAppIconSlot {
        manifest_key: "ipad.notification@2x",
        filename: "Icon-ipad-20@2x.png",
        idiom: "ipad",
        size: "20x20",
        scale: "2x",
        pixels: 40,
    },
    IosAppIconSlot {
        manifest_key: "ipad.settings",
        filename: "Icon-ipad-29.png",
        idiom: "ipad",
        size: "29x29",
        scale: "1x",
        pixels: 29,
    },
    IosAppIconSlot {
        manifest_key: "ipad.settings@2x",
        filename: "Icon-ipad-29@2x.png",
        idiom: "ipad",
        size: "29x29",
        scale: "2x",
        pixels: 58,
    },
    IosAppIconSlot {
        manifest_key: "ipad.spotlight",
        filename: "Icon-ipad-40.png",
        idiom: "ipad",
        size: "40x40",
        scale: "1x",
        pixels: 40,
    },
    IosAppIconSlot {
        manifest_key: "ipad.spotlight@2x",
        filename: "Icon-ipad-40@2x.png",
        idiom: "ipad",
        size: "40x40",
        scale: "2x",
        pixels: 80,
    },
    IosAppIconSlot {
        manifest_key: "ipad.app",
        filename: "Icon-ipad-76.png",
        idiom: "ipad",
        size: "76x76",
        scale: "1x",
        pixels: 76,
    },
    IosAppIconSlot {
        manifest_key: "ipad.app@2x",
        filename: "Icon-ipad-76@2x.png",
        idiom: "ipad",
        size: "76x76",
        scale: "2x",
        pixels: 152,
    },
    IosAppIconSlot {
        manifest_key: "ipad.proapp@2x",
        filename: "Icon-ipad-83.5@2x.png",
        idiom: "ipad",
        size: "83.5x83.5",
        scale: "2x",
        pixels: 167,
    },
    IosAppIconSlot {
        manifest_key: "appstore",
        filename: "Icon-1024.png",
        idiom: "ios-marketing",
        size: "1024x1024",
        scale: "1x",
        pixels: 1024,
    },
];

fn generate_ios_icons(
    project_root: &Path,
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let manifest_icons = manifest_info.and_then(|info| info.ios_icons.as_ref());
    let fallback_source = ios_fallback_icon_source(config, manifest_icons);
    if manifest_icons.is_none() && fallback_source.is_none() {
        return Ok(());
    }
    let appicon = find_dir_named(project_root, "AppIcon.appiconset")
        .unwrap_or_else(|| project_root.join("Assets.xcassets/AppIcon.appiconset"));
    crate::utils::fs::ensure_directory(&appicon).map_err(|e| e.to_string())?;

    let fallback_image = fallback_source
        .as_ref()
        .map(|source| image::open(source).map(|image| image.to_rgba8()))
        .transpose()
        .map_err(|e| e.to_string())?;

    for slot in IOS_APP_ICON_SLOTS {
        if let Some(source) = manifest_icons
            .and_then(|icons| icons.ios.get(slot.manifest_key))
            .map(PathBuf::from)
            .filter(|source| source.exists())
        {
            std::fs::copy(&source, appicon.join(slot.filename))
                .map_err(|e| format!("复制 iOS 图标失败 {}: {}", source.display(), e))?;
            continue;
        }

        let Some(img) = fallback_image.as_ref() else {
            continue;
        };
        let resized =
            image::imageops::resize(img, slot.pixels, slot.pixels, image::imageops::Lanczos3);
        resized
            .save(appicon.join(slot.filename))
            .map_err(|e| e.to_string())?;
    }
    write_appicon_contents(&appicon)?;
    Ok(())
}

fn ios_fallback_icon_source(
    config: &crate::commands::project::ProjectConfig,
    manifest_icons: Option<&crate::commands::resource::IosIconsConfig>,
) -> Option<PathBuf> {
    manifest_icons
        .and_then(|icons| icons.ios.get("appstore"))
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            let source = config.app.icon1024.trim();
            if source.is_empty() {
                return None;
            }
            let source = PathBuf::from(source);
            source.exists().then_some(source)
        })
}

fn write_appicon_contents(appicon: &Path) -> Result<(), String> {
    let images = IOS_APP_ICON_SLOTS
        .iter()
        .map(|slot| {
            serde_json::json!({
                "idiom": slot.idiom,
                "size": slot.size,
                "scale": slot.scale,
                "filename": slot.filename
            })
        })
        .collect::<Vec<_>>();
    let contents = serde_json::json!({
        "images": images,
        "info": { "author": "unipack-tool", "version": 1 }
    });
    let json = serde_json::to_string_pretty(&contents).map_err(|e| e.to_string())?;
    std::fs::write(appicon.join("Contents.json"), json).map_err(|e| e.to_string())
}

fn install_mobileprovision(profile: &str) -> Result<(), String> {
    let src = PathBuf::from(profile);
    if !src.exists() {
        return Err(format!("描述文件不存在: {}", profile));
    }
    let dest_dir = dirs::home_dir()
        .ok_or_else(|| "无法定位 HOME".to_string())?
        .join("Library/MobileDevice/Provisioning Profiles");
    crate::utils::fs::ensure_directory(&dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(src.file_name().unwrap_or_default());
    std::fs::copy(src, dest).map_err(|e| e.to_string())?;
    Ok(())
}

fn import_p12_certificate(config: &crate::commands::project::ProjectConfig) -> Result<(), String> {
    if config.ios.certificate.trim().is_empty() {
        return Ok(());
    }
    let cert = PathBuf::from(&config.ios.certificate);
    if !cert.exists() {
        return Err(format!("P12 证书不存在: {}", cert.display()));
    }
    let password_key = format!("{}-ios-certificate-password", config.id);
    let password = crate::utils::keychain::get_password(&password_key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Keychain 中缺少 iOS P12 证书密码".to_string())?;
    let output = std::process::Command::new("security")
        .arg("import")
        .arg(&cert)
        .arg("-P")
        .arg(password)
        .arg("-A")
        .output()
        .map_err(|e| format!("执行 security import 失败: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "导入 P12 证书失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn write_export_options(
    path: &Path,
    config: &crate::commands::project::ProjectConfig,
) -> Result<(), String> {
    let mut dict = plist::Dictionary::new();
    dict.insert(
        "method".to_string(),
        plist::Value::String(config.ios.export_method.clone()),
    );
    dict.insert(
        "teamID".to_string(),
        plist::Value::String(config.ios.team_id.clone()),
    );
    dict.insert(
        "signingStyle".to_string(),
        plist::Value::String("manual".to_string()),
    );
    plist::Value::Dictionary(dict)
        .to_file_xml(path)
        .map_err(|e| e.to_string())
}

async fn run_xcodebuild(
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

fn resolve_ios_build_environment() -> Result<IosBuildEnvironment, String> {
    let xcodebuild_bin =
        crate::commands::shared::env::resolve_configured_tool_bin("xcode", "xcodebuild")?;
    let developer_dir = xcodebuild_bin
        .parent()
        .and_then(|bin| bin.parent())
        .and_then(|usr| usr.parent())
        .and_then(|developer| {
            if developer.file_name().and_then(|n| n.to_str()) == Some("Developer") {
                Some(developer.to_path_buf())
            } else {
                None
            }
        })
        .or_else(|| {
            let configured =
                crate::commands::shared::env::require_configured_tool_path("xcode").ok()?;
            if configured.extension().and_then(|ext| ext.to_str()) == Some("app") {
                Some(configured.join("Contents").join("Developer"))
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
            "SDK & 环境管理中配置的 Xcode 无效，DEVELOPER_DIR 不存在: {}",
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
        "DEVELOPER_DIR".to_string(),
        env.developer_dir.to_string_lossy().to_string(),
    )]
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

fn find_dir_named(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(path);
            }
            if let Some(found) = find_dir_named(&path, name) {
                return Some(found);
            }
        }
    }
    None
}

fn find_file_or_dir_named_any(roots: &[PathBuf], name: &str) -> Option<PathBuf> {
    for root in roots {
        if root.exists() {
            if let Some(found) = find_file_or_dir_named(root, name) {
                return Some(found);
            }
        }
    }
    None
}

fn find_file_or_dir_named(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file_or_dir_named(&path, name) {
                return Some(found);
            }
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
        } else if path.extension().map(|e| e == ext).unwrap_or(false) {
            return Some(path);
        }
    }
    None
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
        "UniApp".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appicon_contents_includes_marketing_icon() {
        let dir = std::env::temp_dir().join(format!("unipack-appicon-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        write_appicon_contents(&dir).unwrap();

        let content = std::fs::read_to_string(dir.join("Contents.json")).unwrap();
        assert!(content.contains("ios-marketing"));
        assert!(content.contains("Icon-1024.png"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn ios_icons_use_manifest_appstore_as_fallback_source() {
        let project_root =
            std::env::temp_dir().join(format!("unipack-ios-icons-{}", uuid::Uuid::new_v4()));
        let source_dir = project_root.join("unpackage/res/icons");
        std::fs::create_dir_all(&source_dir).unwrap();
        let appstore_icon = source_dir.join("1024x1024.png");
        image::RgbaImage::from_pixel(4, 4, image::Rgba([10, 20, 30, 255]))
            .save(&appstore_icon)
            .unwrap();

        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "icons": {
                        "ios": {
                            "appstore": "unpackage/res/icons/1024x1024.png"
                        }
                    }
                }
            }
        });
        let info = crate::commands::shared::resource::parse_uniapp_manifest(
            &manifest,
            &project_root.join("manifest.json"),
            &project_root,
            None,
        );
        let config = crate::commands::project::ProjectConfig::default();

        generate_ios_icons(&project_root, &config, Some(&info)).unwrap();

        let appicon = project_root.join("Assets.xcassets/AppIcon.appiconset");
        assert!(appicon.join("Icon-1024.png").exists());
        assert!(appicon.join("Icon-iphone-60@3x.png").exists());
        let contents = std::fs::read_to_string(appicon.join("Contents.json")).unwrap();
        assert!(contents.contains("Icon-iphone-60@3x.png"));
        assert!(contents.contains("Icon-ipad-83.5@2x.png"));

        let _ = std::fs::remove_dir_all(project_root);
    }

    #[test]
    fn pbxproj_framework_patch_adds_framework_and_embed_phases() {
        let content = r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		111 /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXNativeTarget section */
		222 /* App */ = {
			isa = PBXNativeTarget;
			buildPhases = (
				111 /* Frameworks */,
			);
		};
/* End PBXNativeTarget section */
"#;
        let with_build_files = insert_into_pbx_section(
            content,
            "PBXBuildFile",
            "\t\tAAA /* DCUniBase.framework in Frameworks */ = {isa = PBXBuildFile; fileRef = BBB /* DCUniBase.framework */; };\n",
        )
        .unwrap();
        let with_file_refs = insert_into_pbx_section(
            &with_build_files,
            "PBXFileReference",
            "\t\tBBB /* DCUniBase.framework */ = {isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = DCUniBase.framework; path = UTSFrameworks/DCUniBase.framework; sourceTree = \"<group>\"; };\n",
        )
        .unwrap();
        let with_framework = insert_into_first_phase_files(
            &with_file_refs,
            "PBXFrameworksBuildPhase",
            "\t\t\t\tAAA /* DCUniBase.framework in Frameworks */,\n",
        )
        .unwrap();
        let patched = insert_embed_framework_entries(
            &with_framework,
            "\t\t\t\tCCC /* DCUniBase.framework in Embed Frameworks */,\n",
        )
        .unwrap();

        assert!(patched.contains("DCUniBase.framework in Frameworks"));
        assert!(patched.contains("PBXCopyFilesBuildPhase"));
        assert!(patched.contains("DCUniBase.framework in Embed Frameworks"));
        assert!(patched.contains("Embed Frameworks"));
    }
}
