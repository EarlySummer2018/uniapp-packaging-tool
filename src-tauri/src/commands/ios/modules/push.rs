use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    enable_pbx_system_capability, register_pbx_linked_files, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    ios_manifest_info_has_detected_module, ios_sdk_config_value_enabled, merge_plist_string_array,
    normalize_ios_manifest_key,
};
use crate::commands::module::manifest_push_unipush_v2_enabled;

const IOS_PUSH_BACKGROUND_MODES: &[&str] = &["remote-notification"];

#[derive(Debug, Clone)]
pub(crate) struct IosPushIntegration {
    pub(crate) linked_count: usize,
    pub(crate) background_modes: Vec<&'static str>,
}

pub(crate) fn ios_push_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    let Some(info) = manifest_info else {
        return false;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return false;
    };
    ios_manifest_info_has_detected_module(info, "Push")
        && manifest_push_unipush_v2_enabled(manifest)
}

pub(crate) fn apply_ios_push_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosPushIntegration>, String> {
    if !ios_push_enabled(manifest_info) {
        return Ok(None);
    }

    let linked_files = ios_push_linked_files();
    validate_ios_push_local_linked_files(project_root, &linked_files)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    enable_pbx_system_capability(project_file, "com.apple.BackgroundModes")?;
    enable_pbx_system_capability(project_file, "com.apple.Push")?;
    patch_ios_push_feature_plist(project_root, project_file)?;

    Ok(Some(IosPushIntegration {
        linked_count,
        background_modes: IOS_PUSH_BACKGROUND_MODES.to_vec(),
    }))
}

pub(crate) fn apply_ios_push_plist_defaults(
    dict: &mut plist::Dictionary,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    if !ios_push_enabled(manifest_info) {
        return;
    }

    merge_plist_string_array(dict, "UIBackgroundModes", IOS_PUSH_BACKGROUND_MODES);
    if let Some(config) = ios_push_getui_config(manifest_info) {
        let mut getui = plist::Dictionary::new();
        getui.insert("appid".into(), plist::Value::String(config.appid));
        getui.insert("appkey".into(), plist::Value::String(config.appkey));
        getui.insert("appsecret".into(), plist::Value::String(config.appsecret));
        dict.insert("getui".into(), plist::Value::Dictionary(getui));
    }
}

fn ios_push_linked_files() -> Vec<IosPbxLinkedFile> {
    vec![
        IosPbxLinkedFile::local_static("liblibPush.a"),
        IosPbxLinkedFile::local_static("libUniPush.a"),
        IosPbxLinkedFile::local_xcframework("GTSDK.xcframework"),
        IosPbxLinkedFile::system_library("libc++.tbd"),
        IosPbxLinkedFile::system_library("libsqlite3.tbd"),
        IosPbxLinkedFile::system_library("libz.tbd"),
        IosPbxLinkedFile::system_library("libresolv.tbd"),
        IosPbxLinkedFile::optional_system_framework("UserNotifications.framework"),
        IosPbxLinkedFile::system_framework("Security.framework"),
        IosPbxLinkedFile::system_framework("MobileCoreServices.framework"),
        IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
        IosPbxLinkedFile::system_framework("CoreLocation.framework"),
        IosPbxLinkedFile::system_framework("AVFoundation.framework"),
        IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
    ]
}

fn patch_ios_push_feature_plist(project_root: &Path, project_file: &Path) -> Result<(), String> {
    if let Some(feature_plist) = find_pandora_api_feature_plist(project_root) {
        return ensure_ios_push_feature_plist_entry(&feature_plist);
    }

    let feature_plist = copy_workspace_sdk_pandora_api_bundle(project_root, project_file)?;
    ensure_ios_push_feature_plist_entry(&feature_plist)
}

fn ensure_ios_push_feature_plist_entry(feature_plist: &Path) -> Result<(), String> {
    let mut value = plist::Value::from_file(&feature_plist).map_err(|e| {
        format!(
            "解析 PandoraApi.bundle/feature.plist 失败 {}: {}",
            feature_plist.display(),
            e
        )
    })?;
    let dict = value
        .as_dictionary_mut()
        .ok_or_else(|| "PandoraApi.bundle/feature.plist 不是 dictionary".to_string())?;
    dict.insert(
        "Push".into(),
        plist::Value::Dictionary(ios_push_feature_plist_entry()),
    );
    value.to_file_xml(&feature_plist).map_err(|e| {
        format!(
            "写入 PandoraApi.bundle/feature.plist 失败 {}: {}",
            feature_plist.display(),
            e
        )
    })
}

