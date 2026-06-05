use serde::{Deserialize, Serialize};

use super::resource::{
    parse_uniapp_manifest, read_manifest_file, DetectedModule, SplashscreenConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceScanResult {
    pub app_id: String,
    pub app_name: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u32>,
    pub hbuilderx_version: Option<String>,
    pub source_path: String,
    pub imported_path: String,
    pub app_resource_path: String,
    pub is_zip: bool,
    pub manifest_path: Option<String>,
    pub splashscreen: Option<SplashscreenConfig>,
    pub detected_modules: Vec<DetectedModule>,
    pub uts: UtsPluginScanResult,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UtsPluginScanResult {
    pub has_uts_plugins: bool,
    pub builtin_modules: Vec<UtsBuiltinModule>,
    pub custom_plugins: Vec<UtsCustomPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtsBuiltinModule {
    pub name: String,
    pub local_aar: String,
    pub online_deps: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtsCustomPlugin {
    pub id: String,
    pub android_dir: Option<String>,
    pub ios_dir: Option<String>,
    pub android_deps: Vec<String>,
    pub ios_frameworks: Vec<String>,
    pub abis: Option<Vec<String>>,
    pub min_sdk_version: Option<u32>,
    pub dependencies: Vec<PluginDependency>,
    pub components: Vec<UtsComponent>,
    pub hooks_class: Option<String>,
    pub gradle_plugins: Vec<String>,
    pub project_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(rename = "source")]
    pub source: Option<String>,
    #[serde(rename = "value")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtsComponent {
    pub name: String,
    pub class: String,
}

#[derive(Debug, Clone, Default)]
pub struct UtsPluginConfig {
    pub abis: Option<Vec<String>>,
    pub min_sdk_version: Option<u32>,
    pub dependencies: Vec<PluginDependency>,
    pub components: Vec<UtsComponent>,
    pub hooks_class: Option<String>,
    pub gradle_plugins: Vec<String>,
    pub project_dependencies: Vec<String>,
}

#[tauri::command]
pub async fn import_uniapp_resource(
    project_id: String,
    resource_path: String,
) -> Result<ResourceScanResult, String> {
    let source = std::path::PathBuf::from(&resource_path);
    if !source.exists() {
        return Err(format!("资源路径不存在: {}", resource_path));
    }

    let import_base = crate::utils::fs::get_project_config_dir(&project_id)
        .join("resources")
        .join(chrono::Local::now().format("%Y%m%d-%H%M%S").to_string());
    crate::utils::fs::ensure_directory(&import_base)
        .map_err(|e| format!("创建资源导入目录失败: {}", e))?;

    let is_zip = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false);

    let imported_root = if is_zip {
        crate::utils::fs::unzip_file(&source, &import_base)
            .map_err(|e| format!("解压资源包失败: {}", e))?;
        find_uniapp_package_root(&import_base)?
    } else if source.is_dir() {
        let dest = import_base.join(
            source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("resource"),
        );
        crate::utils::fs::copy_recursive(&source, &dest)
            .map_err(|e| format!("复制资源目录失败: {}", e))?;
        find_uniapp_package_root(&dest)?
    } else {
        return Err("请选择 HBuilderX 导出的 resources 目录、__UNI__ 目录或 zip 文件".to_string());
    };

    scan_imported_resource(&source, &imported_root, is_zip)
}

pub fn scan_imported_resource(
    source_path: &std::path::Path,
    resource_root: &std::path::Path,
    is_zip: bool,
) -> Result<ResourceScanResult, String> {
    let layout = resolve_resource_layout(resource_root)?;
    let manifest_path = find_manifest(&layout.app_resource_path);
    let manifest = manifest_path
        .as_ref()
        .and_then(|path| read_manifest_file(path).ok());

    let mut warnings = Vec::new();
    let dir_app_id = layout
        .app_resource_path
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| n.starts_with("__UNI__") || n.starts_with("__uni__"))
        .map(String::from);
    let app_id = manifest
        .as_ref()
        .and_then(|m: &serde_json::Value| {
            m.get("appid")
                .or_else(|| m.get("appId"))
                .or_else(|| m.get("id"))
                .and_then(|v| v.as_str())
        })
        .filter(|id| !id.is_empty())
        .map(String::from)
        .or(dir_app_id)
        .ok_or_else(|| "无法从资源目录或 manifest.json 提取 __UNI__ AppId".to_string())?;

    if !app_id.starts_with("__UNI__") && !app_id.starts_with("__uni__") {
        warnings.push(format!("AppId '{}' 不是标准 __UNI__ 前缀", app_id));
    }

    let manifest_info = manifest_path
        .as_ref()
        .zip(manifest.as_ref())
        .map(|(path, manifest)| {
            parse_uniapp_manifest(manifest, path, &layout.app_resource_path, None)
        });
    let uts = scan_uts_plugins(&layout.package_root);

    Ok(ResourceScanResult {
        app_id,
        app_name: manifest_info.as_ref().and_then(|m| m.app_name.clone()),
        version_name: manifest_info.as_ref().and_then(|m| m.version_name.clone()),
        version_code: manifest_info.as_ref().and_then(|m| m.version_code),
        hbuilderx_version: manifest_info
            .as_ref()
            .and_then(|m| m.hbuilderx_version.clone()),
        source_path: source_path.to_string_lossy().to_string(),
        imported_path: layout.package_root.to_string_lossy().to_string(),
        app_resource_path: layout.app_resource_path.to_string_lossy().to_string(),
        is_zip,
        manifest_path: manifest_path.map(|p| p.to_string_lossy().to_string()),
        splashscreen: manifest_info.as_ref().and_then(|m| m.splashscreen.clone()),
        detected_modules: manifest_info
            .map(|m| m.detected_modules)
            .unwrap_or_default(),
        uts,
        warnings,
    })
}

#[derive(Debug, Clone)]
pub struct ResourceLayout {
    pub package_root: std::path::PathBuf,
    pub app_resource_path: std::path::PathBuf,
}

pub fn resolve_resource_layout(resource_root: &std::path::Path) -> Result<ResourceLayout, String> {
    if is_hbuilderx_resources_root(resource_root) {
        let app_resource_path = find_direct_uniapp_app_dir(resource_root).ok_or_else(|| {
            format!(
                "resources 目录中未找到 __UNI__ 应用资源: {}",
                resource_root.display()
            )
        })?;
        return Ok(ResourceLayout {
            package_root: resource_root.to_path_buf(),
            app_resource_path,
        });
    }

    if is_uniapp_app_resource_root(resource_root) {
        return Ok(ResourceLayout {
            package_root: resource_root.to_path_buf(),
            app_resource_path: resource_root.to_path_buf(),
        });
    }

    Err(format!(
        "未找到 UniApp 资源根目录: {}",
        resource_root.display()
    ))
}

fn find_uniapp_package_root(base: &std::path::Path) -> Result<std::path::PathBuf, String> {
    if is_uniapp_package_root(base) {
        return Ok(base.to_path_buf());
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if is_uniapp_package_root(&path) {
                        return Ok(path);
                    }
                    stack.push(path);
                }
            }
        }
    }
    Err(format!("未找到 UniApp 资源根目录: {}", base.display()))
}

