//! 分享模块 (share) manifest 补丁
//!
//! 支持微信分享、QQ分享、新浪微博分享。

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
    package_name: &str,
    patch_groups: &mut BTreeMap<String, ManifestPatchGroup>,
) {
    let mut mod_perms: Vec<String> = Vec::new();
    let mut mod_entries: Vec<String> = Vec::new();

    add_permissions(
        permissions,
        &[
            "android.permission.MODIFY_AUDIO_SETTINGS",
            "android.permission.CHANGE_WIFI_STATE",
        ],
    );
    mod_perms.extend([
        "android.permission.MODIFY_AUDIO_SETTINGS".to_string(),
        "android.permission.CHANGE_WIFI_STATE".to_string(),
    ]);

    if has_report_value(module, "WX_APPID") {
        let wx_appid = placeholder_value(placeholders, "WX_APPID");
        let wx_share_entries = [
            meta_data("WX_APPID", &wx_appid),
            meta_data("WX_SECRET", &placeholder_value(placeholders, "WX_SECRET")),
            wx_entry_activity(package_name, &wx_appid),
        ];
        add_application_entries(application_entries, &wx_share_entries);
        mod_entries.extend(wx_share_entries.iter().cloned());
    }
    if has_report_value(module, "QQ_APPID") {
        let qq_appid = placeholder_value(placeholders, "QQ_APPID");
        let qq_share_entries = [
            meta_data("QQ_APPID", &qq_appid),
            qq_auth_activity(&qq_appid),
            qq_assist_activity(),
        ];
        add_application_entries(application_entries, &qq_share_entries);
        mod_entries.extend(qq_share_entries.iter().cloned());
    }
    if has_report_value(module, "SINA_APPKEY") {
        add_permissions(
            permissions,
            &[
                "android.permission.ACCESS_WIFI_STATE",
                "android.permission.ACCESS_NETWORK_STATE",
            ],
        );
        mod_perms.extend([
            "android.permission.ACCESS_WIFI_STATE".to_string(),
            "android.permission.ACCESS_NETWORK_STATE".to_string(),
        ]);
        let sina_share_entries = [
            meta_data(
                "SINA_APPKEY",
                &format!("_{}", placeholder_value(placeholders, "SINA_APPKEY")),
            ),
            meta_data(
                "SINA_SECRET",
                &placeholder_value(placeholders, "SINA_SECRET"),
            ),
            meta_data(
                "SINA_REDIRECT_URI",
                &placeholder_value(placeholders, "SINA_REDIRECT_URI"),
            ),
            service_entry(
                r#"<activity android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity" android:configChanges="keyboardHidden|orientation" android:exported="false" android:windowSoftInputMode="adjustResize" />"#,
            ),
            service_entry(
                r#"<activity android:name="com.sina.weibo.sdk.share.WbShareTransActivity" android:exported="true" android:launchMode="singleTask" android:theme="@android:style/Theme.Translucent.NoTitleBar.Fullscreen">
    <intent-filter>
        <action android:name="com.sina.weibo.sdk.action.ACTION_SDK_REQ_ACTIVITY" />
        <category android:name="android.intent.category.DEFAULT" />
    </intent-filter>
</activity>"#,
            ),
        ];
        add_application_entries(application_entries, &sina_share_entries);
        mod_entries.extend(sina_share_entries.iter().cloned());
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
