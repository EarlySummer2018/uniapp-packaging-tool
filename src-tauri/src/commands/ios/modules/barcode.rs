use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_has_detected_module,
    ios_manifest_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_BARCODE_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSCameraUsageDescription",
        label: "相机权限说明",
        default_value: "用于扫码时访问相机",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSPhotoLibraryUsageDescription",
        label: "相册权限说明",
        default_value: "用于从相册选择图片进行扫码",
        required: true,
    },
];

pub(crate) fn ios_barcode_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    let Some(info) = manifest_info else {
        return false;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return false;
    };
    ios_manifest_info_has_detected_module(info, "Barcode")
        && ios_manifest_module_enabled(manifest, "Barcode")
}

pub(crate) fn apply_ios_barcode_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_BARCODE_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}
