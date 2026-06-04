//! 定位模块 (geolocation) manifest 补丁
//!
//! 支持百度定位、高德定位、腾讯定位、系统定位。

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
            "android.permission.CHANGE_WIFI_STATE",
            "android.permission.MOUNT_UNMOUNT_FILESYSTEMS",
            "android.permission.READ_LOGS",
            "android.permission.WRITE_SETTINGS",
            "android.permission.ACCESS_BACKGROUND_LOCATION",
            "android.permission.FOREGROUND_SERVICE",
        ],
    );
    mod_perms.extend([
        "android.permission.CHANGE_WIFI_STATE".to_string(),
        "android.permission.MOUNT_UNMOUNT_FILESYSTEMS".to_string(),
        "android.permission.READ_LOGS".to_string(),
        "android.permission.WRITE_SETTINGS".to_string(),
        "android.permission.ACCESS_BACKGROUND_LOCATION".to_string(),
        "android.permission.FOREGROUND_SERVICE".to_string(),
    ]);

    if has_report_value(module, "BAIDU_MAP_AK") {
        let baidu_entries = [
            meta_data(
                "com.baidu.lbsapi.API_KEY",
                &placeholder_value(placeholders, "BAIDU_MAP_AK"),
            ),
            service_entry(
                r#"<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote" />"#,
            ),
        ];
        add_application_entries(application_entries, &baidu_entries);
        mod_entries.extend(baidu_entries.iter().cloned());
    }
    if has_report_value(module, "AMAP_KEY") {
        let amap_entries = [
            meta_data(
                "com.amap.api.v2.apikey",
                &placeholder_value(placeholders, "AMAP_KEY"),
            ),
            service_entry(r#"<service android:name="com.amap.api.location.APSService" />"#),
        ];
        add_application_entries(application_entries, &amap_entries);
        mod_entries.extend(amap_entries.iter().cloned());
    }
    if has_report_value(module, "TENCENT_MAP_KEY") {
        let tencent_entries = [meta_data(
            "TencentMapSDK",
            &placeholder_value(placeholders, "TENCENT_MAP_KEY"),
        )];
        add_application_entries(application_entries, &tencent_entries);
        mod_entries.extend(tencent_entries.iter().cloned());
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
