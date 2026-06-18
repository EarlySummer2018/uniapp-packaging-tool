//! iOS 签名描述文件、证书导入与 ExportOptions 生成。

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct MobileProvisionInfo {
    uuid: String,
    name: String,
    team_ids: Vec<String>,
    app_id_prefixes: Vec<String>,
    application_identifier: String,
    get_task_allow: bool,
    provisioned_devices_count: usize,
    provisions_all_devices: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MobileProvisionValidationMode {
    ProjectGeneration,
    IpaExport,
}

pub(super) fn install_mobileprovision(
    config: &crate::commands::project::ProjectConfig,
    mode: MobileProvisionValidationMode,
) -> Result<MobileProvisionInfo, String> {
    let src = PathBuf::from(&config.ios.provisioning_profile);
    if !src.exists() {
        return Err(format!("描述文件不存在: {}", src.display()));
    }
    let info = parse_mobileprovision(&src)?;
    validate_mobileprovision(&info, config, mode)?;
    let dest_dir = dirs::home_dir()
        .ok_or_else(|| "无法定位 HOME".to_string())?
        .join("Library/MobileDevice/Provisioning Profiles");
    crate::utils::fs::ensure_directory(&dest_dir)
        .map_err(|e| format!("创建描述文件安装目录失败 {}: {}", dest_dir.display(), e))?;
    let dest = dest_dir.join(format!("{}.mobileprovision", info.uuid));
    remove_file_if_exists(&dest)
        .map_err(|e| format!("清理旧描述文件失败 {}: {}", dest.display(), e))?;
    std::fs::copy(&src, &dest).map_err(|e| {
        format!(
            "安装描述文件失败: {} -> {}: {}",
            src.display(),
            dest.display(),
            e
        )
    })?;
    Ok(info)
}

fn remove_file_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
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
        plist::Value::String(export_options_method(&config.ios.export_method).to_string()),
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
    let entitlements = dict
        .get("Entitlements")
        .and_then(|value| value.as_dictionary());
    let get_task_allow = entitlements
        .and_then(|entitlements| plist_bool(entitlements, "get-task-allow"))
        .unwrap_or(false);
    let provisioned_devices_count = dict
        .get("ProvisionedDevices")
        .and_then(|value| value.as_array())
        .map(Vec::len)
        .unwrap_or(0);
    let provisions_all_devices = plist_bool(dict, "ProvisionsAllDevices").unwrap_or(false);
    Ok(MobileProvisionInfo {
        uuid,
        name,
        team_ids,
        app_id_prefixes,
        application_identifier,
        get_task_allow,
        provisioned_devices_count,
        provisions_all_devices,
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

fn plist_bool(dict: &plist::Dictionary, key: &str) -> Option<bool> {
    match dict.get(key) {
        Some(plist::Value::Boolean(value)) => Some(*value),
        _ => None,
    }
}

fn validate_mobileprovision(
    info: &MobileProvisionInfo,
    config: &crate::commands::project::ProjectConfig,
    mode: MobileProvisionValidationMode,
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
    if mode == MobileProvisionValidationMode::IpaExport {
        validate_mobileprovision_export_method(info, &config.ios.export_method)?;
    }
    Ok(())
}

fn validate_mobileprovision_export_method(
    info: &MobileProvisionInfo,
    export_method: &str,
) -> Result<(), String> {
    match export_method {
        "app-store" | "app-store-connect" => {
            if info.get_task_allow || info.provisioned_devices_count > 0 {
                return Err(format!(
                    "iOS 导出方式为 App Store，但描述文件「{}」是开发或 Ad Hoc 描述文件（get-task-allow={}, 设备数={}）。请改用 App Store 分发描述文件和匹配的 Apple Distribution 证书，或将导出方式改为 Development/Ad Hoc。",
                    info.name, info.get_task_allow, info.provisioned_devices_count
                ));
            }
        }
        "development" => {
            if !info.get_task_allow {
                return Err(format!(
                    "iOS 导出方式为 Development，但描述文件「{}」不是开发描述文件（get-task-allow=false）。请改用 Development 描述文件，或调整导出方式。",
                    info.name
                ));
            }
        }
        "ad-hoc" => {
            if info.get_task_allow || info.provisioned_devices_count == 0 {
                return Err(format!(
                    "iOS 导出方式为 Ad Hoc，但描述文件「{}」不是 Ad Hoc 分发描述文件（get-task-allow={}, 设备数={}）。请改用 Ad Hoc 描述文件，或调整导出方式。",
                    info.name, info.get_task_allow, info.provisioned_devices_count
                ));
            }
        }
        "enterprise" => {
            if info.get_task_allow || !info.provisions_all_devices {
                return Err(format!(
                    "iOS 导出方式为 Enterprise，但描述文件「{}」不是企业分发描述文件（get-task-allow={}, ProvisionsAllDevices={}）。请改用 Enterprise 描述文件，或调整导出方式。",
                    info.name, info.get_task_allow, info.provisions_all_devices
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn export_options_method(method: &str) -> &str {
    match method {
        "app-store" => "app-store-connect",
        value => value,
    }
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
    fn remove_existing_mobileprovision_even_when_readonly() {
        let root =
            std::env::temp_dir().join(format!("unipack-mobileprovision-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let profile = root.join("PROFILE.mobileprovision");
        std::fs::write(&profile, "profile").unwrap();
        let mut permissions = std::fs::metadata(&profile).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&profile, permissions).unwrap();

        remove_file_if_exists(&profile).unwrap();

        assert!(!profile.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn mobileprovision_bundle_pattern_strips_team_prefix() {
        let info = MobileProvisionInfo {
            uuid: "UUID".into(),
            name: "Profile".into(),
            team_ids: vec!["TEAM123".into()],
            app_id_prefixes: vec![],
            application_identifier: "TEAM123.com.example.app".into(),
            get_task_allow: false,
            provisioned_devices_count: 0,
            provisions_all_devices: false,
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
        entitlements.insert("get-task-allow".into(), plist::Value::Boolean(true));
        root.insert(
            "Entitlements".into(),
            plist::Value::Dictionary(entitlements),
        );
        root.insert(
            "ProvisionedDevices".into(),
            plist::Value::Array(vec![plist::Value::String("DEVICE1".into())]),
        );
        let info = mobileprovision_info_from_plist(&plist::Value::Dictionary(root)).unwrap();
        assert_eq!(info.uuid, "PROFILE-UUID");
        assert_eq!(info.specifier(), "Profile Name");
        assert_eq!(info.bundle_pattern().as_deref(), Some("com.example.app"));
        assert!(info.get_task_allow);
        assert_eq!(info.provisioned_devices_count, 1);
    }

    #[test]
    fn app_store_export_rejects_development_profile() {
        let info = MobileProvisionInfo {
            uuid: "UUID".into(),
            name: "Development Profile".into(),
            team_ids: vec!["TEAM123".into()],
            app_id_prefixes: vec![],
            application_identifier: "TEAM123.com.example.app".into(),
            get_task_allow: true,
            provisioned_devices_count: 2,
            provisions_all_devices: false,
        };

        let err = validate_mobileprovision_export_method(&info, "app-store").unwrap_err();

        assert!(err.contains("App Store"));
        assert!(err.contains("Development Profile"));
    }

    #[test]
    fn app_store_export_method_uses_xcode_current_name() {
        assert_eq!(export_options_method("app-store"), "app-store-connect");
        assert_eq!(
            export_options_method("app-store-connect"),
            "app-store-connect"
        );
        assert_eq!(export_options_method("ad-hoc"), "ad-hoc");
    }
}