fn ios_push_feature_plist_entry() -> plist::Dictionary {
    let mut server = plist::Dictionary::new();
    server.insert(
        "class".into(),
        plist::Value::String("PGPushServerAct".into()),
    );
    server.insert(
        "identifier".into(),
        plist::Value::String("com.pushserver".into()),
    );

    let mut push = plist::Dictionary::new();
    push.insert("autostart".into(), plist::Value::Boolean(true));
    push.insert("baseclass".into(), plist::Value::String("PGPush".into()));
    push.insert(
        "class".into(),
        plist::Value::String("PGPushActualize".into()),
    );
    push.insert("global".into(), plist::Value::Boolean(true));
    push.insert("server".into(), plist::Value::Dictionary(server));
    push
}

fn find_pandora_api_feature_plist(project_root: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(project_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.extension().and_then(|ext| ext.to_str()) == Some("xcodeproj") {
                continue;
            }
            if let Some(found) = find_pandora_api_feature_plist(&path) {
                return Some(found);
            }
        } else if path.file_name().and_then(|name| name.to_str()) == Some("feature.plist")
            && path_has_ancestor_named(&path, "PandoraApi.bundle")
        {
            return Some(path);
        }
    }
    None
}

fn copy_workspace_sdk_pandora_api_bundle(
    project_root: &Path,
    project_file: &Path,
) -> Result<PathBuf, String> {
    let source_bundle = find_workspace_sdk_pandora_api_bundle(project_root).ok_or_else(|| {
        format!(
            "iOS Push 模块未找到 PandoraApi.bundle/feature.plist: {}",
            project_root.display()
        )
    })?;
    let target_bundle = project_root.join("HBuilder-Hello/PandoraApi.bundle");
    if target_bundle.exists() {
        std::fs::remove_dir_all(&target_bundle).map_err(|e| {
            format!(
                "清理 iOS Push PandoraApi.bundle 副本失败 {}: {}",
                target_bundle.display(),
                e
            )
        })?;
    }
    crate::utils::fs::copy_recursive(&source_bundle, &target_bundle).map_err(|e| {
        format!(
            "复制 iOS Push PandoraApi.bundle 到构建产物失败 {} -> {}: {}",
            source_bundle.display(),
            target_bundle.display(),
            e
        )
    })?;
    patch_pandora_api_bundle_file_reference(project_root, project_file, &target_bundle)?;
    Ok(target_bundle.join("feature.plist"))
}

fn find_workspace_sdk_pandora_api_bundle(project_root: &Path) -> Option<PathBuf> {
    let workspace = project_root.parent()?;
    let bundle = workspace.join("SDK/Bundles/PandoraApi.bundle");
    bundle.join("feature.plist").is_file().then_some(bundle)
}

fn patch_pandora_api_bundle_file_reference(
    project_root: &Path,
    project_file: &Path,
    target_bundle: &Path,
) -> Result<(), String> {
    let relative_path = target_bundle
        .strip_prefix(project_root)
        .map_err(|e| format!("计算 PandoraApi.bundle 相对路径失败: {}", e))?
        .to_string_lossy()
        .replace('\\', "/");
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let pattern = regex::Regex::new(
        r#"(?m)^(\s*[A-Za-z0-9]{24} /\* PandoraApi\.bundle \*/ = \{isa = PBXFileReference; ).*(\};)$"#,
    )
    .map_err(|e| e.to_string())?;
    let updated = pattern.replace(&content, |caps: &regex::Captures| {
        format!(
            "{}lastKnownFileType = \"wrapper.plug-in\"; name = PandoraApi.bundle; path = {}; sourceTree = SOURCE_ROOT; {}",
            caps.get(1).map_or("", |value| value.as_str()),
            render_pbx_value(&relative_path),
            caps.get(2).map_or("", |value| value.as_str())
        )
    });
    if updated == content {
        return Err("project.pbxproj 未找到 PandoraApi.bundle 引用".to_string());
    }
    std::fs::write(&pbxproj, updated.as_ref())
        .map_err(|e| format!("写入 project.pbxproj 失败: {}", e))
}

fn path_has_ancestor_named(path: &Path, name: &str) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.file_name().and_then(|value| value.to_str()) == Some(name))
}

fn render_pbx_value(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/'))
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}

struct IosGetuiConfig {
    appid: String,
    appkey: String,
    appsecret: String,
}

fn ios_push_getui_config(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<IosGetuiConfig> {
    let info = manifest_info?;
    let manifest = info.manifest_value.as_ref()?;
    if !ios_push_enabled(Some(info)) {
        return None;
    }
    let config = ios_push_sdk_config(manifest)?;
    Some(IosGetuiConfig {
        appid: json_string_field(
            config,
            &[
                "appid_ios",
                "appId_ios",
                "GETUI_APPID",
                "plus.unipush.appid",
                "unipush_appid",
                "appid",
                "appId",
            ],
        )?,
        appkey: json_string_field(
            config,
            &[
                "appkey_ios",
                "appKey_ios",
                "plus.unipush.appkey",
                "unipush_appkey",
                "appkey",
                "appKey",
            ],
        )?,
        appsecret: json_string_field(
            config,
            &[
                "appsecret_ios",
                "appSecret_ios",
                "plus.unipush.appsecret",
                "unipush_appsecret",
                "appsecret",
                "appSecret",
            ],
        )?,
    })
}

fn ios_push_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?;
    let push = find_object_value_normalized(sdk_configs, "push")?;
    let config = find_object_value_normalized(push, "unipush")?;
    ios_sdk_config_value_enabled(config, Some("ios")).then_some(config)
}

