//! 广告模块 (uni_ad) manifest 补丁
//!
//! 支持穿山甲(CSJ)、优量汇(GDT)、GroMore、AdMob、快手、Sigmob、百度、华为等。

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

    let uni_ad_entries = [
        meta_data(
            "DCLOUD_AD_SPLASH",
            &placeholder_value(placeholders, "DCLOUD_AD_SPLASH"),
        ),
        meta_data(
            "DCLOUD_STREAMAPP_CHANNEL",
            &placeholder_value(placeholders, "DCLOUD_STREAMAPP_CHANNEL"),
        ),
    ];
    add_application_entries(application_entries, &uni_ad_entries);
    mod_entries.extend(uni_ad_entries.iter().cloned());

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
