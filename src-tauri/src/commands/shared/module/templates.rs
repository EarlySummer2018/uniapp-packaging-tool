use std::collections::HashMap;

use crate::commands::shared::module::types::{
    AndroidModuleTemplate, FaceRecognitionModuleConfig, IosModuleTemplate, LivePusherModuleConfig,
    LocationConfig, LoginModuleConfig, LoginProvider, MapModuleConfig, ModuleConfigTree,
    ModuleTemplate, PaymentModuleConfig, PushModuleConfig, ShareModuleConfig, SimpleModuleConfig,
    SpeechModuleConfig, StatisticModuleConfig, UniAdModuleConfig, UrlSchemeConfig,
};

pub fn android_module_template_key(module_name: &str) -> Option<&'static str> {
    match module_name {
        "Push" | "push" => Some("push"),
        "Share" | "share" => Some("share"),
        "Geolocation" | "Location" | "geolocation" | "location" => Some("geolocation"),
        "Payment" | "payment" | "Pay" | "pay" => Some("payment"),
        "Login" | "OAuth" | "Oauth" | "oauth" | "login" => Some("login"),
        "Map" | "Maps" | "map" | "maps" => Some("map"),
        "Statistic" | "Statistics" | "statistic" | "statistics" => Some("statistic"),
        "Speech" | "speech" => Some("speech"),
        "FaceRecognition"
        | "FaceRecognitionVerify"
        | "FacialRecognitionVerify"
        | "facialRecognitionVerify" => Some("face_recognition"),
        "UniAD" | "uni-ad" | "uniAD" | "ad" | "Ad" => Some("uni_ad"),
        "X5Webview" | "X5TBS" | "Android X5 Webview" | "x5" | "x5_tbs" => Some("x5_tbs"),
        "LivePusher" | "livepusher" => Some("livepusher"),
        "Camera" | "camera" => Some("camera"),
        _ => None,
    }
}

pub fn apply_module_name_to_tree(tree: &mut ModuleConfigTree, name: &str) {
    match name {
        "Push" => {
            tree.push = Some(push_manifest_config());
        }
        "Geolocation" => {
            tree.geolocation = Some(location_manifest_config());
        }
        "Share" => {
            tree.share = Some(share_manifest_config());
        }
        "Login" | "OAuth" => {
            tree.login = Some(login_manifest_config());
        }
        "Payment" => {
            tree.payment = Some(payment_manifest_config());
        }
        "Map" | "Maps" => {
            tree.map = Some(map_manifest_config());
        }
        "Speech" => {
            tree.speech = Some(speech_manifest_config());
        }
        "Statistic" | "Statistics" => {
            tree.statistic = Some(statistic_manifest_config());
        }
        "FaceRecognition" | "FaceRecognitionVerify" | "FacialRecognitionVerify" => {
            tree.face_recognition = Some(face_recognition_manifest_config());
        }
        "UniAD" | "uni-ad" => {
            tree.uni_ad = Some(uni_ad_manifest_config());
        }
        "X5Webview" | "X5TBS" | "Android X5 Webview" => {
            tree.x5_tbs = Some(SimpleModuleConfig { enabled: true });
        }
        "LivePusher" => {
            tree.livepusher = Some(livepusher_manifest_config());
        }
        "Camera" => {
            tree.camera = Some(SimpleModuleConfig { enabled: true });
        }
        "UIWebview" | "UIWebView" => {
            tree.ui_webview = Some(SimpleModuleConfig { enabled: true });
        }
        _ => {}
    }
}

pub fn merge_properties_to_tree(
    tree: &mut ModuleConfigTree,
    _xml_content: &str,
) -> Result<(), String> {
    if let Some(ref mut push) = tree.push {
        push.unipush_appid = Some(String::new());
        push.unipush_appkey = Some(String::new());
        push.unipush_appsecret = Some(String::new());
    }
    Ok(())
}

fn push_manifest_config() -> PushModuleConfig {
    PushModuleConfig {
        enabled: true,
        unipush_appid: Some(String::new()),
        unipush_appkey: Some(String::new()),
        unipush_appsecret: Some(String::new()),
        vendors: Vec::new(),
    }
}

fn location_manifest_config() -> LocationConfig {
    LocationConfig {
        enabled: true,
        engine: "system".to_string(),
        baidu_ak: None,
        amap_key: None,
    }
}

fn share_manifest_config() -> ShareModuleConfig {
    ShareModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        qq: Some(HashMap::new()),
        sina: None,
    }
}

