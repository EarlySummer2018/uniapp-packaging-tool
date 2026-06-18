use std::path::{Path, PathBuf};

use crate::commands::ios::build::pbxproj::{
    raise_pbx_ios_deployment_target, register_pbx_linked_files, register_pbx_resources,
    remove_pbx_linked_or_embedded_files, IosPbxLinkedFile,
};
use crate::commands::module::{
    manifest_payment_provider_value, payment_provider_enabled_for_platform, PaymentProvider,
};

const ALIPAY_SDK_FRAMEWORK: &str = "AlipaySDK.framework";
const ALIPAY_SDK_XCFRAMEWORK: &str = "AlipaySDK.xcframework";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IosPaymentProvider {
    Alipay,
    Weixin,
    Paypal,
    Stripe,
    Iap,
}

impl IosPaymentProvider {
    fn label(self) -> &'static str {
        match self {
            Self::Alipay => "支付宝",
            Self::Weixin => "微信支付",
            Self::Paypal => "PayPal",
            Self::Stripe => "Stripe",
            Self::Iap => "Apple IAP",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IosPaymentIntegration {
    pub(crate) providers: Vec<IosPaymentProvider>,
    pub(crate) linked_count: usize,
    pub(crate) resource_count: usize,
}

impl IosPaymentIntegration {
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

pub(crate) fn apply_ios_payment_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosPaymentIntegration>, String> {
    let Some(info) = manifest_info else {
        return Ok(None);
    };
    let Some(providers) = ios_payment_providers(Some(info)) else {
        return Ok(None);
    };

    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    let linked_files = ios_payment_linked_files(&providers, &libs_dir);
    validate_ios_payment_local_linked_files(&libs_dir, &linked_files)?;
    let resource_sources = ios_payment_resource_sources(project_root, &providers)?;
    if providers.contains(&IosPaymentProvider::Weixin) {
        remove_pbx_linked_or_embedded_files(project_file, &["libWeChatSDK.a"])?;
    }
    if ios_payment_requires_ios_13(&providers) {
        raise_pbx_ios_deployment_target(project_file, "13.0")?;
    }
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;
    let resource_count = copy_ios_payment_resources(project_root, project_file, &resource_sources)?;

    Ok(Some(IosPaymentIntegration {
        providers,
        linked_count,
        resource_count,
    }))
}

pub(crate) fn ios_payment_providers(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<Vec<IosPaymentProvider>> {
    let info = manifest_info?;
    let manifest = info.manifest_value.as_ref()?;
    let mut providers = Vec::new();
    if payment_provider_enabled_for_platform(manifest, PaymentProvider::Alipay, "ios") {
        push_ios_payment_provider(&mut providers, IosPaymentProvider::Alipay);
    }
    if payment_provider_enabled_for_platform(manifest, PaymentProvider::Weixin, "ios") {
        push_ios_payment_provider(&mut providers, IosPaymentProvider::Weixin);
    }
    if payment_provider_enabled_for_platform(manifest, PaymentProvider::Paypal, "ios") {
        push_ios_payment_provider(&mut providers, IosPaymentProvider::Paypal);
    }
    if payment_provider_enabled_for_platform(manifest, PaymentProvider::Stripe, "ios") {
        push_ios_payment_provider(&mut providers, IosPaymentProvider::Stripe);
    }
    if payment_provider_enabled_for_platform(manifest, PaymentProvider::Iap, "ios")
        || payment_provider_enabled_for_platform(manifest, PaymentProvider::Apple, "ios")
    {
        push_ios_payment_provider(&mut providers, IosPaymentProvider::Iap);
    }

    if providers.is_empty() {
        return None;
    }
    Some(providers)
}

fn ios_payment_requires_ios_13(providers: &[IosPaymentProvider]) -> bool {
    providers.iter().any(|provider| {
        matches!(
            provider,
            IosPaymentProvider::Paypal | IosPaymentProvider::Stripe
        )
    })
}

pub(crate) fn ios_payment_provider_value(
    manifest: &serde_json::Value,
    provider: PaymentProvider,
) -> Option<&serde_json::Value> {
    manifest_payment_provider_value(manifest, provider, "ios")
}

fn push_ios_payment_provider(
    providers: &mut Vec<IosPaymentProvider>,
    provider: IosPaymentProvider,
) {
    if !providers.contains(&provider) {
        providers.push(provider);
    }
}

fn ios_payment_linked_files(
    providers: &[IosPaymentProvider],
    libs_dir: &Path,
) -> Vec<IosPbxLinkedFile> {
    let mut files = Vec::new();
    push_ios_linked_file(
        &mut files,
        IosPbxLinkedFile::local_static("liblibPayment.a"),
    );

    if providers.contains(&IosPaymentProvider::Alipay) {
        for file in [
            IosPbxLinkedFile::local_static("libalixpayment.a"),
            ios_payment_alipay_sdk_linked_file(libs_dir),
            IosPbxLinkedFile::system_framework("Security.framework"),
            IosPbxLinkedFile::system_framework("CoreMotion.framework"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
            IosPbxLinkedFile::system_framework("CFNetwork.framework"),
            IosPbxLinkedFile::system_library("libc++.tbd"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosPaymentProvider::Weixin) {
        for file in [
            IosPbxLinkedFile::local_static("libwxpay.a"),
            IosPbxLinkedFile::local_static("libWeChatSDK_pay.a"),
            IosPbxLinkedFile::system_library("libsqlite3.0.tbd"),
            IosPbxLinkedFile::system_library("libz.tbd"),
            IosPbxLinkedFile::system_framework("CoreTelephony.framework"),
            IosPbxLinkedFile::system_framework("SystemConfiguration.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosPaymentProvider::Iap) {
        for file in [
            IosPbxLinkedFile::local_static("libIAPPay.a"),
            IosPbxLinkedFile::system_framework("StoreKit.framework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosPaymentProvider::Paypal) {
        for file in [
            IosPbxLinkedFile::local_static("libpaypalpay.a"),
            IosPbxLinkedFile::local_xcframework("PayPalCheckout.xcframework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    if providers.contains(&IosPaymentProvider::Stripe) {
        for file in [
            IosPbxLinkedFile::local_static("libstripepay.a"),
            IosPbxLinkedFile::local_xcframework("StripeApplePay.xcframework"),
            IosPbxLinkedFile::local_xcframework("StripeCore.xcframework"),
            IosPbxLinkedFile::local_xcframework("StripeUICore.xcframework"),
            IosPbxLinkedFile::local_xcframework("Stripe3DS2.xcframework"),
            IosPbxLinkedFile::local_xcframework("StripePayments.xcframework"),
            IosPbxLinkedFile::local_xcframework("StripePaymentsUI.xcframework"),
            IosPbxLinkedFile::local_xcframework("StripePaymentSheet.xcframework"),
        ] {
            push_ios_linked_file(&mut files, file);
        }
    }

    files
}

fn ios_payment_alipay_sdk_linked_file(libs_dir: &Path) -> IosPbxLinkedFile {
    if libs_dir.join(ALIPAY_SDK_FRAMEWORK).exists() {
        IosPbxLinkedFile::local_framework(ALIPAY_SDK_FRAMEWORK)
    } else if libs_dir.join(ALIPAY_SDK_XCFRAMEWORK).exists() {
        IosPbxLinkedFile::local_xcframework(ALIPAY_SDK_XCFRAMEWORK)
    } else {
        IosPbxLinkedFile::local_framework(ALIPAY_SDK_FRAMEWORK)
    }
}

fn push_ios_linked_file(files: &mut Vec<IosPbxLinkedFile>, file: IosPbxLinkedFile) {
    if !files.iter().any(|existing| existing.name == file.name) {
        files.push(file);
    }
}

fn validate_ios_payment_local_linked_files(
    libs_dir: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    for file in files.iter().copied().filter(|file| file.is_local()) {
        if matches!(file.name, ALIPAY_SDK_FRAMEWORK | ALIPAY_SDK_XCFRAMEWORK) {
            validate_ios_payment_alipay_sdk(libs_dir)?;
            continue;
        }
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS 支付模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn validate_ios_payment_alipay_sdk(libs_dir: &Path) -> Result<(), String> {
    let framework = libs_dir.join(ALIPAY_SDK_FRAMEWORK);
    let xcframework = libs_dir.join(ALIPAY_SDK_XCFRAMEWORK);
    if framework.exists() || xcframework.exists() {
        return Ok(());
    }
    Err(format!(
        "iOS 支付模块缺少 SDK 依赖文件: {} 或 {}",
        framework.display(),
        xcframework.display()
    ))
}

fn ios_payment_resource_sources(
    project_root: &Path,
    providers: &[IosPaymentProvider],
) -> Result<Vec<(String, PathBuf)>, String> {
    let bundles_dir = ios_sdk_support_dir(project_root)?.join("Bundles");
    let mut resources = Vec::new();
    if providers.contains(&IosPaymentProvider::Alipay) {
        push_ios_payment_resource(&mut resources, &bundles_dir, "AlipaySDK.bundle")?;
    }
    Ok(resources)
}

fn push_ios_payment_resource(
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
            "iOS 支付模块缺少 SDK 资源文件: {} ({})",
            name,
            bundles_dir.display()
        ));
    }
    resources.push((name.to_string(), source));
    Ok(())
}

fn copy_ios_payment_resources(
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
                .map_err(|e| format!("清理 iOS 支付资源副本失败 {}: {}", target.display(), e))?;
        } else if target.exists() {
            std::fs::remove_file(&target)
                .map_err(|e| format!("清理 iOS 支付资源副本失败 {}: {}", target.display(), e))?;
        }
        if source.is_dir() {
            crate::utils::fs::copy_recursive(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 支付资源失败 {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        } else {
            std::fs::copy(source, &target).map_err(|e| {
                format!(
                    "复制 iOS 支付资源失败 {} -> {}: {}",
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
