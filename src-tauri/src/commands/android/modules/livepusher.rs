//! 直播推流模块 (livepusher) manifest 补丁

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::commands::shared::module::types::AndroidModuleConfigModule;
use crate::commands::android::project_mod::ManifestPatchGroup;

use super::helpers::*;

pub fn render_patches(
    module: &AndroidModuleConfigModule,
    permissions: &mut BTreeSet<String>,
    application_entries: &mut BTreeSet<String>,
    _pandora_filters: &mut BTreeSet<String>,
    placeholders: &HashMap<String, String>,
    _package_name: &str,
    patch_groups: &mut BTreeMap<String, ManifestPatchGroup>,
) {
    let mut mod_perms: Vec<String> = Vec::new();
    let mut mod_entries: Vec<String> = Vec::new();

    add_permissions(
        permissions,
        &[
            "android.permission.BLUETOOTH",
            "android.permission.CAMERA",
            "android.permission.RECORD_AUDIO",
            "android.permission.MODIFY_AUDIO_SETTINGS",
        ],
    );
    mod_perms.extend([
        "android.permission.BLUETOOTH".to_string(),
        "android.permission.CAMERA".to_string(),
        "android.permission.RECORD_AUDIO".to_string(),
        "android.permission.MODIFY_AUDIO_SETTINGS".to_string(),
    ]);

    if has_report_value(module, "LIVEPUSH_LICENSE_URL") {
        let livepush_entries = [
            meta_data(
                "TXLIVE_LICENSE_URL",
                &placeholder_value(placeholders, "LIVEPUSH_LICENSE_URL"),
            ),
            meta_data(
                "TXLIVE_LICENSE_KEY",
                &placeholder_value(placeholders, "LIVEPUSH_LICENSE_KEY"),
            ),
        ];
        add_application_entries(application_entries, &livepush_entries);
        mod_entries.extend(livepush_entries.iter().cloned());
    }

    // 写入 patch group
    let group_name = module.template_key.clone();
    let group = patch_groups
        .entry(group_name.clone())
        .or_insert_with(|| ManifestPatchGroup {
            module_name: group_name,
            permissions: Vec::new(),
            application_entries: Vec::new(),
            intent_filters: Vec::new(),
        });
    group.permissions.extend(mod_perms);
    group.application_entries.extend(mod_entries);
}