fn login_manifest_config() -> LoginModuleConfig {
    LoginModuleConfig {
        enabled: true,
        providers: vec![
            LoginProvider {
                name: "weixin".to_string(),
                enabled: true,
                config: HashMap::new(),
            },
            LoginProvider {
                name: "qq".to_string(),
                enabled: true,
                config: HashMap::new(),
            },
        ],
    }
}

fn payment_manifest_config() -> PaymentModuleConfig {
    PaymentModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        alipay: Some(HashMap::new()),
    }
}

fn map_manifest_config() -> MapModuleConfig {
    MapModuleConfig {
        enabled: true,
        engine: "amap".into(),
        amap_key: None,
        tencent_map_key: None,
        baidu_map_ak: None,
        google_maps_api_key: None,
    }
}

fn speech_manifest_config() -> SpeechModuleConfig {
    SpeechModuleConfig {
        enabled: true,
        engine: "system".into(),
        xfyun: None,
        baidu: None,
        aliyun: None,
    }
}

fn statistic_manifest_config() -> StatisticModuleConfig {
    StatisticModuleConfig {
        enabled: true,
        provider: "umeng".into(),
        umeng: None,
        mta: None,
        baidu: None,
    }
}

fn face_recognition_manifest_config() -> FaceRecognitionModuleConfig {
    FaceRecognitionModuleConfig {
        enabled: true,
        provider: "dcloud".into(),
        dcloud: None,
        baidu: None,
        aliyun: None,
    }
}

fn uni_ad_manifest_config() -> UniAdModuleConfig {
    UniAdModuleConfig {
        enabled: true,
        csj: Some(HashMap::new()),
        gdt: Some(HashMap::new()),
        gromore: None,
        admob: None,
    }
}

fn livepusher_manifest_config() -> LivePusherModuleConfig {
    LivePusherModuleConfig {
        enabled: true,
        license_url: None,
        license_key: None,
    }
}

pub fn module_applies_to_android(platforms: &[String]) -> bool {
    platforms.is_empty()
        || platforms.iter().any(|platform| {
            let platform = platform.to_ascii_lowercase();
            platform == "all" || platform == "android" || platform == "app"
        })
}

// ---------------------------------------------------------------------------
// Template getters
// ---------------------------------------------------------------------------

pub fn get_module_template_sync(module_name: &str) -> Result<ModuleTemplate, String> {
    match module_name {
        "push" => Ok(get_push_template()),
        "share" => Ok(get_share_template()),
        "geolocation" => Ok(get_geolocation_template()),
        "payment" => Ok(get_payment_template()),
        "login" => Ok(get_login_template()),
        "map" => Ok(get_map_template()),
        "statistic" => Ok(get_statistic_template()),
        "speech" => Ok(get_speech_template()),
        "face_recognition" => Ok(get_face_recognition_template()),
        "uni_ad" => Ok(get_uniad_template()),
        "x5_tbs" => Ok(get_x5_template()),
        "livepusher" => Ok(get_livepusher_template()),
        "camera" => Ok(get_camera_template()),
        _ => Err(format!("Unknown module: {}", module_name)),
    }
}

