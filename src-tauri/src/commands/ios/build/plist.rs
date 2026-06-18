use std::collections::HashMap;
use std::path::Path;

use super::config::{effective_app_name, effective_app_version, effective_app_version_code};
use super::fs_utils::{collect_files_named_skipping_bundles, find_file_named, find_info_plist};
use crate::commands::ios::modules::barcode::{
    apply_ios_barcode_privacy_defaults as apply_barcode_privacy_defaults, ios_barcode_enabled,
};
use crate::commands::ios::modules::bluetooth::{
    apply_ios_bluetooth_background_modes as apply_bluetooth_background_modes,
    apply_ios_bluetooth_privacy_defaults as apply_bluetooth_privacy_defaults,
    ios_bluetooth_background_enabled, ios_bluetooth_enabled,
};
use crate::commands::ios::modules::camera::{
    apply_ios_camera_privacy_defaults as apply_camera_privacy_defaults, ios_camera_enabled,
};
use crate::commands::ios::modules::common::ios_manifest_module_enabled;
use crate::commands::ios::modules::contacts::{
    apply_ios_contacts_privacy_defaults as apply_contacts_privacy_defaults, ios_contacts_enabled,
};
use crate::commands::ios::modules::face_id::{
    apply_ios_face_id_privacy_defaults as apply_face_id_privacy_defaults, ios_face_id_enabled,
};
use crate::commands::ios::modules::facial_recognition_verify::{
    apply_ios_facial_recognition_verify_privacy_defaults as apply_facial_recognition_verify_privacy_defaults,
    ios_facial_recognition_verify_enabled,
};
use crate::commands::ios::modules::fingerprint::{
    apply_ios_fingerprint_privacy_defaults as apply_fingerprint_privacy_defaults,
    ios_fingerprint_enabled,
};
use crate::commands::ios::modules::geolocation::{
    apply_ios_geolocation_privacy_defaults as apply_geolocation_privacy_defaults,
    ios_geolocation_provider_value_enabled, ios_geolocation_providers,
};
use crate::commands::ios::modules::ibeacon::{
    apply_ios_ibeacon_background_modes as apply_ibeacon_background_modes,
    apply_ios_ibeacon_privacy_defaults as apply_ibeacon_privacy_defaults, ios_ibeacon_enabled,
};
use crate::commands::ios::modules::livepusher::{
    apply_ios_livepusher_privacy_defaults as apply_livepusher_privacy_defaults,
    ios_livepusher_enabled,
};
use crate::commands::ios::modules::map::{
    apply_ios_map_privacy_defaults as apply_map_privacy_defaults, ios_map_enabled,
};
use crate::commands::ios::modules::payment::ios_payment_provider_value;
use crate::commands::ios::modules::push::apply_ios_push_plist_defaults as apply_push_plist_defaults;
use crate::commands::ios::modules::record::{
    apply_ios_record_privacy_defaults as apply_record_privacy_defaults, ios_record_enabled,
};
use crate::commands::ios::modules::speech::{
    apply_ios_speech_privacy_defaults as apply_speech_privacy_defaults, ios_speech_enabled,
};
use crate::commands::module::PaymentProvider;

