#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct AndroidRequiredAar {
    pub display_name: &'static str,
    pub exact_names: &'static [&'static str],
    pub versionless_prefixes: &'static [&'static str],
}

pub const ANDROID_REQUIRED_AARS: &[AndroidRequiredAar] = &[
    AndroidRequiredAar {
        display_name: "lib.5plus.base",
        exact_names: &["lib.5plus.base-release.aar"],
        versionless_prefixes: &["lib.5plus.base"],
    },
    AndroidRequiredAar {
        display_name: "android-gif-drawable",
        exact_names: &["lib.android-gif-drawable-release.aar"],
        versionless_prefixes: &["android-gif-drawable", "lib.android-gif-drawable"],
    },
    AndroidRequiredAar {
        display_name: "uniapp-v8",
        exact_names: &["uniapp-v8-release.aar"],
        versionless_prefixes: &["uniapp-v8"],
    },
    AndroidRequiredAar {
        display_name: "oaid",
        exact_names: &["lib.oaid.release.aar"],
        versionless_prefixes: &["oaid_sdk_", "lib.oaid"],
    },
    AndroidRequiredAar {
        display_name: "install-apk",
        exact_names: &["install-apk-release.aar"],
        versionless_prefixes: &["install-apk"],
    },
    AndroidRequiredAar {
        display_name: "breakpad",
        exact_names: &["lib.breakpad-release.aar"],
        versionless_prefixes: &["breakpad-build", "lib.breakpad"],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidSdkLayout {
    pub root: PathBuf,
    pub integrate_project_dir: PathBuf,
    pub libs_dir: PathBuf,
    pub assets_dir: PathBuf,
    /// SDK/src 目录，包含各模块的 Java 源文件（如 wxapi/WXEntryActivity.java）
    pub src_dir: PathBuf,
}

fn get_uni_pack_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
}

#[tauri::command]
pub async fn list_sdks(platform: Option<String>) -> Result<Vec<SdkInfo>, String> {
    let config = load_global_sdk_config_sync()?;
    let all = configured_sdk_infos(&config);
    match platform.as_deref() {
        Some("android") => Ok(all
            .into_iter()
            .filter(|sdk| sdk.platform == SdkPlatform::Android)
            .collect()),
        Some("ios") => Ok(all
            .into_iter()
            .filter(|sdk| sdk.platform == SdkPlatform::Ios)
            .collect()),
        Some("harmony") | Some("harmonyos") => Ok(all
            .into_iter()
            .filter(|sdk| sdk.platform == SdkPlatform::Harmony)
            .collect()),
        Some(_) => Err(format!(
            "Unknown platform: {}",
            platform.unwrap_or_default()
        )),
        None => Ok(all),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkInfo {
    pub name: String,
    pub version: String,
    pub path: String,
    pub platform: SdkPlatform,
    pub is_installed: bool,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SdkPlatform {
    Android,
    Ios,
    Harmony,
}

fn configured_sdk_infos(config: &GlobalSdkConfig) -> Vec<SdkInfo> {
    vec![
        configured_sdk_info(
            "Android 离线SDK",
            &config.dcloud_android_sdk_path,
            SdkPlatform::Android,
        ),
        configured_sdk_info("iOS 离线SDK", &config.dcloud_ios_sdk_path, SdkPlatform::Ios),
        configured_sdk_info(
            "Harmony 工程模板",
            &config.harmony_template_path,
            SdkPlatform::Harmony,
        ),
    ]
}

fn configured_sdk_info(name: &str, path: &str, platform: SdkPlatform) -> SdkInfo {
    let configured = !path.trim().is_empty();
    let exists = configured && Path::new(path).exists();
    SdkInfo {
        name: name.to_string(),
        version: if configured {
            "configured".to_string()
        } else {
            String::new()
        },
        path: path.to_string(),
        platform,
        is_installed: configured,
        is_valid: exists,
    }
}

#[tauri::command]
pub async fn install_sdk(_name: String, _version: String) -> Result<(), String> {
    Err(
        "SDK installation is not yet implemented. Please download from DCloud official website."
            .to_string(),
    )
}

#[tauri::command]
pub async fn uninstall_sdk(_name: String) -> Result<(), String> {
    Err("SDK uninstallation is not yet implemented.".to_string())
}

#[tauri::command]
pub async fn get_sdk_info(_name: String) -> Result<SdkInfo, String> {
    Err("SDK info query is not yet implemented.".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSdkEntry {
    pub platform: String,
    pub path: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSdkConfig {
    pub dcloud_android_sdk_path: String,
    pub dcloud_ios_sdk_path: String,
    pub harmony_template_path: String,
}

fn get_user_sdk_paths_file() -> PathBuf {
    get_uni_pack_home().join("user-sdk-paths.json")
}

pub fn load_global_sdk_config_sync() -> Result<GlobalSdkConfig, String> {
    let entries = load_user_sdk_entries()?;
    Ok(GlobalSdkConfig {
        dcloud_android_sdk_path: latest_sdk_path(&entries, "android"),
        dcloud_ios_sdk_path: latest_sdk_path(&entries, "ios"),
        harmony_template_path: latest_sdk_path(&entries, "harmony"),
    })
}

fn load_user_sdk_entries() -> Result<Vec<UserSdkEntry>, String> {
    let config_file = get_user_sdk_paths_file();
    if !config_file.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&config_file).map_err(|e| format!("读取失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析失败: {}", e))
}

fn save_user_sdk_entries(entries: &[UserSdkEntry]) -> Result<(), String> {
    let config_file = get_user_sdk_paths_file();
    if let Some(parent) = config_file.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    let json = serde_json::to_string_pretty(entries).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&config_file, json).map_err(|e| format!("写入失败: {}", e))
}

fn latest_sdk_path(entries: &[UserSdkEntry], platform: &str) -> String {
    entries
        .iter()
        .rev()
        .find(|entry| entry.platform == platform)
        .map(|entry| entry.path.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn get_global_sdk_config() -> Result<GlobalSdkConfig, String> {
    load_global_sdk_config_sync()
}

#[tauri::command]
pub async fn add_sdk_path(platform: String, path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    if !matches!(platform.as_str(), "android" | "ios" | "harmony") {
        return Err(format!("不支持的 SDK 类型: {}", platform));
    }

    let normalized_path = normalize_global_sdk_path(&platform, &p)?;

    let mut entries = load_user_sdk_entries()?;
    entries.retain(|e| e.platform != platform);

    entries.push(UserSdkEntry {
        platform,
        path: normalized_path.to_string_lossy().to_string(),
        added_at: chrono::Utc::now().to_rfc3339(),
    });

    save_user_sdk_entries(&entries)
}

pub fn normalize_global_sdk_path(platform: &str, path: &Path) -> Result<PathBuf, String> {
    match platform {
        "android" => {
            Ok(crate::commands::android::sdk_layout::resolve_android_sdk_layout(path)?.root)
        }
        "ios" => crate::commands::ios::sdk_layout::resolve_ios_sdk_root(path),
        "harmony" => Ok(canonicalize_or_self(path)),
        _ => return Err(format!("不支持的 SDK 类型: {}", platform)),
    }
}

// --- Generic utility functions used across platforms ---

fn generic_root_candidates(path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    push_unique_path(&mut candidates, path.to_path_buf());

    if let Some(parent) = path.parent() {
        push_unique_path(&mut candidates, parent.to_path_buf());
        if let Some(grandparent) = parent.parent() {
            push_unique_path(&mut candidates, grandparent.to_path_buf());
        }
    }

    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut children = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|child| child.is_dir())
                .collect::<Vec<_>>();
            children.sort();
            for child in children {
                push_unique_path(&mut candidates, child);
            }
        }
    }

    candidates
}

pub fn canonicalize_or_self(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

pub fn format_path_list(paths: &[PathBuf]) -> String {
    let mut labels = paths
        .iter()
        .take(12)
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    if paths.len() > 12 {
        labels.push(format!("另有 {} 个目录", paths.len() - 12));
    }
    labels.join(", ")
}

#[tauri::command]
pub async fn remove_sdk_path(path: String) -> Result<(), String> {
    let config_file = get_user_sdk_paths_file();
    if !config_file.exists() {
        return Ok(());
    }
    let mut entries = load_user_sdk_entries()?;
    entries.retain(|e| e.path != path);
    save_user_sdk_entries(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }

    fn write_android_sdk(root: &Path) {
        std::fs::create_dir_all(root.join("HBuilder-Integrate-AS/simpleDemo")).unwrap();
        let libs = root.join("SDK/libs");
        std::fs::create_dir_all(&libs).unwrap();
        for requirement in crate::commands::android::sdk_layout::ANDROID_REQUIRED_AARS {
            let name = requirement
                .exact_names
                .first()
                .copied()
                .or_else(|| requirement.versionless_prefixes.first().copied())
                .expect("Android SDK test requirement should have a file name");
            std::fs::write(libs.join(name), b"aar").unwrap();
        }
        std::fs::create_dir_all(root.join("SDK/assets")).unwrap();
    }

    #[test]
    fn normalizes_android_sdk_child_path_to_package_root() {
        let root = unique_temp_dir("unipack-normalize-android-sdk");
        write_android_sdk(&root);

        let normalized = normalize_global_sdk_path("android", &root.join("SDK/libs")).unwrap();

        assert_eq!(normalized, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn normalizes_ios_sdk_child_project_to_sdk_root() {
        let root = unique_temp_dir("unipack-normalize-ios-sdk");
        let project = root.join("HBuilder-HelloUniApp");
        std::fs::create_dir_all(project.join("HBuilder-Hello.xcodeproj")).unwrap();
        std::fs::create_dir_all(root.join("SDK/Libs")).unwrap();
        std::fs::create_dir_all(root.join("SDK/Bundles")).unwrap();

        let normalized = normalize_global_sdk_path("ios", &project).unwrap();

        assert_eq!(normalized, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }
}
