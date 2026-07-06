use crate::commands::ios::modules::common::{
    ios_manifest_info_has_detected_module, ios_manifest_module_enabled,
    ios_object_value_normalized, ios_sdk_config_value_enabled,
};
use crate::commands::ios::modules::facial_recognition_verify::ios_facial_recognition_verify_enabled;
use crate::commands::ios::modules::geolocation::{
    ios_geolocation_providers, IosGeolocationProvider,
};
use crate::commands::ios::modules::livepusher::ios_livepusher_enabled;
use crate::commands::ios::modules::map::{ios_map_provider, IosMapProvider};
use crate::commands::ios::modules::oauth::{ios_oauth_providers, IosOauthProvider};
use crate::commands::ios::modules::payment::{ios_payment_providers, IosPaymentProvider};
use crate::commands::ios::modules::push::ios_push_enabled;
use crate::commands::ios::modules::share::{ios_share_providers, IosShareProvider};
use crate::commands::ios::modules::speech::{ios_speech_providers, IosSpeechProvider};
use crate::commands::ios::modules::statistic::{ios_statistic_providers, IosStatisticProvider};
use crate::commands::ios::modules::ui_webview::ios_ui_webview_enabled;
use crate::commands::shared::module::templates::{
    android_module_template_key, module_applies_to_ios,
};
use crate::commands::shared::resource_scan::ResourceScanResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IosPodSubspecs {
    pub(super) values: Vec<String>,
    pub(super) warnings: Vec<String>,
}

pub(super) fn resolve_ios_pod_subspecs(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    scan: &ResourceScanResult,
) -> IosPodSubspecs {
    let mut values = vec!["Core".to_string()];
    let mut warnings = Vec::new();

    add_base_subspecs(&mut values, manifest_info);
    add_provider_subspecs(&mut values, manifest_info);
    add_uts_subspecs(&mut values, manifest_info, scan);
    add_uni_ad_subspecs(&mut values, manifest_info, &mut warnings);

    IosPodSubspecs { values, warnings }
}

fn add_base_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    let Some(info) = manifest_info else {
        return;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return;
    };
    for module in &info.detected_modules {
        if !module_applies_to_ios(&module.platforms) {
            continue;
        }
        let Some(template_key) = android_module_template_key(&module.name) else {
            continue;
        };
        let Some(subspec) = base_subspec_for_template(template_key) else {
            continue;
        };
        if ios_manifest_module_enabled(manifest, &module.name) {
            push_subspec(specs, subspec);
        }
    }
}

fn base_subspec_for_template(template_key: &str) -> Option<&'static str> {
    match template_key {
        "barcode" => Some("Barcode"),
        "bluetooth" => Some("BlueTooth"),
        "camera" => Some("CameraGallery"),
        "contacts" => Some("Contacts"),
        "face_id" => Some("FaceId"),
        "fingerprint" => Some("Fingerprint"),
        "gcanvas" => Some("Canvas"),
        "ibeacon" => Some("IBeacon"),
        "messaging" => Some("Messaging"),
        "record" => Some("Audio"),
        "sqlite" => Some("Sqlite"),
        "video_player" => Some("Video"),
        _ => None,
    }
}

fn add_provider_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    add_geolocation_subspecs(specs, manifest_info);
    add_map_subspecs(specs, manifest_info);
    add_oauth_subspecs(specs, manifest_info);
    add_payment_subspecs(specs, manifest_info);
    add_push_subspecs(specs, manifest_info);
    add_share_subspecs(specs, manifest_info);
    add_speech_subspecs(specs, manifest_info);
    add_statistic_subspecs(specs, manifest_info);
    if ios_livepusher_enabled(manifest_info) {
        push_subspec(specs, "LivePusher");
    }
    if ios_ui_webview_enabled(manifest_info) {
        push_subspec(specs, "UIWebview");
    }
    if ios_facial_recognition_verify_enabled(manifest_info) {
        push_subspec(specs, "UTS");
        push_subspec(specs, "FacialRecognitionVerify");
    }
}

