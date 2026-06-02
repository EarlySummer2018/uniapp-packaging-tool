//! Android APK 构建模块（V2 - 基于方式二：导入工程）
//!
//! ## 架构说明
//! 使用 DCloud 官方 **HBuilder-Integrate-AS** 工程作为模板基础（`android-template-v2`），
//! 通过 [`AndroidProjectModifier`](crate::utils::android_project_mod::AndroidProjectModifier) 在运行时动态修改配置，
//! 替代了 V1 的模板渲染机制（`.tmpl` 文件渲染）。
//!
//! ## 主要改进
//! - ✅ 与官方 SDK 保持一致，减少兼容性问题
//! - ✅ 支持 Android Studio 直接打开工作区进行调试
//! - ✅ 模板升级简单，只需替换 `android-template-v2/` 目录
//! - ✅ 修改逻辑集中，易于维护和扩展
//!
//! ## 文件结构（V2）
//! ```text
//! workspace/                         # 构建工作区
//! ├── simpleDemo/                   # 主应用模块（来自 HBuilder-Integrate-AS）
//! │   ├── build.gradle              # 由 Modifier 动态修改
//! │   ├── libs/                     # 运行时填入 AAR 文件
//! │   └── src/main/...
//! ├── uts-modules/                  # UTS 插件（按需创建）
//! └── settings.gradle               # 包含 UTS 插件 include
//! ```
//!
//! ## 参考文档
//! - [DCloud Android 离线SDK使用说明](https://nativesupport.dcloud.net.cn/AppDocs/usesdk/android.html)
//! - [Android 模块配置教程](../../module-tutorial-android.md)
//! - [迁移计划](../../../.trae/documents/android-packaging-migration-plan-v1-to-v2.md)

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidBuildOptions {
    pub project_path: String,
    pub variant: Option<String>,
    pub clean: Option<bool>,
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub logs: Vec<String>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildArtifact {
    pub platform: String,
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub build_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildLogEvent {
    pub build_id: Option<String>,
    pub platform: String,
    pub level: String,
    pub message: String,
    pub progress: Option<u8>,
}

pub struct AppState {}

const UTS_RUNTIME_DEPS: &[&str] = &[
    "com.squareup.okhttp3:okhttp:3.12.12",
    "androidx.core:core-ktx:1.6.0",
    "org.jetbrains.kotlin:kotlin-stdlib:2.2.0",
    "org.jetbrains.kotlin:kotlin-reflect:2.2.0",
    "org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1",
    "org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1",
    "com.github.getActivity:XXPermissions:18.63",
];

#[derive(Debug, Clone)]
struct AndroidBuildEnvironment {
    gradle_bin: PathBuf,
    java_home: PathBuf,
    android_home: PathBuf,
    gradle_user_home: PathBuf,
}

#[derive(Default)]
struct AndroidManifestPatches {
    permissions: String,
    application_entries: String,
    pandora_entry_intent_filters: String,
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

fn emit_log(window: &tauri::Window, level: &str, message: &str, progress: Option<u8>) {
    let event = BuildLogEvent {
        build_id: None,
        platform: "android".to_string(),
        level: level.to_string(),
        message: message.to_string(),
        progress,
    };
    let _ = window.emit("build-log", event);
}

fn bundled_android_template() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    while path.pop() {
        let template = path.join("bundled").join("android-template-v2");
        if template.exists() {
            return template;
        }
    }
    PathBuf::from("bundled").join("android-template-v2")
}

#[tauri::command]
pub async fn prepare_android_build(options: AndroidBuildOptions) -> Result<BuildResult, String> {
    let env = resolve_android_build_environment()?;
    let project_dir = Path::new(&options.project_path);
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
            "[prepare] Android build environment verified".to_string(),
            format!("[prepare] Gradle: {}", env.gradle_bin.display()),
            format!("[prepare] JAVA_HOME: {}", env.java_home.display()),
            format!("[prepare] ANDROID_HOME: {}", env.android_home.display()),
        ],
        duration_ms: 0,
        error: None,
    })
}

