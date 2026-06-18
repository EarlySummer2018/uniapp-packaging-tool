use crate::commands::shared::module::push::manifest_push_unipush_v2_enabled;
use crate::commands::shared::module::templates::android_module_template_key;
use crate::commands::shared::module::types::AndroidConfigFieldSpec;

use super::payment::{payment_provider_enabled_for_platform, PaymentProvider};
use crate::commands::shared::module::parsing::normalize_config_key;

const MAVEN_CENTRAL_MIRROR_REPOSITORY: &str =
    "maven { url 'https://maven.aliyun.com/repository/public' }";
const PAYPAL_MAVEN_REPOSITORY: &str = r#"maven {
    url "https://cardinalcommerceprod.jfrog.io/artifactory/android"
    credentials {
        username 'paypal_sgerritz'
        password 'AKCp8jQ8tAahqpT5JjZ4FRP2mW7GMoFZ674kGqHmupTesKeAY2G8NcmPKLuTxTGkKjDLRzDUQ'
    }
}"#;

pub(super) fn android_geolocation_enabled(manifest: Option<&serde_json::Value>) -> bool {
    let Some(manifest) = manifest else {
        return true;
    };
    if !android_manifest_module_enabled(manifest, "Geolocation") {
        return false;
    }
    let Some(config) = android_geolocation_sdk_config(manifest) else {
        return false;
    };
    if !config_value_enabled(config) {
        return false;
    }
    [
        &["system"][..],
        &["baidu", "bd"][..],
        &["amap", "gaode"][..],
        &["tencent", "qqmap"][..],
    ]
    .iter()
    .any(|provider_keys| android_geolocation_provider_enabled(manifest, provider_keys))
}

fn android_manifest_module_enabled(manifest: &serde_json::Value, module_name: &str) -> bool {
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
            android_module_names_equivalent(name, module_name) && config_value_enabled(item)
        });
    }
    if let Some(map) = modules.as_object() {
        return map.iter().any(|(name, value)| {
            android_module_names_equivalent(name, module_name) && config_value_enabled(value)
        });
    }
    false
}

pub(super) fn android_module_names_equivalent(left: &str, right: &str) -> bool {
    android_module_template_key(left) == android_module_template_key(right)
        || normalize_config_key(left) == normalize_config_key(right)
}

fn android_geolocation_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["geolocation", "location", "position"]
        .iter()
        .find_map(|key| get_object_value_normalized(sdk_configs, key))
}

pub(super) fn android_geolocation_provider_enabled(
    manifest: &serde_json::Value,
    provider_keys: &[&str],
) -> bool {
    if !android_manifest_module_enabled(manifest, "Geolocation") {
        return false;
    }
    let Some(config) = android_geolocation_sdk_config(manifest) else {
        return false;
    };
    if !config_value_enabled(config) {
        return false;
    }
    let Some(map) = config.as_object() else {
        return false;
    };
    provider_keys.iter().any(|provider_key| {
        get_object_value_normalized(map, provider_key)
            .is_some_and(android_geolocation_provider_value_enabled)
    })
}

fn android_geolocation_provider_value_enabled(value: &serde_json::Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    let enabled = map
        .get("enabled")
        .or_else(|| map.get("enable"))
        .or_else(|| map.get("open"))
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    enabled && config_value_applies_to_android_strict(map)
}

