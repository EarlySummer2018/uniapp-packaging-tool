use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::analysis::{
    analyze_android_module_config_sync, android_module_artifact_enabled_for_manifest,
    android_module_config_report_from_value, android_module_gradle_dependency_enabled_for_manifest,
    android_module_gradle_repositories_for_manifest,
};
use crate::commands::shared::module::parsing::module_config_from_detected_modules;
use crate::commands::shared::module::properties::generate_dcloud_properties;
use crate::commands::shared::module::types::{
    LoginModuleConfig, LoginProvider, MapModuleConfig, ModuleConfigTree, PaymentModuleConfig,
    PushModuleConfig, ShareModuleConfig, SimpleModuleConfig, StatisticModuleConfig,
};
use crate::commands::shared::resource::parse_uniapp_manifest;

#[test]
fn manifest_modules_generate_dcloud_properties() {
    let modules = vec![
        DetectedModule {
            name: "OAuth".to_string(),
            category: "login".to_string(),
            platforms: vec!["all".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "Payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["all".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "Share".to_string(),
            category: "share".to_string(),
            platforms: vec!["all".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
    ];
    let config = module_config_from_detected_modules(&modules);
    // 测试用：传入空白名单（不限制），验证所有模块都能正确生成
    let enabled: Vec<String> = vec![];
    let path = temp_file("unipack-dcloud-properties");

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content
        .contains(r#"<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl""#));
    assert!(content.contains(
        r#"<module name="OAuth-Weixin" value="io.dcloud.feature.oauth.weixin.WeiXinOAuthService"/>"#
    ));
    assert!(content.contains(
        r#"<feature name="Payment" value="io.dcloud.feature.payment.PaymentFeatureImpl">"#
    ));
    assert!(content
        .contains(r#"<module name="AliPay" value="io.dcloud.feature.payment.alipay.AliPay"/>"#));
    assert!(content.contains(r#"<feature name="Share""#));
    assert_eq!(content.matches("<features>").count(), 1);
    assert_eq!(content.matches("</features>").count(), 1);

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_payment_replaces_defaults_and_includes_all_providers() {
    let path = temp_file("unipack-all-payment-properties");
    std::fs::write(
        &path,
        r#"<properties>
    <features>
        <feature name="Payment"><module name="AliPay"/></feature>
    </features>
</properties>"#,
    )
    .unwrap();
    let config = ModuleConfigTree {
        payment: Some(PaymentModuleConfig {
            enabled: true,
            weixin: Some(HashMap::new()),
            alipay: Some(HashMap::new()),
            paypal: Some(HashMap::new()),
            stripe: Some(HashMap::new()),
            google: Some(HashMap::new()),
        }),
        ..Default::default()
    };

    generate_dcloud_properties(&path, &config, &["Payment".to_string()]).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    for provider in [
        "AliPay",
        "Payment-Weixin",
        "Payment-Paypal",
        "Payment-Stripe",
        "Payment-Google",
    ] {
        assert!(content.contains(&format!(r#"<module name="{}""#, provider)));
    }
    assert_eq!(content.matches(r#"<feature name="Payment""#).count(), 1);

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_camera_only_does_not_add_disabled_features() {
    let path = temp_file("unipack-camera-only-properties");
    let config = module_config_with_camera_share_oauth_payment();
    let enabled = vec!["Camera".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content
        .contains(r#"<feature name="Camera" value="io.dcloud.js.camera.CameraFeatureImpl"/>"#));
    assert!(!content.contains(r#"<feature name="Share""#));
    assert!(!content.contains(r#"<feature name="OAuth""#));
    assert!(!content.contains(r#"<feature name="Login""#));
    assert!(!content.contains(r#"<feature name="Payment""#));
    assert!(!content.contains(r#"<feature name="Ad""#));

    let _ = std::fs::remove_file(path);
}

#[test]
fn app_plus_other_modules_are_reported_and_generate_properties() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-other-modules-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "VideoPlayer": {},
                "Barcode": {},
                "Bluetooth": {},
                "iBeacon": {},
                "Contacts": {},
                "Fingerprint": {},
                "Messaging": {},
                "Record": {},
                "SQLite": {},
                "gcanvas": {},
                "Webview-x5": {}
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );
    let report =
        android_module_config_report_from_value(&info.detected_modules, Some(&manifest), None);
    let template_keys = report
        .modules
        .iter()
        .map(|module| module.template_key.as_str())
        .collect::<Vec<_>>();

    for key in [
        "video_player",
        "barcode",
        "bluetooth",
        "ibeacon",
        "contacts",
        "fingerprint",
        "messaging",
        "record",
        "sqlite",
        "gcanvas",
        "x5_tbs",
    ] {
        assert!(template_keys.contains(&key), "{key} should be reported");
    }
    assert!(report
        .modules
        .iter()
        .filter(|module| module.template_key != "livepusher")
        .all(|module| module.fields.is_empty()));

    let config = module_config_from_detected_modules(&info.detected_modules);
    let enabled = info
        .detected_modules
        .iter()
        .map(|module| module.name.clone())
        .collect::<Vec<_>>();
    let path = temp_file("unipack-other-modules-properties");

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    for feature in [
        r#"<feature name="VideoPlayer" value="io.dcloud.media.MediaFeatureImpl"/>"#,
        r#"<feature name="Barcode" value="io.dcloud.feature.barcode2.BarcodeFeatureImpl"/>"#,
        r#"<feature name="Bluetooth" value="io.dcloud.feature.bluetooth.BluetoothFeature"/>"#,
        r#"<feature name="iBeacon" value="io.dcloud.feature.iBeacon.WxBluetoothFeatureImpl"/>"#,
        r#"<feature name="Contacts" value="io.dcloud.feature.contacts.ContactsFeatureImpl"/>"#,
        r#"<feature name="Fingerprint" value="io.dcloud.feature.fingerprint.FingerPrintsImpl"/>"#,
        r#"<feature name="Messaging" value="io.dcloud.adapter.messaging.MessagingPluginImpl"/>"#,
        r#"<feature name="Sqlite" value="io.dcloud.feature.sqlite.DataBaseFeature"/>"#,
        r#"<feature name="X5Webview" value="io.dcloud.feature.X5Webview.X5WebViewService"/>"#,
    ] {
        assert!(content.contains(feature), "{feature} should be generated");
    }

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_push_adds_feature_and_service() {
    let path = temp_file("unipack-push-properties");
    let mut config = ModuleConfigTree::default();
    config.push = Some(PushModuleConfig {
        enabled: true,
        unipush_appid: Some("demo-appid".to_string()),
        unipush_appkey: Some("demo-appkey".to_string()),
        unipush_appsecret: Some("demo-secret".to_string()),
        vendors: Vec::new(),
    });
    let enabled = vec!["Push".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(
        content.contains(r#"<feature name="Push" value="io.dcloud.feature.aps.APSFeatureImpl">"#)
    );
    assert!(content
        .contains(r#"<module name="unipush" value="io.dcloud.feature.unipush.GTPushService"/>"#));
    assert!(
        content.contains(r#"<service name="push" value="io.dcloud.feature.aps.APSFeatureImpl"/>"#)
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_umeng_statistic_replaces_template_defaults() {
    let path = temp_file("unipack-umeng-statistic-properties");
    std::fs::write(
        &path,
        r#"<properties>
	<features>
		<feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl"/>
	</features>
	<services>
		<service name="Statistic" value="io.dcloud.feature.statistics.StatisticsBootImpl"/>
	</services>
</properties>"#,
    )
    .unwrap();
    let mut config = ModuleConfigTree::default();
    config.statistic = Some(StatisticModuleConfig {
        enabled: true,
        provider: "umeng".to_string(),
        umeng: Some(HashMap::new()),
        mta: None,
        baidu: None,
    });
    let enabled = vec!["Statistic".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains(
        r#"<feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl">"#
    ));
    assert!(content.contains(
        r#"<module name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.UmengStatistics"/>"#
    ));
    assert!(content.contains(
        r#"<service name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.StatisticsBootImpl"/>"#
    ));
    assert!(!content.contains(r#"<service name="Statistic" "#));
    assert_eq!(content.matches(r#"feature name="Statistic""#).count(), 1);
    assert_eq!(
        content.matches(r#"service name="Statistic-Umeng""#).count(),
        1
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_adds_canonical_oauth_and_share_features() {
    let path = temp_file("unipack-oauth-share-properties");
    let mut config = ModuleConfigTree::default();
    config.login = Some(LoginModuleConfig {
        enabled: true,
        providers: vec![LoginProvider {
            name: "weixin".to_string(),
            enabled: true,
            config: HashMap::new(),
        }],
    });
    config.share = Some(ShareModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        qq: Some(HashMap::new()),
        sina: None,
    });
    let enabled = vec!["OAuth".to_string(), "Share".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content
        .contains(r#"<feature name="OAuth" value="io.dcloud.feature.oauth.OAuthFeatureImpl""#));
    assert!(content.contains(
        r#"<module name="OAuth-Weixin" value="io.dcloud.feature.oauth.weixin.WeiXinOAuthService"/>"#
    ));
    assert!(!content.contains(r#"<feature name="Login""#));
    assert!(content.contains(r#"<feature name="Share" value="io.dcloud.share.ShareFeatureImpl""#));
    assert!(
        content.contains(r#"<module name="Weixin" value="io.dcloud.share.mm.WeiXinApiManager"/>"#)
    );
    assert!(content.contains(r#"<module name="QQ" value="io.dcloud.share.qq.QQApiManager"/>"#));

    let _ = std::fs::remove_file(path);
}

#[test]
fn dcloud_properties_generation_is_idempotent_and_treats_login_as_oauth() {
    let path = temp_file("unipack-idempotent-properties");
    std::fs::write(
        &path,
        r#"<properties>
	<features>
		<feature name="Login" value="io.dcloud.feature.login.LoginFeatureImpl"/>
	</features>
</properties>"#,
    )
    .unwrap();
    let mut config = ModuleConfigTree::default();
    config.login = Some(LoginModuleConfig {
        enabled: true,
        providers: vec![LoginProvider {
            name: "weixin".to_string(),
            enabled: true,
            config: HashMap::new(),
        }],
    });
    let enabled = vec!["OAuth".to_string()];

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert_eq!(content.matches(r#"feature name="Login""#).count(), 1);
    assert_eq!(content.matches(r#"feature name="OAuth""#).count(), 0);

    let _ = std::fs::remove_file(path);
}

#[test]
fn android_config_report_prefers_manifest_and_lists_missing() {
    let modules = vec![
        DetectedModule {
            name: "Share".to_string(),
            category: "share".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "Payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["ios".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
    ];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "share": {
                        "weixin": { "appid": "wx-demo", "appSecret": "wx-secret" }
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

    assert_eq!(report.modules.len(), 1);
    let share = &report.modules[0];
    assert_eq!(share.template_key, "share");
    assert_eq!(
        share
            .fields
            .iter()
            .find(|field| field.key == "WX_APPID")
            .and_then(|field| field.value.as_deref()),
        Some("wx-demo")
    );
    assert!(!report
        .missing_required
        .iter()
        .any(|missing| missing.key == "QQ_APPID"));
    assert!(report.all_configured);
}

#[test]
fn android_config_report_uses_scoped_user_values_for_duplicate_keys() {
    let modules = vec![
        DetectedModule {
            name: "Share".to_string(),
            category: "share".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "OAuth".to_string(),
            category: "login".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
    ];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "share": { "sinaweibo": {} },
                    "oauth": { "sinaweibo": {} }
                }
            }
        }
    });
    let mut user = HashMap::new();
    user.insert("share.SINA_APPKEY".to_string(), "share-key".to_string());
    user.insert("share.SINA_SECRET".to_string(), "share-secret".to_string());
    user.insert(
        "share.SINA_REDIRECT_URI".to_string(),
        "share-redirect".to_string(),
    );
    user.insert("login.SINA_APPKEY".to_string(), "login-key".to_string());
    user.insert(
        "login.SINA_REDIRECT_URI".to_string(),
        "login-redirect".to_string(),
    );

    let report = android_module_config_report_from_value(&modules, Some(&manifest), Some(&user));
    let share = report
        .modules
        .iter()
        .find(|module| module.template_key == "share")
        .unwrap();
    let login = report
        .modules
        .iter()
        .find(|module| module.template_key == "login")
        .unwrap();

    for (module, key, expected) in [
        (share, "SINA_APPKEY", "share-key"),
        (share, "SINA_SECRET", "share-secret"),
        (share, "SINA_REDIRECT_URI", "share-redirect"),
        (login, "SINA_APPKEY", "login-key"),
        (login, "SINA_REDIRECT_URI", "login-redirect"),
    ] {
        let field = module.fields.iter().find(|field| field.key == key).unwrap();
        assert_eq!(field.value.as_deref(), Some(expected));
        assert_eq!(field.value_source.as_deref(), Some("user"));
        assert!(field.required);
    }
    assert!(report.all_configured);
}

#[test]
fn android_config_report_prefers_manifest_over_cached_values() {
    let modules = vec![DetectedModule {
        name: "Statistic".to_string(),
        category: "statistic".to_string(),
        platforms: vec!["all".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "statistic": { "umeng": { "appkey": "manifest-key" } }
                }
            }
        }
    });
    let mut user = HashMap::new();
    user.insert("UMENG_APPKEY".to_string(), "cached-key".to_string());

    let report = android_module_config_report_from_value(&modules, Some(&manifest), Some(&user));
    let field = report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "UMENG_APPKEY")
        .unwrap();

    assert_eq!(field.value.as_deref(), Some("manifest-key"));
    assert_eq!(field.value_source.as_deref(), Some("manifest"));
    assert!(report.all_configured);
}

#[test]
fn android_config_analysis_uses_cached_manifest_value_when_manifest_path_is_missing() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-missing-manifest-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Share": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "share": {
                        "weixin": {
                            "appid": "wx-cached",
                            "appSecret": "wx-secret"
                        }
                    }
                }
            }
        }
    });
    let mut info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );
    info.manifest_path = project_root
        .join("manifest-was-not-read-again.json")
        .to_string_lossy()
        .to_string();

    let report = analyze_android_module_config_sync(&info, None);
    let share = report
        .modules
        .iter()
        .find(|module| module.template_key == "share")
        .expect("Share config should be produced from cached manifestValue");

    assert_eq!(
        share
            .fields
            .iter()
            .find(|field| field.key == "WX_APPID")
            .and_then(|field| field.value.as_deref()),
        Some("wx-cached")
    );
    assert!(report.all_configured);
}

#[test]
fn android_config_report_requires_only_enabled_provider_fields() {
    let modules = vec![DetectedModule {
        name: "Share".to_string(),
        category: "share".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "share": {
                        "weixin": { "appid": "wx-only" },
                        "qq": false
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

    assert!(report
        .missing_required
        .iter()
        .any(|missing| missing.key == "WX_SECRET"));
    assert!(!report
        .missing_required
        .iter()
        .any(|missing| missing.key == "QQ_APPID"));
    let field_keys = report.modules[0]
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<Vec<_>>();
    assert!(field_keys.contains(&"WX_APPID"));
    assert!(field_keys.contains(&"WX_SECRET"));
    assert!(!field_keys.contains(&"QQ_APPID"));
    assert!(!field_keys.contains(&"SINA_APPKEY"));
}

#[test]
fn android_config_report_shows_only_enabled_oauth_provider_fields() {
    let modules = vec![DetectedModule {
        name: "OAuth".to_string(),
        category: "login".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "OAuth": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "weixin": {
                            "appid": "wx-login",
                            "UniversalLinks": "https://example.com/app/"
                        }
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

    assert_eq!(report.modules.len(), 1);
    let login = &report.modules[0];
    let field_keys = login
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(field_keys, vec!["WX_APPID", "WX_SECRET"]);
    assert_eq!(
        login
            .fields
            .iter()
            .find(|field| field.key == "WX_APPID")
            .and_then(|field| field.value.as_deref()),
        Some("wx-login")
    );
    assert!(report
        .missing_required
        .iter()
        .any(|missing| missing.key == "WX_SECRET"));
    assert!(!report
        .missing_required
        .iter()
        .any(|missing| missing.key == "QQ_APPID"));
}

#[test]
fn enabled_oauth_providers_require_their_official_configuration_fields() {
    let modules = vec![DetectedModule {
        name: "OAuth".to_string(),
        category: "login".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "sinaweibo": {},
                        "miui": {},
                        "facebook": {}
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

    for key in [
        "SINA_APPKEY",
        "SINA_REDIRECT_URI",
        "MIUI_APPID",
        "MIUI_APPSECRET",
        "MIUI_REDIRECT_URI",
        "FACEBOOK_APP_ID",
        "FACEBOOK_CLIENT_TOKEN",
    ] {
        assert!(report
            .modules
            .iter()
            .flat_map(|module| module.fields.iter())
            .any(|field| field.key == key && field.required));
        assert!(report
            .missing_required
            .iter()
            .any(|missing| missing.key == key));
    }
}

#[test]
fn android_config_report_honors_nested_push_and_platform_providers() {
    let modules = vec![
        DetectedModule {
            name: "Push".to_string(),
            category: "push".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "Payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
    ];
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Push": {},
                "Payment": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "push": {
                        "unipush": {
                            "offline": true,
                            "mi": { "appid": "mi-app", "appkey": "mi-key" },
                            "hms": { "appid": "huawei-app" },
                            "oppo": false,
                            "vivo": { "__platform__": ["ios"] }
                        }
                    },
                    "payment": {
                        "weixin": {
                            "__platform__": ["ios"],
                            "appid": "wx-ios-only"
                        },
                        "paypal": {
                            "__platform__": ["ios", "android"],
                            "returnURL_android": "paypal-demo"
                        }
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
    let push = report
        .modules
        .iter()
        .find(|module| module.template_key == "push")
        .unwrap();
    let push_keys = push
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<Vec<_>>();
    assert!(push_keys.contains(&"XIAOMI_APP_ID"));
    assert!(push_keys.contains(&"XIAOMI_APP_KEY"));
    assert!(push_keys.contains(&"HUAWEI_APP_ID"));
    assert!(!push_keys.contains(&"OPPO_APP_KEY"));
    assert!(!push_keys.contains(&"VIVO_APP_ID"));

    let payment = report
        .modules
        .iter()
        .find(|module| module.template_key == "payment")
        .unwrap();
    let payment_keys = payment
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(payment_keys, vec!["PAYPAL_RETURN_SCHEME"]);
    assert_eq!(payment.fields[0].value.as_deref(), Some("paypal-demo"));
}

#[test]
fn paypal_payment_requires_return_scheme_and_official_repository() {
    let modules = vec![DetectedModule {
        name: "Payment".to_string(),
        category: "payment".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "paypal": {}
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
    let field = report
        .modules
        .iter()
        .flat_map(|module| module.fields.iter())
        .find(|field| field.key == "PAYPAL_RETURN_SCHEME")
        .expect("PayPal return scheme field");
    assert!(field.required);
    assert!(report
        .missing_required
        .iter()
        .any(|missing| missing.key == "PAYPAL_RETURN_SCHEME"));

    let repositories = android_module_gradle_repositories_for_manifest("payment", Some(&manifest));
    assert_eq!(repositories.len(), 2);
    assert!(repositories[0].contains("maven.aliyun.com/repository/public"));
    assert!(repositories[1].contains("cardinalcommerceprod.jfrog.io"));
    assert!(repositories[1].contains("paypal_sgerritz"));
}

#[test]
fn stripe_payment_adds_maven_central_mirror_repository() {
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "stripe": {}
                    }
                }
            }
        }
    });

    let repositories = android_module_gradle_repositories_for_manifest("payment", Some(&manifest));

    assert_eq!(repositories, vec![
        "maven { url 'https://maven.aliyun.com/repository/public' }"
    ]);
}

#[test]
fn all_selected_payment_providers_enable_their_artifacts_and_dependencies() {
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "alipay": {},
                        "weixin": {},
                        "paypal": {},
                        "stripe": {},
                        "googlepay": {}
                    }
                }
            }
        }
    });

    for artifact in [
        "payment-alipay-release.aar (支付宝)",
        "payment-weixin-release.aar (微信支付)",
        "payment-paypal-release.aar (PayPal)",
        "payment-stripe-release.aar (Stripe)",
        "payment-google-release.aar (Google Pay)",
    ] {
        assert!(android_module_artifact_enabled_for_manifest(
            "payment",
            artifact,
            Some(&manifest),
        ));
    }
    for dependency in [
        "com.alipay.sdk:alipaysdk-android:15.8.11 (支付宝)",
        "com.tencent.mm.opensdk:wechat-sdk-android-without-mta:6.8.0 (微信支付)",
        "com.paypal.checkout:android-sdk:0.6.2 (PayPal)",
        "androidx.legacy:legacy-support-v4:${rootProject.ext.androidxVersion} (Stripe)",
        "com.stripe:stripe-android:18.2.0 (Stripe)",
        "com.google.android.gms:play-services-wallet:18.1.3 (Google Pay)",
    ] {
        assert!(android_module_gradle_dependency_enabled_for_manifest(
            "payment",
            dependency,
            Some(&manifest),
        ));
    }
}

#[test]
fn payment_androidx_version_is_shown_for_stripe_or_google_pay_with_default_and_user_override() {
    let modules = vec![DetectedModule {
        name: "Payment".to_string(),
        category: "payment".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let stripe_manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "stripe": {}
                    }
                }
            }
        }
    });
    let google_manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "googlepay": {}
                    }
                }
            }
        }
    });
    let paypal_manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "paypal": {}
                    }
                }
            }
        }
    });

    let default_report =
        android_module_config_report_from_value(&modules, Some(&stripe_manifest), None);
    let default_version = default_report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "androidxVersion")
        .unwrap();
    assert_eq!(default_version.value.as_deref(), Some("1.0.0"));
    assert_eq!(default_version.value_source.as_deref(), Some("default"));
    assert!(!default_version.required);

    let google_report =
        android_module_config_report_from_value(&modules, Some(&google_manifest), None);
    assert!(google_report.modules[0]
        .fields
        .iter()
        .any(|field| field.key == "androidxVersion"));

    let paypal_report =
        android_module_config_report_from_value(&modules, Some(&paypal_manifest), None);
    assert!(!paypal_report.modules[0]
        .fields
        .iter()
        .any(|field| field.key == "androidxVersion"));

    let mut user = HashMap::new();
    user.insert("androidxVersion".to_string(), "1.7.0".to_string());
    let user_report =
        android_module_config_report_from_value(&modules, Some(&stripe_manifest), Some(&user));
    let user_version = user_report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "androidxVersion")
        .unwrap();
    assert_eq!(user_version.value.as_deref(), Some("1.7.0"));
    assert_eq!(user_version.value_source.as_deref(), Some("user"));
}

#[test]
fn android_config_report_requires_enabled_push_vendor_fields() {
    let modules = vec![DetectedModule {
        name: "Push".to_string(),
        category: "push".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Push": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "push": {
                        "unipush": {
                            "hms": {},
                            "oppo": {},
                            "vivo": false
                        }
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
    let push = report
        .modules
        .iter()
        .find(|module| module.template_key == "push")
        .unwrap();

    for key in ["HUAWEI_APP_ID", "OPPO_APP_KEY", "OPPO_APP_SECRET"] {
        assert!(push
            .fields
            .iter()
            .any(|field| field.key == key && field.required));
        assert!(report
            .missing_required
            .iter()
            .any(|missing| missing.key == key));
    }
    assert!(!push.fields.iter().any(|field| field.key == "VIVO_APP_ID"));
}

#[test]
fn geolocation_config_requires_only_enabled_provider_keys() {
    let modules = vec![DetectedModule {
        name: "Geolocation".to_string(),
        category: "geolocation".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Geolocation": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "amap": {
                            "__platform__": ["android"],
                            "appkey_android": ""
                        },
                        "baidu": false,
                        "system": true
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
    let geolocation = report
        .modules
        .iter()
        .find(|module| module.template_key == "geolocation")
        .unwrap();

    assert_eq!(geolocation.fields.len(), 1);
    assert_eq!(geolocation.fields[0].key, "AMAP_KEY");
    assert!(geolocation.fields[0].required);
    assert!(report
        .missing_required
        .iter()
        .any(|missing| missing.key == "AMAP_KEY"));
    assert!(!geolocation
        .fields
        .iter()
        .any(|field| field.key == "BAIDU_MAP_AK"));
}

#[test]
fn system_geolocation_has_no_manual_config_fields() {
    let modules = vec![DetectedModule {
        name: "Geolocation".to_string(),
        category: "geolocation".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Geolocation": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "system": true
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

    assert!(report.modules[0].fields.is_empty());
    assert!(report.all_configured);
}

#[test]
fn tencent_geolocation_sdk_version_defaults_and_accepts_user_override() {
    let modules = vec![DetectedModule {
        name: "Geolocation".to_string(),
        category: "geolocation".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "tencent": {
                            "apikey_android": "tencent-demo"
                        }
                    }
                }
            }
        }
    });

    let default_report = android_module_config_report_from_value(&modules, Some(&manifest), None);
    let default_version = default_report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "TENCENT_LOCATION_SDK_VERSION")
        .unwrap();
    assert_eq!(default_version.value.as_deref(), Some("7.5.4.8"));
    assert_eq!(default_version.value_source.as_deref(), Some("default"));
    assert!(!default_version.required);

    let mut user = HashMap::new();
    user.insert(
        "TENCENT_LOCATION_SDK_VERSION".to_string(),
        "2.3.1".to_string(),
    );
    let user_report =
        android_module_config_report_from_value(&modules, Some(&manifest), Some(&user));
    let user_version = user_report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "TENCENT_LOCATION_SDK_VERSION")
        .unwrap();
    assert_eq!(user_version.value.as_deref(), Some("2.3.1"));
    assert_eq!(user_version.value_source.as_deref(), Some("user"));
}

#[test]
fn geolocation_artifacts_and_dependencies_follow_enabled_provider() {
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "amap": {
                            "__platform__": ["android"],
                            "appkey_android": "amap-demo"
                        },
                        "baidu": false,
                        "tencent": false
                    }
                }
            }
        }
    });

    assert!(android_module_artifact_enabled_for_manifest(
        "geolocation",
        "geolocation-amap-release.aar (高德定位)",
        Some(&manifest),
    ));
    assert!(!android_module_artifact_enabled_for_manifest(
        "geolocation",
        "geolocation-baidu-release.aar (百度定位)",
        Some(&manifest),
    ));
    assert!(android_module_gradle_dependency_enabled_for_manifest(
        "geolocation",
        "com.amap.api:location:6.4.5 (高德定位)",
        Some(&manifest),
    ));
    assert!(!android_module_gradle_dependency_enabled_for_manifest(
        "geolocation",
        "com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8 (腾讯定位)",
        Some(&manifest),
    ));
}

#[test]
fn univerify_login_adds_getui_repository_without_push() {
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "univerify": {}
                    }
                }
            }
        }
    });

    assert_eq!(
        android_module_gradle_repositories_for_manifest("login", Some(&manifest)),
        vec!["maven { url 'https://mvn.getui.com/nexus/content/repositories/releases' }"]
    );
}

#[test]
fn statistic_artifacts_keep_umeng_variants_mutually_exclusive() {
    let umeng_manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "statistic": {
                        "umeng": {
                            "appkey": "umeng-demo"
                        }
                    }
                }
            }
        }
    });
    let umeng_google_play_manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "statistic": {
                        "umeng-gp": {
                            "appkey": "umeng-gp-demo"
                        }
                    }
                }
            }
        }
    });

    assert!(android_module_artifact_enabled_for_manifest(
        "statistic",
        "statistic-umeng-release.aar (友盟统计)",
        Some(&umeng_manifest),
    ));
    assert!(!android_module_artifact_enabled_for_manifest(
        "statistic",
        "statistic-umeng-gp-release.aar (友盟 Google Play)",
        Some(&umeng_manifest),
    ));
    assert!(!android_module_artifact_enabled_for_manifest(
        "statistic",
        "statistic-umeng-release.aar (友盟统计)",
        Some(&umeng_google_play_manifest),
    ));
    assert!(android_module_artifact_enabled_for_manifest(
        "statistic",
        "statistic-umeng-gp-release.aar (友盟 Google Play)",
        Some(&umeng_google_play_manifest),
    ));
}