fn is_uniapp_package_root(path: &std::path::Path) -> bool {
    is_hbuilderx_resources_root(path) || is_uniapp_app_resource_root(path)
}

fn is_hbuilderx_resources_root(path: &std::path::Path) -> bool {
    if !path.is_dir() || find_direct_uniapp_app_dir(path).is_none() {
        return false;
    }

    let is_named_resources = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.eq_ignore_ascii_case("resources"))
        .unwrap_or(false);

    is_named_resources || path.join("uni_modules").is_dir()
}

fn is_uniapp_app_resource_root(path: &std::path::Path) -> bool {
    let name_ok = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with("__UNI__") || name.starts_with("__uni__"))
        .unwrap_or(false);
    name_ok || path.join("manifest.json").exists() || path.join("www").exists()
}

fn find_direct_uniapp_app_dir(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(path).ok()?;
    entries.flatten().map(|e| e.path()).find(|entry| {
        entry.is_dir()
            && entry
                .file_name()
                .and_then(|n| n.to_str())
                .map(|name| name.starts_with("__UNI__") || name.starts_with("__uni__"))
                .unwrap_or(false)
    })
}

fn find_manifest(resource_root: &std::path::Path) -> Option<std::path::PathBuf> {
    let direct = resource_root.join("manifest.json");
    if direct.exists() {
        return Some(direct);
    }
    let www = resource_root.join("www").join("manifest.json");
    if www.exists() {
        return Some(www);
    }
    None
}

