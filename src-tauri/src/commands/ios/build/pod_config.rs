use crate::commands::ios::modules::common::ios_object_value_normalized;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PodConfigSection {
    name: &'static str,
    values: Vec<(&'static str, String)>,
}

pub(super) fn write_ios_pod_config(
    project_root: &std::path::Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let content = render_ios_pod_config(manifest_info);
    std::fs::write(project_root.join("uniapp_config.rb"), content)
        .map_err(|e| format!("写入 uniapp_config.rb 失败: {}", e))
}

pub(super) fn render_ios_pod_config(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> String {
    let sections = manifest_info
        .and_then(|info| info.manifest_value.as_ref())
        .map(collect_pod_config_sections)
        .unwrap_or_default();
    render_sections(&sections)
}

fn collect_pod_config_sections(manifest: &serde_json::Value) -> Vec<PodConfigSection> {
    let mut sections = Vec::new();
    push_section(&mut sections, payment_wechat_section(manifest));
    push_section(&mut sections, payment_alipay_section(manifest));
    push_section(&mut sections, map_gaode_section(manifest));
    push_section(&mut sections, statistic_umeng_section(manifest));
    push_section(&mut sections, uniad_section(manifest));
    sections
}

fn payment_wechat_section(manifest: &serde_json::Value) -> Option<PodConfigSection> {
    let provider = manifest_provider(manifest, "weixin", Some("payment"))
        .or_else(|| manifest_provider(manifest, "wechat", Some("payment")))?;
    let mut values = Vec::new();
    if let Some(appid) = json_string_field(provider, &["appid", "appId", "app_id"]) {
        values.push(("appid", appid));
    }
    let universal_links = json_string_field(
        provider,
        &["universal_links", "universalLinks", "UniversalLinks"],
    )
    .or_else(|| universal_links(manifest).into_iter().next());
    if let Some(link) = universal_links {
        values.push(("universal_links", link));
    }
    section("payment_wechat", values)
}

fn payment_alipay_section(manifest: &serde_json::Value) -> Option<PodConfigSection> {
    let provider = manifest_provider(manifest, "alipay", Some("payment"))?;
    let scheme = json_string_field(provider, &["scheme", "returnUrl", "returnURL"])
        .map(|value| url_scheme_value(&value))
        .or_else(|| {
            json_string_field(provider, &["appid", "appId", "app_id"])
                .map(|appid| prefixed_scheme("ap", &appid))
        });
    section(
        "payment_alipay",
        scheme
            .map(|value| vec![("scheme", value)])
            .unwrap_or_default(),
    )
}

fn map_gaode_section(manifest: &serde_json::Value) -> Option<PodConfigSection> {
    let provider = manifest_provider(manifest, "amap", Some("map"))
        .or_else(|| manifest_provider(manifest, "gaode", Some("map")))?;
    let appkey = json_string_field(
        provider,
        &["appkey_ios", "apikey_ios", "appkey", "apikey", "key"],
    );
    section(
        "map_gaode",
        appkey
            .map(|value| vec![("appkey", value)])
            .unwrap_or_default(),
    )
}

fn statistic_umeng_section(manifest: &serde_json::Value) -> Option<PodConfigSection> {
    let provider = manifest_provider(manifest, "umeng", Some("statistic"))
        .or_else(|| manifest_provider(manifest, "umeng-ios", Some("statistic")))?;
    let mut values = Vec::new();
    if let Some(appkey) = json_string_field(provider, &["appkey_ios", "appkey"]) {
        values.push(("appkey", appkey));
    }
    if let Some(channel) = json_string_field(provider, &["channelid_ios", "channelid", "channel"]) {
        values.push(("channel", channel));
    }
    section("statistic_umeng", values)
}

fn uniad_section(manifest: &serde_json::Value) -> Option<PodConfigSection> {
    let mut values = Vec::new();
    let ios = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"));
    if let Some(market_channel) = ios.and_then(|value| {
        json_string_field(
            value,
            &["marketChannel", "market_channel", "UNIAD_MARKET_CHANNEL"],
        )
    }) {
        values.push(("market_channel", market_channel));
    }
    if let Some(ad_id) = ios
        .and_then(|value| json_string_field(value, &["dcloudAdId", "dcloud_ad_id", "adid"]))
        .or_else(|| {
            uni_ad_sdk_config(manifest)
                .and_then(|value| json_string_field(value, &["dcloudAdId", "dcloud_ad_id", "adid"]))
        })
    {
        values.push(("dcloud_ad_id", ad_id));
    }
    section("uniad", values)
}

fn section(name: &'static str, values: Vec<(&'static str, String)>) -> Option<PodConfigSection> {
    (!values.is_empty()).then_some(PodConfigSection { name, values })
}

fn push_section(sections: &mut Vec<PodConfigSection>, section: Option<PodConfigSection>) {
    if let Some(section) = section {
        sections.push(section);
    }
}

fn render_sections(sections: &[PodConfigSection]) -> String {
    let mut content =
        String::from("# Generated by Unipack for HBuilderX 5.13+ local Pod builds.\n");
    if sections.is_empty() {
        content.push_str("UNIAPP_PLIST_VALUES = {}.freeze\n");
        return content;
    }

    content.push_str("UNIAPP_PLIST_VALUES = {\n");
    for section in sections {
        content.push_str(&format!("  {}: {{\n", section.name));
        for (index, (key, value)) in section.values.iter().enumerate() {
            let comma = if index + 1 == section.values.len() {
                ""
            } else {
                ","
            };
            content.push_str(&format!(
                "    {}: '{}'{}\n",
                key,
                ruby_single_quoted(value),
                comma
            ));
        }
        content.push_str("  },\n");
    }
    content.push_str("}.freeze\n");
    content
}

fn manifest_provider<'a>(
    manifest: &'a serde_json::Value,
    provider: &str,
    category: Option<&str>,
) -> Option<&'a serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?;
    if let Some(category) = category {
        for alias in category_aliases(category) {
            if let Some(category_value) = find_object_value_normalized(sdk_configs, alias) {
                if let Some(provider_value) = find_object_value_normalized(category_value, provider)
                {
                    return Some(provider_value);
                }
            }
        }
        return None;
    }
    find_object_value_normalized(sdk_configs, provider)
}

