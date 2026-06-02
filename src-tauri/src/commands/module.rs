use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AndroidModuleMissingConfig {
    pub module_name: String,
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidModuleTemplate {
    pub required_aars: Vec<String>,
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

struct AndroidConfigFieldSpec {
    key: &'static str,
    label: &'static str,
    required: bool,
    secret: bool,
    placeholder: &'static str,
    aliases: &'static [&'static str],
    path_hints: &'static [&'static str],
}

fn get_uni_pack_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
}

fn get_module_config_path(project_id: &str) -> PathBuf {
    get_uni_pack_home()
        .join("module-configs")
        .join(format!("{}.json", project_id))
}

#[tauri::command]
pub async fn parse_project_modules(project_path: String) -> Result<ModuleConfigTree, String> {
    let project_dir = PathBuf::from(&project_path);
    let manifest_path = project_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("manifest.json not found at {}", project_path));
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest.json: {}", e))?;

    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

    let mut tree = ModuleConfigTree::default();

    if let Some(app_plus) = manifest.get("app-plus") {
        if let Some(distribute) = app_plus.get("distribute") {
            if let Some(android) = distribute.get("android") {
                if let Some(modules) = android.get("modules") {
                    if let Some(mods_array) = modules.as_array() {
                        for mod_val in mods_array {
                            if let Some(name) = mod_val.get("name").and_then(|n| n.as_str()) {
                                apply_module_name_to_tree(&mut tree, name);
                            }
                        }
                    }
                }
            }
        }
    }

    let props_path = project_dir
        .join("assets")
        .join("data")
        .join("dcloud_properties.xml");
    if props_path.exists() {
        if let Ok(props_content) = fs::read_to_string(&props_path) {
            merge_properties_to_tree(&mut tree, &props_content)?;
        }
    }

    Ok(tree)
}

pub fn module_config_from_detected_modules(
    modules: &[crate::commands::resource::DetectedModule],
) -> ModuleConfigTree {
    let mut tree = ModuleConfigTree::default();
    for module in modules {
        apply_module_name_to_tree(&mut tree, &module.name);
    }
    tree
}

#[tauri::command]
pub async fn analyze_android_module_config(
    manifest_info: crate::commands::resource::UniappManifestInfo,
    user_config: Option<HashMap<String, String>>,
) -> Result<AndroidModuleConfigReport, String> {
    Ok(analyze_android_module_config_sync(
        &manifest_info,
        user_config.as_ref(),
    ))
}

pub fn analyze_android_module_config_sync(
    manifest_info: &crate::commands::resource::UniappManifestInfo,
    user_config: Option<&HashMap<String, String>>,
) -> AndroidModuleConfigReport {
    let manifest_value = std::fs::read_to_string(&manifest_info.manifest_path)
        .ok()
        .and_then(|content| json5::from_str::<serde_json::Value>(&content).ok())
        .or_else(|| manifest_info_to_value(manifest_info));
    android_module_config_report_from_value(
        &manifest_info.detected_modules,
        manifest_value.as_ref(),
        user_config,
    )
}

pub fn android_module_config_report_from_value(
    modules: &[crate::commands::resource::DetectedModule],
    manifest: Option<&serde_json::Value>,
    user_config: Option<&HashMap<String, String>>,
) -> AndroidModuleConfigReport {
    let mut report = AndroidModuleConfigReport::default();

    for module in modules {
        let Some(template_key) = android_module_template_key(&module.name) else {
            continue;
        };
        if !module_applies_to_android(&module.platforms) {
            continue;
        }

        let mut fields = Vec::new();
        for spec in android_config_field_specs(template_key) {
            if !android_field_visible_for_manifest(template_key, spec, manifest) {
                continue;
            }
            let required = android_field_required_for_manifest(template_key, spec, manifest);
            let user_value = user_config
                .and_then(|config| config.get(spec.key))
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());
            let manifest_value = manifest.and_then(|value| find_manifest_config_value(value, spec));
            let (value, value_source) = if let Some(value) = manifest_value {
                (Some(value), Some("manifest".to_string()))
            } else if let Some(value) = user_value {
                (Some(value.to_string()), Some("user".to_string()))
            } else {
                (None, None)
            };

            let field = AndroidModuleConfigField {
                key: spec.key.to_string(),
                label: spec.label.to_string(),
                required,
                secret: spec.secret,
                value,
                value_source,
                placeholder: spec.placeholder.to_string(),
            };

            if field.required
                && field
                    .value
                    .as_deref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true)
            {
                report.missing_required.push(AndroidModuleMissingConfig {
                    module_name: module.name.clone(),
                    key: field.key.clone(),
                    label: field.label.clone(),
                });
            }
            fields.push(field);
        }

        if !fields.is_empty() {
            report.modules.push(AndroidModuleConfigModule {
                name: module.name.clone(),
                template_key: template_key.to_string(),
                category: module.category.clone(),
                platforms: module.platforms.clone(),
                source: module.source.clone(),
                fields,
            });
        }
    }

    report.all_configured = report.missing_required.is_empty();
    report
}

fn android_field_visible_for_manifest(
    template_key: &str,
    spec: &AndroidConfigFieldSpec,
    manifest: Option<&serde_json::Value>,
) -> bool {
    let Some(manifest) = manifest else {
        return true;
    };

    match template_key {
        "push" => match spec.key {
            "XIAOMI_APP_ID" | "XIAOMI_APP_KEY" => manifest_has_enabled_provider(
                manifest,
                &["push", "unipush", "unipushV2", "uniPush"],
                &["xiaomi", "mi"],
            ),
            "MEIZU_APP_ID" | "MEIZU_APP_KEY" => manifest_has_enabled_provider(
                manifest,
                &["push", "unipush", "unipushV2", "uniPush"],
                &["meizu", "mz"],
            ),
            "HUAWEI_APP_ID" => manifest_has_enabled_provider(
                manifest,
                &["push", "unipush", "unipushV2", "uniPush"],
                &["huawei", "hms", "hw"],
            ),
            "OPPO_APP_KEY" | "OPPO_APP_SECRET" => manifest_has_enabled_provider(
                manifest,
                &["push", "unipush", "unipushV2", "uniPush"],
                &["oppo"],
            ),
            "VIVO_APP_ID" | "VIVO_APP_KEY" => manifest_has_enabled_provider(
                manifest,
                &["push", "unipush", "unipushV2", "uniPush"],
                &["vivo"],
            ),
            "HONOR_APP_ID" => manifest_has_enabled_provider(
                manifest,
                &["push", "unipush", "unipushV2", "uniPush"],
                &["honor"],
            ),
            _ => true,
        },
        "geolocation" => match spec.key {
            "BAIDU_MAP_AK" => manifest_has_enabled_provider(
                manifest,
                &["geolocation", "location", "position"],
                &["baidu", "bd"],
            ),
            "AMAP_KEY" => manifest_has_enabled_provider(
                manifest,
                &["geolocation", "location", "position"],
                &["amap", "gaode"],
            ),
            "TENCENT_MAP_KEY" => manifest_has_enabled_provider(
                manifest,
                &["geolocation", "location", "position"],
                &["tencent", "qqmap"],
            ),
            _ => true,
        },
        "share" => match spec.key {
            "WX_APPID" | "WX_SECRET" => manifest_has_enabled_provider(
                manifest,
                &["share", "shares"],
                &["weixin", "wechat", "wx"],
            ),
            "QQ_APPID" => manifest_has_enabled_provider(manifest, &["share", "shares"], &["qq"]),
            "SINA_APPKEY" | "SINA_SECRET" | "SINA_REDIRECT_URI" => manifest_has_enabled_provider(
                manifest,
                &["share", "shares"],
                &["sina", "weibo", "sinaweibo"],
            ),
            _ => true,
        },
        "login" => match spec.key {
            "WX_APPID" | "WX_SECRET" => manifest_has_enabled_provider(
                manifest,
                &["oauth", "login", "oauths"],
                &["weixin", "wechat", "wx"],
            ),
            "QQ_APPID" => {
                manifest_has_enabled_provider(manifest, &["oauth", "login", "oauths"], &["qq"])
            }
            "GY_APP_ID" => manifest_has_enabled_provider(
                manifest,
                &["oauth", "login", "oauths"],
                &["univerify", "igetui", "getui"],
            ),
            "SINA_APPKEY" | "SINA_REDIRECT_URI" => manifest_has_enabled_provider(
                manifest,
                &["oauth", "login", "oauths"],
                &["sina", "weibo", "sinaweibo"],
            ),
            "MIUI_APPID" | "MIUI_APPSECRET" | "MIUI_REDIRECT_URI" => manifest_has_enabled_provider(
                manifest,
                &["oauth", "login", "oauths"],
                &["miui", "xiaomi", "mi"],
            ),
            "FACEBOOK_APP_ID" | "FACEBOOK_CLIENT_TOKEN" => manifest_has_enabled_provider(
                manifest,
                &["oauth", "login", "oauths"],
                &["facebook", "fb"],
            ),
            _ => true,
        },
        "map" => match spec.key {
            "BAIDU_MAP_AK" => {
                manifest_has_enabled_provider(manifest, &["maps", "map"], &["baidu", "bd"])
            }
            "AMAP_KEY" => {
                manifest_has_enabled_provider(manifest, &["maps", "map"], &["amap", "gaode"])
            }
            "GOOGLE_MAPS_API_KEY" => {
                manifest_has_enabled_provider(manifest, &["maps", "map"], &["google"])
            }
            "TENCENT_MAP_KEY" => {
                manifest_has_enabled_provider(manifest, &["maps", "map"], &["tencent", "qqmap"])
            }
            _ => true,
        },
        "payment" => match spec.key {
            "WX_APPID" => manifest_has_enabled_provider(
                manifest,
                &["payment", "pay", "payments"],
                &["weixin", "wechat", "wx"],
            ),
            "PAYPAL_RETURN_SCHEME" => manifest_has_enabled_provider(
                manifest,
                &["payment", "pay", "payments"],
                &["paypal"],
            ),
            _ => true,
        },
        "speech" => match spec.key {
            "BAIDU_SPEECH_APP_ID" | "BD_SPEECH_APIKEY" | "BD_SPEECH_SECRETKEY" => {
                manifest_has_enabled_provider(
                    manifest,
                    &["speech", "speechRecognition"],
                    &["baidu"],
                )
            }
            "IFLY_APPID" => manifest_has_enabled_provider(
                manifest,
                &["speech", "speechRecognition"],
                &["ifly", "xfyun", "xunfei"],
            ),
            _ => true,
        },
        "statistic" => match spec.key {
            "UMENG_APPKEY" | "UMENG_CHANNEL" => manifest_has_enabled_provider(
                manifest,
                &["statistic", "statistics", "statics"],
                &["umeng"],
            ),
            _ => true,
        },
        "face_recognition" => match spec.key {
            "DCLOUD_LICENSE" => manifest_has_enabled_provider(
                manifest,
                &[
                    "facialRecognitionVerify",
                    "faceRecognition",
                    "face_recognition",
                    "facial",
                    "face",
                    "realname",
                ],
                &["dcloud"],
            ),
            "BDFACE_APIKEY" | "BDFACE_SECRETKEY" => manifest_has_enabled_provider(
                manifest,
                &[
                    "facialRecognitionVerify",
                    "faceRecognition",
                    "face_recognition",
                    "facial",
                    "face",
                    "realname",
                ],
                &["baidu", "bd"],
            ),
            "ALIFACE_ACCESSKEY_ID" | "ALIFACE_ACCESSKEY_SECRET" => manifest_has_enabled_provider(
                manifest,
                &[
                    "facialRecognitionVerify",
                    "faceRecognition",
                    "face_recognition",
                    "facial",
                    "face",
                    "realname",
                ],
                &["aliyun", "ali"],
            ),
            _ => true,
        },
        _ => true,
    }
}

pub fn android_module_artifact_enabled_for_manifest(
    template_key: &str,
    artifact: &str,
    manifest: Option<&serde_json::Value>,
) -> bool {
    android_module_entry_enabled_for_manifest(template_key, artifact, manifest)
}

pub fn android_module_gradle_dependency_enabled_for_manifest(
    template_key: &str,
    dependency: &str,
    manifest: Option<&serde_json::Value>,
) -> bool {
    android_module_entry_enabled_for_manifest(template_key, dependency, manifest)
}