pub(super) fn patch_info_plist(
    project_root: &Path,
    project_file: &Path,
    config: &crate::commands::project::ProjectConfig,
    app_id: &str,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let plist_path = find_info_plist(project_root, project_file)
        .ok_or_else(|| "未找到主工程 Info.plist".to_string())?;
    let mut value =
        plist::Value::from_file(&plist_path).map_err(|e| format!("解析 Info.plist 失败: {}", e))?;
    let dict = value
        .as_dictionary_mut()
        .ok_or_else(|| "Info.plist 不是 dictionary".to_string())?;
    dict.insert(
        "dcloud_appkey".into(),
        plist::Value::String(config.ios.dcloud_app_key.clone()),
    );
    dict.insert(
        "CFBundleDisplayName".into(),
        plist::Value::String(effective_app_name(config, manifest_info)),
    );
    dict.insert(
        "CFBundleShortVersionString".into(),
        plist::Value::String(effective_app_version(config, manifest_info)),
    );
    dict.insert(
        "CFBundleVersion".into(),
        plist::Value::String(effective_app_version_code(config, manifest_info).to_string()),
    );
    dict.insert(
        "marketChannel".into(),
        plist::Value::String(format!("{}|{}||apple", config.ios.bundle_id, app_id)),
    );
    if find_file_named(project_root, "LaunchScreen.storyboard").is_some() {
        dict.insert(
            "UILaunchStoryboardName".into(),
            plist::Value::String("LaunchScreen".into()),
        );
    }
    set_dcloud_default_theme(dict);
    if let Some(info) = manifest_info {
        apply_ios_privacy_descriptions(dict, &info.ios_privacy_descriptions);
        if let Some(manifest) = info.manifest_value.as_ref() {
            apply_ios_manifest_plist(dict, manifest);
        }
        if ios_geolocation_providers(Some(info)).is_some() {
            apply_geolocation_privacy_defaults(dict);
        }
        if ios_map_enabled(Some(info)) {
            apply_map_privacy_defaults(dict);
        }
        if ios_barcode_enabled(Some(info)) {
            apply_barcode_privacy_defaults(dict);
        }
        if ios_camera_enabled(Some(info)) {
            apply_camera_privacy_defaults(dict);
        }
        if ios_bluetooth_enabled(Some(info)) {
            apply_bluetooth_privacy_defaults(dict);
        }
        if ios_bluetooth_background_enabled(Some(info)) {
            apply_bluetooth_background_modes(dict);
        }
        if ios_contacts_enabled(Some(info)) {
            apply_contacts_privacy_defaults(dict);
        }
        if ios_face_id_enabled(Some(info)) {
            apply_face_id_privacy_defaults(dict);
        }
        if ios_fingerprint_enabled(Some(info)) {
            apply_fingerprint_privacy_defaults(dict);
        }
        if ios_ibeacon_enabled(Some(info)) {
            apply_ibeacon_privacy_defaults(dict);
            apply_ibeacon_background_modes(dict);
        }
        if ios_livepusher_enabled(Some(info)) {
            apply_livepusher_privacy_defaults(dict);
        }
        if ios_facial_recognition_verify_enabled(Some(info)) {
            apply_facial_recognition_verify_privacy_defaults(dict);
        }
        apply_push_plist_defaults(dict, Some(info));
        if ios_record_enabled(Some(info)) {
            apply_record_privacy_defaults(dict);
        }
        if ios_speech_enabled(Some(info)) {
            apply_speech_privacy_defaults(dict);
        }
        apply_ios_module_config_privacy_descriptions(dict, Some(info), &config.ios_module_config);
    } else {
        cleanup_ios_privacy_descriptions(dict);
    }
    patch_info_plist_strings(project_root, &effective_app_name(config, manifest_info))?;
    value
        .to_file_xml(&plist_path)
        .map_err(|e| format!("写入 Info.plist 失败: {}", e))
}

pub(super) fn apply_ios_privacy_descriptions(
    dict: &mut plist::Dictionary,
    descriptions: &std::collections::BTreeMap<String, String>,
) {
    cleanup_ios_privacy_descriptions(dict);
    for (key, description) in descriptions {
        let description = description.trim();
        if is_supported_ios_privacy_description_key(key) && !description.is_empty() {
            dict.insert(key.clone(), plist::Value::String(description.to_string()));
        }
    }
    cleanup_ios_privacy_descriptions(dict);
}