fn get_push_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Push".to_string(),
        description: "uniPush 推送模块（支持6厂商通道）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "aps-release.aar".to_string(),
                "aps-unipush-release.aar".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.getui:gtsdk:3.3.7.0 (个推SDK)".to_string(),
                "com.getui:gtc-dcloud:3.2.16.7 (个推核心)".to_string(),
                "com.getui.opt:hwp:3.1.1 (华为)".to_string(),
                "com.huawei.hms:push:6.11.0.300 (华为)".to_string(),
                "com.getui.opt:xmp:3.3.1 (小米)".to_string(),
                "com.assist-v3:oppo:3.3.0 (OPPO)".to_string(),
                "com.google.code.gson:gson:2.6.2 (OPPO)".to_string(),
                "commons-codec:commons-codec:1.6 (OPPO)".to_string(),
                "androidx.annotation:annotation:1.1.0 (OPPO)".to_string(),
                "com.assist-v3:vivo:3.1.1 (vivo)".to_string(),
                "com.getui.opt:mzp:3.2.3 (魅族)".to_string(),
                "com.getui.opt:honor:3.6.0 (荣耀)".to_string(),
                "com.hihonor.mcs:push:7.0.61.303 (荣耀)".to_string(),
            ],
            manifest_placeholders: vec![
                "XIAOMI_APP_ID / XIAOMI_APP_KEY".to_string(),
                "MEIZU_APP_ID / MEIZU_APP_KEY".to_string(),
                "HUAWEI_APP_ID".to_string(),
                "OPPO_APP_KEY / OPPO_APP_SECRET".to_string(),
                "VIVO_APP_ID / VIVO_APP_KEY".to_string(),
                "HONOR_APP_ID".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([
                    ("android:name".to_string(), "MIPUSH_APPID".to_string()),
                    ("android:value".to_string(), "${XIAOMI_APP_ID}".to_string()),
                ]),
                HashMap::from([
                    ("android:name".to_string(), "MIPUSH_APPKEY".to_string()),
                    ("android:value".to_string(), "${XIAOMI_APP_KEY}".to_string()),
                ]),
                HashMap::from([
                    ("android:name".to_string(), "MEIZUPUSH_APPID".to_string()),
                    ("android:value".to_string(), "${MEIZU_APP_ID}".to_string()),
                ]),
                HashMap::from([
                    ("android:name".to_string(), "MEIZUPUSH_APPKEY".to_string()),
                    ("android:value".to_string(), "${MEIZU_APP_KEY}".to_string()),
                ]),
            ],
            activities: vec![],
            properties_xml: "<feature name=\"Push\" value=\"io.dcloud.feature.aps.APSFeatureImpl\"><module name=\"unipush\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "UserNotifications.framework".to_string(),
                "Security.framework".to_string(),
                "CoreTelephony.framework".to_string(),
                "SystemConfiguration.framework".to_string(),
            ],
            required_libraries: vec![
                "libc++.tbd".to_string(),
                "libsqlite3.tbd".to_string(),
                "libz.tbd".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("getui".to_string(), "{appid, appkey, appsecret} (个推/uniPush)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "<key>getui</key><dict><key>appid</key><string></string></dict>".to_string(),
        },
    }
}

fn get_share_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Share".to_string(),
        description: "社交分享模块（微信/QQ/新浪微博）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "share-weixin-release.aar (微信)".to_string(),
                "share-qq-release.aar (QQ)".to_string(),
                "share-sina-release.aar (微博)".to_string(),
                "open_sdk_XXX_lite.jar (QQ SDK)".to_string(),
                "openDefault-XXX.aar (微博 SDK)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信 HX>=3.7.6)".to_string(),
            ],
            manifest_placeholders: vec![
                "WX_APPID / WX_SECRET (微信)".to_string(),
                "QQ_APPID (QQ)".to_string(),
                "SINA_APPKEY / SINA_SECRET / SINA_REDIRECT_URI (微博)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "WX_APPID".to_string()), ("android:value".to_string(), "${WX_APPID}".to_string())]),
                HashMap::from([("android:name".to_string(), "WX_SECRET".to_string()), ("android:value".to_string(), "${WX_SECRET}".to_string())]),
                HashMap::from([("android:name".to_string(), "QQ_APPID".to_string()), ("android:value".to_string(), "${QQ_APPID}".to_string())]),
                HashMap::from([("android:name".to_string(), "SINA_APPKEY".to_string()), ("android:value".to_string(), "${SINA_APPKEY}".to_string())]),
                HashMap::from([("android:name".to_string(), "SINA_SECRET".to_string()), ("android:value".to_string(), "${SINA_SECRET}".to_string())]),
                HashMap::from([("android:name".to_string(), "SINA_REDIRECT_URI".to_string()), ("android:value".to_string(), "${SINA_REDIRECT_URI}".to_string())]),
            ],
            activities: vec![
                ".wxapi.WXEntryActivity (微信回调)".to_string(),
                ".wxapi.WXPayActivity (微信支付)".to_string(),
                "com.tencent.tauth.AuthActivity (QQ授权)".to_string(),
                "com.tencent.connect.common.AssistActivity (QQ辅助)".to_string(),
                "cn.sharesdk.wechat.friends.WXFriendActivity (微博分享页)".to_string(),
            ],
            properties_xml: "<feature name=\"Share\" value=\"io.dcloud.share.ShareFeatureImpl\"><module name=\"Weixin\"/><module name=\"QQ\"/><module name=\"SinaWeibo\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "ImageIO.framework (微博)".to_string(),
                "CoreTelephony.framework (微信)".to_string(),
                "SystemConfiguration.framework (微信)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.0.tbd (微博/QQ)".to_string(),
                "libz.tbd (微信/QQ)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("weixin".to_string(), "{appid, UniversalLinks} (微信)".to_string()),
                ("qq".to_string(), "{appid, Associated Domains} (QQ)".to_string()),
                ("sinaweibo".to_string(), "{appkey, redirectURI, Associated Domains} (微博)".to_string()),
            ]),
            url_schemes: vec![
                UrlSchemeConfig { scheme: "wx{appid}".to_string(), identifier: "weixin".to_string() },
                UrlSchemeConfig { scheme: "tencent{appid}".to_string(), identifier: "tencentopenapi".to_string() },
                UrlSchemeConfig { scheme: "wb{appkey}".to_string(), identifier: "com.weibo".to_string() },
            ],
            plist_entry: "⚠️ iOS 微信分享需在 AppDelegate.m 中添加 handleOpenURL 回调；注意 libWeChatSDK_pay.a 仅用于分享+支付+登录，不用支付功能不要加此版本否则 App Store 审核被拒".to_string(),
        },
    }
}

