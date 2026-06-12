use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_FINGERPRINT_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[IosPrivacyFieldSpec {
    key: "NSFaceIDUsageDescription",
    label: "指纹/面容识别说明",
    default_value: "用于通过指纹或面容识别验证身份",
    required: true,
}];

pub(crate) fn ios_fingerprint_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "Fingerprint")
}

pub(crate) fn apply_ios_fingerprint_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_FINGERPRINT_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}