pub(super) fn apply_ios_module_config_privacy_descriptions(
    dict: &mut plist::Dictionary,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    module_config: &HashMap<String, String>,
) {
    let mut entries = module_config.iter().collect::<Vec<_>>();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (config_key, description) in entries {
        let Some(plist_key) = config_key.strip_prefix("privacy.") else {
            continue;
        };
        if !is_supported_ios_privacy_description_key(plist_key) {
            continue;
        }
        let description = description.trim();
        if description.is_empty() {
            continue;
        }
        if !ios_privacy_key_applies_to_manifest(plist_key, manifest_info) {
            continue;
        }
        dict.insert(
            plist_key.to_string(),
            plist::Value::String(description.to_string()),
        );
    }
    cleanup_ios_privacy_descriptions(dict);
}

fn ios_privacy_key_applies_to_manifest(
    plist_key: &str,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    let Some(info) = manifest_info else {
        return false;
    };
    match plist_key {
        "NSCameraUsageDescription" => {
            ios_camera_enabled(Some(info))
                || ios_barcode_enabled(Some(info))
                || ios_livepusher_enabled(Some(info))
                || ios_facial_recognition_verify_enabled(Some(info))
        }
        "NSMicrophoneUsageDescription" => {
            ios_livepusher_enabled(Some(info))
                || ios_record_enabled(Some(info))
                || ios_speech_enabled(Some(info))
        }
        "NSSpeechRecognitionUsageDescription" => ios_speech_enabled(Some(info)),
        "NSPhotoLibraryUsageDescription" => {
            ios_camera_enabled(Some(info)) || ios_barcode_enabled(Some(info))
        }
        "NSPhotoLibraryAddUsageDescription" => ios_camera_enabled(Some(info)),
        "NSLocationUsageDescription"
        | "NSLocationWhenInUseUsageDescription"
        | "NSLocationAlwaysUsageDescription" => {
            ios_geolocation_providers(Some(info)).is_some() || ios_map_enabled(Some(info))
        }
        "NSLocationAlwaysAndWhenInUseUsageDescription" => {
            ios_geolocation_providers(Some(info)).is_some()
                || ios_map_enabled(Some(info))
                || ios_ibeacon_enabled(Some(info))
        }
        "NSContactsUsageDescription" => ios_contacts_enabled(Some(info)),
        "NSBluetoothAlwaysUsageDescription" | "NSBluetoothPeripheralUsageDescription" => {
            ios_bluetooth_enabled(Some(info))
        }
        "NSFaceIDUsageDescription" => {
            ios_face_id_enabled(Some(info)) || ios_fingerprint_enabled(Some(info))
        }
        _ => true,
    }
}

fn cleanup_ios_privacy_descriptions(dict: &mut plist::Dictionary) {
    promote_duplicate_ios_privacy_descriptions(dict);
    let keys = dict.keys().cloned().collect::<Vec<_>>();
    for key in keys {
        if is_duplicate_ios_privacy_key(&key) || is_empty_legacy_ios_privacy_value(dict, &key) {
            dict.remove(&key);
        }
    }
}

fn promote_duplicate_ios_privacy_descriptions(dict: &mut plist::Dictionary) {
    let keys = dict.keys().cloned().collect::<Vec<_>>();
    for key in keys {
        let Some(base) = duplicate_ios_privacy_base_key(&key) else {
            continue;
        };
        let Some(value) = dict
            .get(&key)
            .and_then(plist::Value::as_string)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        let base_is_empty = dict
            .get(&base)
            .and_then(plist::Value::as_string)
            .map(str::trim)
            .is_none_or(str::is_empty);
        if base_is_empty {
            dict.insert(base, plist::Value::String(value));
        }
    }
}

fn is_duplicate_ios_privacy_key(key: &str) -> bool {
    duplicate_ios_privacy_base_key(key).is_some()
}

fn duplicate_ios_privacy_base_key(key: &str) -> Option<String> {
    let (base, suffix) = key.rsplit_once(" - ")?;
    if suffix.chars().all(|ch| ch.is_ascii_digit())
        && (is_ios_privacy_description_key(base) || is_legacy_ios_privacy_description_key(base))
    {
        Some(base.to_string())
    } else {
        None
    }
}