fn get_geolocation_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Geolocation".to_string(),
        description: "定位模块（百度地图/高德地图/系统定位）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "baidu-libs-release.aar (百度定位)".to_string(),
                "geolocation-baidu-release.aar (百度定位)".to_string(),
                "geolocation-amap-release.aar (高德定位)".to_string(),
                "uni-getLocation-tencent-uni1-release.aar (腾讯定位)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.amap.api:location:6.4.5 (高德定位)".to_string(),
                "com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8 (腾讯定位)".to_string(),
            ],
            manifest_placeholders: vec![
                "BAIDU_MAP_AK (百度地图)".to_string(),
                "AMAP_KEY (高德地图)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "com.baidu.lbsapi.API_KEY".to_string()), ("android:value".to_string(), "${BAIDU_MAP_AK}".to_string())]),
                HashMap::from([("android:name".to_string(), "amap_api_key".to_string()), ("android:value".to_string(), "${AMAP_KEY}".to_string())]),
            ],
            activities: vec![],
            properties_xml: "<feature name=\"Geolocation\"><module name=\"BaiduMap\"/>(或 Amap)</feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "CoreLocation.framework".to_string(),
                "Security.framework (百度)".to_string(),
            ],
            required_libraries: vec![
                "libcrypto.a (百度)".to_string(),
            ],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![],
            plist_entry: "NSLocationWhenInUseUsageDescription / NSLocationAlwaysAndWhenInUseUsageDescription 必须配置".to_string(),
        },
    }
}

fn get_payment_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Payment".to_string(),
        description: "支付模块（微信支付/支付宝）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "payment-alipay-release.aar (支付宝)".to_string(),
                "payment-weixin-release.aar (微信支付)".to_string(),
                "payment-paypal-release.aar (PayPal)".to_string(),
                "payment-stripe-release.aar (Stripe)".to_string(),
                "payment-google-release.aar (Google Pay)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.alipay.sdk:alipaysdk-android:15.8.11 (支付宝)".to_string(),
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信支付)".to_string(),
                "com.paypal.checkout:android-sdk:0.6.2 (PayPal)".to_string(),
                "com.stripe:stripe-android:18.2.0 (Stripe)".to_string(),
                "com.google.android.gms:play-services-wallet:18.1.3 (Google Pay)".to_string(),
            ],
            manifest_placeholders: vec![
                "WX_APPID (微信支付复用分享的 WX_APPID)".to_string(),
            ],
            manifest_meta_data: vec![],
            activities: vec![
                ".wxapi.WXPayEntryActivity (微信支付回调)".to_string(),
            ],
            properties_xml: "<feature name=\"Payment\"><module name=\"WeixinPay\"/><module name=\"Alipay\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "CoreTelephony.framework (微信)".to_string(),
                "SystemConfiguration.framework (微信)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.0.tbd (微信)".to_string(),
                "libz.tbd (微信)".to_string(),
            ],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![
                UrlSchemeConfig { scheme: "wx{appid}".to_string(), identifier: "weixin".to_string() },
            ],
            plist_entry: "iOS 微信支付依赖 libWeChatSDK_pay.a（含分享+支付+登录）或 libWeChatSDK.a（仅分享+登录）".to_string(),
        },
    }
}

