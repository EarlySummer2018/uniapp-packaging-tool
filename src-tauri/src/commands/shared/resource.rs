use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::commands::shared::resource_scan::extract_hbuilderx_version;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipAnalysisResult {
    pub app_name: Option<String>,
    pub app_id: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u32>,
    pub package_names: PlatformPackages,
    pub detected_modules: Vec<DetectedModule>,
    pub has_dcloud_properties: bool,
    pub has_resources: bool,
    pub resource_files: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPackages {
    pub android_package: Option<String>,
    pub ios_bundle_id: Option<String>,
    pub harmony_bundle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidManifestConfig {
    pub package_name: Option<String>,
    pub min_sdk_version: Option<u32>,
    pub target_sdk_version: Option<u32>,
    pub compile_sdk_version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SplashscreenConfig {
    pub android_style: Option<String>,
    pub android: BTreeMap<String, String>,
    pub use_original_msgbox: Option<bool>,
}

/// Android 多密度图标配置（来自 manifest.json app-plus.distribute.icons.android）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidIconsConfig {
    /// 密度名 → 图片绝对路径（如 "hdpi" → "/path/to/72x72.png"）
    pub android: BTreeMap<String, String>,
}

/// iOS 多尺寸图标配置（来自 manifest.json app-plus.distribute.icons.ios）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IosIconsConfig {
    /// slot 名 → 图片绝对路径（如 "iphone.app@3x" → "/path/to/180x180.png"）
    pub ios: BTreeMap<String, String>,
}

/// Push 通知图标配置（来自 manifest.json app-plus.distribute.sdkConfigs.push）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PushIconsConfig {
    /// Android 小图标绝对路径
    pub small: Option<String>,
    /// Android 小图标多密度资源（密度名 → 图片绝对路径）
    pub small_densities: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniappManifestInfo {
    pub app_name: Option<String>,
    pub app_id: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u32>,
    pub hbuilderx_version: Option<String>,
    #[serde(default)]
    pub android_icons: Option<AndroidIconsConfig>,
    #[serde(default)]
    pub ios_icons: Option<IosIconsConfig>,
    #[serde(default)]
    pub push_icons: Option<PushIconsConfig>,
    #[serde(default)]
    pub splashscreen: Option<SplashscreenConfig>,
    #[serde(default)]
    pub manifest_value: Option<serde_json::Value>,
    pub manifest_path: String,
    pub project_root: String,
    pub android: AndroidManifestConfig,
    pub package_names: PlatformPackages,
    pub detected_modules: Vec<DetectedModule>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedModule {
    pub name: String,
    pub category: String,
    pub platforms: Vec<String>,
    pub configured: bool,
    pub required_keys: Vec<String>,
    pub source: String,
}

fn read_zip_entry_to_string(entry: &mut zip::read::ZipFile) -> Result<String, String> {
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    String::from_utf8(buf).map_err(|e| format!("Invalid UTF-8 in {}: {}", entry.name(), e))
}

pub fn read_manifest_file(manifest_path: &Path) -> Result<serde_json::Value, String> {
    let content = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
    parse_manifest_content(&content).map_err(|e| format!("解析 manifest.json 失败: {}", e))
}

fn parse_manifest_content(content: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str::<serde_json::Value>(content).or_else(|json_error| {
        json5::from_str::<serde_json::Value>(content).map_err(|json5_error| {
            format!(
                "{}；已尝试按 JSON5/JSONC 兼容格式解析，仍失败: {}",
                json_error, json5_error
            )
        })
    })
}

#[tauri::command]
pub async fn read_uniapp_manifest(project_path: String) -> Result<UniappManifestInfo, String> {
    read_uniapp_manifest_sync(&project_path)
}

pub fn read_uniapp_manifest_sync(project_path: &str) -> Result<UniappManifestInfo, String> {
    let project_root = PathBuf::from(project_path);
    if !project_root.is_dir() {
        return Err(format!("本地项目路径不存在: {}", project_path));
    }

    let manifest_path = project_root.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("未找到 manifest.json: {}", manifest_path.display()));
    }

    let manifest = read_manifest_file(&manifest_path)?;
    Ok(parse_uniapp_manifest(
        &manifest,
        &manifest_path,
        &project_root,
        None,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedResource {
    pub id: String,
    pub name: String,
    pub r#type: ResourceType,
    pub source_path: String,
    pub size_bytes: u64,
    pub imported_at: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Image,
    Font,
    Audio,
    Video,
    Json,
    Raw,
    Other,
}

#[tauri::command]
pub async fn import_resource(
    _project_path: String,
    resource_type: String,
    source_path: String,
) -> Result<ImportedResource, String> {
    let path = std::path::Path::new(&source_path);
    if !path.exists() {
        return Err(format!("Resource file not found: {}", source_path));
    }

    let metadata = tokio::fs::metadata(&source_path)
        .await
        .map_err(|e| e.to_string())?;

    let res_type = match resource_type.as_str() {
        "image" => ResourceType::Image,
        "font" => ResourceType::Font,
        "audio" => ResourceType::Audio,
        "video" => ResourceType::Video,
        "json" => ResourceType::Json,
        "raw" => ResourceType::Raw,
        _ => ResourceType::Other,
    };

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ImportedResource {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        r#type: res_type,
        source_path,
        size_bytes: metadata.len(),
        imported_at: chrono::Utc::now().to_rfc3339(),
        metadata: serde_json::json!({}),
    })
}

#[tauri::command]
pub async fn import_resources_batch(
    project_path: String,
    resources: Vec<ResourceImportInput>,
) -> Result<Vec<ImportedResource>, String> {
    let mut results = Vec::with_capacity(resources.len());
    for input in resources {
        let result = import_resource(project_path.clone(), input.r#type, input.path).await?;
        results.push(result);
    }
    Ok(results)
}

#[derive(Debug, Deserialize)]
pub struct ResourceImportInput {
    pub path: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[tauri::command]
pub async fn get_resource_list(project_path: String) -> Result<Vec<ImportedResource>, String> {
    let resources_dir = std::path::Path::new(&project_path).join("resources");
    if !resources_dir.exists() {
        return Ok(Vec::new());
    }
    let mut resources = Vec::new();
    let mut entries = tokio::fs::read_dir(&resources_dir)
        .await
        .map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let meta = entry.metadata().await.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        resources.push(ImportedResource {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            r#type: ResourceType::Raw,
            source_path: entry.path().to_string_lossy().to_string(),
            size_bytes: meta.len(),
            imported_at: chrono::Utc::now().to_rfc3339(),
            metadata: serde_json::json!({}),
        });
    }
    Ok(resources)
}

#[tauri::command]
pub async fn remove_resource(_project_path: String, _resource_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn analyze_uploaded_zip(zip_path: String) -> Result<ZipAnalysisResult, String> {
    let file = File::open(&zip_path).map_err(|e| format!("Cannot open zip file: {}", e))?;

    let mut reader =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip format: {}", e))?;

    let mut result = ZipAnalysisResult {
        app_name: None,
        app_id: None,
        version_name: None,
        version_code: None,
        package_names: PlatformPackages {
            android_package: None,
            ios_bundle_id: None,
            harmony_bundle: None,
        },
        detected_modules: vec![],
        has_dcloud_properties: false,
        has_resources: false,
        resource_files: vec![],
        error: None,
    };

    let mut manifest_content: Option<String> = None;
    let mut props_content: Option<String> = None;
    let mut resource_entries: Vec<String> = vec![];

    for i in 0..reader.len() {
        let mut entry = reader.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        if name.ends_with("manifest.json")
            && !name.contains("node_modules")
            && !name.contains("unpackage")
        {
            match read_zip_entry_to_string(&mut entry) {
                Ok(content) => {
                    manifest_content = Some(content);
                }
                Err(_) => {}
            }
        }

        if name.ends_with("dcloud_properties.xml") {
            match read_zip_entry_to_string(&mut entry) {
                Ok(content) => {
                    props_content = Some(content);
                    result.has_dcloud_properties = true;
                }
                Err(_) => {}
            }
        }

        if (name.starts_with("www/")
            || name.contains("/assets/")
            || name.starts_with("unpackage/resources/"))
            && !name.ends_with('/')
        {
            resource_entries.push(name);
            result.has_resources = true;
        }
    }

    result.resource_files = resource_entries;

    if let Some(content) = manifest_content {
        let manifest: serde_json::Value = parse_manifest_content(&content)
            .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;
        let manifest_info = parse_uniapp_manifest(
            &manifest,
            std::path::Path::new("manifest.json"),
            std::path::Path::new("."),
            None,
        );
        result.app_name = manifest_info.app_name;
        result.app_id = manifest_info.app_id;
        result.version_name = manifest_info.version_name;
        result.version_code = manifest_info.version_code;
        result.package_names = manifest_info.package_names;
        result.detected_modules = manifest_info.detected_modules;
    }

    if let Some(ref props) = props_content {
        for module in &mut result.detected_modules {
            module.configured = check_module_configured_in_props(&module.name, props);
        }
    }

    Ok(result)
}

// --- Module matching & manifest parsing ---

fn match_module_to_category(module_name: &str) -> DetectedModule {
    match module_name {
        "Push" => DetectedModule {
            name: "Push".to_string(),
            category: "push".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![
                "unipush_appid".into(),
                "unipush_appkey".into(),
                "unipush_appsecret".into(),
                "XIAOMI_APP_ID".into(),
                "XIAOMI_APP_KEY".into(),
                "MEIZU_APP_ID".into(),
                "MEIZU_APP_KEY".into(),
                "HUAWEI_APP_ID".into(),
                "OPPO_APP_KEY".into(),
                "OPPO_APP_SECRET".into(),
                "VIVO_APP_ID".into(),
                "VIVO_APP_KEY".into(),
                "HONOR_APP_ID".into(),
            ],
            source: String::new(),
        },
        "Geolocation" => DetectedModule {
            name: "Geolocation".to_string(),
            category: "geolocation".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["BAIDU_MAP_AK".into(), "AMAP_KEY".into()],
            source: String::new(),
        },
        "Share" => DetectedModule {
            name: "Share".to_string(),
            category: "share".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![
                "WX_APPID".into(),
                "WX_SECRET".into(),
                "QQ_APPID".into(),
                "SINA_APPKEY".into(),
                "SINA_SECRET".into(),
                "SINA_REDIRECT_URI".into(),
            ],
            source: String::new(),
        },
        "Login" | "OAuth" => DetectedModule {
            name: "Login".to_string(),
            category: "login".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["WX_APPID".into(), "QQ_APPID".into()],
            source: String::new(),
        },
        "Payment" => DetectedModule {
            name: "Payment".to_string(),
            category: "payment".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["WX_APPID".into(), "ALIPAY_PARTNER_ID".into()],
            source: String::new(),
        },
        "Map" | "Maps" => DetectedModule {
            name: "Map".to_string(),
            category: "map".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["AMAP_KEY".into(), "TENCENT_MAP_KEY".into()],
            source: String::new(),
        },
        "Speech" => DetectedModule {
            name: "Speech".to_string(),
            category: "speech".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![
                "IFLY_APPID".into(),
                "BD_SPEECH_APIKEY".into(),
                "ALI_SPEAK_ACCESSKEY_ID".into(),
            ],
            source: String::new(),
        },
        "Statistic" | "Statistics" => DetectedModule {
            name: "Statistic".to_string(),
            category: "statistic".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["UMENG_APPKEY".into(), "MTA_APPID".into()],
            source: String::new(),
        },
        "FaceRecognition" | "FaceRecognitionVerify" => DetectedModule {
            name: "FaceRecognition".to_string(),
            category: "face_recognition".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![
                "DCLOUD_LICENSE".into(),
                "BDFACE_APIKEY".into(),
                "ALIFACE_ACCESSKEY_ID".into(),
            ],
            source: String::new(),
        },
        "UniAD" | "uni-ad" => DetectedModule {
            name: "UniAD".to_string(),
            category: "uni_ad".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        _ => DetectedModule {
            name: module_name.to_string(),
            category: "manifest".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
    }
}

fn check_module_configured_in_props(module_name: &str, props_content: &str) -> bool {
    match module_name {
        "Push" => props_content.contains(r#"feature name="Push""#),
        "Share" => props_content.contains(r#"feature name="Share""#),
        "Geolocation" => props_content.contains("Geolocation"),
        "Login" | "OAuth" => {
            props_content.contains(r#"feature name="Login""#)
                || props_content.contains(r#"feature name="OAuth""#)
        }
        "Payment" => props_content.contains(r#"feature name="Payment""#),
        "Map" | "Maps" => {
            props_content.contains("Mapp") || props_content.contains(r#"feature name="Map""#)
        }
        _ => false,
    }
}

pub fn parse_uniapp_manifest(
    manifest: &serde_json::Value,
    manifest_path: &Path,
    project_root: &Path,
    props_content: Option<&str>,
) -> UniappManifestInfo {
    let app_id = string_field(manifest, &["appid", "appId", "id"]);
    let app_name = string_field(manifest, &["name"]);
    let version_name = string_field(manifest, &["versionName", "version"]);
    let version_code = number_field(manifest, &["versionCode", "version_code"]);
    let hbuilderx_version = extract_hbuilderx_version(manifest);
    let android_icons = find_manifest_android_icons(manifest, project_root);
    let ios_icons = find_manifest_ios_icons(manifest, project_root);
    let push_icons = find_manifest_push_icons(manifest, project_root);
    let splashscreen = find_manifest_splashscreen(manifest, project_root);
    let android_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("android"));
    let ios_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("ios"));
    let harmony_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("harmony"));

    let mut detected_modules = Vec::new();
    if let Some(app_plus) = manifest.get("app-plus") {
        let app_modules = app_plus.get("modules");
        if let Some(modules) = app_modules {
            collect_modules_from_value(modules, "all", &mut detected_modules);
        }
        if let Some(distribute) = app_plus.get("distribute") {
            if let Some(sdk_configs) = distribute.get("sdkConfigs") {
                collect_modules_from_sdk_configs(
                    sdk_configs,
                    "all",
                    &mut detected_modules,
                    app_modules,
                );
            }
            if let Some(android) = distribute.get("android") {
                let android_modules = android.get("modules").or(app_modules);
                if let Some(modules) = android.get("modules") {
                    collect_modules_from_value(modules, "android", &mut detected_modules);
                }
                if let Some(sdk_configs) = android.get("sdkConfigs") {
                    collect_modules_from_sdk_configs(
                        sdk_configs,
                        "android",
                        &mut detected_modules,
                        android_modules,
                    );
                }
            }
            if let Some(ios) = distribute.get("ios") {
                let ios_modules = ios.get("modules").or(app_modules);
                if let Some(modules) = ios.get("modules") {
                    collect_modules_from_value(modules, "ios", &mut detected_modules);
                }
                if let Some(sdk_configs) = ios.get("sdkConfigs") {
                    collect_modules_from_sdk_configs(
                        sdk_configs,
                        "ios",
                        &mut detected_modules,
                        ios_modules,
                    );
                }
            }
            if let Some(harmony) = distribute.get("harmony") {
                if let Some(modules) = harmony.get("modules") {
                    collect_modules_from_value(modules, "harmony", &mut detected_modules);
                }
            }
        }
    }

    if let Some(props) = props_content {
        for module in &mut detected_modules {
            module.configured = check_module_configured_in_props(&module.name, props);
        }
    }

    UniappManifestInfo {
        app_name,
        app_id,
        version_name,
        version_code,
        hbuilderx_version,
        android_icons,
        ios_icons,
        push_icons,
        splashscreen,
        manifest_value: Some(manifest.clone()),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        project_root: project_root.to_string_lossy().to_string(),
        android: AndroidManifestConfig {
            package_name: android_value
                .and_then(|v| string_field(v, &["packageName", "packagename", "applicationId"])),
            min_sdk_version: Some(
                android_value
                    .and_then(|v| {
                        number_field(
                            v,
                            &["minSdkVersion", "minSdk", "min_sdk", "minSdkVersionCode"],
                        )
                    })
                    .unwrap_or(21),
            ),
            target_sdk_version: android_value
                .and_then(|v| number_field(v, &["targetSdkVersion", "targetSdk", "target_sdk"])),
            compile_sdk_version: android_value
                .and_then(|v| number_field(v, &["compileSdkVersion", "compileSdk", "compile_sdk"])),
        },
        package_names: PlatformPackages {
            android_package: android_value
                .and_then(|v| string_field(v, &["packageName", "packagename", "applicationId"])),
            ios_bundle_id: ios_value
                .and_then(|v| string_field(v, &["bundleId", "bundleid", "bundleIdentifier"])),
            harmony_bundle: harmony_value
                .and_then(|v| string_field(v, &["packageName", "bundleName", "bundle"])),
        },
        detected_modules,
        warnings: Vec::new(),
    }
}

fn string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(String::from)
    })
}

fn bool_field(value: &serde_json::Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|v| v.as_bool()))
}

fn number_field(value: &serde_json::Value, keys: &[&str]) -> Option<u32> {
    keys.iter().find_map(|key| {
        let value = value.get(*key)?;
        if let Some(num) = value.as_u64() {
            return Some(num as u32);
        }
        value.as_str().and_then(|s| s.trim().parse::<u32>().ok())
    })
}

fn collect_modules_from_value(
    modules: &serde_json::Value,
    platform: &str,
    detected: &mut Vec<DetectedModule>,
) {
    if let Some(items) = modules.as_array() {
        for item in items {
            if let Some(name) = item
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| item.as_str())
            {
                push_detected_module(detected, name, platform);
            }
        }
        return;
    }

    if let Some(map) = modules.as_object() {
        for (name, value) in map {
            let enabled = value
                .as_bool()
                .or_else(|| {
                    value
                        .as_object()
                        .and_then(|obj| obj.get("enabled"))
                        .and_then(|v| v.as_bool())
                })
                .unwrap_or(true);
            if enabled {
                push_detected_module(detected, name, platform);
            }
        }
    }
}

fn collect_modules_from_sdk_configs(
    sdk_configs: &serde_json::Value,
    platform: &str,
    detected: &mut Vec<DetectedModule>,
    enabled_modules: Option<&serde_json::Value>,
) {
    let Some(map) = sdk_configs.as_object() else {
        return;
    };

    for (key, value) in map {
        if !sdk_config_value_enabled(value) {
            continue;
        }

        if let Some(module_name) = sdk_config_key_to_module_name(key) {
            if enabled_modules
                .map(|modules| module_declared_enabled(modules, module_name))
                .unwrap_or(true)
            {
                push_detected_module(detected, module_name, platform);
            }
        }
    }
}

fn sdk_config_key_to_module_name(key: &str) -> Option<&'static str> {
    match key {
        "push" | "unipush" | "unipushV2" | "uniPush" => Some("Push"),
        "share" | "shares" => Some("Share"),
        "oauth" | "login" | "oauths" => Some("OAuth"),
        "payment" | "pay" | "payments" => Some("Payment"),
        "maps" | "map" => Some("Maps"),
        "geolocation" | "location" | "position" => Some("Geolocation"),
        "speech" | "speechRecognition" => Some("Speech"),
        "statistic" | "statistics" | "statics" => Some("Statistic"),
        "ad" | "ads" | "uni-ad" | "uniAD" | "uniad" => Some("UniAD"),
        "facialRecognitionVerify" | "faceRecognition" | "face_recognition" => {
            Some("FacialRecognitionVerify")
        }
        "x5" | "x5Webview" | "x5_webview" => Some("X5Webview"),
        "livepusher" | "livePusher" => Some("LivePusher"),
        _ => None,
    }
}

fn module_declared_enabled(modules: &serde_json::Value, module_name: &str) -> bool {
    if let Some(items) = modules.as_array() {
        return items.iter().any(|item| {
            let Some(name) = item
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| item.as_str())
            else {
                return false;
            };
            module_names_equivalent(name, module_name) && sdk_config_value_enabled(item)
        });
    }

    if let Some(map) = modules.as_object() {
        return map.iter().any(|(name, value)| {
            module_names_equivalent(name, module_name) && sdk_config_value_enabled(value)
        });
    }

    false
}

fn module_names_equivalent(left: &str, right: &str) -> bool {
    let left_module = match_module_to_category(left);
    let right_module = match_module_to_category(right);
    if left_module.category != "manifest" || right_module.category != "manifest" {
        left_module.category == right_module.category
    } else {
        normalize_manifest_key(left) == normalize_manifest_key(right)
    }
}

fn normalize_manifest_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn sdk_config_value_enabled(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(flag) => *flag,
        serde_json::Value::Null => false,
        serde_json::Value::Object(map) => {
            let enabled = map
                .get("enabled")
                .or_else(|| map.get("enable"))
                .or_else(|| map.get("open"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            enabled && config_value_applies_to_platform(map, None)
        }
        _ => true,
    }
}

fn config_value_applies_to_platform(
    map: &serde_json::Map<String, serde_json::Value>,
    platform: Option<&str>,
) -> bool {
    let Some(platforms) = map.get("__platform__") else {
        return true;
    };
    let Some(platform) = platform else {
        return true;
    };
    let platform = platform.to_ascii_lowercase();
    match platforms {
        serde_json::Value::Array(items) => items.iter().any(|item| {
            item.as_str()
                .map(|candidate| {
                    let candidate = candidate.to_ascii_lowercase();
                    candidate == platform || candidate == "app" || candidate == "all"
                })
                .unwrap_or(false)
        }),
        serde_json::Value::String(candidate) => {
            let candidate = candidate.to_ascii_lowercase();
            candidate == platform || candidate == "app" || candidate == "all"
        }
        _ => true,
    }
}

fn push_detected_module(detected: &mut Vec<DetectedModule>, raw_name: &str, platform: &str) {
    let mut module = match_module_to_category(raw_name);
    module.name = raw_name.to_string();
    module.source = "manifest.json".to_string();
    let platform = platform.to_string();
    if let Some(existing) = detected.iter_mut().find(|m| m.name == module.name) {
        if !existing.platforms.contains(&platform) {
            existing.platforms.push(platform);
        }
        return;
    }
    module.platforms.push(platform);
    detected.push(module);
}

fn find_manifest_android_icons(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<AndroidIconsConfig> {
    let icons_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("icons"))?;

    let android = icons_value
        .get("android")
        .and_then(|v| v.as_object())
        .map(|items| {
            items
                .iter()
                .filter_map(|(density, path)| {
                    path.as_str()
                        .map(str::trim)
                        .filter(|p| !p.is_empty())
                        .map(|p| {
                            (
                                density.to_string(),
                                resolve_manifest_asset_path(p, project_root),
                            )
                        })
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    if android.is_empty() {
        return None;
    }

    Some(AndroidIconsConfig { android })
}

fn find_manifest_ios_icons(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<IosIconsConfig> {
    let ios_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("icons"))
        .and_then(|v| v.get("ios"))?;

    let mut ios = BTreeMap::new();
    if let Some(path) = ios_value
        .get("appstore")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        ios.insert(
            "appstore".to_string(),
            resolve_manifest_asset_path(path, project_root),
        );
    }

    for idiom in ["iphone", "ipad"] {
        let Some(items) = ios_value.get(idiom).and_then(|v| v.as_object()) else {
            continue;
        };
        for (slot, path) in items {
            let Some(path) = path.as_str().map(str::trim).filter(|path| !path.is_empty()) else {
                continue;
            };
            ios.insert(
                format!("{}.{}", idiom, slot),
                resolve_manifest_asset_path(path, project_root),
            );
        }
    }

    if ios.is_empty() {
        return None;
    }

    Some(IosIconsConfig { ios })
}

fn find_manifest_push_icons(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<PushIconsConfig> {
    let push_config = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("sdkConfigs"))
        .and_then(|v| v.get("push"))?;

    let mut config = PushIconsConfig::default();
    for small_value in push_small_icon_values(push_config) {
        collect_push_small_icon_value(small_value, project_root, &mut config);
    }

    if config.small.is_none() && config.small_densities.is_empty() {
        None
    } else {
        Some(config)
    }
}

fn push_small_icon_values(push_config: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut values = Vec::new();
    if let Some(value) = push_config
        .get("icons")
        .and_then(|icons| icons.get("small"))
    {
        values.push(value);
    }
    for key in ["unipush", "unipushV2", "uniPush"] {
        if let Some(value) = push_config
            .get(key)
            .and_then(|provider| provider.get("icons"))
            .and_then(|icons| icons.get("small"))
        {
            values.push(value);
        }
    }
    values
}

fn collect_push_small_icon_value(
    value: &serde_json::Value,
    project_root: &Path,
    config: &mut PushIconsConfig,
) {
    if let Some(path) = value
        .as_str()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        config.small = Some(resolve_manifest_asset_path(path, project_root));
        return;
    }

    let Some(items) = value.as_object() else {
        return;
    };
    for (density, path) in items {
        let Some(path) = path.as_str().map(str::trim).filter(|path| !path.is_empty()) else {
            continue;
        };
        config.small_densities.insert(
            density.to_string(),
            resolve_manifest_asset_path(path, project_root),
        );
    }
}

fn find_manifest_splashscreen(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<SplashscreenConfig> {
    let value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("splashscreen"))
        .or_else(|| manifest.get("app-plus").and_then(|v| v.get("splashscreen")))
        .or_else(|| manifest.get("splashscreen"))?;

    let android = value
        .get("android")
        .and_then(|v| v.as_object())
        .map(|items| {
            items
                .iter()
                .filter_map(|(density, path)| {
                    path.as_str()
                        .map(str::trim)
                        .filter(|path| !path.is_empty())
                        .map(|path| {
                            (
                                density.to_string(),
                                resolve_manifest_asset_path(path, project_root),
                            )
                        })
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let config = SplashscreenConfig {
        android_style: string_field(value, &["androidStyle", "android_style"]),
        android,
        use_original_msgbox: bool_field(value, &["useOriginalMsgbox", "use_original_msgbox"]),
    };

    if config.android_style.is_none()
        && config.android.is_empty()
        && config.use_original_msgbox.is_none()
    {
        None
    } else {
        Some(config)
    }
}

fn resolve_manifest_asset_path(path: &str, project_root: &Path) -> String {
    if path.contains("://") || path.starts_with("data:") {
        return path.to_string();
    }
    let path = PathBuf::from(path);
    let absolute = if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    };
    absolute.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest_reads_distribute_app_icons() {
        let project_root =
            std::env::temp_dir().join(format!("unipack-app-icons-{}", uuid::Uuid::new_v4()));
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "icons": {
                        "android": {
                            "hdpi": "unpackage/res/icons/72x72.png",
                            "xhdpi": "unpackage/res/icons/96x96.png"
                        },
                        "ios": {
                            "appstore": "unpackage/res/icons/1024x1024.png",
                            "iphone": {
                                "app@3x": "unpackage/res/icons/180x180.png"
                            },
                            "ipad": {
                                "proapp@2x": "unpackage/res/icons/167x167.png"
                            }
                        }
                    }
                }
            }
        });

        let info = parse_uniapp_manifest(
            &manifest,
            &project_root.join("manifest.json"),
            &project_root,
            None,
        );

        let android = &info
            .android_icons
            .as_ref()
            .expect("Android icons should be parsed")
            .android;
        assert_eq!(
            android.get("xhdpi"),
            Some(
                &project_root
                    .join("unpackage/res/icons/96x96.png")
                    .to_string_lossy()
                    .to_string()
            )
        );

        let ios = &info
            .ios_icons
            .as_ref()
            .expect("iOS icons should be parsed")
            .ios;
        assert_eq!(
            ios.get("appstore"),
            Some(
                &project_root
                    .join("unpackage/res/icons/1024x1024.png")
                    .to_string_lossy()
                    .to_string()
            )
        );
        assert_eq!(
            ios.get("iphone.app@3x"),
            Some(
                &project_root
                    .join("unpackage/res/icons/180x180.png")
                    .to_string_lossy()
                    .to_string()
            )
        );
        assert_eq!(
            ios.get("ipad.proapp@2x"),
            Some(
                &project_root
                    .join("unpackage/res/icons/167x167.png")
                    .to_string_lossy()
                    .to_string()
            )
        );
    }

    #[test]
    fn read_uniapp_manifest_caches_raw_manifest_value() {
        let project_root =
            std::env::temp_dir().join(format!("unipack-manifest-cache-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&project_root).unwrap();
        std::fs::write(
            project_root.join("manifest.json"),
            r#"{
                "appid": "__UNI__CACHE",
                "app-plus": {
                    "distribute": {
                        "icons": {
                            "android": {
                                "hdpi": "unpackage/res/icons/72x72.png"
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        let info = read_uniapp_manifest_sync(project_root.to_str().unwrap()).unwrap();
        let cached = info
            .manifest_value
            .as_ref()
            .expect("raw manifest JSON should be cached");

        assert_eq!(
            cached.get("appid").and_then(|value| value.as_str()),
            Some("__UNI__CACHE")
        );
        assert_eq!(
            cached
                .pointer("/app-plus/distribute/icons/android/hdpi")
                .and_then(|value| value.as_str()),
            Some("unpackage/res/icons/72x72.png")
        );

        let _ = std::fs::remove_dir_all(project_root);
    }

    #[test]
    fn parse_manifest_reads_push_small_icon_path() {
        let project_root =
            std::env::temp_dir().join(format!("unipack-push-icon-{}", uuid::Uuid::new_v4()));
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "icons": {
                                "small": "static/push_icon.png"
                            }
                        }
                    }
                }
            }
        });

        let info = parse_uniapp_manifest(
            &manifest,
            &project_root.join("manifest.json"),
            &project_root,
            None,
        );

        let small_icon = info
            .push_icons
            .as_ref()
            .and_then(|icons| icons.small.as_ref())
            .expect("push small icon should be parsed");
        assert_eq!(
            small_icon,
            &project_root
                .join("static/push_icon.png")
                .to_string_lossy()
                .to_string()
        );
    }

    #[test]
    fn parse_manifest_reads_nested_unipush_small_icon_densities() {
        let project_root = std::env::temp_dir().join(format!(
            "unipack-push-icon-density-{}",
            uuid::Uuid::new_v4()
        ));
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "unipush": {
                                "icons": {
                                    "small": {
                                        "hdpi": "static/push/36x36.png",
                                        "xhdpi": "static/push/48x48.png"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let info = parse_uniapp_manifest(
            &manifest,
            &project_root.join("manifest.json"),
            &project_root,
            None,
        );

        let densities = &info
            .push_icons
            .as_ref()
            .expect("push icons should be parsed")
            .small_densities;
        assert_eq!(
            densities.get("hdpi"),
            Some(
                &project_root
                    .join("static/push/36x36.png")
                    .to_string_lossy()
                    .to_string()
            )
        );
        assert_eq!(
            densities.get("xhdpi"),
            Some(
                &project_root
                    .join("static/push/48x48.png")
                    .to_string_lossy()
                    .to_string()
            )
        );
    }
}
