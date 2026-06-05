use std::collections::HashMap;

use crate::commands::resource::{DetectedModule, UniappManifestInfo};
use crate::commands::shared::module::field_specs::android_config_field_specs;
use crate::commands::shared::module::parsing::normalize_config_key;
use crate::commands::shared::module::templates::{
    android_module_template_key, module_applies_to_android,
};
use crate::commands::shared::module::types::{
    AndroidConfigFieldSpec, AndroidModuleConfigField, AndroidModuleConfigModule,
    AndroidModuleConfigReport, AndroidModuleMissingConfig,
};

// parse_project_modules 与 module_config_from_detected_modules 已移至 parsing.rs

#[tauri::command]
pub async fn analyze_android_module_config(
    manifest_info: UniappManifestInfo,
    user_config: Option<HashMap<String, String>>,
) -> Result<AndroidModuleConfigReport, String> {
    Ok(analyze_android_module_config_sync(
        &manifest_info,
        user_config.as_ref(),
    ))
}

pub fn analyze_android_module_config_sync(
    manifest_info: &UniappManifestInfo,
    user_config: Option<&HashMap<String, String>>,
) -> AndroidModuleConfigReport {
    let manifest_value = manifest_value_from_info(manifest_info);
    android_module_config_report_from_value(
        &manifest_info.detected_modules,
        manifest_value.as_ref(),
        user_config,
    )
}

pub fn android_module_config_report_from_value(
    modules: &[DetectedModule],
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
                field_type: spec.field_type.to_string(),
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

// ---------------------------------------------------------------------------
// Manifest analysis helpers
// ---------------------------------------------------------------------------

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
            "HUAWEI_APP_ID" | "HUAWEI_AGCONNECT_JSON" => manifest_has_enabled_provider(
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

pub fn manifest_value_from_info(manifest_info: &UniappManifestInfo) -> Option<serde_json::Value> {
    manifest_info.manifest_value.clone().or_else(|| {
        std::fs::read_to_string(&manifest_info.manifest_path)
            .ok()
            .and_then(|content| json5::from_str::<serde_json::Value>(&content).ok())
    })
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
#[cfg(test)]
mod analysis_tests;