fn get_login_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Login".to_string(),
        description: "登录模块（微信/QQ/苹果/一键登录）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "oauth-univerify-release.aar (一键登录)".to_string(),
                "oauth-weixin-release.aar (微信登录)".to_string(),
                "oauth-qq-release.aar (QQ登录)".to_string(),
                "open_sdk_XXX_lite.jar (QQ SDK)".to_string(),
                "openDefault-XXX.aar (微博 SDK)".to_string(),
                "oauth-sina-release.aar (微博登录)".to_string(),
                "oauth-miui-release.aar (小米登录)".to_string(),
                "oauth-google-release.aar (Google登录)".to_string(),
                "oauth-facebook-release.aar (Facebook登录)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信登录)".to_string(),
                "com.getui:gtc-dcloud:3.2.16.7 (一键登录)".to_string(),
                "com.getui:gysdk:3.1.7.0 (一键登录)".to_string(),
                "com.google.android.gms:play-services-auth:19.2.0 (Google登录)".to_string(),
                "com.facebook.android:facebook-login:17.0.2 (Facebook登录)".to_string(),
            ],
            manifest_placeholders: vec![
                "WX_APPID (微信登录)".to_string(),
                "QQ_APPID (QQ登录)".to_string(),
            ],
            manifest_meta_data: vec![],
            activities: vec![
                ".wxapi.WXEntryActivity (微信登录回调)".to_string(),
                "com.tencent.tauth.AuthActivity (QQ登录)".to_string(),
            ],
            properties_xml: "<feature name=\"Login\"><module name=\"WeixinLogin\"/><module name=\"QQLogin\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "AuthenticationServices.framework (Apple 登录)".to_string(),
                "CoreTelephony.framework (微信)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.0.tbd (微信)".to_string(),
                "libz.tbd (微信)".to_string(),
            ],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![
                UrlSchemeConfig { scheme: "wx{appid}".to_string(), identifier: "weixin".to_string() },
            ],
            plist_entry: "Apple Sign-In 需要在 Xcode Signing & Capabilities 添加 Sign in with Apple".to_string(),
        },
    }
}

fn get_map_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Map".to_string(),
        description: "地图模块（高德/腾讯/Google/Apple Maps）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "baidu-libs-release.aar (百度地图)".to_string(),
                "map-baidu-release.aar (百度地图)".to_string(),
                "weex_amap-release.aar (高德 nvue 页面)".to_string(),
                "map-amap-release.aar (高德 vue 页面)".to_string(),
                "weex_google-map-release.aar (Google nvue 页面)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.amap.api:3dmap:latest.release (高德地图，版本以 SDK demo 为准)".to_string(),
                "com.amap.api:search:latest.release (高德搜索，版本以 SDK demo 为准)".to_string(),
                "com.google.android.gms:play-services-maps:18.0.1 (Google地图)".to_string(),
            ],
            manifest_placeholders: vec![
                "AMAP_KEY (高德地图 Key)".to_string(),
                "TENCENT_MAP_KEY (腾讯地图 Key)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "com.amap.api.v2.apikey".to_string()), ("android:value".to_string(), "${AMAP_KEY}".to_string())]),
            ],
            activities: vec![
                "com.amap.api.maps2d.MapActivity (高德地图容器)".to_string(),
            ],
            properties_xml: "<feature name=\"Maps\"><module name=\"Amap\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "CoreLocation.framework".to_string(),
                "AMapFoundationKit.framework (高德基础)".to_string(),
                "MAMapKit.framework (高德地图)".to_string(),
                "QMapKit.framework (腾讯地图)".to_string(),
            ],
            required_libraries: vec![
                "libz.tbd (高德/腾讯)".to_string(),
                "libc++.tbd (高德)".to_string(),
                "libsqlite3.tbd (高德)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("NSLocationWhenInUseUsageDescription".to_string(), "需要您的位置信息来显示附近地点".to_string()),
                ("NSLocationAlwaysAndWhenInUseUsageDescription".to_string(), "需要持续获取位置以提供导航服务".to_string()),
                ("amap_key".to_string(), "(高德地图Key)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 地图模块需在 Info.plist 配置 NSLocation 相关权限描述；高德需在 Podfile 添加 pod 'AMap3DMap'".to_string(),
        },
    }
}

