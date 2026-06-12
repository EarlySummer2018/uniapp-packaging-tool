use crate::commands::shared::module::parsing::normalize_config_key;

pub const PUSH_UNSUPPORTED_VERSION_MESSAGE: &str = "Push（消息推送）仅支持2.0版本";

pub fn manifest_push_unipush_v2_enabled(manifest: &serde_json::Value) -> bool {
    manifest_push_requested(manifest)
        && manifest_push_unipush_config(manifest)
            .and_then(|config| config.get("version"))
            .is_some_and(push_version_is_v2)
}

pub fn manifest_push_unsupported_version(manifest: &serde_json::Value) -> bool {
    manifest_push_requested(manifest) && !manifest_push_unipush_v2_enabled(manifest)
}

pub fn manifest_push_requested(manifest: &serde_json::Value) -> bool {
    manifest_push_module_key_enabled(manifest)
        && manifest_push_unipush_config(manifest).is_some_and(config_value_enabled)
}

fn manifest_push_module_key_enabled(manifest: &serde_json::Value) -> bool {
    let Some(modules) = manifest
        .get("app-plus")
        .and_then(|value| value.get("modules"))
    else {
        return false;
    };

    if let Some(map) = modules.as_object() {
        return map
            .iter()
            .any(|(key, value)| module_name_is_push(key) && config_value_enabled(value));
    }

    if let Some(items) = modules.as_array() {
        return items.iter().any(|item| {
            let Some(name) = item
                .get("name")
                .and_then(|value| value.as_str())
                .or_else(|| item.as_str())
            else {
                return false;
            };
            module_name_is_push(name) && config_value_enabled(item)
        });
    }

    false
}

fn manifest_push_unipush_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    object_value_normalized(sdk_configs, "push")
        .and_then(serde_json::Value::as_object)
        .and_then(|push| object_value_normalized(push, "unipush"))
}

fn object_value_normalized<'a>(
    map: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let normalized_key = normalize_config_key(key);
    map.iter()
        .find(|(candidate, _)| normalize_config_key(candidate) == normalized_key)
        .map(|(_, value)| value)
}

fn module_name_is_push(value: &str) -> bool {
    crate::commands::shared::module::templates::android_module_template_key(value) == Some("push")
        || normalize_config_key(value) == "push"
}

fn config_value_enabled(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(flag) => *flag,
        serde_json::Value::Null => false,
        serde_json::Value::Object(map) => map
            .get("enabled")
            .or_else(|| map.get("enable"))
            .or_else(|| map.get("open"))
            .and_then(|value| value.as_bool())
            .unwrap_or(true),
        _ => true,
    }
}

fn push_version_is_v2(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Number(number) => number
            .as_f64()
            .map(|value| (value - 2.0).abs() < f64::EPSILON)
            .unwrap_or(false),
        serde_json::Value::String(value) => value
            .trim()
            .parse::<f64>()
            .map(|value| (value - 2.0).abs() < f64::EPSILON)
            .unwrap_or(false),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_requires_module_key_and_sdk_configs_push_unipush_v2() {
        let manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Push": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "unipush": {
                                "version": "2"
                            }
                        }
                    }
                }
            }
        });

        assert!(manifest_push_requested(&manifest));
        assert!(manifest_push_unipush_v2_enabled(&manifest));
        assert!(!manifest_push_unsupported_version(&manifest));
    }

    #[test]
    fn push_is_not_enabled_without_sdk_configs_push_unipush_path() {
        let manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Push": {}
                },
                "distribute": {
                    "push": {
                        "unipush": {
                            "version": "2"
                        }
                    }
                }
            }
        });

        assert!(!manifest_push_requested(&manifest));
        assert!(!manifest_push_unipush_v2_enabled(&manifest));
        assert!(!manifest_push_unsupported_version(&manifest));
    }

    #[test]
    fn push_reports_unsupported_when_requested_version_is_not_v2() {
        let manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Push": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "unipush": {
                                "version": 1
                            }
                        }
                    }
                }
            }
        });

        assert!(manifest_push_requested(&manifest));
        assert!(!manifest_push_unipush_v2_enabled(&manifest));
        assert!(manifest_push_unsupported_version(&manifest));
    }
}