#[tauri::command]
pub async fn run_android_build(
    options: AndroidBuildOptions,
    app_handle: tauri::AppHandle,
) -> Result<BuildResult, String> {
    let start = std::time::Instant::now();
    let project_dir = Path::new(&options.project_path);
    let env = resolve_android_build_environment()?;
    let mut args = Vec::new();
    if options.clean.unwrap_or(false) {
        args.push("clean".to_string());
    }
    args.push(format!(
        "assemble{}",
        options.variant.unwrap_or_else(|| "Debug".to_string())
    ));
    if let Some(extra) = options.extra_args {
        args.extend(extra);
    }
    let output = crate::utils::process::run_command_streaming_with_env(
        &env.gradle_bin.to_string_lossy(),
        &args,
        &project_dir.to_string_lossy(),
        &android_process_env(&env),
        app_handle,
        "android-build",
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(BuildResult {
        success: output.success,
        output_path: None,
        logs: output.logs,
        duration_ms: start.elapsed().as_millis() as u64,
        error: (!output.success).then(|| "Build failed".to_string()),
    })
}

#[tauri::command]
pub async fn build_android_apk(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    module_config: Option<HashMap<String, String>>,
    window: tauri::Window,
    _state: tauri::State<'_, AppState>,
) -> Result<BuildArtifact, String> {
    let build_id = build_id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("android-{}", timestamp()));
    emit_log(&window, "info", "开始 Android APK 构建流程", Some(2));

    let config = crate::commands::project::load_project_config_sync(&project_id)?;
    let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    validate_android_config(&config, &sdk_config)?;
    let android_env = resolve_android_build_environment()?;

    let resource_dir = PathBuf::from(&resource_path);
    if !resource_dir.exists() {
        return Err(format!("资源路径不存在: {}", resource_path));
    }
    let scan =
        crate::commands::resource::scan_imported_resource(&resource_dir, &resource_dir, false)?;
    let app_resource_dir = PathBuf::from(&scan.app_resource_path);
    emit_log(
        &window,
        "info",
        &format!("检测到 UniApp AppId: {}", scan.app_id),
        Some(5),
    );

    let workspace_base = crate::utils::fs::get_project_config_dir(&project_id).join("workspace");
    crate::utils::fs::ensure_directory(&workspace_base)
        .map_err(|e| format!("创建工作区基础目录失败: {}", e))?;
    let workspace = workspace_base.join(safe_file_name(&build_id));
    let template_dir = bundled_android_template();
    if !template_dir.exists() {
        return Err(format!(
            "Android 模板目录不存在: {}",
            template_dir.display()
        ));
    }
    crate::utils::fs::copy_recursive(&template_dir, &workspace)
        .map_err(|e| format!("复制 Android 模板失败: {}", e))?;
    emit_log(&window, "success", "模板已复制到工作区", Some(10));

    let sdk_layout = crate::commands::sdk::resolve_android_sdk_layout(&PathBuf::from(
        &sdk_config.dcloud_android_sdk_path,
    ))?;
    let sdk_libs = sdk_layout.libs_dir.clone();
    let libs_dst = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("libs");
    crate::utils::fs::ensure_directory(&libs_dst).map_err(|e| e.to_string())?;
    copy_required_aars(&sdk_libs, &libs_dst, &window)?;
    emit_log(&window, "success", "DCloud SDK 基础 AAR 已注入", Some(18));

    let manifest_modules = manifest_info
        .as_ref()
        .map(|info| info.detected_modules.as_slice())
        .unwrap_or(scan.detected_modules.as_slice());
    let merged_module_config = merged_android_module_config(&config, module_config);
    let module_config_report = manifest_info.as_ref().map(|info| {
        crate::commands::module::analyze_android_module_config_sync(
            info,
            merged_module_config.as_ref(),
        )
    });
    let manifest_value = manifest_info.as_ref().and_then(|info| {
        std::fs::read_to_string(&info.manifest_path)
            .ok()
            .and_then(|content| json5::from_str::<serde_json::Value>(&content).ok())
    });
    if let Some(report) = &module_config_report {
        if !report.all_configured {
            let missing = report
                .missing_required
                .iter()
                .map(|item| format!("{}: {}", item.module_name, item.label))
                .collect::<Vec<_>>()
                .join("；");
            return Err(format!("Android 模块配置未填写完整: {}", missing));
        }
        emit_android_module_config_report(&window, report);
    }
    let mut extra_deps = BTreeSet::new();
    let mut extra_repos = BTreeSet::new();
    let mut plugin_project_deps = BTreeSet::new();
    let mut plugin_includes = BTreeSet::new();

    if scan.uts.has_uts_plugins {
        extra_repos.insert("maven { url 'https://jitpack.io' }".to_string());
        for dep in UTS_RUNTIME_DEPS {
            extra_deps.insert((*dep).to_string());
        }
        copy_optional_aar(&sdk_libs, &libs_dst, "utsplugin-release.aar", &window)?;

        for module in &scan.uts.builtin_modules {
            copy_optional_aar(&sdk_libs, &libs_dst, &module.local_aar, &window)?;
            for dep in &module.online_deps {
                extra_deps.insert(dep.clone());
            }
        }

        let custom_root = workspace.join("uts-modules");
        for plugin in &scan.uts.custom_plugins {
            if let Some(android_dir) = &plugin.android_dir {
                let module_dir = custom_root.join(&plugin.id);
                crate::utils::fs::copy_recursive(Path::new(android_dir), &module_dir)
                    .map_err(|e| format!("复制 UTS 插件 {} 失败: {}", plugin.id, e))?;
                if module_dir.join("build.gradle").exists()
                    || module_dir.join("build.gradle.kts").exists()
                {
                    plugin_includes.insert(format!(
                        "include ':{0}'\nproject(':{0}').projectDir = file('uts-modules/{0}')",
                        plugin.id
                    ));
                    plugin_project_deps.insert(format!("implementation project(':{}')", plugin.id));
                }
                copy_custom_android_libs(&plugin.id, Path::new(android_dir), &libs_dst, &window)?;
                for dep in &plugin.android_deps {
                    extra_deps.insert(dep.clone());
                }
            }
        }
        emit_log(&window, "success", "UTS 插件依赖已扫描并注入", Some(26));
    }

    copy_sdk_assets(&sdk_layout.assets_dir, &workspace, &window)?;
    apply_android_manifest_modules(
        manifest_modules,
        module_config_report.as_ref(),
        manifest_value.as_ref(),
        &sdk_libs,
        &libs_dst,
        &workspace,
        &mut extra_repos,
        &mut extra_deps,
        &window,
    )?;

    let manifest_patches = render_android_module_manifest_patches(
        module_config_report.as_ref(),
        &config.android.package_name,
        &scan.app_id,
    );
    let manifest_placeholders = render_android_module_manifest_placeholders(
        module_config_report.as_ref(),
        manifest_modules,
        &config.android.package_name,
    );

    let store_key = format!("{}-android-store-password", config.id);
    let key_key = format!("{}-android-key-password", config.id);
    let store_password = crate::utils::keychain::get_password(&store_key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Keychain 中缺少 Android Store 密码".to_string())?;
    let key_password = crate::utils::keychain::get_password(&key_key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Keychain 中缺少 Android Key 密码".to_string())?;

    let modification_ctx = crate::utils::android_project_mod::BuildModificationContext {
        project_name: safe_file_name(&config.name),
        package_name: config.android.package_name.clone(),
        appid: scan.app_id.clone(),
        dcloud_appkey: config.android.dcloud_app_key.clone(),
        app_name: config.app.name.clone(),
        version_code: config.app.version_code,
        version_name: config.app.version.clone(),
        compile_sdk: config.android.compile_sdk_version,
        target_sdk: config.android.target_sdk_version,
        min_sdk: config.android.min_sdk_version,
        keystore_path: config.android.keystore.path.clone(),
        key_alias: config.android.keystore.alias.clone(),
        key_password,
        store_password,
        android_allow_backup: if android_build_requires_allow_backup_false(&extra_deps) {
            "false".to_string()
        } else {
            "true".to_string()
        },
        extra_repositories: extra_repos.into_iter().collect(),
        extra_dependencies: extra_deps
            .iter()
            .map(|dep| render_gradle_dependency_line(dep))
            .collect(),
        plugin_includes: plugin_includes.into_iter().collect(),
        plugin_project_dependencies: plugin_project_deps.into_iter().collect(),
        module_permissions: manifest_patches
            .permissions
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.trim().to_string())
            .collect(),
        module_application_entries: manifest_patches
            .application_entries
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        module_pandora_entry_intent_filters: manifest_patches
            .pandora_entry_intent_filters
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        manifest_placeholders,
        dependency_excludes: render_dependency_excludes(
            &extra_deps
                .iter()
                .map(|dep| render_gradle_dependency_line(dep))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    };

    let modifier = crate::utils::android_project_mod::AndroidProjectModifier::new(
        template_dir,
        workspace.clone(),
    )?;
    modifier.apply_all_modifications(&modification_ctx)?;
    emit_log(
        &window,
        "success",
        "Android 工程配置已通过 V2 修改器应用",
        Some(38),
    );

    import_uniapp_assets(&app_resource_dir, &workspace, &scan.app_id)?;
    emit_log(
        &window,
        "success",
        "UniApp 资源已导入 assets/apps",
        Some(48),
    );

    update_dcloud_control(&workspace, &scan.app_id)?;
    emit_log(&window, "success", "dcloud_control.xml 已更新", Some(55));

    generate_icons(&config, &workspace, &window)?;
    emit_log(&window, "success", "Android 图标已生成", Some(64));

    let splashscreen = manifest_info
        .as_ref()
        .and_then(|info| info.splashscreen.as_ref())
        .or(scan.splashscreen.as_ref());
    apply_android_splashscreen(splashscreen, &workspace, &window)?;
    emit_log(&window, "success", "Android 启动图已处理", Some(66));

    emit_log(
        &window,
        "info",
        "执行 Gradle assembleRelease --stacktrace",
        Some(70),
    );
    let app_handle = window.app_handle().clone();
    let output = crate::utils::process::run_command_streaming_with_env(
        &android_env.gradle_bin.to_string_lossy(),
        &["assembleRelease".to_string(), "--stacktrace".to_string()],
        &workspace.to_string_lossy(),
        &android_process_env(&android_env),
        app_handle,
        "build-log",
    )
    .await
    .map_err(|e| format!("执行 Gradle 失败: {}", e))?;
    if !output.success {
        return Err(format!("Gradle 构建失败，退出码: {:?}", output.exit_code));
    }

    let apk = find_apk_in_workspace(&workspace)
        .into_iter()
        .next()
        .ok_or_else(|| "Gradle 成功结束，但未找到 APK 产物".to_string())?;
    let output_dir = expand_home(&config.output_dir);
    crate::utils::fs::ensure_directory(&output_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;
    let ts = timestamp();
    let dest = output_dir.join(format!("{}-v{}.apk", ts, config.app.version));
    std::fs::copy(&apk, &dest).map_err(|e| format!("复制 APK 到输出目录失败: {}", e))?;
    let size_bytes = std::fs::metadata(&dest)
        .map(|m| m.len())
        .unwrap_or_default();
    emit_log(
        &window,
        "success",
        &format!("Android 打包完成: {}", dest.display()),
        Some(100),
    );

    Ok(BuildArtifact {
        platform: "android".to_string(),
        path: dest.to_string_lossy().to_string(),
        file_name: dest
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app.apk")
            .to_string(),
        size_bytes,
        build_id,
    })
}

fn validate_android_config(
    config: &crate::commands::project::ProjectConfig,
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<(), String> {
    if sdk_config.dcloud_android_sdk_path.trim().is_empty() {
        return Err("请先在 SDK & 环境管理中配置 DCloud Android 离线 SDK 路径".to_string());
    }
    if config.android.package_name.trim().is_empty() {
        return Err("请先配置 Android 包名".to_string());
    }
    if config.android.dcloud_app_key.trim().is_empty() {
        return Err("请先配置 Android DCloud AppKey".to_string());
    }
    if config.android.keystore.path.trim().is_empty()
        || config.android.keystore.alias.trim().is_empty()
        || !config.android.keystore.has_store_password
        || !config.android.keystore.has_key_password
    {
        return Err(
            "Android release 构建需要完整 Keystore 路径、Alias、Store 密码和 Key 密码".to_string(),
        );
    }
    Ok(())
}

fn merged_android_module_config(
    project: &crate::commands::project::ProjectConfig,
    module_config: Option<HashMap<String, String>>,
) -> Option<HashMap<String, String>> {
    let mut merged = project.android_module_config.clone();
    if let Some(module_config) = module_config {
        for (key, value) in module_config {
            if value.trim().is_empty() {
                merged.remove(&key);
            } else {
                merged.insert(key, value);
            }
        }
    }
    (!merged.is_empty()).then_some(merged)
}

fn copy_required_aars(
    sdk_libs: &Path,
    libs_dst: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    for requirement in crate::commands::sdk::ANDROID_REQUIRED_AARS {
        let src = crate::commands::sdk::resolve_android_required_aar(sdk_libs, requirement)
            .ok_or_else(|| format!("SDK 缺少必需 AAR: {}", requirement.display_name))?;
        let file_name = src
            .file_name()
            .ok_or_else(|| format!("SDK AAR 文件名无效: {}", src.display()))?;
        crate::utils::fs::copy_file(&src, &libs_dst.join(file_name))
            .map_err(|e| format!("复制 {} 失败: {}", requirement.display_name, e))?;
    }
    emit_log(
        window,
        "info",
        &format!(
            "已复制 {} 个基础 AAR",
            crate::commands::sdk::ANDROID_REQUIRED_AARS.len()
        ),
        None,
    );
    Ok(())
}

fn copy_optional_aar(
    sdk_libs: &Path,
    libs_dst: &Path,
    aar_name: &str,
    window: &tauri::Window,
) -> Result<(), String> {
    let src = sdk_libs.join(aar_name);
    if src.exists() {
        crate::utils::fs::copy_file(&src, &libs_dst.join(aar_name))
            .map_err(|e| format!("复制 {} 失败: {}", aar_name, e))?;
    } else {
        emit_log(
            window,
            "warn",
            &format!("SDK 中未找到可选 AAR: {}", aar_name),
            None,
        );
    }
    Ok(())
}

fn apply_android_manifest_modules(
    modules: &[crate::commands::resource::DetectedModule],
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    manifest: Option<&serde_json::Value>,
    sdk_libs: &Path,
    libs_dst: &Path,
    workspace: &Path,
    extra_repos: &mut BTreeSet<String>,
    extra_deps: &mut BTreeSet<String>,
    window: &tauri::Window,
) -> Result<(), String> {
    let supported = modules
        .iter()
        .filter(|module| android_module_template_key(&module.name).is_some())
        .collect::<Vec<_>>();
    if supported.is_empty() {
        emit_log(
            window,
            "info",
            "manifest 中未检测到需要迁移的 Android UniApp 模块",
            None,
        );
        return Ok(());
    }

    let config = module_config_tree_for_android_build(modules, config_report);
    let properties_path = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/assets/data/dcloud_properties.xml");
    crate::commands::module::generate_dcloud_properties(&properties_path, &config)?;

    for module in supported {
        emit_log(
            window,
            "info",
            &format!("检测到 {} 模块", module.name),
            None,
        );
        emit_log(
            window,
            "info",
            &format!("开始打包 {} 模块", module.name),
            None,
        );
        emit_log(
            window,
            "info",
            &format!("开始拷贝 {} 模块涉及的 aar 文件", module.name),
            None,
        );

        let template_key = android_module_template_key(&module.name)
            .expect("supported modules are filtered by template key");
        let template = crate::commands::module::get_module_template_sync(template_key)?;
        copy_android_module_artifacts(
            &module.name,
            template_key,
            &template.android_config.required_aars,
            manifest,
            sdk_libs,
            libs_dst,
            window,
        )?;
        for repo in crate::commands::module::android_module_gradle_repositories_for_manifest(
            template_key,
            manifest,
        ) {
            extra_repos.insert(repo.to_string());
        }
        for dep in android_gradle_dependencies(
            template_key,
            &template.android_config.gradle_dependencies,
            manifest,
        ) {
            extra_deps.insert(dep);
        }

        emit_log(
            window,
            "info",
            &format!("{} 模块涉及的 aar 文件 拷贝完成", module.name),
            None,
        );
        emit_log(
            window,
            "info",
            &format!("开始修改 {} 模块需要的配置项", module.name),
            None,
        );
        emit_log(
            window,
            "success",
            &format!("{} 模块配置/打包完成", module.name),
            None,
        );
    }
    Ok(())
}

fn android_module_template_key(module_name: &str) -> Option<&'static str> {
    crate::commands::module::android_module_template_key(module_name)
}

fn copy_android_module_artifacts(
    module_name: &str,
    template_key: &str,
    required_artifacts: &[String],
    manifest: Option<&serde_json::Value>,
    sdk_libs: &Path,
    libs_dst: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let mut copied = 0usize;
    for artifact in required_artifacts {
        if !crate::commands::module::android_module_artifact_enabled_for_manifest(
            template_key,
            artifact,
            manifest,
        ) {
            continue;
        }
        let Some(pattern) = clean_android_artifact_pattern(artifact) else {
            continue;
        };
        let Some(src) = find_android_sdk_artifact(sdk_libs, &pattern) else {
            emit_log(
                window,
                "warn",
                &format!("{} 模块未找到可选依赖文件: {}", module_name, pattern),
                None,
            );
            continue;
        };
        let Some(file_name) = src.file_name() else {
            continue;
        };
        crate::utils::fs::copy_file(&src, &libs_dst.join(file_name)).map_err(|e| {
            format!(
                "复制 {} 模块依赖 {} 失败: {}",
                module_name,
                src.display(),
                e
            )
        })?;
        copied += 1;
    }
    emit_log(
        window,
        "info",
        &format!("{} 模块已复制 {} 个本地依赖", module_name, copied),
        None,
    );
    Ok(())
}

fn clean_android_artifact_pattern(raw: &str) -> Option<String> {
    let name = raw.split_whitespace().next()?.trim();
    if name.is_empty() || name.starts_with('(') {
        return None;
    }
    if !(name.ends_with(".aar") || name.ends_with(".jar")) {
        return None;
    }
    Some(name.to_string())
}

fn find_android_sdk_artifact(sdk_libs: &Path, artifact_pattern: &str) -> Option<PathBuf> {
    let direct = sdk_libs.join(artifact_pattern);
    if direct.exists() {
        return Some(direct);
    }
    let stem = android_artifact_search_stem(artifact_pattern);
    let mut matches = std::fs::read_dir(sdk_libs)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let ext = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default();
            if ext != "aar" && ext != "jar" {
                return false;
            }
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            crate::commands::sdk::android_artifact_name_matches(artifact_pattern, file_name)
                || (!stem.is_empty() && (file_name.starts_with(&stem) || file_name.contains(&stem)))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

fn android_artifact_search_stem(pattern: &str) -> String {
    crate::commands::sdk::android_artifact_versionless_stem(pattern)
}

fn android_gradle_dependencies(
    template_key: &str,
    raw_deps: &[String],
    manifest: Option<&serde_json::Value>,
) -> Vec<String> {
    raw_deps
        .iter()
        .filter(|dep| {
            crate::commands::module::android_module_gradle_dependency_enabled_for_manifest(
                template_key,
                dep,
                manifest,
            )
        })
        .filter_map(|dep| dep.split_whitespace().next())
        .map(str::trim)
        .filter(|dep| dep.matches(':').count() >= 2)
        .map(ToString::to_string)
        .collect()
}

fn emit_android_module_config_report(
    window: &tauri::Window,
    report: &crate::commands::module::AndroidModuleConfigReport,
) {
    if report.modules.is_empty() {
        return;
    }

    emit_log(
        window,
        "info",
        &format!(
            "Android 模块配置清单: {} 个模块需要配置项",
            report.modules.len()
        ),
        None,
    );
    for module in &report.modules {
        emit_log(
            window,
            "info",
            &format!("模块 {} 需要配置 {} 项", module.name, module.fields.len()),
            None,
        );
        for field in &module.fields {
            let status = if field
                .value
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
            {
                match field.value_source.as_deref() {
                    Some("manifest") => "已从 manifest 读取",
                    Some("user") => "已在构建中心填写",
                    _ => "已填写",
                }
            } else if field.required {
                "缺失必填"
            } else {
                "未填写可选"
            };
            emit_log(
                window,
                if field.required && status == "缺失必填" {
                    "warn"
                } else {
                    "info"
                },
                &format!("  - {} ({})", field.label, status),
                None,
            );
        }
    }
}

fn module_config_tree_for_android_build(
    modules: &[crate::commands::resource::DetectedModule],
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
) -> crate::commands::module::ModuleConfigTree {
    let mut tree = crate::commands::module::module_config_from_detected_modules(modules);
    let Some(report) = report else {
        return tree;
    };

    if let Some(ref mut push) = tree.push {
        push.unipush_appid = report_value(report, "push", "GETUI_APPID");
        push.unipush_appkey = report_value(report, "push", "plus.unipush.appkey");
        push.unipush_appsecret = report_value(report, "push", "plus.unipush.appsecret");
    }

    if let Some(ref mut geolocation) = tree.geolocation {
        geolocation.baidu_ak = report_value(report, "geolocation", "BAIDU_MAP_AK");
        geolocation.amap_key = report_value(report, "geolocation", "AMAP_KEY");
        geolocation.engine = if geolocation.baidu_ak.is_some() {
            "baidu".to_string()
        } else if geolocation.amap_key.is_some() {
            "amap".to_string()
        } else {
            "system".to_string()
        };
    }

    if let Some(ref mut share) = tree.share {
        if let Some(wx_appid) = report_value(report, "share", "WX_APPID") {
            let mut value = HashMap::new();
            value.insert("WX_APPID".to_string(), wx_appid);
            if let Some(secret) = report_value(report, "share", "WX_SECRET") {
                value.insert("WX_SECRET".to_string(), secret);
            }
            share.weixin = Some(value);
        }
        if let Some(qq_appid) = report_value(report, "share", "QQ_APPID") {
            share.qq = Some(HashMap::from([("QQ_APPID".to_string(), qq_appid)]));
        }
        if let Some(sina_appkey) = report_value(report, "share", "SINA_APPKEY") {
            let mut value = HashMap::from([("SINA_APPKEY".to_string(), sina_appkey)]);
            if let Some(secret) = report_value(report, "share", "SINA_SECRET") {
                value.insert("SINA_SECRET".to_string(), secret);
            }
            if let Some(uri) = report_value(report, "share", "SINA_REDIRECT_URI") {
                value.insert("SINA_REDIRECT_URI".to_string(), uri);
            }
            share.sina = Some(value);
        }
    }

    if let Some(ref mut login) = tree.login {
        let mut providers = Vec::new();
        if let Some(wx_appid) = report_value(report, "login", "WX_APPID") {
            let mut config = HashMap::from([("WX_APPID".to_string(), wx_appid)]);
            if let Some(secret) = report_value(report, "login", "WX_SECRET") {
                config.insert("WX_SECRET".to_string(), secret);
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "weixin".to_string(),
                enabled: true,
                config,
            });
        }
        if let Some(qq_appid) = report_value(report, "login", "QQ_APPID") {
            providers.push(crate::commands::module::LoginProvider {
                name: "qq".to_string(),
                enabled: true,
                config: HashMap::from([("QQ_APPID".to_string(), qq_appid)]),
            });
        }
        if let Some(gy_appid) = report_value(report, "login", "GY_APP_ID") {
            providers.push(crate::commands::module::LoginProvider {
                name: "univerify".to_string(),
                enabled: true,
                config: HashMap::from([("GY_APP_ID".to_string(), gy_appid)]),
            });
        }
        if !providers.is_empty() {
            login.providers = providers;
        }
    }

    if let Some(ref mut payment) = tree.payment {
        if let Some(wx_appid) = report_value(report, "payment", "WX_APPID") {
            payment.weixin = Some(HashMap::from([("WX_APPID".to_string(), wx_appid)]));
        }
    }

    if let Some(ref mut map) = tree.map {
        map.baidu_map_ak = report_value(report, "map", "BAIDU_MAP_AK");
        map.amap_key = report_value(report, "map", "AMAP_KEY");
        map.tencent_map_key = report_value(report, "map", "TENCENT_MAP_KEY");
        map.google_maps_api_key = report_value(report, "map", "GOOGLE_MAPS_API_KEY");
        map.engine = if map.baidu_map_ak.is_some() {
            "baidu".to_string()
        } else if map.tencent_map_key.is_some() {
            "tencent".to_string()
        } else if map.google_maps_api_key.is_some() {
            "google".to_string()
        } else {
            "amap".to_string()
        };
    }

    if let Some(ref mut statistic) = tree.statistic {
        if let Some(appkey) = report_value(report, "statistic", "UMENG_APPKEY") {
            statistic.umeng = Some(HashMap::from([
                ("UMENG_APPKEY".to_string(), appkey),
                (
                    "UMENG_CHANNEL".to_string(),
                    report_value(report, "statistic", "UMENG_CHANNEL").unwrap_or_default(),
                ),
            ]));
            statistic.provider = "umeng".to_string();
        }
    }

    if let Some(ref mut livepusher) = tree.livepusher {
        livepusher.license_url = report_value(report, "livepusher", "LIVEPUSH_LICENSE_URL");
        livepusher.license_key = report_value(report, "livepusher", "LIVEPUSH_LICENSE_KEY");
    }

    tree
}

fn report_value(
    report: &crate::commands::module::AndroidModuleConfigReport,
    template_key: &str,
    key: &str,
) -> Option<String> {
    report
        .modules
        .iter()
        .find(|module| module.template_key == template_key)
        .and_then(|module| module.fields.iter().find(|field| field.key == key))
        .and_then(|field| field.value.clone())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn copy_custom_android_libs(
    plugin_id: &str,
    android_dir: &Path,
    libs_dst: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let mut copied = 0usize;
    for ext in ["aar", "jar"] {
        for src in crate::utils::fs::find_files_by_extension(android_dir, ext)
            .map_err(|e| format!("扫描 UTS 插件 {} 本地依赖失败: {}", plugin_id, e))?
        {
            let Some(file_name) = src.file_name() else {
                continue;
            };
            crate::utils::fs::copy_file(&src, &libs_dst.join(file_name))
                .map_err(|e| format!("复制 UTS 插件 {} 本地依赖失败: {}", plugin_id, e))?;
            copied += 1;
        }
    }
    if copied > 0 {
        emit_log(
            window,
            "info",
            &format!("UTS 插件 {} 已复制 {} 个本地依赖", plugin_id, copied),
            None,
        );
    }
    Ok(())
}

fn copy_sdk_assets(
    sdk_assets: &Path,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let src = sdk_assets.join("data");
    let dst = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/assets/data");
    if src.exists() {
        crate::utils::fs::copy_recursive(&src, &dst)
            .map_err(|e| format!("复制 SDK assets/data 失败: {}", e))?;
    } else {
        emit_log(
            window,
            "warn",
            &format!("SDK assets/data 不存在: {}", src.display()),
            None,
        );
    }
    Ok(())
}

fn android_build_requires_allow_backup_false(extra_dependencies: &BTreeSet<String>) -> bool {
    extra_dependencies
        .iter()
        .any(|dep| dep == "com.getui:gysdk:3.1.7.0")
}

fn render_android_module_manifest_placeholders(
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    detected_modules: &[crate::commands::resource::DetectedModule],
    package_name: &str,
) -> String {
    let mut entries = BTreeMap::new();
    let mut has_push_module = detected_modules
        .iter()
        .any(|module| android_module_template_key(&module.name) == Some("push"));

    if let Some(report) = report {
        for module in &report.modules {
            if module.template_key == "push" {
                has_push_module = true;
            }
            for field in &module.fields {
                if let Some(value) = field
                    .value
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    insert_manifest_placeholder_entries(&mut entries, &field.key, value);
                }
            }
        }
    }

    if has_push_module {
        insert_push_manifest_placeholder_defaults(&mut entries, package_name);
    }

    if entries.is_empty() {
        return String::new();
    }

    let entries = entries
        .into_iter()
        .map(|(key, value)| {
            format!(
                "            \"{}\": \"{}\"",
                key,
                escape_gradle_string(&value)
            )
        })
        .collect::<Vec<_>>();
    format!(
        "\n        manifestPlaceholders = [\n{}\n        ]",
        entries.join(",\n")
    )
}

fn insert_manifest_placeholder_entries(
    target: &mut BTreeMap<String, String>,
    key: &str,
    value: &str,
) {
    for placeholder in manifest_placeholder_aliases(key) {
        target.insert(placeholder.to_string(), value.to_string());
    }
}

fn insert_push_manifest_placeholder_defaults(
    target: &mut BTreeMap<String, String>,
    package_name: &str,
) {
    for (placeholder, value) in [
        ("GETUI_APPID", ""),
        ("PUSH_APPID", ""),
        ("plus.unipush.appid", ""),
        ("plus.unipush.appkey", ""),
        ("PUSH_APPKEY", ""),
        ("plus.unipush.appsecret", ""),
        ("PUSH_APPSECRET", ""),
        ("apk.applicationId", package_name),
        ("XIAOMI_APP_ID", ""),
        ("XIAOMI_APP_KEY", ""),
        ("MEIZU_APP_ID", ""),
        ("MEIZU_APP_KEY", ""),
        ("HUAWEI_APP_ID", ""),
        ("OPPO_APP_KEY", ""),
        ("OPPO_APP_SECRET", ""),
        ("VIVO_APP_ID", ""),
        ("VIVO_APP_KEY", ""),
        ("HONOR_APP_ID", ""),
    ] {
        target
            .entry(placeholder.to_string())
            .or_insert_with(|| value.to_string());
    }
}

fn manifest_placeholder_aliases(key: &str) -> Vec<&str> {
    match key {
        "GETUI_APPID" => vec!["GETUI_APPID", "PUSH_APPID", "plus.unipush.appid"],
        "plus.unipush.appkey" => vec!["plus.unipush.appkey", "PUSH_APPKEY"],
        "plus.unipush.appsecret" => vec!["plus.unipush.appsecret", "PUSH_APPSECRET"],
        _ => vec![key],
    }
}

fn render_android_module_manifest_patches(
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    package_name: &str,
    app_id: &str,
) -> AndroidManifestPatches {
    let Some(report) = report else {
        return AndroidManifestPatches::default();
    };

    let mut permissions = BTreeSet::new();
    let mut application_entries = BTreeSet::new();
    let mut pandora_entry_intent_filters = BTreeSet::new();

    for module in &report.modules {
        let placeholders = module_placeholders(module);
        match module.template_key.as_str() {
            "push" => {
                add_application_entries(
                    &mut application_entries,
                    &[
                        meta_data(
                            "GETUI_APPID",
                            &placeholder_value(&placeholders, "GETUI_APPID"),
                        ),
                        meta_data(
                            "plus.unipush.appid",
                            &placeholder_value(&placeholders, "plus.unipush.appid"),
                        ),
                        meta_data(
                            "plus.unipush.appkey",
                            &placeholder_value(&placeholders, "plus.unipush.appkey"),
                        ),
                        meta_data(
                            "plus.unipush.appsecret",
                            &placeholder_value(&placeholders, "plus.unipush.appsecret"),
                        ),
                    ],
                );
                pandora_entry_intent_filters.insert(indent_manifest_fragment(
                    r#"<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:host="io.dcloud.unipush" android:path="/" android:scheme="unipush" />
</intent-filter>"#,
                    12,
                ));
            }
            "geolocation" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.CHANGE_WIFI_STATE",
                        "android.permission.MOUNT_UNMOUNT_FILESYSTEMS",
                        "android.permission.READ_LOGS",
                        "android.permission.WRITE_SETTINGS",
                        "android.permission.ACCESS_BACKGROUND_LOCATION",
                        "android.permission.FOREGROUND_SERVICE",
                    ],
                );
                if has_report_value(module, "BAIDU_MAP_AK") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "com.baidu.lbsapi.API_KEY",
                                &placeholder_value(&placeholders, "BAIDU_MAP_AK"),
                            ),
                            service_entry(
                                r#"<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote" />"#,
                            ),
                        ],
                    );
                }
                if has_report_value(module, "AMAP_KEY") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "com.amap.api.v2.apikey",
                                &placeholder_value(&placeholders, "AMAP_KEY"),
                            ),
                            service_entry(
                                r#"<service android:name="com.amap.api.location.APSService" />"#,
                            ),
                        ],
                    );
                }
                if has_report_value(module, "TENCENT_MAP_KEY") {
                    add_application_entries(
                        &mut application_entries,
                        &[meta_data(
                            "TencentMapSDK",
                            &placeholder_value(&placeholders, "TENCENT_MAP_KEY"),
                        )],
                    );
                }
            }
            "share" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.MODIFY_AUDIO_SETTINGS",
                        "android.permission.CHANGE_WIFI_STATE",
                    ],
                );
                if has_report_value(module, "WX_APPID") {
                    let wx_appid = placeholder_value(&placeholders, "WX_APPID");
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data("WX_APPID", &wx_appid),
                            meta_data("WX_SECRET", &placeholder_value(&placeholders, "WX_SECRET")),
                            wx_entry_activity(package_name, &wx_appid),
                        ],
                    );
                }
                if has_report_value(module, "QQ_APPID") {
                    let qq_appid = placeholder_value(&placeholders, "QQ_APPID");
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data("QQ_APPID", &qq_appid),
                            qq_auth_activity(&qq_appid),
                            qq_assist_activity(),
                        ],
                    );
                }
                if has_report_value(module, "SINA_APPKEY") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "SINA_APPKEY",
                                &placeholder_value(&placeholders, "SINA_APPKEY"),
                            ),
                            meta_data(
                                "SINA_SECRET",
                                &placeholder_value(&placeholders, "SINA_SECRET"),
                            ),
                            meta_data(
                                "SINA_REDIRECT_URI",
                                &placeholder_value(&placeholders, "SINA_REDIRECT_URI"),
                            ),
                            service_entry(
                                r#"<activity android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity" android:configChanges="keyboardHidden|orientation" android:exported="false" android:windowSoftInputMode="adjustResize" />"#,
                            ),
                            service_entry(
                                r#"<activity android:name="com.sina.weibo.sdk.share.WbShareTransActivity" android:launchMode="singleTask" android:theme="@android:style/Theme.Translucent.NoTitleBar.Fullscreen" />"#,
                            ),
                        ],
                    );
                }
            }
            "login" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.MODIFY_AUDIO_SETTINGS",
                        "com.xiaomi.permission.AUTH_SERVICE",
                    ],
                );
                if has_report_value(module, "WX_APPID") {
                    let wx_appid = placeholder_value(&placeholders, "WX_APPID");
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data("WX_APPID", &wx_appid),
                            meta_data("WX_SECRET", &placeholder_value(&placeholders, "WX_SECRET")),
                            wx_entry_activity(package_name, &wx_appid),
                        ],
                    );
                }
                if has_report_value(module, "QQ_APPID") {
                    let qq_appid = placeholder_value(&placeholders, "QQ_APPID");
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data("QQ_APPID", &qq_appid),
                            qq_auth_activity(&qq_appid),
                            qq_assist_activity(),
                        ],
                    );
                }
                if has_report_value(module, "GY_APP_ID") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "GETUI_APPID",
                                &placeholder_value(&placeholders, "GETUI_APPID"),
                            ),
                            meta_data("GY_APP_ID", &placeholder_value(&placeholders, "GY_APP_ID")),
                        ],
                    );
                }
                if has_report_value(module, "SINA_APPKEY") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "SINA_APPKEY",
                                &placeholder_value(&placeholders, "SINA_APPKEY"),
                            ),
                            meta_data(
                                "SINA_REDIRECT_URI",
                                &placeholder_value(&placeholders, "SINA_REDIRECT_URI"),
                            ),
                            service_entry(
                                r#"<activity android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity" android:configChanges="keyboardHidden|orientation" android:exported="false" android:windowSoftInputMode="adjustResize" />"#,
                            ),
                        ],
                    );
                }
                if has_report_value(module, "MIUI_APPID") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "MIUI_APPID",
                                &placeholder_value(&placeholders, "MIUI_APPID"),
                            ),
                            meta_data(
                                "MIUI_APPSECRET",
                                &placeholder_value(&placeholders, "MIUI_APPSECRET"),
                            ),
                            meta_data(
                                "MIUI_REDIRECT_URI",
                                &placeholder_value(&placeholders, "MIUI_REDIRECT_URI"),
                            ),
                            service_entry(
                                r#"<activity android:name="com.xiaomi.account.openauth.AuthorizeActivity" />"#,
                            ),
                        ],
                    );
                }
            }
            "map" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.CHANGE_WIFI_STATE",
                        "android.permission.MOUNT_UNMOUNT_FILESYSTEMS",
                        "android.permission.READ_LOGS",
                        "android.permission.WRITE_SETTINGS",
                        "android.permission.ACCESS_LOCATION_EXTRA_COMMANDS",
                    ],
                );
                if has_report_value(module, "BAIDU_MAP_AK") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "com.baidu.lbsapi.API_KEY",
                                &placeholder_value(&placeholders, "BAIDU_MAP_AK"),
                            ),
                            service_entry(
                                r#"<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote" />"#,
                            ),
                        ],
                    );
                }
                if has_report_value(module, "AMAP_KEY") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "com.amap.api.v2.apikey",
                                &placeholder_value(&placeholders, "AMAP_KEY"),
                            ),
                            service_entry(
                                r#"<service android:name="com.amap.api.location.APSService" />"#,
                            ),
                        ],
                    );
                }
                if has_report_value(module, "GOOGLE_MAPS_API_KEY") {
                    add_application_entries(
                        &mut application_entries,
                        &[meta_data(
                            "com.google.android.geo.API_KEY",
                            &placeholder_value(&placeholders, "GOOGLE_MAPS_API_KEY"),
                        )],
                    );
                }
                if has_report_value(module, "TENCENT_MAP_KEY") {
                    add_application_entries(
                        &mut application_entries,
                        &[meta_data(
                            "TencentMapSDK",
                            &placeholder_value(&placeholders, "TENCENT_MAP_KEY"),
                        )],
                    );
                }
            }
            "payment" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.MODIFY_AUDIO_SETTINGS",
                        "android.permission.ACCESS_COARSE_LOCATION",
                    ],
                );
                if has_report_value(module, "WX_APPID") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data("WX_APPID", &placeholder_value(&placeholders, "WX_APPID")),
                            service_entry(
                                r#"<activity android:name="io.dcloud.feature.payment.weixin.WXPayProcessMeadiatorActivity" android:exported="false" android:excludeFromRecents="true" android:theme="@style/TranslucentTheme" />"#,
                            ),
                            service_entry(&format!(
                                r#"<activity android:name="{}.wxapi.WXPayEntryActivity" android:exported="true" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:launchMode="singleTop" />"#,
                                package_name
                            )),
                        ],
                    );
                }
            }
            "speech" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.CHANGE_NETWORK_STATE",
                        "android.permission.RECORD_AUDIO",
                    ],
                );
                if has_report_value(module, "BAIDU_SPEECH_APP_ID") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "com.baidu.speech.APP_ID",
                                &placeholder_value(&placeholders, "BAIDU_SPEECH_APP_ID"),
                            ),
                            meta_data(
                                "com.baidu.speech.API_KEY",
                                &placeholder_value(&placeholders, "BD_SPEECH_APIKEY"),
                            ),
                            meta_data(
                                "com.baidu.speech.SECRET_KEY",
                                &placeholder_value(&placeholders, "BD_SPEECH_SECRETKEY"),
                            ),
                            service_entry(
                                r#"<service android:name="com.baidu.speech.VoiceRecognitionService" android:exported="false" />"#,
                            ),
                        ],
                    );
                }
                if has_report_value(module, "IFLY_APPID") {
                    add_application_entries(
                        &mut application_entries,
                        &[meta_data(
                            "IFLY_APPKEY",
                            &placeholder_value(&placeholders, "IFLY_APPID"),
                        )],
                    );
                }
            }
            "statistic" => {
                add_application_entries(
                    &mut application_entries,
                    &[
                        meta_data(
                            "UMENG_APPKEY",
                            &placeholder_value(&placeholders, "UMENG_APPKEY"),
                        ),
                        meta_data(
                            "UMENG_CHANNEL",
                            &placeholder_value(&placeholders, "UMENG_CHANNEL"),
                        ),
                    ],
                );
            }
            "uni_ad" => {
                add_application_entries(
                    &mut application_entries,
                    &[
                        meta_data(
                            "DCLOUD_AD_SPLASH",
                            &placeholder_value(&placeholders, "DCLOUD_AD_SPLASH"),
                        ),
                        meta_data(
                            "DCLOUD_STREAMAPP_CHANNEL",
                            &placeholder_value(&placeholders, "DCLOUD_STREAMAPP_CHANNEL"),
                        ),
                    ],
                );
                if has_report_value(module, "DCLOUD_STREAMAPP_CHANNEL") {
                    application_entries.insert(indent_manifest_fragment(
                        &format!(
                            r#"<provider android:name="com.bytedance.sdk.openadsdk.TTFileProvider" android:authorities="{}.TTFileProvider" android:exported="false" android:grantUriPermissions="true">
    <meta-data android:name="android.support.FILE_PROVIDER_PATHS" android:resource="@xml/file_paths" />
</provider>
<provider android:name="com.bytedance.sdk.openadsdk.multipro.TTMultiProvider" android:authorities="{}.TTMultiProvider" android:exported="false" />"#,
                            package_name, package_name
                        ),
                        8,
                    ));
                }
            }
            "livepusher" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.BLUETOOTH",
                        "android.permission.CAMERA",
                        "android.permission.RECORD_AUDIO",
                        "android.permission.MODIFY_AUDIO_SETTINGS",
                    ],
                );
                if has_report_value(module, "LIVEPUSH_LICENSE_URL") {
                    add_application_entries(
                        &mut application_entries,
                        &[
                            meta_data(
                                "TXLIVE_LICENSE_URL",
                                &placeholder_value(&placeholders, "LIVEPUSH_LICENSE_URL"),
                            ),
                            meta_data(
                                "TXLIVE_LICENSE_KEY",
                                &placeholder_value(&placeholders, "LIVEPUSH_LICENSE_KEY"),
                            ),
                        ],
                    );
                }
            }
            "face_recognition" => {
                add_application_entries(
                    &mut application_entries,
                    &[meta_data(
                        "DCLOUD_LICENSE",
                        &placeholder_value(&placeholders, "DCLOUD_LICENSE"),
                    )],
                );
            }
            _ => {}
        }
    }

    if report
        .modules
        .iter()
        .any(|module| module.template_key == "uni_ad")
    {
        let fallback_channel = format!("{}|{}||default", package_name, app_id);
        application_entries.insert(format!(
            "        <!-- uni-AD 默认渠道示例: {} -->",
            fallback_channel
        ));
    }

    let permissions = permissions
        .into_iter()
        .map(|permission| format!("    <uses-permission android:name=\"{}\" />", permission))
        .collect::<Vec<_>>()
        .join("\n");
    let application_entries = application_entries
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    let pandora_entry_intent_filters = pandora_entry_intent_filters
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");

    AndroidManifestPatches {
        permissions: prefix_if_nonempty(permissions, "\n"),
        application_entries: prefix_if_nonempty(application_entries, "\n"),
        pandora_entry_intent_filters: prefix_if_nonempty(pandora_entry_intent_filters, "\n"),
    }
}

