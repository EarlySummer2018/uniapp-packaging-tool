use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    register_pbx_linked_files, register_pbx_resources, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_manifest_info_has_detected_module,
    ios_manifest_module_enabled, ios_object_value_normalized, ios_sdk_config_value_enabled,
    IosPrivacyFieldSpec,
};

pub(crate) const IOS_MAP_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSLocationWhenInUseUsageDescription",
        label: "使用期间定位说明",
        default_value: "我们需要使用您的位置信息来显示当前位置",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSLocationAlwaysAndWhenInUseUsageDescription",
        label: "始终和使用期间定位说明",
        default_value: "我们需要使用您的位置信息来提供持续导航服务",
        required: true,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosMapProvider {
    Baidu,
    Amap,
    Google,
}

impl IosMapProvider {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Baidu => "百度地图",
            Self::Amap => "高德地图",
            Self::Google => "Google 地图",
        }
    }

    fn pod_name(self) -> &'static str {
        match self {
            Self::Baidu => "Map-Baidu",
            Self::Amap => "Map-Gaode",
            Self::Google => "Map-Google",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosMapPageType {
    Vue,
    Nvue,
}

impl IosMapPageType {
    fn label(self) -> &'static str {
        match self {
            Self::Vue => "vue",
            Self::Nvue => "nvue",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IosMapIntegration {
    pub(crate) provider: IosMapProvider,
    pub(crate) page_type: IosMapPageType,
    pub(crate) local_pod: bool,
    pub(crate) linked_count: usize,
    pub(crate) resource_count: usize,
}

impl IosMapIntegration {
    pub(crate) fn summary(&self) -> String {
        if self.local_pod {
            format!(
                "{}，{} 页面，本地 Pod 集成 {}",
                self.provider.label(),
                self.page_type.label(),
                self.provider.pod_name()
            )
        } else {
            format!(
                "{}，{} 页面，手动 SDK 集成",
                self.provider.label(),
                self.page_type.label()
            )
        }
    }
}

pub(crate) fn ios_map_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    let Some(info) = manifest_info else {
        return false;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return false;
    };
    ios_manifest_info_has_detected_module(info, "Map")
        && ios_manifest_module_enabled(manifest, "Map")
        && ios_map_provider(manifest).is_some()
}

pub(crate) fn apply_ios_map_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosMapIntegration>, String> {
    let Some(info) = manifest_info else {
        return Ok(None);
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return Ok(None);
    };
    if !ios_manifest_info_has_detected_module(info, "Map")
        || !ios_manifest_module_enabled(manifest, "Map")
    {
        return Ok(None);
    }
    let Some(provider) = ios_map_provider(manifest) else {
        return Ok(None);
    };
    let page_type = ios_map_page_type(manifest, provider);
    let local_pod = ios_map_local_pod_enabled(manifest);

    if local_pod {
        return Ok(Some(IosMapIntegration {
            provider,
            page_type,
            local_pod,
            linked_count: 0,
            resource_count: 0,
        }));
    }

    let linked_files = ios_map_linked_files(provider, page_type);
    validate_ios_map_local_linked_files(project_root, &linked_files)?;
    let resource_sources = ios_map_resource_sources(project_root, provider, page_type)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    let resource_count = copy_ios_map_resources(project_root, project_file, &resource_sources)?;

    Ok(Some(IosMapIntegration {
        provider,
        page_type,
        local_pod,
        linked_count,
        resource_count,
    }))
}

pub(crate) fn apply_ios_map_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_MAP_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, field.default_value);
    }
}

pub(crate) fn ios_map_provider(manifest: &serde_json::Value) -> Option<IosMapProvider> {
    let config_value = ios_map_sdk_config(manifest)?;
    if !ios_sdk_config_value_enabled(config_value, None) {
        return None;
    }
    let config = config_value.as_object()?;
    if ios_map_provider_enabled(config, &["baidu", "bd"]) {
        Some(IosMapProvider::Baidu)
    } else if ios_map_provider_enabled(config, &["amap", "gaode"]) {
        Some(IosMapProvider::Amap)
    } else if ios_map_provider_enabled(config, &["google", "googleMap"]) {
        Some(IosMapProvider::Google)
    } else {
        None
    }
}

