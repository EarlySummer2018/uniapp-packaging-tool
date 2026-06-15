use std::collections::HashMap;
use std::path::Path;

use crate::commands::ios::build::pbxproj::{
    register_pbx_embedded_frameworks, register_pbx_linked_files, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_LIVEPUSHER_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSCameraUsageDescription",
        label: "相机权限说明",
        default_value: "用于视频直播",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSMicrophoneUsageDescription",
        label: "麦克风权限说明",
        default_value: "用于音频采集",
        required: true,
    },
];

#[derive(Debug, Clone)]
pub(crate) struct IosLivePusherIntegration {
    pub(crate) linked_count: usize,
    pub(crate) embedded_count: usize,
}

pub(crate) fn ios_livepusher_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "LivePusher")
}

pub(crate) fn apply_ios_livepusher_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    ios_module_config: &HashMap<String, String>,
) -> Result<Option<IosLivePusherIntegration>, String> {
    if !ios_livepusher_enabled(manifest_info) {
        return Ok(None);
    }

    let custom_component_mode = ios_livepusher_custom_component_mode(ios_module_config);
    let linked_files = ios_livepusher_linked_files(custom_component_mode);
    let embedded_files = ios_livepusher_embedded_frameworks();
    validate_ios_livepusher_local_files(project_root, &linked_files)?;
    validate_ios_livepusher_local_files(project_root, &embedded_files)?;

    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    let embedded_count = register_pbx_embedded_frameworks(project_file, &embedded_files)?;

    Ok(Some(IosLivePusherIntegration {
        linked_count,
        embedded_count,
    }))
}

pub(crate) fn apply_ios_livepusher_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_LIVEPUSHER_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}

fn ios_livepusher_linked_files(custom_component_mode: bool) -> Vec<IosPbxLinkedFile> {
    let mut files = vec![
        IosPbxLinkedFile::local_static("liblibLivePush.a"),
        IosPbxLinkedFile::local_static("libDCUniGPUImage.a"),
        IosPbxLinkedFile::local_framework("UPLiveSDKDll.framework"),
        IosPbxLinkedFile::system_framework("AVFoundation.framework"),
        IosPbxLinkedFile::system_framework("QuartzCore.framework"),
        IosPbxLinkedFile::system_framework("OpenGLES.framework"),
        IosPbxLinkedFile::system_framework("AudioToolbox.framework"),
        IosPbxLinkedFile::system_framework("VideoToolbox.framework"),
        IosPbxLinkedFile::system_framework("Accelerate.framework"),
        IosPbxLinkedFile::system_framework("CoreMedia.framework"),
        IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
        IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
        IosPbxLinkedFile::system_framework("CoreMotion.framework"),
        IosPbxLinkedFile::system_library("libz.tbd"),
        IosPbxLinkedFile::system_library("libbz2.tbd"),
        IosPbxLinkedFile::system_library("libiconv.tbd"),
    ];
    if custom_component_mode {
        files.insert(2, IosPbxLinkedFile::local_static("libDCUniLivePush.a"));
    }
    files
}

fn ios_livepusher_embedded_frameworks() -> Vec<IosPbxLinkedFile> {
    vec![IosPbxLinkedFile::local_framework("UPLiveSDKDll.framework")]
}

fn validate_ios_livepusher_local_files(
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
                "iOS LivePusher 模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn ios_livepusher_custom_component_mode(config: &HashMap<String, String>) -> bool {
    config
        .get("livepusher.customComponentMode")
        .or_else(|| config.get("customComponentMode"))
        .is_some_and(|value| ios_bool_config_enabled(value))
}

fn ios_bool_config_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on" | "是" | "开启"
    )
}