fn add_permissions(target: &mut BTreeSet<String>, permissions: &[&str]) {
    for permission in permissions {
        target.insert((*permission).to_string());
    }
}

fn add_application_entries(target: &mut BTreeSet<String>, entries: &[String]) {
    for entry in entries {
        target.insert(entry.clone());
    }
}

fn has_report_value(
    module: &crate::commands::module::AndroidModuleConfigModule,
    key: &str,
) -> bool {
    module
        .fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| field.value.as_deref())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn module_placeholders(
    module: &crate::commands::module::AndroidModuleConfigModule,
) -> HashMap<String, String> {
    module
        .fields
        .iter()
        .filter_map(|field| {
            field
                .value
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|_| (field.key.clone(), format!("${{{}}}", field.key)))
        })
        .collect()
}

fn placeholder_value(placeholders: &HashMap<String, String>, key: &str) -> String {
    placeholders.get(key).cloned().unwrap_or_default()
}

fn meta_data(name: &str, value: &str) -> String {
    indent_manifest_fragment(
        &format!(
            r#"<meta-data android:name="{}" android:value="{}" />"#,
            name, value
        ),
        8,
    )
}

fn service_entry(entry: &str) -> String {
    indent_manifest_fragment(entry, 8)
}

fn wx_entry_activity(package_name: &str, scheme: &str) -> String {
    indent_manifest_fragment(
        &format!(
            r#"<activity android:name="{}.wxapi.WXEntryActivity" android:label="@string/app_name" android:exported="true" android:launchMode="singleTop">
    <intent-filter>
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <data android:scheme="{}" />
    </intent-filter>
</activity>"#,
            package_name, scheme
        ),
        8,
    )
}

