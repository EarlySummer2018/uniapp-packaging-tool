//! Android Manifest Placeholder 渲染

use std::collections::BTreeMap;

use crate::commands::android::types::escape_gradle_string;

pub fn render_android_module_manifest_placeholders(
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    detected_modules: &[crate::commands::resource::DetectedModule],
    package_name: &str,
) -> String {
    let mut entries = BTreeMap::new();
    let mut has_push_module = detected_modules
        .iter()
        .any(|module| super::artifacts::android_module_template_key(&module.name) == Some("push"));

    if let Some(report) = report {
        for module in &report.modules {
            if module.template_key == "push" {
                has_push_module = true;
            }
            for field in &module.fields {
                if field.field_type == "file" {
                    continue;
                }
                if let Some(value) = field
                    .value
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    insert_manifest_placeholder_entries(
                        &mut entries,
                        &module.template_key,
                        &field.key,
                        value,
                    );
                }
            }
        }
    }

    if has_push_module {
        insert_push_manifest_placeholder_defaults(&mut entries, package_name);
    }
    if let Some(report) = report {
        insert_univerify_manifest_placeholder_overrides(&mut entries, report);
    }

    if entries.is_empty() {
        return String::new();
    }

    let entries = entries
        .into_iter()
        .map(|(key, value)| {
            format!(
                "            \"{}\": \"{}\"",
                key,
                escape_gradle_string(&value)
            )
        })
        .collect::<Vec<_>>();
    format!(
        "\n        manifestPlaceholders = [\n{}\n        ]",
        entries.join(",\n")
    )
}

fn insert_manifest_placeholder_entries(
    target: &mut BTreeMap<String, String>,
    template_key: &str,
    key: &str,
    value: &str,
) {
    for placeholder in manifest_placeholder_aliases(template_key, key) {
        target.insert(placeholder.to_string(), value.to_string());
    }
}

fn insert_push_manifest_placeholder_defaults(
    target: &mut BTreeMap<String, String>,
    package_name: &str,
) {
    for (placeholder, value) in [
        ("GETUI_APPID", ""),
        ("GT_INSTALL_CHANNEL", "HBuilder"),
        ("PUSH_APPID", ""),
        ("plus.unipush.appid", ""),
        ("plus.unipush.appkey", ""),
        ("PUSH_APPKEY", ""),
        ("plus.unipush.appsecret", ""),
        ("PUSH_APPSECRET", ""),
        ("apk.applicationId", package_name),
    ] {
        target
            .entry(placeholder.to_string())
            .or_insert_with(|| value.to_string());
    }
}

fn insert_univerify_manifest_placeholder_overrides(
    target: &mut BTreeMap<String, String>,
    report: &crate::commands::module::AndroidModuleConfigReport,
) {
    let Some(value) = report_field_value(report, "login", "GY_APP_ID") else {
        return;
    };
    target.insert("GETUI_APPID".to_string(), value.clone());
    target.insert("GY_APP_ID".to_string(), value);
    target
        .entry("GT_INSTALL_CHANNEL".to_string())
        .or_insert_with(|| "HBuilder".to_string());
}