pub fn android_module_gradle_repositories_for_manifest(
    template_key: &str,
    manifest: Option<&serde_json::Value>,
) -> Vec<&'static str> {
    let Some(manifest) = manifest else {
        return match template_key {
            "push" => vec![
                "maven { url 'https://mvn.getui.com/nexus/content/repositories/releases' }",
                "maven { url 'https://developer.huawei.com/repo/' }",
                "maven { url 'https://developer.hihonor.com/repo/' }",
            ],
            _ => Vec::new(),
        };
    };

    match template_key {
        "push" => {
            let mut repos =
                vec!["maven { url 'https://mvn.getui.com/nexus/content/repositories/releases' }"];
            if push_provider_enabled(manifest, &["huawei", "hms", "hw"]) {
                repos.push("maven { url 'https://developer.huawei.com/repo/' }");
            }
            if push_provider_enabled(manifest, &["honor"]) {
                repos.push("maven { url 'https://developer.hihonor.com/repo/' }");
            }
            repos
        }
        _ => Vec::new(),
    }
}

fn android_module_entry_enabled_for_manifest(
    template_key: &str,
    entry: &str,
    manifest: Option<&serde_json::Value>,
) -> bool {
    let Some(manifest) = manifest else {
        return true;
    };
    let note = android_entry_provider_note(entry);

    match template_key {
        "push" => provider_entry_enabled(
            &note,
            &[
                (
                    &["xiaomi", "mi", "小米"][..],
                    &["push", "unipush", "unipushV2", "uniPush"][..],
                    &["xiaomi", "mi"][..],
                ),
                (
                    &["meizu", "mz", "魅族"][..],
                    &["push", "unipush", "unipushV2", "uniPush"][..],
                    &["meizu", "mz"][..],
                ),
                (
                    &["huawei", "hms", "华为"][..],
                    &["push", "unipush", "unipushV2", "uniPush"][..],
                    &["huawei", "hms", "hw"][..],
                ),
                (
                    &["oppo"][..],
                    &["push", "unipush", "unipushV2", "uniPush"][..],
                    &["oppo"][..],
                ),
                (
                    &["vivo"][..],
                    &["push", "unipush", "unipushV2", "uniPush"][..],
                    &["vivo"][..],
                ),
                (
                    &["honor", "荣耀"][..],
                    &["push", "unipush", "unipushV2", "uniPush"][..],
                    &["honor"][..],
                ),
            ],
            manifest,
        ),
        "share" => provider_entry_enabled(
            &note,
            &[
                (
                    &["weixin", "wechat", "wx", "微信"][..],
                    &["share", "shares"][..],
                    &["weixin", "wechat", "wx"][..],
                ),
                (&["qq"][..], &["share", "shares"][..], &["qq"][..]),
                (
                    &["sina", "weibo", "微博"][..],
                    &["share", "shares"][..],
                    &["sina", "weibo", "sinaweibo"][..],
                ),
            ],
            manifest,
        ),
        "login" => provider_entry_enabled(
            &note,
            &[
                (
                    &["univerify", "igetui", "getui", "一键登录"][..],
                    &["oauth", "login", "oauths"][..],
                    &["univerify", "igetui", "getui"][..],
                ),
                (
                    &["weixin", "wechat", "wx", "微信"][..],
                    &["oauth", "login", "oauths"][..],
                    &["weixin", "wechat", "wx"][..],
                ),
                (&["qq"][..], &["oauth", "login", "oauths"][..], &["qq"][..]),
                (
                    &["sina", "weibo", "微博"][..],
                    &["oauth", "login", "oauths"][..],
                    &["sina", "weibo", "sinaweibo"][..],
                ),
                (
                    &["miui", "xiaomi", "mi", "小米"][..],
                    &["oauth", "login", "oauths"][..],
                    &["miui", "xiaomi", "mi"][..],
                ),
                (
                    &["google"][..],
                    &["oauth", "login", "oauths"][..],
                    &["google"][..],
                ),
                (
                    &["facebook", "fb"][..],
                    &["oauth", "login", "oauths"][..],
                    &["facebook", "fb"][..],
                ),
            ],
            manifest,
        ),
        "geolocation" => provider_entry_enabled(
            &note,
            &[
                (
                    &["baidu", "bd", "百度"][..],
                    &["geolocation", "location", "position"][..],
                    &["baidu", "bd"][..],
                ),
                (
                    &["amap", "gaode", "高德"][..],
                    &["geolocation", "location", "position"][..],
                    &["amap", "gaode"][..],
                ),
                (
                    &["tencent", "qqmap", "腾讯"][..],
                    &["geolocation", "location", "position"][..],
                    &["tencent", "qqmap"][..],
                ),
            ],
            manifest,
        ),
        "payment" => provider_entry_enabled(
            &note,
            &[
                (
                    &["alipay", "支付宝"][..],
                    &["payment", "pay", "payments"][..],
                    &["alipay"][..],
                ),
                (
                    &["weixin", "wechat", "wx", "微信"][..],
                    &["payment", "pay", "payments"][..],
                    &["weixin", "wechat", "wx"][..],
                ),
                (
                    &["paypal"][..],
                    &["payment", "pay", "payments"][..],
                    &["paypal"][..],
                ),
                (
                    &["stripe"][..],
                    &["payment", "pay", "payments"][..],
                    &["stripe"][..],
                ),
                (
                    &["google"][..],
                    &["payment", "pay", "payments"][..],
                    &["google", "googlepay", "google_pay"][..],
                ),
            ],
            manifest,
        ),
        "map" => provider_entry_enabled(
            &note,
            &[
                (
                    &["baidu", "bd", "百度"][..],
                    &["maps", "map"][..],
                    &["baidu", "bd"][..],
                ),
                (
                    &["amap", "gaode", "高德"][..],
                    &["maps", "map"][..],
                    &["amap", "gaode"][..],
                ),
                (&["google"][..], &["maps", "map"][..], &["google"][..]),
                (
                    &["tencent", "qqmap", "腾讯"][..],
                    &["maps", "map"][..],
                    &["tencent", "qqmap"][..],
                ),
            ],
            manifest,
        ),
        "statistic" => provider_entry_enabled(
            &note,
            &[
                (
                    &["umeng", "友盟"][..],
                    &["statistic", "statistics", "statics"][..],
                    &["umeng"][..],
                ),
                (
                    &["google", "谷歌"][..],
                    &["statistic", "statistics", "statics"][..],
                    &["google"][..],
                ),
            ],
            manifest,
        ),
        "speech" => provider_entry_enabled(
            &note,
            &[
                (
                    &["baidu", "bd", "百度"][..],
                    &["speech", "speechRecognition"][..],
                    &["baidu"][..],
                ),
                (
                    &["ifly", "xfyun", "xunfei", "讯飞"][..],
                    &["speech", "speechRecognition"][..],
                    &["ifly", "xfyun", "xunfei"][..],
                ),
            ],
            manifest,
        ),
        "uni_ad" => provider_entry_enabled(
            &note,
            &[
                (
                    &["csj", "chuanshanjia", "穿山甲"][..],
                    &["ad", "ads", "uni-ad", "uniAD", "uniad"][..],
                    &["csj", "chuanshanjia"][..],
                ),
                (
                    &["gdt", "youlianghui", "优量汇"][..],
                    &["ad", "ads", "uni-ad", "uniAD", "uniad"][..],
                    &["gdt", "youlianghui"][..],
                ),
                (
                    &["gromore"][..],
                    &["ad", "ads", "uni-ad", "uniAD", "uniad"][..],
                    &["gromore"][..],
                ),
                (
                    &["admob"][..],
                    &["ad", "ads", "uni-ad", "uniAD", "uniad"][..],
                    &["admob"][..],
                ),
                (
                    &["huawei", "hms", "华为"][..],
                    &["ad", "ads", "uni-ad", "uniAD", "uniad"][..],
                    &["huawei", "hms", "hw"][..],
                ),
            ],
            manifest,
        ),
        _ => true,
    }
}

fn provider_entry_enabled(
    note: &str,
    providers: &[(&[&str], &[&str], &[&str])],
    manifest: &serde_json::Value,
) -> bool {
    for (markers, module_keys, provider_keys) in providers {
        if android_entry_mentions_any(note, markers) {
            return manifest_has_enabled_provider(manifest, module_keys, provider_keys);
        }
    }
    true
}

fn android_entry_provider_note(entry: &str) -> String {
    let mut notes = Vec::new();
    let mut rest = entry;
    while let Some(start) = rest.find('(') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find(')') else {
            break;
        };
        notes.push(after_start[..end].trim());
        rest = &after_start[end + 1..];
    }
    if notes.is_empty() {
        entry.to_string()
    } else {
        notes.join(" ")
    }
}

fn android_entry_mentions_any(text: &str, markers: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    let normalized = normalize_config_key(text);
    markers.iter().any(|marker| {
        let marker_lower = marker.to_ascii_lowercase();
        let marker_normalized = normalize_config_key(marker);
        lower.contains(&marker_lower)
            || (!marker_normalized.is_empty() && normalized.contains(&marker_normalized))
    })
}

fn push_provider_enabled(manifest: &serde_json::Value, provider_keys: &[&str]) -> bool {
    manifest_has_enabled_provider(
        manifest,
        &["push", "unipush", "unipushV2", "uniPush"],
        provider_keys,
    )
}

fn apply_module_name_to_tree(tree: &mut ModuleConfigTree, name: &str) {
    match name {
        "Push" => {
            tree.push = Some(push_manifest_config());
        }
        "Geolocation" => {
            tree.geolocation = Some(location_manifest_config());
        }
        "Share" => {
            tree.share = Some(share_manifest_config());
        }
        "Login" | "OAuth" => {
            tree.login = Some(login_manifest_config());
        }
        "Payment" => {
            tree.payment = Some(payment_manifest_config());
        }
        "Map" | "Maps" => {
            tree.map = Some(map_manifest_config());
        }
        "Speech" => {
            tree.speech = Some(speech_manifest_config());
        }
        "Statistic" | "Statistics" => {
            tree.statistic = Some(statistic_manifest_config());
        }
        "FaceRecognition" | "FaceRecognitionVerify" | "FacialRecognitionVerify" => {
            tree.face_recognition = Some(face_recognition_manifest_config());
        }
        "UniAD" | "uni-ad" => {
            tree.uni_ad = Some(uni_ad_manifest_config());
        }
        "X5Webview" | "X5TBS" | "Android X5 Webview" => {
            tree.x5_tbs = Some(SimpleModuleConfig { enabled: true });
        }
        "LivePusher" => {
            tree.livepusher = Some(livepusher_manifest_config());
        }
        "UIWebview" | "UIWebView" => {
            tree.ui_webview = Some(SimpleModuleConfig { enabled: true });
        }
        _ => {}
    }
}

pub fn android_module_template_key(module_name: &str) -> Option<&'static str> {
    match module_name {
        "Push" | "push" => Some("push"),
        "Share" | "share" => Some("share"),
        "Geolocation" | "Location" | "geolocation" | "location" => Some("geolocation"),
        "Payment" | "payment" | "Pay" | "pay" => Some("payment"),
        "Login" | "OAuth" | "Oauth" | "oauth" | "login" => Some("login"),
        "Map" | "Maps" | "map" | "maps" => Some("map"),
        "Statistic" | "Statistics" | "statistic" | "statistics" => Some("statistic"),
        "Speech" | "speech" => Some("speech"),
        "FaceRecognition"
        | "FaceRecognitionVerify"
        | "FacialRecognitionVerify"
        | "facialRecognitionVerify" => Some("face_recognition"),
        "UniAD" | "uni-ad" | "uniAD" | "ad" | "Ad" => Some("uni_ad"),
        "X5Webview" | "X5TBS" | "Android X5 Webview" | "x5" | "x5_tbs" => Some("x5_tbs"),
        "LivePusher" | "livepusher" => Some("livepusher"),
        _ => None,
    }
}

fn module_applies_to_android(platforms: &[String]) -> bool {
    platforms.is_empty()
        || platforms.iter().any(|platform| {
            let platform = platform.to_ascii_lowercase();
            platform == "all" || platform == "android" || platform == "app"
        })
}

