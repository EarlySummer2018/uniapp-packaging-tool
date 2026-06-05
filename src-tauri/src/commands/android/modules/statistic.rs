//! 统计模块 (statistic) manifest 补丁
//!
//! 支持友盟统计、谷歌统计(GA/Firebase)。

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::commands::android::project_mod::ManifestPatchGroup;
use crate::commands::shared::module::types::AndroidModuleConfigModule;

use super::helpers::*;

pub fn render_patches(
    module: &AndroidModuleConfigModule,
    _permissions: &mut BTreeSet<String>,
    application_entries: &mut BTreeSet<String>,
    _pandora_filters: &mut BTreeSet<String>,
    placeholders: &HashMap<String, String>,
    _package_name: &str,
    patch_groups: &mut BTreeMap<String, ManifestPatchGroup>,
) {
    let mut mod_entries: Vec<String> = Vec::new();

    let stat_entries = [
        meta_data(
            "UMENG_APPKEY",
            &placeholder_value(placeholders, "UMENG_APPKEY"),
        ),
        meta_data(
            "UMENG_CHANNEL",
            &placeholder_value(placeholders, "UMENG_CHANNEL"),
        ),
    ];
    add_application_entries(application_entries, &stat_entries);
    mod_entries.extend(stat_entries.iter().cloned());

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
    group.application_entries.extend(mod_entries);
}