fn qq_auth_activity(scheme: &str) -> String {
    indent_manifest_fragment(
        &format!(
            r#"<activity android:name="com.tencent.tauth.AuthActivity" android:launchMode="singleTask" android:noHistory="true">
    <intent-filter>
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data android:scheme="{}" />
    </intent-filter>
</activity>"#,
            scheme
        ),
        8,
    )
}

fn qq_assist_activity() -> String {
    indent_manifest_fragment(
        r#"<activity android:name="com.tencent.connect.common.AssistActivity" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:configChanges="keyboardHidden|orientation" android:screenOrientation="behind" />"#,
        8,
    )
}

fn indent_manifest_fragment(fragment: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    fragment
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line.trim())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_gradle_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn render_gradle_dependency_line(dep: &str) -> String {
    match dep {
        "com.getui:gtsdk:3.3.7.0" => {
            "    implementation('com.getui:gtsdk:3.3.7.0'){ exclude(group: 'com.getui') }"
                .to_string()
        }
        "com.getui:gysdk:3.1.7.0" => {
            "    implementation('com.getui:gysdk:3.1.7.0'){ exclude(group: 'com.getui', module: 'gtc') }".to_string()
        }
        _ => format!("    implementation '{}'", dep),
    }
}

fn render_dependency_excludes(_extra_dependencies: &str) -> String {
    String::new()
}

