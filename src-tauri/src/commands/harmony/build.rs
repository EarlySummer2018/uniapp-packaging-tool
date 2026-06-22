//! 鸿蒙 HAP 构建模块

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};

use crate::commands::shared::resource::DetectedModule;

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
    let harmony_modules = harmony_detected_modules(manifest_info.as_ref());
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
    patch_oh_package(&workspace, &config, &harmony_modules)?;
    patch_entry_ability(&workspace)?;
    patch_harmony_signing_files(&workspace, &config)?;
    import_harmony_resource(&workspace, &app_resource_dir, &scan.app_id)?;
    emit_harmony_log(
        &window,
        &build_id,
        "info",
        &format!(
            "已按文档注入运行时依赖 (@dcloudio/uni-app-runtime: {}) 和 EntryAbility 初始化",
            config.harmony.runtime_version
        ),
        Some(50),
    );
    if !harmony_modules.is_empty() {
        let injected: Vec<&str> = harmony_modules
            .iter()
            .flat_map(|m| harmony_ohpm_packages_for_category(&m.category))
            .map(|(pkg, _)| pkg)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        emit_harmony_log(
            &window,
            &build_id,
            "info",
            &format!(
                "已按 manifest 声明的鸿蒙原生模块注入依赖: {}",
                injected.join(", ")
            ),
            None,
        );
    }
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
    let harmony_modules = harmony_detected_modules(manifest_info.as_ref());
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
    patch_oh_package(&workspace, &config, &harmony_modules)?;
    patch_entry_ability(&workspace)?;
    patch_harmony_signing_files(&workspace, &config)?;
    import_harmony_resource(&workspace, &app_resource_dir, &scan.app_id)?;
    emit_harmony_log(
        &window,
        &build_id,
        "info",
        &format!(
            "已按文档注入运行时依赖 (@dcloudio/uni-app-runtime: {}) 和 EntryAbility 初始化",
            config.harmony.runtime_version
        ),
        Some(35),
    );
    if !harmony_modules.is_empty() {
        let injected: Vec<&str> = harmony_modules
            .iter()
            .flat_map(|m| harmony_ohpm_packages_for_category(&m.category))
            .map(|(pkg, _)| pkg)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        emit_harmony_log(
            &window,
            &build_id,
            "info",
            &format!(
                "已按 manifest 声明的鸿蒙原生模块注入依赖: {}",
                injected.join(", ")
            ),
            None,
        );
    }
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
    if config.harmony.runtime_version.trim().is_empty() {
        return Err(
            "请在项目配置中填写鸿蒙运行时版本（@dcloudio/uni-app-runtime 的版本号）"
                .to_string(),
        );
    }
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

/// 官方文档要求的 EntryAbility.ets 模板（UniEntryAbility + initUniModules）
const ENTRY_ABILITY_TEMPLATE: &str = r#"import { UniEntryAbility } from "@dcloudio/uni-app-runtime";
import { initUniModules } from "../uni_modules/index.generated";
import BuildProfile from "BuildProfile";

initUniModules();

export default class EntryAbility extends UniEntryAbility {
  constructor() {
    super("HBuilder", { debug: BuildProfile.DEBUG });
  }
}
"#;

