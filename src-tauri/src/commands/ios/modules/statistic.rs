use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    register_pbx_linked_files, register_pbx_resources, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    ios_manifest_info_has_detected_module, ios_manifest_module_enabled,
    ios_object_value_normalized, ios_sdk_config_value_enabled,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosStatisticProvider {
    Umeng,
    Firebase,
}

impl IosStatisticProvider {
    fn label(self) -> &'static str {
        match self {
            Self::Umeng => "友盟统计",
            Self::Firebase => "Firebase Analytics",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IosStatisticIntegration {
    pub(crate) providers: Vec<IosStatisticProvider>,
    pub(crate) linked_count: usize,
    pub(crate) resource_count: usize,
}

impl IosStatisticIntegration {
    pub(crate) fn summary(&self) -> String {
        let providers = self
            .providers
            .iter()
            .map(|provider| provider.label())
            .collect::<Vec<_>>()
            .join("、");
        format!("{}，自动迁移依赖", providers)
    }
}

pub(crate) fn apply_ios_statistic_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosStatisticIntegration>, String> {
    let Some(info) = manifest_info else {
        return Ok(None);
    };
    let Some(providers) = ios_statistic_providers(Some(info)) else {
        return Ok(None);
    };

    let linked_files = ios_statistic_linked_files(&providers);
    validate_ios_statistic_local_linked_files(project_root, &linked_files)?;
    let resource_sources = ios_statistic_resource_sources(project_root, &providers)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    let resource_count =
        copy_ios_statistic_resources(project_root, project_file, &resource_sources)?;
    patch_ios_statistic_feature_plist(project_root, &providers)?;

    Ok(Some(IosStatisticIntegration {
        providers,
        linked_count,
        resource_count,
    }))
}

pub(crate) fn ios_statistic_providers(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<Vec<IosStatisticProvider>> {
    let info = manifest_info?;
    let manifest = info.manifest_value.as_ref()?;
    if !ios_manifest_info_has_detected_module(info, "Statistic")
        || !ios_manifest_module_enabled(manifest, "Statistic")
    {
        return None;
    }
    let statistic_config = ios_statistic_sdk_config(manifest)?;
    if !ios_sdk_config_value_enabled(statistic_config, Some("ios")) {
        return None;
    }

    let mut providers = Vec::new();
    if ios_statistic_provider_enabled(statistic_config, &["umeng", "umeng-ios"]) {
        push_ios_statistic_provider(&mut providers, IosStatisticProvider::Umeng);
    }
    if ios_statistic_provider_enabled(
        statistic_config,
        &["firebase", "google", "googleFirebase", "firebaseAnalytics"],
    ) {
        push_ios_statistic_provider(&mut providers, IosStatisticProvider::Firebase);
    }

    if providers.is_empty() {
        return None;
    }
    Some(providers)
}

pub(crate) fn ios_statistic_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["statistic", "statistics", "statics"]
        .iter()
        .find_map(|key| ios_object_value_normalized(sdk_configs, key))
}

fn ios_statistic_provider_enabled(
    statistic_config: &serde_json::Value,
    provider_keys: &[&str],
) -> bool {
    let Some(config) = statistic_config.as_object() else {
        return false;
    };
    provider_keys.iter().any(|provider_key| {
        ios_object_value_normalized(config, provider_key)
            .is_some_and(|value| ios_sdk_config_value_enabled(value, Some("ios")))
    })
}

fn push_ios_statistic_provider(
    providers: &mut Vec<IosStatisticProvider>,
    provider: IosStatisticProvider,
) {
    if !providers.contains(&provider) {
        providers.push(provider);
    }
}

fn ios_statistic_linked_files(providers: &[IosStatisticProvider]) -> Vec<IosPbxLinkedFile> {
    let mut files = Vec::new();
    push_ios_linked_file(
        &mut files,
        IosPbxLinkedFile::local_static("liblibStatistic.a"),
    );

    if providers.contains(&IosStatisticProvider::Umeng) {
        for file in [
            IosPbxLinkedFile::local_static("libUmengStatistic.a"),
            IosPbxLinkedFile::local_xcframework("UMDevice.xcframework"),
            IosPbxLinkedFile::local_xcframework("UMCommon.xcframework"),
            IosPbxLinkedFile::local_framework("UMAPM.framework"),
            IosPbxLinkedFile::system_library("libz.tbd"),
            IosPbxLinkedFile::system_library("libsqlite3.tbd"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
            IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosStatisticProvider::Firebase) {
        for file in [
            IosPbxLinkedFile::local_static("libGoogleStatistic.a"),
            IosPbxLinkedFile::local_xcframework("FirebaseCore.xcframework"),
            IosPbxLinkedFile::local_xcframework("FirebaseCoreInternal.xcframework"),
            IosPbxLinkedFile::local_xcframework("FirebaseInstallations.xcframework"),
            IosPbxLinkedFile::local_xcframework("GoogleAppMeasurement.xcframework"),
            IosPbxLinkedFile::local_xcframework("GoogleAppMeasurementIdentitySupport.xcframework"),
            IosPbxLinkedFile::local_xcframework("GoogleUtilities.xcframework"),
            IosPbxLinkedFile::local_xcframework("FBLPromises.xcframework"),
            IosPbxLinkedFile::local_xcframework("nanopb.xcframework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    files
}

fn push_ios_linked_file(files: &mut Vec<IosPbxLinkedFile>, file: IosPbxLinkedFile) {
    if !files.iter().any(|existing| existing.name == file.name) {
        files.push(file);
    }
}

fn validate_ios_statistic_local_linked_files(
    project_root: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    for file in files.iter().copied().filter(|file| file.is_local()) {
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS 统计模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn ios_statistic_resource_sources(
    project_root: &Path,
    providers: &[IosStatisticProvider],
) -> Result<Vec<(String, PathBuf)>, String> {
    if !providers.contains(&IosStatisticProvider::Firebase) {
        return Ok(Vec::new());
    }
    let bundles_dir = ios_sdk_support_dir(project_root)?.join("Bundles");
    let source = bundles_dir.join("GoogleService-Info.plist");
    if !source.exists() {
        return Err(format!(
            "iOS 统计模块缺少 Firebase 资源文件: GoogleService-Info.plist ({})",
            bundles_dir.display()
        ));
    }
    Ok(vec![("GoogleService-Info.plist".to_string(), source)])
}

fn copy_ios_statistic_resources(
    project_root: &Path,
    project_file: &Path,
    resource_sources: &[(String, PathBuf)],
) -> Result<usize, String> {
    if resource_sources.is_empty() {
        return Ok(0);
    }

    let target_dir = ios_project_resource_target_dir(project_root);
    crate::utils::fs::ensure_directory(&target_dir).map_err(|e| e.to_string())?;
    for (name, source) in resource_sources {
        let target = target_dir.join(name);
        if target.is_dir() {
            std::fs::remove_dir_all(&target)
                .map_err(|e| format!("清理 iOS 统计资源副本失败 {}: {}", target.display(), e))?;
        } else if target.exists() {
            std::fs::remove_file(&target)
                .map_err(|e| format!("清理 iOS 统计资源副本失败 {}: {}", target.display(), e))?;
        }
        if source.is_dir() {
            crate::utils::fs::copy_recursive(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 统计资源失败 {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        } else {
            std::fs::copy(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 统计资源失败 {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        }
    }

    let resource_names = resource_sources
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    register_pbx_resources(project_file, &resource_names)?;
    Ok(resource_names.len())
}

fn patch_ios_statistic_feature_plist(
    project_root: &Path,
    providers: &[IosStatisticProvider],
) -> Result<(), String> {
    let Some(feature_plist) = find_pandora_api_feature_plist(project_root) else {
        return Ok(());
    };
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
    if providers.contains(&IosStatisticProvider::Firebase) {
        dict.insert(
            "Statistic".into(),
            plist::Value::Dictionary(ios_statistic_feature_plist_entry(
                "GoogleStatistic",
                "GoogleStatisticServer",
                "com.firebase.startup",
            )),
        );
    } else if providers.contains(&IosStatisticProvider::Umeng) {
        dict.insert(
            "Statistic".into(),
            plist::Value::Dictionary(ios_statistic_feature_plist_entry(
                "UmengStatistic",
                "UmengStatisticServer",
                "com.umeng.startup",
            )),
        );
    }
    value.to_file_xml(&feature_plist).map_err(|e| {
        format!(
            "写入 PandoraApi.bundle/feature.plist 失败 {}: {}",
            feature_plist.display(),
            e
        )
    })
}

fn ios_statistic_feature_plist_entry(
    class_name: &str,
    server_class: &str,
    server_identifier: &str,
) -> plist::Dictionary {
    let mut server = plist::Dictionary::new();
    server.insert("class".into(), plist::Value::String(server_class.into()));
    server.insert(
        "identifier".into(),
        plist::Value::String(server_identifier.into()),
    );

    let mut statistic = plist::Dictionary::new();
    statistic.insert("autostart".into(), plist::Value::Boolean(false));
    statistic.insert("class".into(), plist::Value::String(class_name.into()));
    statistic.insert("global".into(), plist::Value::Boolean(true));
    statistic.insert("server".into(), plist::Value::Dictionary(server));
    statistic
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

fn path_has_ancestor_named(path: &Path, name: &str) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.file_name().and_then(|value| value.to_str()) == Some(name))
}

fn ios_sdk_support_dir(project_root: &Path) -> Result<PathBuf, String> {
    project_root
        .parent()
        .map(|workspace| workspace.join("SDK"))
        .ok_or_else(|| format!("iOS 工程路径异常: {}", project_root.display()))
}

fn ios_project_resource_target_dir(project_root: &Path) -> PathBuf {
    let hbuilder_dir = project_root.join("HBuilder-Hello");
    if hbuilder_dir.is_dir() {
        hbuilder_dir
    } else {
        project_root.to_path_buf()
    }
}