fn prefix_if_nonempty(value: String, prefix: &str) -> String {
    if value.is_empty() {
        value
    } else {
        format!("{}{}", prefix, value)
    }
}

fn import_uniapp_assets(resource_dir: &Path, workspace: &Path, app_id: &str) -> Result<(), String> {
    let apps_root = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/assets/apps");
    crate::utils::fs::ensure_directory(&apps_root).map_err(|e| e.to_string())?;
    let dest = apps_root.join(app_id);
    crate::utils::fs::copy_recursive(resource_dir, &dest)
        .map_err(|e| format!("导入 UniApp 资源失败: {}", e))
}

fn update_dcloud_control(workspace: &Path, app_id: &str) -> Result<(), String> {
    let path = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/assets/data/dcloud_control.xml");
    if !path.exists() {
        return Err(format!("dcloud_control.xml 不存在: {}", path.display()));
    }
    let content = crate::utils::fs::read_file_to_string(&path).map_err(|e| e.to_string())?;
    let updated = crate::utils::xml::set_xml_attribute(&content, "/apps/app", "appid", app_id)
        .or_else(|_| crate::utils::xml::set_xml_attribute(&content, "/hbuilder", "appid", app_id))
        .map_err(|e| format!("设置 dcloud_control.xml appid 失败: {}", e))?;
    crate::utils::fs::write_string_to_file(&path, &updated).map_err(|e| e.to_string())
}