fn android_config_field_specs(template_key: &str) -> &'static [AndroidConfigFieldSpec] {
    match template_key {
        "push" => &[
            AndroidConfigFieldSpec {
                key: "GETUI_APPID",
                label: "uniPush AppID",
                required: true,
                secret: false,
                placeholder: "DCloud 开发者中心 uniPush AppID",
                aliases: &[
                    "GETUI_APPID",
                    "plus.unipush.appid",
                    "unipush_appid",
                    "appid",
                    "appId",
                ],
                path_hints: &["push", "unipush", "getui", "igt"],
            },
            AndroidConfigFieldSpec {
                key: "plus.unipush.appkey",
                label: "uniPush AppKey",
                required: true,
                secret: true,
                placeholder: "DCloud 开发者中心 uniPush AppKey",
                aliases: &["plus.unipush.appkey", "unipush_appkey", "appkey", "appKey"],
                path_hints: &["push", "unipush", "getui", "igt"],
            },
            AndroidConfigFieldSpec {
                key: "plus.unipush.appsecret",
                label: "uniPush AppSecret",
                required: true,
                secret: true,
                placeholder: "DCloud 开发者中心 uniPush AppSecret",
                aliases: &[
                    "plus.unipush.appsecret",
                    "unipush_appsecret",
                    "appsecret",
                    "appSecret",
                ],
                path_hints: &["push", "unipush", "getui", "igt"],
            },
            AndroidConfigFieldSpec {
                key: "XIAOMI_APP_ID",
                label: "小米推送 AppID",
                required: false,
                secret: false,
                placeholder: "启用小米通道时填写",
                aliases: &["XIAOMI_APP_ID", "xiaomi_app_id", "appid", "appId"],
                path_hints: &["xiaomi", "mi"],
            },
            AndroidConfigFieldSpec {
                key: "XIAOMI_APP_KEY",
                label: "小米推送 AppKey",
                required: false,
                secret: true,
                placeholder: "启用小米通道时填写",
                aliases: &["XIAOMI_APP_KEY", "xiaomi_app_key", "appkey", "appKey"],
                path_hints: &["xiaomi", "mi"],
            },
            AndroidConfigFieldSpec {
                key: "MEIZU_APP_ID",
                label: "魅族推送 AppID",
                required: false,
                secret: false,
                placeholder: "启用魅族通道时填写",
                aliases: &["MEIZU_APP_ID", "meizu_app_id", "appid", "appId"],
                path_hints: &["meizu", "mz"],
            },
            AndroidConfigFieldSpec {
                key: "MEIZU_APP_KEY",
                label: "魅族推送 AppKey",
                required: false,
                secret: true,
                placeholder: "启用魅族通道时填写",
                aliases: &["MEIZU_APP_KEY", "meizu_app_key", "appkey", "appKey"],
                path_hints: &["meizu", "mz"],
            },
            AndroidConfigFieldSpec {
                key: "HUAWEI_APP_ID",
                label: "华为推送 AppID",
                required: false,
                secret: false,
                placeholder: "启用华为通道时填写",
                aliases: &["HUAWEI_APP_ID", "huawei_app_id", "appid", "appId"],
                path_hints: &["huawei", "hms", "hw"],
            },
            AndroidConfigFieldSpec {
                key: "OPPO_APP_KEY",
                label: "OPPO 推送 AppKey",
                required: false,
                secret: true,
                placeholder: "启用 OPPO 通道时填写",
                aliases: &["OPPO_APP_KEY", "oppo_app_key", "appkey", "appKey"],
                path_hints: &["oppo"],
            },
            AndroidConfigFieldSpec {
                key: "OPPO_APP_SECRET",
                label: "OPPO 推送 AppSecret",
                required: false,
                secret: true,
                placeholder: "启用 OPPO 通道时填写",
                aliases: &[
                    "OPPO_APP_SECRET",
                    "oppo_app_secret",
                    "appsecret",
                    "appSecret",
                ],
                path_hints: &["oppo"],
            },
            AndroidConfigFieldSpec {
                key: "VIVO_APP_ID",
                label: "vivo 推送 AppID",
                required: false,
                secret: false,
                placeholder: "启用 vivo 通道时填写",
                aliases: &["VIVO_APP_ID", "vivo_app_id", "appid", "appId"],
                path_hints: &["vivo"],
            },
            AndroidConfigFieldSpec {
                key: "VIVO_APP_KEY",
                label: "vivo 推送 AppKey",
                required: false,
                secret: true,
                placeholder: "启用 vivo 通道时填写",
                aliases: &["VIVO_APP_KEY", "vivo_app_key", "appkey", "appKey"],
                path_hints: &["vivo"],
            },
            AndroidConfigFieldSpec {
                key: "HONOR_APP_ID",
                label: "荣耀推送 AppID",
                required: false,
                secret: false,
                placeholder: "启用荣耀通道时填写",
                aliases: &["HONOR_APP_ID", "honor_app_id", "appid", "appId"],
                path_hints: &["honor", "rongyao"],
            },
        ],
        "geolocation" => &[
            AndroidConfigFieldSpec {
                key: "BAIDU_MAP_AK",
                label: "百度定位 AK",
                required: false,
                secret: true,
                placeholder: "使用百度定位时填写",
                aliases: &[
                    "BAIDU_MAP_AK",
                    "BAIDU_AK",
                    "baidu_ak",
                    "appkey_android",
                    "appKey",
                    "key",
                ],
                path_hints: &["geolocation", "location", "baidu", "sdkconfigs", "maps"],
            },
            AndroidConfigFieldSpec {
                key: "AMAP_KEY",
                label: "高德定位 AppKey",
                required: false,
                secret: true,
                placeholder: "使用高德定位时填写",
                aliases: &["AMAP_KEY", "amap_key", "appkey_android", "appKey", "key"],
                path_hints: &["geolocation", "location", "amap", "sdkconfigs", "maps"],
            },
            AndroidConfigFieldSpec {
                key: "TENCENT_MAP_KEY",
                label: "腾讯定位 AppKey",
                required: false,
                secret: true,
                placeholder: "使用腾讯定位时填写",
                aliases: &[
                    "TENCENT_MAP_KEY",
                    "TencentMapSDK",
                    "tencent_map_key",
                    "apikey_android",
                    "apikey_ios",
                    "appKey",
                    "key",
                ],
                path_hints: &["geolocation", "location", "tencent", "qqmap", "maps"],
            },
        ],
        "share" => &[
            AndroidConfigFieldSpec {
                key: "WX_APPID",
                label: "微信分享 AppID",
                required: true,
                secret: false,
                placeholder: "微信开放平台 AppID",
                aliases: &["WX_APPID", "weixin_appid", "wx_appid", "appid", "appId"],
                path_hints: &["share", "weixin", "weixinweb", "wechat", "wx"],
            },
            AndroidConfigFieldSpec {
                key: "WX_SECRET",
                label: "微信分享 Secret",
                required: true,
                secret: true,
                placeholder: "微信开放平台 Secret",
                aliases: &[
                    "WX_SECRET",
                    "weixin_secret",
                    "wx_secret",
                    "appsecret",
                    "appSecret",
                    "secret",
                ],
                path_hints: &["share", "weixin", "weixinweb", "wechat", "wx"],
            },
            AndroidConfigFieldSpec {
                key: "QQ_APPID",
                label: "QQ 分享 AppID",
                required: true,
                secret: false,
                placeholder: "QQ 互联 AppID",
                aliases: &["QQ_APPID", "qq_appid", "appid", "appId"],
                path_hints: &["share", "qq"],
            },
            AndroidConfigFieldSpec {
                key: "SINA_APPKEY",
                label: "新浪微博 AppKey",
                required: false,
                secret: true,
                placeholder: "启用微博分享时填写",
                aliases: &["SINA_APPKEY", "sina_appkey", "appkey", "appKey"],
                path_hints: &["share", "sina", "weibo"],
            },
            AndroidConfigFieldSpec {
                key: "SINA_SECRET",
                label: "新浪微博 Secret",
                required: false,
                secret: true,
                placeholder: "启用微博分享时填写",
                aliases: &["SINA_SECRET", "sina_secret", "secret"],
                path_hints: &["share", "sina", "weibo"],
            },
            AndroidConfigFieldSpec {
                key: "SINA_REDIRECT_URI",
                label: "新浪微博 Redirect URI",
                required: false,
                secret: false,
                placeholder: "启用微博分享时填写",
                aliases: &[
                    "SINA_REDIRECT_URI",
                    "sina_redirect_uri",
                    "redirectUri",
                    "redirect_uri",
                ],
                path_hints: &["share", "sina", "weibo"],
            },
        ],
        "login" => &[
            AndroidConfigFieldSpec {
                key: "WX_APPID",
                label: "微信登录 AppID",
                required: true,
                secret: false,
                placeholder: "微信开放平台 AppID",
                aliases: &["WX_APPID", "weixin_appid", "wx_appid", "appid", "appId"],
                path_hints: &["oauth", "login", "weixin", "wechat", "wx"],
            },
            AndroidConfigFieldSpec {
                key: "WX_SECRET",
                label: "微信登录 Secret",
                required: true,
                secret: true,
                placeholder: "微信开放平台 AppSecret",
                aliases: &[
                    "WX_SECRET",
                    "weixin_secret",
                    "wx_secret",
                    "appsecret",
                    "appSecret",
                    "secret",
                ],
                path_hints: &["oauth", "login", "weixin", "wechat", "wx"],
            },
            AndroidConfigFieldSpec {
                key: "QQ_APPID",
                label: "QQ 登录 AppID",
                required: true,
                secret: false,
                placeholder: "QQ 互联 AppID",
                aliases: &["QQ_APPID", "qq_appid", "appid", "appId"],
                path_hints: &["oauth", "login", "qq"],
            },
            AndroidConfigFieldSpec {
                key: "GY_APP_ID",
                label: "一键登录应用 ID",
                required: false,
                secret: false,
                placeholder: "启用一键登录时填写",
                aliases: &[
                    "GY_APP_ID",
                    "GETUI_APPID",
                    "univerify_appid",
                    "appid",
                    "appId",
                ],
                path_hints: &["oauth", "login", "univerify", "igetui", "getui"],
            },
            AndroidConfigFieldSpec {
                key: "SINA_APPKEY",
                label: "微博登录 AppKey",
                required: false,
                secret: true,
                placeholder: "启用微博登录时填写",
                aliases: &["SINA_APPKEY", "sina_appkey", "appkey", "appKey"],
                path_hints: &["oauth", "login", "sina", "weibo"],
            },
            AndroidConfigFieldSpec {
                key: "SINA_REDIRECT_URI",
                label: "微博登录 Redirect URI",
                required: false,
                secret: false,
                placeholder: "启用微博登录时填写",
                aliases: &["SINA_REDIRECT_URI", "redirect_uri", "redirectUri"],
                path_hints: &["oauth", "login", "sina", "weibo"],
            },
            AndroidConfigFieldSpec {
                key: "MIUI_APPID",
                label: "小米登录 AppID",
                required: false,
                secret: false,
                placeholder: "启用小米登录时填写",
                aliases: &["MIUI_APPID", "miui_appid", "appid", "appId"],
                path_hints: &["oauth", "login", "miui", "xiaomi"],
            },
            AndroidConfigFieldSpec {
                key: "MIUI_APPSECRET",
                label: "小米登录 AppSecret",
                required: false,
                secret: true,
                placeholder: "启用小米登录时填写",
                aliases: &["MIUI_APPSECRET", "miui_appsecret", "appsecret", "appSecret"],
                path_hints: &["oauth", "login", "miui", "xiaomi"],
            },
            AndroidConfigFieldSpec {
                key: "MIUI_REDIRECT_URI",
                label: "小米登录 RegURL",
                required: false,
                secret: false,
                placeholder: "启用小米登录时填写",
                aliases: &["MIUI_REDIRECT_URI", "redirect_uri", "redirectUri", "regUrl"],
                path_hints: &["oauth", "login", "miui", "xiaomi"],
            },
            AndroidConfigFieldSpec {
                key: "FACEBOOK_APP_ID",
                label: "Facebook App ID",
                required: false,
                secret: false,
                placeholder: "启用 Facebook 登录时填写",
                aliases: &["FACEBOOK_APP_ID", "facebook_app_id", "appId", "appid"],
                path_hints: &["oauth", "login", "facebook", "fb"],
            },
            AndroidConfigFieldSpec {
                key: "FACEBOOK_CLIENT_TOKEN",
                label: "Facebook Client Token",
                required: false,
                secret: true,
                placeholder: "启用 Facebook 登录时填写",
                aliases: &[
                    "FACEBOOK_CLIENT_TOKEN",
                    "facebook_client_token",
                    "clientToken",
                    "client_token",
                ],
                path_hints: &["oauth", "login", "facebook", "fb"],
            },
        ],
        "map" => &[
            AndroidConfigFieldSpec {
                key: "BAIDU_MAP_AK",
                label: "百度地图 AK",
                required: false,
                secret: true,
                placeholder: "使用百度地图时填写",
                aliases: &[
                    "BAIDU_MAP_AK",
                    "BAIDU_AK",
                    "baidu_map_ak",
                    "appkey_android",
                    "appKey",
                    "key",
                ],
                path_hints: &["maps", "map", "baidu", "sdkconfigs"],
            },
            AndroidConfigFieldSpec {
                key: "AMAP_KEY",
                label: "高德地图 AppKey",
                required: false,
                secret: true,
                placeholder: "使用高德地图时填写",
                aliases: &["AMAP_KEY", "amap_key", "appkey_android", "appKey", "key"],
                path_hints: &["maps", "map", "amap", "sdkconfigs"],
            },
            AndroidConfigFieldSpec {
                key: "GOOGLE_MAPS_API_KEY",
                label: "Google Maps API Key",
                required: false,
                secret: true,
                placeholder: "使用 Google 地图时填写",
                aliases: &[
                    "GOOGLE_MAPS_API_KEY",
                    "com.google.android.geo.API_KEY",
                    "google_maps_api_key",
                    "apiKey",
                    "key",
                ],
                path_hints: &["maps", "map", "google", "sdkconfigs"],
            },
            AndroidConfigFieldSpec {
                key: "TENCENT_MAP_KEY",
                label: "腾讯地图 Key",
                required: false,
                secret: true,
                placeholder: "使用腾讯地图时填写",
                aliases: &[
                    "TENCENT_MAP_KEY",
                    "TencentMapSDK",
                    "tencent_map_key",
                    "apikey_android",
                    "apikey_ios",
                    "appKey",
                    "key",
                ],
                path_hints: &["maps", "map", "tencent", "qqmap", "sdkconfigs"],
            },
        ],
        "payment" => &[
            AndroidConfigFieldSpec {
                key: "WX_APPID",
                label: "微信支付 AppID",
                required: true,
                secret: false,
                placeholder: "微信支付 AppID",
                aliases: &["WX_APPID", "weixin_appid", "wx_appid", "appid", "appId"],
                path_hints: &["payment", "pay", "weixin", "wechat", "wx"],
            },
            AndroidConfigFieldSpec {
                key: "PAYPAL_RETURN_SCHEME",
                label: "PayPal Return Scheme",
                required: false,
                secret: false,
                placeholder: "启用 PayPal 时填写",
                aliases: &[
                    "PAYPAL_RETURN_SCHEME",
                    "returnUrl",
                    "returnURL",
                    "returnURL_android",
                    "return_url_android",
                    "scheme",
                ],
                path_hints: &["payment", "pay", "paypal"],
            },
        ],
        "speech" => &[
            AndroidConfigFieldSpec {
                key: "BAIDU_SPEECH_APP_ID",
                label: "百度语音 AppID",
                required: false,
                secret: false,
                placeholder: "使用百度语音时填写",
                aliases: &[
                    "BAIDU_SPEECH_APP_ID",
                    "com.baidu.speech.APP_ID",
                    "appid",
                    "appId",
                ],
                path_hints: &["speech", "baidu"],
            },
            AndroidConfigFieldSpec {
                key: "BD_SPEECH_APIKEY",
                label: "百度语音 API Key",
                required: false,
                secret: true,
                placeholder: "使用百度语音时填写",
                aliases: &[
                    "BD_SPEECH_APIKEY",
                    "com.baidu.speech.API_KEY",
                    "apikey",
                    "apiKey",
                ],
                path_hints: &["speech", "baidu"],
            },
            AndroidConfigFieldSpec {
                key: "BD_SPEECH_SECRETKEY",
                label: "百度语音 Secret Key",
                required: false,
                secret: true,
                placeholder: "使用百度语音时填写",
                aliases: &[
                    "BD_SPEECH_SECRETKEY",
                    "com.baidu.speech.SECRET_KEY",
                    "secretkey",
                    "secretKey",
                ],
                path_hints: &["speech", "baidu"],
            },
            AndroidConfigFieldSpec {
                key: "IFLY_APPID",
                label: "讯飞语音 AppID",
                required: false,
                secret: false,
                placeholder: "使用讯飞语音时填写",
                aliases: &["IFLY_APPID", "IFLY_APPKEY", "appid", "appId"],
                path_hints: &["speech", "ifly", "xunfei"],
            },
        ],
        "statistic" => &[
            AndroidConfigFieldSpec {
                key: "UMENG_APPKEY",
                label: "友盟 AppKey",
                required: true,
                secret: true,
                placeholder: "友盟统计 AppKey",
                aliases: &["UMENG_APPKEY", "appkey_android", "appkey", "appKey"],
                path_hints: &["statistic", "statistics", "statics", "umeng"],
            },
            AndroidConfigFieldSpec {
                key: "UMENG_CHANNEL",
                label: "友盟渠道号",
                required: false,
                secret: false,
                placeholder: "渠道号，可选",
                aliases: &["UMENG_CHANNEL", "channelid_android", "channel", "channelId"],
                path_hints: &["statistic", "statistics", "statics", "umeng"],
            },
        ],
        "face_recognition" => &[
            AndroidConfigFieldSpec {
                key: "DCLOUD_LICENSE",
                label: "DCloud 实人认证 License",
                required: false,
                secret: true,
                placeholder: "DCloud 实人认证 License",
                aliases: &["DCLOUD_LICENSE", "dcloud_license", "license"],
                path_hints: &["facial", "face", "realname"],
            },
            AndroidConfigFieldSpec {
                key: "BDFACE_APIKEY",
                label: "百度人脸 API Key",
                required: false,
                secret: true,
                placeholder: "使用百度人脸时填写",
                aliases: &["BDFACE_APIKEY", "bd_api_key", "apiKey", "apikey"],
                path_hints: &["facial", "face", "baidu"],
            },
            AndroidConfigFieldSpec {
                key: "BDFACE_SECRETKEY",
                label: "百度人脸 Secret Key",
                required: false,
                secret: true,
                placeholder: "使用百度人脸时填写",
                aliases: &[
                    "BDFACE_SECRETKEY",
                    "bd_secret_key",
                    "secretKey",
                    "secretkey",
                ],
                path_hints: &["facial", "face", "baidu"],
            },
            AndroidConfigFieldSpec {
                key: "ALIFACE_ACCESSKEY_ID",
                label: "阿里实人认证 AccessKey ID",
                required: false,
                secret: true,
                placeholder: "使用阿里实人认证时填写",
                aliases: &["ALIFACE_ACCESSKEY_ID", "ali_access_key_id", "accessKeyId"],
                path_hints: &["facial", "face", "aliyun", "ali"],
            },
            AndroidConfigFieldSpec {
                key: "ALIFACE_ACCESSKEY_SECRET",
                label: "阿里实人认证 AccessKey Secret",
                required: false,
                secret: true,
                placeholder: "使用阿里实人认证时填写",
                aliases: &[
                    "ALIFACE_ACCESSKEY_SECRET",
                    "ali_access_key_secret",
                    "accessKeySecret",
                ],
                path_hints: &["facial", "face", "aliyun", "ali"],
            },
        ],
        "uni_ad" => &[
            AndroidConfigFieldSpec {
                key: "DCLOUD_STREAMAPP_CHANNEL",
                label: "uni-AD 渠道配置",
                required: true,
                secret: false,
                placeholder: "包名|appid|广告标识|渠道",
                aliases: &["DCLOUD_STREAMAPP_CHANNEL", "streamapp_channel", "channel"],
                path_hints: &["ad", "uniad", "uni-ad"],
            },
            AndroidConfigFieldSpec {
                key: "DCLOUD_AD_SPLASH",
                label: "是否开启开屏广告",
                required: false,
                secret: false,
                placeholder: "true / false",
                aliases: &["DCLOUD_AD_SPLASH", "ad_splash", "splash"],
                path_hints: &["ad", "uniad", "uni-ad"],
            },
        ],
        "livepusher" => &[
            AndroidConfigFieldSpec {
                key: "LIVEPUSH_LICENSE_URL",
                label: "LivePusher License URL",
                required: false,
                secret: false,
                placeholder: "直播推流 License URL",
                aliases: &[
                    "LIVEPUSH_LICENSE_URL",
                    "TXLIVE_LICENSE_URL",
                    "license_url",
                    "licenseUrl",
                ],
                path_hints: &["livepusher", "live", "push"],
            },
            AndroidConfigFieldSpec {
                key: "LIVEPUSH_LICENSE_KEY",
                label: "LivePusher License Key",
                required: false,
                secret: true,
                placeholder: "直播推流 License Key",
                aliases: &[
                    "LIVEPUSH_LICENSE_KEY",
                    "TXLIVE_LICENSE_KEY",
                    "license_key",
                    "licenseKey",
                ],
                path_hints: &["livepusher", "live", "push"],
            },
        ],
        _ => &[],
    }
}

