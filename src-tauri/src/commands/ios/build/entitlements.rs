use std::path::{Path, PathBuf};

use super::fs_utils::find_file_with_ext_skipping_bundles;
use super::plist::{collect_json_strings, dedup_non_empty_strings, universal_links};

fn ios_manifest_associated_domains(manifest: &serde_json::Value) -> Vec<String> {
    let mut domains = Vec::new();
    if let Some(value) = manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("capabilities"))
        .and_then(|value| value.get("entitlements"))
        .and_then(|value| value.get("com.apple.developer.associated-domains"))
    {
        collect_json_strings(value, &mut domains);
    }
    dedup_non_empty_strings(
        domains
            .into_iter()
            .filter_map(|domain| normalize_associated_domain(&domain))
            .collect(),
    )
}

fn normalize_associated_domain(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(domain) = universal_link_domain(value) {
        return Some(format!("applinks:{}", domain));
    }
    if value.contains(':') {
        Some(value.to_string())
    } else {
        Some(format!("applinks:{}", value))
    }
}

pub(super) fn patch_ios_entitlements(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<usize, String> {
    let Some(manifest) = manifest_info.and_then(|info| info.manifest_value.as_ref()) else {
        return Ok(0);
    };
    let mut domains = ios_manifest_associated_domains(manifest);
    domains.extend(
        universal_links(manifest)
            .into_iter()
            .filter_map(|link| universal_link_domain(&link))
            .map(|domain| format!("applinks:{}", domain)),
    );
    let domains = dedup_non_empty_strings(domains);
    if domains.is_empty() {
        return Ok(0);
    }
    let entitlements = find_entitlements(project_root, project_file).ok_or_else(|| {
        "manifest 配置了 UniversalLinks，但 iOS 工程中未找到 entitlements 文件".to_string()
    })?;
    let mut value = plist::Value::from_file(&entitlements).map_err(|e| {
        format!(
            "解析 iOS entitlements 失败 {}: {}",
            entitlements.display(),
            e
        )
    })?;
    let dict = value.as_dictionary_mut().ok_or_else(|| {
        format!(
            "iOS entitlements 不是 dictionary: {}",
            entitlements.display()
        )
    })?;
    dict.insert(
        "com.apple.developer.associated-domains".into(),
        plist::Value::Array(domains.iter().cloned().map(plist::Value::String).collect()),
    );
    value.to_file_xml(&entitlements).map_err(|e| {
        format!(
            "写入 iOS entitlements 失败 {}: {}",
            entitlements.display(),
            e
        )
    })?;
    Ok(domains.len())
}

fn universal_link_domain(link: &str) -> Option<String> {
    let value = link
        .trim()
        .strip_prefix("https://")
        .or_else(|| link.trim().strip_prefix("http://"))?;
    value
        .split('/')
        .next()
        .map(|domain| domain.split(':').next().unwrap_or(domain).trim())
        .filter(|domain| !domain.is_empty())
        .map(String::from)
}

fn find_entitlements(project_root: &Path, project_file: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(project_file.join("project.pbxproj")).ok()?;
    let pattern = regex::Regex::new(r#"CODE_SIGN_ENTITLEMENTS = "?([^";]+)"?;"#).ok()?;
    for capture in pattern.captures_iter(&content) {
        let relative = capture.get(1)?.as_str().trim_matches('"');
        let candidate = project_root.join(relative);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    find_file_with_ext_skipping_bundles(project_root, "entitlements")
}
