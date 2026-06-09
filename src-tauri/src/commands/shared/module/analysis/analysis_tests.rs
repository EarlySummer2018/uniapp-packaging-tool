use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::analysis::{
    analyze_android_module_config_sync, android_module_artifact_enabled_for_manifest,
    android_module_config_report_from_value, android_module_gradle_dependency_enabled_for_manifest,
};
use crate::commands::shared::module::parsing::module_config_from_detected_modules;
use crate::commands::shared::module::properties::generate_dcloud_properties;
use crate::commands::shared::module::types::{
    LoginModuleConfig, LoginProvider, MapModuleConfig, ModuleConfigTree, PaymentModuleConfig,
    PushModuleConfig, ShareModuleConfig, SimpleModuleConfig,
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
                            "appkey_android": "amap-demo"
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

    assert_eq!(field.value.as_deref(), Some("nvue"));
    assert_eq!(field.field_type, "select");
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
    });
    config
}
