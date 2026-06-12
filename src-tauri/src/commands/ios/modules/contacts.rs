use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_CONTACTS_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[IosPrivacyFieldSpec {
    key: "NSContactsUsageDescription",
    label: "通讯录权限说明",
    default_value: "用于读取和管理通讯录联系人",
    required: true,
}];

pub(crate) fn ios_contacts_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "Contacts")
}

pub(crate) fn apply_ios_contacts_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_CONTACTS_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}