pub(crate) fn ios_map_page_type(
    manifest: &serde_json::Value,
    provider: IosMapProvider,
) -> IosMapPageType {
    let default = match provider {
        IosMapProvider::Amap => IosMapPageType::Nvue,
        IosMapProvider::Baidu | IosMapProvider::Google => IosMapPageType::Vue,
    };
    let Some(value) = ios_map_sdk_config(manifest).and_then(|config| {
        let config = config.as_object()?;
        ["pageType", "page_type", "MAP_PAGE_TYPE", "page"]
            .iter()
            .find_map(|key| ios_object_value_normalized(config, key))
            .and_then(serde_json::Value::as_str)
    }) else {
        return default;
    };
    normalize_ios_map_page_type(provider, value)
}

pub(crate) fn ios_map_local_pod_enabled(manifest: &serde_json::Value) -> bool {
    ios_map_sdk_config(manifest)
        .and_then(|config| {
            let config = config.as_object()?;
            [
                "localPod",
                "local_pod",
                "useLocalPod",
                "use_local_pod",
                "LOCAL_POD",
            ]
            .iter()
            .find_map(|key| ios_object_value_normalized(config, key))
        })
        .is_some_and(ios_bool_value_enabled)
}

fn ios_map_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["maps", "map"]
        .iter()
        .find_map(|key| ios_object_value_normalized(sdk_configs, key))
}

fn ios_map_provider_enabled(
    config: &serde_json::Map<String, serde_json::Value>,
    provider_keys: &[&str],
) -> bool {
    provider_keys.iter().any(|provider_key| {
        ios_object_value_normalized(config, provider_key)
            .is_some_and(ios_sdk_config_value_enabled_for_map_provider)
    })
}

fn ios_sdk_config_value_enabled_for_map_provider(value: &serde_json::Value) -> bool {
    ios_sdk_config_value_enabled(value, None)
}

fn normalize_ios_map_page_type(provider: IosMapProvider, value: &str) -> IosMapPageType {
    match provider {
        IosMapProvider::Baidu => IosMapPageType::Vue,
        IosMapProvider::Amap => IosMapPageType::Nvue,
        IosMapProvider::Google if normalized(value) == "nvue" => IosMapPageType::Nvue,
        IosMapProvider::Google => IosMapPageType::Vue,
    }
}

