use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::manifest::{bool_field, string_field};
use super::types::{AndroidIconsConfig, IosIconsConfig, PushIconsConfig, SplashscreenConfig};

pub(super) fn find_manifest_android_icons(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<AndroidIconsConfig> {
    let icons_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("icons"))?;

    let android = icons_value
        .get("android")
        .and_then(|v| v.as_object())
        .map(|items| {
            items
                .iter()
                .filter_map(|(density, path)| {
                    path.as_str()
                        .map(str::trim)
                        .filter(|p| !p.is_empty())
                        .map(|p| {
                            (
                                density.to_string(),
                                resolve_manifest_asset_path(p, project_root),
                            )
                        })
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    if android.is_empty() {
        return None;
    }

    Some(AndroidIconsConfig { android })
}

pub(super) fn find_manifest_ios_icons(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<IosIconsConfig> {
    let ios_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("icons"))
        .and_then(|v| v.get("ios"))?;

    let mut ios = BTreeMap::new();
    if let Some(path) = ios_value
        .get("appstore")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        ios.insert(
            "appstore".to_string(),
            resolve_manifest_asset_path(path, project_root),
        );
    }

    for idiom in ["iphone", "ipad"] {
        let Some(items) = ios_value.get(idiom).and_then(|v| v.as_object()) else {
            continue;
        };
        for (slot, path) in items {
            let Some(path) = path.as_str().map(str::trim).filter(|path| !path.is_empty()) else {
                continue;
            };
            ios.insert(
                format!("{}.{}", idiom, slot),
                resolve_manifest_asset_path(path, project_root),
            );
        }
    }

    if ios.is_empty() {
        return None;
    }

    Some(IosIconsConfig { ios })
}

pub(super) fn find_manifest_ios_privacy_descriptions(
    manifest: &serde_json::Value,
) -> BTreeMap<String, String> {
    let Some(map) = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("ios"))
        .and_then(|v| v.get("privacyDescription"))
        .and_then(|v| v.as_object())
    else {
        return BTreeMap::new();
    };

    map.iter()
        .filter_map(|(key, value)| {
            let key = key.trim();
            if !is_supported_ios_privacy_description_key(key) {
                return None;
            }
            let value = value.as_str().map(str::trim).filter(|v| !v.is_empty())?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
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

pub(super) fn find_manifest_push_icons(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<PushIconsConfig> {
    let distribute = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.as_object())?;

    let mut config = PushIconsConfig::default();
    if let Some(push_config) = distribute.get("push") {
        for small_value in push_small_icon_values(push_config) {
            collect_push_small_icon_value(small_value, project_root, &mut config);
        }
    }
    if let Some(push_config) = distribute.get("sdkConfigs").and_then(|v| v.get("push")) {
        for small_value in push_small_icon_values(push_config) {
            collect_push_small_icon_value(small_value, project_root, &mut config);
        }
    }

    if config.small.is_none() && config.small_densities.is_empty() {
        None
    } else {
        Some(config)
    }
}

fn push_small_icon_values(push_config: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut values = Vec::new();
    if let Some(value) = push_config
        .get("icons")
        .and_then(|icons| icons.get("small"))
    {
        values.push(value);
    }
    for key in ["unipush", "unipushV2", "uniPush"] {
        if let Some(value) = push_config
            .get(key)
            .and_then(|provider| provider.get("icons"))
            .and_then(|icons| icons.get("small"))
        {
            values.push(value);
        }
    }
    values
}

fn collect_push_small_icon_value(
    value: &serde_json::Value,
    project_root: &Path,
    config: &mut PushIconsConfig,
) {
    if let Some(path) = value
        .as_str()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        config.small = Some(resolve_manifest_asset_path(path, project_root));
        return;
    }

    let Some(items) = value.as_object() else {
        return;
    };
    for (density, path) in items {
        let Some(path) = path.as_str().map(str::trim).filter(|path| !path.is_empty()) else {
            continue;
        };
        config.small_densities.insert(
            density.to_string(),
            resolve_manifest_asset_path(path, project_root),
        );
    }
}

pub(super) fn find_manifest_splashscreen(
    manifest: &serde_json::Value,
    project_root: &Path,
) -> Option<SplashscreenConfig> {
    let value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("splashscreen"))
        .or_else(|| manifest.get("app-plus").and_then(|v| v.get("splashscreen")))
        .or_else(|| manifest.get("splashscreen"))?;

    let android = value
        .get("android")
        .and_then(|v| v.as_object())
        .map(|items| {
            items
                .iter()
                .filter_map(|(density, path)| {
                    path.as_str()
                        .map(str::trim)
                        .filter(|path| !path.is_empty())
                        .map(|path| {
                            (
                                density.to_string(),
                                resolve_manifest_asset_path(path, project_root),
                            )
                        })
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let config = SplashscreenConfig {
        android_style: string_field(value, &["androidStyle", "android_style"]),
        android,
        ios_style: string_field(value, &["iosStyle", "ios_style"]),
        ios_storyboard: value
            .get("ios")
            .and_then(|ios| string_field(ios, &["storyboard"]))
            .map(|path| resolve_manifest_asset_path(&path, project_root)),
        use_original_msgbox: bool_field(value, &["useOriginalMsgbox", "use_original_msgbox"]),
    };

    if config.android_style.is_none()
        && config.android.is_empty()
        && config.ios_style.is_none()
        && config.ios_storyboard.is_none()
        && config.use_original_msgbox.is_none()
    {
        None
    } else {
        Some(config)
    }
}

fn resolve_manifest_asset_path(path: &str, project_root: &Path) -> String {
    if path.contains("://") || path.starts_with("data:") {
        return path.to_string();
    }
    let path = PathBuf::from(path);
    let absolute = if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    };
    absolute.to_string_lossy().to_string()
}