fn is_empty_legacy_ios_privacy_value(dict: &plist::Dictionary, key: &str) -> bool {
    if !is_legacy_ios_privacy_description_key(key) {
        return false;
    }
    dict.get(key)
        .and_then(plist::Value::as_string)
        .map(str::trim)
        .is_some_and(str::is_empty)
}

fn is_ios_privacy_description_key(key: &str) -> bool {
    is_supported_ios_privacy_description_key(key)
}

fn is_legacy_ios_privacy_description_key(key: &str) -> bool {
    key == "NSLocationWhenInUseDescription"
}

fn is_supported_ios_privacy_description_key(key: &str) -> bool {
    IOS_PRIVACY_DESCRIPTION_KEYS.contains(&key)
}

const IOS_PRIVACY_DESCRIPTION_KEYS: &[&str] = &[
    "NSPhotoLibraryUsageDescription",
    "NSPhotoLibraryAddUsageDescription",
    "NSCameraUsageDescription",
    "NSMicrophoneUsageDescription",
    "NSLocationUsageDescription",
    "NSLocationWhenInUseUsageDescription",
    "NSLocationAlwaysUsageDescription",
    "NSLocationAlwaysAndWhenInUseUsageDescription",
    "NSCalendarsUsageDescription",
    "NSContactsUsageDescription",
    "NSBluetoothPeripheralUsageDescription",
    "NSBluetoothAlwaysUsageDescription",
    "NSSpeechRecognitionUsageDescription",
    "NSRemindersUsageDescription",
    "NSMotionUsageDescription",
    "NSHealthUpdateUsageDescription",
    "NSHealthShareUsageDescription",
    "NSAppleMusicUsageDescription",
    "NFCReaderUsageDescription",
    "NSHealthClinicalHealthRecordsShareUsageDescription",
    "NSHomeKitUsageDescription",
    "NSSiriUsageDescription",
    "NSFaceIDUsageDescription",
    "NSLocalNetworkUsageDescription",
    "NSUserTrackingUsageDescription",
];

pub(super) fn patch_info_plist_strings(project_root: &Path, app_name: &str) -> Result<(), String> {
    let mut files = Vec::new();
    collect_files_named_skipping_bundles(project_root, "InfoPlist.strings", &mut files);
    for path in files {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取 InfoPlist.strings 失败 {}: {}", path.display(), e))?;
        let updated = set_info_plist_string_value(&content, "CFBundleDisplayName", app_name);
        std::fs::write(&path, updated)
            .map_err(|e| format!("写入 InfoPlist.strings 失败 {}: {}", path.display(), e))?;
    }
    Ok(())
}

pub(super) fn set_info_plist_string_value(content: &str, key: &str, value: &str) -> String {
    let pattern = regex::Regex::new(&format!(
        r#"(?m)^(\s*(?:"{}"|{})\s*=\s*)"(?:\\.|[^"\\])*"(\s*;)"#,
        regex::escape(key),
        regex::escape(key)
    ))
    .expect("valid InfoPlist.strings regex");
    let escaped = escape_info_plist_strings_value(value);
    if pattern.is_match(content) {
        return pattern
            .replace_all(content, |caps: &regex::Captures| {
                format!(
                    "{}\"{}\"{}",
                    caps.get(1).map_or("", |value| value.as_str()),
                    escaped,
                    caps.get(2).map_or("", |value| value.as_str())
                )
            })
            .into_owned();
    }

    let mut updated = content.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&format!("\"{}\" = \"{}\";\n", key, escaped));
    updated
}

