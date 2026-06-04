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
                if let Some(value) = field
                    .value
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    insert_manifest_placeholder_entries(&mut entries, &field.key, value);
                }
            }
        }
    }

    if has_push_module {
        insert_push_manifest_placeholder_defaults(&mut entries, package_name);
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
    key: &str,
    value: &str,
) {
    for placeholder in manifest_placeholder_aliases(key) {
        target.insert(placeholder.to_string(), value.to_string());
    }
}

fn insert_push_manifest_placeholder_defaults(
    target: &mut BTreeMap<String, String>,
    package_name: &str,
) {
    for (placeholder, value) in [
        ("GETUI_APPID", ""),
        ("GY_APP_ID", ""),
        ("GT_INSTALL_CHANNEL", "HBuilder"),
        ("PUSH_APPID", ""),
        ("plus.unipush.appid", ""),
        ("plus.unipush.appkey", ""),
        ("PUSH_APPKEY", ""),
        ("plus.unipush.appsecret", ""),
        ("PUSH_APPSECRET", ""),
        ("apk.applicationId", package_name),
        ("XIAOMI_APP_ID", ""),
        ("XIAOMI_APP_KEY", ""),
        ("MEIZU_APP_ID", ""),
        ("MEIZU_APP_KEY", ""),
        ("HUAWEI_APP_ID", ""),
        ("OPPO_APP_KEY", ""),
        ("OPPO_APP_SECRET", ""),
        ("VIVO_APP_ID", ""),
        ("VIVO_APP_KEY", ""),
        ("HONOR_APP_ID", ""),
    ] {
        target
            .entry(placeholder.to_string())
            .or_insert_with(|| value.to_string());
    }
}

pub fn manifest_placeholder_aliases(key: &str) -> Vec<&'static str> {
    match key {
        "GETUI_APPID" => vec![
            "GETUI_APPID",
            "GY_APP_ID",
            "PUSH_APPID",
            "plus.unipush.appid",
        ],
        "plus.unipush.appkey" => vec!["plus.unipush.appkey", "PUSH_APPKEY"],
        "plus.unipush.appsecret" => vec!["plus.unipush.appsecret", "PUSH_APPSECRET"],
        _ => vec![Box::leak(key.to_owned().into_boxed_str())],
    }
}
