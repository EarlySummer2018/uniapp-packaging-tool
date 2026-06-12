pub(crate) fn ios_manifest_info_has_detected_module(
    info: &crate::commands::resource::UniappManifestInfo,
    module_name: &str,
) -> bool {
    info.detected_modules.iter().any(|module| {
        crate::commands::module::android_module_template_key(&module.name)
            == crate::commands::module::android_module_template_key(module_name)
            && crate::commands::shared::module::templates::module_applies_to_ios(&module.platforms)
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IosPrivacyFieldSpec {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) default_value: &'static str,
    pub(crate) required: bool,
}

pub(crate) fn ios_manifest_info_module_enabled(
    info: Option<&crate::commands::resource::UniappManifestInfo>,
    module_name: &str,
) -> bool {
    let Some(info) = info else {
        return false;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return false;
    };
    ios_manifest_info_has_detected_module(info, module_name)
        && ios_manifest_module_enabled(manifest, module_name)
}

pub(crate) fn ios_manifest_module_enabled(manifest: &serde_json::Value, module_name: &str) -> bool {
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
            ios_module_names_equivalent(name, module_name)
                && ios_sdk_config_value_enabled(item, Some("ios"))
        });
    }
    if let Some(map) = modules.as_object() {
        return map.iter().any(|(name, value)| {
            ios_module_names_equivalent(name, module_name)
                && ios_sdk_config_value_enabled(value, Some("ios"))
        });
    }
    false
}

pub(crate) fn ios_module_names_equivalent(left: &str, right: &str) -> bool {
    crate::commands::module::android_module_template_key(left)
        == crate::commands::module::android_module_template_key(right)
        || normalize_ios_manifest_key(left) == normalize_ios_manifest_key(right)
}

pub(crate) fn normalize_ios_manifest_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

pub(crate) fn ios_object_value_normalized<'a>(
    map: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let normalized_key = normalize_ios_manifest_key(key);
    map.iter()
        .find(|(candidate, _)| normalize_ios_manifest_key(candidate) == normalized_key)
        .map(|(_, value)| value)
}

pub(crate) fn ios_sdk_config_value_enabled(
    value: &serde_json::Value,
    platform: Option<&str>,
) -> bool {
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
            enabled && ios_config_value_applies_to_platform(map, platform)
        }
        _ => true,
    }
}

pub(crate) fn insert_missing_plist_string(dict: &mut plist::Dictionary, key: &str, value: &str) {
    let missing_or_empty = dict
        .get(key)
        .and_then(plist::Value::as_string)
        .map(str::trim)
        .is_none_or(str::is_empty);
    if missing_or_empty {
        dict.insert(key.to_string(), plist::Value::String(value.to_string()));
    }
}

pub(crate) fn merge_plist_string_array(dict: &mut plist::Dictionary, key: &str, values: &[&str]) {
    let mut merged = Vec::new();
    if let Some(existing) = dict.get(key) {
        collect_plist_strings(existing, &mut merged);
    }
    merged.extend(values.iter().map(|value| (*value).to_string()));
    let merged = dedup_non_empty_strings(merged);
    if !merged.is_empty() {
        dict.insert(
            key.to_string(),
            plist::Value::Array(merged.into_iter().map(plist::Value::String).collect()),
        );
    }
}

fn collect_plist_strings(value: &plist::Value, output: &mut Vec<String>) {
    match value {
        plist::Value::String(value) => {
            output.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(String::from),
            );
        }
        plist::Value::Array(values) => {
            for value in values {
                collect_plist_strings(value, output);
            }
        }
        _ => {}
    }
}

fn dedup_non_empty_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

pub(crate) fn ios_config_value_applies_to_platform_strict(
    map: &serde_json::Map<String, serde_json::Value>,
    platform: &str,
) -> bool {
    let Some(platforms) = map.get("__platform__") else {
        return false;
    };
    let platform = platform.to_ascii_lowercase();
    ios_platforms_contain(platforms, &platform, false)
}

fn ios_config_value_applies_to_platform(
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
    ios_platforms_contain(platforms, &platform, true)
}

fn ios_platforms_contain(
    platforms: &serde_json::Value,
    platform: &str,
    default_for_unknown: bool,
) -> bool {
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
        _ => default_for_unknown,
    }
}
