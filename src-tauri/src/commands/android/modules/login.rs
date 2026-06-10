//! 登录鉴权模块 (login) manifest 补丁
//!
//! 支持微信登录、QQ登录、新浪登录、一键登录(GY/univerify)、小米登录。

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

    add_permissions(permissions, &["android.permission.MODIFY_AUDIO_SETTINGS"]);
    mod_perms.push("android.permission.MODIFY_AUDIO_SETTINGS".to_string());

    if has_report_value(module, "WX_APPID") {
        let wx_appid = placeholder_value(placeholders, "WX_APPID");
        let wx_login_entries = [
            meta_data("WX_APPID", &wx_appid),
            meta_data("WX_SECRET", &placeholder_value(placeholders, "WX_SECRET")),
            wx_entry_activity(package_name, &wx_appid),
        ];
        add_application_entries(application_entries, &wx_login_entries);
        mod_entries.extend(wx_login_entries.iter().cloned());
    }
    if has_report_value(module, "QQ_APPID") {
        let qq_appid = placeholder_value(placeholders, "QQ_APPID");
        let qq_login_entries = [
            meta_data("QQ_APPID", &qq_appid),
            qq_auth_activity(&qq_appid),
            qq_assist_activity(),
        ];
        add_application_entries(application_entries, &qq_login_entries);
        mod_entries.extend(qq_login_entries.iter().cloned());
    }
    if has_report_value(module, "GY_APP_ID") {
        let gy_app_id = placeholder_value(placeholders, "GY_APP_ID");
        let gy_entries = [
            meta_data("GETUI_APPID", &gy_app_id),
            meta_data("GY_APP_ID", &gy_app_id),
        ];
        add_application_entries(application_entries, &gy_entries);
        mod_entries.extend(gy_entries.iter().cloned());
    }
    if has_report_value(module, "SINA_APPKEY") {
        let sina_login_entries = [
            meta_data(
                "SINA_APPKEY",
                &format!("_{}", placeholder_value(placeholders, "SINA_APPKEY")),
            ),
            meta_data(
                "SINA_REDIRECT_URI",
                &placeholder_value(placeholders, "SINA_REDIRECT_URI"),
            ),
            service_entry(
                r#"<activity android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity" android:configChanges="keyboardHidden|orientation" android:exported="false" android:windowSoftInputMode="adjustResize" />"#,
            ),
        ];
        add_application_entries(application_entries, &sina_login_entries);
        mod_entries.extend(sina_login_entries.iter().cloned());
    }
    if has_report_value(module, "MIUI_APPID") {
        add_permissions(permissions, &["com.xiaomi.permission.AUTH_SERVICE"]);
        mod_perms.push("com.xiaomi.permission.AUTH_SERVICE".to_string());
        let miui_entries = [
            meta_data(
                "MIUI_APPID",
                &format!("_{}", placeholder_value(placeholders, "MIUI_APPID")),
            ),
            meta_data(
                "MIUI_APPSECRET",
                &placeholder_value(placeholders, "MIUI_APPSECRET"),
            ),
            meta_data(
                "MIUI_REDIRECT_URI",
                &placeholder_value(placeholders, "MIUI_REDIRECT_URI"),
            ),
            service_entry(
                r#"<activity android:name="com.xiaomi.account.openauth.AuthorizeActivity" />"#,
            ),
        ];
        add_application_entries(application_entries, &miui_entries);
        mod_entries.extend(miui_entries.iter().cloned());
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
