//! 模块补丁公共辅助函数
//!
//! 从 render_android_module_manifest_patches 中提取的、被多个模块共用的工具函数。

#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};

use crate::commands::android::types::indent_manifest_fragment;
use crate::commands::shared::module::types::AndroidModuleConfigModule;

/// 将权限字符串添加到集合中
pub(crate) fn add_permissions(permissions: &mut BTreeSet<String>, perms: &[&str]) {
    for p in perms {
        permissions.insert((*p).to_string());
    }
}

/// 将 application 条目（XML 片段）添加到集合中
pub(crate) fn add_application_entries(entries: &mut BTreeSet<String>, items: &[String]) {
    for item in items {
        entries.insert(item.clone());
    }
}

/// 检查模块 report 中是否有某字段的非空值
pub(crate) fn has_report_value(module: &AndroidModuleConfigModule, key: &str) -> bool {
    module
        .fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| field.value.as_deref())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

/// 从模块字段构建 placeholder 映射（key → "${key}"）
pub(crate) fn module_placeholders(module: &AndroidModuleConfigModule) -> HashMap<String, String> {
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

/// 从 placeholders 中取值，缺失时返回空字符串
pub(crate) fn placeholder_value(placeholders: &HashMap<String, String>, key: &str) -> String {
    placeholders.get(key).cloned().unwrap_or_default()
}

/// 生成 `<meta-data>` XML 片段（8 空格缩进）
pub(crate) fn meta_data(name: &str, value: &str) -> String {
    indent_manifest_fragment(
        &format!(
            r#"<meta-data android:name="{}" android:value="{}" />"#,
            name, value
        ),
        8,
    )
}

/// 生成缩进后的 XML 条目（通用，8 空格缩进）
pub(crate) fn service_entry(entry: &str) -> String {
    indent_manifest_fragment(entry, 8)
}

/// 微信 WXEntryActivity 声明（用于分享 / 登录回调）
pub(crate) fn wx_entry_activity(package_name: &str, scheme: &str) -> String {
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

/// QQ AuthActivity 声明（用于 QQ 分享 / 登录回调）
pub(crate) fn qq_auth_activity(scheme: &str) -> String {
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

/// QQ AssistActivity 声明
pub(crate) fn qq_assist_activity() -> String {
    indent_manifest_fragment(
        r#"<activity android:name="com.tencent.connect.common.AssistActivity" android:theme="@android:style/Theme.Translucent.NoTitleBar" android:configChanges="keyboardHidden|orientation" android:screenOrientation="behind" />"#,
        8,
    )
}
