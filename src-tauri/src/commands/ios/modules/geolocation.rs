use std::path::Path;

use crate::commands::ios::build::pbxproj::{register_pbx_linked_files, IosPbxLinkedFile};
use crate::commands::ios::modules::common::{
    insert_missing_plist_string, ios_config_value_applies_to_platform_strict,
    ios_manifest_info_has_detected_module, ios_manifest_module_enabled,
    ios_object_value_normalized, ios_sdk_config_value_enabled, IosPrivacyFieldSpec,
};

pub(crate) const IOS_GEOLOCATION_PRIVACY_FIELDS: &[IosPrivacyFieldSpec] = &[
    IosPrivacyFieldSpec {
        key: "NSLocationUsageDescription",
        label: "定位权限说明",
        default_value: "用于获取当前位置以提供定位相关服务",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSLocationWhenInUseUsageDescription",
        label: "使用期间定位说明",
        default_value: "用于在使用应用期间获取当前位置",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSLocationAlwaysUsageDescription",
        label: "持续定位说明",
        default_value: "用于在需要时持续获取位置以提供定位相关服务",
        required: true,
    },
    IosPrivacyFieldSpec {
        key: "NSLocationAlwaysAndWhenInUseUsageDescription",
        label: "始终和使用期间定位说明",
        default_value: "用于在使用期间或后台需要时获取位置",
        required: true,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosGeolocationProvider {
    System,
    Amap,
    Baidu,
}

impl IosGeolocationProvider {
    fn label(self) -> &'static str {
        match self {
            Self::System => "系统定位",
            Self::Amap => "高德定位",
            Self::Baidu => "百度定位",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IosGeolocationIntegration {
    pub(crate) providers: Vec<IosGeolocationProvider>,
    pub(crate) linked_count: usize,
}

impl IosGeolocationIntegration {
    pub(crate) fn provider_summary(&self) -> String {
        self.providers
            .iter()
            .map(|provider| provider.label())
            .collect::<Vec<_>>()
            .join("、")
    }
}

pub(crate) fn apply_ios_geolocation_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosGeolocationIntegration>, String> {
    let Some(providers) = ios_geolocation_providers(manifest_info) else {
        return Ok(None);
    };
    let linked_files = ios_geolocation_linked_files(&providers);
    validate_ios_local_linked_files(project_root, &linked_files)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    Ok(Some(IosGeolocationIntegration {
        providers,
        linked_count,
    }))
}

pub(crate) fn ios_geolocation_providers(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<Vec<IosGeolocationProvider>> {
    let info = manifest_info?;
    let manifest = info.manifest_value.as_ref()?;
    if !ios_manifest_info_has_detected_module(info, "Geolocation") {
        return None;
    }
    if !ios_manifest_module_enabled(manifest, "Geolocation") {
        return None;
    }
    let geolocation_config = ios_geolocation_sdk_config(manifest);
    if geolocation_config.is_some_and(|value| !ios_sdk_config_value_enabled(value, Some("ios"))) {
        return None;
    }

    let mut providers = Vec::new();
    if ios_geolocation_provider_enabled(geolocation_config, &["system"]) {
        push_ios_geolocation_provider(&mut providers, IosGeolocationProvider::System);
    }
    if ios_geolocation_provider_enabled(geolocation_config, &["baidu", "bd"]) {
        push_ios_geolocation_provider(&mut providers, IosGeolocationProvider::Baidu);
    }
    if ios_geolocation_provider_enabled(geolocation_config, &["amap", "gaode"]) {
        push_ios_geolocation_provider(&mut providers, IosGeolocationProvider::Amap);
    }

    if providers.is_empty() {
        return None;
    }
    Some(providers)
}

pub(crate) fn apply_ios_geolocation_privacy_defaults(dict: &mut plist::Dictionary) {
    for field in IOS_GEOLOCATION_PRIVACY_FIELDS {
        insert_missing_plist_string(dict, field.key, "用于获取当前位置以提供定位相关服务");
    }
}

fn ios_geolocation_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .get("geolocation")
        .or_else(|| {
            manifest
                .get("app-plus")?
                .get("distribute")?
                .get("sdkConfigs")?
                .get("location")
        })
        .or_else(|| {
            manifest
                .get("app-plus")?
                .get("distribute")?
                .get("sdkConfigs")?
                .get("position")
        })
}

fn ios_geolocation_provider_enabled(
    geolocation_config: Option<&serde_json::Value>,
    provider_names: &[&str],
) -> bool {
    let Some(config) = geolocation_config.and_then(serde_json::Value::as_object) else {
        return false;
    };
    provider_names.iter().any(|name| {
        ios_object_value_normalized(config, name)
            .is_some_and(ios_geolocation_provider_value_enabled)
    })
}

pub(crate) fn ios_geolocation_provider_value_enabled(value: &serde_json::Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    let enabled = map
        .get("enabled")
        .or_else(|| map.get("enable"))
        .or_else(|| map.get("open"))
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    enabled && ios_config_value_applies_to_platform_strict(map, "ios")
}

fn push_ios_geolocation_provider(
    providers: &mut Vec<IosGeolocationProvider>,
    provider: IosGeolocationProvider,
) {
    if !providers.contains(&provider) {
        providers.push(provider);
    }
}

fn ios_geolocation_linked_files(providers: &[IosGeolocationProvider]) -> Vec<IosPbxLinkedFile> {
    let mut files = Vec::new();
    push_ios_linked_file(
        &mut files,
        IosPbxLinkedFile::local_static("liblibGeolocation.a"),
    );
    push_ios_linked_file(
        &mut files,
        IosPbxLinkedFile::system_framework("Foundation.framework"),
    );
    push_ios_linked_file(
        &mut files,
        IosPbxLinkedFile::system_framework("CoreLocation.framework"),
    );

    if providers.contains(&IosGeolocationProvider::Baidu) {
        for file in [
            IosPbxLinkedFile::local_static("libBaiduLocationPlugin.a"),
            IosPbxLinkedFile::local_static("libBaiduKeyVerify.a"),
            IosPbxLinkedFile::local_static("libssl.a"),
            IosPbxLinkedFile::local_static("libcrypto.a"),
            IosPbxLinkedFile::local_framework("BaiduMapAPI_Utils.framework"),
            IosPbxLinkedFile::local_framework("BaiduMapAPI_Base.framework"),
            IosPbxLinkedFile::local_framework("BaiduMapAPI_Search.framework"),
            IosPbxLinkedFile::local_framework("BMKLocationKit.framework"),
            IosPbxLinkedFile::system_library("libc++.tbd"),
            IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
            IosPbxLinkedFile::system_framework("Security.framework"),
            IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosGeolocationProvider::Amap) {
        for file in [
            IosPbxLinkedFile::local_static("libAMapLocationPlugin.a"),
            IosPbxLinkedFile::local_framework("AMapFoundationKit.framework"),
            IosPbxLinkedFile::local_framework("AMapLocationKit.framework"),
            IosPbxLinkedFile::system_library("libc++.tbd"),
            IosPbxLinkedFile::system_library("libz.tbd"),
            IosPbxLinkedFile::system_framework("ExternalAccessory.framework"),
            IosPbxLinkedFile::system_framework("GLKit.framework"),
            IosPbxLinkedFile::system_framework("Security.framework"),
            IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
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
                "iOS 定位模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}
