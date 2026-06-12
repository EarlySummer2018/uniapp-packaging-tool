use std::collections::{BTreeMap, HashMap};

use crate::commands::ios::modules::barcode::IOS_BARCODE_PRIVACY_FIELDS;
use crate::commands::ios::modules::bluetooth::IOS_BLUETOOTH_PRIVACY_FIELDS;
use crate::commands::ios::modules::camera::IOS_CAMERA_PRIVACY_FIELDS;
use crate::commands::ios::modules::common::IosPrivacyFieldSpec;
use crate::commands::ios::modules::contacts::IOS_CONTACTS_PRIVACY_FIELDS;
use crate::commands::ios::modules::face_id::IOS_FACE_ID_PRIVACY_FIELDS;
use crate::commands::ios::modules::fingerprint::IOS_FINGERPRINT_PRIVACY_FIELDS;
use crate::commands::ios::modules::geolocation::IOS_GEOLOCATION_PRIVACY_FIELDS;
use crate::commands::ios::modules::ibeacon::IOS_IBEACON_PRIVACY_FIELDS;
use crate::commands::ios::modules::livepusher::IOS_LIVEPUSHER_PRIVACY_FIELDS;
use crate::commands::ios::modules::record::IOS_RECORD_PRIVACY_FIELDS;
use crate::commands::ios::modules::video_player::ios_video_player_allows_arbitrary_loads;
use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::push::manifest_push_unipush_v2_enabled;
use crate::commands::shared::module::templates::{
    android_module_template_key, get_module_template_sync, module_applies_to_ios,
};
use crate::commands::shared::module::types::{
    IosModuleConfigField, IosModuleConfigModule, IosModuleConfigReport,
};

use super::android_manifest::android_module_names_equivalent;
use super::common::{
    config_value_applies_to_platform, config_value_applies_to_platform_strict,
    get_object_value_normalized,
};

pub fn ios_module_config_report_from_value(
    modules: &[DetectedModule],
    manifest: Option<&serde_json::Value>,
    ios_privacy_descriptions: Option<&BTreeMap<String, String>>,
    user_config: Option<&HashMap<String, String>>,
) -> IosModuleConfigReport {
    let mut report = IosModuleConfigReport::default();

    for module in modules {
        let Some(template_key) = android_module_template_key(&module.name) else {
            continue;
        };
        if !module_applies_to_ios(&module.platforms) {
            continue;
        }
        if template_key == "geolocation" {
            if let Some(module_config) = ios_geolocation_module_config(
                module,
                manifest,
                ios_privacy_descriptions,
                user_config,
            ) {
                report.modules.push(module_config);
            }
            continue;
        }
        if template_key == "push" {
            if let Some(module_config) = ios_push_module_config(module, manifest, user_config) {
                report.modules.push(module_config);
            }
            continue;
        }
        match template_key {
            "barcode" | "camera" | "contacts" | "face_id" | "fingerprint" | "ibeacon"
            | "livepusher" | "record" => {
                if let Some(module_config) = ios_privacy_module_config(
                    module,
                    manifest,
                    ios_privacy_descriptions,
                    user_config,
                ) {
                    report.modules.push(module_config);
                }
                continue;
            }
            "bluetooth" => {
                if let Some(module_config) = ios_bluetooth_module_config(
                    module,
                    manifest,
                    ios_privacy_descriptions,
                    user_config,
                ) {
                    report.modules.push(module_config);
                }
                continue;
            }
            "video_player" => {
                if let Some(module_config) =
                    ios_video_player_module_config(module, manifest, user_config)
                {
                    report.modules.push(module_config);
                }
                continue;
            }
            _ => {}
        }

        if get_module_template_sync(template_key).is_err() {
            continue;
        }

        report.modules.push(IosModuleConfigModule {
            name: module.name.clone(),
            template_key: template_key.to_string(),
            category: module.category.clone(),
            platforms: module.platforms.clone(),
            source: module.source.clone(),
            fields: Vec::new(),
        });
    }

    report
}

