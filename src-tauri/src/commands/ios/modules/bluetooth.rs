use std::path::Path;

use crate::commands::ios::build::pbxproj::enable_pbx_system_capability;
use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_has_detected_module,
    ios_manifest_module_enabled, merge_plist_string_array, IosPrivacyFieldSpec,
};

pub(crate) const IOS_BLUETOOTH_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSBluetoothAlwaysUsageDescription",
        label: "蓝牙权限说明（始终使用）",
        default_value: "用于连接和管理低功耗蓝牙设备",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSBluetoothPeripheralUsageDescription",
        label: "蓝牙权限说明（iOS 13+）",
        default_value: "用于连接和管理低功耗蓝牙设备",
        required: true,
    },
];

#[derive(Debug, Clone)]
pub(crate) struct IosBluetoothIntegration {
    pub(crate) background_enabled: bool,
}

pub(crate) fn ios_bluetooth_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    let Some(info) = manifest_info else {
        return false;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return false;
    };
    ios_manifest_info_has_detected_module(info, "Bluetooth")
        && ios_manifest_module_enabled(manifest, "Bluetooth")
}

pub(crate) fn ios_bluetooth_background_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    let Some(info) = manifest_info else {
        return false;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return false;
    };
    ios_bluetooth_enabled(Some(info)) && ios_manifest_has_bluetooth_background_modes(manifest)
}

pub(crate) fn apply_ios_bluetooth_module(
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosBluetoothIntegration>, String> {
    if !ios_bluetooth_enabled(manifest_info) {
        return Ok(None);
    }
    let background_enabled = ios_bluetooth_background_enabled(manifest_info);
    if background_enabled {
        enable_pbx_system_capability(project_file, "com.apple.BackgroundModes")?;
    }
    Ok(Some(IosBluetoothIntegration { background_enabled }))
}

pub(crate) fn apply_ios_bluetooth_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_BLUETOOTH_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}

pub(crate) fn apply_ios_bluetooth_background_modes(dict: &mut plist::Dictionary) {
    merge_plist_string_array(
        dict,
        "UIBackgroundModes",
        &["bluetooth-central", "bluetooth-peripheral"],
    );
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
