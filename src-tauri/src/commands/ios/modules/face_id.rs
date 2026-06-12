use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_FACE_ID_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[IosPrivacyFieldSpec {
    key: "NSFaceIDUsageDescription",
    label: "Face ID 权限说明",
    default_value: "用于通过面容识别验证身份",
    required: true,
}];

pub(crate) fn ios_face_id_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "FaceID")
}

pub(crate) fn apply_ios_face_id_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_FACE_ID_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}