#[test]
fn amap_map_replaces_amap_geolocation_sdk_integration() {
    let modules = vec![
        DetectedModule {
            name: "Geolocation".to_string(),
            category: "geolocation".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
        DetectedModule {
            name: "Maps".to_string(),
            category: "map".to_string(),
            platforms: vec!["android".to_string()],
            configured: false,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        },
    ];
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Geolocation": {},
                "Maps": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "amap": {
                            "appkey_android": ""
                        }
                    },
                    "maps": {
                        "amap": {
                            "appkey_android": ""
                        }
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
    let geolocation = report
        .modules
        .iter()
        .find(|module| module.template_key == "geolocation")
        .unwrap();
    let map = report
        .modules
        .iter()
        .find(|module| module.template_key == "map")
        .unwrap();

    assert!(geolocation.fields.is_empty());
    assert!(map
        .fields
        .iter()
        .any(|field| field.key == "AMAP_KEY" && field.required));
    assert!(report
        .missing_required
        .iter()
        .any(|missing| missing.module_name == "Maps" && missing.key == "AMAP_KEY"));
    assert!(!android_module_artifact_enabled_for_manifest(
        "geolocation",
        "geolocation-amap-release.aar (高德定位)",
        Some(&manifest),
    ));
    assert!(!android_module_gradle_dependency_enabled_for_manifest(
        "geolocation",
        "com.amap.api:location:6.4.5 (高德定位)",
        Some(&manifest),
    ));
    assert!(android_module_artifact_enabled_for_manifest(
        "map",
        "map-amap-release.aar (高德 vue 页面)",
        Some(&manifest),
    ));
}

#[test]
fn map_page_type_defaults_to_vue_and_allows_amap_nvue() {
    let modules = vec![DetectedModule {
        name: "Maps".to_string(),
        category: "map".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "maps": {
                        "amap": {
                            "appkey_android": "amap-demo",
                            "sdkVersion": "9.9.9"
                        }
                    },
                    "push": {
                        "unipush": {
                            "version": "2"
                        }
                    }
                }
            }
        }
    });
    let mut user = HashMap::new();
    user.insert("MAP_PAGE_TYPE".to_string(), "nvue".to_string());

    let report = android_module_config_report_from_value(&modules, Some(&manifest), Some(&user));
    let field = report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "MAP_PAGE_TYPE")
        .unwrap();
    let version_field = report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "AMAP_SDK_VERSION")
        .unwrap();

    assert_eq!(field.value.as_deref(), Some("nvue"));
    assert_eq!(field.field_type, "select");
    assert_eq!(
        version_field.value.as_deref(),
        Some("10.0.700_loc6.4.5_sea9.7.2")
    );
    assert_eq!(version_field.value_source.as_deref(), Some("default"));
}