fn escape_info_plist_strings_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn apply_ios_manifest_plist(dict: &mut plist::Dictionary, manifest: &serde_json::Value) {
    let schemes = ios_manifest_url_schemes(manifest);
    if !schemes.is_empty() {
        dict.insert(
            "CFBundleURLTypes".into(),
            plist::Value::Array(
                schemes
                    .iter()
                    .map(|scheme| {
                        let mut entry = plist::Dictionary::new();
                        entry.insert(
                            "CFBundleURLName".into(),
                            plist::Value::String(format!("unipack.{}", scheme)),
                        );
                        entry.insert(
                            "CFBundleURLSchemes".into(),
                            plist::Value::Array(vec![plist::Value::String(scheme.clone())]),
                        );
                        plist::Value::Dictionary(entry)
                    })
                    .collect(),
            ),
        );
    }

    let query_schemes = ios_manifest_query_schemes(manifest);
    if !query_schemes.is_empty() {
        merge_plist_string_array(dict, "LSApplicationQueriesSchemes", query_schemes);
    }

    let background_modes = ios_manifest_background_modes(manifest);
    if !background_modes.is_empty() {
        merge_plist_string_array(dict, "UIBackgroundModes", background_modes);
    }
    apply_ios_manifest_transport_security(dict, manifest);

    if let Some(appid) = provider_value(manifest, "weixin", &["appid"]) {
        set_plist_dictionary_values(dict, "weixin", &[("appid", appid)]);
    }
    if let Some(link) = universal_links(manifest).into_iter().next() {
        dict.insert("UniversalLinks".into(), plist::Value::String(link));
    }
    let sina_appkey = provider_value(manifest, "sina", &["appkey"]);
    let sina_redirect = provider_value(manifest, "sina", &["redirect_uri", "redirectURI"]);
    if sina_appkey.is_some() || sina_redirect.is_some() {
        let mut values = Vec::new();
        if let Some(value) = sina_appkey {
            values.push(("appkey", value));
        }
        if let Some(value) = sina_redirect {
            values.push(("redirectURI", value));
        }
        set_plist_dictionary_values(dict, "sinaweibo", &values);
    }
    if let Some(value) = provider_value(manifest, "google", &["clientid", "clientId"]) {
        dict.insert("GIDClientID".into(), plist::Value::String(value));
    }
    if let Some(value) = provider_value(manifest, "facebook", &["appid"]) {
        dict.insert("FacebookAppID".into(), plist::Value::String(value));
    }
    if let Some(value) = provider_value(manifest, "facebook", &["client_token", "clientToken"]) {
        dict.insert("FacebookClientToken".into(), plist::Value::String(value));
    }
    if let Some(value) = ios_location_or_map_provider_value(
        manifest,
        "amap",
        &["appkey_ios", "apikey_ios", "appkey", "apikey", "key"],
    ) {
        set_plist_dictionary_values(dict, "amap", &[("appkey", value.clone())]);
        dict.insert("AMapApiKey".into(), plist::Value::String(value));
    }
    if let Some(value) = ios_location_or_map_provider_value(
        manifest,
        "baidu",
        &["appkey_ios", "apikey_ios", "appkey", "apikey", "ak", "key"],
    ) {
        set_plist_dictionary_values(dict, "baidu", &[("appkey", value.clone())]);
        dict.insert("BaiduMapApiKey".into(), plist::Value::String(value));
    }
    if let Some(speech) = manifest_provider(manifest, "baidu", Some("speech")) {
        let app_id = json_string_field(speech, &["appid"]);
        let api_key = json_string_field(speech, &["apikey", "apiKey"]);
        let secret_key = json_string_field(speech, &["secretkey", "secretKey"]);
        let mut values = Vec::new();
        if let Some(value) = app_id.clone() {
            values.push(("APP_ID", value.clone()));
            dict.insert("BDSpeechAPPID".into(), plist::Value::String(value));
        }
        if let Some(value) = api_key.clone() {
            values.push(("API_KEY", value.clone()));
            dict.insert("BDSpeechAPIKey".into(), plist::Value::String(value));
        }
        if let Some(value) = secret_key.clone() {
            values.push(("SECRET_KEY", value.clone()));
            dict.insert("BDSpeechSecretKey".into(), plist::Value::String(value));
        }
        set_plist_dictionary_values(dict, "baiduspeech", &values);
    }
    if let Some(umeng) = manifest_provider(manifest, "umeng", Some("statics")) {
        if let Some(appkey) = json_string_field(umeng, &["appkey_ios", "appkey"]) {
            set_plist_dictionary_values(dict, "umeng", &[("appkey", appkey.clone())]);
            dict.insert("UMENG_APPKEY".into(), plist::Value::String(appkey));
        }
        if let Some(channel) = json_string_field(umeng, &["channelid_ios", "channelid"]) {
            dict.insert("UMENG_CHANNEL".into(), plist::Value::String(channel));
        }
    }
}

