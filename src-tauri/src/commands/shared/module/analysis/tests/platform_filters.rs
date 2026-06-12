use super::*;

#[test]
fn geolocation_artifacts_and_dependencies_follow_enabled_provider() {
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
fn geolocation_sdk_config_without_module_does_not_enable_android_build_logic() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-geolocation-gate-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "baidu": {
                            "__platform__": ["android"],
                            "appkey_android": "baidu-demo"
                        }
                    }
                }
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    assert!(!info
        .detected_modules
        .iter()
        .any(|module| module.name == "Geolocation"));

    let stale_modules = vec![DetectedModule {
        name: "Geolocation".to_string(),
        category: "geolocation".to_string(),
        platforms: vec!["android".to_string()],
        configured: false,
        required_keys: vec![],
        source: "manifest.json".to_string(),
    }];
    let report = android_module_config_report_from_value(&stale_modules, Some(&manifest), None);
    assert!(!report
        .modules
        .iter()
        .any(|module| module.template_key == "geolocation"));
    assert!(!android_module_artifact_enabled_for_manifest(
        "geolocation",
        "geolocation-baidu-release.aar (百度定位)",
        Some(&manifest),
    ));

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn geolocation_provider_requires_android_platform() {
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
                        },
                        "baidu": {
                            "__platform__": ["ios"],
                            "appkey_android": "baidu-demo"
                        },
                        "amap": {
                            "appkey_android": "amap-demo"
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

    assert!(geolocation.fields.is_empty());
    assert!(!android_module_artifact_enabled_for_manifest(
        "geolocation",
        "geolocation-baidu-release.aar (百度定位)",
        Some(&manifest),
    ));
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
}

