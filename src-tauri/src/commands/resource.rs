use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniappManifestInfo {
    pub app_name: Option<String>,
    pub app_id: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<u32>,
    pub hbuilderx_version: Option<String>,
    pub icon1024: Option<String>,
    #[serde(default)]
    pub splashscreen: Option<SplashscreenConfig>,
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

fn read_manifest_file(manifest_path: &Path) -> Result<serde_json::Value, String> {
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

fn parse_uniapp_manifest(
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
    let icon1024 = find_manifest_icon(manifest, project_root);
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
        icon1024,
        splashscreen,
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

fn find_manifest_icon(manifest: &serde_json::Value, project_root: &Path) -> Option<String> {
    let app_plus = manifest.get("app-plus");
    let candidate = app_plus
        .and_then(|v| string_field(v, &["icon1024", "icon", "iconPath", "appIcon"]))
        .or_else(|| {
            app_plus
                .and_then(|v| v.get("distribute"))
                .and_then(|v| string_field(v, &["icon1024", "icon", "iconPath", "appIcon"]))
        })
        .or_else(|| string_field(manifest, &["icon1024", "icon", "iconPath", "appIcon"]))
        .or_else(|| find_icon_candidate(manifest))?;
    let path = PathBuf::from(&candidate);
    let absolute = if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    };
    Some(absolute.to_string_lossy().to_string())
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

fn find_icon_candidate(value: &serde_json::Value) -> Option<String> {
    fn visit(value: &serde_json::Value, key_hint: &str, candidates: &mut Vec<(u8, String)>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, item) in map {
                    let hint = if key_hint.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", key_hint, key)
                    };
                    visit(item, &hint, candidates);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    visit(item, key_hint, candidates);
                }
            }
            serde_json::Value::String(path) => {
                let lower_path = path.to_ascii_lowercase();
                let lower_hint = key_hint.to_ascii_lowercase();
                let is_image = lower_path.ends_with(".png")
                    || lower_path.ends_with(".jpg")
                    || lower_path.ends_with(".jpeg")
                    || lower_path.ends_with(".webp");
                let is_icon = lower_hint.contains("icon")
                    || lower_path.contains("icon")
                    || lower_path.contains("logo");
                if is_image && is_icon {
                    let score = if lower_hint.contains("1024")
                        || lower_hint.contains("appstore")
                        || lower_path.contains("1024")
                    {
                        0
                    } else if lower_path.ends_with(".png") {
                        1
                    } else {
                        2
                    };
                    candidates.push((score, path.clone()));
                }
            }
            _ => {}
        }
    }

    let mut candidates = Vec::new();
    visit(value, "", &mut candidates);
    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    candidates.into_iter().map(|(_, path)| path).next()
}

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
        .and_then(|m| {
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

fn extract_hbuilderx_version(manifest: &serde_json::Value) -> Option<String> {
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
    let android_dir = plugin_root.join("utssdk").join("app-android");
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
        id: id.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }

    fn write_manifest(app_dir: &std::path::Path, appid: &str) {
        std::fs::create_dir_all(app_dir).unwrap();
        std::fs::write(
            app_dir.join("manifest.json"),
            format!(
                r#"{{"appid":"{}","name":"Demo","versionName":"1.0.0","versionCode":1}}"#,
                appid
            ),
        )
        .unwrap();
    }

    #[test]
    fn builtin_module_adds_dependency_modules() {
        let root = unique_temp_dir("unipack-uts-scan");
        let choose_media = root.join("uni_modules/uni-chooseMedia");
        std::fs::create_dir_all(&choose_media).unwrap();

        let scan = scan_uts_plugins(&root);

        let names = scan
            .builtin_modules
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"uni-chooseMedia"));
        assert!(names.contains(&"uni-prompt"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn custom_plugin_reads_android_deps() {
        let root = unique_temp_dir("unipack-custom-uts");
        let plugin = root.join("uni_modules/my-plugin/utssdk/app-android");
        std::fs::create_dir_all(&plugin).unwrap();
        std::fs::write(
            plugin.join("config.json"),
            r#"{"dependencies":["a:b:1","c:d:2"]}"#,
        )
        .unwrap();

        let scan = scan_uts_plugins(&root);

        let custom = scan
            .custom_plugins
            .iter()
            .find(|p| p.id == "my-plugin")
            .unwrap();
        assert_eq!(custom.android_deps, vec!["a:b:1", "c:d:2"]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resources_root_with_sibling_uni_modules_is_supported() {
        let root = unique_temp_dir("unipack-resources-root");
        let app_dir = root.join("__UNI__AA97490");
        write_manifest(&app_dir, "__UNI__AA97490");
        std::fs::create_dir_all(root.join("uni_modules/uni-storage")).unwrap();

        let scan = scan_imported_resource(&root, &root, false).unwrap();

        assert_eq!(scan.app_id, "__UNI__AA97490");
        assert_eq!(std::path::PathBuf::from(scan.imported_path), root);
        assert_eq!(std::path::PathBuf::from(scan.app_resource_path), app_dir);
        assert!(scan
            .uts
            .builtin_modules
            .iter()
            .any(|m| m.name == "uni-storage"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resources_root_without_uni_modules_keeps_package_root() {
        let temp = unique_temp_dir("unipack-no-modules");
        let root = temp.join("resources");
        let app_dir = root.join("__UNI__NO_MODULES");
        write_manifest(&app_dir, "__UNI__NO_MODULES");

        let scan = scan_imported_resource(&root, &root, false).unwrap();

        assert_eq!(scan.app_id, "__UNI__NO_MODULES");
        assert_eq!(std::path::PathBuf::from(scan.imported_path), root);
        assert_eq!(std::path::PathBuf::from(scan.app_resource_path), app_dir);
        assert!(!scan.uts.has_uts_plugins);

        let _ = std::fs::remove_dir_all(temp);
    }

    #[test]
    fn direct_uni_dir_stays_compatible() {
        let root = unique_temp_dir("__UNI__DIRECT");
        write_manifest(&root, "__UNI__DIRECT");

        let scan = scan_imported_resource(&root, &root, false).unwrap();

        assert_eq!(scan.app_id, "__UNI__DIRECT");
        assert_eq!(scan.imported_path, root.to_string_lossy());
        assert_eq!(scan.app_resource_path, root.to_string_lossy());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn wrapped_zip_extract_layout_resolves_resources_root() {
        let extract_root = unique_temp_dir("unipack-zip-layout");
        let resources = extract_root.join("outer/resources");
        let app_dir = resources.join("__UNI__ZIP");
        write_manifest(&app_dir, "__UNI__ZIP");
        std::fs::create_dir_all(resources.join("uni_modules/uni-getNetworkType")).unwrap();

        let package_root = find_uniapp_package_root(&extract_root).unwrap();
        let scan = scan_imported_resource(&extract_root, &package_root, true).unwrap();

        assert_eq!(package_root, resources);
        assert_eq!(scan.app_id, "__UNI__ZIP");
        assert_eq!(std::path::PathBuf::from(scan.app_resource_path), app_dir);
        assert!(scan
            .uts
            .builtin_modules
            .iter()
            .any(|m| m.name == "uni-getNetworkType"));

        let _ = std::fs::remove_dir_all(extract_root);
    }

    #[test]
    fn manifest_info_reads_basic_android_and_modules() {
        let root = unique_temp_dir("unipack-manifest-info");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(root.join("static/storyboard")).unwrap();
        std::fs::write(root.join("static/icon.png"), "fake").unwrap();
        std::fs::write(root.join("static/storyboard/480x762.9.png"), "fake").unwrap();
        std::fs::write(root.join("static/storyboard/720x1242.9.png"), "fake").unwrap();
        std::fs::write(
            root.join("manifest.json"),
            r#"{
                "name": "Care",
                "appid": "__UNI__AA97490",
                "versionName": "1.2.3",
                "versionCode": "12",
                "app-plus": {
                    "icon": "static/icon.png",
                    "distribute": {
                        "android": {
                            "packageName": "com.example.care",
                            "minSdkVersion": 23,
                            "targetSdkVersion": "35",
                            "compileSdkVersion": 35,
                            "modules": [{"name":"Push"}, {"name":"Maps"}]
                        },
                        "ios": {
                            "bundleId": "com.example.care",
                            "modules": [{"name":"Share"}]
                        }
                    },
                    "splashscreen": {
                        "androidStyle": "default",
                        "android": {
                            "hdpi": "static/storyboard/480x762.9.png",
                            "xhdpi": "static/storyboard/720x1242.9.png"
                        },
                        "useOriginalMsgbox": true
                    }
                }
            }"#,
        )
        .unwrap();

        let info = read_uniapp_manifest_sync(&root.to_string_lossy()).unwrap();

        assert_eq!(info.app_name.as_deref(), Some("Care"));
        assert_eq!(info.app_id.as_deref(), Some("__UNI__AA97490"));
        assert_eq!(info.version_name.as_deref(), Some("1.2.3"));
        assert_eq!(info.version_code, Some(12));
        assert_eq!(
            info.android.package_name.as_deref(),
            Some("com.example.care")
        );
        assert_eq!(info.android.min_sdk_version, Some(23));
        assert_eq!(info.android.target_sdk_version, Some(35));
        assert_eq!(info.android.compile_sdk_version, Some(35));
        assert_eq!(
            info.icon1024.as_deref(),
            Some(root.join("static/icon.png").to_string_lossy().as_ref())
        );
        let splashscreen = info.splashscreen.as_ref().unwrap();
        assert_eq!(splashscreen.android_style.as_deref(), Some("default"));
        assert_eq!(splashscreen.use_original_msgbox, Some(true));
        assert_eq!(
            splashscreen.android.get("hdpi").map(String::as_str),
            Some(
                root.join("static/storyboard/480x762.9.png")
                    .to_string_lossy()
                    .as_ref()
            )
        );
        assert!(info.detected_modules.iter().any(|m| m.name == "Push"));
        assert!(info.detected_modules.iter().any(|m| m.name == "Maps"));
        assert!(info.detected_modules.iter().any(|m| m.name == "Share"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_info_accepts_hbuilderx_json5_style() {
        let root = unique_temp_dir("unipack-manifest-json5");
        std::fs::create_dir_all(root.join("unpackage/res/icons")).unwrap();
        std::fs::write(root.join("unpackage/res/icons/1024x1024.png"), "fake").unwrap();
        std::fs::write(
            root.join("manifest.json"),
            r#"{
                // HBuilderX sometimes keeps comments and unquoted keys here.
                name: 'Json5 Care',
                appid: '__UNI__JSON5',
                versionName: '2.0.0',
                versionCode: 20,
                "app-plus": {
                    distribute: {
                        android: {
                            packageName: 'com.example.json5',
                            minSdkVersion: '24',
                            targetSdkVersion: 35,
                            compileSdkVersion: 35,
                            modules: {
                                Push: true,
                                Share: { enabled: false },
                            },
                        },
                    },
                    icons: {
                        android: {
                            hdpi: 'unpackage/res/icons/1024x1024.png',
                        },
                    },
                },
            }"#,
        )
        .unwrap();

        let info = read_uniapp_manifest_sync(&root.to_string_lossy()).unwrap();

        assert_eq!(info.app_name.as_deref(), Some("Json5 Care"));
        assert_eq!(info.app_id.as_deref(), Some("__UNI__JSON5"));
        assert_eq!(
            info.android.package_name.as_deref(),
            Some("com.example.json5")
        );
        assert_eq!(info.android.min_sdk_version, Some(24));
        assert_eq!(
            info.icon1024.as_deref(),
            Some(
                root.join("unpackage/res/icons/1024x1024.png")
                    .to_string_lossy()
                    .as_ref()
            )
        );
        assert!(info.detected_modules.iter().any(|m| m.name == "Push"));
        assert!(!info.detected_modules.iter().any(|m| m.name == "Share"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_info_keeps_hbuilderx_module_names_from_local_project() {
        let root = unique_temp_dir("unipack-manifest-local-modules");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("manifest.json"),
            r#"{
                "name": "Local Modules",
                "appid": "__UNI__LOCAL",
                "app-plus": {
                    "modules": {
                        "OAuth": {},
                        "Push": {},
                        "Camera": {},
                        "Payment": {},
                        "Share": {},
                        "VideoPlayer": {}
                    }
                }
            }"#,
        )
        .unwrap();

        let info = read_uniapp_manifest_sync(&root.to_string_lossy()).unwrap();
        let names: Vec<&str> = info
            .detected_modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();

        assert!(names.contains(&"OAuth"));
        assert!(names.contains(&"Push"));
        assert!(names.contains(&"Camera"));
        assert!(names.contains(&"Payment"));
        assert!(names.contains(&"Share"));
        assert!(names.contains(&"VideoPlayer"));
        assert_eq!(info.detected_modules.len(), 6);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_info_defaults_min_sdk_to_21_when_missing() {
        let root = unique_temp_dir("unipack-manifest-default-min-sdk");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("manifest.json"),
            r#"{
                "name": "Default Min SDK",
                "appid": "__UNI__MINSDK",
                "app-plus": {
                    "distribute": {
                        "android": {
                            "targetSdkVersion": 35
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        let info = read_uniapp_manifest_sync(&root.to_string_lossy()).unwrap();

        assert_eq!(info.android.min_sdk_version, Some(21));
        assert_eq!(info.android.target_sdk_version, Some(35));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_info_detects_modules_from_sdk_configs_when_module_is_enabled() {
        let root = unique_temp_dir("unipack-manifest-sdk-configs");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("manifest.json"),
            r#"{
                "appid": "__UNI__SDKCONFIGS",
                "app-plus": {
                    "modules": {
                        "Share": {},
                        "OAuth": {},
                        "Maps": {},
                        "Payment": false
                    },
                    "distribute": {
                        "sdkConfigs": {
                            "share": { "weixin": { "appid": "wx" } },
                            "payment": { "weixin": { "appid": "wx" } },
                            "oauth": { "qq": { "appid": "qq" } }
                        },
                        "android": {
                            "sdkConfigs": {
                                "maps": { "amap": { "key": "amap" } }
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        let info = read_uniapp_manifest_sync(&root.to_string_lossy()).unwrap();
        let names = info
            .detected_modules
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"Share"));
        assert!(names.contains(&"OAuth"));
        assert!(names.contains(&"Maps"));
        assert!(!names.contains(&"Payment"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_info_ignores_sdk_configs_without_enabled_module_declaration() {
        let root = unique_temp_dir("unipack-manifest-disabled-module-sdk-configs");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("manifest.json"),
            r#"{
                "appid": "__UNI__DISABLED",
                "app-plus": {
                    "modules": {
                        "OAuth": false,
                        "Share": {}
                    },
                    "distribute": {
                        "sdkConfigs": {
                            "oauth": { "weixin": { "appid": "wx" } },
                            "share": { "weixin": { "appid": "wx" } }
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        let info = read_uniapp_manifest_sync(&root.to_string_lossy()).unwrap();
        let names = info
            .detected_modules
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"Share"));
        assert!(!names.contains(&"OAuth"));

        let _ = std::fs::remove_dir_all(root);
    }
}
