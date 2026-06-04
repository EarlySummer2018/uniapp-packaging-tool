//! 语音输入模块 (speech) manifest 补丁
//!
//! 支持百度语音、讯飞语音。

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
            "android.permission.CHANGE_NETWORK_STATE",
            "android.permission.RECORD_AUDIO",
        ],
    );
    mod_perms.extend([
        "android.permission.CHANGE_NETWORK_STATE".to_string(),
        "android.permission.RECORD_AUDIO".to_string(),
    ]);

    if has_report_value(module, "BAIDU_SPEECH_APP_ID") {
        let baidu_speech_entries = [
            meta_data(
                "com.baidu.speech.APP_ID",
                &placeholder_value(placeholders, "BAIDU_SPEECH_APP_ID"),
            ),
            meta_data(
                "com.baidu.speech.API_KEY",
                &placeholder_value(placeholders, "BD_SPEECH_APIKEY"),
            ),
            meta_data(
                "com.baidu.speech.SECRET_KEY",
                &placeholder_value(placeholders, "BD_SPEECH_SECRETKEY"),
            ),
            service_entry(
                r#"<service android:name="com.baidu.speech.VoiceRecognitionService" android:exported="false" />"#,
            ),
        ];
        add_application_entries(application_entries, &baidu_speech_entries);
        mod_entries.extend(baidu_speech_entries.iter().cloned());
    }
    if has_report_value(module, "IFLY_APPID") {
        let ifly_entries = [meta_data(
            "IFLY_APPKEY",
            &placeholder_value(placeholders, "IFLY_APPID"),
        )];
        add_application_entries(application_entries, &ifly_entries);
        mod_entries.extend(ifly_entries.iter().cloned());
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