fn uni_ad_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["ad", "ads", "uni-ad", "uniAD", "uniad"]
        .iter()
        .find_map(|key| ios_object_value_normalized(sdk_configs, key))
}

fn category_aliases(category: &str) -> &'static [&'static str] {
    match category {
        "map" | "maps" => &["maps", "map"],
        "payment" => &["payment", "payments"],
        "statistic" | "statistics" | "statics" => &["statistic", "statistics", "statics"],
        _ => &[],
    }
}

fn find_object_value_normalized<'a>(
    value: &'a serde_json::Value,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let map = value.as_object()?;
    ios_object_value_normalized(map, key)
}

fn json_string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(String::from)
    })
}

fn universal_links(manifest: &serde_json::Value) -> Vec<String> {
    let mut links = Vec::new();
    collect_values_for_key(manifest, "UniversalLinks", &mut links);
    dedup_non_empty_strings(links)
}

fn collect_values_for_key(value: &serde_json::Value, key: &str, output: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(values) => {
            for (name, value) in values {
                if name.eq_ignore_ascii_case(key) {
                    collect_json_strings(value, output);
                } else {
                    collect_values_for_key(value, key, output);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_values_for_key(value, key, output);
            }
        }
        _ => {}
    }
}

fn collect_json_strings(value: &serde_json::Value, output: &mut Vec<String>) {
    match value {
        serde_json::Value::String(value) => {
            output.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(String::from),
            );
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_json_strings(value, output);
            }
        }
        _ => {}
    }
}

fn dedup_non_empty_strings(values: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() && !result.iter().any(|existing| existing == value) {
            result.push(value.to_string());
        }
    }
    result
}

fn prefixed_scheme(prefix: &str, value: &str) -> String {
    if value.starts_with(prefix) {
        value.to_string()
    } else {
        format!("{}{}", prefix, value)
    }
}

fn url_scheme_value(value: &str) -> String {
    value
        .split_once("://")
        .map(|(scheme, _)| scheme)
        .unwrap_or(value)
        .trim_matches('/')
        .to_string()
}

fn ruby_single_quoted(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
