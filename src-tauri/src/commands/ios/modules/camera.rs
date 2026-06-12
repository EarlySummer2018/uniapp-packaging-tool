use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_CAMERA_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSCameraUsageDescription",
        label: "相机权限说明",
        default_value: "用于拍摄照片或视频",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSPhotoLibraryUsageDescription",
        label: "相册权限说明",
        default_value: "用于读取和写入相册中的照片或视频",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSPhotoLibraryAddUsageDescription",
        label: "保存到相册说明",
        default_value: "用于将图片或视频保存到系统相册",
        required: false,
    },
];

pub(crate) fn ios_camera_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "Camera")
}

pub(crate) fn apply_ios_camera_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_CAMERA_PRIVACY_FIELDS
        .iter()
        .filter(|field| field.required)
    {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}
