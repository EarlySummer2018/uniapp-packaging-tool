use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    register_pbx_embedded_frameworks, register_pbx_linked_files, register_pbx_resources,
    remove_pbx_linked_or_embedded_files, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_module_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_FACIAL_RECOGNITION_VERIFY_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] =
    &[IosPrivacyFieldSpec {
        key: "NSCameraUsageDescription",
        label: "实人认证相机权限说明",
        default_value: "我们需要使用摄像头进行人脸识别验证",
        required: true,
    }];

const IOS_FACIAL_RECOGNITION_VERIFY_BUNDLES: &[&str] = &[
    "APBToygerFacade.bundle",
    "BioAuthEngine.bundle",
    "ToygerNative.bundle",
];

const IOS_UTS_DUPLICATE_LINKED_FILES: &[&str] = &[
    "liblibPDRCore.a",
    "liblibWeex.a",
    "libcoreSupport.a",
    "storage.framework",
    "libSDWebImage.a",
    "KSCrash.framework",
];

#[derive(Debug, Clone)]
pub(crate) struct IosFacialRecognitionVerifyIntegration {
    pub(crate) linked_count: usize,
    pub(crate) embedded_count: usize,
    pub(crate) resource_count: usize,
    pub(crate) removed_duplicate_count: usize,
}

pub(crate) fn ios_facial_recognition_verify_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "FacialRecognitionVerify")
}

pub(crate) fn apply_ios_facial_recognition_verify_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosFacialRecognitionVerifyIntegration>, String> {
    if !ios_facial_recognition_verify_enabled(manifest_info) {
        return Ok(None);
    }

    let linked_files = ios_facial_recognition_verify_linked_files();
    validate_ios_local_linked_files(project_root, &linked_files)?;
    validate_ios_resource_bundles(project_root)?;

    let removed_duplicate_count =
        remove_pbx_linked_or_embedded_files(project_file, IOS_UTS_DUPLICATE_LINKED_FILES)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    let embedded_count = register_pbx_embedded_frameworks(
        project_file,
        &[
            IosPbxLinkedFile::local_framework("DCUniBase.framework"),
            IosPbxLinkedFile::local_framework("DCloudUTSFoundation.framework"),
        ],
    )?;
    let resource_count = copy_ios_resource_bundles(project_root, project_file)?;

    Ok(Some(IosFacialRecognitionVerifyIntegration {
        linked_count,
        embedded_count,
        resource_count,
        removed_duplicate_count,
    }))
}

pub(crate) fn apply_ios_facial_recognition_verify_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_FACIAL_RECOGNITION_VERIFY_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}