fn report_field_value(
    report: &crate::commands::module::AndroidModuleConfigReport,
    template_key: &str,
    field_key: &str,
) -> Option<String> {
    report
        .modules
        .iter()
        .find(|module| module.template_key == template_key)
        .and_then(|module| {
            module
                .fields
                .iter()
                .find(|field| field.key == field_key)
                .and_then(|field| field.value.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
}

pub fn manifest_placeholder_aliases<'a>(template_key: &'a str, key: &'a str) -> Vec<&'a str> {
    match (template_key, key) {
        ("push", "GETUI_APPID") => vec!["GETUI_APPID", "PUSH_APPID", "plus.unipush.appid"],
        ("push", "plus.unipush.appkey") => vec!["plus.unipush.appkey", "PUSH_APPKEY"],
        ("push", "plus.unipush.appsecret") => vec!["plus.unipush.appsecret", "PUSH_APPSECRET"],
        ("login", "GY_APP_ID") => vec!["GY_APP_ID", "GETUI_APPID"],
        ("login", "WX_APPID") | ("share", "WX_APPID") => vec!["WX_APPID", "plus.weixin.appid"],
        ("map", "GOOGLE_MAPS_API_KEY") => {
            vec!["GOOGLE_MAPS_API_KEY", "plus.google_map.APIKey_android"]
        }
        ("map", "MAP_PAGE_TYPE") => vec![],
        ("payment", "PAYPAL_RETURN_SCHEME") => {
            vec!["PAYPAL_RETURN_SCHEME", "plus.paypal.returnUrl", "returnUrl"]
        }
        _ => vec![key],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::shared::module::types::{
        AndroidModuleConfigField, AndroidModuleConfigModule, AndroidModuleConfigReport,
    };

    fn push_report(fields: Vec<AndroidModuleConfigField>) -> AndroidModuleConfigReport {
        AndroidModuleConfigReport {
            modules: vec![AndroidModuleConfigModule {
                name: "Push".to_string(),
                template_key: "push".to_string(),
                category: "push".to_string(),
                platforms: vec!["android".to_string()],
                source: "test".to_string(),
                fields,
            }],
            missing_required: Vec::new(),
            all_configured: true,
        }
    }

    fn text_field(key: &str, value: &str) -> AndroidModuleConfigField {
        AndroidModuleConfigField {
            key: key.to_string(),
            label: key.to_string(),
            required: false,
            secret: false,
            value: Some(value.to_string()),
            value_source: Some("test".to_string()),
            placeholder: String::new(),
            field_type: "text".to_string(),
        }
    }

    fn file_field(key: &str, value: &str) -> AndroidModuleConfigField {
        AndroidModuleConfigField {
            field_type: "file".to_string(),
            ..text_field(key, value)
        }
    }

    #[test]
    fn push_defaults_do_not_include_unselected_vendor_placeholders() {
        let report = push_report(vec![
            text_field("GETUI_APPID", "getui-appid"),
            text_field("plus.unipush.appkey", "getui-key"),
            text_field("plus.unipush.appsecret", "getui-secret"),
        ]);

        let rendered =
            render_android_module_manifest_placeholders(Some(&report), &[], "com.example.app");

        assert!(rendered.contains("\"GETUI_APPID\": \"getui-appid\""));
        assert!(rendered.contains("\"apk.applicationId\": \"com.example.app\""));
        assert!(!rendered.contains("XIAOMI_APP_ID"));
        assert!(!rendered.contains("MEIZU_APP_ID"));
    }

    #[test]
    fn push_placeholders_keep_selected_vendor_values_and_skip_file_fields() {
        let report = push_report(vec![
            text_field("GETUI_APPID", "getui-appid"),
            text_field("OPPO_APP_KEY", "oppo-key"),
            file_field("HUAWEI_AGCONNECT_JSON", "{\"client\":{}}"),
        ]);

        let rendered =
            render_android_module_manifest_placeholders(Some(&report), &[], "com.example.app");

        assert!(rendered.contains("\"OPPO_APP_KEY\": \"oppo-key\""));
        assert!(!rendered.contains("HUAWEI_AGCONNECT_JSON"));
        assert!(!rendered.contains("{\\\"client\\\""));
    }

    #[test]
    fn push_and_univerify_placeholders_do_not_leave_empty_getui_appid() {
        let report = AndroidModuleConfigReport {
            modules: vec![
                AndroidModuleConfigModule {
                    name: "Push".to_string(),
                    template_key: "push".to_string(),
                    category: "push".to_string(),
                    platforms: vec!["android".to_string()],
                    source: "test".to_string(),
                    fields: vec![
                        text_field("GETUI_APPID", "push-appid"),
                        text_field("plus.unipush.appkey", "push-appkey"),
                        text_field("plus.unipush.appsecret", "push-secret"),
                    ],
                },
                AndroidModuleConfigModule {
                    name: "OAuth".to_string(),
                    template_key: "login".to_string(),
                    category: "oauth".to_string(),
                    platforms: vec!["android".to_string()],
                    source: "test".to_string(),
                    fields: vec![text_field("GY_APP_ID", "univerify-appid")],
                },
            ],
            missing_required: Vec::new(),
            all_configured: true,
        };

        let rendered =
            render_android_module_manifest_placeholders(Some(&report), &[], "com.example.app");

        assert!(rendered.contains("\"GETUI_APPID\": \"univerify-appid\""));
        assert!(rendered.contains("\"GY_APP_ID\": \"univerify-appid\""));
        assert!(rendered.contains("\"plus.unipush.appid\": \"push-appid\""));
        assert!(rendered.contains("\"PUSH_APPID\": \"push-appid\""));
        assert!(!rendered.contains("\"GETUI_APPID\": \"\""));
    }
}
