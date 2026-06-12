use std::path::{Path, PathBuf};

use crate::commands::shared::resource_scan::extract_hbuilderx_version;

use super::assets::{
    find_manifest_android_icons, find_manifest_ios_icons, find_manifest_ios_privacy_descriptions,
    find_manifest_push_icons, find_manifest_splashscreen,
};
use super::module_detection::{
    check_module_configured_in_props, collect_modules_from_sdk_configs, collect_modules_from_value,
};
use super::types::{AndroidManifestConfig, PlatformPackages, UniappManifestInfo};

pub fn read_manifest_file(manifest_path: &Path) -> Result<serde_json::Value, String> {
    let content = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
    parse_manifest_content(&content).map_err(|e| format!("解析 manifest.json 失败: {}", e))
}

pub(super) fn parse_manifest_content(content: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str::<serde_json::Value>(content).or_else(|json_error| {
        json5::from_str::<serde_json::Value>(content).map_err(|json5_error| {
            format!(
                "{}；已尝试按 JSON5/JSONC 兼容格式解析，仍失败: {}",
                json_error, json5_error
            )
        })
    })
}

pub fn read_uniapp_manifest_sync(project_path: &str) -> Result<UniappManifestInfo, String> {
    let project_root = PathBuf::from(project_path);
    if !project_root.is_dir() {
        return Err(format!("本地项目路径不存在: {}", project_path));
    }

    let manifest_path = project_root.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("未找到 manifest.json: {}", manifest_path.display()));
    }

    let manifest = read_manifest_file(&manifest_path)?;
    Ok(parse_uniapp_manifest(
        &manifest,
        &manifest_path,
        &project_root,
        None,
    ))
}

