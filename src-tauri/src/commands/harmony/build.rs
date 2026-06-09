//! 鸿蒙 HAP 构建模块

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonyBuildOptions {
    pub project_path: String,
    pub module: Option<String>,
    pub mode: Option<String>,
    pub clean: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub logs: Vec<String>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

fn emit_harmony_log(
    window: &tauri::Window,
    build_id: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    let event = crate::commands::build_android::BuildLogEvent {
        build_id: Some(build_id.to_string()),
        platform: "harmony".to_string(),
        level: level.to_string(),
        message: message.to_string(),
        progress,
    };
    let _ = window.emit("build-log", event);
}

#[tauri::command]
pub async fn prepare_harmony_build(options: HarmonyBuildOptions) -> Result<BuildResult, String> {
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    let template = require_configured_harmony_template(&sdk_config)?;
    let project_dir = std::path::Path::new(&options.project_path);
    if !project_dir.exists() {
        return Err(format!(
            "Project path does not exist: {}",
            options.project_path
        ));
    }

    Ok(BuildResult {
        success: true,
        output_path: None,
        logs: vec![
            "[prepare] HarmonyOS build environment checking...".to_string(),
            format!("[prepare] Harmony 工程模板: {}", template.display()),
            format!(
                "[prepare] Module: {}",
                options.module.as_deref().unwrap_or("entry")
            ),
            format!(
                "[prepare] Mode: {}",
                options.mode.as_deref().unwrap_or("debug")
            ),
        ],
        duration_ms: 0,
        error: None,
    })
}

#[tauri::command]
pub async fn run_harmony_build(
    options: HarmonyBuildOptions,
    app_handle: tauri::AppHandle,
) -> Result<BuildResult, String> {
    let start = std::time::Instant::now();

    let module = options.module.unwrap_or_else(|| "entry".to_string());
    let mode = options.mode.unwrap_or_else(|| "debug".to_string());
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    let _template = require_configured_harmony_template(&sdk_config)?;

    let hvigorw = std::path::Path::new(&options.project_path).join("hvigorw");
    if !hvigorw.exists() {
        return Err("hvigorw not found. Is this a valid HarmonyOS project?".to_string());
    }

    let mut args = vec![
        format!("assemble{}", mode),
        "-p".to_string(),
        format!("module={}", module),
    ];
    if options.clean.unwrap_or(false) {
        args.insert(0, "clean".to_string());
    }

    let output = crate::utils::process::run_command_streaming(
        &hvigorw.to_string_lossy(),
        &args,
        &options.project_path,
        app_handle,
        "harmony-build",
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
            Some("HarmonyOS build failed".to_string())
        },
    })
}

#[tauri::command]
pub async fn generate_harmony_project(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    window: tauri::Window,
) -> Result<String, String> {
    let _ = manifest_info;
    let build_id = build_id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "harmony-gen-{}",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            )
        });
    emit_harmony_log(
        &window,
        &build_id,
        "info",
        "开始生成 Harmony 工程（不执行打包）",
        Some(2),
    );
    let config = crate::commands::project::load_project_config_sync(&project_id)?;
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    validate_harmony_config(&config, &sdk_config)?;

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
    let harmony_template = crate::commands::sdk::resolve_harmony_template_root(Path::new(
        &sdk_config.harmony_template_path,
    ))?;
    crate::utils::fs::copy_recursive(&harmony_template, &workspace)
        .map_err(|e| format!("复制 Harmony 工程失败: {}", e))?;
    emit_harmony_log(
        &window,
        &build_id,
        "success",
        "Harmony 工程已复制到工作区",
        Some(18),
    );

    patch_harmony_json_files(&workspace, &config)?;
    patch_harmony_signing_files(&workspace, &config)?;
    import_harmony_resource(&workspace, &app_resource_dir, &scan.app_id)?;
    emit_harmony_log(
        &window,
        &build_id,
        "success",
        "Harmony 资源和配置已注入",
        Some(85),
    );

    let hvigorw = if cfg!(windows) {
        workspace.join("hvigorw.bat")
    } else {
        workspace.join("hvigorw")
    };
    if !hvigorw.exists() {
        return Err(format!("未找到 hvigorw: {}", hvigorw.display()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hvigorw)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hvigorw, perms).map_err(|e| e.to_string())?;
    }

    let workspace_display = workspace.to_string_lossy().to_string();
    emit_harmony_log(
        &window,
        &build_id,
        "success",
        &format!("Harmony 工程已生成: {}", workspace_display),
        Some(100),
    );
    Ok(workspace_display)
}

