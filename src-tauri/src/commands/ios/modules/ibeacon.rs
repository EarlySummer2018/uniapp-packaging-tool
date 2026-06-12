use std::path::Path;

use crate::commands::ios::build::pbxproj::enable_pbx_system_capability;
use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, merge_plist_string_array,
    IosPrivacyFieldSpec,
};

pub(crate) const IOS_IBEACON_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[IosPrivacyFieldSpec {
    key: "NSLocationAlwaysAndWhenInUseUsageDescription",
    label: "Beacon 位置权限说明",
    default_value: "扫描蓝牙 Beacon 设备",
    required: true,
}];

const IOS_IBEACON_BACKGROUND_MODES: &[&str] = &["location", "bluetooth-central"];

#[derive(Debug, Clone)]
pub(crate) struct IosIBeaconIntegration {
    pub(crate) background_modes: Vec<&'static str>,
}

pub(crate) fn ios_ibeacon_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "iBeacon")
}

pub(crate) fn apply_ios_ibeacon_module(
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosIBeaconIntegration>, String> {
    if !ios_ibeacon_enabled(manifest_info) {
        return Ok(None);
    }
    enable_pbx_system_capability(project_file, "com.apple.BackgroundModes")?;
    Ok(Some(IosIBeaconIntegration {
        background_modes: IOS_IBEACON_BACKGROUND_MODES.to_vec(),
    }))
}

pub(crate) fn apply_ios_ibeacon_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_IBEACON_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}

pub(crate) fn apply_ios_ibeacon_background_modes(dict: &mut plist::Dictionary) {
    merge_plist_string_array(dict, "UIBackgroundModes", IOS_IBEACON_BACKGROUND_MODES);
}