fn apply_ios_manifest_transport_security(
    dict: &mut plist::Dictionary,
    manifest: &serde_json::Value,
) {
    let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("NSAppTransportSecurity"))
    else {
        return;
    };

    if let Some(value) = plist_transport_security_value(value) {
        dict.insert("NSAppTransportSecurity".into(), value);
    }
}

fn plist_transport_security_value(value: &serde_json::Value) -> Option<plist::Value> {
    match value {
        serde_json::Value::Bool(flag) => {
            let mut dict = plist::Dictionary::new();
            dict.insert(
                "NSAllowsArbitraryLoads".into(),
                plist::Value::Boolean(*flag),
            );
            Some(plist::Value::Dictionary(dict))
        }
        serde_json::Value::Object(map) => {
            let mut dict = plist::Dictionary::new();
            for key in [
                "NSAllowsArbitraryLoads",
                "NSAllowsArbitraryLoadsInWebContent",
                "NSAllowsLocalNetworking",
            ] {
                if let Some(flag) = map.get(key).and_then(serde_json::Value::as_bool) {
                    dict.insert(key.to_string(), plist::Value::Boolean(flag));
                }
            }
            Some(plist::Value::Dictionary(dict))
        }
        _ => None,
    }
}

fn set_plist_dictionary_values(dict: &mut plist::Dictionary, key: &str, values: &[(&str, String)]) {
    if values.is_empty() {
        return;
    }
    let mut nested = match dict.remove(key) {
        Some(plist::Value::Dictionary(value)) => value,
        _ => plist::Dictionary::new(),
    };
    for (name, value) in values {
        nested.insert((*name).to_string(), plist::Value::String(value.clone()));
    }
    dict.insert(key.to_string(), plist::Value::Dictionary(nested));
}

fn ios_location_or_map_provider_value(
    manifest: &serde_json::Value,
    provider: &str,
    keys: &[&str],
) -> Option<String> {
    ios_geolocation_provider_value(manifest, provider, keys)
        .or_else(|| provider_value_from_category(manifest, provider, "maps", keys))
}

pub(super) fn ios_geolocation_provider_value(
    manifest: &serde_json::Value,
    provider: &str,
    keys: &[&str],
) -> Option<String> {
    if !ios_manifest_module_enabled(manifest, "Geolocation") {
        return None;
    }
    let value = manifest_provider(manifest, provider, Some("geolocation"))?;
    if !ios_geolocation_provider_value_enabled(value) {
        return None;
    }
    json_string_field(value, keys)
}

fn provider_value_from_category(
    manifest: &serde_json::Value,
    provider: &str,
    category: &str,
    keys: &[&str],
) -> Option<String> {
    manifest_provider(manifest, provider, Some(category))
        .and_then(|value| json_string_field(value, keys))
}