#[test]
fn google_map_reads_dcloud_android_api_key_from_manifest() {
    let modules = vec![DetectedModule {
        name: "Maps".to_string(),
        category: "map".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "maps": {
                        "google": {
                            "APIKey_ios": "ios-google-key",
                            "APIKey_android": "android-google-key"
                        }
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);
    let map = report
        .modules
        .iter()
        .find(|module| module.template_key == "map")
        .unwrap();
    let key = map
        .fields
        .iter()
        .find(|field| field.key == "GOOGLE_MAPS_API_KEY")
        .unwrap();
    let page_type = map
        .fields
        .iter()
        .find(|field| field.key == "MAP_PAGE_TYPE")
        .unwrap();

    assert_eq!(key.value.as_deref(), Some("android-google-key"));
    assert_eq!(key.value_source.as_deref(), Some("manifest"));
    assert_eq!(page_type.value.as_deref(), Some("nvue"));
    assert!(!report
        .missing_required
        .iter()
        .any(|missing| missing.key == "GOOGLE_MAPS_API_KEY"));
    assert!(android_module_artifact_enabled_for_manifest(
        "map",
        "weex_google-map-release.aar (Google nvue 页面)",
        Some(&manifest),
    ));
    assert!(android_module_gradle_dependency_enabled_for_manifest(
        "map",
        "com.google.android.gms:play-services-maps:18.0.1 (Google地图)",
        Some(&manifest),
    ));
}

#[test]
fn amap_map_sdk_version_prefers_build_center_value() {
    let modules = vec![DetectedModule {
        name: "Maps".to_string(),
        category: "map".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "maps": {
                        "amap": {
                            "appkey_android": "amap-demo",
                            "sdkVersion": "9.9.9"
                        }
                    }
                }
            }
        }
    });
    let mut user = HashMap::new();
    user.insert(
        "AMAP_SDK_VERSION".to_string(),
        "10.0.700_custom".to_string(),
    );

    let report = android_module_config_report_from_value(&modules, Some(&manifest), Some(&user));
    let version_field = report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "AMAP_SDK_VERSION")
        .unwrap();

    assert_eq!(version_field.value.as_deref(), Some("10.0.700_custom"));
    assert_eq!(version_field.value_source.as_deref(), Some("user"));
}