fn generate_icons(
    config: &crate::commands::project::ProjectConfig,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    if config.app.icon1024.trim().is_empty() {
        emit_log(window, "warn", "未配置 1024 图标，跳过图标生成", None);
        return Ok(());
    }
    let source = PathBuf::from(&config.app.icon1024);
    if !source.exists() {
        emit_log(
            window,
            "warn",
            &format!("图标文件不存在: {}", source.display()),
            None,
        );
        return Ok(());
    }
    let img = image::open(&source)
        .map_err(|e| format!("读取图标失败: {}", e))?
        .to_rgba8();
    let res_dir = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/res");
    for (dir, size) in [
        ("drawable-ldpi", 36),
        ("drawable-mdpi", 48),
        ("drawable-hdpi", 72),
        ("drawable-xhdpi", 96),
        ("drawable-xxhdpi", 144),
        ("drawable-xxxhdpi", 192),
    ] {
        let out_dir = res_dir.join(dir);
        crate::utils::fs::ensure_directory(&out_dir).map_err(|e| e.to_string())?;
        for name in ["icon.png", "push.png", "splash.png"] {
            let resized = image::imageops::resize(&img, size, size, image::imageops::Lanczos3);
            resized
                .save(out_dir.join(name))
                .map_err(|e| format!("写入图标失败: {}", e))?;
        }
    }
    Ok(())
}

