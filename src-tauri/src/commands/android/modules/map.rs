//! 地图模块 (map) manifest 补丁
//!
//! 支持百度地图、高德地图、Google地图、腾讯地图。

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::commands::android::project_mod::ManifestPatchGroup;
use crate::commands::shared::module::types::AndroidModuleConfigModule;

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

    let provider = active_map_provider(module);
    let map_permissions = match provider {
        Some(MapProvider::Baidu) => &[
            "android.permission.CHANGE_WIFI_STATE",
            "android.permission.MOUNT_UNMOUNT_FILESYSTEMS",
            "android.permission.READ_LOGS",
            "android.permission.WRITE_SETTINGS",
        ][..],
        Some(MapProvider::Amap) => &[
            "android.permission.CHANGE_WIFI_STATE",
            "android.permission.MOUNT_UNMOUNT_FILESYSTEMS",
        ][..],
        Some(MapProvider::Google) => &["android.permission.ACCESS_LOCATION_EXTRA_COMMANDS"][..],
        Some(MapProvider::Tencent) | None => &[][..],
    };
    add_permissions(permissions, map_permissions);
    mod_perms.extend(
        map_permissions
            .iter()
            .map(|permission| (*permission).to_string()),
    );

    if provider == Some(MapProvider::Baidu) {
        let baidu_map_entries = [
            meta_data(
                "com.baidu.lbsapi.API_KEY",
                &placeholder_value(placeholders, "BAIDU_MAP_AK"),
            ),
            service_entry(
                r#"<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote" />"#,
            ),
        ];
        add_application_entries(application_entries, &baidu_map_entries);
        mod_entries.extend(baidu_map_entries.iter().cloned());
    }
    if provider == Some(MapProvider::Amap) {
        let amap_map_entries = [
            meta_data(
                "com.amap.api.v2.apikey",
                &placeholder_value(placeholders, "AMAP_KEY"),
            ),
            service_entry(r#"<service android:name="com.amap.api.location.APSService" />"#),
        ];
        add_application_entries(application_entries, &amap_map_entries);
        mod_entries.extend(amap_map_entries.iter().cloned());
    }
    if provider == Some(MapProvider::Google) {
        let google_entries = [meta_data(
            "com.google.android.geo.API_KEY",
            &placeholder_value(placeholders, "GOOGLE_MAPS_API_KEY"),
        )];
        add_application_entries(application_entries, &google_entries);
        mod_entries.extend(google_entries.iter().cloned());
    }
    if provider == Some(MapProvider::Tencent) {
        let tencent_map_entries = [meta_data(
            "TencentMapSDK",
            &placeholder_value(placeholders, "TENCENT_MAP_KEY"),
        )];
        add_application_entries(application_entries, &tencent_map_entries);
        mod_entries.extend(tencent_map_entries.iter().cloned());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapProvider {
    Baidu,
    Amap,
    Google,
    Tencent,
}

fn active_map_provider(module: &AndroidModuleConfigModule) -> Option<MapProvider> {
    if has_report_value(module, "BAIDU_MAP_AK") {
        Some(MapProvider::Baidu)
    } else if has_report_value(module, "AMAP_KEY") {
        Some(MapProvider::Amap)
    } else if has_report_value(module, "GOOGLE_MAPS_API_KEY") {
        Some(MapProvider::Google)
    } else if has_report_value(module, "TENCENT_MAP_KEY") {
        Some(MapProvider::Tencent)
    } else {
        None
    }
}