fn get_statistic_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Statistic".to_string(),
        description: "统计分析模块（友盟/腾讯MTA/百度统计/DCloud统计）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "statistic-release.aar".to_string(),
                "statistic-umeng-release.aar (友盟统计)".to_string(),
                "statistic-umeng-gp-release.aar (友盟 Google Play)".to_string(),
                "statistic-google-release.aar (谷歌统计)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.umeng.umsdk:common:9.6.1 (友盟基础库)".to_string(),
                "com.umeng.umsdk:asms:1.8.0 (友盟)".to_string(),
                "com.umeng.umsdk:abtest:1.0.1 (友盟)".to_string(),
                "com.umeng.umsdk:apm:1.9.1 (友盟)".to_string(),
                "com.google.firebase:firebase-analytics:21.3.0 (谷歌统计)".to_string(),
            ],
            manifest_placeholders: vec![
                "UMENG_APPKEY (友盟 AppKey)".to_string(),
                "UMENG_CHANNEL (渠道号, 可选)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "UMENG_APPKEY".to_string()), ("android:value".to_string(), "${UMENG_APPKEY}".to_string())]),
                HashMap::from([("android:name".to_string(), "UMENG_CHANNEL".to_string()), ("android:value".to_string(), "${UMENG_CHANNEL}".to_string())]),
            ],
            activities: vec![],
            properties_xml: "<feature name=\"Statistic\"><module name=\"Umeng\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "UMCommon.framework (友盟)".to_string(),
                "UMAnalytics.framework (友盟统计)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.tbd (友盟)".to_string(),
                "libz.tbd (友盟)".to_string(),
                "libresolv.tbd (友盟)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("UMENG_APPKEY".to_string(), "(友盟AppKey)".to_string()),
                ("UMENG_CHANNEL".to_string(), "(App Store)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 统计模块：友盟需在 Podfile 添加 pod 'UMCCommon' + pod 'UMCSecurityPlugins'; 腾讯MTA 需添加 pod 'MTA'".to_string(),
        },
    }
}

fn get_speech_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Speech".to_string(),
        description: "语音识别模块（讯飞/百度/阿里 + iOS系统语音）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "speech-release.aar".to_string(),
                "speech_baidu-release.aar (百度语音)".to_string(),
                "speech_ifly-release.aar (讯飞语音)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![],
            manifest_placeholders: vec![
                "IFLY_APPID (讯飞 AppID)".to_string(),
                "BD_SPEECH_APIKEY (百度 API Key)".to_string(),
                "BD_SPEECH_SECRETKEY (百度 Secret Key)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "IFLYTEK_APPKEY".to_string()), ("android:value".to_string(), "${IFLY_APPID}".to_string())]),
            ],
            activities: vec![],
            properties_xml: "<feature name=\"Speech\"><module name=\"Xfyun\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "Speech.framework (系统语音识别)".to_string(),
            ],
            required_libraries: vec![
                "libc++.tbd".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("NSSpeechRecognitionUsageDescription".to_string(), "需要使用语音识别功能来输入文字".to_string()),
                ("NMicrophoneUsageDescription".to_string(), "需要麦克风权限来进行语音输入".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 使用系统 SFSpeechRecognizer 进行语音识别；需在 Xcode 设置 Speech Recognition 能力".to_string(),
        },
    }
}

fn get_face_recognition_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "FaceRecognition".to_string(),
        description: "实人认证模块（DCloud/百度/阿里云）— 仅 Android".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "uni-facialRecognitionVerify-release.aar".to_string(),
                "aliyun-base-XXX.aar".to_string(),
                "aliyun-facade-XXX.aar".to_string(),
                "aliyun-face-XXX.aar".to_string(),
                "aliyun-faceaudio-XXX.aar".to_string(),
                "aliyun-facelanguage-XXX.aar".to_string(),
                "aliyun-photoinus-XXX.aar".to_string(),
                "aliyun-wishverify-XXX.aar".to_string(),
                "Android-XXX.jiagu.aar".to_string(),
                "10042.aar".to_string(),
                "APSecuritySDK-DeepSec.aar".to_string(),
                "facialRecognitionVerify-support-release.aar".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.squareup.okhttp3:okhttp:3.11.0".to_string(),
                "com.squareup.okio:okio:1.14.0".to_string(),
                "Com.aliyun.dpa:oss-android-sdk:+".to_string(),
            ],
            manifest_placeholders: vec![
                "DCLOUD_LICENSE (DCloud 许可证)".to_string(),
                "BDFACE_APIKEY (百度 API Key)".to_string(),
                "BDFACE_SECRETKEY (百度 Secret Key)".to_string(),
                "ALIFACE_ACCESSKEY_ID (阿里 AccessKeyId)".to_string(),
                "ALIFACE_ACCESSKEY_SECRET (阿里 AccessKeySecret)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "DCLOUD_LICENSE".to_string()), ("android:value".to_string(), "${DCLOUD_LICENSE}".to_string())]),
            ],
            activities: vec![
                "com.baidu.idl.face.ui.FaceLivenessActivity (百度活体检测)".to_string(),
            ],
            properties_xml: "<feature name=\"FaceRecognition\"><module name=\"DCloud\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![],
            required_libraries: vec![],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![],
            plist_entry: "⚠️ 实人认证模块当前仅支持 Android 平台。iOS 端如需人脸识别请使用原生 Face ID / Vision Framework。".to_string(),
        },
    }
}

