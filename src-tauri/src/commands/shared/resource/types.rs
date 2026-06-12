use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub exclude_permissions: Vec<String>,
    #[serde(default)]
    pub schemes: Vec<String>,
    #[serde(default)]
    pub abi_filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SplashscreenConfig {
    pub android_style: Option<String>,
    pub android: BTreeMap<String, String>,
    pub ios_style: Option<String>,
    pub ios_storyboard: Option<String>,
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

/// Push 通知图标配置（来自 manifest.json app-plus.distribute.push 或 sdkConfigs.push）
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
    pub ios_privacy_descriptions: BTreeMap<String, String>,
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

#[derive(Debug, Deserialize)]
pub struct ResourceImportInput {
    pub path: String,
    #[serde(rename = "type")]
    pub r#type: String,
}
