use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    register_pbx_linked_file_specs, register_pbx_linked_files, register_pbx_resources,
    IosPbxFileSpec, IosPbxLinkedFile,
};
use crate::commands::ios::modules::common::{
    ios_manifest_info_has_detected_module, ios_manifest_module_enabled,
    ios_object_value_normalized, ios_sdk_config_value_enabled,
};
use crate::commands::module::{payment_provider_enabled_for_platform, PaymentProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosOauthProvider {
    Univerify,
    Sina,
    Qq,
    Weixin,
    Apple,
    Google,
    Facebook,
}

impl IosOauthProvider {
    fn label(self) -> &'static str {
        match self {
            Self::Univerify => "一键登录",
            Self::Sina => "新浪微博登录",
            Self::Qq => "QQ 登录",
            Self::Weixin => "微信登录",
            Self::Apple => "苹果授权登录",
            Self::Google => "Google 登录",
            Self::Facebook => "Facebook 登录",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IosOauthIntegration {
    pub(crate) providers: Vec<IosOauthProvider>,
    pub(crate) linked_count: usize,
    pub(crate) resource_count: usize,
    pub(crate) facebook_compat_xcframework_count: usize,
}

impl IosOauthIntegration {
    pub(crate) fn summary(&self) -> String {
        let providers = if self.providers.is_empty() {
            "登录鉴权".to_string()
        } else {
            self.providers
                .iter()
                .map(|provider| provider.label())
                .collect::<Vec<_>>()
                .join("、")
        };
        let facebook_compat = if self.facebook_compat_xcframework_count > 0 {
            "，Facebook 静态 SDK 已做 Xcode 签名兼容"
        } else {
            ""
        };
        format!("{}，自动迁移依赖{}", providers, facebook_compat)
    }
}

pub(crate) fn apply_ios_oauth_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosOauthIntegration>, String> {
    let Some(info) = manifest_info else {
        return Ok(None);
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return Ok(None);
    };
    if !ios_manifest_info_has_detected_module(info, "OAuth")
        || !ios_manifest_module_enabled(manifest, "OAuth")
    {
        return Ok(None);
    }

    let providers = ios_oauth_providers(Some(info)).unwrap_or_default();
    if providers.is_empty() {
        return Ok(None);
    }

    let linked_files = ios_oauth_linked_files(&providers, ios_weixin_payment_enabled(Some(info)));
    validate_ios_oauth_local_linked_files(project_root, &linked_files)?;
    let facebook_xcframework_specs =
        prepare_ios_facebook_xcframework_specs(project_root, &providers)?;
    let resource_sources = ios_oauth_resource_sources(project_root, &providers)?;
    let mut linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    linked_count += register_pbx_linked_file_specs(project_file, &facebook_xcframework_specs)?;
    let resource_count = copy_ios_oauth_resources(project_root, project_file, &resource_sources)?;
    let facebook_compat_xcframework_count = facebook_xcframework_specs.len();

    Ok(Some(IosOauthIntegration {
        providers,
        linked_count,
        resource_count,
        facebook_compat_xcframework_count,
    }))
}

pub(crate) fn ios_oauth_providers(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<Vec<IosOauthProvider>> {
    let info = manifest_info?;
    let manifest = info.manifest_value.as_ref()?;
    if !ios_manifest_info_has_detected_module(info, "OAuth")
        || !ios_manifest_module_enabled(manifest, "OAuth")
    {
        return None;
    }
    let oauth_config = ios_oauth_sdk_config(manifest)?;
    if !ios_sdk_config_value_enabled(oauth_config, Some("ios")) {
        return None;
    }

    let mut providers = Vec::new();
    if ios_oauth_provider_enabled(oauth_config, &["univerify", "igetui", "getui"]) {
        push_ios_oauth_provider(&mut providers, IosOauthProvider::Univerify);
    }
    if ios_oauth_provider_enabled(oauth_config, &["sina", "weibo", "sinaweibo"]) {
        push_ios_oauth_provider(&mut providers, IosOauthProvider::Sina);
    }
    if ios_oauth_provider_enabled(oauth_config, &["qq"]) {
        push_ios_oauth_provider(&mut providers, IosOauthProvider::Qq);
    }
    if ios_oauth_provider_enabled(oauth_config, &["weixin", "wechat", "wx"]) {
        push_ios_oauth_provider(&mut providers, IosOauthProvider::Weixin);
    }
    if ios_oauth_provider_enabled(oauth_config, &["apple", "appleid", "applelogin"]) {
        push_ios_oauth_provider(&mut providers, IosOauthProvider::Apple);
    }
    if ios_oauth_provider_enabled(oauth_config, &["google", "googleid", "googlelogin"]) {
        push_ios_oauth_provider(&mut providers, IosOauthProvider::Google);
    }
    if ios_oauth_provider_enabled(oauth_config, &["facebook", "fb"]) {
        push_ios_oauth_provider(&mut providers, IosOauthProvider::Facebook);
    }

    if providers.is_empty() {
        return None;
    }
    Some(providers)
}

fn ios_oauth_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["oauth", "login", "oauths"]
        .iter()
        .find_map(|key| ios_object_value_normalized(sdk_configs, key))
}

fn ios_oauth_provider_enabled(oauth_config: &serde_json::Value, provider_keys: &[&str]) -> bool {
    let Some(config) = oauth_config.as_object() else {
        return false;
    };
    provider_keys.iter().any(|provider_key| {
        ios_object_value_normalized(config, provider_key)
            .is_some_and(|value| ios_sdk_config_value_enabled(value, Some("ios")))
    })
}

fn push_ios_oauth_provider(providers: &mut Vec<IosOauthProvider>, provider: IosOauthProvider) {
    if !providers.contains(&provider) {
        providers.push(provider);
    }
}

fn ios_oauth_linked_files(
    providers: &[IosOauthProvider],
    use_weixin_pay_sdk: bool,
) -> Vec<IosPbxLinkedFile> {
    let mut files = Vec::new();
    push_ios_linked_file(&mut files, IosPbxLinkedFile::local_static("liblibOauth.a"));

    if providers.contains(&IosOauthProvider::Univerify) {
        for file in [
            IosPbxLinkedFile::local_framework("UniVerify.framework"),
            IosPbxLinkedFile::local_xcframework("GTCommonSDK.xcframework"),
            IosPbxLinkedFile::local_xcframework("GeYanSdk.xcframework"),
            IosPbxLinkedFile::system_library("libz.tbd"),
            IosPbxLinkedFile::system_library("libc++.tbd"),
            IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
            IosPbxLinkedFile::system_framework("AdSupport.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosOauthProvider::Sina) {
        for file in [
            IosPbxLinkedFile::local_static("libSinaWBOauth.a"),
            IosPbxLinkedFile::local_static("libWeiboSDK.a"),
            IosPbxLinkedFile::system_framework("ImageIO.framework"),
            IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosOauthProvider::Qq) {
        for file in [
            IosPbxLinkedFile::local_static("libQQOauth.a"),
            IosPbxLinkedFile::local_xcframework("TencentOpenAPI.xcframework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosOauthProvider::Weixin) {
        for file in [
            IosPbxLinkedFile::local_static("libWXOauth.a"),
            IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
            IosPbxLinkedFile::system_library("libz.tbd"),
            IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
        if !use_weixin_pay_sdk {
            push_ios_linked_file(&mut files, IosPbxLinkedFile::local_static("libWeChatSDK.a"));
        }
    }

    if providers.contains(&IosOauthProvider::Apple) {
        for file in [
            IosPbxLinkedFile::local_static("libAppleOauth.a"),
            IosPbxLinkedFile::optional_system_framework("AuthenticationServices.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosOauthProvider::Google) {
        for file in [
            IosPbxLinkedFile::local_static("libGoogleOauth.a"),
            IosPbxLinkedFile::local_xcframework("GoogleSignIn.xcframework"),
            IosPbxLinkedFile::local_xcframework("AppAuth.xcframework"),
            IosPbxLinkedFile::local_xcframework("GTMAppAuth.xcframework"),
            IosPbxLinkedFile::local_xcframework("GTMSessionFetcher.xcframework"),
            IosPbxLinkedFile::system_framework("CoreText.framework"),
            IosPbxLinkedFile::system_framework("CoreGraphics.framework"),
            IosPbxLinkedFile::system_framework("LocalAuthentication.framework"),
            IosPbxLinkedFile::system_framework("SafariServices.framework"),
            IosPbxLinkedFile::system_framework("Security.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosOauthProvider::Facebook) {
        for file in [
            IosPbxLinkedFile::local_static("libFBOauth.a"),
            IosPbxLinkedFile::system_library("libc++.tbd"),
            IosPbxLinkedFile::system_framework("Accelerate.framework"),
            IosPbxLinkedFile::system_framework("Accounts.framework"),
            IosPbxLinkedFile::system_framework("AdSupport.framework"),
            IosPbxLinkedFile::system_framework("AudioToolbox.framework"),
            IosPbxLinkedFile::system_framework("CoreGraphics.framework"),
            IosPbxLinkedFile::system_framework("QuartzCore.framework"),
            IosPbxLinkedFile::system_framework("Security.framework"),
            IosPbxLinkedFile::system_framework("Social.framework"),
            IosPbxLinkedFile::system_framework("StoreKit.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    files
}

fn ios_weixin_payment_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    manifest_info
        .and_then(|info| info.manifest_value.as_ref())
        .is_some_and(|manifest| {
            payment_provider_enabled_for_platform(manifest, PaymentProvider::Weixin, "ios")
        })
}

fn push_ios_linked_file(files: &mut Vec<IosPbxLinkedFile>, file: IosPbxLinkedFile) {
    if !files.iter().any(|existing| existing.name == file.name) {
        files.push(file);
    }
}

fn validate_ios_oauth_local_linked_files(
    project_root: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    for file in files.iter().copied().filter(|file| file.is_local()) {
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS Oauth 模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

const IOS_FACEBOOK_XCFRAMEWORKS: &[&str] = &[
    "FBSDKCoreKit.xcframework",
    "FBAEMKit.xcframework",
    "FBSDKCoreKit_Basics.xcframework",
    "FBSDKLoginKit.xcframework",
];

fn prepare_ios_facebook_xcframework_specs(
    project_root: &Path,
    providers: &[IosOauthProvider],
) -> Result<Vec<IosPbxFileSpec>, String> {
    if !providers.contains(&IosOauthProvider::Facebook) {
        return Ok(Vec::new());
    }

    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    let target_dir = project_root.join("UniPackSanitizedSDK");
    crate::utils::fs::ensure_directory(&target_dir).map_err(|e| e.to_string())?;

    let mut specs = Vec::new();
    for name in IOS_FACEBOOK_XCFRAMEWORKS {
        let source = libs_dir.join(name);
        if !source.exists() {
            return Err(format!(
                "iOS Oauth 模块缺少 SDK 依赖文件: {}",
                source.display()
            ));
        }

        let target = target_dir.join(name);
        remove_file_or_dir_if_exists(&target).map_err(|e| {
            format!(
                "清理 iOS Oauth Facebook SDK 兼容副本失败 {}: {}",
                target.display(),
                e
            )
        })?;
        crate::utils::fs::copy_recursive(&source, &target).map_err(|e| {
            format!(
                "复制 iOS Oauth Facebook SDK 兼容副本失败 {} -> {}: {}",
                source.display(),
                target.display(),
                e
            )
        })?;
        remove_code_signature_dirs(&target).map_err(|e| {
            format!(
                "清理 iOS Oauth Facebook SDK 签名目录失败 {}: {}",
                target.display(),
                e
            )
        })?;
        specs.push(IosPbxFileSpec::project_xcframework(
            (*name).to_string(),
            format!("UniPackSanitizedSDK/{}", name),
        ));
    }

    Ok(specs)
}

fn remove_file_or_dir_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(path),
        Ok(_) => std::fs::remove_file(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_code_signature_dirs(root: &Path) -> std::io::Result<usize> {
    let mut removed = 0usize;
    remove_code_signature_dirs_inner(root, &mut removed)?;
    Ok(removed)
}

fn remove_code_signature_dirs_inner(root: &Path, removed: &mut usize) -> std::io::Result<()> {
    if !root.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_dir() {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_CodeSignature") {
            std::fs::remove_dir_all(&path)?;
            *removed += 1;
        } else {
            remove_code_signature_dirs_inner(&path, removed)?;
        }
    }
    Ok(())
}

fn ios_oauth_resource_sources(
    project_root: &Path,
    providers: &[IosOauthProvider],
) -> Result<Vec<(String, PathBuf)>, String> {
    let bundles_dir = ios_sdk_support_dir(project_root)?.join("Bundles");
    let mut resources = Vec::new();
    if providers.contains(&IosOauthProvider::Univerify) {
        push_ios_oauth_resource(&mut resources, &bundles_dir, "TYRZResource.bundle")?;
    }
    if providers.contains(&IosOauthProvider::Sina) {
        push_ios_oauth_resource(&mut resources, &bundles_dir, "WeiboSDK.bundle")?;
    }
    if providers.contains(&IosOauthProvider::Google) {
        push_ios_oauth_resource(&mut resources, &bundles_dir, "GoogleSignIn.bundle")?;
    }
    Ok(resources)
}

fn push_ios_oauth_resource(
    resources: &mut Vec<(String, PathBuf)>,
    bundles_dir: &Path,
    name: &str,
) -> Result<(), String> {
    if resources.iter().any(|(existing, _)| existing == name) {
        return Ok(());
    }
    let source = bundles_dir.join(name);
    if !source.exists() {
        return Err(format!(
            "iOS Oauth 模块缺少 SDK 资源文件: {} ({})",
            name,
            bundles_dir.display()
        ));
    }
    resources.push((name.to_string(), source));
    Ok(())
}

fn copy_ios_oauth_resources(
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
                .map_err(|e| format!("清理 iOS Oauth 资源副本失败 {}: {}", target.display(), e))?;
        } else if target.exists() {
            std::fs::remove_file(&target)
                .map_err(|e| format!("清理 iOS Oauth 资源副本失败 {}: {}", target.display(), e))?;
        }
        if source.is_dir() {
            crate::utils::fs::copy_recursive(source, &target).map_err(|e| {
                format!(
                    "复制 iOS Oauth 资源失败 {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        } else {
            std::fs::copy(source, &target).map_err(|e| {
                format!(
                    "复制 iOS Oauth 资源失败 {} -> {}: {}",
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
