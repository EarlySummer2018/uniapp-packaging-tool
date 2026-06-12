use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_LIVEPUSHER_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSCameraUsageDescription",
        label: "相机权限说明",
        default_value: "用于视频直播",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSMicrophoneUsageDescription",
        label: "麦克风权限说明",
        default_value: "用于音频采集",
        required: true,
    },
];

pub(crate) fn ios_livepusher_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "LivePusher")
}

pub(crate) fn apply_ios_livepusher_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_LIVEPUSHER_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}