fn get_uniad_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "UniAD".to_string(),
        description: "uni-AD 广告模块（穿山甲/优量汇/Gromore/AdMob）".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "ads-release.aar".to_string(),
                "ads-csj-release.aar (穿山甲)".to_string(),
                "open_ad_sdk.aar (穿山甲/GroMore)".to_string(),
                "ads-gdt-release.aar (腾讯优量汇)".to_string(),
                "GDTSDK.unionNormal.aar (优量汇/GDT)".to_string(),
                "ads-ks-release.aar (快手广告联盟)".to_string(),
                "ks_adsdk-ad.aar (快手广告联盟)".to_string(),
                "ads-ks-content-release.aar (快手内容联盟)".to_string(),
                "kssdk-allad-content.aar (快手内容联盟)".to_string(),
                "ads-sigmob-release.aar (Sigmob)".to_string(),
                "windAd.aar (Sigmob)".to_string(),
                "wind-common.aar (Sigmob)".to_string(),
                "ads-bd-release.aar (百度广告)".to_string(),
                "Baidu_MobAds_SDK.aar (百度广告)".to_string(),
                "ads-hw-release.aar (华为广告)".to_string(),
                "ads-gromore-release.aar (GroMore)".to_string(),
                "ads-wm-release.aar (uniMP激励视频)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![
                "com.huawei.hms:ads-lite:13.4.56.302 (华为广告)".to_string(),
                "com.huawei.hms:ads-omsdk:1.3.35 (华为广告)".to_string(),
            ],
            manifest_placeholders: vec![
                "CSJ_APP_ID (穿山甲 AppID)".to_string(),
                "GDT_APPID (优量汇 AppID)".to_string(),
                "ADMOB_APP_ID (AdMob AppID)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "CSJ_APP_ID".to_string()), ("android:value".to_string(), "${CSJ_APP_ID}".to_string())]),
                HashMap::from([("android:name".to_string(), "GDT_APPID".to_string()), ("android:value".to_string(), "${GDT_APPID}".to_string())]),
                HashMap::from([("android:name".to_string(), "com.google.android.gms.ads.APPLICATION_ID".to_string()), ("android:value".to_string(), "${ADMOB_APP_ID}".to_string())]),
            ],
            activities: vec![
                "com.bytedance.sdk.openad.sdk.activity.TTFullScreenVideoActivity (穿山甲全屏视频)".to_string(),
                "com.qq.e.ads.ADActivity (优量汇广告页)".to_string(),
            ],
            properties_xml: "<feature name=\"UniAD\"><module name=\"CSJ\"/><module name=\"GDT\"/></feature>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "BUAdSDK.framework (穿山甲/iOS)".to_string(),
                "GDTMobSDK.framework (优量汇/iOS)".to_string(),
                "GoogleMobileAdsFramework.framework (AdMob)".to_string(),
            ],
            required_libraries: vec![
                "libsqlite3.tbd (穿山甲)".to_string(),
                "libz.tbd (穿山甲/优量汇)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("CSJ_AppID".to_string(), "(穿山甲 AppID)".to_string()),
                ("GDT_AppKey".to_string(), "(优量汇 AppKey)".to_string()),
                ("GADApplicationIdentifier".to_string(), "(AdMob AppID)".to_string()),
                ("SKAdNetworkItems".to_string(), "需配置 SKAdNetworkIdentifier 列表".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 广告模块：穿山甲需在 Podfile 添加 pod 'Bytedance-UnionADS'；优量汇添加 pod 'GDTMobSDK'；AdMob 添加 pod 'Google-Mobile-Ads-SDK'".to_string(),
        },
    }
}