fn ios_bool_value_enabled(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(flag) => *flag,
        serde_json::Value::String(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on" | "是" | "开启"
        ),
        serde_json::Value::Number(value) => value.as_i64().is_some_and(|value| value != 0),
        _ => false,
    }
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn ios_map_linked_files(
    provider: IosMapProvider,
    page_type: IosMapPageType,
) -> Vec<IosPbxLinkedFile> {
    let mut files = Vec::new();
    push_ios_linked_file(&mut files, IosPbxLinkedFile::local_static("liblibMap.a"));

    match provider {
        IosMapProvider::Baidu => {
            for file in [
                IosPbxLinkedFile::local_static("libbmapimp.a"),
                IosPbxLinkedFile::local_static("libBaiduKeyVerify.a"),
                IosPbxLinkedFile::local_static("libssl.a"),
                IosPbxLinkedFile::local_static("libcrypto.a"),
                IosPbxLinkedFile::local_framework("BaiduMapAPI_Utils.framework"),
                IosPbxLinkedFile::local_framework("BaiduMapAPI_Base.framework"),
                IosPbxLinkedFile::local_framework("BaiduMapAPI_Search.framework"),
                IosPbxLinkedFile::local_framework("BaiduMapAPI_Map.framework"),
                IosPbxLinkedFile::local_framework("BMKLocationKit.framework"),
                IosPbxLinkedFile::system_library("libc++.tbd"),
                IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
                IosPbxLinkedFile::system_library("libz.tbd"),
                IosPbxLinkedFile::system_framework("QuartzCore.framework"),
                IosPbxLinkedFile::system_framework("CoreGraphics.framework"),
                IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
                IosPbxLinkedFile::system_framework("Accelerate.framework"),
                IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
                IosPbxLinkedFile::system_framework("Security.framework"),
                IosPbxLinkedFile::system_framework("MapKit.framework"),
                IosPbxLinkedFile::system_framework("OpenGLES.framework"),
                IosPbxLinkedFile::system_framework("CoreLocation.framework"),
            ] {
                push_ios_linked_file(&mut files, file);
            }
        }
        IosMapProvider::Amap => {
            if page_type == IosMapPageType::Nvue {
                for file in [
                    IosPbxLinkedFile::local_static("libDCUniMap.a"),
                    IosPbxLinkedFile::local_static("libDCUniAmap.a"),
                    IosPbxLinkedFile::local_framework("Masonry.framework"),
                ] {
                    push_ios_linked_file(&mut files, file);
                }
            } else {
                push_ios_linked_file(&mut files, IosPbxLinkedFile::local_static("libAMapImp.a"));
            }
            for file in [
                IosPbxLinkedFile::local_framework("AMapSearchKit.framework"),
                IosPbxLinkedFile::local_framework("MAMapKit.framework"),
                IosPbxLinkedFile::local_framework("AMapFoundationKit.framework"),
                IosPbxLinkedFile::system_framework("MapKit.framework"),
                IosPbxLinkedFile::system_framework("CoreLocation.framework"),
                IosPbxLinkedFile::system_library("libc++.tbd"),
                IosPbxLinkedFile::system_framework("GLKit.framework"),
            ] {
                push_ios_linked_file(&mut files, file);
            }
        }
        IosMapProvider::Google => {
            for file in [
                IosPbxLinkedFile::local_static("libDCUniMap.a"),
                IosPbxLinkedFile::local_static("libDCUniGoogleMap.a"),
                IosPbxLinkedFile::local_framework("GoogleMapsBase.framework"),
                IosPbxLinkedFile::local_framework("GoogleMaps.framework"),
                IosPbxLinkedFile::local_framework("GoogleMapsCore.framework"),
                IosPbxLinkedFile::system_framework("Accelerate.framework"),
                IosPbxLinkedFile::system_framework("CoreData.framework"),
                IosPbxLinkedFile::system_framework("CoreGraphics.framework"),
                IosPbxLinkedFile::system_framework("CoreImage.framework"),
                IosPbxLinkedFile::system_framework("CoreLocation.framework"),
                IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
                IosPbxLinkedFile::system_framework("CoreText.framework"),
                IosPbxLinkedFile::system_framework("GLKit.framework"),
                IosPbxLinkedFile::system_framework("ImageIO.framework"),
                IosPbxLinkedFile::system_library("libc++.tbd"),
                IosPbxLinkedFile::system_library("libz.tbd"),
                IosPbxLinkedFile::system_framework("Metal.framework"),
                IosPbxLinkedFile::system_framework("OpenGLES.framework"),
                IosPbxLinkedFile::system_framework("QuartzCore.framework"),
                IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
            ] {
                push_ios_linked_file(&mut files, file);
            }
        }
    }
    files
}

fn push_ios_linked_file(files: &mut Vec<IosPbxLinkedFile>, file: IosPbxLinkedFile) {
    if !files.iter().any(|existing| existing.name == file.name) {
        files.push(file);
    }
}

fn validate_ios_map_local_linked_files(
    project_root: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    for file in files.iter().copied().filter(|file| file.is_local()) {
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS 地图模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn ios_map_resource_sources(
    project_root: &Path,
    provider: IosMapProvider,
    page_type: IosMapPageType,
) -> Result<Vec<(String, PathBuf)>, String> {
    let bundles_dir = ios_sdk_support_dir(project_root)?.join("Bundles");
    let mut names = match provider {
        IosMapProvider::Baidu => vec!["mapapi.bundle"],
        IosMapProvider::Amap => vec!["AMap.bundle", "userPosition@2x.png"],
        IosMapProvider::Google => vec!["GoogleMaps.bundle"],
    };
    if provider == IosMapProvider::Amap && page_type == IosMapPageType::Vue {
        names.retain(|name| *name != "userPosition@2x.png");
    }

    let mut missing = Vec::new();
    let mut sources = Vec::new();
    for name in names {
        let source = bundles_dir.join(name);
        if source.exists() {
            sources.push((name.to_string(), source));
        } else {
            missing.push(name);
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "iOS 地图模块缺少 SDK 资源文件: {} ({})",
            missing.join("、"),
            bundles_dir.display()
        ));
    }
    Ok(sources)
}

fn copy_ios_map_resources(
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
                .map_err(|e| format!("清理 iOS 地图资源副本失败 {}: {}", target.display(), e))?;
        } else if target.exists() {
            std::fs::remove_file(&target)
                .map_err(|e| format!("清理 iOS 地图资源副本失败 {}: {}", target.display(), e))?;
        }
        if source.is_dir() {
            crate::utils::fs::copy_recursive(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 地图资源失败 {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        } else {
            std::fs::copy(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 地图资源失败 {} -> {}: {}",
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
