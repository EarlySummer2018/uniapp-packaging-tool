use std::collections::HashMap;

use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::field_specs::android_config_field_specs;
use crate::commands::shared::module::parsing::normalize_config_key;
use crate::commands::shared::module::push::manifest_push_unipush_v2_enabled;
use crate::commands::shared::module::templates::{
    android_module_template_key, module_applies_to_android,
};
use crate::commands::shared::module::types::{
    AndroidConfigFieldSpec, AndroidModuleConfigField, AndroidModuleConfigModule,
    AndroidModuleConfigReport, AndroidModuleMissingConfig,
};

use super::android_manifest::{
    android_amap_map_enabled, android_geolocation_enabled, android_geolocation_provider_enabled,
    find_manifest_config_value, manifest_has_any_enabled_module, manifest_has_enabled_provider,
};
use super::payment::{payment_provider_enabled_for_platform, PaymentProvider};

const DEFAULT_AMAP_MAP_SDK_VERSION: &str = "10.0.700_loc6.4.5_sea9.7.2";
const DEFAULT_TENCENT_LOCATION_SDK_VERSION: &str = "7.5.4.8";
const DEFAULT_ANDROIDX_VERSION: &str = "1.0.0";

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
        if template_key == "push" && !manifest.is_some_and(manifest_push_unipush_v2_enabled) {
            continue;
        }
        if template_key == "geolocation" && !android_geolocation_enabled(manifest) {
            continue;
        }
        if template_key == "payment" && !manifest.is_some_and(android_payment_enabled) {
            continue;
        }

        let mut fields = Vec::new();
        for spec in android_config_field_specs(template_key) {
            if !android_field_visible_for_manifest(template_key, spec, manifest) {
                continue;
            }
            let required = android_field_required_for_manifest(template_key, spec, manifest);
            let (value, value_source) = if template_key == "map" && spec.key == "MAP_PAGE_TYPE" {
                android_map_page_type_field_value(manifest, user_config, spec)
            } else if template_key == "map" && spec.key == "AMAP_SDK_VERSION" {
                android_user_defaulted_field_value(
                    user_config,
                    template_key,
                    spec,
                    DEFAULT_AMAP_MAP_SDK_VERSION,
                )
            } else if template_key == "geolocation" && spec.key == "TENCENT_LOCATION_SDK_VERSION" {
                android_user_defaulted_field_value(
                    user_config,
                    template_key,
                    spec,
                    DEFAULT_TENCENT_LOCATION_SDK_VERSION,
                )
            } else if template_key == "payment" && spec.key == "androidxVersion" {
                android_user_defaulted_field_value(
                    user_config,
                    template_key,
                    spec,
                    DEFAULT_ANDROIDX_VERSION,
                )
            } else {
                let user_value = user_config_field_value(user_config, template_key, spec);
                let manifest_value =
                    manifest.and_then(|value| find_manifest_config_value(value, spec));
                if let Some(value) = manifest_value {
                    (Some(value), Some("manifest".to_string()))
                } else if let Some(value) = user_value {
                    (Some(value), Some("user".to_string()))
                } else {
                    (None, None)
                }
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

        report.modules.push(AndroidModuleConfigModule {
            name: module.name.clone(),
            template_key: template_key.to_string(),
            category: module.category.clone(),
            platforms: module.platforms.clone(),
            source: module.source.clone(),
            fields,
        });
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
            _ if !manifest_push_unipush_v2_enabled(manifest) => false,
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
            "BAIDU_MAP_AK" => android_geolocation_provider_enabled(manifest, &["baidu", "bd"]),
            "AMAP_KEY" => {
                android_geolocation_provider_enabled(manifest, &["amap", "gaode"])
                    && !android_amap_map_enabled(Some(manifest))
            }
            "TENCENT_MAP_KEY" | "TENCENT_LOCATION_SDK_VERSION" => {
                android_geolocation_provider_enabled(manifest, &["tencent", "qqmap"])
            }
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
            "MAP_PAGE_TYPE" => manifest_has_any_map_provider(manifest),
            "BAIDU_MAP_AK" => {
                manifest_has_enabled_provider(manifest, &["maps", "map"], &["baidu", "bd"])
            }
            "AMAP_KEY" => {
                manifest_has_enabled_provider(manifest, &["maps", "map"], &["amap", "gaode"])
            }
            "AMAP_SDK_VERSION" => {
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
            "WX_APPID" => {
                manifest_has_enabled_provider(
                    manifest,
                    &["payment", "pay", "payments"],
                    &["weixin", "wechat", "wx"],
                ) && payment_provider_enabled_for_platform(
                    manifest,
                    PaymentProvider::Weixin,
                    "android",
                )
            }
            "PAYPAL_RETURN_SCHEME" => {
                payment_provider_enabled_for_platform(manifest, PaymentProvider::Paypal, "android")
            }
            "androidxVersion" => {
                payment_provider_enabled_for_platform(manifest, PaymentProvider::Stripe, "android")
                    || payment_provider_enabled_for_platform(
                        manifest,
                        PaymentProvider::Google,
                        "android",
                    )
            }
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

fn android_payment_enabled(manifest: &serde_json::Value) -> bool {
    [
        PaymentProvider::Alipay,
        PaymentProvider::Weixin,
        PaymentProvider::Paypal,
        PaymentProvider::Stripe,
        PaymentProvider::Google,
    ]
    .iter()
    .any(|provider| payment_provider_enabled_for_platform(manifest, *provider, "android"))
}

fn android_map_page_type_field_value(
    manifest: Option<&serde_json::Value>,
    user_config: Option<&HashMap<String, String>>,
    spec: &AndroidConfigFieldSpec,
) -> (Option<String>, Option<String>) {
    let provider = manifest
        .and_then(android_map_provider_for_manifest)
        .unwrap_or("amap");
    let default_value = android_default_map_page_type(provider);
    let user_value = user_config_field_value(user_config, "map", spec);
    let manifest_value = manifest.and_then(|value| find_manifest_config_value(value, spec));

    if let Some(value) = manifest_value {
        let normalized = normalize_android_map_page_type(provider, &value);
        if normalized != default_value
            || normalize_config_key(&value) == normalize_config_key(default_value)
        {
            return (Some(normalized.to_string()), Some("manifest".to_string()));
        }
    }
    if let Some(value) = user_value {
        let normalized = normalize_android_map_page_type(provider, &value);
        if normalized != default_value
            || normalize_config_key(&value) == normalize_config_key(default_value)
        {
            return (Some(normalized.to_string()), Some("user".to_string()));
        }
    }

    (Some(default_value.to_string()), Some("default".to_string()))
}

fn android_user_defaulted_field_value(
    user_config: Option<&HashMap<String, String>>,
    template_key: &str,
    spec: &AndroidConfigFieldSpec,
    default_value: &str,
) -> (Option<String>, Option<String>) {
    let user_value = user_config_field_value(user_config, template_key, spec);
    if let Some(value) = user_value {
        return (Some(value), Some("user".to_string()));
    }

    (Some(default_value.to_string()), Some("default".to_string()))
}

fn user_config_field_value(
    user_config: Option<&HashMap<String, String>>,
    template_key: &str,
    spec: &AndroidConfigFieldSpec,
) -> Option<String> {
    let config = user_config?;
    let scoped_key = if template_key.is_empty() {
        None
    } else {
        Some(format!("{}.{}", template_key, spec.key))
    };

    scoped_key
        .as_deref()
        .into_iter()
        .chain(std::iter::once(spec.key))
        .find_map(|key| {
            config
                .get(key)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
}

fn android_default_map_page_type(provider: &str) -> &'static str {
    match provider {
        "google" => "nvue",
        _ => "vue",
    }
}

fn normalize_android_map_page_type(provider: &str, value: &str) -> &'static str {
    match provider {
        "baidu" => "vue",
        "google" => "nvue",
        _ if normalize_config_key(value) == "nvue" => "nvue",
        _ => "vue",
    }
}

fn manifest_has_any_map_provider(manifest: &serde_json::Value) -> bool {
    ["baidu", "bd", "amap", "gaode", "google", "tencent", "qqmap"]
        .iter()
        .any(|provider| manifest_has_enabled_provider(manifest, &["maps", "map"], &[*provider]))
}

fn android_map_provider_for_manifest(manifest: &serde_json::Value) -> Option<&'static str> {
    if manifest_has_enabled_provider(manifest, &["maps", "map"], &["baidu", "bd"]) {
        Some("baidu")
    } else if manifest_has_enabled_provider(manifest, &["maps", "map"], &["amap", "gaode"]) {
        Some("amap")
    } else if manifest_has_enabled_provider(manifest, &["maps", "map"], &["google"]) {
        Some("google")
    } else if manifest_has_enabled_provider(manifest, &["maps", "map"], &["tencent", "qqmap"]) {
        Some("tencent")
    } else {
        None
    }
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
            "WX_APPID" => {
                payment_provider_enabled_for_platform(manifest, PaymentProvider::Weixin, "android")
            }
            "PAYPAL_RETURN_SCHEME" => {
                payment_provider_enabled_for_platform(manifest, PaymentProvider::Paypal, "android")
            }
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
        "geolocation" => {
            matches!(spec.key, "BAIDU_MAP_AK" | "AMAP_KEY" | "TENCENT_MAP_KEY")
                && android_field_visible_for_manifest(template_key, spec, Some(manifest))
        }
        "map" => {
            matches!(
                spec.key,
                "BAIDU_MAP_AK" | "AMAP_KEY" | "GOOGLE_MAPS_API_KEY" | "TENCENT_MAP_KEY"
            ) && android_field_visible_for_manifest(template_key, spec, Some(manifest))
        }
        "share" => {
            matches!(
                spec.key,
                "SINA_APPKEY" | "SINA_SECRET" | "SINA_REDIRECT_URI"
            ) && android_field_visible_for_manifest(template_key, spec, Some(manifest))
        }
        _ => false,
    }
}
