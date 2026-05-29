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
        exact_names: &[
            "android-gif-drawable-release@1.2.23.aar",
            "lib.android-gif-drawable-release.aar",
        ],
        versionless_prefixes: &["android-gif-drawable", "lib.android-gif-drawable"],
    },
    AndroidRequiredAar {
        display_name: "uniapp-v8",
        exact_names: &["uniapp-v8-release.aar"],
        versionless_prefixes: &["uniapp-v8"],
    },
    AndroidRequiredAar {
        display_name: "oaid",
        exact_names: &["oaid_sdk_1.0.25.aar", "lib.oaid.release.aar"],
        versionless_prefixes: &["oaid_sdk_", "lib.oaid"],
    },
    AndroidRequiredAar {
        display_name: "install-apk",
        exact_names: &["install-apk-release.aar"],
        versionless_prefixes: &["install-apk"],
    },
    AndroidRequiredAar {
        display_name: "breakpad",
        exact_names: &["breakpad-build-release.aar", "lib.breakpad-release.aar"],
        versionless_prefixes: &["breakpad-build", "lib.breakpad"],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidSdkLayout {
    pub root: PathBuf,
    pub libs_dir: PathBuf,
    pub assets_dir: PathBuf,
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
        path: canonicalize_or_self(&normalized_path)
            .to_string_lossy()
            .to_string(),
        added_at: chrono::Utc::now().to_rfc3339(),
    });

    save_user_sdk_entries(&entries)
}

fn normalize_global_sdk_path(platform: &str, path: &Path) -> Result<PathBuf, String> {
    match platform {
        "android" => Ok(resolve_android_sdk_layout(path)?.root),
        "ios" => resolve_ios_sdk_root(path),
        "harmony" => resolve_harmony_template_root(path),
        _ => return Err(format!("不支持的 SDK 类型: {}", platform)),
    }
}

pub fn resolve_android_sdk_layout(path: &Path) -> Result<AndroidSdkLayout, String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    let candidates = android_sdk_root_candidates(path);
    let mut checked_libs = Vec::new();
    let mut missing_reports = Vec::new();

    for root in candidates {
        if let Some(layout) = android_layout_from_root(&root) {
            push_unique_path(&mut checked_libs, layout.libs_dir.clone());
            let missing = missing_android_required_aars(&layout.libs_dir);
            if missing.is_empty() {
                return Ok(AndroidSdkLayout {
                    root: canonicalize_or_self(&layout.root),
                    libs_dir: canonicalize_or_self(&layout.libs_dir),
                    assets_dir: canonicalize_or_self(&layout.assets_dir),
                });
            }
            missing_reports.push(format!(
                "{} 缺少 {}",
                layout.libs_dir.display(),
                format_missing_android_aars(&missing)
            ));
        } else {
            push_unique_path(&mut checked_libs, root.join("SDK").join("libs"));
            push_unique_path(&mut checked_libs, root.join("libs"));
        }
    }

    if missing_reports.is_empty() {
        Err(format!(
            "未找到 DCloud Android 离线 SDK 的 libs 目录。已检查: {}",
            format_path_list(&checked_libs)
        ))
    } else {
        Err(format!(
            "DCloud Android 离线 SDK 缺少核心 AAR。已检查: {}。缺少: {}",
            format_path_list(&checked_libs),
            missing_reports.join("; ")
        ))
    }
}

pub fn resolve_ios_sdk_root(path: &Path) -> Result<PathBuf, String> {
    let project = resolve_ios_sdk_project(path)?;
    let root = project.parent().unwrap_or(&project);
    Ok(canonicalize_or_self(root))
}

pub fn resolve_ios_sdk_project(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    let mut checked = Vec::new();
    for root in ios_sdk_root_candidates(path) {
        if is_ios_hello_project(&root) {
            return Ok(canonicalize_or_self(&root));
        }
        push_unique_path(&mut checked, root.clone());

        if let Some(project) = find_ios_hello_project_child(&root) {
            return Ok(canonicalize_or_self(&project));
        }
        push_unique_path(&mut checked, root.join("HBuilder-Hello*"));
    }

    Err(format!(
        "DCloud iOS 离线 SDK 中未找到 HBuilder-Hello* Xcode 工程。已检查: {}",
        format_path_list(&checked)
    ))
}

pub fn resolve_harmony_template_root(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    let mut checked = Vec::new();
    for root in generic_root_candidates(path) {
        if has_harmony_wrapper(&root) {
            return Ok(canonicalize_or_self(&root));
        }
        push_unique_path(&mut checked, root.join(harmony_wrapper_name()));
    }

    Err(format!(
        "Harmony 工程模板中未找到 {}。已检查: {}",
        harmony_wrapper_name(),
        format_path_list(&checked)
    ))
}