fn manifest_info_to_value(
    manifest_info: &crate::commands::resource::UniappManifestInfo,
) -> Option<serde_json::Value> {
    serde_json::to_value(manifest_info).ok()
}

fn find_manifest_config_value(
    manifest: &serde_json::Value,
    spec: &AndroidConfigFieldSpec,
) -> Option<String> {
    let mut candidates = Vec::new();
    collect_manifest_config_candidates(manifest, spec, &mut Vec::new(), &mut candidates);
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.into_iter().map(|(_, value)| value).next()
}

fn android_field_required_for_manifest(
    template_key: &str,
    spec: &AndroidConfigFieldSpec,
    manifest: Option<&serde_json::Value>,
) -> bool {
    if !spec.required {
        return android_optional_field_required_for_manifest(template_key, spec, manifest);
    }
    let Some(manifest) = manifest else {
        return spec.required;
    };

    match template_key {
        "share" => match spec.key {
            "WX_APPID" | "WX_SECRET" => manifest_has_enabled_provider(
                manifest,
                &["share", "shares"],
                &["weixin", "wechat", "wx"],
            ),
            "QQ_APPID" => manifest_has_enabled_provider(manifest, &["share", "shares"], &["qq"]),
            _ => spec.required,
        },
        "login" => match spec.key {
            "WX_APPID" | "WX_SECRET" => manifest_has_enabled_provider(
                manifest,
                &["oauth", "login", "oauths"],
                &["weixin", "wechat", "wx"],
            ),
            "QQ_APPID" => {
                manifest_has_enabled_provider(manifest, &["oauth", "login", "oauths"], &["qq"])
            }
            _ => spec.required,
        },
        "payment" => match spec.key {
            "WX_APPID" => manifest_has_enabled_provider(
                manifest,
                &["payment", "pay", "payments"],
                &["weixin", "wechat", "wx"],
            ),
            _ => spec.required,
        },
        "statistic" => match spec.key {
            "UMENG_APPKEY" => manifest_has_enabled_provider(
                manifest,
                &["statistic", "statistics", "statics"],
                &["umeng"],
            ),
            _ => spec.required,
        },
        "uni_ad" => match spec.key {
            "DCLOUD_STREAMAPP_CHANNEL" => manifest_has_any_enabled_module(
                manifest,
                &["ad", "ads", "uni-ad", "uniAD", "uniad"],
            ),
            _ => spec.required,
        },
        _ => spec.required,
    }
}

fn android_optional_field_required_for_manifest(
    template_key: &str,
    spec: &AndroidConfigFieldSpec,
    manifest: Option<&serde_json::Value>,
) -> bool {
    let Some(manifest) = manifest else {
        return false;
    };

    match template_key {
        "push" => {
            matches!(
                spec.key,
                "XIAOMI_APP_ID"
                    | "XIAOMI_APP_KEY"
                    | "MEIZU_APP_ID"
                    | "MEIZU_APP_KEY"
                    | "HUAWEI_APP_ID"
                    | "OPPO_APP_KEY"
                    | "OPPO_APP_SECRET"
                    | "VIVO_APP_ID"
                    | "VIVO_APP_KEY"
                    | "HONOR_APP_ID"
            ) && android_field_visible_for_manifest(template_key, spec, Some(manifest))
        }
        _ => false,
    }
}

fn manifest_has_enabled_provider(
    manifest: &serde_json::Value,
    module_keys: &[&str],
    provider_keys: &[&str],
) -> bool {
    let mut found = false;
    visit_manifest_objects(manifest, &mut |map| {
        for module_key in module_keys {
            if let Some(value) = get_object_value_normalized(map, module_key) {
                if provider_keys
                    .iter()
                    .any(|provider| manifest_provider_enabled(value, provider))
                {
                    found = true;
                }
            }
        }
    });
    found
}

