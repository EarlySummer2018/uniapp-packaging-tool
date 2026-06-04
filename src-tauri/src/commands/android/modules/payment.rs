//! 支付模块 (payment) manifest 补丁
//!
//! 支持微信支付、支付宝。

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::commands::shared::module::types::AndroidModuleConfigModule;
use crate::utils::android_project_mod::ManifestPatchGroup;

use super::helpers::*;

pub fn render_patches(
    module: &AndroidModuleConfigModule,
    permissions: &mut BTreeSet<String>,
    application_entries: &mut BTreeSet<String>,
    _pandora_filters: &mut BTreeSet<String>,
    placeholders: &HashMap<String, String>,
    package_name: &str,
    patch_groups: &mut BTreeMap<String, ManifestPatchGroup>,
) {
    let mut mod_perms: Vec<String> = Vec::new();
    let mut mod_entries: Vec<String> = Vec::new();

    add_permissions(
        permissions,
        &[
            "android.permission.MODIFY_AUDIO_SETTINGS",
            "android.permission.ACCESS_COARSE_LOCATION",
        ],
    );
    mod_perms.extend([
        "android.permission.MODIFY_AUDIO_SETTINGS".to_string(),
        "android.permission.ACCESS_COARSE_LOCATION".to_string(),
    ]);

    if has_report_value(module, "WX_APPID") {
        let wx_pay_entries = [
            meta_data("WX_APPID", &placeholder_value(placeholders, "WX_APPID")),
            service_entry(
                r#"<activity android:name="io.dcloud.feature.payment.weixin.WXPayProcessMeadiatorActivity" android:exported="false" android:excludeFromRecents="true" android:theme="@style/TranslucentTheme" />"#,
            ),
            service_entry(&format!(
                r#"<activity android:name="{}.wxapi.WXPayEntryActivity" android:exported="true" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:launchMode="singleTop" />"#,
                package_name
            )),
        ];
        add_application_entries(application_entries, &wx_pay_entries);
        mod_entries.extend(wx_pay_entries.iter().cloned());
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
