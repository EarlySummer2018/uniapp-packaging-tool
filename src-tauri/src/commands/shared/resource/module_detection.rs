use super::types::DetectedModule;

fn match_module_to_category(module_name: &str) -> DetectedModule {
    match module_name {
        "Push" => DetectedModule {
            name: "Push".to_string(),
            category: "push".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![
                "unipush_appid".into(),
                "unipush_appkey".into(),
                "unipush_appsecret".into(),
                "XIAOMI_APP_ID".into(),
                "XIAOMI_APP_KEY".into(),
                "MEIZU_APP_ID".into(),
                "MEIZU_APP_KEY".into(),
                "HUAWEI_APP_ID".into(),
                "OPPO_APP_KEY".into(),
                "OPPO_APP_SECRET".into(),
                "VIVO_APP_ID".into(),
                "VIVO_APP_KEY".into(),
                "HONOR_APP_ID".into(),
            ],
            source: String::new(),
        },
        "Geolocation" => DetectedModule {
            name: "Geolocation".to_string(),
            category: "geolocation".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["BAIDU_MAP_AK".into(), "AMAP_KEY".into()],
            source: String::new(),
        },
        "Share" => DetectedModule {
            name: "Share".to_string(),
            category: "share".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![
                "WX_APPID".into(),
                "WX_SECRET".into(),
                "QQ_APPID".into(),
                "SINA_APPKEY".into(),
                "SINA_SECRET".into(),
                "SINA_REDIRECT_URI".into(),
            ],
            source: String::new(),
        },
        "Login" | "OAuth" => DetectedModule {
            name: "Login".to_string(),
            category: "login".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["WX_APPID".into(), "QQ_APPID".into()],
            source: String::new(),
        },
        "Payment" => DetectedModule {
            name: "Payment".to_string(),
            category: "payment".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["WX_APPID".into(), "ALIPAY_PARTNER_ID".into()],
            source: String::new(),
        },
        "Map" | "Maps" => DetectedModule {
            name: "Map".to_string(),
            category: "map".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["AMAP_KEY".into(), "TENCENT_MAP_KEY".into()],
            source: String::new(),
        },
        "Speech" => DetectedModule {
            name: "Speech".to_string(),
            category: "speech".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![
                "IFLY_APPID".into(),
                "BD_SPEECH_APIKEY".into(),
                "ALI_SPEAK_ACCESSKEY_ID".into(),
            ],
            source: String::new(),
        },
        "Statistic" | "Statistics" => DetectedModule {
            name: "Statistic".to_string(),
            category: "statistic".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["UMENG_APPKEY".into(), "MTA_APPID".into()],
            source: String::new(),
        },
        "FaceRecognition" | "FaceRecognitionVerify" => DetectedModule {
            name: "FaceRecognition".to_string(),
            category: "face_recognition".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![
                "DCLOUD_LICENSE".into(),
                "BDFACE_APIKEY".into(),
                "ALIFACE_ACCESSKEY_ID".into(),
            ],
            source: String::new(),
        },
        "FaceID" | "FaceId" | "faceID" | "faceId" | "face_id" => DetectedModule {
            name: "FaceID".to_string(),
            category: "face_id".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec!["NSFaceIDUsageDescription".into()],
            source: String::new(),
        },
        "UniAD" | "uni-ad" => DetectedModule {
            name: "UniAD".to_string(),
            category: "uni_ad".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "X5Webview" | "X5TBS" | "Webview-x5" | "webview-x5" | "Android X5 Webview" => {
            DetectedModule {
                name: "X5Webview".to_string(),
                category: "x5_tbs".to_string(),
                platforms: vec![],
                configured: false,
                required_keys: vec![],
                source: String::new(),
            }
        }
        "UIWebview" | "UIWebView" | "uiWebview" | "uiWebView" | "ui_webview" => DetectedModule {
            name: "UIWebview".to_string(),
            category: "ui_webview".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "LivePusher" | "livepusher" | "livePusher" => DetectedModule {
            name: "LivePusher".to_string(),
            category: "livepusher".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "Camera" | "camera" => DetectedModule {
            name: "Camera".to_string(),
            category: "camera".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "VideoPlayer" | "videoplayer" | "videoPlayer" => DetectedModule {
            name: "VideoPlayer".to_string(),
            category: "video_player".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "Barcode" | "BarCode" | "barcode" | "barCode" => DetectedModule {
            name: "Barcode".to_string(),
            category: "barcode".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "Bluetooth" | "bluetooth" | "Blutooth" | "blutooth" => DetectedModule {
            name: "Bluetooth".to_string(),
            category: "bluetooth".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "iBeacon" | "IBeacon" | "ibeacon" => DetectedModule {
            name: "iBeacon".to_string(),
            category: "ibeacon".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "Contacts" | "Contact" | "contacts" | "contact" => DetectedModule {
            name: "Contacts".to_string(),
            category: "contacts".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "Fingerprint" | "fingerprint" => DetectedModule {
            name: "Fingerprint".to_string(),
            category: "fingerprint".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "Messaging" | "messaging" => DetectedModule {
            name: "Messaging".to_string(),
            category: "messaging".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "Record" | "record" => DetectedModule {
            name: "Record".to_string(),
            category: "record".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "SQLite" | "Sqlite" | "sqlite" => DetectedModule {
            name: "SQLite".to_string(),
            category: "sqlite".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        "GCanvas" | "gcanvas" => DetectedModule {
            name: "gcanvas".to_string(),
            category: "gcanvas".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
        _ => DetectedModule {
            name: module_name.to_string(),
            category: "manifest".to_string(),
            platforms: vec![],
            configured: false,
            required_keys: vec![],
            source: String::new(),
        },
    }
}

pub(super) fn check_module_configured_in_props(module_name: &str, props_content: &str) -> bool {
    match module_name {
        "Push" => props_content.contains(r#"feature name="Push""#),
        "Share" => props_content.contains(r#"feature name="Share""#),
        "Geolocation" => props_content.contains("Geolocation"),
        "Login" | "OAuth" => {
            props_content.contains(r#"feature name="Login""#)
                || props_content.contains(r#"feature name="OAuth""#)
        }
        "Payment" => props_content.contains(r#"feature name="Payment""#),
        "Map" | "Maps" => {
            props_content.contains("Mapp") || props_content.contains(r#"feature name="Map""#)
        }
        "X5Webview" | "X5TBS" | "Webview-x5" | "webview-x5" => {
            props_content.contains(r#"feature name="X5Webview""#)
        }
        "UIWebview" | "UIWebView" | "uiWebview" | "uiWebView" | "ui_webview" => {
            props_content.contains(r#"feature name="UIWebview""#)
                || props_content.contains(r#"feature name="UIWebView""#)
        }
        "LivePusher" | "livepusher" | "livePusher" => {
            props_content.contains(r#"feature name="LivePusher""#)
        }
        "Camera" | "camera" => props_content.contains(r#"feature name="Camera""#),
        "VideoPlayer" | "videoplayer" | "videoPlayer" => {
            props_content.contains(r#"feature name="VideoPlayer""#)
        }
        "Barcode" | "BarCode" | "barcode" | "barCode" => {
            props_content.contains(r#"feature name="Barcode""#)
        }
        "Bluetooth" | "bluetooth" | "Blutooth" | "blutooth" => {
            props_content.contains(r#"feature name="Bluetooth""#)
        }
        "iBeacon" | "IBeacon" | "ibeacon" => props_content.contains(r#"feature name="iBeacon""#),
        "Contacts" | "Contact" | "contacts" | "contact" => {
            props_content.contains(r#"feature name="Contacts""#)
        }
        "FaceID" | "FaceId" | "faceID" | "faceId" | "face_id" => {
            props_content.contains(r#"feature name="FaceID""#)
                || props_content.contains(r#"feature name="FaceId""#)
        }
        "Fingerprint" | "fingerprint" => props_content.contains(r#"feature name="Fingerprint""#),
        "Messaging" | "messaging" => props_content.contains(r#"feature name="Messaging""#),
        "Record" | "record" => props_content.contains(r#"feature name="Record""#),
        "SQLite" | "Sqlite" | "sqlite" => props_content.contains(r#"feature name="Sqlite""#),
        _ => false,
    }
}

pub(super) fn collect_modules_from_value(
    modules: &serde_json::Value,
    platform: &str,
    detected: &mut Vec<DetectedModule>,
) {
    if let Some(items) = modules.as_array() {
        for item in items {
            if let Some(name) = item
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| item.as_str())
            {
                push_detected_module(detected, name, platform);
            }
        }
        return;
    }

    if let Some(map) = modules.as_object() {
        for (name, value) in map {
            let enabled = value
                .as_bool()
                .or_else(|| {
                    value
                        .as_object()
                        .and_then(|obj| obj.get("enabled"))
                        .and_then(|v| v.as_bool())
                })
                .unwrap_or(true);
            if enabled {
                push_detected_module(detected, name, platform);
            }
        }
    }
}

pub(super) fn collect_modules_from_sdk_configs(
    sdk_configs: &serde_json::Value,
    platform: &str,
    detected: &mut Vec<DetectedModule>,
    enabled_modules: Option<&serde_json::Value>,
) {
    let Some(map) = sdk_configs.as_object() else {
        return;
    };

    for (key, value) in map {
        if !sdk_config_value_enabled(value) {
            continue;
        }

        if let Some(module_name) = sdk_config_key_to_module_name(key) {
            if module_name == "Payment" {
                if enabled_modules
                    .map(|modules| module_declared_enabled(modules, module_name))
                    .unwrap_or(false)
                    && payment_config_has_provider(value)
                {
                    push_detected_module(detected, module_name, platform);
                }
                continue;
            }
            if module_name == "Geolocation" {
                if enabled_modules
                    .map(|modules| module_declared_enabled(modules, module_name))
                    .unwrap_or(false)
                {
                    push_detected_module(detected, module_name, platform);
                }
                continue;
            }
            if enabled_modules
                .map(|modules| module_declared_enabled(modules, module_name))
                .unwrap_or(true)
            {
                push_detected_module(detected, module_name, platform);
            }
        }
    }
}

fn sdk_config_key_to_module_name(key: &str) -> Option<&'static str> {
    match key {
        "push" | "unipush" | "unipushV2" | "uniPush" => Some("Push"),
        "share" | "shares" => Some("Share"),
        "oauth" | "login" | "oauths" => Some("OAuth"),
        "payment" | "pay" | "payments" => Some("Payment"),
        "maps" | "map" => Some("Maps"),
        "geolocation" | "location" | "position" => Some("Geolocation"),
        "speech" | "speechRecognition" => Some("Speech"),
        "statistic" | "statistics" | "statics" => Some("Statistic"),
        "ad" | "ads" | "uni-ad" | "uniAD" | "uniad" => Some("UniAD"),
        "facialRecognitionVerify" | "faceRecognition" | "face_recognition" => {
            Some("FacialRecognitionVerify")
        }
        "faceID" | "faceId" | "FaceID" | "FaceId" | "face_id" => Some("FaceID"),
        "x5" | "x5Webview" | "x5_webview" | "webview-x5" | "Webview-x5" => Some("X5Webview"),
        "uiWebview" | "uiWebView" | "UIWebview" | "UIWebView" | "ui_webview" => Some("UIWebview"),
        "livepusher" | "livePusher" => Some("LivePusher"),
        "camera" => Some("Camera"),
        "videoPlayer" | "videoplayer" => Some("VideoPlayer"),
        "barcode" | "Barcode" | "barCode" | "BarCode" => Some("Barcode"),
        "bluetooth" | "blutooth" => Some("Bluetooth"),
        "iBeacon" | "ibeacon" => Some("iBeacon"),
        "contacts" | "contact" => Some("Contacts"),
        "fingerprint" => Some("Fingerprint"),
        "messaging" => Some("Messaging"),
        "record" => Some("Record"),
        "sqlite" | "SQLite" => Some("SQLite"),
        "gcanvas" | "GCanvas" => Some("gcanvas"),
        _ => None,
    }
}

fn module_declared_enabled(modules: &serde_json::Value, module_name: &str) -> bool {
    if let Some(items) = modules.as_array() {
        return items.iter().any(|item| {
            let Some(name) = item
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| item.as_str())
            else {
                return false;
            };
            module_names_equivalent(name, module_name) && sdk_config_value_enabled(item)
        });
    }

    if let Some(map) = modules.as_object() {
        return map.iter().any(|(name, value)| {
            module_names_equivalent(name, module_name) && sdk_config_value_enabled(value)
        });
    }

    false
}

fn module_names_equivalent(left: &str, right: &str) -> bool {
    let left_module = match_module_to_category(left);
    let right_module = match_module_to_category(right);
    if left_module.category != "manifest" || right_module.category != "manifest" {
        left_module.category == right_module.category
    } else {
        normalize_manifest_key(left) == normalize_manifest_key(right)
    }
}

fn normalize_manifest_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn sdk_config_value_enabled(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(flag) => *flag,
        serde_json::Value::Null => false,
        serde_json::Value::Object(map) => {
            let enabled = map
                .get("enabled")
                .or_else(|| map.get("enable"))
                .or_else(|| map.get("open"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            enabled && config_value_applies_to_platform(map, None)
        }
        _ => true,
    }
}

fn payment_config_has_provider(value: &serde_json::Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    map.iter().any(|(key, value)| {
        matches!(
            normalize_manifest_key(key).as_str(),
            "alipay"
                | "weixin"
                | "wechat"
                | "wx"
                | "paypal"
                | "stripe"
                | "google"
                | "googlepay"
                | "apple"
                | "applepay"
                | "iap"
                | "appleiap"
                | "inapp"
                | "inapppurchase"
        ) && sdk_config_value_enabled(value)
    })
}

fn config_value_applies_to_platform(
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
        _ => true,
    }
}

fn push_detected_module(detected: &mut Vec<DetectedModule>, raw_name: &str, platform: &str) {
    let mut module = match_module_to_category(raw_name);
    module.name = raw_name.to_string();
    module.source = "manifest.json".to_string();
    let platform = platform.to_string();
    if let Some(existing) = detected.iter_mut().find(|m| m.name == module.name) {
        if !existing.platforms.contains(&platform) {
            existing.platforms.push(platform);
        }
        return;
    }
    module.platforms.push(platform);
    detected.push(module);
}

/// 将 `app-harmony.distribute.modules` 中的 `uni-*` key 映射到受支持的模块分类。
///
/// 鸿蒙原生模块按官方文档以 `uni-*` 形式的 key 声明（例如 `uni-push`、`uni-oauth`）。
/// 仅返回本轮鸿蒙构建会处理的分类；`uni-share`、`uni-location` 等暂不在范围内返回 `None`。
/// 见 https://uniapp.dcloud.net.cn/collocation/manifest.html#app-harmony
pub(super) fn match_harmony_module_key_to_category(key: &str) -> Option<&'static str> {
    match key {
        "uni-push" => Some("push"),
        "uni-oauth" => Some("login"),
        "uni-payment" => Some("payment"),
        "uni-facialrecognitionverify" => Some("face_recognition"),
        "uni-map" => Some("map"),
        _ => None,
    }
}

/// 收集 `app-harmony.distribute.modules` 中声明的鸿蒙原生模块。
///
/// 与 `collect_modules_from_value` 不同，这里的 key 是 `uni-*` 形式，需要通过
/// `match_harmony_module_key_to_category` 映射到标准分类后构造 `DetectedModule`，
/// 并标记 `platforms = ["harmony"]`，确保不会泄漏进 Android/iOS 报表。
pub(super) fn collect_harmony_modules(
    modules: &serde_json::Value,
    detected: &mut Vec<DetectedModule>,
) {
    let Some(map) = modules.as_object() else {
        return;
    };
    for (key, value) in map {
        let Some(category) = match_harmony_module_key_to_category(key) else {
            continue;
        };
        if key == "uni-map" {
            let Some(config) = value.as_object() else {
                continue;
            };
            if config.is_empty() {
                continue;
            }
        }
        let enabled = value
            .as_bool()
            .or_else(|| {
                value
                    .as_object()
                    .and_then(|obj| obj.get("enabled"))
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(true);
        if !enabled {
            continue;
        }
        if detected
            .iter()
            .any(|m| m.category == category && m.platforms.iter().any(|p| p == "harmony"))
        {
            continue;
        }
        detected.push(DetectedModule {
            name: key.to_string(),
            category: category.to_string(),
            platforms: vec!["harmony".to_string()],
            configured: false,
            required_keys: Vec::new(),
            source: "app-harmony".to_string(),
        });
    }
}