/// 在工程根目录 oh-package.json5 中注入 @dcloudio/uni-app-runtime 依赖，
/// 并按 manifest 中声明的鸿蒙原生模块注入对应的 @uni_modules/* 依赖。
fn patch_oh_package(
    workspace: &Path,
    config: &crate::commands::project::ProjectConfig,
    modules: &[DetectedModule],
) -> Result<(), String> {
    let runtime_version = config.harmony.runtime_version.trim();
    if runtime_version.is_empty() {
        return Err(
            "请在项目配置中填写鸿蒙运行时版本（@dcloudio/uni-app-runtime 的版本号）"
                .to_string(),
        );
    }

    // 鸿蒙工程根目录和 entry 等模块各有 oh-package.json5，必须改根目录的。
    // 策略：优先取 workspace 直接子层的，找不到再递归搜索。
    let root_oh_pkg = workspace.join("oh-package.json5");
    let oh_pkg_path = if root_oh_pkg.exists() {
        root_oh_pkg
    } else if let Some(found) = find_file_named(workspace, "oh-package.json5") {
        found
    } else {
        return Err("未在模板工程中找到 oh-package.json5".to_string());
    };

    let content = std::fs::read_to_string(&oh_pkg_path).map_err(|e| e.to_string())?;
    let mut value: serde_json::Value =
        json5::from_str(&content).map_err(|e| format!("解析 oh-package.json5 失败: {}", e))?;

    let deps = value
        .as_object_mut()
        .ok_or("oh-package.json5 根节点不是对象")?
        .entry("dependencies")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("oh-package.json5 的 dependencies 不是对象")?;

    deps.insert(
        "@dcloudio/uni-app-runtime".to_string(),
        serde_json::Value::String(runtime_version.to_string()),
    );

    // 注入鸿蒙原生模块依赖。使用 BTreeMap 按包名天然去重——例如支付模块需要
    // uni-facialrecognitionverify，而实人认证模块也依赖同一个包，合并后只写入一次。
    let mut ohpm_packages: BTreeMap<&str, &str> = BTreeMap::new();
    for module in modules {
        if !module.platforms.iter().any(|p| p == "harmony") {
            continue;
        }
        for (package, version) in harmony_ohpm_packages_for_category(&module.category) {
            ohpm_packages.insert(package, version);
        }
    }
    for (package, version) in &ohpm_packages {
        deps.insert(
            package.to_string(),
            serde_json::Value::String(version.to_string()),
        );
    }

    let updated = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    std::fs::write(&oh_pkg_path, updated).map_err(|e| e.to_string())?;

    Ok(())
}

/// 将模块分类映射到鸿蒙端需要注入的 ohpm 包列表（包名、版本）。
///
/// 版本号与包名取自官方鸿蒙模块文档：
/// - push:    https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/push.html
/// - oauth:   https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/oauth.html
/// - pay:     https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/pay.html
/// - 实人认证: https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/facialRecognitionVerify.html
///
/// 注意：
/// - 地图（map）在鸿蒙端为内置 web 方案，官方文档明确「无需配置模块依赖」，本轮返回空。
/// - 支付（payment）按用户要求同时注入 uni-payment-alipay 与 uni-facialrecognitionverify，
///   去重由 `patch_oh_package` 中的 BTreeMap 负责。
fn harmony_ohpm_packages_for_category(category: &str) -> Vec<(&'static str, &'static str)> {
    match category {
        "push" => vec![("@uni_modules/uni-push", "1.0.1")],
        "login" => vec![("@uni_modules/uni-oauth-huawei", "1.0.1")],
        "payment" => vec![
            ("@uni_modules/uni-payment-alipay", "1.0.1"),
            ("@uni_modules/uni-facialrecognitionverify", "1.0.2"),
        ],
        "face_recognition" => vec![("@uni_modules/uni-facialrecognitionverify", "1.0.2")],
        // 地图鸿蒙端为内置 web 方案，无需模块依赖。
        "map" => Vec::new(),
        _ => Vec::new(),
    }
}