#[tauri::command]
pub async fn build_harmony_hap(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    window: tauri::Window,
) -> Result<crate::commands::build_android::BuildArtifact, String> {
    let _ = manifest_info;
    let build_id = build_id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("harmony-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    emit_harmony_log(
        &window,
        &build_id,
        "info",
        "开始 Harmony HAP 构建流程",
        Some(2),
    );
    let config = crate::commands::project::load_project_config_sync(&project_id)?;
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    validate_harmony_config(&config, &sdk_config)?;

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
    let harmony_template = crate::commands::sdk::resolve_harmony_template_root(Path::new(
        &sdk_config.harmony_template_path,
    ))?;
    crate::utils::fs::copy_recursive(&harmony_template, &workspace)
        .map_err(|e| format!("复制 Harmony 工程失败: {}", e))?;
    emit_harmony_log(
        &window,
        &build_id,
        "success",
        "Harmony 工程已复制到工作区",
        Some(15),
    );

    patch_harmony_json_files(&workspace, &config)?;
    patch_harmony_signing_files(&workspace, &config)?;
    import_harmony_resource(&workspace, &app_resource_dir, &scan.app_id)?;
    emit_harmony_log(
        &window,
        &build_id,
        "success",
        "Harmony 资源和配置已注入",
        Some(45),
    );

    let hvigorw = if cfg!(windows) {
        workspace.join("hvigorw.bat")
    } else {
        workspace.join("hvigorw")
    };
    if !hvigorw.exists() {
        return Err(format!("未找到 hvigorw: {}", hvigorw.display()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hvigorw)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hvigorw, perms).map_err(|e| e.to_string())?;
    }

    let args = vec![
        "assembleHap".to_string(),
        "--mode".to_string(),
        "release".to_string(),
    ];
    emit_harmony_log(
        &window,
        &build_id,
        "info",
        "执行 hvigorw assembleHap",
        Some(65),
    );
    let output = crate::utils::process::run_command_streaming_with_env_tagged(
        &hvigorw.to_string_lossy(),
        &args,
        &workspace.to_string_lossy(),
        &[],
        window.app_handle().clone(),
        "build-log",
        crate::utils::process::StreamLogMeta {
            build_id: build_id.clone(),
            platform: "harmony".to_string(),
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    if !output.success {
        return Err(format!("Harmony 构建失败，退出码: {:?}", output.exit_code));
    }

    let hap = find_file_with_ext(&workspace, "hap")
        .ok_or_else(|| "构建成功后未找到 HAP 文件".to_string())?;
    let output_dir = expand_home(&config.output_dir);
    crate::utils::fs::ensure_directory(&output_dir).map_err(|e| e.to_string())?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let dest = output_dir.join(format!("{}-v{}.hap", ts, config.app.version));
    std::fs::copy(&hap, &dest).map_err(|e| format!("复制 HAP 失败: {}", e))?;
    let size_bytes = std::fs::metadata(&dest)
        .map(|m| m.len())
        .unwrap_or_default();
    emit_harmony_log(
        &window,
        &build_id,
        "success",
        &format!("Harmony 打包完成: {}", dest.display()),
        Some(100),
    );

    Ok(crate::commands::build_android::BuildArtifact {
        platform: "harmony".to_string(),
        path: dest.to_string_lossy().to_string(),
        file_name: dest
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app.hap")
            .to_string(),
        size_bytes,
        build_id,
    })
}

fn validate_harmony_config(
    config: &crate::commands::project::ProjectConfig,
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<(), String> {
    require_configured_harmony_template(sdk_config)?;
    if config.harmony.bundle_name.trim().is_empty() {
        return Err("请先配置 Bundle Name".to_string());
    }
    let signing = &config.harmony.signing_config;
    if signing.store_file.trim().is_empty()
        || signing.key_alias.trim().is_empty()
        || !signing.has_store_password
        || !signing.has_key_password
    {
        return Err(
            "Harmony release 构建需要完整签名文件、Key Alias、Store 密码和 Key 密码".to_string(),
        );
    }
    Ok(())
}

fn require_configured_harmony_template(
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<PathBuf, String> {
    if sdk_config.harmony_template_path.trim().is_empty() {
        return Err("请先在 SDK & 环境管理中配置 Harmony 工程模板路径".to_string());
    }
    crate::commands::sdk::resolve_harmony_template_root(Path::new(
        &sdk_config.harmony_template_path,
    ))
}

fn patch_harmony_json_files(
    workspace: &Path,
    config: &crate::commands::project::ProjectConfig,
) -> Result<(), String> {
    for name in ["app.json5", "module.json5"] {
        if let Some(path) = find_file_named(workspace, name) {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let updated = content
                .replace(
                    "\"bundleName\": \"\"",
                    &format!("\"bundleName\": \"{}\"", config.harmony.bundle_name),
                )
                .replace(
                    "\"bundleName\":\"\"",
                    &format!("\"bundleName\":\"{}\"", config.harmony.bundle_name),
                )
                .replace(
                    "\"bundleName\": \"com.example.myapplication\"",
                    &format!("\"bundleName\": \"{}\"", config.harmony.bundle_name),
                )
                .replace(
                    "\"bundleName\":\"com.example.myapplication\"",
                    &format!("\"bundleName\":\"{}\"", config.harmony.bundle_name),
                )
                .replace(
                    "\"label\": \"$string:app_name\"",
                    &format!("\"label\": \"{}\"", config.app.name),
                )
                .replace(
                    "\"versionName\": \"1.0.0\"",
                    &format!("\"versionName\": \"{}\"", config.app.version),
                )
                .replace(
                    "\"versionCode\": 1",
                    &format!("\"versionCode\": {}", config.app.version_code),
                );
            std::fs::write(path, updated).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn patch_harmony_signing_files(
    workspace: &Path,
    config: &crate::commands::project::ProjectConfig,
) -> Result<(), String> {
    let store_password_key = format!("{}-harmony-store-password", config.id);
    let key_password_key = format!("{}-harmony-key-password", config.id);
    let store_password = crate::utils::keychain::get_password(&store_password_key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Keychain 中缺少 Harmony Store 密码".to_string())?;
    let key_password = crate::utils::keychain::get_password(&key_password_key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Keychain 中缺少 Harmony Key 密码".to_string())?;

    let signing = &config.harmony.signing_config;
    let signing_json = serde_json::json!({
        "storeFile": signing.store_file,
        "storePassword": store_password,
        "keyAlias": signing.key_alias,
        "keyPassword": key_password,
        "bundleName": config.harmony.bundle_name,
        "versionName": config.app.version,
        "versionCode": config.app.version_code,
        "runtimeVersion": config.harmony.runtime_version,
    });
    let signing_path = workspace.join("unipack-signing.json");
    let signing_text = serde_json::to_string_pretty(&signing_json).map_err(|e| e.to_string())?;
    std::fs::write(&signing_path, signing_text).map_err(|e| e.to_string())?;

    for name in ["build-profile.json5", "build-profile.json"] {
        if let Some(path) = find_file_named(workspace, name) {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let updated = content
                .replace("\"signingConfigs\": []", &format!("\"signingConfigs\": [{{\"name\":\"release\",\"material\":{{\"storeFile\":\"{}\",\"storePassword\":\"{}\",\"keyAlias\":\"{}\",\"keyPassword\":\"{}\"}}}}]", escape_json5(&signing.store_file), escape_json5(&store_password), escape_json5(&signing.key_alias), escape_json5(&key_password)))
                .replace("\"signingConfig\": \"\"", "\"signingConfig\": \"release\"");
            std::fs::write(path, updated).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn import_harmony_resource(
    workspace: &Path,
    resource_dir: &Path,
    app_id: &str,
) -> Result<(), String> {
    let rawfile = find_dir_named(workspace, "rawfile")
        .unwrap_or_else(|| workspace.join("entry/src/main/resources/rawfile"));
    let dest = rawfile.join("apps").join(app_id);
    crate::utils::fs::copy_recursive(resource_dir, &dest)
        .map_err(|e| format!("复制 Harmony 资源失败: {}", e))
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

fn escape_json5(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json5_escape_handles_windows_paths_and_quotes() {
        assert_eq!(
            escape_json5(r#"C:\keys\"release".p12"#),
            r#"C:\\keys\\\"release\".p12"#
        );
    }
}