fn manifest_has_any_enabled_module(manifest: &serde_json::Value, module_keys: &[&str]) -> bool {
    let mut found = false;
    visit_manifest_objects(manifest, &mut |map| {
        for module_key in module_keys {
            if let Some(value) = get_object_value_normalized(map, module_key) {
                if config_value_enabled(value) {
                    found = true;
                }
            }
        }
    });
    found
}

fn visit_manifest_objects<F>(value: &serde_json::Value, visitor: &mut F)
where
    F: FnMut(&serde_json::Map<String, serde_json::Value>),
{
    match value {
        serde_json::Value::Object(map) => {
            visitor(map);
            for item in map.values() {
                visit_manifest_objects(item, visitor);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                visit_manifest_objects(item, visitor);
            }
        }
        _ => {}
    }
}

fn manifest_provider_enabled(value: &serde_json::Value, provider_key: &str) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    map.iter().any(|(key, item)| {
        (normalize_config_key(key) == normalize_config_key(provider_key)
            && config_value_enabled(item))
            || manifest_provider_enabled(item, provider_key)
    })
}

fn get_object_value_normalized<'a>(
    map: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let normalized_key = normalize_config_key(key);
    map.iter()
        .find(|(candidate, _)| normalize_config_key(candidate) == normalized_key)
        .map(|(_, value)| value)
}

fn config_value_enabled(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(flag) => *flag,
        serde_json::Value::Null => false,
        serde_json::Value::Object(map) => {
            let enabled = map
                .get("enabled")
                .or_else(|| map.get("enable"))
                .or_else(|| map.get("open"))
                .and_then(|value| value.as_bool())
                .unwrap_or(true);
            enabled && config_value_applies_to_android(map)
        }
        _ => true,
    }
}

fn config_value_applies_to_android(map: &serde_json::Map<String, serde_json::Value>) -> bool {
    let Some(platforms) = map.get("__platform__") else {
        return true;
    };
    match platforms {
        serde_json::Value::Array(items) => items.iter().any(|item| {
            item.as_str()
                .map(|platform| {
                    let platform = platform.to_ascii_lowercase();
                    platform == "android" || platform == "app" || platform == "all"
                })
                .unwrap_or(false)
        }),
        serde_json::Value::String(platform) => {
            let platform = platform.to_ascii_lowercase();
            platform == "android" || platform == "app" || platform == "all"
        }
        _ => true,
    }
}

fn collect_manifest_config_candidates(
    value: &serde_json::Value,
    spec: &AndroidConfigFieldSpec,
    path: &mut Vec<String>,
    candidates: &mut Vec<(u32, String)>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                path.push(key.clone());
                collect_manifest_config_candidates(item, spec, path, candidates);
                path.pop();
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_manifest_config_candidates(item, spec, path, candidates);
            }
        }
        serde_json::Value::String(text) => {
            push_manifest_config_candidate(text.trim(), spec, path, candidates);
        }
        serde_json::Value::Number(number) => {
            push_manifest_config_candidate(&number.to_string(), spec, path, candidates);
        }
        serde_json::Value::Bool(flag) => {
            push_manifest_config_candidate(
                if *flag { "true" } else { "false" },
                spec,
                path,
                candidates,
            );
        }
        _ => {}
    }
}

fn push_manifest_config_candidate(
    raw_value: &str,
    spec: &AndroidConfigFieldSpec,
    path: &[String],
    candidates: &mut Vec<(u32, String)>,
) {
    if raw_value.is_empty() {
        return;
    }
    let Some(last_key) = path.last() else {
        return;
    };
    let normalized_key = normalize_config_key(last_key);
    let Some(alias) = spec
        .aliases
        .iter()
        .find(|alias| normalize_config_key(alias) == normalized_key)
    else {
        return;
    };

    let exact_spec_key = normalized_key == normalize_config_key(spec.key);
    let generic_alias = is_generic_manifest_alias(alias);
    let hint_matches = count_path_hint_matches(path, spec.path_hints);
    let required_hint_matches = if spec.path_hints.len() <= 1 {
        spec.path_hints.len()
    } else {
        2
    };

    if generic_alias && hint_matches < required_hint_matches {
        return;
    }

    let alias_score = if exact_spec_key {
        100
    } else if generic_alias {
        20
    } else {
        60
    };
    let score = alias_score + (hint_matches as u32 * 10);
    candidates.push((score, raw_value.to_string()));
}

fn is_generic_manifest_alias(alias: &str) -> bool {
    matches!(
        normalize_config_key(alias).as_str(),
        "appid"
            | "appkey"
            | "appsecret"
            | "secret"
            | "key"
            | "apikey"
            | "secretkey"
            | "appkeyandroid"
            | "channel"
            | "channelidandroid"
            | "redirecturi"
            | "redirecturl"
            | "license"
            | "scheme"
    )
}

fn count_path_hint_matches(path: &[String], hints: &[&str]) -> usize {
    let normalized_path = path
        .iter()
        .map(|part| normalize_config_key(part))
        .collect::<Vec<_>>();
    hints
        .iter()
        .filter(|hint| {
            let hint = normalize_config_key(hint);
            normalized_path
                .iter()
                .any(|part| part.contains(&hint) || hint.contains(part))
        })
        .count()
}

fn normalize_config_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn push_manifest_config() -> PushModuleConfig {
    PushModuleConfig {
        enabled: true,
        unipush_appid: Some(String::new()),
        unipush_appkey: Some(String::new()),
        unipush_appsecret: Some(String::new()),
        vendors: Vec::new(),
    }
}

fn location_manifest_config() -> LocationConfig {
    LocationConfig {
        enabled: true,
        engine: "system".to_string(),
        baidu_ak: None,
        amap_key: None,
    }
}

fn share_manifest_config() -> ShareModuleConfig {
    ShareModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        qq: Some(HashMap::new()),
        sina: None,
    }
}

fn login_manifest_config() -> LoginModuleConfig {
    LoginModuleConfig {
        enabled: true,
        providers: vec![
            LoginProvider {
                name: "weixin".to_string(),
                enabled: true,
                config: HashMap::new(),
            },
            LoginProvider {
                name: "qq".to_string(),
                enabled: true,
                config: HashMap::new(),
            },
        ],
    }
}

fn payment_manifest_config() -> PaymentModuleConfig {
    PaymentModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        alipay: Some(HashMap::new()),
    }
}

fn map_manifest_config() -> MapModuleConfig {
    MapModuleConfig {
        enabled: true,
        engine: "amap".into(),
        amap_key: None,
        tencent_map_key: None,
        baidu_map_ak: None,
        google_maps_api_key: None,
    }
}

fn speech_manifest_config() -> SpeechModuleConfig {
    SpeechModuleConfig {
        enabled: true,
        engine: "system".into(),
        xfyun: None,
        baidu: None,
        aliyun: None,
    }
}

fn statistic_manifest_config() -> StatisticModuleConfig {
    StatisticModuleConfig {
        enabled: true,
        provider: "umeng".into(),
        umeng: None,
        mta: None,
        baidu: None,
    }
}

fn face_recognition_manifest_config() -> FaceRecognitionModuleConfig {
    FaceRecognitionModuleConfig {
        enabled: true,
        provider: "dcloud".into(),
        dcloud: None,
        baidu: None,
        aliyun: None,
    }
}

fn uni_ad_manifest_config() -> UniAdModuleConfig {
    UniAdModuleConfig {
        enabled: true,
        csj: Some(HashMap::new()),
        gdt: Some(HashMap::new()),
        gromore: None,
        admob: None,
    }
}

fn livepusher_manifest_config() -> LivePusherModuleConfig {
    LivePusherModuleConfig {
        enabled: true,
        license_url: None,
        license_key: None,
    }
}

fn merge_properties_to_tree(tree: &mut ModuleConfigTree, _xml_content: &str) -> Result<(), String> {
    if let Some(ref mut push) = tree.push {
        push.unipush_appid = Some(String::new());
        push.unipush_appkey = Some(String::new());
        push.unipush_appsecret = Some(String::new());
    }
    Ok(())
}

#[tauri::command]
pub async fn save_module_config(
    project_path: String,
    config: ModuleConfigTree,
) -> Result<(), String> {
    let project_dir = PathBuf::from(&project_path);

    let data_dir = project_dir.join("assets").join("data");
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create assets/data directory: {}", e))?;

    let props_path = data_dir.join("dcloud_properties.xml");
    generate_dcloud_properties(&props_path, &config)?;

    if let Some(ref push) = config.push {
        save_push_vendor_config(&project_dir, push)?;
    }

    let project_id = extract_project_id(&project_dir)?;
    let config_path = get_module_config_path(&project_id);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let json_content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize module config: {}", e))?;
    fs::write(&config_path, json_content)
        .map_err(|e| format!("Failed to write module config: {}", e))?;

    Ok(())
}

fn extract_project_id(project_dir: &Path) -> Result<String, String> {
    let manifest_path = project_dir.join("manifest.json");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Cannot read manifest.json: {}", e))?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Cannot parse manifest.json: {}", e))?;
    json.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No 'id' field in manifest.json".to_string())
}