fn ios_push_module_config(
    module: &DetectedModule,
    manifest: Option<&serde_json::Value>,
    user_config: Option<&HashMap<String, String>>,
) -> Option<IosModuleConfigModule> {
    let manifest = manifest?;
    if !manifest_push_unipush_v2_enabled(manifest) {
        return None;
    }
    let push_config = ios_push_sdk_config(manifest);
    Some(IosModuleConfigModule {
        name: module.name.clone(),
        template_key: "push".to_string(),
        category: module.category.clone(),
        platforms: module.platforms.clone(),
        source: module.source.clone(),
        fields: vec![
            ios_push_getui_field(
                push_config,
                "getui.appid",
                "个推 AppID",
                &[
                    "appid_ios",
                    "appId_ios",
                    "GETUI_APPID",
                    "plus.unipush.appid",
                    "unipush_appid",
                    "appid",
                    "appId",
                ],
                false,
                user_config,
            ),
            ios_push_getui_field(
                push_config,
                "getui.appkey",
                "个推 AppKey",
                &[
                    "appkey_ios",
                    "appKey_ios",
                    "plus.unipush.appkey",
                    "unipush_appkey",
                    "appkey",
                    "appKey",
                ],
                true,
                user_config,
            ),
            ios_push_getui_field(
                push_config,
                "getui.appsecret",
                "个推 AppSecret",
                &[
                    "appsecret_ios",
                    "appSecret_ios",
                    "plus.unipush.appsecret",
                    "unipush_appsecret",
                    "appsecret",
                    "appSecret",
                ],
                true,
                user_config,
            ),
        ],
    })
}

fn ios_push_getui_field(
    push_config: Option<&serde_json::Value>,
    field_key: &str,
    label: &str,
    aliases: &[&str],
    secret: bool,
    user_config: Option<&HashMap<String, String>>,
) -> IosModuleConfigField {
    let (value, value_source) =
        if let Some(value) = ios_user_config_field_value(user_config, "push", field_key) {
            (Some(value), Some("user".to_string()))
        } else if let Some(value) =
            push_config.and_then(|value| json_string_field_nested(value, aliases))
        {
            (Some(value), Some("manifest".to_string()))
        } else {
            (None, None)
        };

    IosModuleConfigField {
        key: field_key.to_string(),
        label: label.to_string(),
        required: true,
        secret,
        value,
        value_source,
        placeholder: format!("请输入 {}", label),
        field_type: "text".to_string(),
    }
}

fn ios_geolocation_module_config(
    module: &DetectedModule,
    manifest: Option<&serde_json::Value>,
    ios_privacy_descriptions: Option<&BTreeMap<String, String>>,
    user_config: Option<&HashMap<String, String>>,
) -> Option<IosModuleConfigModule> {
    let manifest = manifest?;
    if !ios_manifest_module_enabled(manifest, "Geolocation") {
        return None;
    }
    let geolocation_config = ios_geolocation_sdk_config(manifest)?;
    if !ios_config_value_enabled(geolocation_config) {
        return None;
    }

    let mut fields = Vec::new();
    let mut has_ios_provider = false;
    if ios_geolocation_provider_enabled(geolocation_config, &["system"]) {
        has_ios_provider = true;
    }
    if ios_geolocation_provider_enabled(geolocation_config, &["baidu", "bd"]) {
        has_ios_provider = true;
        fields.push(ios_geolocation_app_key_field(
            geolocation_config,
            "baidu",
            &["baidu", "bd"],
            "百度定位 AppKey",
            &["appkey_ios", "apikey_ios", "appkey", "apikey", "ak", "key"],
            user_config,
        ));
    }
    if ios_geolocation_provider_enabled(geolocation_config, &["amap", "gaode"]) {
        has_ios_provider = true;
        fields.push(ios_geolocation_app_key_field(
            geolocation_config,
            "amap",
            &["amap", "gaode"],
            "高德定位 AppKey",
            &["appkey_ios", "apikey_ios", "appkey", "apikey", "key"],
            user_config,
        ));
    }
    if !has_ios_provider {
        return None;
    }

    fields.extend(IOS_GEOLOCATION_PRIVACY_FIELDS.iter().map(|field| {
        ios_privacy_field("geolocation", field, ios_privacy_descriptions, user_config)
    }));

    Some(IosModuleConfigModule {
        name: module.name.clone(),
        template_key: "geolocation".to_string(),
        category: module.category.clone(),
        platforms: module.platforms.clone(),
        source: module.source.clone(),
        fields,
    })
}

