use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    register_pbx_linked_files, register_pbx_resources, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_has_detected_module,
    ios_manifest_module_enabled, ios_object_value_normalized, ios_sdk_config_value_enabled,
    IosPrivacyFieldSpec,
};

pub(crate) const IOS_SPEECH_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSSpeechRecognitionUsageDescription",
        label: "语音识别权限说明",
        default_value: "用于识别语音输入内容",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSMicrophoneUsageDescription",
        label: "麦克风权限说明",
        default_value: "用于采集语音输入音频",
        required: true,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosSpeechProvider {
    Baidu,
    Ifly,
}

impl IosSpeechProvider {
    fn label(self) -> &'static str {
        match self {
            Self::Baidu => "百度语音",
            Self::Ifly => "讯飞语音",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IosSpeechIntegration {
    pub(crate) providers: Vec<IosSpeechProvider>,
    pub(crate) linked_count: usize,
    pub(crate) resource_count: usize,
}

impl IosSpeechIntegration {
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

pub(crate) fn ios_speech_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_speech_providers(manifest_info).is_some()
}

pub(crate) fn apply_ios_speech_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosSpeechIntegration>, String> {
    let Some(info) = manifest_info else {
        return Ok(None);
    };
    let Some(providers) = ios_speech_providers(Some(info)) else {
        return Ok(None);
    };

    let linked_files = ios_speech_linked_files(&providers);
    validate_ios_speech_local_linked_files(project_root, &linked_files)?;
    let resource_sources = ios_speech_resource_sources(project_root, &providers)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    let resource_count = copy_ios_speech_resources(project_root, project_file, &resource_sources)?;

    Ok(Some(IosSpeechIntegration {
        providers,
        linked_count,
        resource_count,
    }))
}

pub(crate) fn ios_speech_providers(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<Vec<IosSpeechProvider>> {
    let info = manifest_info?;
    let manifest = info.manifest_value.as_ref()?;
    if !ios_manifest_info_has_detected_module(info, "Speech")
        || !ios_manifest_module_enabled(manifest, "Speech")
    {
        return None;
    }
    let speech_config = ios_speech_sdk_config(manifest)?;
    if !ios_sdk_config_value_enabled(speech_config, Some("ios")) {
        return None;
    }

    let mut providers = Vec::new();
    if ios_speech_provider_enabled(speech_config, &["baidu", "bd"]) {
        push_ios_speech_provider(&mut providers, IosSpeechProvider::Baidu);
    }
    if ios_speech_provider_enabled(speech_config, &["ifly", "xfyun", "xunfei"]) {
        push_ios_speech_provider(&mut providers, IosSpeechProvider::Ifly);
    }

    if providers.is_empty() {
        return None;
    }
    Some(providers)
}

pub(crate) fn apply_ios_speech_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_SPEECH_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}

pub(crate) fn ios_speech_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["speech", "speechRecognition"]
        .iter()
        .find_map(|key| ios_object_value_normalized(sdk_configs, key))
}

fn ios_speech_provider_enabled(speech_config: &serde_json::Value, provider_keys: &[&str]) -> bool {
    let Some(config) = speech_config.as_object() else {
        return false;
    };
    provider_keys.iter().any(|provider_key| {
        ios_object_value_normalized(config, provider_key)
            .is_some_and(|value| ios_sdk_config_value_enabled(value, Some("ios")))
    })
}

fn push_ios_speech_provider(providers: &mut Vec<IosSpeechProvider>, provider: IosSpeechProvider) {
    if !providers.contains(&provider) {
        providers.push(provider);
    }
}

fn ios_speech_linked_files(providers: &[IosSpeechProvider]) -> Vec<IosPbxLinkedFile> {
    let mut files = Vec::new();
    push_ios_linked_file(&mut files, IosPbxLinkedFile::local_static("liblibSpeech.a"));

    if providers.contains(&IosSpeechProvider::Baidu) {
        for file in [
            IosPbxLinkedFile::local_static("libBaiduSpeechSDK.a"),
            IosPbxLinkedFile::local_static("libbaiduSpeech.a"),
            IosPbxLinkedFile::system_library("libc++.tbd"),
            IosPbxLinkedFile::system_library("libz.tbd"),
            IosPbxLinkedFile::system_library("libsqlite3.tbd"),
            IosPbxLinkedFile::system_framework("AudioToolbox.framework"),
            IosPbxLinkedFile::system_framework("AVFoundation.framework"),
            IosPbxLinkedFile::system_framework("CFNetwork.framework"),
            IosPbxLinkedFile::system_framework("CoreLocation.framework"),
            IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
            IosPbxLinkedFile::system_framework("GLKit.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }
    if providers.contains(&IosSpeechProvider::Ifly) {
        push_ios_linked_file(
            &mut files,
            IosPbxLinkedFile::local_framework("iflyMSC.framework"),
        );
    }

    files
}

fn push_ios_linked_file(files: &mut Vec<IosPbxLinkedFile>, file: IosPbxLinkedFile) {
    if !files.iter().any(|existing| existing.name == file.name) {
        files.push(file);
    }
}

fn validate_ios_speech_local_linked_files(
    project_root: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    for file in files.iter().copied().filter(|file| file.is_local()) {
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS 语音输入模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn ios_speech_resource_sources(
    project_root: &Path,
    providers: &[IosSpeechProvider],
) -> Result<Vec<(String, PathBuf)>, String> {
    if !providers.contains(&IosSpeechProvider::Baidu) {
        return Ok(Vec::new());
    }
    let bundles_dir = ios_sdk_support_dir(project_root)?.join("Bundles");
    let source = bundles_dir.join("BDSClientEASRResources");
    if !source.exists() {
        return Err(format!(
            "iOS 语音输入模块缺少 SDK 资源文件: BDSClientEASRResources ({})",
            bundles_dir.display()
        ));
    }
    Ok(vec![("BDSClientEASRResources".to_string(), source)])
}

fn copy_ios_speech_resources(
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
            std::fs::remove_dir_all(&target).map_err(|e| {
                format!("清理 iOS 语音输入资源副本失败 {}: {}", target.display(), e)
            })?;
        } else if target.exists() {
            std::fs::remove_file(&target).map_err(|e| {
                format!("清理 iOS 语音输入资源副本失败 {}: {}", target.display(), e)
            })?;
        }
        if source.is_dir() {
            crate::utils::fs::copy_recursive(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 语音输入资源失败 {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        } else {
            std::fs::copy(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 语音输入资源失败 {} -> {}: {}",
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