fn android_sdk_root_candidates(path: &Path) -> Vec<PathBuf> {
    let mut candidates = generic_root_candidates(path);
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

fn ios_sdk_root_candidates(path: &Path) -> Vec<PathBuf> {
    generic_root_candidates(path)
}

fn android_layout_from_root(root: &Path) -> Option<AndroidSdkLayout> {
    let sdk_libs = root.join("SDK").join("libs");
    if sdk_libs.is_dir() {
        return Some(AndroidSdkLayout {
            root: root.to_path_buf(),
            libs_dir: sdk_libs,
            assets_dir: root.join("SDK").join("assets"),
        });
    }

    let libs = root.join("libs");
    if libs.is_dir() {
        return Some(AndroidSdkLayout {
            root: root.to_path_buf(),
            libs_dir: libs,
            assets_dir: root.join("assets"),
        });
    }

    None
}

pub fn resolve_android_required_aar(
    libs_dir: &Path,
    requirement: &AndroidRequiredAar,
) -> Option<PathBuf> {
    for name in requirement.exact_names {
        let path = libs_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }

    let mut matches = std::fs::read_dir(libs_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            if path.extension().and_then(|ext| ext.to_str()) != Some("aar") {
                return false;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return false;
            };
            requirement
                .versionless_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

fn missing_android_required_aars(libs_dir: &Path) -> Vec<&'static AndroidRequiredAar> {
    ANDROID_REQUIRED_AARS
        .iter()
        .filter_map(|requirement| {
            let found = resolve_android_required_aar(libs_dir, requirement).is_some();
            (!found).then_some(requirement)
        })
        .collect()
}

fn format_missing_android_aars(missing: &[&AndroidRequiredAar]) -> String {
    missing
        .iter()
        .map(|requirement| {
            if requirement.versionless_prefixes.is_empty() {
                requirement.display_name.to_string()
            } else {
                format!(
                    "{}(文件名前缀: {})",
                    requirement.display_name,
                    requirement.versionless_prefixes.join(" 或 ")
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn find_ios_hello_project_child(root: &Path) -> Option<PathBuf> {
    let mut children = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|child| child.is_dir())
        .collect::<Vec<_>>();
    children.sort();
    children
        .into_iter()
        .find(|child| is_ios_hello_project(child))
}

fn is_ios_hello_project(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.starts_with("HBuilder-Hello") && has_xcode_project(path)
}

fn has_xcode_project(path: &Path) -> bool {
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            let entry_path = entry.path();
            matches!(
                entry_path.extension().and_then(|ext| ext.to_str()),
                Some("xcodeproj" | "xcworkspace")
            )
        })
}

fn has_harmony_wrapper(root: &Path) -> bool {
    root.join(harmony_wrapper_name()).exists()
}

fn harmony_wrapper_name() -> &'static str {
    if cfg!(windows) {
        "hvigorw.bat"
    } else {
        "hvigorw"
    }
}

fn canonicalize_or_self(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn format_path_list(paths: &[PathBuf]) -> String {
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

    fn write_required_aars(libs_dir: &Path, legacy_names: bool) {
        std::fs::create_dir_all(libs_dir).unwrap();
        for requirement in ANDROID_REQUIRED_AARS {
            let name = if legacy_names && requirement.exact_names.len() > 1 {
                requirement.exact_names[1]
            } else {
                requirement.exact_names[0]
            };
            std::fs::write(libs_dir.join(name), b"aar").unwrap();
        }
    }

    #[test]
    fn android_package_root_with_sdk_libs_is_supported() {
        let root = unique_temp_dir("unipack-android-sdk-root");
        let libs = root.join("SDK/libs");
        write_required_aars(&libs, false);
        std::fs::create_dir_all(root.join("SDK/assets/data")).unwrap();

        let layout = resolve_android_sdk_layout(&root).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        assert_eq!(layout.libs_dir, libs.canonicalize().unwrap());
        assert_eq!(
            layout.assets_dir,
            root.join("SDK/assets").canonicalize().unwrap()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_legacy_aar_names_are_supported() {
        let root = unique_temp_dir("unipack-android-sdk-legacy");
        write_required_aars(&root.join("SDK/libs"), true);
        std::fs::create_dir_all(root.join("SDK/assets")).unwrap();

        let layout = resolve_android_sdk_layout(&root).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_versioned_aar_names_are_supported() {
        let root = unique_temp_dir("unipack-android-sdk-versioned");
        let libs = root.join("SDK/libs");
        write_required_aars(&libs, false);
        std::fs::remove_file(libs.join("android-gif-drawable-release@1.2.23.aar")).unwrap();
        std::fs::write(libs.join("android-gif-drawable-1.2.29.aar"), b"aar").unwrap();
        std::fs::remove_file(libs.join("oaid_sdk_1.0.25.aar")).unwrap();
        std::fs::write(libs.join("oaid_sdk_1.2.0.aar"), b"aar").unwrap();
        std::fs::create_dir_all(root.join("SDK/assets")).unwrap();

        let layout = resolve_android_sdk_layout(&root).unwrap();
        let gif =
            resolve_android_required_aar(&layout.libs_dir, &ANDROID_REQUIRED_AARS[1]).unwrap();
        let oaid =
            resolve_android_required_aar(&layout.libs_dir, &ANDROID_REQUIRED_AARS[3]).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        assert_eq!(
            gif.file_name().and_then(|n| n.to_str()),
            Some("android-gif-drawable-1.2.29.aar")
        );
        assert_eq!(
            oaid.file_name().and_then(|n| n.to_str()),
            Some("oaid_sdk_1.2.0.aar")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_sdk_child_selection_is_supported() {
        let root = unique_temp_dir("unipack-android-sdk-child");
        let sdk = root.join("SDK");
        write_required_aars(&sdk.join("libs"), false);
        std::fs::create_dir_all(sdk.join("assets")).unwrap();

        let layout = resolve_android_sdk_layout(&sdk).unwrap();

        assert_eq!(layout.root, sdk.canonicalize().unwrap());
        assert_eq!(layout.libs_dir, sdk.join("libs").canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_libs_child_selection_is_supported() {
        let root = unique_temp_dir("unipack-android-sdk-libs");
        let sdk = root.join("SDK");
        let libs = sdk.join("libs");
        write_required_aars(&libs, false);
        std::fs::create_dir_all(sdk.join("assets")).unwrap();

        let layout = resolve_android_sdk_layout(&libs).unwrap();

        assert_eq!(layout.root, sdk.canonicalize().unwrap());
        assert_eq!(layout.libs_dir, libs.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_parent_selection_searches_children() {
        let parent = unique_temp_dir("unipack-android-sdk-parent");
        let root = parent.join("Android-SDK@20260414");
        write_required_aars(&root.join("SDK/libs"), false);
        std::fs::create_dir_all(root.join("SDK/assets")).unwrap();

        let layout = resolve_android_sdk_layout(&parent).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(parent);
    }

    #[test]
    fn android_missing_aar_error_lists_checked_dir_and_candidates() {
        let root = unique_temp_dir("unipack-android-sdk-missing");
        std::fs::create_dir_all(root.join("SDK/libs")).unwrap();

        let err = resolve_android_sdk_layout(&root).unwrap_err();

        assert!(err.contains(&root.join("SDK/libs").display().to_string()));
        assert!(err.contains("android-gif-drawable"));
        assert!(err.contains("文件名前缀"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn local_downloaded_android_sdk_layout_is_supported_when_present() {
        let root =
            PathBuf::from("/Users/huangxiangrui/Downloads/5.07/Android-SDK@5.07.82603_20260414");
        if !root.exists() {
            return;
        }

        let layout = resolve_android_sdk_layout(&root).unwrap();
        let gif =
            resolve_android_required_aar(&layout.libs_dir, &ANDROID_REQUIRED_AARS[1]).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        assert_eq!(
            gif.file_name().and_then(|n| n.to_str()),
            Some("android-gif-drawable-1.2.29.aar")
        );
    }

    #[test]
    fn ios_parent_selection_finds_hello_project() {
        let root = unique_temp_dir("unipack-ios-sdk");
        let hello = root.join("HBuilder-HelloUniApp");
        std::fs::create_dir_all(hello.join("Demo.xcodeproj")).unwrap();

        let found = resolve_ios_sdk_project(&root).unwrap();
        let saved_root = resolve_ios_sdk_root(&root).unwrap();

        assert_eq!(found, hello.canonicalize().unwrap());
        assert_eq!(saved_root, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn harmony_parent_selection_finds_template() {
        let parent = unique_temp_dir("unipack-harmony-template-parent");
        let template = parent.join("HarmonyTemplate");
        std::fs::create_dir_all(&template).unwrap();
        std::fs::write(template.join(harmony_wrapper_name()), b"#!/bin/sh").unwrap();

        let found = resolve_harmony_template_root(&parent).unwrap();

        assert_eq!(found, template.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(parent);
    }
}