fn ios_privacy_module_config(
    module: &DetectedModule,
    manifest: Option<&serde_json::Value>,
    ios_privacy_descriptions: Option<&BTreeMap<String, String>>,
    user_config: Option<&HashMap<String, String>>,
) -> Option<IosModuleConfigModule> {
    let manifest = manifest?;
    let Some(template_key) = android_module_template_key(&module.name) else {
        return None;
    };
    if !ios_manifest_module_enabled(manifest, &module.name) {
        return None;
    }
    let privacy_fields = match template_key {
        "barcode" => IOS_BARCODE_PRIVACY_FIELDS,
        "camera" => IOS_CAMERA_PRIVACY_FIELDS,
        "contacts" => IOS_CONTACTS_PRIVACY_FIELDS,
        "face_id" => IOS_FACE_ID_PRIVACY_FIELDS,
        "fingerprint" => IOS_FINGERPRINT_PRIVACY_FIELDS,
        "ibeacon" => IOS_IBEACON_PRIVACY_FIELDS,
        "livepusher" => IOS_LIVEPUSHER_PRIVACY_FIELDS,
        "record" => IOS_RECORD_PRIVACY_FIELDS,
        _ => return None,
    };
    Some(IosModuleConfigModule {
        name: module.name.clone(),
        template_key: template_key.to_string(),
        category: module.category.clone(),
        platforms: module.platforms.clone(),
        source: module.source.clone(),
        fields: privacy_fields
            .iter()
            .map(|field| {
                ios_privacy_field(template_key, field, ios_privacy_descriptions, user_config)
            })
            .collect(),
    })
}

fn ios_bluetooth_module_config(
    module: &DetectedModule,
    manifest: Option<&serde_json::Value>,
    ios_privacy_descriptions: Option<&BTreeMap<String, String>>,
    user_config: Option<&HashMap<String, String>>,
) -> Option<IosModuleConfigModule> {
    let manifest = manifest?;
    if !ios_manifest_module_enabled(manifest, "Bluetooth") {
        return None;
    }
    let mut fields = IOS_BLUETOOTH_PRIVACY_FIELDS
        .iter()
        .map(|field| ios_privacy_field("bluetooth", field, ios_privacy_descriptions, user_config))
        .collect::<Vec<_>>();
    fields.push(ios_bluetooth_background_field(manifest, user_config));

    Some(IosModuleConfigModule {
        name: module.name.clone(),
        template_key: "bluetooth".to_string(),
        category: module.category.clone(),
        platforms: module.platforms.clone(),
        source: module.source.clone(),
        fields,
    })
}

fn ios_video_player_module_config(
    module: &DetectedModule,
    manifest: Option<&serde_json::Value>,
    user_config: Option<&HashMap<String, String>>,
) -> Option<IosModuleConfigModule> {
    let manifest = manifest?;
    if !ios_manifest_module_enabled(manifest, "VideoPlayer") {
        return None;
    }
    Some(IosModuleConfigModule {
        name: module.name.clone(),
        template_key: "video_player".to_string(),
        category: module.category.clone(),
        platforms: module.platforms.clone(),
        source: module.source.clone(),
        fields: vec![ios_video_player_ats_field(manifest, user_config)],
    })
}

