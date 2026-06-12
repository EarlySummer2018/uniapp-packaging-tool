use super::*;

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
                            "version": 2,
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

    assert_eq!(
        repositories,
        vec!["maven { url 'https://maven.aliyun.com/repository/public' }"]
    );
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
                            "version": "2",
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
fn android_config_report_skips_push_when_unipush_version_is_not_v2() {
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
                            "version": "1",
                            "hms": {}
                        }
                    }
                }
            }
        }
    });

    let report = android_module_config_report_from_value(&modules, Some(&manifest), None);

    assert!(!report
        .modules
        .iter()
        .any(|module| module.template_key == "push"));
    assert!(report.missing_required.is_empty());
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
                        "system": {
                            "__platform__": ["android"]
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
                        "system": {
                            "__platform__": ["android"]
                        }
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
            "modules": {
                "Geolocation": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "tencent": {
                            "__platform__": ["android"],
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