fn get_x5_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "X5TBS".to_string(),
        description: "腾讯 X5 TBS 内核 WebView — 仅 Android".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "webview-x5-release.aar".to_string(),
                "weex_webview-x5-release.aar (uni-app项目)".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![],
            manifest_placeholders: vec![],
            manifest_meta_data: vec![],
            activities: vec![
                "com.tencent.smtt.sdk.VideoActivity (TBS 视频播放器)".to_string(),
                "com.tencent.smtt.sdk.TbsDownloaderActivity (TBS 下载器)".to_string(),
            ],
            properties_xml: "<feature name=\"X5Webview\" value=\"io.dcloud.feature.X5Webview.X5WebViewService\"/>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![],
            required_libraries: vec![],
            info_plist_keys: HashMap::new(),
            url_schemes: vec![],
            plist_entry: "⚠️ X5 TBS WebView 仅支持 Android 平台。iOS 端默认使用 WKWebView，无需额外配置。".to_string(),
        },
    }
}

fn get_livepusher_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "LivePusher".to_string(),
        description: "直播推流模块 — 主要支持 iOS".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![
                "weex_livepusher-release.aar".to_string(),
            ],
            vendor_aars: vec![],
            gradle_dependencies: vec![],
            manifest_placeholders: vec![
                "LIVEPUSH_LICENSE_URL (直播 License URL)".to_string(),
                "LIVEPUSH_LICENSE_KEY (直播 License Key)".to_string(),
            ],
            manifest_meta_data: vec![
                HashMap::from([("android:name".to_string(), "TXLIVE_LICENSE_URL".to_string()), ("android:value".to_string(), "${LIVEPUSH_LICENSE_URL}".to_string())]),
            ],
            activities: vec![
                "com.tencent.liteav.activity.TCActivity (腾讯直播容器)".to_string(),
            ],
            properties_xml: "<feature name=\"LivePusher\"/>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![
                "TLBB.framework (腾讯直播推流)".to_string(),
                "TXLiteAVSDK_Professional.framework (腾讯云音视频)".to_string(),
                "RPLivePlayerLib.framework (七牛推流, 可选)".to_string(),
                "LFLiveKit.framework (LFLiveKit, 可选)".to_string(),
            ],
            required_libraries: vec![
                "libc++.tbd (腾讯直播)".to_string(),
                "libresolv.tbd (腾讯直播)".to_string(),
                "libsqlite3.tbd (腾讯直播)".to_string(),
                "libz.tbd (腾讯直播)".to_string(),
            ],
            info_plist_keys: HashMap::from([
                ("NSCameraUsageDescription".to_string(), "需要摄像头权限进行直播推流".to_string()),
                ("NSMicrophoneUsageDescription".to_string(), "需要麦克风权限进行直播推流".to_string()),
                ("TXLIVE_LICENSE_URL".to_string(), "(腾讯云直播License URL)".to_string()),
                ("TXLIVE_LICENSE_KEY".to_string(), "(腾讯云直播License Key)".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 直播推流模块：推荐使用腾讯云 LiteAVSDK (TXLiteAVSDK)；Podfile 添加 pod 'TXLiteAVSDK_Professional'；需配置相机+麦克风权限".to_string(),
        },
    }
}

fn get_camera_template() -> ModuleTemplate {
    ModuleTemplate {
        module_name: "Camera".to_string(),
        description: "相机/相册模块 — SDK 内置模块，无需额外 AAR".to_string(),
        android_config: AndroidModuleTemplate {
            required_aars: vec![],
            vendor_aars: vec![],
            gradle_dependencies: vec![],
            manifest_placeholders: vec![],
            manifest_meta_data: vec![],
            activities: vec![],
            properties_xml: "<feature name=\"Camera\" value=\"io.dcloud.js.camera.CameraFeatureImpl\"/>".to_string(),
        },
        ios_config: IosModuleTemplate {
            required_frameworks: vec![],
            required_libraries: vec![],
            info_plist_keys: HashMap::from([
                ("NSCameraUsageDescription".to_string(), "需要使用相机功能".to_string()),
                ("NSPhotoLibraryUsageDescription".to_string(), "需要访问相册".to_string()),
            ]),
            url_schemes: vec![],
            plist_entry: "iOS 相机/相册模块：需配置 NSCameraUsageDescription 和 NSPhotoLibraryUsageDescription 权限描述".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_template_does_not_declare_third_party_activities() {
        let template = get_push_template();

        assert!(template.android_config.activities.is_empty());
    }
}