fn ios_geolocation_app_key_field(
    geolocation_config: &serde_json::Value,
    canonical_provider: &str,
    provider_keys: &[&str],
    label: &str,
    config_keys: &[&str],
    user_config: Option<&HashMap<String, String>>,
) -> IosModuleConfigField {
    let field_key = format!("{}.appkey_ios", canonical_provider);
    let (value, value_source) =
        if let Some(value) = ios_user_config_field_value(user_config, "geolocation", &field_key) {
            (Some(value), Some("user".to_string()))
        } else if let Some(value) =
            ios_geolocation_provider_string_field(geolocation_config, provider_keys, config_keys)
        {
            (Some(value), Some("manifest".to_string()))
        } else {
            (None, None)
        };

    IosModuleConfigField {
        key: field_key,
        label: label.to_string(),
        required: true,
        secret: true,
        value,
        value_source,
        placeholder: "请输入 iOS AppKey".to_string(),
        field_type: "text".to_string(),
    }
}

fn ios_privacy_field(
    template_key: &str,
    field: &IosPrivacyFieldSpec,
    ios_privacy_descriptions: Option<&BTreeMap<String, String>>,
    user_config: Option<&HashMap<String, String>>,
) -> IosModuleConfigField {
    let key = field.key;
    let field_key = format!("privacy.{}", key);
    let (value, value_source) =
        if let Some(value) = ios_user_config_field_value(user_config, template_key, &field_key) {
            (Some(value), Some("user".to_string()))
        } else if let Some(value) = ios_privacy_descriptions
            .and_then(|descriptions| descriptions.get(key))
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
        {
            (Some(value), Some("manifest".to_string()))
        } else if field.required {
            (
                Some(field.default_value.to_string()),
                Some("default".to_string()),
            )
        } else {
            (None, None)
        };

    IosModuleConfigField {
        key: field_key,
        label: field.label.to_string(),
        required: field.required,
        secret: false,
        value,
        value_source,
        placeholder: field.default_value.to_string(),
        field_type: "textarea".to_string(),
    }
}

fn ios_video_player_ats_field(
    manifest: &serde_json::Value,
    user_config: Option<&HashMap<String, String>>,
) -> IosModuleConfigField {
    let field_key = "allowArbitraryLoads";
    let (value, value_source) =
        if let Some(value) = ios_user_config_field_value(user_config, "video_player", field_key) {
            (
                Some(normalize_bool_field_value(&value).to_string()),
                Some("user".to_string()),
            )
        } else if ios_video_player_allows_arbitrary_loads(manifest) {
            (Some("true".to_string()), Some("manifest".to_string()))
        } else {
            (Some("false".to_string()), Some("default".to_string()))
        };

    IosModuleConfigField {
        key: field_key.to_string(),
        label: "允许任意网络视频加载".to_string(),
        required: false,
        secret: false,
        value,
        value_source,
        placeholder: "默认关闭".to_string(),
        field_type: "select".to_string(),
    }
}

fn ios_bluetooth_background_field(
    manifest: &serde_json::Value,
    user_config: Option<&HashMap<String, String>>,
) -> IosModuleConfigField {
    let field_key = "backgroundBluetooth";
    let (value, value_source) =
        if let Some(value) = ios_user_config_field_value(user_config, "bluetooth", field_key) {
            (
                Some(normalize_bool_field_value(&value).to_string()),
                Some("user".to_string()),
            )
        } else if ios_manifest_has_bluetooth_background_modes(manifest) {
            (Some("true".to_string()), Some("manifest".to_string()))
        } else {
            (Some("false".to_string()), Some("default".to_string()))
        };

    IosModuleConfigField {
        key: field_key.to_string(),
        label: "后台蓝牙功能".to_string(),
        required: false,
        secret: false,
        value,
        value_source,
        placeholder: "默认关闭".to_string(),
        field_type: "select".to_string(),
    }
}

fn ios_manifest_has_bluetooth_background_modes(manifest: &serde_json::Value) -> bool {
    let mut modes = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("UIBackgroundModes"))
    {
        collect_json_strings(value, &mut modes);
    }
    modes
        .iter()
        .any(|mode| matches!(mode.as_str(), "bluetooth-central" | "bluetooth-peripheral"))
}

