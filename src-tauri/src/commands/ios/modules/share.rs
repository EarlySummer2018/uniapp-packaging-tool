use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    register_pbx_linked_files, register_pbx_resources, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    ios_manifest_info_has_detected_module, ios_manifest_module_enabled,
    ios_object_value_normalized, ios_sdk_config_value_enabled,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosShareProvider {
    Weixin,
    Qq,
    Sina,
}

impl IosShareProvider {
    fn label(self) -> &'static str {
        match self {
            Self::Weixin => "微信分享",
            Self::Qq => "QQ 分享",
            Self::Sina => "新浪微博分享",
        }
    }

    fn pod_name(self) -> &'static str {
        match self {
            Self::Weixin => "Share-Wechat",
            Self::Qq => "Share-QQ",
            Self::Sina => "Share-Sina",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IosShareIntegration {
    pub(crate) providers: Vec<IosShareProvider>,
    pub(crate) local_pod: bool,
    pub(crate) linked_count: usize,
    pub(crate) resource_count: usize,
}

impl IosShareIntegration {
    pub(crate) fn summary(&self) -> String {
        let providers = self
            .providers
            .iter()
            .map(|provider| provider.label())
            .collect::<Vec<_>>()
            .join("、");
        if self.local_pod {
            let pods = self
                .providers
                .iter()
                .map(|provider| provider.pod_name())
                .collect::<Vec<_>>()
                .join("、");
            format!("{}，本地 Pod 集成 Share、{}", providers, pods)
        } else {
            format!("{}，手动 SDK 集成", providers)
        }
    }
}

pub(crate) fn ios_share_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_share_providers(manifest_info).is_some()
}

pub(crate) fn apply_ios_share_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosShareIntegration>, String> {
    let Some(info) = manifest_info else {
        return Ok(None);
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return Ok(None);
    };
    let Some(providers) = ios_share_providers(Some(info)) else {
        return Ok(None);
    };
    let local_pod = ios_share_local_pod_enabled(manifest);

    if local_pod {
        return Ok(Some(IosShareIntegration {
            providers,
            local_pod,
            linked_count: 0,
            resource_count: 0,
        }));
    }

    let linked_files = ios_share_linked_files(&providers);
    validate_ios_share_local_linked_files(project_root, &linked_files)?;
    let resource_sources = ios_share_resource_sources(project_root, &providers)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    let resource_count = copy_ios_share_resources(project_root, project_file, &resource_sources)?;

    Ok(Some(IosShareIntegration {
        providers,
        local_pod,
        linked_count,
        resource_count,
    }))
}

pub(crate) fn ios_share_providers(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<Vec<IosShareProvider>> {
    let info = manifest_info?;
    let manifest = info.manifest_value.as_ref()?;
    if !ios_manifest_info_has_detected_module(info, "Share")
        || !ios_manifest_module_enabled(manifest, "Share")
    {
        return None;
    }
    let share_config = ios_share_sdk_config(manifest)?;
    if !ios_sdk_config_value_enabled(share_config, Some("ios")) {
        return None;
    }

    let mut providers = Vec::new();
    if ios_share_provider_enabled(share_config, &["weixin", "wechat", "wx"]) {
        push_ios_share_provider(&mut providers, IosShareProvider::Weixin);
    }
    if ios_share_provider_enabled(share_config, &["qq"]) {
        push_ios_share_provider(&mut providers, IosShareProvider::Qq);
    }
    if ios_share_provider_enabled(share_config, &["sina", "weibo", "sinaweibo"]) {
        push_ios_share_provider(&mut providers, IosShareProvider::Sina);
    }

    if providers.is_empty() {
        return None;
    }
    Some(providers)
}

pub(crate) fn ios_share_local_pod_enabled(manifest: &serde_json::Value) -> bool {
    ios_share_sdk_config(manifest)
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

fn ios_share_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["share", "shares"]
        .iter()
        .find_map(|key| ios_object_value_normalized(sdk_configs, key))
}

fn ios_share_provider_enabled(share_config: &serde_json::Value, provider_keys: &[&str]) -> bool {
    let Some(config) = share_config.as_object() else {
        return false;
    };
    provider_keys.iter().any(|provider_key| {
        ios_object_value_normalized(config, provider_key)
            .is_some_and(|value| ios_sdk_config_value_enabled(value, Some("ios")))
    })
}

fn push_ios_share_provider(providers: &mut Vec<IosShareProvider>, provider: IosShareProvider) {
    if !providers.contains(&provider) {
        providers.push(provider);
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

fn ios_share_linked_files(providers: &[IosShareProvider]) -> Vec<IosPbxLinkedFile> {
    let mut files = Vec::new();
    push_ios_linked_file(&mut files, IosPbxLinkedFile::local_static("liblibShare.a"));

    if providers.contains(&IosShareProvider::Weixin) {
        for file in [
            IosPbxLinkedFile::local_static("libweixinShare.a"),
            IosPbxLinkedFile::local_static("libWeChatSDK.a"),
            IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
            IosPbxLinkedFile::system_library("libz.tbd"),
            IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosShareProvider::Qq) {
        push_ios_linked_file(&mut files, IosPbxLinkedFile::local_static("libQQShare.a"));
        push_ios_linked_file(
            &mut files,
            IosPbxLinkedFile::local_xcframework("TencentOpenAPI.xcframework"),
        );
    }

    if providers.contains(&IosShareProvider::Sina) {
        for file in [
            IosPbxLinkedFile::local_static("libSinaShare.a"),
            IosPbxLinkedFile::local_static("libWeiboSDK.a"),
            IosPbxLinkedFile::system_framework("ImageIO.framework"),
            IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
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

fn validate_ios_share_local_linked_files(
    project_root: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    for file in files.iter().copied().filter(|file| file.is_local()) {
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS 分享模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn ios_share_resource_sources(
    project_root: &Path,
    providers: &[IosShareProvider],
) -> Result<Vec<(String, PathBuf)>, String> {
    if !providers.contains(&IosShareProvider::Sina) {
        return Ok(Vec::new());
    }

    let bundles_dir = ios_sdk_support_dir(project_root)?.join("Bundles");
    let source = bundles_dir.join("WeiboSDK.bundle");
    if !source.exists() {
        return Err(format!(
            "iOS 分享模块缺少 SDK 资源文件: WeiboSDK.bundle ({})",
            bundles_dir.display()
        ));
    }
    Ok(vec![("WeiboSDK.bundle".to_string(), source)])
}

fn copy_ios_share_resources(
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
                .map_err(|e| format!("清理 iOS 分享资源副本失败 {}: {}", target.display(), e))?;
        } else if target.exists() {
            std::fs::remove_file(&target)
                .map_err(|e| format!("清理 iOS 分享资源副本失败 {}: {}", target.display(), e))?;
        }
        if source.is_dir() {
            crate::utils::fs::copy_recursive(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 分享资源失败 {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        } else {
            std::fs::copy(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 分享资源失败 {} -> {}: {}",
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
