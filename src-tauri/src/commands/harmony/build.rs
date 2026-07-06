//! 鸿蒙 HAP 构建模块

use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};

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
pub async fn generate_harmony_project(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    window: tauri::Window,
) -> Result<String, String> {
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
    let resource_package_dir = PathBuf::from(&scan.imported_path);
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
    patch_harmony_module_metadata(&workspace, manifest_info.as_ref())?;
    patch_oh_package(&workspace, &config, &resource_package_dir)?;
    patch_harmony_build_profile(&workspace, &resource_package_dir)?;
    patch_entry_ability(&workspace)?;
    patch_harmony_signing_files(&workspace, &config)?;
    import_harmony_resource(&workspace, &app_resource_dir)?;
    integrate_harmony_uni_modules(&workspace, &resource_package_dir)?;
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
    let resource_package_dir = PathBuf::from(&scan.imported_path);
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
    patch_harmony_module_metadata(&workspace, manifest_info.as_ref())?;
    patch_oh_package(&workspace, &config, &resource_package_dir)?;
    patch_harmony_build_profile(&workspace, &resource_package_dir)?;
    patch_entry_ability(&workspace)?;
    patch_harmony_signing_files(&workspace, &config)?;
    import_harmony_resource(&workspace, &app_resource_dir)?;
    integrate_harmony_uni_modules(&workspace, &resource_package_dir)?;
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
        cloud_run_url: None,
    })
}