fn merge_plist_string_array(dict: &mut plist::Dictionary, key: &str, values: Vec<String>) {
    let mut merged = Vec::new();
    if let Some(existing) = dict.get(key) {
        collect_plist_strings(existing, &mut merged);
    }
    merged.extend(values);
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

fn ios_manifest_url_schemes(manifest: &serde_json::Value) -> Vec<String> {
    let mut schemes = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("urltypes"))
    {
        collect_json_strings(value, &mut schemes);
    }
    if let Some(value) = provider_value(manifest, "weixin", &["appid"]) {
        schemes.push(value);
    }
    if let Some(value) = ios_payment_provider_value(manifest, PaymentProvider::Alipay)
        .and_then(|value| json_string_field(value, &["appId", "appid", "app_id"]))
    {
        schemes.push(prefixed_scheme("ap", &value));
    }
    if let Some(value) = ios_payment_provider_value(manifest, PaymentProvider::Weixin)
        .and_then(|value| json_string_field(value, &["appid", "appId"]))
    {
        schemes.push(value);
    }
    if let Some(value) = ios_payment_provider_value(manifest, PaymentProvider::Paypal)
        .and_then(|value| json_string_field(value, &["returnUrl", "returnURL", "scheme"]))
    {
        schemes.push(url_scheme_value(&value));
    }
    if let Some(value) = ios_payment_provider_value(manifest, PaymentProvider::Stripe)
        .and_then(|value| json_string_field(value, &["returnUrl", "returnURL", "scheme"]))
    {
        schemes.push(url_scheme_value(&value));
    }
    if let Some(value) = provider_value(manifest, "qq", &["appid"]) {
        schemes.push(prefixed_scheme("tencent", &value));
    }
    if let Some(value) = provider_value(manifest, "sina", &["appkey"]) {
        schemes.push(prefixed_scheme("wb", &value));
    }
    if let Some(value) = provider_value(manifest, "facebook", &["appid"]) {
        schemes.push(prefixed_scheme("fb", &value));
    }
    dedup_non_empty_strings(schemes)
}

fn ios_manifest_query_schemes(manifest: &serde_json::Value) -> Vec<String> {
    let mut schemes = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("urlschemewhitelist"))
    {
        collect_json_strings(value, &mut schemes);
    }
    if provider_value(manifest, "weixin", &["appid"]).is_some() {
        schemes.extend(["weixin".into(), "weixinULAPI".into()]);
    }
    if ios_payment_provider_value(manifest, PaymentProvider::Alipay).is_some() {
        schemes.extend(["alipay".into(), "alipays".into()]);
    }
    if ios_payment_provider_value(manifest, PaymentProvider::Weixin).is_some() {
        schemes.extend(
            ["weixin", "weixinULAPI", "weixinuniversallink"]
                .into_iter()
                .map(String::from),
        );
    }
    if ios_payment_provider_value(manifest, PaymentProvider::Paypal).is_some() {
        schemes.extend(["paypal", "paypalsandbox"].into_iter().map(String::from));
    }
    if provider_value(manifest, "qq", &["appid"]).is_some() {
        schemes.extend(
            [
                "mqq",
                "mqqapi",
                "mqqOpensdkSSoLogin",
                "mqqopensdkapiV2",
                "mqqopensdkapiV3",
                "mqqwpa",
                "mqzone",
            ]
            .into_iter()
            .map(String::from),
        );
    }
    if provider_value(manifest, "sina", &["appkey"]).is_some() {
        schemes.extend(
            ["sinaweibo", "sinaweibohd", "weibosdk", "weibosdk2.5"]
                .into_iter()
                .map(String::from),
        );
    }
    if provider_value(manifest, "facebook", &["appid"]).is_some() {
        schemes.extend(
            ["fb", "fbapi", "fb-messenger-share-api", "fbauth2"]
                .into_iter()
                .map(String::from),
        );
    }
    dedup_non_empty_strings(schemes)
}

fn ios_manifest_background_modes(manifest: &serde_json::Value) -> Vec<String> {
    let mut modes = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("UIBackgroundModes"))
    {
        collect_json_strings(value, &mut modes);
    }
    dedup_non_empty_strings(modes)
}

