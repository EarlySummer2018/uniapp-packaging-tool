//! 模块补丁分发注册
//!
//! 遍历 report 中的所有模块，按 template_key 分发到各子模块的 `render_patches` 函数，
//! 收集全局权限 / application 条目 / pandora intent-filter 以及按模块分组的补丁。

#![allow(dead_code)]

mod face_recognition;
mod geolocation;
mod helpers;
mod livepusher;
mod login;
mod map;
mod payment;
mod push;
mod share;
mod speech;
mod statistic;
mod uni_ad;

pub(crate) use helpers::*;

use std::collections::{BTreeMap, BTreeSet};

use crate::commands::android::types::{prefix_if_nonempty, AndroidManifestPatches};
use crate::commands::shared::module::types::AndroidModuleConfigReport;

/// 分发入口：遍历 modules 并调用各模块的 render_patches
///
/// 返回 `(AndroidManifestPatches, Vec<ManifestPatchGroup>)`，
/// 与原 `render_android_module_manifest_patches_impl` 的返回值一致。
pub fn render_all_patches(
    report: Option<&AndroidModuleConfigReport>,
    package_name: &str,
    app_id: &str,
) -> (
    AndroidManifestPatches,
    Vec<crate::commands::android::project_mod::ManifestPatchGroup>,
) {
    let Some(report) = report else {
        return (AndroidManifestPatches::default(), Vec::new());
    };

    let mut permissions = BTreeSet::new();
    let mut application_entries = BTreeSet::new();
    let mut pandora_entry_intent_filters = BTreeSet::new();
    let mut groups_map: BTreeMap<
        String,
        crate::commands::android::project_mod::ManifestPatchGroup,
    > = BTreeMap::new();
    let has_univerify = report
        .modules
        .iter()
        .any(|module| module.template_key == "login" && has_report_value(module, "GY_APP_ID"));

    for module in &report.modules {
        let placeholders = module_placeholders(module);

        match module.template_key.as_str() {
            "push" => push::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                has_univerify,
                &mut groups_map,
            ),
            "geolocation" => geolocation::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "share" => share::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "login" => login::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "map" => map::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "payment" => payment::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "speech" => speech::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "statistic" => statistic::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "uni_ad" => uni_ad::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "livepusher" => livepusher::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            "face_recognition" => face_recognition::render_patches(
                module,
                &mut permissions,
                &mut application_entries,
                &mut pandora_entry_intent_filters,
                &placeholders,
                package_name,
                &mut groups_map,
            ),
            _ => {}
        }
    }

    // uni-AD 默认渠道注释（与原逻辑一致）
    if report
        .modules
        .iter()
        .any(|module| module.template_key == "uni_ad")
    {
        let fallback_channel = format!("{}|{}||default", package_name, app_id);
        application_entries.insert(format!(
            "        <!-- uni-AD 默认渠道示例: {} -->",
            fallback_channel
        ));
    }

    let permissions_str = permissions
        .into_iter()
        .map(|permission| format!("    <uses-permission android:name=\"{}\" />", permission))
        .collect::<Vec<_>>()
        .join("\n");
    let application_entries_str = application_entries
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    let pandora_entry_intent_filters_str = pandora_entry_intent_filters
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");

    let patches = AndroidManifestPatches {
        permissions: prefix_if_nonempty(permissions_str, "\n"),
        application_entries: prefix_if_nonempty(application_entries_str, "\n"),
        pandora_entry_intent_filters: prefix_if_nonempty(pandora_entry_intent_filters_str, "\n"),
    };
    let groups: Vec<crate::commands::android::project_mod::ManifestPatchGroup> =
        groups_map.into_values().collect();
    (patches, groups)
}