pub fn generate_dcloud_properties(path: &Path, config: &ModuleConfigTree) -> Result<(), String> {
    let mut features = Vec::new();
    let mut services = Vec::new();

    if let Some(ref push) = config.push {
        if push.enabled {
            let mut feature =
                "    <feature name=\"Push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\">\n"
                    .to_string();
            if push.unipush_appid.is_some() || push.unipush_appkey.is_some() {
                feature.push_str(
                    "      <module name=\"unipush\" value=\"io.dcloud.feature.unipush.GTPushService\"/>\n",
                );
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
            services.push(
                "    <service name=\"push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\"/>\n"
                    .to_string(),
            );
        }
    }

    if let Some(ref share) = config.share {
        if share.enabled {
            features.push(
                "    <feature name=\"Share\" value=\"io.dcloud.feature.share.ShareFeatureImpl\"/>\n"
                    .to_string(),
            );
        }
    }

    if let Some(ref map) = config.map {
        if map.enabled {
            let mut feature = "    <feature name=\"Maps\">".to_string();
            match map.engine.as_str() {
                "amap" => feature.push_str("<module name=\"Amap\"/></feature>\n"),
                "tencent" => feature.push_str("<module name=\"TencentMap\"/></feature>\n"),
                _ => feature.push_str("</feature>\n"),
            }
            features.push(feature);
        }
    }

    if let Some(ref login) = config.login {
        if login.enabled {
            let mut feature =
                "    <feature name=\"Login\" value=\"io.dcloud.feature.login.LoginFeatureImpl\">\n"
                    .to_string();
            for provider in &login.providers {
                if provider.enabled {
                    match provider.name.as_str() {
                        "weixin" => feature.push_str("      <module name=\"WeixinLogin\"/>\n"),
                        "qq" => feature.push_str("      <module name=\"QQLogin\"/>\n"),
                        "apple" => feature.push_str("      <module name=\"AppleLogin\"/>\n"),
                        "univerify" => feature.push_str("      <module name=\"Univerify\"/>\n"),
                        _ => {}
                    }
                }
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
        }
    }

    if let Some(ref payment) = config.payment {
        if payment.enabled {
            let mut feature = "    <feature name=\"Payment\">\n".to_string();
            if payment.weixin.is_some() {
                feature.push_str("      <module name=\"WeixinPay\"/>\n");
            }
            if payment.alipay.is_some() {
                feature.push_str("      <module name=\"Alipay\"/>\n");
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
        }
    }

    if let Some(ref speech) = config.speech {
        if speech.enabled {
            let engine_module = match speech.engine.as_str() {
                "xunfei" => "Xfyun",
                "baidu" => "Baidu",
                "ali" => "Ali",
                _ => "System",
            };
            features.push(format!(
                "    <feature name=\"Speech\"><module name=\"{}\"/></feature>\n",
                engine_module
            ));
        }
    }

    if let Some(ref stat) = config.statistic {
        if stat.enabled {
            let provider_module = match stat.provider.as_str() {
                "umeng" => "Umeng",
                "mta" => "MTA",
                "baidu" => "Baidu",
                _ => "DCloud",
            };
            features.push(format!(
                "    <feature name=\"Statistic\"><module name=\"{}\"/></feature>\n",
                provider_module
            ));
        }
    }

    if let Some(ref fr) = config.face_recognition {
        if fr.enabled {
            let provider_module = match fr.provider.as_str() {
                "dcloud" => "DCloud",
                "baidu" => "Baidu",
                "aliyun" => "Aliyun",
                _ => "DCloud",
            };
            features.push(format!(
                "    <feature name=\"FaceRecognition\"><module name=\"{}\"/></feature>\n",
                provider_module
            ));
        }
    }

    if let Some(ref ad) = config.uni_ad {
        if ad.enabled {
            let mut feature = "    <feature name=\"UniAD\">\n".to_string();
            if ad.csj.is_some() {
                feature.push_str("      <module name=\"CSJ\"/>\n");
            }
            if ad.gdt.is_some() {
                feature.push_str("      <module name=\"GDT\"/>\n");
            }
            if ad.gromore.is_some() {
                feature.push_str("      <module name=\"Gromore\"/>\n");
            }
            if ad.admob.is_some() {
                feature.push_str("      <module name=\"AdMob\"/>\n");
            }
            feature.push_str("    </feature>\n");
            features.push(feature);
        }
    }

    if let Some(ref x5) = config.x5_tbs {
        if x5.enabled {
            features.push("    <feature name=\"X5Webview\" value=\"io.dcloud.feature.X5Webview.X5WebViewService\"/>\n".to_string());
        }
    }

    if let Some(ref lp) = config.livepusher {
        if lp.enabled {
            features.push("    <feature name=\"LivePusher\"/>\n".to_string());
        }
    }

    let mut xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<dcloud-properties>
"#
    .to_string();
    if !features.is_empty() {
        xml.push_str("  <features>\n");
        for feature in features {
            xml.push_str(&feature);
        }
        xml.push_str("  </features>\n");
    }
    if !services.is_empty() {
        xml.push_str("  <services>\n");
        for service in services {
            xml.push_str(&service);
        }
        xml.push_str("  </services>\n");
    }

    xml.push_str("</dcloud-properties>\n");

    fs::write(path, xml).map_err(|e| format!("Failed to write dcloud_properties.xml: {}", e))
}

fn save_push_vendor_config(_project_dir: &Path, _push: &PushModuleConfig) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn get_module_template(module_name: String) -> Result<ModuleTemplate, String> {
    get_module_template_sync(&module_name)
}

pub fn get_module_template_sync(module_name: &str) -> Result<ModuleTemplate, String> {
    match module_name {
        "push" => Ok(get_push_template()),
        "share" => Ok(get_share_template()),
        "geolocation" => Ok(get_geolocation_template()),
        "payment" => Ok(get_payment_template()),
        "login" => Ok(get_login_template()),
        "map" => Ok(get_map_template()),
        "statistic" => Ok(get_statistic_template()),
        "speech" => Ok(get_speech_template()),
        "face_recognition" => Ok(get_face_recognition_template()),
        "uni_ad" => Ok(get_uniad_template()),
        "x5_tbs" => Ok(get_x5_template()),
        "livepusher" => Ok(get_livepusher_template()),
        _ => Err(format!("Unknown module: {}", module_name)),
    }
}

fn get_push_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Push".to_string(),
        description: "uniPush 推送模块（支持6厂商通道）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "aps-release.aar".to_string(),
                "aps-unipush-release.aar".to_string(),
            ],
            gradle_dependencies: vec![
                "com.getui:gtsdk:3.3.7.0".to_string(),
                "com.getui:gtc-dcloud:3.2.16.7".to_string(),
                "com.getui.opt:xmp:3.3.1 (小米)".to_string(),
                "com.getui.opt:mzp:3.2.3 (魅族)".to_string(),
                "com.getui.opt:hwp:3.1.1 (华为)".to_string(),
                "com.huawei.hms:push:6.11.0.300 (华为)".to_string(),
                "com.assist-v3:oppo:3.3.0 (OPPO)".to_string(),
                "com.google.code.gson:gson:2.6.2 (OPPO)".to_string(),
                "commons-codec:commons-codec:1.6 (OPPO)".to_string(),
                "androidx.annotation:annotation:1.1.0 (OPPO)".to_string(),
                "com.assist-v3:vivo:3.1.1 (vivo)".to_string(),
                "com.getui.opt:honor:3.6.0 (荣耀)".to_string(),
                "com.hihonor.mcs:push:7.0.61.303 (荣耀)".to_string(),
            ],
            manifest_placeholders: vec![
                "XIAOMI_APP_ID / XIAOMI_APP_KEY".to_string(),
                "MEIZU_APP_ID / MEIZU_APP_KEY".to_string(),
                "HUAWEI_APP_ID".to_string(),
                "OPPO_APP_KEY / OPPO_APP_SECRET".to_string(),
                "VIVO_APP_ID / VIVO_APP_KEY".to_string(),
                "HONOR_APP_ID".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([
                    ("android:name".to_string(), "MIPUSH_APPID".to_string()),
                    ("android:value".to_string(), "${XIAOMI_APP_ID}".to_string()),
                ]),
                HashMap::from([
                    ("android:name".to_string(), "MIPUSH_APPKEY".to_string()),
                    ("android:value".to_string(), "${XIAOMI_APP_KEY}".to_string()),
                ]),
                HashMap::from([
                    ("android:name".to_string(), "MEIZUPUSH_APPID".to_string()),
                    ("android:value".to_string(), "${MEIZU_APP_ID}".to_string()),
                ]),
                HashMap::from([
                    ("android:name".to_string(), "MEIZUPUSH_APPKEY".to_string()),
                    ("android:value".to_string(), "${MEIZU_APP_KEY}".to_string()),
                ]),
            ],
            activities: vec![
                "com.tencent.tauth.AuthActivity (QQ)".to_string(),
                "cn.sharesdk.wechat.friends.WXFriendActivity (需要分享时)".to_string(),
            ],
            properties_xml: "<feature name=\"Push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\"><module name=\"unipush\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "UserNotifications.framework".to_string(),
                "Security.framework".to_string(),
                "CoreTelephony.framework".to_string(),
                "SystemConfiguration.framework".to_string(),
            ],
            required_libraries: vec![
                "libc++.tbd".to_string(),
                "libsqlite3.tbd".to_string(),
                "libz.tbd".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("getui".to_string(), "{appid, appkey, appsecret} (个推/uniPush)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "<key>getui</key><dict><key>appid</key><string></string></dict>".to_string(),
        },
    }
}

fn get_share_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Share".to_string(),
        description: "社交分享模块（微信/QQ/新浪微博）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "share-weixin-release.aar (微信)".to_string(),
                "share-qq-release.aar (QQ)".to_string(),
                "share-sina-release.aar (微博)".to_string(),
                "open_sdk_XXX_lite.jar (QQ SDK)".to_string(),
                "openDefault-XXX.aar (微博 SDK)".to_string(),
            ],
            gradle_dependencies: vec![
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信 HX>=3.7.6)".to_string(),
            ],
            manifest_placeholders: vec![
                "WX_APPID / WX_SECRET (微信)".to_string(),
                "QQ_APPID (QQ)".to_string(),
                "SINA_APPKEY / SINA_SECRET / SINA_REDIRECT_URI (微博)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "WX_APPID".to_string()), ("android:value".to_string(), "${WX_APPID}".to_string())]),
                HashMap::from([("android:name".to_string(), "WX_SECRET".to_string()), ("android:value".to_string(), "${WX_SECRET}".to_string())]),
                HashMap::from([("android:name".to_string(), "QQ_APPID".to_string()), ("android:value".to_string(), "${QQ_APPID}".to_string())]),
                HashMap::from([("android:name".to_string(), "SINA_APPKEY".to_string()), ("android:value".to_string(), "${SINA_APPKEY}".to_string())]),
                HashMap::from([("android:name".to_string(), "SINA_SECRET".to_string()), ("android:value".to_string(), "${SINA_SECRET}".to_string())]),
                HashMap::from([("android:name".to_string(), "SINA_REDIRECT_URI".to_string()), ("android:value".to_string(), "${SINA_REDIRECT_URI}".to_string())]),
            ],
            activities: vec![
                ".wxapi.WXEntryActivity (微信回调)".to_string(),
                ".wxapi.WXPayActivity (微信支付)".to_string(),
                "com.tencent.tauth.AuthActivity (QQ授权)".to_string(),
                "com.tencent.connect.common.AssistActivity (QQ辅助)".to_string(),
                "cn.sharesdk.wechat.friends.WXFriendActivity (微博分享页)".to_string(),
            ],
            properties_xml: "<feature name=\"Share\" value=\"io.dcloud.share.ShareFeatureImpl\"><module name=\"Weixin\"/><module name=\"QQ\"/><module name=\"SinaWeibo\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "ImageIO.framework (微博)".to_string(),
                "CoreTelephony.framework (微信)".to_string(),
                "SystemConfiguration.framework (微信)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.0.tbd (微博/QQ)".to_string(),
                "libz.tbd (微信/QQ)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("weixin".to_string(), "{appid, UniversalLinks} (微信)".to_string()),
                ("qq".to_string(), "{appid, Associated Domains} (QQ)".to_string()),
                ("sinaweibo".to_string(), "{appkey, redirectURI, Associated Domains} (微博)".to_string()),
            ]),
            url_schemes: vec![
                UrlSchemeConfig { scheme: "wx{appid}".to_string(), identifier: "weixin".to_string() },
                UrlSchemeConfig { scheme: "tencent{appid}".to_string(), identifier: "tencentopenapi".to_string() },
                UrlSchemeConfig { scheme: "wb{appkey}".to_string(), identifier: "com.weibo".to_string() },
            ],
            plist_entry: "⚠️ iOS 微信分享需在 AppDelegate.m 中添加 handleOpenURL 回调；注意 libWeChatSDK_pay.a 仅用于分享+支付+登录，不用支付功能不要加此版本否则 App Store 审核被拒".to_string(),
        },
    }
}

fn get_geolocation_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Geolocation".to_string(),
        description: "定位模块（百度地图/高德地图/系统定位）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "baidu-libs-release.aar (百度定位)".to_string(),
                "geolocation-baidu-release.aar (百度定位)".to_string(),
                "geolocation-amap-release.aar (高德定位)".to_string(),
                "uni-getLocation-tencent-uni1-release.aar (腾讯定位)".to_string(),
            ],
            gradle_dependencies: vec![
                "com.amap.api:location:6.4.5 (高德定位)".to_string(),
                "com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8 (腾讯定位)".to_string(),
            ],
            manifest_placeholders: vec![
                "BAIDU_MAP_AK (百度地图)".to_string(),
                "AMAP_KEY (高德地图)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "com.baidu.lbsapi.API_KEY".to_string()), ("android:value".to_string(), "${BAIDU_MAP_AK}".to_string())]),
                HashMap::from([("android:name".to_string(), "amap_api_key".to_string()), ("android:value".to_string(), "${AMAP_KEY}".to_string())]),
            ],
            activities: vec![],
            properties_xml: "<feature name=\"Geolocation\"><module name=\"BaiduMap\"/>(或 Amap)</feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "CoreLocation.framework".to_string(),
                "Security.framework (百度)".to_string(),
            ],
            required_libraries: vec![
                "libcrypto.a (百度)".to_string(),
            ],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![],
            plist_entry: "NSLocationWhenInUseUsageDescription / NSLocationAlwaysAndWhenInUseUsageDescription 必须配置".to_string(),
        },
    }
}

fn get_payment_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Payment".to_string(),
        description: "支付模块（微信支付/支付宝）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "payment-alipay-release.aar (支付宝)".to_string(),
                "payment-weixin-release.aar (微信支付)".to_string(),
                "payment-paypal-release.aar (PayPal)".to_string(),
                "payment-stripe-release.aar (Stripe)".to_string(),
                "payment-google-release.aar (Google Pay)".to_string(),
            ],
            gradle_dependencies: vec![
                "com.alipay.sdk:alipaysdk-android:15.8.11 (支付宝)".to_string(),
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信支付)".to_string(),
                "com.paypal.checkout:android-sdk:0.6.2 (PayPal)".to_string(),
                "com.stripe:stripe-android:18.2.0 (Stripe)".to_string(),
                "com.google.android.gms:play-services-wallet:18.1.3 (Google Pay)".to_string(),
            ],
            manifest_placeholders: vec![
                "WX_APPID (微信支付复用分享的 WX_APPID)".to_string(),
            ],
            manifest_meta_data: vec![],
            activities: vec![
                ".wxapi.WXPayActivity (微信支付回调)".to_string(),
                ".wxapi.WXEntryActivity (微信支付回调)".to_string(),
            ],
            properties_xml: "<feature name=\"Payment\"><module name=\"WeixinPay\"/><module name=\"Alipay\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "CoreTelephony.framework (微信)".to_string(),
                "SystemConfiguration.framework (微信)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.0.tbd (微信)".to_string(),
                "libz.tbd (微信)".to_string(),
            ],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![
                UrlSchemeConfig { scheme: "wx{appid}".to_string(), identifier: "weixin".to_string() },
            ],
            plist_entry: "iOS 微信支付依赖 libWeChatSDK_pay.a（含分享+支付+登录）或 libWeChatSDK.a（仅分享+登录）".to_string(),
        },
    }
}

