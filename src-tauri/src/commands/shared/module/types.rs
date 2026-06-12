use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleConfigTree {
    pub push: Option<PushModuleConfig>,
    pub geolocation: Option<LocationConfig>,
    pub share: Option<ShareModuleConfig>,
    pub login: Option<LoginModuleConfig>,
    pub payment: Option<PaymentModuleConfig>,
    pub map: Option<MapModuleConfig>,
    pub speech: Option<SpeechModuleConfig>,
    pub statistic: Option<StatisticModuleConfig>,
    pub face_recognition: Option<FaceRecognitionModuleConfig>,
    pub uni_ad: Option<UniAdModuleConfig>,
    pub x5_tbs: Option<SimpleModuleConfig>,
    pub livepusher: Option<LivePusherModuleConfig>,
    pub camera: Option<SimpleModuleConfig>,
    pub video_player: Option<SimpleModuleConfig>,
    pub barcode: Option<SimpleModuleConfig>,
    pub bluetooth: Option<SimpleModuleConfig>,
    pub ibeacon: Option<SimpleModuleConfig>,
    pub contacts: Option<SimpleModuleConfig>,
    pub fingerprint: Option<SimpleModuleConfig>,
    pub messaging: Option<SimpleModuleConfig>,
    pub record: Option<SimpleModuleConfig>,
    pub sqlite: Option<SimpleModuleConfig>,
    pub gcanvas: Option<SimpleModuleConfig>,
    pub ui_webview: Option<SimpleModuleConfig>,
    pub uts_plugins: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushModuleConfig {
    pub enabled: bool,
    pub unipush_appid: Option<String>,
    pub unipush_appkey: Option<String>,
    pub unipush_appsecret: Option<String>,
    pub vendors: Vec<VendorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorConfig {
    pub name: String,
    pub enabled: bool,
    pub config: HashMap<String, String>,
    /// 文件类型配置项：key -> base64 编码的文件内容
    #[serde(default)]
    pub file_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    pub enabled: bool,
    pub engine: String,
    pub baidu_ak: Option<String>,
    pub amap_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareModuleConfig {
    pub enabled: bool,
    pub weixin: Option<HashMap<String, String>>,
    pub qq: Option<HashMap<String, String>>,
    pub sina: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginModuleConfig {
    pub enabled: bool,
    pub providers: Vec<LoginProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginProvider {
    pub name: String,
    pub enabled: bool,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentModuleConfig {
    pub enabled: bool,
    pub weixin: Option<HashMap<String, String>>,
    pub alipay: Option<HashMap<String, String>>,
    #[serde(default)]
    pub paypal: Option<HashMap<String, String>>,
    #[serde(default)]
    pub stripe: Option<HashMap<String, String>>,
    #[serde(default)]
    pub google: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapModuleConfig {
    pub enabled: bool,
    pub engine: String,
    pub amap_key: Option<String>,
    pub tencent_map_key: Option<String>,
    pub baidu_map_ak: Option<String>,
    pub google_maps_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechModuleConfig {
    pub enabled: bool,
    pub engine: String,
    pub xfyun: Option<HashMap<String, String>>,
    pub baidu: Option<HashMap<String, String>>,
    pub aliyun: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticModuleConfig {
    pub enabled: bool,
    pub provider: String,
    pub umeng: Option<HashMap<String, String>>,
    pub mta: Option<HashMap<String, String>>,
    pub baidu: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceRecognitionModuleConfig {
    pub enabled: bool,
    pub provider: String,
    pub dcloud: Option<HashMap<String, String>>,
    pub baidu: Option<HashMap<String, String>>,
    pub aliyun: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniAdModuleConfig {
    pub enabled: bool,
    pub csj: Option<HashMap<String, String>>,
    pub gdt: Option<HashMap<String, String>>,
    pub gromore: Option<HashMap<String, String>>,
    pub admob: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleModuleConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePusherModuleConfig {
    pub enabled: bool,
    pub license_url: Option<String>,
    pub license_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTemplate {
    pub module_name: String,
    pub description: String,
    pub android_config: AndroidModuleTemplate,
    pub ios_config: IosModuleTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidModuleConfigReport {
    pub modules: Vec<AndroidModuleConfigModule>,
    pub missing_required: Vec<AndroidModuleMissingConfig>,
    pub all_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidModuleConfigModule {
    pub name: String,
    pub template_key: String,
    pub category: String,
    pub platforms: Vec<String>,
    pub source: String,
    pub fields: Vec<AndroidModuleConfigField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidModuleConfigField {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub secret: bool,
    pub value: Option<String>,
    pub value_source: Option<String>,
    pub placeholder: String,
    /// 字段类型：text（文本输入）或 file（文件选择，值为 base64）
    #[serde(default)]
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidModuleMissingConfig {
    pub module_name: String,
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IosModuleConfigReport {
    pub modules: Vec<IosModuleConfigModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IosModuleConfigModule {
    pub name: String,
    pub template_key: String,
    pub category: String,
    pub platforms: Vec<String>,
    pub source: String,
    pub fields: Vec<IosModuleConfigField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IosModuleConfigField {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub secret: bool,
    pub value: Option<String>,
    pub value_source: Option<String>,
    pub placeholder: String,
    #[serde(default)]
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidModuleTemplate {
    pub required_aars: Vec<String>,
    /// 厂商 SDK 的本地 AAR 文件（从 DCloud SDK/libs 复制），仅当用户配置了对应厂商时才注入
    /// 与 required_aars 的区别：required_aars 是模块核心依赖（始终复制），
    /// vendor_aars 是按厂商条件复制的可选依赖
    pub vendor_aars: Vec<String>,
    pub gradle_dependencies: Vec<String>,
    pub manifest_placeholders: Vec<String>,
    pub manifest_meta_data: Vec<HashMap<String, String>>,
    pub activities: Vec<String>,
    pub properties_xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosModuleTemplate {
    pub required_frameworks: Vec<String>,
    pub required_libraries: Vec<String>,
    pub info_plist_keys: HashMap<String, String>,
    pub url_schemes: Vec<UrlSchemeConfig>,
    pub plist_entry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlSchemeConfig {
    pub scheme: String,
    pub identifier: String,
}

pub(crate) struct AndroidConfigFieldSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub required: bool,
    pub secret: bool,
    pub placeholder: &'static str,
    pub aliases: &'static [&'static str],
    pub path_hints: &'static [&'static str],
    /// 字段类型：text（文本输入）或 file（文件选择）
    #[allow(dead_code)]
    pub field_type: &'static str,
}

fn get_uni_pack_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
}

pub fn get_module_config_path(project_id: &str) -> PathBuf {
    get_uni_pack_home()
        .join("module-configs")
        .join(format!("{}.json", project_id))
}