fn config_value_applies_to_android_strict(
    map: &serde_json::Map<String, serde_json::Value>,
) -> bool {
    let Some(platforms) = map.get("__platform__") else {
        return false;
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
        _ => false,
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
            "login" => {
                vec!["maven { url 'https://mvn.getui.com/nexus/content/repositories/releases' }"]
            }
            "payment" => vec![MAVEN_CENTRAL_MIRROR_REPOSITORY, PAYPAL_MAVEN_REPOSITORY],
            _ => Vec::new(),
        };
    };

    match template_key {
        "login" => {
            if manifest_has_enabled_provider(
                manifest,
                &["oauth", "login", "oauths"],
                &["univerify", "igetui", "getui"],
            ) {
                vec!["maven { url 'https://mvn.getui.com/nexus/content/repositories/releases' }"]
            } else {
                Vec::new()
            }
        }
        "push" => {
            if !manifest_push_unipush_v2_enabled(manifest) {
                return Vec::new();
            }
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
        "payment" => {
            let mut repos = Vec::new();
            if payment_provider_enabled_for_platform(manifest, PaymentProvider::Paypal, "android")
                || payment_provider_enabled_for_platform(
                    manifest,
                    PaymentProvider::Stripe,
                    "android",
                )
                || payment_provider_enabled_for_platform(
                    manifest,
                    PaymentProvider::Google,
                    "android",
                )
            {
                repos.push(MAVEN_CENTRAL_MIRROR_REPOSITORY);
            }
            if payment_provider_enabled_for_platform(manifest, PaymentProvider::Paypal, "android") {
                repos.push(PAYPAL_MAVEN_REPOSITORY);
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
        return template_key != "push";
    };
    if template_key == "push" && !manifest_push_unipush_v2_enabled(manifest) {
        return false;
    }
    let note = android_entry_provider_note(entry);
    if template_key == "geolocation" && !android_geolocation_enabled(Some(manifest)) {
        return false;
    }
    if template_key == "payment" {
        return payment_entry_enabled_for_android(&note, manifest);
    }
    if template_key == "geolocation"
        && android_entry_mentions_any(&note, &["amap", "gaode", "高德"])
        && android_amap_map_enabled(Some(manifest))
    {
        return false;
    }
    if template_key == "statistic"
        && android_entry_mentions_any(&note, &["google play", "googleplay", "umeng gp"])
    {
        return manifest_has_enabled_provider(
            manifest,
            &["statistic", "statistics", "statics"],
            &[
                "umeng-gp",
                "umeng_gp",
                "umenggp",
                "umeng-google-play",
                "umengGooglePlay",
            ],
        );
    }

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
        "geolocation" => geolocation_provider_entry_enabled(&note, manifest),
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

fn payment_entry_enabled_for_android(note: &str, manifest: &serde_json::Value) -> bool {
    if android_entry_mentions_any(note, &["apple", "iap", "in app purchase", "apple pay"]) {
        return false;
    }
    for (markers, provider) in [
        (&["alipay", "支付宝"][..], PaymentProvider::Alipay),
        (
            &["weixin", "wechat", "wx", "微信"][..],
            PaymentProvider::Weixin,
        ),
        (&["paypal"][..], PaymentProvider::Paypal),
        (&["stripe"][..], PaymentProvider::Stripe),
        (&["google"][..], PaymentProvider::Google),
    ] {
        if android_entry_mentions_any(note, markers) {
            return payment_provider_enabled_for_platform(manifest, provider, "android");
        }
    }
    false
}

pub fn android_amap_map_enabled(manifest: Option<&serde_json::Value>) -> bool {
    manifest
        .map(|manifest| {
            manifest_has_enabled_provider(manifest, &["maps", "map"], &["amap", "gaode"])
        })
        .unwrap_or(false)
}

pub fn android_amap_geolocation_enabled(manifest: Option<&serde_json::Value>) -> bool {
    manifest
        .map(|manifest| android_geolocation_provider_enabled(manifest, &["amap", "gaode"]))
        .unwrap_or(false)
}

fn geolocation_provider_entry_enabled(note: &str, manifest: &serde_json::Value) -> bool {
    if android_entry_mentions_any(note, &["baidu", "bd", "百度"]) {
        return android_geolocation_provider_enabled(manifest, &["baidu", "bd"]);
    }
    if android_entry_mentions_any(note, &["amap", "gaode", "高德"]) {
        return android_geolocation_provider_enabled(manifest, &["amap", "gaode"]);
    }
    if android_entry_mentions_any(note, &["tencent", "qqmap", "腾讯"]) {
        return android_geolocation_provider_enabled(manifest, &["tencent", "qqmap"]);
    }
    true
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
    if !manifest_push_unipush_v2_enabled(manifest) {
        return false;
    }
    manifest_has_enabled_provider(
        manifest,
        &["push", "unipush", "unipushV2", "uniPush"],
        provider_keys,
    )
}

pub(super) fn find_manifest_config_value(
    manifest: &serde_json::Value,
    spec: &AndroidConfigFieldSpec,
) -> Option<String> {
    let mut candidates = Vec::new();
    collect_manifest_config_candidates(manifest, spec, &mut Vec::new(), &mut candidates);
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.into_iter().map(|(_, value)| value).next()
}

pub(super) fn manifest_has_enabled_provider(
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

pub(super) fn manifest_has_any_enabled_module(
    manifest: &serde_json::Value,
    module_keys: &[&str],
) -> bool {
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