fn get_login_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Login".to_string(),
        description: "登录模块（微信/QQ/苹果/一键登录）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "oauth-univerify-release.aar (一键登录)".to_string(),
                "oauth-weixin-release.aar (微信登录)".to_string(),
                "oauth-qq-release.aar (QQ登录)".to_string(),
                "open_sdk_XXX_lite.jar (QQ SDK)".to_string(),
                "openDefault-XXX.aar (微博 SDK)".to_string(),
                "oauth-sina-release.aar (微博登录)".to_string(),
                "oauth-miui-release.aar (小米登录)".to_string(),
                "oauth-google-release.aar (Google登录)".to_string(),
                "oauth-facebook-release.aar (Facebook登录)".to_string(),
            ],
            gradle_dependencies: vec![
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信登录)".to_string(),
                "com.getui:gtc-dcloud:3.2.16.7 (一键登录)".to_string(),
                "com.getui:gysdk:3.1.7.0 (一键登录)".to_string(),
                "com.google.android.gms:play-services-auth:19.2.0 (Google登录)".to_string(),
                "com.facebook.android:facebook-login:17.0.2 (Facebook登录)".to_string(),
            ],
            manifest_placeholders: vec![
                "WX_APPID (微信登录)".to_string(),
                "QQ_APPID (QQ登录)".to_string(),
            ],
            manifest_meta_data: vec![],
            activities: vec![
                ".wxapi.WXEntryActivity (微信登录回调)".to_string(),
                "com.tencent.tauth.AuthActivity (QQ登录)".to_string(),
            ],
            properties_xml: "<feature name=\"Login\"><module name=\"WeixinLogin\"/><module name=\"QQLogin\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "AuthenticationServices.framework (Apple 登录)".to_string(),
                "CoreTelephony.framework (微信)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.0.tbd (微信)".to_string(),
                "libz.tbd (微信)".to_string(),
            ],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![
                UrlSchemeConfig { scheme: "wx{appid}".to_string(), identifier: "weixin".to_string() },
            ],
            plist_entry: "Apple Sign-In 需要在 Xcode Signing & Capabilities 添加 Sign in with Apple".to_string(),
        },
    }
}

fn get_map_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Map".to_string(),
        description: "地图模块（高德/腾讯/Google/Apple Maps）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "baidu-libs-release.aar (百度地图)".to_string(),
                "map-baidu-release.aar (百度地图)".to_string(),
                "weex_amap-release.aar (高德 nvue 页面)".to_string(),
                "map-amap-release.aar (高德 vue 页面)".to_string(),
                "weex_google-map-release.aar (Google nvue 页面)".to_string(),
            ],
            gradle_dependencies: vec![
                "com.amap.api:3dmap:latest.release (高德地图，版本以 SDK demo 为准)".to_string(),
                "com.amap.api:search:latest.release (高德搜索，版本以 SDK demo 为准)".to_string(),
                "com.google.android.gms:play-services-maps:18.0.1 (Google地图)".to_string(),
            ],
            manifest_placeholders: vec![
                "AMAP_KEY (高德地图 Key)".to_string(),
                "TENCENT_MAP_KEY (腾讯地图 Key)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "com.amap.api.v2.apikey".to_string()), ("android:value".to_string(), "${AMAP_KEY}".to_string())]),
            ],
            activities: vec![
                "com.amap.api.maps2d.MapActivity (高德地图容器)".to_string(),
            ],
            properties_xml: "<feature name=\"Maps\"><module name=\"Amap\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "CoreLocation.framework".to_string(),
                "AMapFoundationKit.framework (高德基础)".to_string(),
                "MAMapKit.framework (高德地图)".to_string(),
                "QMapKit.framework (腾讯地图)".to_string(),
            ],
            required_libraries: vec![
                "libz.tbd (高德/腾讯)".to_string(),
                "libc++.tbd (高德)".to_string(),
                "libsqlite3.tbd (高德)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("NSLocationWhenInUseUsageDescription".to_string(), "需要您的位置信息来显示附近地点".to_string()),
                ("NSLocationAlwaysAndWhenInUseUsageDescription".to_string(), "需要持续获取位置以提供导航服务".to_string()),
                ("amap_key".to_string(), "(高德地图Key)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 地图模块需在 Info.plist 配置 NSLocation 相关权限描述；高德需在 Podfile 添加 pod 'AMap3DMap'".to_string(),
        },
    }
}

fn get_statistic_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Statistic".to_string(),
        description: "统计分析模块（友盟/腾讯MTA/百度统计/DCloud统计）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "statistic-release.aar".to_string(),
                "statistic-umeng-release.aar (友盟统计)".to_string(),
                "statistic-umeng-gp-release.aar (友盟 Google Play)".to_string(),
                "statistic-google-release.aar (谷歌统计)".to_string(),
            ],
            gradle_dependencies: vec![
                "com.umeng.umsdk:common:9.6.1 (友盟基础库)".to_string(),
                "com.umeng.umsdk:asms:1.8.0 (友盟)".to_string(),
                "com.umeng.umsdk:abtest:1.0.1 (友盟)".to_string(),
                "com.umeng.umsdk:apm:1.9.1 (友盟)".to_string(),
                "com.google.firebase:firebase-analytics:21.3.0 (谷歌统计)".to_string(),
            ],
            manifest_placeholders: vec![
                "UMENG_APPKEY (友盟 AppKey)".to_string(),
                "UMENG_CHANNEL (渠道号, 可选)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "UMENG_APPKEY".to_string()), ("android:value".to_string(), "${UMENG_APPKEY}".to_string())]),
                HashMap::from([("android:name".to_string(), "UMENG_CHANNEL".to_string()), ("android:value".to_string(), "${UMENG_CHANNEL}".to_string())]),
            ],
            activities: vec![],
            properties_xml: "<feature name=\"Statistic\"><module name=\"Umeng\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "UMCommon.framework (友盟)".to_string(),
                "UMAnalytics.framework (友盟统计)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.tbd (友盟)".to_string(),
                "libz.tbd (友盟)".to_string(),
                "libresolv.tbd (友盟)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("UMENG_APPKEY".to_string(), "(友盟AppKey)".to_string()),
                ("UMENG_CHANNEL".to_string(), "(App Store)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 统计模块：友盟需在 Podfile 添加 pod 'UMCCommon' + pod 'UMCSecurityPlugins'; 腾讯MTA 需添加 pod 'MTA'".to_string(),
        },
    }
}

fn get_speech_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Speech".to_string(),
        description: "语音识别模块（讯飞/百度/阿里 + iOS系统语音）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "speech-release.aar".to_string(),
                "speech_baidu-release.aar (百度语音)".to_string(),
                "speech_ifly-release.aar (讯飞语音)".to_string(),
            ],
            gradle_dependencies: vec![],
            manifest_placeholders: vec![
                "IFLY_APPID (讯飞 AppID)".to_string(),
                "BD_SPEECH_APIKEY (百度 API Key)".to_string(),
                "BD_SPEECH_SECRETKEY (百度 Secret Key)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "IFLYTEK_APPKEY".to_string()), ("android:value".to_string(), "${IFLY_APPID}".to_string())]),
            ],
            activities: vec![],
            properties_xml: "<feature name=\"Speech\"><module name=\"Xfyun\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "Speech.framework (系统语音识别)".to_string(),
            ],
            required_libraries: vec![
                "libc++.tbd".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("NSSpeechRecognitionUsageDescription".to_string(), "需要使用语音识别功能来输入文字".to_string()),
                ("NMicrophoneUsageDescription".to_string(), "需要麦克风权限来进行语音输入".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 使用系统 SFSpeechRecognizer 进行语音识别；需在 Xcode 设置 Speech Recognition 能力".to_string(),
        },
    }
}

fn get_face_recognition_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "FaceRecognition".to_string(),
        description: "实人认证模块（DCloud/百度/阿里云）— 仅 Android".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "uni-facialRecognitionVerify-release.aar".to_string(),
                "aliyun-base-XXX.aar".to_string(),
                "aliyun-facade-XXX.aar".to_string(),
                "aliyun-face-XXX.aar".to_string(),
                "aliyun-faceaudio-XXX.aar".to_string(),
                "aliyun-facelanguage-XXX.aar".to_string(),
                "aliyun-photoinus-XXX.aar".to_string(),
                "aliyun-wishverify-XXX.aar".to_string(),
                "Android-XXX.jiagu.aar".to_string(),
                "10042.aar".to_string(),
                "APSecuritySDK-DeepSec.aar".to_string(),
                "facialRecognitionVerify-support-release.aar".to_string(),
            ],
            gradle_dependencies: vec![
                "com.squareup.okhttp3:okhttp:3.11.0".to_string(),
                "com.squareup.okio:okio:1.14.0".to_string(),
                "Com.aliyun.dpa:oss-android-sdk:+".to_string(),
            ],
            manifest_placeholders: vec![
                "DCLOUD_LICENSE (DCloud 许可证)".to_string(),
                "BDFACE_APIKEY (百度 API Key)".to_string(),
                "BDFACE_SECRETKEY (百度 Secret Key)".to_string(),
                "ALIFACE_ACCESSKEY_ID (阿里 AccessKeyId)".to_string(),
                "ALIFACE_ACCESSKEY_SECRET (阿里 AccessKeySecret)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "DCLOUD_LICENSE".to_string()), ("android:value".to_string(), "${DCLOUD_LICENSE}".to_string())]),
            ],
            activities: vec![
                "com.baidu.idl.face.ui.FaceLivenessActivity (百度活体检测)".to_string(),
            ],
            properties_xml: "<feature name=\"FaceRecognition\"><module name=\"DCloud\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![],
            required_libraries: vec![],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![],
            plist_entry: "⚠️ 实人认证模块当前仅支持 Android 平台。iOS 端如需人脸识别请使用原生 Face ID / Vision Framework。".to_string(),
        },
    }
}

fn get_uniad_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "UniAD".to_string(),
        description: "uni-AD 广告模块（穿山甲/优量汇/Gromore/AdMob）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "ads-release.aar".to_string(),
                "ads-csj-release.aar (穿山甲)".to_string(),
                "open_ad_sdk.aar (穿山甲/GroMore)".to_string(),
                "ads-gdt-release.aar (腾讯优量汇)".to_string(),
                "GDTSDK.unionNormal.aar (优量汇/GDT)".to_string(),
                "ads-ks-release.aar (快手广告联盟)".to_string(),
                "ks_adsdk-ad.aar (快手广告联盟)".to_string(),
                "ads-ks-content-release.aar (快手内容联盟)".to_string(),
                "kssdk-allad-content.aar (快手内容联盟)".to_string(),
                "ads-sigmob-release.aar (Sigmob)".to_string(),
                "windAd.aar (Sigmob)".to_string(),
                "wind-common.aar (Sigmob)".to_string(),
                "ads-bd-release.aar (百度广告)".to_string(),
                "Baidu_MobAds_SDK.aar (百度广告)".to_string(),
                "ads-hw-release.aar (华为广告)".to_string(),
                "ads-gromore-release.aar (GroMore)".to_string(),
                "ads-wm-release.aar (uniMP激励视频)".to_string(),
            ],
            gradle_dependencies: vec![
                "com.huawei.hms:ads-lite:13.4.56.302 (华为广告)".to_string(),
                "com.huawei.hms:ads-omsdk:1.3.35 (华为广告)".to_string(),
            ],
            manifest_placeholders: vec![
                "CSJ_APP_ID (穿山甲 AppID)".to_string(),
                "GDT_APPID (优量汇 AppID)".to_string(),
                "ADMOB_APP_ID (AdMob AppID)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "CSJ_APP_ID".to_string()), ("android:value".to_string(), "${CSJ_APP_ID}".to_string())]),
                HashMap::from([("android:name".to_string(), "GDT_APPID".to_string()), ("android:value".to_string(), "${GDT_APPID}".to_string())]),
                HashMap::from([("android:name".to_string(), "com.google.android.gms.ads.APPLICATION_ID".to_string()), ("android:value".to_string(), "${ADMOB_APP_ID}".to_string())]),
            ],
            activities: vec![
                "com.bytedance.sdk.openad.sdk.activity.TTFullScreenVideoActivity (穿山甲全屏视频)".to_string(),
                "com.qq.e.ads.ADActivity (优量汇广告页)".to_string(),
            ],
            properties_xml: "<feature name=\"UniAD\"><module name=\"CSJ\"/><module name=\"GDT\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "BUAdSDK.framework (穿山甲/iOS)".to_string(),
                "GDTMobSDK.framework (优量汇/iOS)".to_string(),
                "GoogleMobileAdsFramework.framework (AdMob)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.tbd (穿山甲)".to_string(),
                "libz.tbd (穿山甲/优量汇)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("CSJ_AppID".to_string(), "(穿山甲 AppID)".to_string()),
                ("GDT_AppKey".to_string(), "(优量汇 AppKey)".to_string()),
                ("GADApplicationIdentifier".to_string(), "(AdMob AppID)".to_string()),
                ("SKAdNetworkItems".to_string(), "需配置 SKAdNetworkIdentifier 列表".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 广告模块：穿山甲需在 Podfile 添加 pod 'Bytedance-UnionADS'；优量汇添加 pod 'GDTMobSDK'；AdMob 添加 pod 'Google-Mobile-Ads-SDK'".to_string(),
        },
    }
}