fn collect_json_strings(value: &serde_json::Value, output: &mut Vec<String>) {
    match value {
        serde_json::Value::String(value) => {
            output.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string),
            );
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_json_strings(value, output);
            }
        }
        _ => {}
    }
}

fn normalize_bool_field_value(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" | "是" | "开启" => "true",
        _ => "false",
    }
}

fn ios_geolocation_provider_string_field(
    geolocation_config: &serde_json::Value,
    provider_keys: &[&str],
    config_keys: &[&str],
) -> Option<String> {
    let config = geolocation_config.as_object()?;
    provider_keys.iter().find_map(|provider_key| {
        let value = get_object_value_normalized(config, provider_key)?;
        if !ios_geolocation_provider_value_enabled(value) {
            return None;
        }
        json_string_field(value, config_keys)
    })
}

fn json_string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

fn ios_user_config_field_value(
    user_config: Option<&HashMap<String, String>>,
    template_key: &str,
    field_key: &str,
) -> Option<String> {
    let config = user_config?;
    let scoped_key = if template_key.is_empty() {
        None
    } else {
        Some(format!("{}.{}", template_key, field_key))
    };

    scoped_key
        .as_deref()
        .into_iter()
        .chain(std::iter::once(field_key))
        .find_map(|key| {
            config
                .get(key)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
}

fn ios_manifest_module_enabled(manifest: &serde_json::Value, module_name: &str) -> bool {
    let Some(modules) = manifest
        .get("app-plus")
        .and_then(|value| value.get("modules"))
    else {
        return false;
    };
    if let Some(items) = modules.as_array() {
        return items.iter().any(|item| {
            let Some(name) = item
                .get("name")
                .and_then(|value| value.as_str())
                .or_else(|| item.as_str())
            else {
                return false;
            };
            android_module_names_equivalent(name, module_name) && ios_config_value_enabled(item)
        });
    }
    if let Some(map) = modules.as_object() {
        return map.iter().any(|(name, value)| {
            android_module_names_equivalent(name, module_name) && ios_config_value_enabled(value)
        });
    }
    false
}

fn ios_geolocation_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["geolocation", "location", "position"]
        .iter()
        .find_map(|key| get_object_value_normalized(sdk_configs, key))
}

fn ios_push_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    let config = ["push", "unipush", "getui"]
        .iter()
        .find_map(|key| get_object_value_normalized(sdk_configs, key))?;
    ios_config_value_enabled(config).then_some(config)
}

fn json_string_field_nested(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    let map = value.as_object()?;
    if !ios_config_value_enabled(value) {
        return None;
    }
    for key in keys {
        if let Some(value) = get_object_value_normalized(map, key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(value.to_string());
        }
    }
    for key in ["getui", "unipush", "igt", "ios"] {
        if let Some(value) = get_object_value_normalized(map, key)
            .and_then(|value| json_string_field_nested(value, keys))
        {
            return Some(value);
        }
    }
    if map.len() == 1 {
        return map
            .values()
            .next()
            .and_then(|value| json_string_field_nested(value, keys));
    }
    None
}

fn ios_geolocation_provider_enabled(
    geolocation_config: &serde_json::Value,
    provider_keys: &[&str],
) -> bool {
    let Some(config) = geolocation_config.as_object() else {
        return false;
    };
    provider_keys.iter().any(|provider_key| {
        get_object_value_normalized(config, provider_key)
            .is_some_and(ios_geolocation_provider_value_enabled)
    })
}

fn ios_geolocation_provider_value_enabled(value: &serde_json::Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    let enabled = map
        .get("enabled")
        .or_else(|| map.get("enable"))
        .or_else(|| map.get("open"))
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    enabled && config_value_applies_to_platform_strict(map, "ios")
}

fn ios_config_value_enabled(value: &serde_json::Value) -> bool {
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
            enabled && config_value_applies_to_platform(map, "ios")
        }
        _ => true,
    }
}