fn add_geolocation_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    let Some(providers) = ios_geolocation_providers(manifest_info) else {
        return;
    };
    for provider in providers {
        match provider {
            IosGeolocationProvider::System => push_subspec(specs, "Geolocation"),
            IosGeolocationProvider::Amap => push_subspec(specs, "Geolocation-Gaode"),
            IosGeolocationProvider::Baidu => push_subspec(specs, "Geolocation-Baidu"),
        }
    }
}

fn add_map_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    let Some(info) = manifest_info else {
        return;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return;
    };
    if !ios_manifest_info_has_detected_module(info, "Map")
        || !ios_manifest_module_enabled(manifest, "Map")
    {
        return;
    };
    match ios_map_provider(manifest) {
        Some(IosMapProvider::Baidu) => push_subspec(specs, "Map-Baidu"),
        Some(IosMapProvider::Amap) => push_subspec(specs, "Map-Gaode"),
        Some(IosMapProvider::Google) => push_subspec(specs, "Map-Google"),
        None => {}
    }
}

fn add_oauth_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    let Some(providers) = ios_oauth_providers(manifest_info) else {
        return;
    };
    let wechat_pay_enabled = ios_payment_providers(manifest_info)
        .is_some_and(|items| items.contains(&IosPaymentProvider::Weixin));
    for provider in providers {
        match provider {
            IosOauthProvider::Univerify => push_subspec(specs, "Oauth-Univerify"),
            IosOauthProvider::Sina => push_subspec(specs, "Oauth-Sina"),
            IosOauthProvider::Qq => push_subspec(specs, "Oauth-QQ"),
            IosOauthProvider::Weixin if wechat_pay_enabled => {
                push_subspec(specs, "Oauth-Wechat-PaySDK")
            }
            IosOauthProvider::Weixin => push_subspec(specs, "Oauth-Wechat"),
            IosOauthProvider::Apple => push_subspec(specs, "Oauth-Apple"),
            IosOauthProvider::Google => push_subspec(specs, "Oauth-Google"),
            IosOauthProvider::Facebook => push_subspec(specs, "Oauth-Facebook"),
        }
    }
}

fn add_payment_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    if !manifest_info.is_some_and(|info| ios_manifest_info_has_detected_module(info, "Payment")) {
        return;
    }
    let Some(providers) = ios_payment_providers(manifest_info) else {
        return;
    };
    for provider in providers {
        match provider {
            IosPaymentProvider::Alipay => push_subspec(specs, "Payment-AliPay"),
            IosPaymentProvider::Weixin => push_subspec(specs, "Payment-Wechat"),
            IosPaymentProvider::Paypal => push_subspec(specs, "Payment-Paypal"),
            IosPaymentProvider::Stripe => push_subspec(specs, "Payment-Stripe"),
            IosPaymentProvider::Iap => push_subspec(specs, "Payment-IAP"),
        }
    }
}

fn add_push_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    if ios_push_enabled(manifest_info) {
        push_subspec(specs, "Push-UniPush");
    }
}

fn add_share_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    let Some(providers) = ios_share_providers(manifest_info) else {
        return;
    };
    let wechat_pay_enabled = ios_payment_providers(manifest_info)
        .is_some_and(|items| items.contains(&IosPaymentProvider::Weixin));
    for provider in providers {
        match provider {
            IosShareProvider::Weixin if wechat_pay_enabled => {
                push_subspec(specs, "Share-Wechat-PaySDK")
            }
            IosShareProvider::Weixin => push_subspec(specs, "Share-Wechat"),
            IosShareProvider::Qq => push_subspec(specs, "Share-QQ"),
            IosShareProvider::Sina => push_subspec(specs, "Share-Sina"),
        }
    }
}

fn add_speech_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    let Some(providers) = ios_speech_providers(manifest_info) else {
        return;
    };
    for provider in providers {
        match provider {
            IosSpeechProvider::Baidu => push_subspec(specs, "Speech-Baidu"),
            IosSpeechProvider::Ifly => push_subspec(specs, "Speech-Ifly"),
        }
    }
}