#[test]
fn ios_module_config_lists_geolocation_providers_by_ios_platform() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-ios-module-report-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Geolocation": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "system": {
                            "__platform__": ["ios", "android"]
                        },
                        "baidu": {
                            "__platform__": ["ios"],
                            "appkey_ios": "baidu-ios"
                        },
                        "amap": {
                            "__platform__": ["android"],
                            "appkey_ios": "amap-ios"
                        },
                        "tencent": {
                            "__platform__": ["ios"],
                            "apikey_ios": "tencent-ios"
                        }
                    }
                }
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let report = analyze_ios_module_config_sync(&info, None);
    let geolocation = report
        .modules
        .iter()
        .find(|module| module.template_key == "geolocation")
        .expect("geolocation should be listed for iOS");

    assert!(geolocation
        .fields
        .iter()
        .any(|field| field.key == "baidu.appkey_ios"
            && field.value.as_deref() == Some("baidu-ios")
            && field.value_source.as_deref() == Some("manifest")));
    assert!(!geolocation
        .fields
        .iter()
        .any(|field| field.key == "amap.appkey_ios"));
    assert!(!geolocation
        .fields
        .iter()
        .any(|field| field.key == "tencent.apikey_ios"));
    assert_eq!(
        geolocation
            .fields
            .iter()
            .filter(|field| field.key.starts_with("privacy."))
            .count(),
        4
    );

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn ios_module_config_lists_push_getui_fields() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-ios-push-module-report-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Push": {}
            },
            "distribute": {
                "push": {
                    "unipush": {
                        "version": "2"
                    }
                },
                "sdkConfigs": {
                    "push": {
                        "getui": {
                            "__platform__": ["ios"],
                            "appid": "manifest-getui-appid",
                            "appkey": "manifest-getui-appkey",
                            "appsecret": "manifest-getui-appsecret"
                        }
                    }
                }
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );
    let mut user = HashMap::new();
    user.insert(
        "push.getui.appkey".to_string(),
        "user-getui-appkey".to_string(),
    );

    let report = analyze_ios_module_config_sync(&info, Some(&user));
    let push = report
        .modules
        .iter()
        .find(|module| module.template_key == "push")
        .expect("push should be listed for iOS");

    assert!(push.fields.iter().any(|field| {
        field.key == "getui.appid"
            && field.value.as_deref() == Some("manifest-getui-appid")
            && field.value_source.as_deref() == Some("manifest")
            && field.required
            && !field.secret
    }));
    assert!(push.fields.iter().any(|field| {
        field.key == "getui.appkey"
            && field.value.as_deref() == Some("user-getui-appkey")
            && field.value_source.as_deref() == Some("user")
            && field.secret
    }));
    assert!(push.fields.iter().any(|field| {
        field.key == "getui.appsecret"
            && field.value.as_deref() == Some("manifest-getui-appsecret")
            && field.value_source.as_deref() == Some("manifest")
            && field.secret
    }));

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn ios_module_config_skips_push_when_unipush_version_is_not_v2() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-ios-push-module-skip-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Push": {}
            },
            "distribute": {
                "push": {
                    "unipush": {
                        "version": "1"
                    }
                },
                "sdkConfigs": {
                    "push": {
                        "getui": {
                            "__platform__": ["ios"],
                            "appid": "manifest-getui-appid"
                        }
                    }
                }
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let report = analyze_ios_module_config_sync(&info, None);

    assert!(!report
        .modules
        .iter()
        .any(|module| module.template_key == "push"));

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn ios_module_config_lists_barcode_and_bluetooth_fields() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-ios-barcode-bluetooth-report-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "BarCode": {},
                "Blutooth": {}
            },
            "distribute": {
                "ios": {
                    "privacyDescription": {
                        "NSCameraUsageDescription": "manifest 相机说明"
                    },
                    "UIBackgroundModes": ["audio"]
                }
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let report = analyze_ios_module_config_sync(&info, None);
    let barcode = report
        .modules
        .iter()
        .find(|module| module.template_key == "barcode")
        .expect("barcode should be listed for iOS");
    let bluetooth = report
        .modules
        .iter()
        .find(|module| module.template_key == "bluetooth")
        .expect("bluetooth should be listed for iOS");

    assert!(barcode.fields.iter().any(|field| {
        field.key == "privacy.NSCameraUsageDescription"
            && field.value.as_deref() == Some("manifest 相机说明")
            && field.value_source.as_deref() == Some("manifest")
    }));
    assert!(barcode.fields.iter().any(|field| {
        field.key == "privacy.NSPhotoLibraryUsageDescription"
            && field.value.as_deref() == Some("用于从相册选择图片进行扫码")
            && field.value_source.as_deref() == Some("default")
    }));
    assert!(bluetooth.fields.iter().any(|field| {
        field.key == "privacy.NSBluetoothAlwaysUsageDescription"
            && field.value.as_deref() == Some("用于连接和管理低功耗蓝牙设备")
    }));
    assert!(bluetooth.fields.iter().any(|field| {
        field.key == "backgroundBluetooth"
            && field.value.as_deref() == Some("false")
            && field.field_type == "select"
    }));

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn ios_module_config_lists_media_contact_auth_and_video_fields() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-ios-extra-module-report-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Camera": {},
                "Contacts": {},
                "FaceID": {},
                "Fingerprint": {},
                "iBeacon": {},
                "LivePusher": {},
                "Record": {},
                "VideoPlayer": {},
                "Messaging": {},
                "SQLite": {},
                "UIWebview": {}
            },
            "distribute": {
                "ios": {
                    "privacyDescription": {
                        "NSContactsUsageDescription": "manifest 通讯录说明"
                    },
                    "NSAppTransportSecurity": {
                        "NSAllowsArbitraryLoads": true
                    }
                }
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let report = analyze_ios_module_config_sync(&info, None);
    let module = |key: &str| {
        report
            .modules
            .iter()
            .find(|module| module.template_key == key)
            .unwrap_or_else(|| panic!("{key} should be listed for iOS"))
    };

    assert!(module("camera").fields.iter().any(|field| {
        field.key == "privacy.NSCameraUsageDescription"
            && field.value.as_deref() == Some("用于拍摄照片或视频")
    }));
    assert!(module("camera").fields.iter().any(|field| {
        field.key == "privacy.NSPhotoLibraryAddUsageDescription"
            && !field.required
            && field.value.is_none()
    }));
    assert!(module("contacts").fields.iter().any(|field| {
        field.key == "privacy.NSContactsUsageDescription"
            && field.value.as_deref() == Some("manifest 通讯录说明")
            && field.value_source.as_deref() == Some("manifest")
    }));
    assert!(module("face_id")
        .fields
        .iter()
        .any(|field| field.key == "privacy.NSFaceIDUsageDescription"));
    assert!(module("fingerprint")
        .fields
        .iter()
        .any(|field| field.key == "privacy.NSFaceIDUsageDescription"));
    assert!(module("ibeacon").fields.iter().any(|field| {
        field.key == "privacy.NSLocationAlwaysAndWhenInUseUsageDescription"
            && field.value.as_deref() == Some("扫描蓝牙 Beacon 设备")
    }));
    assert!(module("livepusher")
        .fields
        .iter()
        .any(|field| field.key == "privacy.NSMicrophoneUsageDescription"));
    assert!(module("record")
        .fields
        .iter()
        .any(|field| field.key == "privacy.NSMicrophoneUsageDescription"));
    assert!(module("video_player").fields.iter().any(|field| {
        field.key == "allowArbitraryLoads"
            && field.value.as_deref() == Some("true")
            && field.value_source.as_deref() == Some("manifest")
            && field.field_type == "select"
    }));
    assert!(module("messaging").fields.is_empty());
    assert!(module("sqlite").fields.is_empty());
    assert!(module("ui_webview").fields.is_empty());

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn ios_module_config_ignores_geolocation_without_module_switch() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-ios-module-report-disabled-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "baidu": {
                            "__platform__": ["ios"],
                            "appkey_ios": "baidu-ios"
                        }
                    }
                }
            }
        }
    });
    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let report = analyze_ios_module_config_sync(&info, None);
    assert!(!report
        .modules
        .iter()
        .any(|module| module.template_key == "geolocation"));

    let _ = std::fs::remove_dir_all(project_root);
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
                            "__platform__": ["android"],
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