fn provider_value(manifest: &serde_json::Value, provider: &str, keys: &[&str]) -> Option<String> {
    [
        "oauth",
        "share",
        "payment",
        "geolocation",
        "maps",
        "speech",
        "statics",
    ]
    .into_iter()
    .find_map(|category| {
        manifest_provider(manifest, provider, Some(category))
            .and_then(|value| json_string_field(value, keys))
    })
}

fn manifest_provider<'a>(
    manifest: &'a serde_json::Value,
    provider: &str,
    category: Option<&str>,
) -> Option<&'a serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?;
    match category {
        Some(category) => find_manifest_provider_in_category(sdk_configs, category, provider),
        None => find_object_value_normalized(sdk_configs, provider),
    }
}

fn find_manifest_provider_in_category<'a>(
    sdk_configs: &'a serde_json::Value,
    category: &str,
    provider: &str,
) -> Option<&'a serde_json::Value> {
    manifest_category_aliases(category)
        .iter()
        .find_map(|category| {
            find_object_value_normalized(sdk_configs, category)
                .and_then(|category_value| find_object_value_normalized(category_value, provider))
        })
}

fn manifest_category_aliases(category: &str) -> Vec<&str> {
    match category {
        "maps" | "map" => vec!["maps", "map"],
        "speech" | "speechRecognition" => vec!["speech", "speechRecognition"],
        "statics" | "statistic" | "statistics" => vec!["statistic", "statistics", "statics"],
        "oauth" | "login" | "oauths" => vec!["oauth", "login", "oauths"],
        "share" | "shares" => vec!["share", "shares"],
        _ => vec![category],
    }
}

fn find_object_value_normalized<'a>(
    value: &'a serde_json::Value,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let map = value.as_object()?;
    let normalized_key = normalize_manifest_key(key);
    map.iter()
        .find(|(candidate, _)| normalize_manifest_key(candidate) == normalized_key)
        .map(|(_, value)| value)
}

fn normalize_manifest_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn json_string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(String::from)
    })
}

pub(super) fn collect_json_strings(value: &serde_json::Value, output: &mut Vec<String>) {
    match value {
        serde_json::Value::String(value) => {
            output.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(String::from),
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

fn prefixed_scheme(prefix: &str, value: &str) -> String {
    if value.starts_with(prefix) {
        value.to_string()
    } else {
        format!("{}{}", prefix, value)
    }
}

fn url_scheme_value(value: &str) -> String {
    value
        .split_once("://")
        .map(|(scheme, _)| scheme)
        .unwrap_or(value)
        .trim_matches('/')
        .to_string()
}

pub(super) fn dedup_non_empty_strings(values: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() && !result.iter().any(|existing| existing == value) {
            result.push(value.to_string());
        }
    }
    result
}

pub(super) fn universal_links(manifest: &serde_json::Value) -> Vec<String> {
    let mut links = Vec::new();
    collect_values_for_key(manifest, "UniversalLinks", &mut links);
    dedup_non_empty_strings(links)
}

fn collect_values_for_key(value: &serde_json::Value, key: &str, output: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(values) => {
            for (name, value) in values {
                if name.eq_ignore_ascii_case(key) {
                    collect_json_strings(value, output);
                } else {
                    collect_values_for_key(value, key, output);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_values_for_key(value, key, output);
            }
        }
        _ => {}
    }
}

fn set_dcloud_default_theme(dict: &mut plist::Dictionary) {
    let existing = dict.remove("DCloudConfig");
    let mut dcloud = match existing {
        Some(plist::Value::Dictionary(value)) => value,
        _ => plist::Dictionary::new(),
    };
    dcloud.insert("defaultTheme".into(), plist::Value::String("auto".into()));
    dict.insert("DCloudConfig".into(), plist::Value::Dictionary(dcloud));
}