/// 从 manifest 解析结果中提取需要参与鸿蒙构建的模块（platforms 含 "harmony"）。
fn harmony_detected_modules(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Vec<DetectedModule> {
    let Some(info) = manifest_info else {
        return Vec::new();
    };
    info.detected_modules
        .iter()
        .filter(|m| m.platforms.iter().any(|p| p == "harmony"))
        .cloned()
        .collect()
}

/// 用官方 UniEntryAbility 模板覆盖 EntryAbility.ets
fn patch_entry_ability(workspace: &Path) -> Result<(), String> {
    // DevEco Studio 标准路径
    let standard_path = workspace
        .join("entry")
        .join("src")
        .join("main")
        .join("ets")
        .join("entryability")
        .join("EntryAbility.ets");

    let target = if standard_path.exists() {
        standard_path
    } else if let Some(found) = find_file_named(workspace, "EntryAbility.ets") {
        found
    } else {
        return Err(
            "未在模板工程中找到 EntryAbility.ets，请确认模板是 DevEco Studio 创建的完整工程"
                .to_string(),
        );
    };

    // 确保父目录存在
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    std::fs::write(&target, ENTRY_ABILITY_TEMPLATE).map_err(|e| e.to_string())?;
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

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }

    fn make_test_config(runtime_version: &str) -> crate::commands::project::ProjectConfig {
        let mut config = crate::commands::project::ProjectConfig::default();
        config.harmony = crate::commands::project::HarmonyConfig {
            enabled: true,
            bundle_name: "com.test.app".to_string(),
            runtime_version: runtime_version.to_string(),
            signing_config: crate::commands::project::HarmonySigningConfig {
                store_file: String::new(),
                key_alias: String::new(),
                has_store_password: true,
                has_key_password: true,
            },
        };
        config
    }

    /// 构造一个标记为鸿蒙平台的 `DetectedModule`。
    fn harmony_module(category: &str) -> DetectedModule {
        DetectedModule {
            name: format!("uni-{}", category),
            category: category.to_string(),
            platforms: vec!["harmony".to_string()],
            configured: false,
            required_keys: Vec::new(),
            source: "app-harmony".to_string(),
        }
    }

    #[test]
    fn json5_escape_handles_windows_paths_and_quotes() {
        assert_eq!(
            escape_json5(r#"C:\keys\"release".p12"#),
            r#"C:\\keys\\\"release\".p12"#
        );
    }

    #[test]
    fn patch_oh_package_injects_runtime_dependency() {
        let dir = unique_temp_dir("unipack-test-oh-pkg");
        std::fs::create_dir_all(&dir).unwrap();

        // 模拟根目录 oh-package.json5（无 dependencies）
        std::fs::write(
            dir.join("oh-package.json5"),
            r#"{ "name": "MyApp", "version": "1.0.0" }"#,
        )
        .unwrap();

        let config = make_test_config("1.0.2");
        patch_oh_package(&dir, &config, &[]).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        assert_eq!(
            value["dependencies"]["@dcloudio/uni-app-runtime"],
            "1.0.2"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_oh_package_preserves_existing_dependencies() {
        let dir = unique_temp_dir("unipack-test-oh-pkg-existing");
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(
            dir.join("oh-package.json5"),
            r#"{ "name": "MyApp", "dependencies": { "some-lib": "2.0.0" } }"#,
        )
        .unwrap();

        let config = make_test_config("1.0.3");
        patch_oh_package(&dir, &config, &[]).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        assert_eq!(value["dependencies"]["some-lib"], "2.0.0");
        assert_eq!(
            value["dependencies"]["@dcloudio/uni-app-runtime"],
            "1.0.3"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_oh_package_rejects_empty_runtime_version() {
        let dir = unique_temp_dir("unipack-test-oh-pkg-empty");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("oh-package.json5"), r#"{ "name": "MyApp" }"#).unwrap();

        let config = make_test_config("");
        let result = patch_oh_package(&dir, &config, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("运行时版本"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_oh_package_injects_module_dependencies() {
        let dir = unique_temp_dir("unipack-test-oh-pkg-modules");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("oh-package.json5"),
            r#"{ "name": "MyApp", "dependencies": {} }"#,
        )
        .unwrap();

        let config = make_test_config("1.0.2");
        let modules = vec![
            harmony_module("push"),
            harmony_module("login"),
            harmony_module("payment"),
            harmony_module("face_recognition"),
        ];
        patch_oh_package(&dir, &config, &modules).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        let deps = &value["dependencies"];
        assert_eq!(deps["@uni_modules/uni-push"], "1.0.1");
        assert_eq!(deps["@uni_modules/uni-oauth-huawei"], "1.0.1");
        assert_eq!(deps["@uni_modules/uni-payment-alipay"], "1.0.1");
        assert_eq!(deps["@uni_modules/uni-facialrecognitionverify"], "1.0.2");
        // 运行时依赖依然注入。
        assert_eq!(deps["@dcloudio/uni-app-runtime"], "1.0.2");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_oh_package_dedups_facialrecognitionverify() {
        // 支付与实人认证模块同时开启：uni-facialrecognitionverify 只应出现一次。
        let dir = unique_temp_dir("unipack-test-oh-pkg-dedup");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("oh-package.json5"),
            r#"{ "name": "MyApp", "dependencies": {} }"#,
        )
        .unwrap();

        let config = make_test_config("1.0.2");
        let modules = vec![harmony_module("payment"), harmony_module("face_recognition")];
        patch_oh_package(&dir, &config, &modules).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        // 用文本计数确保 key 只出现一次。
        let occurrences = content.matches("@uni_modules/uni-facialrecognitionverify").count();
        assert_eq!(occurrences, 1, "uni-facialrecognitionverify 应去重后只出现一次");

        let value: serde_json::Value = json5::from_str(&content).unwrap();
        assert_eq!(
            value["dependencies"]["@uni_modules/uni-facialrecognitionverify"],
            "1.0.2"
        );
        assert_eq!(
            value["dependencies"]["@uni_modules/uni-payment-alipay"],
            "1.0.1"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_oh_package_ignores_non_harmony_modules() {
        // platforms 不含 harmony 的模块（如来自 app-plus 的 Android/iOS 模块）应被忽略。
        let dir = unique_temp_dir("unipack-test-oh-pkg-ignore");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("oh-package.json5"),
            r#"{ "name": "MyApp", "dependencies": {} }"#,
        )
        .unwrap();

        let config = make_test_config("1.0.2");
        let android_push = DetectedModule {
            name: "Push".to_string(),
            category: "push".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: Vec::new(),
            source: "manifest.json".to_string(),
        };
        patch_oh_package(&dir, &config, &[android_push]).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        // 仅运行时依赖，模块依赖不应被注入。
        assert_eq!(
            value["dependencies"]["@dcloudio/uni-app-runtime"],
            "1.0.2"
        );
        assert!(value["dependencies"].get("@uni_modules/uni-push").is_none());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_entry_ability_overwrites_at_standard_path() {
        let dir = unique_temp_dir("unipack-test-entry-ability");
        let ability_dir = dir
            .join("entry")
            .join("src")
            .join("main")
            .join("ets")
            .join("entryability");
        std::fs::create_dir_all(&ability_dir).unwrap();

        // 写一个假的原始文件
        std::fs::write(ability_dir.join("EntryAbility.ets"), "old content").unwrap();

        patch_entry_ability(&dir).unwrap();

        let content =
            std::fs::read_to_string(ability_dir.join("EntryAbility.ets")).unwrap();
        assert!(content.contains("UniEntryAbility"));
        assert!(content.contains("initUniModules"));
        assert!(content.contains("@dcloudio/uni-app-runtime"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_entry_ability_errs_when_missing() {
        let dir = unique_temp_dir("unipack-test-entry-ability-missing");
        std::fs::create_dir_all(&dir).unwrap();

        let result = patch_entry_ability(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("EntryAbility.ets"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn validate_harmony_config_rejects_empty_runtime_version() {
        let dir = unique_temp_dir("unipack-test-validate-rt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("hvigorw"), "").unwrap();

        let config = make_test_config("");
        let sdk_config = crate::commands::sdk::GlobalSdkConfig {
            dcloud_android_sdk_path: String::new(),
            dcloud_ios_sdk_path: String::new(),
            harmony_template_path: dir.to_string_lossy().to_string(),
        };
        let result = validate_harmony_config(&config, &sdk_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("运行时版本"));

        let _ = std::fs::remove_dir_all(dir);
    }
}