fn json_string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    if let serde_json::Value::Object(map) = value {
        if !ios_sdk_config_value_enabled(value, Some("ios")) {
            return None;
        }
        for key in keys {
            if let Some(value) = find_object_value_normalized(value, key)
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                return Some(value.to_string());
            }
        }
        for key in ["unipush", "getui", "igt", "ios"] {
            if let Some(value) = find_object_value_normalized(value, key)
                .and_then(|value| json_string_field(value, keys))
            {
                return Some(value);
            }
        }
        if map.len() == 1 {
            return map
                .values()
                .next()
                .and_then(|value| json_string_field(value, keys));
        }
    }
    None
}

fn find_object_value_normalized<'a>(
    value: &'a serde_json::Value,
    key: &str,
) -> Option<&'a serde_json::Value> {
    let map = value.as_object()?;
    let normalized_key = normalize_ios_manifest_key(key);
    map.iter()
        .find(|(candidate, _)| normalize_ios_manifest_key(candidate) == normalized_key)
        .map(|(_, value)| value)
}

fn validate_ios_push_local_linked_files(
    project_root: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    let libs_dir = project_root
        .parent()
        .map(|workspace| workspace.join("SDK/Libs"))
        .ok_or_else(|| format!("iOS 工程路径异常: {}", project_root.display()))?;
    for file in files.iter().copied().filter(|file| file.is_local()) {
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS Push 模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_push_feature_plist_copies_workspace_sdk_bundle_into_project() {
        let root = std::env::temp_dir().join(format!(
            "unipack-ios-push-sdk-feature-{}",
            uuid::Uuid::new_v4()
        ));
        let project_root = root.join("HBuilder-Hello");
        let project_file = project_root.join("HBuilder-Hello.xcodeproj");
        let source_feature_plist = root.join("SDK/Bundles/PandoraApi.bundle/feature.plist");
        let target_feature_plist =
            project_root.join("HBuilder-Hello/PandoraApi.bundle/feature.plist");
        std::fs::create_dir_all(&project_file).unwrap();
        std::fs::create_dir_all(source_feature_plist.parent().unwrap()).unwrap();
        std::fs::write(
            project_file.join("project.pbxproj"),
            r#"/* Begin PBXFileReference section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* PandoraApi.bundle */ = {isa = PBXFileReference; lastKnownFileType = "wrapper.plug-in"; path = PandoraApi.bundle; sourceTree = "<group>"; };
/* End PBXFileReference section */
"#,
        )
        .unwrap();

        let mut server = plist::Dictionary::new();
        server.insert("class".into(), plist::Value::String("PGPushServer".into()));
        server.insert(
            "identifier".into(),
            plist::Value::String("com.pushserver".into()),
        );
        let mut push = plist::Dictionary::new();
        push.insert("autostart".into(), plist::Value::Boolean(true));
        push.insert("baseclass".into(), plist::Value::String("PGPush".into()));
        push.insert("global".into(), plist::Value::Boolean(true));
        push.insert("server".into(), plist::Value::Dictionary(server));
        let mut feature = plist::Dictionary::new();
        feature.insert("Push".into(), plist::Value::Dictionary(push));
        plist::Value::Dictionary(feature)
            .to_file_xml(&source_feature_plist)
            .unwrap();

        patch_ios_push_feature_plist(&project_root, &project_file).unwrap();

        let feature_plist = plist::Value::from_file(&target_feature_plist).unwrap();
        let push = feature_plist
            .as_dictionary()
            .and_then(|dict| dict.get("Push"))
            .and_then(plist::Value::as_dictionary)
            .unwrap();
        let server = push
            .get("server")
            .and_then(plist::Value::as_dictionary)
            .unwrap();
        assert_eq!(
            push.get("class").and_then(plist::Value::as_string),
            Some("PGPushActualize")
        );
        assert_eq!(
            server.get("class").and_then(plist::Value::as_string),
            Some("PGPushServerAct")
        );
        let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
        assert!(pbxproj.contains("path = HBuilder-Hello/PandoraApi.bundle"));
        assert!(pbxproj.contains("sourceTree = SOURCE_ROOT"));

        let _ = std::fs::remove_dir_all(root);
    }
}
