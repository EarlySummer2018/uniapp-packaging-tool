//! iOS 签名描述文件、证书导入与 ExportOptions 生成。

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct MobileProvisionInfo {
    uuid: String,
    name: String,
    team_ids: Vec<String>,
    app_id_prefixes: Vec<String>,
    application_identifier: String,
}

impl MobileProvisionInfo {
    pub(super) fn specifier(&self) -> String {
        if self.name.trim().is_empty() {
            self.uuid.clone()
        } else {
            self.name.clone()
        }
    }

    fn bundle_pattern(&self) -> Option<String> {
        let app_id = self.application_identifier.trim();
        if app_id.is_empty() {
            return None;
        }
        for prefix in self.team_ids.iter().chain(self.app_id_prefixes.iter()) {
            if let Some(rest) = app_id.strip_prefix(&format!("{}.", prefix)) {
                return Some(rest.to_string());
            }
        }
        app_id.split_once('.').map(|(_, rest)| rest.to_string())
    }
}

pub(super) fn install_mobileprovision(
    config: &crate::commands::project::ProjectConfig,
) -> Result<MobileProvisionInfo, String> {
    let src = PathBuf::from(&config.ios.provisioning_profile);
    if !src.exists() {
        return Err(format!("描述文件不存在: {}", src.display()));
    }
    let info = parse_mobileprovision(&src)?;
    validate_mobileprovision(&info, config)?;
    let dest_dir = dirs::home_dir()
        .ok_or_else(|| "无法定位 HOME".to_string())?
        .join("Library/MobileDevice/Provisioning Profiles");
    crate::utils::fs::ensure_directory(&dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(format!("{}.mobileprovision", info.uuid));
    std::fs::copy(&src, &dest).map_err(|e| format!("安装描述文件失败: {}", e))?;
    Ok(info)
}

pub(super) fn import_p12_certificate(
    config: &crate::commands::project::ProjectConfig,
) -> Result<(), String> {
    if config.ios.certificate.trim().is_empty() {
        return Ok(());
    }
    let cert = PathBuf::from(&config.ios.certificate);
    if !cert.exists() {
        return Err(format!("P12 证书不存在: {}", cert.display()));
    }
    let password_key = format!("{}-ios-certificate-password", config.id);
    let password = crate::utils::keychain::get_password(&password_key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Keychain 中缺少 iOS P12 证书密码".to_string())?;
    let output = std::process::Command::new("security")
        .arg("import")
        .arg(&cert)
        .arg("-P")
        .arg(password)
        .arg("-A")
        .output()
        .map_err(|e| format!("执行 security import 失败: {}", e))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("already exists") || stderr.contains("已存在") {
        return Ok(());
    }
    Err(format!("导入 P12 证书失败: {}", stderr.trim()))
}

pub(super) fn write_export_options(
    path: &Path,
    config: &crate::commands::project::ProjectConfig,
    profile: &MobileProvisionInfo,
) -> Result<(), String> {
    let mut dict = plist::Dictionary::new();
    dict.insert(
        "method".into(),
        plist::Value::String(config.ios.export_method.clone()),
    );
    dict.insert(
        "teamID".into(),
        plist::Value::String(config.ios.team_id.clone()),
    );
    dict.insert("signingStyle".into(), plist::Value::String("manual".into()));
    let mut profiles = plist::Dictionary::new();
    profiles.insert(
        config.ios.bundle_id.clone(),
        plist::Value::String(profile.specifier()),
    );
    dict.insert(
        "provisioningProfiles".into(),
        plist::Value::Dictionary(profiles),
    );
    plist::Value::Dictionary(dict)
        .to_file_xml(path)
        .map_err(|e| format!("写入 ExportOptions.plist 失败: {}", e))
}

fn parse_mobileprovision(path: &Path) -> Result<MobileProvisionInfo, String> {
    let output = std::process::Command::new("security")
        .args(["cms", "-D", "-i"])
        .arg(path)
        .output()
        .map_err(|e| format!("执行 security cms 解析描述文件失败: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "解析描述文件失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let value = plist::Value::from_reader(std::io::Cursor::new(output.stdout))
        .map_err(|e| format!("解析描述文件 plist 失败: {}", e))?;
    mobileprovision_info_from_plist(&value)
}

fn mobileprovision_info_from_plist(value: &plist::Value) -> Result<MobileProvisionInfo, String> {
    let dict = value
        .as_dictionary()
        .ok_or_else(|| "描述文件 plist 不是 dictionary".to_string())?;
    let uuid = plist_string(dict, "UUID")
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "描述文件缺少 UUID".to_string())?;
    let name = plist_string(dict, "Name").unwrap_or_else(|| uuid.clone());
    let team_ids = plist_string_array(dict, "TeamIdentifier");
    let app_id_prefixes = plist_string_array(dict, "ApplicationIdentifierPrefix");
    let application_identifier = dict
        .get("Entitlements")
        .and_then(|value| value.as_dictionary())
        .and_then(|entitlements| plist_string(entitlements, "application-identifier"))
        .ok_or_else(|| "描述文件缺少 Entitlements.application-identifier".to_string())?;
    Ok(MobileProvisionInfo {
        uuid,
        name,
        team_ids,
        app_id_prefixes,
        application_identifier,
    })
}

fn plist_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    dict.get(key).and_then(|value| match value {
        plist::Value::String(value) => Some(value.trim().to_string()),
        _ => None,
    })
}

fn plist_string_array(dict: &plist::Dictionary, key: &str) -> Vec<String> {
    match dict.get(key) {
        Some(plist::Value::Array(items)) => items
            .iter()
            .filter_map(|item| match item {
                plist::Value::String(value) => {
                    let value = value.trim();
                    (!value.is_empty()).then(|| value.to_string())
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn validate_mobileprovision(
    info: &MobileProvisionInfo,
    config: &crate::commands::project::ProjectConfig,
) -> Result<(), String> {
    if !info.team_ids.is_empty() && !info.team_ids.iter().any(|team| team == &config.ios.team_id) {
        return Err(format!(
            "描述文件 Team ID 不匹配：配置为 {}，描述文件为 {}",
            config.ios.team_id,
            info.team_ids.join(", ")
        ));
    }
    let pattern = info
        .bundle_pattern()
        .ok_or_else(|| "无法从描述文件提取 Bundle ID".to_string())?;
    if !bundle_id_matches_profile_pattern(&config.ios.bundle_id, &pattern) {
        return Err(format!(
            "描述文件 Bundle ID 不匹配：配置为 {}，描述文件为 {}",
            config.ios.bundle_id, pattern
        ));
    }
    Ok(())
}

fn bundle_id_matches_profile_pattern(bundle_id: &str, pattern: &str) -> bool {
    let bundle_id = bundle_id.trim();
    let pattern = pattern.trim();
    if pattern == "*" || pattern == bundle_id {
        return true;
    }
    pattern
        .strip_suffix(".*")
        .and_then(|prefix| bundle_id.strip_prefix(prefix))
        .map(|rest| rest.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobileprovision_bundle_pattern_strips_team_prefix() {
        let info = MobileProvisionInfo {
            uuid: "UUID".into(),
            name: "Profile".into(),
            team_ids: vec!["TEAM123".into()],
            app_id_prefixes: vec![],
            application_identifier: "TEAM123.com.example.app".into(),
        };
        assert_eq!(info.bundle_pattern().as_deref(), Some("com.example.app"));
    }

    #[test]
    fn profile_pattern_supports_wildcard_suffix() {
        assert!(bundle_id_matches_profile_pattern(
            "com.example.app",
            "com.example.*"
        ));
        assert!(!bundle_id_matches_profile_pattern(
            "com.other.app",
            "com.example.*"
        ));
    }

    #[test]
    fn mobileprovision_info_reads_required_fields() {
        let mut root = plist::Dictionary::new();
        root.insert("UUID".into(), plist::Value::String("PROFILE-UUID".into()));
        root.insert("Name".into(), plist::Value::String("Profile Name".into()));
        root.insert(
            "TeamIdentifier".into(),
            plist::Value::Array(vec![plist::Value::String("TEAM123".into())]),
        );
        let mut entitlements = plist::Dictionary::new();
        entitlements.insert(
            "application-identifier".into(),
            plist::Value::String("TEAM123.com.example.app".into()),
        );
        root.insert(
            "Entitlements".into(),
            plist::Value::Dictionary(entitlements),
        );
        let info = mobileprovision_info_from_plist(&plist::Value::Dictionary(root)).unwrap();
        assert_eq!(info.uuid, "PROFILE-UUID");
        assert_eq!(info.specifier(), "Profile Name");
        assert_eq!(info.bundle_pattern().as_deref(), Some("com.example.app"));
    }
}