pub fn extract_hbuilderx_version(manifest: &serde_json::Value) -> Option<String> {
    for key in [
        "hbuilderxVersion",
        "hbuilderXVersion",
        "compilerVersion",
        "uniCompilerVersion",
    ] {
        if let Some(value) = manifest.get(key).and_then(|v| v.as_str()) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    manifest
        .get("app-plus")
        .and_then(|v| v.get("compilerVersion"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

// --- UTS Plugin Scanning ---

pub fn scan_uts_plugins(resource_root: &std::path::Path) -> UtsPluginScanResult {
    let uni_modules = resource_root
        .join("uni_modules")
        .is_dir()
        .then(|| resource_root.join("uni_modules"))
        .or_else(|| {
            resolve_resource_layout(resource_root)
                .ok()
                .map(|layout| layout.app_resource_path.join("uni_modules"))
                .filter(|path| path.is_dir())
        })
        .unwrap_or_else(|| resource_root.join("uni_modules"));
    if !uni_modules.is_dir() {
        return UtsPluginScanResult::default();
    }

    let mut result = UtsPluginScanResult {
        has_uts_plugins: true,
        builtin_modules: Vec::new(),
        custom_plugins: Vec::new(),
    };

    if let Ok(entries) = std::fs::read_dir(&uni_modules) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            if let Some(module) = builtin_uts_module(&id) {
                push_builtin_with_dependencies(&mut result.builtin_modules, module);
            } else {
                result
                    .custom_plugins
                    .push(scan_custom_uts_plugin(&id, &path));
            }
        }
    }

    result
}

fn push_builtin_with_dependencies(modules: &mut Vec<UtsBuiltinModule>, module: UtsBuiltinModule) {
    for dep in module.depends_on.clone() {
        if let Some(dep_module) = builtin_uts_module(&dep) {
            push_builtin_with_dependencies(modules, dep_module);
        }
    }
    if !modules.iter().any(|m| m.name == module.name) {
        modules.push(module);
    }
}

pub fn builtin_uts_module(name: &str) -> Option<UtsBuiltinModule> {
    let module = match name {
        "uni-createRequestPermissionListener" => (
            "uni-createRequestPermissionListener-release.aar",
            vec![],
            vec![],
        ),
        "uni-getNetworkType" => ("uni-getNetworkType-release.aar", vec![], vec![]),
        "uni-installApk" => ("uni-installApk-release.aar", vec![], vec![]),
        "uni-network" => (
            "uni-network-release.aar",
            vec!["com.squareup.okhttp3:okhttp:3.12.12"],
            vec![],
        ),
        "uni-privacy" => ("uni-privacy-release.aar", vec![], vec![]),
        "uni-chooseMedia" => (
            "uni-chooseMedia-release.aar",
            vec![
                "androidx.appcompat:appcompat:1.6.1",
                "androidx.activity:activity-ktx:1.9.2",
            ],
            vec!["uni-prompt"],
        ),
        "uni-getAppBaseInfo" => ("uni-getAppBaseInfo-release.aar", vec![], vec![]),
        "uni-storage" => ("uni-storage-release.aar", vec![], vec![]),
        "uni-getSystemInfo" => ("uni-getSystemInfo-release.aar", vec![], vec![]),
        "uni-getDeviceInfo" => ("uni-getDeviceInfo-release.aar", vec![], vec![]),
        "uni-openAppAuthorizeSetting" => {
            ("uni-openAppAuthorizeSetting-release.aar", vec![], vec![])
        }
        "uni-exit" => ("uni-exit-release.aar", vec![], vec![]),
        "uni-getAccessibilityInfo" => ("uni-getAccessibilityInfo-release.aar", vec![], vec![]),
        "uni-getAppAuthorizeSetting" => ("uni-getAppAuthorizeSetting-release.aar", vec![], vec![]),
        "uni-getSystemSetting" => ("uni-getSystemSetting-release.aar", vec![], vec![]),
        "uni-prompt" => (
            "uni-prompt-release.aar",
            vec![
                "androidx.recyclerview:recyclerview:1.0.0",
                "androidx.appcompat:appcompat:1.0.0",
            ],
            vec![],
        ),
        "uni-getLocation-tencent-uni1" => (
            "uni-getLocation-tencent-uni1-release.aar",
            vec!["com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8"],
            vec![],
        ),
        _ => return None,
    };

    Some(UtsBuiltinModule {
        name: name.to_string(),
        local_aar: module.0.to_string(),
        online_deps: module.1.into_iter().map(String::from).collect(),
        depends_on: module.2.into_iter().map(String::from).collect(),
    })
}

fn scan_custom_uts_plugin(id: &str, plugin_root: &std::path::Path) -> UtsCustomPlugin {
    let package_id = read_uts_package_id(plugin_root);
    let module_id = package_id.clone().unwrap_or_else(|| id.to_string());
    let source_android_dir = plugin_root.join("utssdk").join("app-android");
    let android_dir =
        resolve_android_uts_dir(plugin_root, &source_android_dir, id, package_id.as_deref());
    let ios_dir = plugin_root.join("utssdk").join("app-ios");
    let config = parse_uts_plugin_config(&android_dir.join("config.json"));
    let mut android_deps: Vec<String> = config
        .dependencies
        .iter()
        .filter_map(|dep| dep.source.clone().or(dep.value.clone()))
        .collect();
    android_deps.sort();
    android_deps.dedup();
    let ios_frameworks = if ios_dir.exists() {
        crate::utils::fs::find_files_by_extension(&ios_dir, "framework")
            .unwrap_or_default()
            .into_iter()
            .chain(
                crate::utils::fs::find_files_by_extension(&ios_dir, "xcframework")
                    .unwrap_or_default(),
            )
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    } else {
        Vec::new()
    };

    UtsCustomPlugin {
        id: module_id,
        android_dir: android_dir
            .exists()
            .then(|| android_dir.to_string_lossy().to_string()),
        ios_dir: ios_dir
            .exists()
            .then(|| ios_dir.to_string_lossy().to_string()),
        android_deps,
        ios_frameworks,
        abis: config.abis,
        min_sdk_version: config.min_sdk_version,
        dependencies: config.dependencies,
        components: config.components,
        hooks_class: config.hooks_class,
        gradle_plugins: config.gradle_plugins,
        project_dependencies: config.project_dependencies,
    }
}

fn read_uts_package_id(plugin_root: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(plugin_root.join("package.json")).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    value
        .get("id")
        .and_then(|id| id.as_str())
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(String::from)
}

fn resolve_android_uts_dir(
    plugin_root: &std::path::Path,
    source_android_dir: &std::path::Path,
    folder_id: &str,
    package_id: Option<&str>,
) -> std::path::PathBuf {
    if android_uts_dir_has_native_sources(source_android_dir) {
        return source_android_dir.to_path_buf();
    }

    find_compiled_android_uts_dir(plugin_root, folder_id, package_id)
        .unwrap_or_else(|| source_android_dir.to_path_buf())
}

fn android_uts_dir_has_native_sources(android_dir: &std::path::Path) -> bool {
    let src_dir = android_dir.join("src");
    if !src_dir.is_dir() {
        return false;
    }
    for ext in ["kt", "java"] {
        if crate::utils::fs::find_files_by_extension(&src_dir, ext)
            .map(|files| !files.is_empty())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

fn find_compiled_android_uts_dir(
    plugin_root: &std::path::Path,
    folder_id: &str,
    package_id: Option<&str>,
) -> Option<std::path::PathBuf> {
    let uni_modules = plugin_root.parent()?;
    let project_root = uni_modules.parent()?;
    let candidate_roots = [
        project_root.join("unpackage/resources/uni_modules"),
        project_root.join("unpackage/dist/build/app-plus/uni_modules"),
        project_root.join("unpackage/dist/dev/app-plus/uni_modules"),
    ];
    let mut names = vec![folder_id.to_string()];
    if let Some(package_id) = package_id {
        if !names.iter().any(|name| name == package_id) {
            names.push(package_id.to_string());
        }
    }

    for root in candidate_roots {
        if !root.is_dir() {
            continue;
        }
        for name in &names {
            if let Some(plugin_dir) = find_uni_module_dir(&root, name) {
                let android_dir = plugin_dir.join("utssdk").join("app-android");
                if android_uts_dir_has_native_sources(&android_dir) {
                    return Some(android_dir);
                }
            }
        }
    }

    None
}

fn find_uni_module_dir(root: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let exact = root.join(name);
    if exact.is_dir() {
        return Some(exact);
    }
    let target = name.to_ascii_lowercase();
    std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .map(|file_name| file_name.to_ascii_lowercase() == target)
                    .unwrap_or(false)
        })
}

pub fn parse_uts_plugin_config(config_path: &std::path::Path) -> UtsPluginConfig {
    let Ok(content) = std::fs::read_to_string(config_path) else {
        return UtsPluginConfig::default();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return UtsPluginConfig::default();
    };

    UtsPluginConfig {
        abis: value.get("abis").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect()
        }),
        min_sdk_version: value
            .get("minSdkVersion")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
        dependencies: parse_dependencies_array(&value),
        components: parse_components_array(&value),
        hooks_class: value
            .get("hooksClass")
            .and_then(|v| v.as_str())
            .map(String::from),
        gradle_plugins: value
            .get("project")
            .and_then(|p| p.get("plugins"))
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        project_dependencies: value
            .get("project")
            .and_then(|p| p.get("dependencies"))
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn parse_dependencies_array(value: &serde_json::Value) -> Vec<PluginDependency> {
    let mut result = Vec::new();

    if let Some(arr) = value.get("dependencies").and_then(|d| d.as_array()) {
        for item in arr {
            if let Some(s) = item.as_str() {
                result.push(PluginDependency {
                    id: None,
                    source: None,
                    value: Some(s.to_string()),
                });
            } else if let Some(obj) = item.as_object() {
                result.push(PluginDependency {
                    id: obj.get("id").and_then(|v| v.as_str()).map(String::from),
                    source: obj.get("source").and_then(|v| v.as_str()).map(String::from),
                    value: None,
                });
            }
        }
    }

    result
}

fn parse_components_array(value: &serde_json::Value) -> Vec<UtsComponent> {
    match value.get("components").and_then(|c| c.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|item| {
                Some(UtsComponent {
                    name: item.get("name")?.as_str()?.to_string(),
                    class: item.get("class")?.as_str()?.to_string(),
                })
            })
            .collect(),
        None => Vec::new(),
    }
}
