//! Push / UniPush 模块 manifest 补丁

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::commands::shared::module::types::AndroidModuleConfigModule;
use crate::commands::android::project_mod::ManifestPatchGroup;

use super::helpers::*;

pub fn render_patches(
    module: &AndroidModuleConfigModule,
    _permissions: &mut BTreeSet<String>,
    application_entries: &mut BTreeSet<String>,
    pandora_filters: &mut BTreeSet<String>,
    placeholders: &HashMap<String, String>,
    _package_name: &str,
    patch_groups: &mut BTreeMap<String, ManifestPatchGroup>,
) {
    let push_entries = [
        meta_data(
            "GETUI_APPID",
            &placeholder_value(placeholders, "GETUI_APPID"),
        ),
        meta_data(
            "plus.unipush.appid",
            &placeholder_value(placeholders, "plus.unipush.appid"),
        ),
        meta_data(
            "plus.unipush.appkey",
            &placeholder_value(placeholders, "plus.unipush.appkey"),
        ),
        meta_data(
            "plus.unipush.appsecret",
            &placeholder_value(placeholders, "plus.unipush.appsecret"),
        ),
    ];
    add_application_entries(application_entries, &push_entries);

    let push_filter = crate::commands::android::types::indent_manifest_fragment(
        r#"<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:host="io.dcloud.unipush" android:path="/" android:scheme="unipush" />
</intent-filter>"#,
        12,
    );
    pandora_filters.insert(push_filter.clone());

    // 写入 patch group
    let group_name = module.template_key.clone();
    let group = patch_groups
        .entry(group_name.clone())
        .or_insert_with(|| ManifestPatchGroup {
            module_name: group_name,
            permissions: Vec::new(),
            application_entries: push_entries.to_vec(),
            intent_filters: vec![push_filter.clone()],
        });
    group
        .application_entries
        .extend(push_entries.iter().cloned());
    group.intent_filters.push(push_filter);
}
