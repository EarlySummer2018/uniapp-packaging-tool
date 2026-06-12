use crate::commands::resource::UniappManifestInfo;
use crate::commands::shared::module::parsing::normalize_config_key;

pub(super) fn config_value_applies_to_platform(
    map: &serde_json::Map<String, serde_json::Value>,
    platform: &str,
) -> bool {
    let Some(platforms) = map.get("__platform__") else {
        return true;
    };
    config_platforms_contain(platforms, platform, true)
}

pub(super) fn config_value_applies_to_platform_strict(
    map: &serde_json::Map<String, serde_json::Value>,
    platform: &str,
) -> bool {
    let Some(platforms) = map.get("__platform__") else {
        return false;
    };
    config_platforms_contain(platforms, platform, false)
}

pub(super) fn config_platforms_contain(
    platforms: &serde_json::Value,
    platform: &str,
    default_for_unknown: bool,
) -> bool {
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
        _ => default_for_unknown,
    }
}

pub fn manifest_value_from_info(manifest_info: &UniappManifestInfo) -> Option<serde_json::Value> {
    manifest_info.manifest_value.clone().or_else(|| {
        std::fs::read_to_string(&manifest_info.manifest_path)
            .ok()
            .and_then(|content| json5::from_str::<serde_json::Value>(&content).ok())
    })
}

pub(super) fn get_object_value_normalized<'a>(
    map: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let normalized_key = normalize_config_key(key);
    map.iter()
        .find(|(candidate, _)| normalize_config_key(candidate) == normalized_key)
        .map(|(_, value)| value)
}