fn validate_harmony_config(
    config: &crate::commands::project::ProjectConfig,
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<(), String> {
    require_configured_harmony_template(sdk_config)?;
    if config.harmony.runtime_version.trim().is_empty() {
        return Err(
            "请在项目配置中填写鸿蒙运行时版本（@dcloudio/uni-app-runtime 的版本号）".to_string(),
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

fn patch_harmony_module_metadata(
    workspace: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let Some(manifest_info) = manifest_info else {
        return Ok(());
    };
    if !manifest_info.detected_modules.iter().any(|module| {
        module.name == "uni-map"
            && module.category == "map"
            && module.source == "app-harmony"
            && module
                .platforms
                .iter()
                .any(|platform| platform == "harmony")
    }) {
        return Ok(());
    }

    let manifest_value =
        crate::commands::shared::module::analysis::manifest_value_from_info(manifest_info);
    let Some(key) = crate::commands::shared::module::analysis::harmony_uni_map_tencent_key(
        manifest_value.as_ref(),
    ) else {
        return Err(
            "Harmony 地图模块已开启，请在 manifest.json 配置 app-harmony.distribute.modules.uni-map.tencent.key"
                .to_string(),
        );
    };

    let module_json_path = workspace
        .join("entry")
        .join("src")
        .join("main")
        .join("module.json5");
    let module_json_path = if module_json_path.is_file() {
        module_json_path
    } else {
        find_file_named(workspace, "module.json5")
            .ok_or_else(|| "未在模板工程中找到 entry/src/main/module.json5".to_string())?
    };

    let mut value = read_json5_file(&module_json_path, "module.json5")?;
    upsert_harmony_module_metadata(&mut value, "TENCENT_MAP_KEY", key)?;
    let updated = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    std::fs::write(module_json_path, updated).map_err(|e| e.to_string())?;
    Ok(())
}

fn upsert_harmony_module_metadata(
    module_json: &mut serde_json::Value,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let root = module_json
        .as_object_mut()
        .ok_or("module.json5 根节点不是对象")?;
    let module = root
        .entry("module")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .ok_or("module.json5 的 module 不是对象")?;
    let metadata = module
        .entry("metadata")
        .or_insert_with(|| serde_json::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or("module.json5 的 module.metadata 不是数组")?;

    for item in metadata.iter_mut() {
        let Some(item) = item.as_object_mut() else {
            continue;
        };
        if item.get("name").and_then(|value| value.as_str()) == Some(name) {
            item.insert(
                "value".to_string(),
                serde_json::Value::String(value.to_string()),
            );
            return Ok(());
        }
    }

    metadata.push(serde_json::json!({
        "name": name,
        "value": value
    }));
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
        if let Some(path) = find_project_file_named(workspace, name) {
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

/// 在工程根目录 oh-package.json5 中注入 @dcloudio/uni-app-runtime，
/// 并按文档合并 /resource/uni_modules/oh-package.json5 里的依赖。
fn patch_oh_package(
    workspace: &Path,
    config: &crate::commands::project::ProjectConfig,
    resource_package_dir: &Path,
) -> Result<(), String> {
    let runtime_version = config.harmony.runtime_version.trim();
    if runtime_version.is_empty() {
        return Err(
            "请在项目配置中填写鸿蒙运行时版本（@dcloudio/uni-app-runtime 的版本号）".to_string(),
        );
    }

    let oh_pkg_path = find_project_file_named(workspace, "oh-package.json5")
        .ok_or_else(|| "未在模板工程中找到 oh-package.json5".to_string())?;

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

    let generated_oh_pkg = resource_package_dir
        .join("uni_modules")
        .join("oh-package.json5");
    if generated_oh_pkg.is_file() {
        let generated = read_json5_file(&generated_oh_pkg, "编译产物 oh-package.json5")?;
        let generated_deps = generated
            .get("dependencies")
            .and_then(|deps| deps.as_object())
            .ok_or_else(|| {
                format!(
                    "编译产物 {} 缺少 dependencies 对象",
                    generated_oh_pkg.display()
                )
            })?;
        for (name, dep) in generated_deps {
            deps.insert(name.clone(), dep.clone());
        }
    }

    deps.insert(
        "@dcloudio/uni-app-runtime".to_string(),
        serde_json::Value::String(runtime_version.to_string()),
    );

    let updated = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    std::fs::write(&oh_pkg_path, updated).map_err(|e| e.to_string())?;

    Ok(())
}

/// 按文档合并 /resource/uni_modules/build-profile.json5 的 modules，
/// 并确保 HBuilderX 4.51+ 所需 compatibleSdkVersionStage 为 beta6。
fn patch_harmony_build_profile(
    workspace: &Path,
    resource_package_dir: &Path,
) -> Result<(), String> {
    let build_profile_path = find_project_file_named(workspace, "build-profile.json5")
        .or_else(|| find_project_file_named(workspace, "build-profile.json"))
        .ok_or_else(|| "未在模板工程中找到 build-profile.json5".to_string())?;
    let mut value = read_json5_file(&build_profile_path, "build-profile.json5")?;

    let generated_build_profile = resource_package_dir
        .join("uni_modules")
        .join("build-profile.json5");
    if generated_build_profile.is_file() {
        let generated = read_json5_file(&generated_build_profile, "编译产物 build-profile.json5")?;
        if let Some(generated_modules) = generated.get("modules").and_then(|m| m.as_array()) {
            merge_modules_array(&mut value, generated_modules)?;
        }
    }
    ensure_compatible_sdk_beta6(&mut value)?;

    let updated = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    std::fs::write(&build_profile_path, updated).map_err(|e| e.to_string())?;
    Ok(())
}

fn merge_modules_array(
    build_profile: &mut serde_json::Value,
    generated_modules: &[serde_json::Value],
) -> Result<(), String> {
    let root = build_profile
        .as_object_mut()
        .ok_or("build-profile.json5 根节点不是对象")?;
    let modules = root
        .entry("modules")
        .or_insert_with(|| serde_json::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or("build-profile.json5 的 modules 不是数组")?;

    for generated_module in generated_modules {
        let generated_name = generated_module.get("name").and_then(|name| name.as_str());
        if let Some(name) = generated_name {
            if let Some(existing) = modules
                .iter_mut()
                .find(|module| module.get("name").and_then(|value| value.as_str()) == Some(name))
            {
                *existing = generated_module.clone();
                continue;
            }
        }
        modules.push(generated_module.clone());
    }
    Ok(())
}

fn ensure_compatible_sdk_beta6(build_profile: &mut serde_json::Value) -> Result<(), String> {
    let root = build_profile
        .as_object_mut()
        .ok_or("build-profile.json5 根节点不是对象")?;
    let app = root
        .entry("app")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .ok_or("build-profile.json5 的 app 不是对象")?;
    let products = app
        .entry("products")
        .or_insert_with(|| serde_json::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or("build-profile.json5 的 app.products 不是数组")?;

    if products.is_empty() {
        products.push(serde_json::json!({
            "name": "default",
            "compatibleSdkVersionStage": "beta6"
        }));
        return Ok(());
    }

    for product in products {
        let product = product
            .as_object_mut()
            .ok_or("build-profile.json5 的 app.products 项不是对象")?;
        product.insert(
            "compatibleSdkVersionStage".to_string(),
            serde_json::Value::String("beta6".to_string()),
        );
    }
    Ok(())
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

fn import_harmony_resource(workspace: &Path, resource_dir: &Path) -> Result<(), String> {
    let dest = workspace.join("entry/src/main/resources/resfile/apps/HBuilder");
    copy_directory_replace(resource_dir, &dest).map_err(|e| format!("复制 Harmony 资源失败: {}", e))
}

fn integrate_harmony_uni_modules(
    workspace: &Path,
    resource_package_dir: &Path,
) -> Result<(), String> {
    let source_uni_modules = resource_package_dir.join("uni_modules");
    let target_ets_dir = workspace.join("entry/src/main/ets/uni_modules");
    std::fs::create_dir_all(&target_ets_dir).map_err(|e| e.to_string())?;

    let source_index = source_uni_modules.join("index.generated.ets");
    let target_index = target_ets_dir.join("index.generated.ets");
    if source_index.is_file() {
        crate::utils::fs::copy_file(&source_index, &target_index)
            .map_err(|e| format!("复制 index.generated.ets 失败: {}", e))?;
    } else {
        std::fs::write(&target_index, "export function initUniModules() {\n}\n")
            .map_err(|e| format!("创建默认 index.generated.ets 失败: {}", e))?;
    }

    if !source_uni_modules.is_dir() {
        return Ok(());
    }

    let target_uni_modules = workspace.join("uni_modules");
    std::fs::create_dir_all(&target_uni_modules).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(&source_uni_modules).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let source_path = entry.path();
        if !source_path.is_dir() {
            continue;
        }
        let target_path = target_uni_modules.join(entry.file_name());
        copy_directory_replace(&source_path, &target_path)
            .map_err(|e| format!("复制 uni_modules 模块失败: {}", e))?;
    }

    Ok(())
}

fn copy_directory_replace(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if dst.exists() {
        std::fs::remove_dir_all(dst)?;
    }
    crate::utils::fs::copy_recursive(src, dst)
}

fn read_json5_file(path: &Path, label: &str) -> Result<serde_json::Value, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("读取 {} 失败: {}", label, e))?;
    json5::from_str(&content).map_err(|e| format!("解析 {} 失败: {}", label, e))
}

fn find_project_file_named(workspace: &Path, name: &str) -> Option<PathBuf> {
    let direct = workspace.join(name);
    if direct.is_file() {
        Some(direct)
    } else {
        find_file_named(workspace, name)
    }
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

    fn make_harmony_map_manifest_info(
        manifest_value: serde_json::Value,
    ) -> crate::commands::resource::UniappManifestInfo {
        crate::commands::resource::UniappManifestInfo {
            app_name: None,
            app_id: None,
            version_name: None,
            version_code: None,
            hbuilderx_version: None,
            android_icons: None,
            ios_icons: None,
            push_icons: None,
            splashscreen: None,
            ios_privacy_descriptions: Default::default(),
            manifest_value: Some(manifest_value),
            manifest_path: String::new(),
            project_root: String::new(),
            android: crate::commands::shared::resource::AndroidManifestConfig {
                package_name: None,
                min_sdk_version: None,
                target_sdk_version: None,
                compile_sdk_version: None,
                permissions: Vec::new(),
                exclude_permissions: Vec::new(),
                schemes: Vec::new(),
                abi_filters: Vec::new(),
            },
            package_names: crate::commands::shared::resource::PlatformPackages {
                android_package: None,
                ios_bundle_id: None,
                harmony_bundle: None,
            },
            detected_modules: vec![crate::commands::resource::DetectedModule {
                name: "uni-map".to_string(),
                category: "map".to_string(),
                platforms: vec!["harmony".to_string()],
                configured: false,
                required_keys: Vec::new(),
                source: "app-harmony".to_string(),
            }],
            warnings: Vec::new(),
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
        patch_oh_package(&dir, &config, &dir).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        assert_eq!(value["dependencies"]["@dcloudio/uni-app-runtime"], "1.0.2");
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
        patch_oh_package(&dir, &config, &dir).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        assert_eq!(value["dependencies"]["some-lib"], "2.0.0");
        assert_eq!(value["dependencies"]["@dcloudio/uni-app-runtime"], "1.0.3");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_oh_package_rejects_empty_runtime_version() {
        let dir = unique_temp_dir("unipack-test-oh-pkg-empty");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("oh-package.json5"), r#"{ "name": "MyApp" }"#).unwrap();

        let config = make_test_config("");
        let result = patch_oh_package(&dir, &config, &dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("运行时版本"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_oh_package_merges_generated_dependencies() {
        let dir = unique_temp_dir("unipack-test-oh-pkg-modules");
        std::fs::create_dir_all(&dir).unwrap();
        let resource = dir.join("resource");
        std::fs::create_dir_all(resource.join("uni_modules")).unwrap();
        std::fs::write(
            dir.join("oh-package.json5"),
            r#"{ "name": "MyApp", "dependencies": { "existing": "1.0.0" } }"#,
        )
        .unwrap();
        std::fs::write(
            resource.join("uni_modules/oh-package.json5"),
            r#"{
              "dependencies": {
                "@uni_modules/uni-getbatteryinfo": "./uni_modules/uni-getBatteryInfo",
                "@uni_modules/uni-push": "1.0.1"
              }
            }"#,
        )
        .unwrap();

        let config = make_test_config("1.0.2");
        patch_oh_package(&dir, &config, &resource).unwrap();

        let content = std::fs::read_to_string(dir.join("oh-package.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        let deps = &value["dependencies"];
        assert_eq!(deps["existing"], "1.0.0");
        assert_eq!(deps["@dcloudio/uni-app-runtime"], "1.0.2");
        assert_eq!(
            deps["@uni_modules/uni-getbatteryinfo"],
            "./uni_modules/uni-getBatteryInfo"
        );
        assert_eq!(deps["@uni_modules/uni-push"], "1.0.1");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_harmony_build_profile_merges_modules_and_beta6() {
        let dir = unique_temp_dir("unipack-test-build-profile");
        std::fs::create_dir_all(&dir).unwrap();
        let resource = dir.join("resource");
        std::fs::create_dir_all(resource.join("uni_modules")).unwrap();
        std::fs::write(
            dir.join("build-profile.json5"),
            r#"{
              "app": { "products": [{ "name": "default" }] },
              "modules": [{ "name": "entry", "srcPath": "./entry" }]
            }"#,
        )
        .unwrap();
        std::fs::write(
            resource.join("uni_modules/build-profile.json5"),
            r#"{
              "modules": [
                { "name": "uni_modules__uni_getbatteryinfo", "srcPath": "./uni_modules/uni-getBatteryInfo" }
              ]
            }"#,
        )
        .unwrap();

        patch_harmony_build_profile(&dir, &resource).unwrap();

        let content = std::fs::read_to_string(dir.join("build-profile.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        assert_eq!(
            value["app"]["products"][0]["compatibleSdkVersionStage"],
            "beta6"
        );
        assert!(value["modules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|module| module["name"] == "uni_modules__uni_getbatteryinfo"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn import_harmony_resource_uses_documented_resfile_hbuilder_path() {
        let dir = unique_temp_dir("unipack-test-harmony-resource");
        let workspace = dir.join("workspace");
        let resource = dir.join("__UNI__DEMO");
        std::fs::create_dir_all(&resource).unwrap();
        std::fs::write(resource.join("manifest.json"), "{}").unwrap();

        import_harmony_resource(&workspace, &resource).unwrap();

        assert!(workspace
            .join("entry/src/main/resources/resfile/apps/HBuilder/manifest.json")
            .is_file());
        assert!(!workspace
            .join("entry/src/main/resources/rawfile/apps/__UNI__DEMO")
            .exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_harmony_module_metadata_injects_tencent_map_key() {
        let dir = unique_temp_dir("unipack-test-harmony-map-metadata");
        let module_dir = dir.join("entry/src/main");
        std::fs::create_dir_all(&module_dir).unwrap();
        std::fs::write(
            module_dir.join("module.json5"),
            r#"{
              "module": {
                "name": "entry",
                "metadata": [
                  { "name": "EXISTING_KEY", "value": "old" }
                ]
              }
            }"#,
        )
        .unwrap();
        let manifest_info = make_harmony_map_manifest_info(serde_json::json!({
            "app-harmony": {
                "distribute": {
                    "modules": {
                        "uni-map": {
                            "tencent": {
                                "key": "312312"
                            }
                        }
                    }
                }
            }
        }));

        patch_harmony_module_metadata(&dir, Some(&manifest_info)).unwrap();

        let content = std::fs::read_to_string(module_dir.join("module.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        let metadata = value["module"]["metadata"].as_array().unwrap();
        assert!(metadata
            .iter()
            .any(|item| item["name"] == "TENCENT_MAP_KEY" && item["value"] == "312312"));
        assert!(metadata
            .iter()
            .any(|item| item["name"] == "EXISTING_KEY" && item["value"] == "old"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_harmony_module_metadata_updates_existing_tencent_map_key() {
        let dir = unique_temp_dir("unipack-test-harmony-map-metadata-update");
        let module_dir = dir.join("entry/src/main");
        std::fs::create_dir_all(&module_dir).unwrap();
        std::fs::write(
            module_dir.join("module.json5"),
            r#"{
              "module": {
                "metadata": [
                  { "name": "TENCENT_MAP_KEY", "value": "old-key" }
                ]
              }
            }"#,
        )
        .unwrap();
        let manifest_info = make_harmony_map_manifest_info(serde_json::json!({
            "app-harmony": {
                "distribute": {
                    "modules": {
                        "uni-map": {
                            "tencent": {
                                "key": "new-key"
                            }
                        }
                    }
                }
            }
        }));

        patch_harmony_module_metadata(&dir, Some(&manifest_info)).unwrap();

        let content = std::fs::read_to_string(module_dir.join("module.json5")).unwrap();
        let value: serde_json::Value = json5::from_str(&content).unwrap();
        let metadata = value["module"]["metadata"].as_array().unwrap();
        assert_eq!(
            metadata
                .iter()
                .filter(|item| item["name"] == "TENCENT_MAP_KEY")
                .count(),
            1
        );
        assert!(metadata
            .iter()
            .any(|item| item["name"] == "TENCENT_MAP_KEY" && item["value"] == "new-key"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_harmony_module_metadata_requires_tencent_map_key_when_map_enabled() {
        let dir = unique_temp_dir("unipack-test-harmony-map-metadata-missing");
        let module_dir = dir.join("entry/src/main");
        std::fs::create_dir_all(&module_dir).unwrap();
        std::fs::write(
            module_dir.join("module.json5"),
            r#"{ "module": { "metadata": [] } }"#,
        )
        .unwrap();
        let manifest_info = make_harmony_map_manifest_info(serde_json::json!({
            "app-harmony": {
                "distribute": {
                    "modules": {
                        "uni-map": {
                            "tencent": {}
                        }
                    }
                }
            }
        }));

        let result = patch_harmony_module_metadata(&dir, Some(&manifest_info));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("uni-map.tencent.key"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn integrate_harmony_uni_modules_copies_generated_entry_and_modules() {
        let dir = unique_temp_dir("unipack-test-harmony-uni-modules");
        let workspace = dir.join("workspace");
        let resource = dir.join("resource");
        let source_uni_modules = resource.join("uni_modules");
        std::fs::create_dir_all(source_uni_modules.join("uni-getBatteryInfo")).unwrap();
        std::fs::write(
            source_uni_modules.join("index.generated.ets"),
            "export function initUniModules() { return 'generated' }\n",
        )
        .unwrap();
        std::fs::write(
            source_uni_modules.join("uni-getBatteryInfo/oh-package.json5"),
            "{ name: 'uni-getBatteryInfo' }",
        )
        .unwrap();

        integrate_harmony_uni_modules(&workspace, &resource).unwrap();

        let index = std::fs::read_to_string(
            workspace.join("entry/src/main/ets/uni_modules/index.generated.ets"),
        )
        .unwrap();
        assert!(index.contains("generated"));
        assert!(workspace
            .join("uni_modules/uni-getBatteryInfo/oh-package.json5")
            .is_file());
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

        let content = std::fs::read_to_string(ability_dir.join("EntryAbility.ets")).unwrap();
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
