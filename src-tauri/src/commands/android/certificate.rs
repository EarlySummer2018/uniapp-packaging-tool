//! Android Keystore 证书管理

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
    crate::commands::shared::env::resolve_configured_tool_bin_with_candidates(
        "java",
        keytool_bin_names(),
    )
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

fn get_certificates_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
        .join("certificates")
}
