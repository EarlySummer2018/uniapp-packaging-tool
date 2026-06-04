use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::analysis::android_module_config_report_from_value;
use crate::commands::shared::module::parsing::module_config_from_detected_modules;
use crate::commands::shared::module::properties::generate_dcloud_properties;

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
    let path = temp_file("unipack-dcloud-properties");

    generate_dcloud_properties(&path, &config).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains(r#"<feature name="Login""#));
    assert!(content.contains(r#"<module name="WeixinLogin"/>"#));
    assert!(content.contains(r#"<feature name="Payment">"#));
    assert!(content.contains(r#"<module name="Alipay"/>"#));
    assert!(content.contains(r#"<feature name="Share""#));
    assert_eq!(content.matches("<features>").count(), 1);
    assert_eq!(content.matches("</features>").count(), 1);

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

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}.xml", name, uuid::Uuid::new_v4()))
}