fn apply_android_splashscreen(
    splashscreen: Option<&crate::commands::resource::SplashscreenConfig>,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let res_dir = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/res");
    let Some(config) = splashscreen else {
        write_default_android_splashscreen(&res_dir)?;
        emit_log(
            window,
            "info",
            "manifest 未配置 Android 启动图，已使用默认白底",
            None,
        );
        return Ok(());
    };

    if config.android.is_empty() {
        write_default_android_splashscreen(&res_dir)?;
        emit_log(
            window,
            "info",
            "manifest 未提供 Android 启动图密度资源，已使用默认白底",
            None,
        );
        return Ok(());
    }

    let mut copied = 0usize;
    for (density, source) in &config.android {
        let Some(drawable_dir) = android_splash_drawable_dir(density) else {
            emit_log(
                window,
                "warn",
                &format!("忽略不支持的 Android 启动图密度: {}", density),
                None,
            );
            continue;
        };
        let source_path = PathBuf::from(source);
        if !source_path.exists() {
            emit_log(
                window,
                "warn",
                &format!("Android 启动图不存在: {}", source_path.display()),
                None,
            );
            continue;
        }
        let target_name = if source_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(".9.png"))
            .unwrap_or(false)
        {
            "unipack_splash_image.9.png"
        } else {
            "unipack_splash_image.png"
        };
        let target = res_dir.join(drawable_dir).join(target_name);
        crate::utils::fs::copy_file(&source_path, &target)
            .map_err(|e| format!("复制 Android 启动图失败 {}: {}", source_path.display(), e))?;
        copied += 1;
    }

    if copied > 0 {
        write_android_window_background(&res_dir, "@drawable/unipack_splash_image")?;
        emit_log(
            window,
            "info",
            &format!("已导入 {} 张 Android 自定义启动图", copied),
            None,
        );
    } else {
        write_default_android_splashscreen(&res_dir)?;
        emit_log(
            window,
            "warn",
            "Android 自定义启动图均未找到，已使用默认白底",
            None,
        );
    }
    Ok(())
}

fn android_splash_drawable_dir(density: &str) -> Option<&'static str> {
    match density.to_ascii_lowercase().as_str() {
        "ldpi" => Some("drawable-ldpi"),
        "mdpi" => Some("drawable-mdpi"),
        "hdpi" => Some("drawable-hdpi"),
        "xhdpi" => Some("drawable-xhdpi"),
        "xxhdpi" => Some("drawable-xxhdpi"),
        "xxxhdpi" => Some("drawable-xxxhdpi"),
        _ => None,
    }
}

fn write_default_android_splashscreen(res_dir: &Path) -> Result<(), String> {
    write_android_window_background(res_dir, "@android:color/white")?;
    Ok(())
}

fn write_android_window_background(res_dir: &Path, drawable_ref: &str) -> Result<(), String> {
    let drawable_dir = res_dir.join("drawable");
    crate::utils::fs::ensure_directory(&drawable_dir).map_err(|e| e.to_string())?;
    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<layer-list xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:drawable="{}" />
</layer-list>
"#,
        drawable_ref
    );
    crate::utils::fs::write_string_to_file(&drawable_dir.join("unipack_splash.xml"), &content)
        .map_err(|e| format!("写入 Android 启动背景资源失败: {}", e))
}

