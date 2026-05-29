use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreDetail {
    pub path: String,
    pub alias: String,
    pub sha1: String,
    pub sha256: String,
    pub md5: String,
    pub valid_until: String,
    pub algorithm: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosCertInfo {
    pub common_name: String,
    pub serial_number: String,
    pub expires: String,
    pub team_name: String,
    pub is_development: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosProfileInfo {
    pub name: String,
    pub uuid: String,
    pub team_name: String,
    pub app_id_prefix: String,
    pub bundle_identifier: String,
    pub expiration_date: String,
    pub platform: String,
    pub is_ad_hoc: bool,
    pub is_dev: bool,
    pub is_app_store: bool,
    pub is_enterprise: bool,
}

fn get_certificates_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
        .join("certificates")
}

#[tauri::command]
pub async fn analyze_android_keystore(
    keystore_path: String,
    password: String,
) -> Result<KeystoreDetail, String> {
    let keytool = resolve_configured_keytool()?;
    let output = Command::new(&keytool)
        .args(["-v", "-list", "-keystore", &keystore_path])
        .arg("-storepass")
        .arg(&password)
        .output()
        .map_err(|e| format!("执行 keytool 失败: {} ({})", e, keytool.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("keytool failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let alias = extract_keytool_value(&stdout, "Alias name:")
        .unwrap_or_else(|| "unknown".to_string())
        .trim_start_matches(':')
        .trim()
        .to_string();

    let sha1 = extract_keytool_value(&stdout, "SHA1:").unwrap_or_default();
    let sha256 = extract_keytool_value(&stdout, "SHA256:").unwrap_or_default();
    let md5 = extract_keytool_value(&stdout, "MD5:").unwrap_or_default();
    let valid_until = extract_keytool_value(&stdout, "until: ").unwrap_or_default();
    let algorithm =
        extract_keytool_value(&stdout, "Signature algorithm name: ").unwrap_or_default();
    let issuer = extract_keytool_block(&stdout, "Issuer: ");

    Ok(KeystoreDetail {
        path: keystore_path,
        alias,
        sha1,
        sha256,
        md5,
        valid_until,
        algorithm,
        issuer,
    })
}

fn extract_keytool_value(output: &str, prefix: &str) -> Option<String> {
    output
        .lines()
        .find(|l| l.trim().starts_with(prefix))
        .and_then(|l| l.trim().strip_prefix(prefix).map(|s| s.trim().to_string()))
}

fn extract_keytool_block(output: &str, prefix: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let start_idx = lines
        .iter()
        .position(|l| l.trim().starts_with(prefix))
        .unwrap_or(0);
    let end_idx = lines[start_idx..]
        .iter()
        .position(|l| l.is_empty())
        .unwrap_or(lines.len() - start_idx - 1);
    lines[start_idx..start_idx + end_idx]
        .iter()
        .map(|l| l.trim().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[tauri::command]
pub async fn generate_android_keystore(
    output_path: String,
    alias: String,
    password: String,
    validity_years: u32,
    dname: HashMap<String, String>,
) -> Result<String, String> {
    let parent_dir = PathBuf::from(&output_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    if !parent_dir.is_empty() {
        std::fs::create_dir_all(&parent_dir)
            .map_err(|e| format!("Failed to create directory {}: {}", parent_dir, e))?;
    }

    let dname_str = build_dname_string(&dname);

    let validity_days = validity_years * 365;

    let keytool = resolve_configured_keytool()?;
    let result = Command::new(&keytool)
        .args([
            "-genkeypair",
            "-alias",
            &alias,
            "-keyalg",
            "RSA",
            "-keysize",
            "2048",
            "-validity",
            &validity_days.to_string(),
            "-keystore",
            &output_path,
            "-storepass",
            &password,
            "-keypass",
            &password,
            "-dname",
            &dname_str,
        ])
        .output()
        .map_err(|e| format!("执行 keytool 失败: {} ({})", e, keytool.display()))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("keytool genkeypair failed: {}", stderr.trim()));
    }

    let cert_dir = get_certificates_dir();
    let android_dir = cert_dir.join("android");
    std::fs::create_dir_all(&android_dir).ok();

    Ok(output_path)
}

fn resolve_configured_keytool() -> Result<PathBuf, String> {
    crate::commands::env::resolve_configured_tool_bin_with_candidates("java", keytool_bin_names())
}

fn keytool_bin_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["keytool.exe", "keytool"]
    } else {
        &["keytool"]
    }
}

fn build_dname_string(dname: &HashMap<String, String>) -> String {
    let order = ["CN", "OU", "O", "L", "ST", "C"];
    let parts: Vec<String> = order
        .iter()
        .filter_map(|key| dname.get(*key).map(|v| format!("{}={}", key, v)))
        .collect();
    parts.join(",")
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn list_ios_certificates() -> Result<Vec<IosCertInfo>, String> {
    let output = Command::new("security")
        .args(["find-identity", "-v", "-p", "codesigning"])
        .output()
        .map_err(|e| format!("Failed to execute security: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut certs = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("1)")
            || trimmed.starts_with("2)")
            || trimmed.starts_with("3)")
            || trimmed.starts_with("4)")
            || trimmed.starts_with("5)")
            || trimmed.starts_with("6)")
            || trimmed.starts_with("7)")
            || trimmed.starts_with("8)")
            || trimmed.starts_with("9)")
            || trimmed.starts_with("10)")
        {
            if let Some(hash_pos) = trimmed.find(')') {
                let rest = &trimmed[hash_pos + 1..];
                if rest.starts_with("\"") {
                    if let Some(end_quote) = rest[1..].find('"') {
                        let common_name = &rest[1..=end_quote];
                        let hash = rest[end_quote + 2..].trim();
                        let is_development = common_name.contains("Development")
                            || common_name.contains("development");
                        certs.push(IosCertInfo {
                            common_name: common_name.to_string(),
                            serial_number: hash.to_string(),
                            expires: String::new(),
                            team_name: String::new(),
                            is_development,
                        });
                    }
                }
            }
        }
    }

    Ok(certs)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn list_ios_certificates() -> Result<Vec<IosCertInfo>, String> {
    Err("iOS certificate listing is only supported on macOS".to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn list_ios_provisioning_profiles() -> Result<Vec<IosProfileInfo>, String> {
    let profiles_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("MobileDevice")
        .join("Provisioning Profiles");

    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&profiles_dir)
        .map_err(|e| format!("Failed to read provisioning profiles directory: {}", e))?;

    let mut profiles = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .map(|e| e == "mobileprovision")
            .unwrap_or(false)
        {
            match parse_mobile_provision(&path) {
                Ok(profile) => profiles.push(profile),
                Err(_) => continue,
            }
        }
    }

    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

#[cfg(target_os = "macos")]
fn parse_mobile_provision(path: &std::path::Path) -> Result<IosProfileInfo, String> {
    let output = Command::new("security")
        .args(["cms", "-D", "-i", &path.to_string_lossy()])
        .output()
        .map_err(|e| format!("Failed to decode mobileprovision: {}", e))?;

    if !output.status.success() {
        return Err("Failed to decode mobileprovision file".to_string());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let plist: serde_json::Value = plist::from_bytes(content.as_bytes())
        .map_err(|e| format!("Failed to parse mobileprovision plist: {}", e))?;

    let name = plist
        .get("Name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let team_name = plist
        .get("TeamName")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let app_id_prefix = plist
        .get("ApplicationIdentifierPrefix")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let bundle_identifier = plist
        .get("Entitlements")
        .and_then(|e| e.get("application-identifier"))
        .and_then(|v| v.as_str())
        .unwrap_or("*")
        .to_string();
    let expiration_date = plist
        .get("ExpirationDate")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let platform = plist
        .get("Platform")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let profile_type = plist.get("Name").and_then(|v| v.as_str()).unwrap_or("");

    let is_ad_hoc = profile_type.contains("AdHoc") || profile_type.contains("ad hoc");
    let is_dev = profile_type.contains("Development") || profile_type.contains("development");
    let is_app_store = profile_type.contains("AppStore") || profile_type.contains("App Store");
    let is_enterprise = profile_type.contains("Enterprise") || profile_type.contains("InHouse");

    let uuid = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(IosProfileInfo {
        name,
        uuid,
        team_name,
        app_id_prefix,
        bundle_identifier,
        expiration_date,
        platform,
        is_ad_hoc,
        is_dev,
        is_app_store,
        is_enterprise,
    })
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn list_ios_provisioning_profiles() -> Result<Vec<IosProfileInfo>, String> {
    Err("iOS provisioning profiles listing is only supported on macOS".to_string())
}