#[test]
fn baidu_map_page_type_forces_vue() {
    let modules = vec![DetectedModule {
        name: "Maps".to_string(),
        category: "map".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "maps": {
                        "baidu": {
                            "appkey_android": "baidu-demo"
                        }
                    }
                }
            }
        }
    });
    let mut user = HashMap::new();
    user.insert("MAP_PAGE_TYPE".to_string(), "nvue".to_string());

    let report = android_module_config_report_from_value(&modules, Some(&manifest), Some(&user));
    let field = report.modules[0]
        .fields
        .iter()
        .find(|field| field.key == "MAP_PAGE_TYPE")
        .unwrap();

    assert_eq!(field.value.as_deref(), Some("vue"));
    assert_eq!(field.value_source.as_deref(), Some("default"));
}

#[test]
fn map_dcloud_properties_follow_selected_provider() {
    let path = temp_file("unipack-map-provider-properties");
    let enabled = vec!["Maps".to_string()];
    let mut config = ModuleConfigTree::default();
    config.map = Some(MapModuleConfig {
        enabled: true,
        engine: "amap".to_string(),
        amap_key: Some("amap-demo".to_string()),
        tencent_map_key: None,
        baidu_map_ak: None,
        google_maps_api_key: None,
    });

    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let amap_content = std::fs::read_to_string(&path).unwrap();
    assert!(amap_content.contains(
        r#"<feature name="Maps" value="io.dcloud.js.map.amap.JsMapPluginImpl"></feature>"#
    ));
    assert!(!amap_content.contains(r#"<service name="Maps""#));

    config.map = Some(MapModuleConfig {
        enabled: true,
        engine: "baidu".to_string(),
        amap_key: None,
        tencent_map_key: None,
        baidu_map_ak: Some("baidu-demo".to_string()),
        google_maps_api_key: None,
    });
    generate_dcloud_properties(&path, &config, &enabled).unwrap();
    let baidu_content = std::fs::read_to_string(&path).unwrap();
    assert!(baidu_content
        .contains(r#"<feature name="Maps" value="io.dcloud.js.map.JsMapPluginImpl"></feature>"#));
    assert!(
        baidu_content.contains(r#"<service name="Maps" value="io.dcloud.js.map.MapInitImpl"/>"#)
    );
    assert!(!baidu_content.contains("io.dcloud.js.map.amap.JsMapPluginImpl"));

    let _ = std::fs::remove_file(path);
}

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}.xml", name, uuid::Uuid::new_v4()))
}

fn module_config_with_camera_share_oauth_payment() -> ModuleConfigTree {
    let mut config = ModuleConfigTree::default();
    config.camera = Some(SimpleModuleConfig { enabled: true });
    config.share = Some(ShareModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        qq: Some(HashMap::new()),
        sina: Some(HashMap::new()),
    });
    config.login = Some(LoginModuleConfig {
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
    });
    config.payment = Some(PaymentModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        alipay: Some(HashMap::new()),
        paypal: Some(HashMap::new()),
        stripe: Some(HashMap::new()),
        google: Some(HashMap::new()),
    });
    config
}