fn get_x5_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "X5TBS".to_string(),
        description: "腾讯 X5 TBS 内核 WebView — 仅 Android".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "webview-x5-release.aar".to_string(),
                "weex_webview-x5-release.aar (uni-app项目)".to_string(),
            ],
            gradle_dependencies: vec![],
            manifest_placeholders: vec![],
            manifest_meta_data: vec![],
            activities: vec![
                "com.tencent.smtt.sdk.VideoActivity (TBS 视频播放器)".to_string(),
                "com.tencent.smtt.sdk.TbsDownloaderActivity (TBS 下载器)".to_string(),
            ],
            properties_xml: "<feature name=\"X5Webview\" value=\"io.dcloud.feature.X5Webview.X5WebViewService\"/>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![],
            required_libraries: vec![],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![],
            plist_entry: "⚠️ X5 TBS WebView 仅支持 Android 平台。iOS 端默认使用 WKWebView，无需额外配置。".to_string(),
        },
    }
}

fn get_livepusher_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "LivePusher".to_string(),
        description: "直播推流模块 — 主要支持 iOS".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "weex_livepusher-release.aar".to_string(),
            ],
            gradle_dependencies: vec![],
            manifest_placeholders: vec![
                "LIVEPUSH_LICENSE_URL (直播 License URL)".to_string(),
                "LIVEPUSH_LICENSE_KEY (直播 License Key)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "TXLIVE_LICENSE_URL".to_string()), ("android:value".to_string(), "${LIVEPUSH_LICENSE_URL}".to_string())]),
            ],
            activities: vec![
                "com.tencent.liteav.activity.TCActivity (腾讯直播容器)".to_string(),
            ],
            properties_xml: "<feature name=\"LivePusher\"/>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "TLBB.framework (腾讯直播推流)".to_string(),
                "TXLiteAVSDK_Professional.framework (腾讯云音视频)".to_string(),
                "RPLivePlayerLib.framework (七牛推流, 可选)".to_string(),
                "LFLiveKit.framework (LFLiveKit, 可选)".to_string(),
            ],
            required_libraries: vec![
                "libc++.tbd (腾讯直播)".to_string(),
                "libresolv.tbd (腾讯直播)".to_string(),
                "libsqlite3.tbd (腾讯直播)".to_string(),
                "libz.tbd (腾讯直播)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("NSCameraUsageDescription".to_string(), "需要摄像头权限进行直播推流".to_string()),
                ("NSMicrophoneUsageDescription".to_string(), "需要麦克风权限进行直播推流".to_string()),
                ("TXLIVE_LICENSE_URL".to_string(), "(腾讯云直播License URL)".to_string()),
                ("TXLIVE_LICENSE_KEY".to_string(), "(腾讯云直播License Key)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 直播推流模块：推荐使用腾讯云 LiteAVSDK (TXLiteAVSDK)；Podfile 添加 pod 'TXLiteAVSDK_Professional'；需配置相机+麦克风权限".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}.xml", name, uuid::Uuid::new_v4()))
    }

    #[test]
    fn manifest_modules_generate_dcloud_properties() {
        let modules = vec![
            crate::commands::resource::DetectedModule {
                name: "OAuth".to_string(),
                category: "login".to_string(),
                platforms: vec!["all".to_string()],
                configured: false,
                required_keys: vec![],
                source: "manifest.json".to_string(),
            },
            crate::commands::resource::DetectedModule {
                name: "Payment".to_string(),
                category: "payment".to_string(),
                platforms: vec!["all".to_string()],
                configured: false,
                required_keys: vec![],
                source: "manifest.json".to_string(),
            },
            crate::commands::resource::DetectedModule {
                name: "Share".to_string(),
                category: "share".to_string(),
                platforms: vec!["all".to_string()],
                configured: false,
                required_keys: vec![],
                source: "manifest.json".to_string(),
            },
        ];
        let config = module_config_from_detected_modules(&modules);
        let path = temp_file("unipack-dcloud-properties");

        generate_dcloud_properties(&path, &config).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();

        assert!(content.contains(r#"<feature name="Login""#));
        assert!(content.contains(r#"<module name="WeixinLogin"/>"#));
        assert!(content.contains(r#"<feature name="Payment">"#));
        assert!(content.contains(r#"<module name="Alipay"/>"#));
        assert!(content.contains(r#"<feature name="Share""#));
        assert_eq!(content.matches("<features>").count(), 1);
        assert_eq!(content.matches("</features>").count(), 1);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn android_config_report_prefers_manifest_and_lists_missing() {
        let modules = vec![
            crate::commands::resource::DetectedModule {
                name: "Share".to_string(),
                category: "share".to_string(),
                platforms: vec!["android".to_string()],
                configured: false,
                required_keys: vec![],
                source: "manifest.json".to_string(),
            },
            crate::commands::resource::DetectedModule {
                name: "Payment".to_string(),
                category: "payment".to_string(),
                platforms: vec!["ios".to_string()],
                configured: false,
                required_keys: vec![],
                source: "manifest.json".to_string(),
            },
        ];
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "share": {
                            "weixin": { "appid": "wx-demo", "appSecret": "wx-secret" }
                        }
                    }
                }
            }
        });

        let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

        assert_eq!(report.modules.len(), 1);
        let share = &report.modules[0];
        assert_eq!(share.template_key, "share");
        assert_eq!(
            share
                .fields
                .iter()
                .find(|field| field.key == "WX_APPID")
                .and_then(|field| field.value.as_deref()),
            Some("wx-demo")
        );
        assert!(!report
            .missing_required
            .iter()
            .any(|missing| missing.key == "QQ_APPID"));
        assert!(report.all_configured);
    }

    #[test]
    fn android_config_report_prefers_manifest_over_cached_values() {
        let modules = vec![crate::commands::resource::DetectedModule {
            name: "Statistic".to_string(),
            category: "statistic".to_string(),
            platforms: vec!["all".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        }];
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "statistic": { "umeng": { "appkey": "manifest-key" } }
                    }
                }
            }
        });
        let mut user = HashMap::new();
        user.insert("UMENG_APPKEY".to_string(), "cached-key".to_string());

        let report =
            android_module_config_report_from_value(&modules, Some(&manifest), Some(&user));
        let field = report.modules[0]
            .fields
            .iter()
            .find(|field| field.key == "UMENG_APPKEY")
            .unwrap();

        assert_eq!(field.value.as_deref(), Some("manifest-key"));
        assert_eq!(field.value_source.as_deref(), Some("manifest"));
        assert!(report.all_configured);
    }

    #[test]
    fn android_config_report_requires_only_enabled_provider_fields() {
        let modules = vec![crate::commands::resource::DetectedModule {
            name: "Share".to_string(),
            category: "share".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        }];
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "share": {
                            "weixin": { "appid": "wx-only" },
                            "qq": false
                        }
                    }
                }
            }
        });

        let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

        assert!(report
            .missing_required
            .iter()
            .any(|missing| missing.key == "WX_SECRET"));
        assert!(!report
            .missing_required
            .iter()
            .any(|missing| missing.key == "QQ_APPID"));
        let field_keys = report.modules[0]
            .fields
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>();
        assert!(field_keys.contains(&"WX_APPID"));
        assert!(field_keys.contains(&"WX_SECRET"));
        assert!(!field_keys.contains(&"QQ_APPID"));
        assert!(!field_keys.contains(&"SINA_APPKEY"));
    }

    #[test]
    fn android_config_report_shows_only_enabled_oauth_provider_fields() {
        let modules = vec![crate::commands::resource::DetectedModule {
            name: "OAuth".to_string(),
            category: "login".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        }];
        let manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "OAuth": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "oauth": {
                            "weixin": {
                                "appid": "wx-login",
                                "UniversalLinks": "https://example.com/app/"
                            }
                        }
                    }
                }
            }
        });

        let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

        assert_eq!(report.modules.len(), 1);
        let login = &report.modules[0];
        let field_keys = login
            .fields
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(field_keys, vec!["WX_APPID", "WX_SECRET"]);
        assert_eq!(
            login
                .fields
                .iter()
                .find(|field| field.key == "WX_APPID")
                .and_then(|field| field.value.as_deref()),
            Some("wx-login")
        );
        assert!(report
            .missing_required
            .iter()
            .any(|missing| missing.key == "WX_SECRET"));
        assert!(!report
            .missing_required
            .iter()
            .any(|missing| missing.key == "QQ_APPID"));
    }

    #[test]
    fn android_config_report_honors_nested_push_and_platform_providers() {
        let modules = vec![
            crate::commands::resource::DetectedModule {
                name: "Push".to_string(),
                category: "push".to_string(),
                platforms: vec!["android".to_string()],
                configured: false,
                required_keys: vec![],
                source: "manifest.json".to_string(),
            },
            crate::commands::resource::DetectedModule {
                name: "Payment".to_string(),
                category: "payment".to_string(),
                platforms: vec!["android".to_string()],
                configured: false,
                required_keys: vec![],
                source: "manifest.json".to_string(),
            },
        ];
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
                                "offline": true,
                                "mi": { "appid": "mi-app", "appkey": "mi-key" },
                                "hms": { "appid": "huawei-app" },
                                "oppo": false,
                                "vivo": { "__platform__": ["ios"] }
                            }
                        },
                        "payment": {
                            "weixin": {
                                "__platform__": ["ios"],
                                "appid": "wx-ios-only"
                            },
                            "paypal": {
                                "__platform__": ["ios", "android"],
                                "returnURL_android": "paypal-demo"
                            }
                        }
                    }
                }
            }
        });

        let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
        let push = report
            .modules
            .iter()
            .find(|module| module.template_key == "push")
            .unwrap();
        let push_keys = push
            .fields
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>();
        assert!(push_keys.contains(&"XIAOMI_APP_ID"));
        assert!(push_keys.contains(&"XIAOMI_APP_KEY"));
        assert!(push_keys.contains(&"HUAWEI_APP_ID"));
        assert!(!push_keys.contains(&"OPPO_APP_KEY"));
        assert!(!push_keys.contains(&"VIVO_APP_ID"));

        let payment = report
            .modules
            .iter()
            .find(|module| module.template_key == "payment")
            .unwrap();
        let payment_keys = payment
            .fields
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(payment_keys, vec!["PAYPAL_RETURN_SCHEME"]);
        assert_eq!(payment.fields[0].value.as_deref(), Some("paypal-demo"));
    }

    #[test]
    fn android_config_report_requires_enabled_push_vendor_fields() {
        let modules = vec![crate::commands::resource::DetectedModule {
            name: "Push".to_string(),
            category: "push".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        }];
        let manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Push": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "unipush": {
                                "hms": {},
                                "oppo": {},
                                "vivo": false
                            }
                        }
                    }
                }
            }
        });

        let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
        let push = report
            .modules
            .iter()
            .find(|module| module.template_key == "push")
            .unwrap();

        for key in ["HUAWEI_APP_ID", "OPPO_APP_KEY", "OPPO_APP_SECRET"] {
            assert!(push
                .fields
                .iter()
                .any(|field| field.key == key && field.required));
            assert!(report
                .missing_required
                .iter()
                .any(|missing| missing.key == key));
        }
        assert!(!push.fields.iter().any(|field| field.key == "VIVO_APP_ID"));
    }
}