fn resolve_android_build_environment() -> Result<AndroidBuildEnvironment, String> {
    let gradle_bin = crate::commands::env::resolve_configured_tool_bin_with_candidates(
        "gradle",
        gradle_bin_names(),
    )?;
    let java_bin = crate::commands::env::resolve_configured_tool_bin_with_candidates(
        "java",
        java_bin_names(),
    )?;
    let android_home = crate::commands::env::require_configured_tool_path("android_sdk")?;
    let java_home = java_bin
        .parent()
        .and_then(|bin| bin.parent())
        .ok_or_else(|| {
            format!(
                "无法从 Java 可执行文件推导 JAVA_HOME: {}",
                java_bin.display()
            )
        })?
        .to_path_buf();
    let gradle_user_home = crate::utils::fs::get_unipack_home().join("gradle-home");
    crate::utils::fs::ensure_directory(&gradle_user_home)
        .map_err(|e| format!("创建 Gradle 用户目录失败: {}", e))?;

    Ok(AndroidBuildEnvironment {
        gradle_bin,
        java_home,
        android_home,
        gradle_user_home,
    })
}

fn android_process_env(env: &AndroidBuildEnvironment) -> Vec<(String, String)> {
    vec![
        (
            "JAVA_HOME".to_string(),
            env.java_home.to_string_lossy().to_string(),
        ),
        (
            "ANDROID_HOME".to_string(),
            env.android_home.to_string_lossy().to_string(),
        ),
        (
            "ANDROID_SDK_ROOT".to_string(),
            env.android_home.to_string_lossy().to_string(),
        ),
        (
            "GRADLE_USER_HOME".to_string(),
            env.gradle_user_home.to_string_lossy().to_string(),
        ),
    ]
}

fn gradle_bin_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["gradle.bat", "gradle"]
    } else {
        &["gradle"]
    }
}

fn java_bin_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["java.exe", "java"]
    } else {
        &["java"]
    }
}

fn find_apk_in_workspace(workspace: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    find_apks_recursive(
        &workspace
            .join(crate::utils::android_project_mod::MODULE_NAME)
            .join("build/outputs"),
        &mut results,
    );
    results.sort_by(|a, b| {
        let time_a = a.metadata().ok().and_then(|m| m.modified().ok());
        let time_b = b.metadata().ok().and_then(|m| m.modified().ok());
        time_b.cmp(&time_a)
    });
    results
}

fn find_apks_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_apks_recursive(&path, results);
            } else if path.extension().map(|e| e == "apk").unwrap_or(false) {
                results.push(path);
            }
        }
    }
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
    fn android_gradle_dependencies_strip_human_notes() {
        let deps = android_gradle_dependencies(
            "share",
            &[
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信)".to_string(),
                "百度地图 SDK 或 高德地图 SDK（见官方文档）".to_string(),
                "com.alipay.sdk:alipay-sdk-java".to_string(),
            ],
            None,
        );

        assert_eq!(
            deps,
            vec!["com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0".to_string()]
        );
    }

    #[test]
    fn android_gradle_dependencies_follow_enabled_manifest_providers() {
        let manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Push": {},
                    "Payment": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "unipush": {
                                "hms": {},
                                "oppo": false,
                                "honor": { "__platform__": ["ios"] }
                            }
                        },
                        "payment": {
                            "weixin": { "__platform__": ["android"] }
                        }
                    }
                }
            }
        });

        let push_deps = android_gradle_dependencies(
            "push",
            &[
                "com.getui:gtsdk:3.3.7.0".to_string(),
                "com.getui:gtc-dcloud:3.2.16.7".to_string(),
                "com.getui.opt:hwp:3.1.1 (华为)".to_string(),
                "com.huawei.hms:push:6.11.0.300 (华为)".to_string(),
                "com.assist-v3:oppo:3.3.0 (OPPO)".to_string(),
                "com.getui.opt:honor:3.6.0 (荣耀)".to_string(),
            ],
            Some(&manifest),
        );
        assert_eq!(
            push_deps,
            vec![
                "com.getui:gtsdk:3.3.7.0".to_string(),
                "com.getui:gtc-dcloud:3.2.16.7".to_string(),
                "com.getui.opt:hwp:3.1.1".to_string(),
                "com.huawei.hms:push:6.11.0.300".to_string(),
            ]
        );

        let payment_deps = android_gradle_dependencies(
            "payment",
            &[
                "com.alipay.sdk:alipaysdk-android:15.8.11 (支付宝)".to_string(),
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信支付)"
                    .to_string(),
                "com.paypal.checkout:android-sdk:0.6.2 (PayPal)".to_string(),
            ],
            Some(&manifest),
        );
        assert_eq!(
            payment_deps,
            vec!["com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0".to_string()]
        );
    }

    #[test]
    fn render_gradle_dependency_line_excludes_getui_from_gtsdk() {
        assert_eq!(
            render_gradle_dependency_line("com.getui:gtsdk:3.3.7.0"),
            "    implementation('com.getui:gtsdk:3.3.7.0'){ exclude(group: 'com.getui') }"
        );
        assert_eq!(
            render_gradle_dependency_line("com.getui:gysdk:3.1.7.0"),
            "    implementation('com.getui:gysdk:3.1.7.0'){ exclude(group: 'com.getui', module: 'gtc') }"
        );
        assert_eq!(
            render_gradle_dependency_line("com.getui:gtc-dcloud:3.2.16.7"),
            "    implementation 'com.getui:gtc-dcloud:3.2.16.7'"
        );
        assert!(
            render_dependency_excludes("    implementation 'com.getui:gtc-dcloud:3.2.16.7'")
                .is_empty()
        );
        assert!(
            render_dependency_excludes("    implementation 'androidx.core:core:1.12.0'").is_empty()
        );
    }

    #[test]
    fn gysdk_forces_allow_backup_false() {
        let mut deps = BTreeSet::new();
        deps.insert("com.getui:gysdk:3.1.7.0".to_string());
        assert!(android_build_requires_allow_backup_false(&deps));

        deps.clear();
        deps.insert("com.getui:gtc-dcloud:3.2.16.7".to_string());
        assert!(!android_build_requires_allow_backup_false(&deps));
    }

    #[test]
    fn render_manifest_placeholders_adds_push_defaults_without_report() {
        let detected_modules = vec![crate::commands::resource::DetectedModule {
            name: "Push".to_string(),
            category: "push".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        }];

        let placeholders =
            render_android_module_manifest_placeholders(None, &detected_modules, "io.demo.app");

        assert!(placeholders.contains(r#""PUSH_APPID": ""#));
        assert!(placeholders.contains(r#""plus.unipush.appid": ""#));
        assert!(placeholders.contains(r#""apk.applicationId": "io.demo.app""#));
        assert!(placeholders.contains(r#""PUSH_APPKEY": ""#));
        assert!(placeholders.contains(r#""PUSH_APPSECRET": ""#));
    }

    #[test]
    fn clean_android_artifact_name_keeps_real_files_only() {
        assert_eq!(
            clean_android_artifact_pattern("share-weixin-release.aar (微信)").as_deref(),
            Some("share-weixin-release.aar")
        );
        assert_eq!(
            clean_android_artifact_pattern("(使用微信分享SDK即可覆盖微信支付)"),
            None
        );
        assert_eq!(clean_android_artifact_pattern("not-a-file"), None);
        assert_eq!(
            android_artifact_search_stem("aliyun-face-XXX.aar"),
            "aliyun-face"
        );
        assert_eq!(
            android_artifact_search_stem("open_sdk_XXX_lite.jar"),
            "open_sdk-lite"
        );
    }

    #[test]
    fn android_sdk_artifact_lookup_ignores_versions() {
        let root = std::env::temp_dir().join(format!(
            "unipack-android-artifact-versionless-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("gtc-3.5.1.0.aar"), b"aar").unwrap();
        std::fs::write(root.join("open_sdk_3.5.0.0_dawfrwafr_lite.jar"), b"jar").unwrap();

        let gtc = find_android_sdk_artifact(&root, "gtc-3.2.16.0.aar").unwrap();
        let open_sdk = find_android_sdk_artifact(&root, "open_sdk_XXX_lite.jar").unwrap();

        assert_eq!(
            gtc.file_name().and_then(|name| name.to_str()),
            Some("gtc-3.5.1.0.aar")
        );
        assert_eq!(
            open_sdk.file_name().and_then(|name| name.to_str()),
            Some("open_sdk_3.5.0.0_dawfrwafr_lite.jar")
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