pub fn parse_uniapp_manifest(
    manifest: &serde_json::Value,
    manifest_path: &Path,
    project_root: &Path,
    props_content: Option<&str>,
) -> UniappManifestInfo {
    let app_id = string_field(manifest, &["appid", "appId", "id"]);
    let app_name = string_field(manifest, &["name"]);
    let version_name = string_field(manifest, &["versionName", "version"]);
    let version_code = number_field(manifest, &["versionCode", "version_code"]);
    let hbuilderx_version = extract_hbuilderx_version(manifest);
    let android_icons = find_manifest_android_icons(manifest, project_root);
    let ios_icons = find_manifest_ios_icons(manifest, project_root);
    let push_icons = find_manifest_push_icons(manifest, project_root);
    let splashscreen = find_manifest_splashscreen(manifest, project_root);
    let ios_privacy_descriptions = find_manifest_ios_privacy_descriptions(manifest);
    let android_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("android"));
    let ios_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("ios"));
    let harmony_value = manifest
        .get("app-plus")
        .and_then(|v| v.get("distribute"))
        .and_then(|v| v.get("harmony"));

    let mut detected_modules = Vec::new();
    if let Some(app_plus) = manifest.get("app-plus") {
        let app_modules = app_plus.get("modules");
        if let Some(modules) = app_modules {
            collect_modules_from_value(modules, "all", &mut detected_modules);
        }
        if let Some(distribute) = app_plus.get("distribute") {
            if let Some(sdk_configs) = distribute.get("sdkConfigs") {
                collect_modules_from_sdk_configs(
                    sdk_configs,
                    "all",
                    &mut detected_modules,
                    app_modules,
                );
            }
            if let Some(android) = distribute.get("android") {
                let android_modules = android.get("modules").or(app_modules);
                if let Some(modules) = android.get("modules") {
                    collect_modules_from_value(modules, "android", &mut detected_modules);
                }
                if let Some(sdk_configs) = android.get("sdkConfigs") {
                    collect_modules_from_sdk_configs(
                        sdk_configs,
                        "android",
                        &mut detected_modules,
                        android_modules,
                    );
                }
            }
            if let Some(ios) = distribute.get("ios") {
                let ios_modules = ios.get("modules").or(app_modules);
                if let Some(modules) = ios.get("modules") {
                    collect_modules_from_value(modules, "ios", &mut detected_modules);
                }
                if let Some(sdk_configs) = ios.get("sdkConfigs") {
                    collect_modules_from_sdk_configs(
                        sdk_configs,
                        "ios",
                        &mut detected_modules,
                        ios_modules,
                    );
                }
            }
            if let Some(harmony) = distribute.get("harmony") {
                if let Some(modules) = harmony.get("modules") {
                    collect_modules_from_value(modules, "harmony", &mut detected_modules);
                }
            }
        }
    }

    if let Some(props) = props_content {
        for module in &mut detected_modules {
            module.configured = check_module_configured_in_props(&module.name, props);
        }
    }

    UniappManifestInfo {
        app_name,
        app_id,
        version_name,
        version_code,
        hbuilderx_version,
        android_icons,
        ios_icons,
        push_icons,
        splashscreen,
        ios_privacy_descriptions,
        manifest_value: Some(manifest.clone()),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        project_root: project_root.to_string_lossy().to_string(),
        android: AndroidManifestConfig {
            package_name: android_value
                .and_then(|v| string_field(v, &["packageName", "packagename", "applicationId"])),
            min_sdk_version: Some(
                android_value
                    .and_then(|v| {
                        number_field(
                            v,
                            &["minSdkVersion", "minSdk", "min_sdk", "minSdkVersionCode"],
                        )
                    })
                    .unwrap_or(21),
            ),
            target_sdk_version: android_value
                .and_then(|v| number_field(v, &["targetSdkVersion", "targetSdk", "target_sdk"])),
            compile_sdk_version: android_value
                .and_then(|v| number_field(v, &["compileSdkVersion", "compileSdk", "compile_sdk"])),
            permissions: android_value
                .and_then(|v| string_list_field(v, &["permissions"]))
                .unwrap_or_default(),
            exclude_permissions: android_value
                .and_then(|v| string_list_field(v, &["excludePermissions", "exclude_permissions"]))
                .unwrap_or_default(),
            schemes: android_value
                .and_then(|v| string_list_field(v, &["schemes", "urlSchemes", "url_schemes"]))
                .unwrap_or_default(),
            abi_filters: android_value
                .and_then(|v| string_list_field(v, &["abiFilters", "abi_filters"]))
                .unwrap_or_default(),
        },
        package_names: PlatformPackages {
            android_package: android_value
                .and_then(|v| string_field(v, &["packageName", "packagename", "applicationId"])),
            ios_bundle_id: ios_value
                .and_then(|v| string_field(v, &["bundleId", "bundleid", "bundleIdentifier"])),
            harmony_bundle: harmony_value
                .and_then(|v| string_field(v, &["packageName", "bundleName", "bundle"])),
        },
        detected_modules,
        warnings: Vec::new(),
    }
}

pub(super) fn string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(String::from)
    })
}

fn string_list_field(value: &serde_json::Value, keys: &[&str]) -> Option<Vec<String>> {
    keys.iter().find_map(|key| {
        let value = value.get(*key)?;
        let mut items = Vec::new();
        collect_string_list_values(value, &mut items);
        if items.is_empty() {
            None
        } else {
            Some(dedup_strings(items))
        }
    })
}

fn collect_string_list_values(value: &serde_json::Value, items: &mut Vec<String>) {
    match value {
        serde_json::Value::String(raw) => {
            for item in raw.split(',') {
                let item = item.trim();
                if !item.is_empty() {
                    items.push(item.to_string());
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_string_list_values(value, items);
            }
        }
        _ => {}
    }
}

fn dedup_strings(items: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for item in items {
        if !result.iter().any(|existing| existing == &item) {
            result.push(item);
        }
    }
    result
}

pub(super) fn bool_field(value: &serde_json::Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|v| v.as_bool()))
}

fn number_field(value: &serde_json::Value, keys: &[&str]) -> Option<u32> {
    keys.iter().find_map(|key| {
        let value = value.get(*key)?;
        if let Some(num) = value.as_u64() {
            return Some(num as u32);
        }
        value.as_str().and_then(|s| s.trim().parse::<u32>().ok())
    })
}
