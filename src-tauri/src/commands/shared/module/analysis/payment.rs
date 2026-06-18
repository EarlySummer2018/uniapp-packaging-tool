use super::android_manifest::android_module_names_equivalent;
use super::common::get_object_value_normalized;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentProvider {
    Alipay,
    Weixin,
    Paypal,
    Stripe,
    Google,
    Apple,
    Iap,
}

impl PaymentProvider {
    pub(crate) fn keys(self) -> &'static [&'static str] {
        match self {
            Self::Alipay => &["alipay", "ali"],
            Self::Weixin => &["weixin", "wechat", "wx"],
            Self::Paypal => &["paypal"],
            Self::Stripe => &["stripe"],
            Self::Google => &["google", "googlepay", "google_pay"],
            Self::Apple => &["apple", "applepay", "apple_pay"],
            Self::Iap => &["iap", "appleiap", "apple_iap", "inapp", "in_app_purchase"],
        }
    }

    fn requires_platform(self) -> bool {
        matches!(
            self,
            Self::Alipay | Self::Weixin | Self::Paypal | Self::Stripe
        )
    }
}

pub fn payment_provider_enabled_for_platform(
    manifest: &serde_json::Value,
    provider: PaymentProvider,
    platform: &str,
) -> bool {
    if platform.eq_ignore_ascii_case("ios") && provider == PaymentProvider::Google {
        return false;
    }
    if platform.eq_ignore_ascii_case("android")
        && matches!(provider, PaymentProvider::Apple | PaymentProvider::Iap)
    {
        return false;
    }
    if !manifest_payment_module_enabled(manifest) {
        return false;
    }
    let Some(payment) = manifest_payment_sdk_config(manifest) else {
        return false;
    };
    if !payment_config_value_enabled(payment) {
        return false;
    }
    let Some(map) = payment.as_object() else {
        return false;
    };

    provider.keys().iter().any(|provider_key| {
        get_object_value_normalized(map, provider_key).is_some_and(|value| {
            payment_provider_value_enabled(value, provider.requires_platform().then_some(platform))
        })
    })
}

pub fn manifest_payment_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["payment", "pay", "payments"]
        .iter()
        .find_map(|key| get_object_value_normalized(sdk_configs, key))
}

pub fn manifest_payment_provider_value<'a>(
    manifest: &'a serde_json::Value,
    provider: PaymentProvider,
    platform: &str,
) -> Option<&'a serde_json::Value> {
    if !payment_provider_enabled_for_platform(manifest, provider, platform) {
        return None;
    }
    let payment = manifest_payment_sdk_config(manifest)?.as_object()?;
    provider
        .keys()
        .iter()
        .find_map(|key| get_object_value_normalized(payment, key))
}

fn manifest_payment_module_enabled(manifest: &serde_json::Value) -> bool {
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
            android_module_names_equivalent(name, "Payment") && module_switch_value_enabled(item)
        });
    }

    if let Some(map) = modules.as_object() {
        return map.iter().any(|(name, value)| {
            android_module_names_equivalent(name, "Payment") && module_switch_value_enabled(value)
        });
    }

    false
}

fn module_switch_value_enabled(value: &serde_json::Value) -> bool {
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

fn payment_provider_value_enabled(value: &serde_json::Value, platform: Option<&str>) -> bool {
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
            enabled
                && platform.is_none_or(|platform| payment_value_applies_to_platform(map, platform))
        }
        _ => true,
    }
}

fn payment_config_value_enabled(value: &serde_json::Value) -> bool {
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
            enabled
                && map.iter().any(|(key, value)| {
                    !matches!(
                        normalize_payment_key(key).as_str(),
                        "enabled" | "enable" | "open" | "localpod" | "uselocalpod"
                    ) && !value.is_null()
                })
        }
        _ => true,
    }
}

fn payment_value_applies_to_platform(
    map: &serde_json::Map<String, serde_json::Value>,
    platform: &str,
) -> bool {
    let Some(platforms) = map.get("__platform__") else {
        return false;
    };
    payment_platforms_contain(platforms, platform)
}

fn payment_platforms_contain(platforms: &serde_json::Value, platform: &str) -> bool {
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
        _ => false,
    }
}

fn normalize_payment_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}