fn ios_facial_recognition_verify_linked_files() -> Vec<IosPbxLinkedFile> {
    vec![
        IosPbxLinkedFile::local_framework("DCUniBase.framework"),
        IosPbxLinkedFile::local_framework("DCloudUTSFoundation.framework"),
        IosPbxLinkedFile::local_framework("uniFacialRecognitionVerify.framework"),
        IosPbxLinkedFile::local_framework("AliyunFaceAuthFacade.framework"),
        IosPbxLinkedFile::local_framework("AliyunMobileRPC.framework"),
        IosPbxLinkedFile::local_framework("AliyunOSSiOS.framework"),
        IosPbxLinkedFile::local_framework("APBToygerFacade.framework"),
        IosPbxLinkedFile::local_framework("APPSecuritySDK.framework"),
        IosPbxLinkedFile::local_framework("BioAuthAPI.framework"),
        IosPbxLinkedFile::local_framework("BioAuthEngine.framework"),
        IosPbxLinkedFile::local_framework("deviceiOS.framework"),
        IosPbxLinkedFile::local_framework("DTFIdentityManager.framework"),
        IosPbxLinkedFile::local_framework("DTFSensorServices.framework"),
        IosPbxLinkedFile::local_framework("DTFUIModule.framework"),
        IosPbxLinkedFile::local_framework("DTFUtility.framework"),
        IosPbxLinkedFile::local_framework("MPRemoteLogging.framework"),
        IosPbxLinkedFile::local_framework("ToygerNative.framework"),
        IosPbxLinkedFile::local_framework("ToygerService.framework"),
        IosPbxLinkedFile::system_framework("CoreGraphics.framework"),
        IosPbxLinkedFile::system_framework("Accelerate.framework"),
        IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
        IosPbxLinkedFile::system_framework("AssetsLibrary.framework"),
        IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
        IosPbxLinkedFile::system_framework("QuartzCore.framework"),
        IosPbxLinkedFile::system_framework("CoreFoundation.framework"),
        IosPbxLinkedFile::system_framework("CoreLocation.framework"),
        IosPbxLinkedFile::system_framework("ImageIO.framework"),
        IosPbxLinkedFile::system_framework("CoreMedia.framework"),
        IosPbxLinkedFile::system_framework("CoreMotion.framework"),
        IosPbxLinkedFile::system_framework("AVFoundation.framework"),
        IosPbxLinkedFile::system_framework("WebKit.framework"),
        IosPbxLinkedFile::system_framework("AudioToolbox.framework"),
        IosPbxLinkedFile::system_framework("CFNetwork.framework"),
        IosPbxLinkedFile::system_framework("MobileCoreServices.framework"),
        IosPbxLinkedFile::system_framework("AdSupport.framework"),
        IosPbxLinkedFile::system_library("libresolv.tbd"),
        IosPbxLinkedFile::system_library("libz.tbd"),
        IosPbxLinkedFile::system_library("libc++.tbd"),
        IosPbxLinkedFile::system_library("libc++.1.tbd"),
        IosPbxLinkedFile::system_library("libc++abi.tbd"),
        IosPbxLinkedFile::system_library("libz.1.2.8.tbd"),
    ]
}

fn copy_ios_resource_bundles(project_root: &Path, project_file: &Path) -> Result<usize, String> {
    let bundle_sources = ios_sdk_resource_bundle_sources(project_root)?;
    let target_dir = ios_project_resource_target_dir(project_root);
    crate::utils::fs::ensure_directory(&target_dir).map_err(|e| e.to_string())?;

    for (bundle, source) in &bundle_sources {
        let target = target_dir.join(bundle);
        if target.is_dir() {
            std::fs::remove_dir_all(&target).map_err(|e| {
                format!("清理 iOS 实人认证资源副本失败 {}: {}", target.display(), e)
            })?;
        } else if target.exists() {
            std::fs::remove_file(&target).map_err(|e| {
                format!("清理 iOS 实人认证资源副本失败 {}: {}", target.display(), e)
            })?;
        }
        crate::utils::fs::copy_recursive(source, &target).map_err(|e| {
            format!(
                "复制 iOS 实人认证资源失败 {} -> {}: {}",
                source.display(),
                target.display(),
                e
            )
        })?;
    }

    let resource_names = bundle_sources
        .iter()
        .map(|(bundle, _)| (*bundle).to_string())
        .collect::<Vec<_>>();
    register_pbx_resources(project_file, &resource_names)?;
    Ok(resource_names.len())
}

fn validate_ios_local_linked_files(
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
                "iOS 实人认证模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn validate_ios_resource_bundles(project_root: &Path) -> Result<(), String> {
    ios_sdk_resource_bundle_sources(project_root).map(|_| ())
}

fn ios_sdk_resource_bundle_sources(
    project_root: &Path,
) -> Result<Vec<(&'static str, PathBuf)>, String> {
    let bundles_dir = ios_sdk_support_dir(project_root)?.join("Bundles");
    let mut sources = Vec::new();
    let mut missing = Vec::new();

    for bundle in IOS_FACIAL_RECOGNITION_VERIFY_BUNDLES {
        let source = bundles_dir.join(bundle);
        if source.is_dir() {
            sources.push((*bundle, source));
        } else {
            missing.push(*bundle);
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "iOS 实人认证模块缺少 SDK 资源文件: {}；请确认用户配置的 DCloud iOS 离线 SDK 包含: {}",
            missing.join("、"),
            bundles_dir.display()
        ));
    }

    Ok(sources)
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