fn add_statistic_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) {
    let Some(providers) = ios_statistic_providers(manifest_info) else {
        return;
    };
    for provider in providers {
        match provider {
            IosStatisticProvider::Umeng => push_subspec(specs, "Statistic-Umeng"),
            IosStatisticProvider::Firebase => push_subspec(specs, "Statistic-Firebase"),
        }
    }
}

fn add_uts_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    scan: &ResourceScanResult,
) {
    let builtin_ios_required = scan
        .uts
        .builtin_modules
        .iter()
        .any(|module| module.ios_dir.is_some());
    if scan.uts.has_ios_uts_plugins
        || builtin_ios_required
        || ios_facial_recognition_verify_enabled(manifest_info)
    {
        push_subspec(specs, "UTS");
    }
}

fn add_uni_ad_subspecs(
    specs: &mut Vec<String>,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
    warnings: &mut Vec<String>,
) {
    let Some(info) = manifest_info else {
        return;
    };
    let Some(manifest) = info.manifest_value.as_ref() else {
        return;
    };
    if !ios_manifest_info_has_detected_module(info, "UniAD") {
        return;
    }
    let Some(config) = uni_ad_sdk_config(manifest) else {
        return;
    };
    if !ios_sdk_config_value_enabled(config, Some("ios")) {
        return;
    }

    let before = specs.len();
    for (aliases, subspec) in UNI_AD_PROVIDER_SUBSPECS {
        if provider_enabled(config, aliases) {
            push_subspec(specs, subspec);
        }
    }
    if specs.len() == before {
        warnings.push(
            "检测到 uni-AD 模块，但未识别到具体 iOS 广告平台；请按官方文档确认 UniAd-* subspec"
                .into(),
        );
    }
}

const UNI_AD_PROVIDER_SUBSPECS: &[(&[&str], &str)] = &[
    (&["csj", "chuanshanjia"], "UniAd-CSJ"),
    (&["gromore", "groMore"], "UniAd-Gromore"),
    (&["gdt", "youlianghui"], "UniAd-GDT"),
    (&["ks", "kuaishou"], "UniAd-KS"),
    (&["sigmob"], "UniAd-Sigmob"),
    (&["baidu", "bd"], "UniAd-Baidu"),
    (&["wm", "weixin", "wechat"], "UniAd-WM"),
    (&["wa", "wangmai"], "UniAd-WA"),
    (&["applovin", "appLovin"], "UniAd-AppLovin"),
    (&["gg", "google", "admob"], "UniAd-GG"),
    (&["ggpangle", "admobpangle"], "UniAd-GG-Pangle"),
    (&["gmcontent", "gromorecontent"], "UniAd-GM-Content"),
    (&["inmobi"], "UniAd-InMobi"),
    (&["ironsource", "ironSource"], "UniAd-IronSource"),
    (&["kscontent", "kuaishoucontent"], "UniAd-KS-Content"),
    (&["liftoff", "vungle"], "UniAd-Liftoff"),
    (&["meta", "facebook"], "UniAd-Meta"),
    (&["mintegral"], "UniAd-Mintegral"),
    (&["pangle"], "UniAd-Pangle"),
    (&["unity", "unityads"], "UniAd-Unity"),
    (&["oct"], "UniAd-Oct"),
    (&["fl", "fanlian"], "UniAd-FL"),
    (&["yt"], "UniAd-YT"),
];

fn uni_ad_sdk_config(manifest: &serde_json::Value) -> Option<&serde_json::Value> {
    let sdk_configs = manifest
        .get("app-plus")?
        .get("distribute")?
        .get("sdkConfigs")?
        .as_object()?;
    ["ad", "ads", "uni-ad", "uniAD", "uniad"]
        .iter()
        .find_map(|key| ios_object_value_normalized(sdk_configs, key))
}

fn provider_enabled(config: &serde_json::Value, aliases: &[&str]) -> bool {
    let Some(map) = config.as_object() else {
        return false;
    };
    aliases.iter().any(|alias| {
        ios_object_value_normalized(map, alias)
            .is_some_and(|value| ios_sdk_config_value_enabled(value, Some("ios")))
    })
}

fn push_subspec(specs: &mut Vec<String>, value: &str) {
    if !specs.iter().any(|existing| existing == value) {
        specs.push(value.to_string());
    }
}
