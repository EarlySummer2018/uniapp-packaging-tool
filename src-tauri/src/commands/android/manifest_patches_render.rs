//! Android Manifest 补丁渲染实现
//!
//! 包含 render_android_module_manifest_patches 主函数及其全部辅助函数

#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};

// 从兄弟模块导入类型
use super::types::AndroidManifestPatches;

// ===== Manifest 补丁渲染实现 =====

pub fn render_android_module_manifest_patches(
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    package_name: &str,
    app_id: &str,
) -> (
    AndroidManifestPatches,
    Vec<crate::commands::android::project_mod::ManifestPatchGroup>,
) {
    render_android_module_manifest_patches_impl(report, package_name, app_id)
}
pub fn render_android_module_manifest_patches_impl(
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    package_name: &str,
    app_id: &str,
) -> (
    AndroidManifestPatches,
    Vec<crate::commands::android::project_mod::ManifestPatchGroup>,
) {
    use crate::commands::android::types::{
        indent_manifest_fragment, prefix_if_nonempty, AndroidManifestPatches,
    };

    let Some(report) = report else {
        return (AndroidManifestPatches::default(), Vec::new());
    };

    let mut permissions = BTreeSet::new();
    let mut application_entries = BTreeSet::new();
    let mut pandora_entry_intent_filters = BTreeSet::new();
    let mut groups_map: std::collections::BTreeMap<
        String,
        crate::commands::android::project_mod::ManifestPatchGroup,
    > = std::collections::BTreeMap::new();

    for module in &report.modules {
        let placeholders = module_placeholders(module);
        let mut mod_perms: Vec<String> = Vec::new();
        let mut mod_entries: Vec<String> = Vec::new();
        let mut mod_filters: Vec<String> = Vec::new();

        match module.template_key.as_str() {
            "push" => {
                let push_entries = [
                    meta_data(
                        "GETUI_APPID",
                        &placeholder_value(&placeholders, "GETUI_APPID"),
                    ),
                    meta_data(
                        "plus.unipush.appid",
                        &placeholder_value(&placeholders, "plus.unipush.appid"),
                    ),
                    meta_data(
                        "plus.unipush.appkey",
                        &placeholder_value(&placeholders, "plus.unipush.appkey"),
                    ),
                    meta_data(
                        "plus.unipush.appsecret",
                        &placeholder_value(&placeholders, "plus.unipush.appsecret"),
                    ),
                ];
                add_application_entries(&mut application_entries, &push_entries);
                mod_entries.extend(push_entries.iter().cloned());

                let push_filter = indent_manifest_fragment(
                    r#"<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:host="io.dcloud.unipush" android:path="/" android:scheme="unipush" />
</intent-filter>"#,
                    12,
                );
                pandora_entry_intent_filters.insert(push_filter.clone());
                mod_filters.push(push_filter);
            }
            "geolocation" => {
                add_permissions(
                    &mut permissions,
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
                            &placeholder_value(&placeholders, "BAIDU_MAP_AK"),
                        ),
                        service_entry(
                            r#"<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &baidu_entries);
                    mod_entries.extend(baidu_entries.iter().cloned());
                }
                if has_report_value(module, "AMAP_KEY") {
                    let amap_entries = [
                        meta_data(
                            "com.amap.api.v2.apikey",
                            &placeholder_value(&placeholders, "AMAP_KEY"),
                        ),
                        service_entry(
                            r#"<service android:name="com.amap.api.location.APSService" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &amap_entries);
                    mod_entries.extend(amap_entries.iter().cloned());
                }
                if has_report_value(module, "TENCENT_MAP_KEY") {
                    let tencent_entries = [meta_data(
                        "TencentMapSDK",
                        &placeholder_value(&placeholders, "TENCENT_MAP_KEY"),
                    )];
                    add_application_entries(&mut application_entries, &tencent_entries);
                    mod_entries.extend(tencent_entries.iter().cloned());
                }
            }
            "share" => {
                add_permissions(
                    &mut permissions,
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
                    let wx_appid = placeholder_value(&placeholders, "WX_APPID");
                    let wx_share_entries = [
                        meta_data("WX_APPID", &wx_appid),
                        meta_data("WX_SECRET", &placeholder_value(&placeholders, "WX_SECRET")),
                        wx_entry_activity(package_name, &wx_appid),
                    ];
                    add_application_entries(&mut application_entries, &wx_share_entries);
                    mod_entries.extend(wx_share_entries.iter().cloned());
                }
                if has_report_value(module, "QQ_APPID") {
                    let qq_appid = placeholder_value(&placeholders, "QQ_APPID");
                    let qq_share_entries = [
                        meta_data("QQ_APPID", &qq_appid),
                        qq_auth_activity(&qq_appid),
                        qq_assist_activity(),
                    ];
                    add_application_entries(&mut application_entries, &qq_share_entries);
                    mod_entries.extend(qq_share_entries.iter().cloned());
                }
                if has_report_value(module, "SINA_APPKEY") {
                    let sina_share_entries = [
                        meta_data(
                            "SINA_APPKEY",
                            &placeholder_value(&placeholders, "SINA_APPKEY"),
                        ),
                        meta_data(
                            "SINA_SECRET",
                            &placeholder_value(&placeholders, "SINA_SECRET"),
                        ),
                        meta_data(
                            "SINA_REDIRECT_URI",
                            &placeholder_value(&placeholders, "SINA_REDIRECT_URI"),
                        ),
                        service_entry(
                            r#"<activity android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity" android:configChanges="keyboardHidden|orientation" android:exported="false" android:windowSoftInputMode="adjustResize" />"#,
                        ),
                        service_entry(
                            r#"<activity android:name="com.sina.weibo.sdk.share.WbShareTransActivity" android:launchMode="singleTask" android:theme="@android:style/Theme.Translucent.NoTitleBar.Fullscreen" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &sina_share_entries);
                    mod_entries.extend(sina_share_entries.iter().cloned());
                }
            }
            "login" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.MODIFY_AUDIO_SETTINGS",
                        "com.xiaomi.permission.AUTH_SERVICE",
                    ],
                );
                mod_perms.extend([
                    "android.permission.MODIFY_AUDIO_SETTINGS".to_string(),
                    "com.xiaomi.permission.AUTH_SERVICE".to_string(),
                ]);
                if has_report_value(module, "WX_APPID") {
                    let wx_appid = placeholder_value(&placeholders, "WX_APPID");
                    let wx_login_entries = [
                        meta_data("WX_APPID", &wx_appid),
                        meta_data("WX_SECRET", &placeholder_value(&placeholders, "WX_SECRET")),
                        wx_entry_activity(package_name, &wx_appid),
                    ];
                    add_application_entries(&mut application_entries, &wx_login_entries);
                    mod_entries.extend(wx_login_entries.iter().cloned());
                }
                if has_report_value(module, "QQ_APPID") {
                    let qq_appid = placeholder_value(&placeholders, "QQ_APPID");
                    let qq_login_entries = [
                        meta_data("QQ_APPID", &qq_appid),
                        qq_auth_activity(&qq_appid),
                        qq_assist_activity(),
                    ];
                    add_application_entries(&mut application_entries, &qq_login_entries);
                    mod_entries.extend(qq_login_entries.iter().cloned());
                }
                if has_report_value(module, "GY_APP_ID") {
                    let gy_entries = [
                        meta_data(
                            "GETUI_APPID",
                            &placeholder_value(&placeholders, "GETUI_APPID"),
                        ),
                        meta_data("GY_APP_ID", &placeholder_value(&placeholders, "GY_APP_ID")),
                    ];
                    add_application_entries(&mut application_entries, &gy_entries);
                    mod_entries.extend(gy_entries.iter().cloned());
                }
                if has_report_value(module, "SINA_APPKEY") {
                    let sina_login_entries = [
                        meta_data(
                            "SINA_APPKEY",
                            &placeholder_value(&placeholders, "SINA_APPKEY"),
                        ),
                        meta_data(
                            "SINA_REDIRECT_URI",
                            &placeholder_value(&placeholders, "SINA_REDIRECT_URI"),
                        ),
                        service_entry(
                            r#"<activity android:name="com.sina.weibo.sdk.web.WeiboSdkWebActivity" android:configChanges="keyboardHidden|orientation" android:exported="false" android:windowSoftInputMode="adjustResize" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &sina_login_entries);
                    mod_entries.extend(sina_login_entries.iter().cloned());
                }
                if has_report_value(module, "MIUI_APPID") {
                    let miui_entries = [
                        meta_data(
                            "MIUI_APPID",
                            &placeholder_value(&placeholders, "MIUI_APPID"),
                        ),
                        meta_data(
                            "MIUI_APPSECRET",
                            &placeholder_value(&placeholders, "MIUI_APPSECRET"),
                        ),
                        meta_data(
                            "MIUI_REDIRECT_URI",
                            &placeholder_value(&placeholders, "MIUI_REDIRECT_URI"),
                        ),
                        service_entry(
                            r#"<activity android:name="com.xiaomi.account.openauth.AuthorizeActivity" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &miui_entries);
                    mod_entries.extend(miui_entries.iter().cloned());
                }
            }
            "map" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.CHANGE_WIFI_STATE",
                        "android.permission.MOUNT_UNMOUNT_FILESYSTEMS",
                        "android.permission.READ_LOGS",
                        "android.permission.WRITE_SETTINGS",
                        "android.permission.ACCESS_LOCATION_EXTRA_COMMANDS",
                    ],
                );
                mod_perms.extend([
                    "android.permission.CHANGE_WIFI_STATE".to_string(),
                    "android.permission.MOUNT_UNMOUNT_FILESYSTEMS".to_string(),
                    "android.permission.READ_LOGS".to_string(),
                    "android.permission.WRITE_SETTINGS".to_string(),
                    "android.permission.ACCESS_LOCATION_EXTRA_COMMANDS".to_string(),
                ]);
                if has_report_value(module, "BAIDU_MAP_AK") {
                    let baidu_map_entries = [
                        meta_data(
                            "com.baidu.lbsapi.API_KEY",
                            &placeholder_value(&placeholders, "BAIDU_MAP_AK"),
                        ),
                        service_entry(
                            r#"<service android:name="com.baidu.location.f" android:enabled="true" android:process=":remote" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &baidu_map_entries);
                    mod_entries.extend(baidu_map_entries.iter().cloned());
                }
                if has_report_value(module, "AMAP_KEY") {
                    let amap_map_entries = [
                        meta_data(
                            "com.amap.api.v2.apikey",
                            &placeholder_value(&placeholders, "AMAP_KEY"),
                        ),
                        service_entry(
                            r#"<service android:name="com.amap.api.location.APSService" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &amap_map_entries);
                    mod_entries.extend(amap_map_entries.iter().cloned());
                }
                if has_report_value(module, "GOOGLE_MAPS_API_KEY") {
                    let google_entries = [meta_data(
                        "com.google.android.geo.API_KEY",
                        &placeholder_value(&placeholders, "GOOGLE_MAPS_API_KEY"),
                    )];
                    add_application_entries(&mut application_entries, &google_entries);
                    mod_entries.extend(google_entries.iter().cloned());
                }
                if has_report_value(module, "TENCENT_MAP_KEY") {
                    let tencent_map_entries = [meta_data(
                        "TencentMapSDK",
                        &placeholder_value(&placeholders, "TENCENT_MAP_KEY"),
                    )];
                    add_application_entries(&mut application_entries, &tencent_map_entries);
                    mod_entries.extend(tencent_map_entries.iter().cloned());
                }
            }
            "payment" => {
                add_permissions(
                    &mut permissions,
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
                        meta_data("WX_APPID", &placeholder_value(&placeholders, "WX_APPID")),
                        service_entry(
                            r#"<activity android:name="io.dcloud.feature.payment.weixin.WXPayProcessMeadiatorActivity" android:exported="false" android:excludeFromRecents="true" android:theme="@style/TranslucentTheme" />"#,
                        ),
                        service_entry(&format!(
                            r#"<activity android:name="{}.wxapi.WXPayEntryActivity" android:exported="true" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:launchMode="singleTop" />"#,
                            package_name
                        )),
                    ];
                    add_application_entries(&mut application_entries, &wx_pay_entries);
                    mod_entries.extend(wx_pay_entries.iter().cloned());
                }
            }
            "speech" => {
                add_permissions(
                    &mut permissions,
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
                            &placeholder_value(&placeholders, "BAIDU_SPEECH_APP_ID"),
                        ),
                        meta_data(
                            "com.baidu.speech.API_KEY",
                            &placeholder_value(&placeholders, "BD_SPEECH_APIKEY"),
                        ),
                        meta_data(
                            "com.baidu.speech.SECRET_KEY",
                            &placeholder_value(&placeholders, "BD_SPEECH_SECRETKEY"),
                        ),
                        service_entry(
                            r#"<service android:name="com.baidu.speech.VoiceRecognitionService" android:exported="false" />"#,
                        ),
                    ];
                    add_application_entries(&mut application_entries, &baidu_speech_entries);
                    mod_entries.extend(baidu_speech_entries.iter().cloned());
                }
                if has_report_value(module, "IFLY_APPID") {
                    let ifly_entries = [meta_data(
                        "IFLY_APPKEY",
                        &placeholder_value(&placeholders, "IFLY_APPID"),
                    )];
                    add_application_entries(&mut application_entries, &ifly_entries);
                    mod_entries.extend(ifly_entries.iter().cloned());
                }
            }
            "statistic" => {
                let stat_entries = [
                    meta_data(
                        "UMENG_APPKEY",
                        &placeholder_value(&placeholders, "UMENG_APPKEY"),
                    ),
                    meta_data(
                        "UMENG_CHANNEL",
                        &placeholder_value(&placeholders, "UMENG_CHANNEL"),
                    ),
                ];
                add_application_entries(&mut application_entries, &stat_entries);
                mod_entries.extend(stat_entries.iter().cloned());
            }
            "uni_ad" => {
                let uni_ad_entries = [
                    meta_data(
                        "DCLOUD_AD_SPLASH",
                        &placeholder_value(&placeholders, "DCLOUD_AD_SPLASH"),
                    ),
                    meta_data(
                        "DCLOUD_STREAMAPP_CHANNEL",
                        &placeholder_value(&placeholders, "DCLOUD_STREAMAPP_CHANNEL"),
                    ),
                ];
                add_application_entries(&mut application_entries, &uni_ad_entries);
                mod_entries.extend(uni_ad_entries.iter().cloned());
                if has_report_value(module, "DCLOUD_STREAMAPP_CHANNEL") {
                    application_entries.insert(indent_manifest_fragment(
                        &format!(r#"<provider android:name="com.bytedance.sdk.openadsdk.TTFileProvider" android:authorities="{}.TTFileProvider" android:exported="false" android:grantUriPermissions="true">
    <meta-data android:name="android.support.FILE_PROVIDER_PATHS" android:resource="@xml/file_paths" />
</provider>
<provider android:name="com.bytedance.sdk.openadsdk.multipro.TTMultiProvider" android:authorities="{}.TTMultiProvider" android:exported="false" />"#, package_name, package_name),
                        8,
                    ));
                }
            }
            "livepusher" => {
                add_permissions(
                    &mut permissions,
                    &[
                        "android.permission.BLUETOOTH",
                        "android.permission.CAMERA",
                        "android.permission.RECORD_AUDIO",
                        "android.permission.MODIFY_AUDIO_SETTINGS",
                    ],
                );
                mod_perms.extend([
                    "android.permission.BLUETOOTH".to_string(),
                    "android.permission.CAMERA".to_string(),
                    "android.permission.RECORD_AUDIO".to_string(),
                    "android.permission.MODIFY_AUDIO_SETTINGS".to_string(),
                ]);
                if has_report_value(module, "LIVEPUSH_LICENSE_URL") {
                    let livepush_entries = [
                        meta_data(
                            "TXLIVE_LICENSE_URL",
                            &placeholder_value(&placeholders, "LIVEPUSH_LICENSE_URL"),
                        ),
                        meta_data(
                            "TXLIVE_LICENSE_KEY",
                            &placeholder_value(&placeholders, "LIVEPUSH_LICENSE_KEY"),
                        ),
                    ];
                    add_application_entries(&mut application_entries, &livepush_entries);
                    mod_entries.extend(livepush_entries.iter().cloned());
                }
            }
            "face_recognition" => {
                let face_entries = [meta_data(
                    "DCLOUD_LICENSE",
                    &placeholder_value(&placeholders, "DCLOUD_LICENSE"),
                )];
                add_application_entries(&mut application_entries, &face_entries);
                mod_entries.extend(face_entries.iter().cloned());
            }
            _ => {}
        }

        // 将本模块的条目合并到分组映射
        if !mod_perms.is_empty() || !mod_entries.is_empty() || !mod_filters.is_empty() {
            let group_name = module.template_key.clone();
            let group = groups_map.entry(group_name.clone()).or_insert_with(|| {
                crate::commands::android::project_mod::ManifestPatchGroup {
                    module_name: group_name,
                    permissions: Vec::new(),
                    application_entries: Vec::new(),
                    intent_filters: Vec::new(),
                }
            });
            group.permissions.extend(mod_perms);
            group.application_entries.extend(mod_entries);
            group.intent_filters.extend(mod_filters);
        }
    }

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

// ===== Manifest 补丁辅助函数 =====

fn add_permissions(target: &mut BTreeSet<String>, permissions: &[&str]) {
    for permission in permissions {
        target.insert((*permission).to_string());
    }
}

fn add_application_entries(target: &mut BTreeSet<String>, entries: &[String]) {
    for entry in entries {
        target.insert(entry.clone());
    }
}

fn has_report_value(
    module: &crate::commands::module::AndroidModuleConfigModule,
    key: &str,
) -> bool {
    module
        .fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| field.value.as_deref())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn module_placeholders(
    module: &crate::commands::module::AndroidModuleConfigModule,
) -> HashMap<String, String> {
    module
        .fields
        .iter()
        .filter_map(|field| {
            field
                .value
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|_| (field.key.clone(), format!("${{{}}}", field.key)))
        })
        .collect()
}

fn placeholder_value(placeholders: &HashMap<String, String>, key: &str) -> String {
    placeholders.get(key).cloned().unwrap_or_default()
}

fn meta_data(name: &str, value: &str) -> String {
    use crate::commands::android::types::indent_manifest_fragment;
    indent_manifest_fragment(
        &format!(
            r#"<meta-data android:name="{}" android:value="{}" />"#,
            name, value
        ),
        8,
    )
}

fn service_entry(entry: &str) -> String {
    use crate::commands::android::types::indent_manifest_fragment;
    indent_manifest_fragment(entry, 8)
}

fn wx_entry_activity(package_name: &str, scheme: &str) -> String {
    use crate::commands::android::types::indent_manifest_fragment;
    indent_manifest_fragment(
        &format!(
            r#"<activity android:name="{}.wxapi.WXEntryActivity" android:label="@string/app_name" android:exported="true" android:launchMode="singleTop">
    <intent-filter>
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <data android:scheme="{}" />
    </intent-filter>
</activity>"#,
            package_name, scheme
        ),
        8,
    )
}

fn qq_auth_activity(scheme: &str) -> String {
    use crate::commands::android::types::indent_manifest_fragment;
    indent_manifest_fragment(
        &format!(
            r#"<activity android:name="com.tencent.tauth.AuthActivity" android:launchMode="singleTask" android:noHistory="true">
    <intent-filter>
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data android:scheme="{}" />
    </intent-filter>
</activity>"#,
            scheme
        ),
        8,
    )
}

fn qq_assist_activity() -> String {
    use crate::commands::android::types::indent_manifest_fragment;
    indent_manifest_fragment(
        r#"<activity android:name="com.tencent.connect.common.AssistActivity" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:configChanges="keyboardHidden|orientation" android:screenOrientation="behind" />"#,
        8,
    )
}

// ===== 依赖排除渲染 =====

pub fn render_dependency_excludes_impl(_extra_dependencies: &str) -> String {
    use crate::commands::android::types::render_dependency_excludes;
    render_dependency_excludes(_extra_dependencies)
}
