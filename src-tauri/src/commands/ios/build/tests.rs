use std::path::Path;

use ::plist as plist_crate;

use super::super::icons::generate_app_icons;
use super::config::{
    effective_app_name, effective_app_version, effective_app_version_code,
    resolve_ios_manifest_info, validate_ios_app_id,
};
use super::entitlements::patch_ios_entitlements;
use super::fs_utils::{find_scheme_name, find_xcodeproj, link_ios_sdk_support};
use super::pbxproj::{
    append_pbx_build_setting_flag, legacy_simulator_x86_64_required, patch_pbxproj,
    register_pbx_embedded_frameworks, register_pbx_linked_files, set_pbx_build_setting,
    IosPbxLinkedFile,
};
use super::plist::{
    apply_ios_privacy_descriptions, ios_geolocation_provider_value, patch_info_plist,
    set_info_plist_string_value,
};
use super::runtime::{import_app_resource, patch_control_xml, resolve_ios_runtime_layout};
use super::splashscreen::apply_ios_splashscreen;
use crate::commands::ios::modules::bluetooth::apply_ios_bluetooth_module;
use crate::commands::ios::modules::facial_recognition_verify::apply_ios_facial_recognition_verify_module;
use crate::commands::ios::modules::geolocation::{
    apply_ios_geolocation_module, ios_geolocation_providers, IosGeolocationProvider,
};
use crate::commands::ios::modules::ibeacon::apply_ios_ibeacon_module;
use crate::commands::ios::modules::livepusher::apply_ios_livepusher_module;
use crate::commands::ios::modules::push::apply_ios_push_module;

#[test]
fn ios_build_reloads_manifest_from_configured_local_project() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-local-manifest-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(root.join("unpackage/res/icons")).unwrap();
    std::fs::write(root.join("unpackage/res/icons/1024.png"), "icon").unwrap();
    std::fs::write(
        root.join("manifest.json"),
        r#"{
                "name": "Manifest App",
                "appid": "__UNI__MANIFEST",
                "versionName": "2.3.4",
                "versionCode": "234",
                "app-plus": {
                    "distribute": {
                        "ios": {
                            "privacyDescription": {
                                "NSCameraUsageDescription": "用于扫码"
                            }
                        },
                        "icons": {
                            "ios": {
                                "appstore": "unpackage/res/icons/1024.png"
                            }
                        }
                    }
                }
            }"#,
    )
    .unwrap();
    let mut config = crate::commands::project::ProjectConfig::default();
    config.local_path = root.to_string_lossy().to_string();
    config.app.name = "Config App".into();
    config.app.version = "1.0.0".into();
    config.app.version_code = 1;

    let info = resolve_ios_manifest_info(&config, None).unwrap().unwrap();

    assert_eq!(effective_app_name(&config, Some(&info)), "Manifest App");
    assert_eq!(effective_app_version(&config, Some(&info)), "2.3.4");
    assert_eq!(effective_app_version_code(&config, Some(&info)), 234);
    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSCameraUsageDescription")
            .map(String::as_str),
        Some("用于扫码")
    );
    let expected_icon = root
        .join("unpackage/res/icons/1024.png")
        .to_string_lossy()
        .to_string();
    assert_eq!(
        info.ios_icons
            .as_ref()
            .and_then(|icons| icons.ios.get("appstore"))
            .map(String::as_str),
        Some(expected_icon.as_str())
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_build_center_overrides_survive_local_manifest_reload() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-build-center-overrides-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let manifest = serde_json::json!({
        "name": "Manifest App",
        "appid": "__UNI__MANIFEST",
        "app-plus": {
            "modules": {
                "Geolocation": {}
            },
            "distribute": {
                "ios": {
                    "UIBackgroundModes": ["bluetooth-central"]
                },
                "sdkConfigs": {
                    "geolocation": {
                        "baidu": {
                            "__platform__": ["ios"],
                            "appkey_ios": "local-baidu-key"
                        }
                    }
                }
            }
        }
    });
    std::fs::write(
        root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let mut config = crate::commands::project::ProjectConfig::default();
    config.local_path = root.to_string_lossy().to_string();

    let mut supplied = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    supplied.ios_privacy_descriptions.insert(
        "NSLocationWhenInUseUsageDescription".into(),
        "构建中心定位说明".into(),
    );
    supplied.manifest_value.as_mut().unwrap()["app-plus"]["distribute"]["sdkConfigs"]
        ["geolocation"]["baidu"]["appkey_ios"] =
        serde_json::Value::String("build-center-baidu-key".into());
    supplied.manifest_value.as_mut().unwrap()["app-plus"]["distribute"]["ios"]
        ["UIBackgroundModes"] = serde_json::Value::Array(Vec::new());

    let info = resolve_ios_manifest_info(&config, Some(&supplied))
        .unwrap()
        .unwrap();

    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSLocationWhenInUseUsageDescription")
            .map(String::as_str),
        Some("构建中心定位说明")
    );
    assert_eq!(
        info.manifest_value.as_ref().and_then(|manifest| {
            manifest
                .get("app-plus")?
                .get("distribute")?
                .get("sdkConfigs")?
                .get("geolocation")?
                .get("baidu")?
                .get("appkey_ios")?
                .as_str()
        }),
        Some("build-center-baidu-key")
    );
    assert_eq!(
        info.manifest_value.as_ref().and_then(|manifest| {
            manifest
                .get("app-plus")?
                .get("distribute")?
                .get("ios")?
                .get("UIBackgroundModes")?
                .as_array()
                .map(Vec::len)
        }),
        Some(0)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_build_rejects_manifest_and_resource_app_id_mismatch() {
    let manifest = serde_json::json!({ "appid": "__UNI__MANIFEST" });
    let root = std::env::temp_dir().join(format!("unipack-ios-appid-{}", uuid::Uuid::new_v4()));
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    let error = validate_ios_app_id("__UNI__RESOURCE", Some(&info)).unwrap_err();

    assert!(error.contains("__UNI__MANIFEST"));
    assert!(error.contains("__UNI__RESOURCE"));
}

#[test]
fn ios_manifest_basic_info_and_privacy_are_written_to_xcode_project() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-manifest-apply-{}",
        uuid::Uuid::new_v4()
    ));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    let plist_path = root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
    let localized_plist = root.join("HBuilder-Hello/en.lproj/InfoPlist.strings");
    std::fs::create_dir_all(localized_plist.parent().unwrap()).unwrap();
    std::fs::write(
        &localized_plist,
        "/* Localized */\nCFBundleDisplayName=\"HBuilder Hello\";\n",
    )
    .unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&plist_path)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"buildSettings = {
				CURRENT_PROJECT_VERSION = 1;
				DEVELOPMENT_TEAM = OLDTEAM;
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
				MARKETING_VERSION = 1.0.0;
				PRODUCT_BUNDLE_IDENTIFIER = io.dcloud.HBuilder;
			};
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "name": "Manifest App",
        "versionName": "3.4.5",
        "versionCode": 345,
        "app-plus": {
            "modules": {
                "Geolocation": {}
            },
            "distribute": {
                "ios": {
                    "urltypes": ["manifest-app"],
                    "urlschemewhitelist": "alipays,alipay,wechat",
                    "UIBackgroundModes": "audio",
                    "privacyDescription": {
                        "NSPhotoLibraryUsageDescription": "用于选择图片"
                    },
                    "capabilities": {
                        "entitlements": {
                            "com.apple.developer.associated-domains": [
                                "applinks:www.hubeijianmeishiye.cn"
                            ]
                        }
                    }
                },
                "sdkConfigs": {
                    "oauth": {
                        "google": {
                            "clientid": "google-client-id"
                        }
                    },
                    "share": {
                        "weixin": {
                            "appid": "wx-manifest",
                            "UniversalLinks": "https://example.com/app/"
                        }
                    },
                    "geolocation": {
                        "amap": {
                            "__platform__": ["ios", "android"],
                            "appkey_ios": "amap-ios-key"
                        },
                        "baidu": {
                            "__platform__": ["ios", "android"],
                            "appkey": "baidu-global-key"
                        }
                    },
                    "statics": {
                        "umeng": {
                            "appkey_ios": "umeng-ios-key",
                            "channelid_ios": "App Store"
                        }
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    let mut config = crate::commands::project::ProjectConfig::default();
    config.app.name = "Config App".into();
    config.app.version = "1.0.0".into();
    config.app.version_code = 1;
    config.ios.bundle_id = "com.example.manifest".into();
    config.ios.team_id = "TEAM123".into();
    config.ios.dcloud_app_key = "app-key".into();

    patch_pbxproj(&project_file, &config, Some(&info)).unwrap();
    patch_info_plist(
        &root,
        &project_file,
        &config,
        "__UNI__MANIFEST",
        Some(&info),
    )
    .unwrap();

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("MARKETING_VERSION = 3.4.5;"));
    assert!(pbxproj.contains("CURRENT_PROJECT_VERSION = 345;"));
    assert!(pbxproj.contains("INFOPLIST_KEY_CFBundleDisplayName = \"Manifest App\";"));
    let plist = plist_crate::Value::from_file(&plist_path).unwrap();
    let dict = plist.as_dictionary().unwrap();
    assert_eq!(
        dict.get("CFBundleDisplayName")
            .and_then(plist_crate::Value::as_string),
        Some("Manifest App")
    );
    assert_eq!(
        dict.get("CFBundleShortVersionString")
            .and_then(plist_crate::Value::as_string),
        Some("3.4.5")
    );
    assert_eq!(
        dict.get("CFBundleVersion")
            .and_then(plist_crate::Value::as_string),
        Some("345")
    );
    assert_eq!(
        dict.get("NSPhotoLibraryUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用于选择图片")
    );
    assert_eq!(
        dict.get("GIDClientID")
            .and_then(plist_crate::Value::as_string),
        Some("google-client-id")
    );
    assert_eq!(
        dict.get("AMapApiKey")
            .and_then(plist_crate::Value::as_string),
        Some("amap-ios-key")
    );
    assert_eq!(
        dict.get("BaiduMapApiKey")
            .and_then(plist_crate::Value::as_string),
        Some("baidu-global-key")
    );
    assert_eq!(
        dict.get("NSLocationWhenInUseUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用于获取当前位置以提供定位相关服务")
    );
    assert_eq!(
        dict.get("UMENG_APPKEY")
            .and_then(plist_crate::Value::as_string),
        Some("umeng-ios-key")
    );
    let url_schemes = dict
        .get("CFBundleURLTypes")
        .and_then(plist_crate::Value::as_array)
        .unwrap()
        .iter()
        .filter_map(plist_crate::Value::as_dictionary)
        .filter_map(|entry| entry.get("CFBundleURLSchemes"))
        .filter_map(plist_crate::Value::as_array)
        .flatten()
        .filter_map(plist_crate::Value::as_string)
        .collect::<Vec<_>>();
    assert!(url_schemes.contains(&"manifest-app"));
    assert!(url_schemes.contains(&"wx-manifest"));
    let query_schemes = dict
        .get("LSApplicationQueriesSchemes")
        .and_then(plist_crate::Value::as_array)
        .unwrap()
        .iter()
        .filter_map(plist_crate::Value::as_string)
        .collect::<Vec<_>>();
    assert!(query_schemes.contains(&"alipays"));
    assert!(query_schemes.contains(&"alipay"));
    assert!(query_schemes.contains(&"wechat"));
    let background_modes = dict
        .get("UIBackgroundModes")
        .and_then(plist_crate::Value::as_array)
        .unwrap()
        .iter()
        .filter_map(plist_crate::Value::as_string)
        .collect::<Vec<_>>();
    assert_eq!(background_modes, vec!["audio"]);
    assert!(std::fs::read_to_string(&localized_plist)
        .unwrap()
        .contains("CFBundleDisplayName=\"Manifest App\";"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_barcode_and_bluetooth_modules_patch_privacy_and_background_modes() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-barcode-bluetooth-{}",
        uuid::Uuid::new_v4()
    ));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    let plist_path = root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&plist_path)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXProject section */
		000000000000000000000001 /* Project object */ = {
			isa = PBXProject;
			attributes = {
				TargetAttributes = {
					000000000000000000000002 = {
					};
				};
			};
		};
/* End PBXProject section */
buildSettings = {
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
			};
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "appid": "__UNI__MODULES",
        "app-plus": {
            "modules": {
                "Barcode": {},
                "Bluetooth": {}
            },
            "distribute": {
                "ios": {
                    "UIBackgroundModes": ["audio", "bluetooth-central", "bluetooth-peripheral"],
                    "privacyDescription": {
                        "NSCameraUsageDescription": "用户配置扫码相机说明"
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    let mut config = crate::commands::project::ProjectConfig::default();
    config.ios.dcloud_app_key = "app-key".into();
    config.ios.bundle_id = "com.example.modules".into();

    patch_info_plist(&root, &project_file, &config, "__UNI__MODULES", Some(&info)).unwrap();
    apply_ios_bluetooth_module(&project_file, Some(&info)).unwrap();

    let plist = plist_crate::Value::from_file(&plist_path).unwrap();
    let dict = plist.as_dictionary().unwrap();
    assert_eq!(
        dict.get("NSCameraUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用户配置扫码相机说明")
    );
    assert_eq!(
        dict.get("NSPhotoLibraryUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用于从相册选择图片进行扫码")
    );
    assert_eq!(
        dict.get("NSBluetoothAlwaysUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用于连接和管理低功耗蓝牙设备")
    );
    let background_modes = dict
        .get("UIBackgroundModes")
        .and_then(plist_crate::Value::as_array)
        .unwrap()
        .iter()
        .filter_map(plist_crate::Value::as_string)
        .collect::<Vec<_>>();
    assert!(background_modes.contains(&"audio"));
    assert!(background_modes.contains(&"bluetooth-central"));
    assert!(background_modes.contains(&"bluetooth-peripheral"));

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("com.apple.BackgroundModes"));
    assert!(pbxproj.contains("enabled = 1;"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_extra_modules_patch_privacy_background_modes_and_ats() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-extra-modules-{}",
        uuid::Uuid::new_v4()
    ));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    let plist_path = root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&plist_path)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXProject section */
		000000000000000000000001 /* Project object */ = {
			isa = PBXProject;
			attributes = {
				TargetAttributes = {
					000000000000000000000002 = {
					};
				};
			};
		};
/* End PBXProject section */
buildSettings = {
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
			};
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "appid": "__UNI__EXTRA_MODULES",
        "app-plus": {
            "modules": {
                "Camera": {},
                "Contacts": {},
                "FaceID": {},
                "Fingerprint": {},
                "iBeacon": {},
                "LivePusher": {},
                "Record": {},
                "VideoPlayer": {}
            },
            "distribute": {
                "ios": {
                    "privacyDescription": {
                        "NSContactsUsageDescription": "用户配置通讯录说明",
                        "NSPhotoLibraryAddUsageDescription": "用户配置保存到相册说明"
                    },
                    "NSAppTransportSecurity": {
                        "NSAllowsArbitraryLoads": true
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    let mut config = crate::commands::project::ProjectConfig::default();
    config.ios.dcloud_app_key = "app-key".into();
    config.ios.bundle_id = "com.example.extra".into();
    config.ios_module_config.insert(
        "privacy.NSMicrophoneUsageDescription".into(),
        "构建中心共享麦克风说明".into(),
    );
    config.ios_module_config.insert(
        "privacy.NSFaceIDUsageDescription".into(),
        "构建中心共享 Face ID 说明".into(),
    );

    patch_info_plist(
        &root,
        &project_file,
        &config,
        "__UNI__EXTRA_MODULES",
        Some(&info),
    )
    .unwrap();
    apply_ios_ibeacon_module(&project_file, Some(&info)).unwrap();

    let plist = plist_crate::Value::from_file(&plist_path).unwrap();
    let dict = plist.as_dictionary().unwrap();
    assert_eq!(
        dict.get("NSCameraUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用于拍摄照片或视频")
    );
    assert_eq!(
        dict.get("NSPhotoLibraryUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用于读取和写入相册中的照片或视频")
    );
    assert_eq!(
        dict.get("NSPhotoLibraryAddUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用户配置保存到相册说明")
    );
    assert_eq!(
        dict.get("NSContactsUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用户配置通讯录说明")
    );
    assert_eq!(
        dict.get("NSFaceIDUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("构建中心共享 Face ID 说明")
    );
    assert_eq!(
        dict.get("NSLocationAlwaysAndWhenInUseUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("扫描蓝牙 Beacon 设备")
    );
    assert_eq!(
        dict.get("NSMicrophoneUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("构建中心共享麦克风说明")
    );
    let background_modes = dict
        .get("UIBackgroundModes")
        .and_then(plist_crate::Value::as_array)
        .unwrap()
        .iter()
        .filter_map(plist_crate::Value::as_string)
        .collect::<Vec<_>>();
    assert!(background_modes.contains(&"location"));
    assert!(background_modes.contains(&"bluetooth-central"));
    assert_eq!(
        dict.get("NSAppTransportSecurity")
            .and_then(plist_crate::Value::as_dictionary)
            .and_then(|ats| ats.get("NSAllowsArbitraryLoads"))
            .and_then(|value| match value {
                plist_crate::Value::Boolean(flag) => Some(*flag),
                _ => None,
            }),
        Some(true)
    );

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("com.apple.BackgroundModes"));
    assert!(pbxproj.contains("enabled = 1;"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_geolocation_ignores_sdk_config_until_module_is_enabled() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-location-gate-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "geolocation": {
                        "baidu": {
                            "__platform__": ["ios", "android"],
                            "appkey_ios": "baidu-ios-key"
                        }
                    }
                }
            }
        }
    });
    let disabled = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    assert!(ios_geolocation_providers(Some(&disabled)).is_none());
    assert_eq!(
        ios_geolocation_provider_value(&manifest, "baidu", &["appkey_ios", "appkey", "key"]),
        None
    );

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
                            "__platform__": ["ios", "android"],
                            "appkey_ios": "baidu-ios-key"
                        },
                        "tencent": {
                            "__platform__": ["ios", "android"],
                            "apikey_ios": "tencent-ios-key"
                        },
                        "amap": {
                            "name": "amap_2331r423",
                            "__platform__": ["android"],
                            "appkey_ios": "amap-ios-key"
                        }
                    }
                }
            }
        }
    });
    let enabled = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    assert_eq!(
        ios_geolocation_providers(Some(&enabled)).unwrap(),
        vec![
            IosGeolocationProvider::System,
            IosGeolocationProvider::Baidu,
        ]
    );
    assert_eq!(
        ios_geolocation_provider_value(&manifest, "baidu", &["appkey_ios", "appkey", "key"])
            .as_deref(),
        Some("baidu-ios-key")
    );
    assert_eq!(
        ios_geolocation_provider_value(&manifest, "amap", &["appkey_ios", "appkey", "key"]),
        None
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_geolocation_system_provider_links_xcode_dependencies_idempotently() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-location-pbx-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    std::fs::create_dir_all(root.join("SDK/Libs")).unwrap();
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::write(root.join("SDK/Libs/liblibGeolocation.a"), "lib").unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXGroup section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* Frameworks */ = {
			isa = PBXGroup;
			children = (
			);
			name = Frameworks;
			sourceTree = "<group>";
		};
/* End PBXGroup section */
"#,
    )
    .unwrap();
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
                        }
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    let integration = apply_ios_geolocation_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();
    assert_eq!(integration.providers, vec![IosGeolocationProvider::System]);
    assert_eq!(integration.linked_count, 3);

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("liblibGeolocation.a in Frameworks"));
    assert!(pbxproj.contains("Foundation.framework in Frameworks"));
    assert!(pbxproj.contains("CoreLocation.framework in Frameworks"));
    assert!(pbxproj.contains("../SDK/Libs/liblibGeolocation.a"));

    let integration = apply_ios_geolocation_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();
    assert_eq!(integration.linked_count, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(
        pbxproj.matches("liblibGeolocation.a in Frameworks").count(),
        2
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_livepusher_links_and_embeds_documented_dependencies_idempotently() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-livepusher-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let libs_dir = root.join("SDK/Libs");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&libs_dir).unwrap();
    std::fs::write(libs_dir.join("liblibLivePush.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libDCUniGPUImage.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libDCUniLivePush.a"), "lib").unwrap();
    std::fs::create_dir_all(libs_dir.join("UPLiveSDKDll.framework")).unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXGroup section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* Frameworks */ = {
			isa = PBXGroup;
			children = (
			);
			name = Frameworks;
			sourceTree = "<group>";
		};
/* End PBXGroup section */
/* Begin PBXNativeTarget section */
		CCCCCCCCCCCCCCCCCCCCCCCC /* HBuilder-Hello */ = {
			isa = PBXNativeTarget;
			buildPhases = (
				AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */,
			);
			name = "HBuilder-Hello";
		};
/* End PBXNativeTarget section */
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "appid": "__UNI__LIVEPUSHER",
        "app-plus": {
            "modules": {
                "LivePusher": {}
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    let integration = apply_ios_livepusher_module(
        &project_root,
        &project_file,
        Some(&info),
        &std::collections::HashMap::new(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(integration.linked_count, 16);
    assert_eq!(integration.embedded_count, 1);

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("liblibLivePush.a in Frameworks"));
    assert!(pbxproj.contains("libDCUniGPUImage.a in Frameworks"));
    assert!(!pbxproj.contains("libDCUniLivePush.a in Frameworks"));
    assert!(pbxproj.contains("UPLiveSDKDll.framework in Frameworks"));
    assert!(pbxproj.contains("UPLiveSDKDll.framework in Embed Frameworks"));
    assert!(pbxproj.contains("AVFoundation.framework in Frameworks"));
    assert!(pbxproj.contains("VideoToolbox.framework in Frameworks"));
    assert!(pbxproj.contains("CoreMedia.framework in Frameworks"));
    assert!(pbxproj.contains("libbz2.tbd in Frameworks"));
    assert!(pbxproj.contains("libiconv.tbd in Frameworks"));
    assert!(pbxproj.contains("CodeSignOnCopy"));
    assert!(pbxproj.contains("../SDK/Libs/UPLiveSDKDll.framework"));

    let mut custom_component_config = std::collections::HashMap::new();
    custom_component_config.insert(
        "livepusher.customComponentMode".to_string(),
        "true".to_string(),
    );
    let integration = apply_ios_livepusher_module(
        &project_root,
        &project_file,
        Some(&info),
        &custom_component_config,
    )
    .unwrap()
    .unwrap();
    assert_eq!(integration.linked_count, 1);
    assert_eq!(integration.embedded_count, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("libDCUniLivePush.a in Frameworks"));

    let integration = apply_ios_livepusher_module(
        &project_root,
        &project_file,
        Some(&info),
        &custom_component_config,
    )
    .unwrap()
    .unwrap();
    assert_eq!(integration.linked_count, 0);
    assert_eq!(integration.embedded_count, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(
        pbxproj
            .matches("UPLiveSDKDll.framework in Frameworks")
            .count(),
        2
    );
    assert_eq!(
        pbxproj
            .matches("UPLiveSDKDll.framework in Embed Frameworks")
            .count(),
        2
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_push_module_patches_plist_and_links_xcode_dependencies() {
    let root = std::env::temp_dir().join(format!("unipack-ios-push-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let plist_path = project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
    let feature_plist_path = project_root.join("HBuilder-Hello/PandoraApi.bundle/feature.plist");
    let libs_dir = root.join("SDK/Libs");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(feature_plist_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(libs_dir.join("GTSDK.xcframework")).unwrap();
    std::fs::write(libs_dir.join("liblibPush.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libUniPush.a"), "lib").unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&plist_path)
        .unwrap();
    let mut feature_plist = plist_crate::Dictionary::new();
    feature_plist.insert("Existing".into(), plist_crate::Value::String("keep".into()));
    plist_crate::Value::Dictionary(feature_plist)
        .to_file_xml(&feature_plist_path)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXGroup section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* Frameworks */ = {
			isa = PBXGroup;
			children = (
			);
			name = Frameworks;
			sourceTree = "<group>";
		};
/* End PBXGroup section */
/* Begin PBXProject section */
		CCCCCCCCCCCCCCCCCCCCCCCC /* Project object */ = {
			isa = PBXProject;
			attributes = {
				TargetAttributes = {
					DDDDDDDDDDDDDDDDDDDDDDDD = {
					};
				};
			};
		};
/* End PBXProject section */
buildSettings = {
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
			};
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "appid": "__UNI__PUSH",
        "app-plus": {
            "modules": {
                "Push": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "push": {
                        "unipush": {
                            "version": "2",
                            "appid": "getui-ios-appid",
                            "appkey": "getui-ios-appkey",
                            "appsecret": "getui-ios-appsecret"
                        }
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    let mut config = crate::commands::project::ProjectConfig::default();
    config.ios.dcloud_app_key = "app-key".into();
    config.ios.bundle_id = "com.example.push".into();

    patch_info_plist(
        &project_root,
        &project_file,
        &config,
        "__UNI__PUSH",
        Some(&info),
    )
    .unwrap();
    let integration = apply_ios_push_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();

    assert_eq!(integration.linked_count, 14);
    assert_eq!(integration.background_modes, vec!["remote-notification"]);
    let plist = plist_crate::Value::from_file(&plist_path).unwrap();
    let dict = plist.as_dictionary().unwrap();
    let getui = dict
        .get("getui")
        .and_then(plist_crate::Value::as_dictionary)
        .unwrap();
    assert_eq!(
        getui.get("appid").and_then(plist_crate::Value::as_string),
        Some("getui-ios-appid")
    );
    assert_eq!(
        getui.get("appkey").and_then(plist_crate::Value::as_string),
        Some("getui-ios-appkey")
    );
    assert_eq!(
        getui
            .get("appsecret")
            .and_then(plist_crate::Value::as_string),
        Some("getui-ios-appsecret")
    );
    let background_modes = dict
        .get("UIBackgroundModes")
        .and_then(plist_crate::Value::as_array)
        .unwrap()
        .iter()
        .filter_map(plist_crate::Value::as_string)
        .collect::<Vec<_>>();
    assert_eq!(background_modes, vec!["remote-notification"]);

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("liblibPush.a in Frameworks"));
    assert!(!pbxproj.contains("libGeTuiPush.a in Frameworks"));
    assert!(pbxproj.contains("libUniPush.a in Frameworks"));
    assert!(pbxproj.contains("GTSDK.xcframework in Frameworks"));
    assert!(pbxproj.contains("lastKnownFileType = wrapper.xcframework"));
    assert!(pbxproj.contains("libresolv.tbd in Frameworks"));
    assert!(pbxproj.contains("UserNotifications.framework in Frameworks"));
    assert!(pbxproj.contains("settings = {ATTRIBUTES = (Weak, ); };"));
    assert!(pbxproj.contains("com.apple.BackgroundModes"));
    assert!(pbxproj.contains("com.apple.Push"));
    let feature_plist = plist_crate::Value::from_file(&feature_plist_path).unwrap();
    let feature_dict = feature_plist.as_dictionary().unwrap();
    assert_eq!(
        feature_dict
            .get("Existing")
            .and_then(plist_crate::Value::as_string),
        Some("keep")
    );
    let push = feature_dict
        .get("Push")
        .and_then(plist_crate::Value::as_dictionary)
        .unwrap();
    assert_eq!(
        push.get("autostart")
            .and_then(plist_crate::Value::as_boolean),
        Some(true)
    );
    assert_eq!(
        push.get("baseclass")
            .and_then(plist_crate::Value::as_string),
        Some("PGPush")
    );
    assert_eq!(
        push.get("class").and_then(plist_crate::Value::as_string),
        Some("PGPushActualize")
    );
    assert_eq!(
        push.get("global").and_then(plist_crate::Value::as_boolean),
        Some(true)
    );
    let server = push
        .get("server")
        .and_then(plist_crate::Value::as_dictionary)
        .unwrap();
    assert_eq!(
        server.get("class").and_then(plist_crate::Value::as_string),
        Some("PGPushServerAct")
    );
    assert_eq!(
        server
            .get("identifier")
            .and_then(plist_crate::Value::as_string),
        Some("com.pushserver")
    );

    let integration = apply_ios_push_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();
    assert_eq!(integration.linked_count, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(
        pbxproj.matches("GTSDK.xcframework in Frameworks").count(),
        2
    );
    assert_eq!(
        pbxproj
            .matches("UserNotifications.framework in Frameworks")
            .count(),
        2
    );
    assert_eq!(pbxproj.matches("ATTRIBUTES = (Weak, );").count(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_facial_recognition_verify_module_patches_project_dependencies_and_resources() {
    let root = std::env::temp_dir().join(format!("unipack-ios-frv-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let plist_path = project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
    let libs_dir = root.join("SDK/Libs");
    let resources_dir = root.join("SDK/Bundles");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&libs_dir).unwrap();
    std::fs::create_dir_all(&resources_dir).unwrap();
    for framework in [
        "DCUniBase.framework",
        "DCloudUTSFoundation.framework",
        "uniFacialRecognitionVerify.framework",
        "AliyunFaceAuthFacade.framework",
        "AliyunMobileRPC.framework",
        "AliyunOSSiOS.framework",
        "APBToygerFacade.framework",
        "APPSecuritySDK.framework",
        "BioAuthAPI.framework",
        "BioAuthEngine.framework",
        "deviceiOS.framework",
        "DTFIdentityManager.framework",
        "DTFSensorServices.framework",
        "DTFUIModule.framework",
        "DTFUtility.framework",
        "MPRemoteLogging.framework",
        "ToygerNative.framework",
        "ToygerService.framework",
    ] {
        std::fs::create_dir_all(libs_dir.join(framework)).unwrap();
    }
    for bundle in [
        "APBToygerFacade.bundle",
        "BioAuthEngine.bundle",
        "ToygerNative.bundle",
    ] {
        std::fs::create_dir_all(resources_dir.join(bundle)).unwrap();
        std::fs::write(resources_dir.join(bundle).join("marker.txt"), "resource").unwrap();
    }
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&plist_path)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
		000000000000000000000001 /* liblibPDRCore.a in Frameworks */ = {isa = PBXBuildFile; fileRef = 000000000000000000000002 /* liblibPDRCore.a */; };
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
		000000000000000000000002 /* liblibPDRCore.a */ = {isa = PBXFileReference; lastKnownFileType = archive.ar; name = liblibPDRCore.a; path = ../SDK/Libs/liblibPDRCore.a; sourceTree = "<group>"; };
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
				000000000000000000000001 /* liblibPDRCore.a in Frameworks */,
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXGroup section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* Frameworks */ = {
			isa = PBXGroup;
			children = (
				000000000000000000000002 /* liblibPDRCore.a */,
			);
			name = Frameworks;
			sourceTree = "<group>";
		};
		CCCCCCCCCCCCCCCCCCCCCCCC /* Supporting Files */ = {
			isa = PBXGroup;
			children = (
			);
			name = "Supporting Files";
			sourceTree = "<group>";
		};
/* End PBXGroup section */
/* Begin PBXNativeTarget section */
		DDDDDDDDDDDDDDDDDDDDDDDD /* HBuilder-Hello */ = {
			isa = PBXNativeTarget;
			buildPhases = (
				AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */,
				EEEEEEEEEEEEEEEEEEEEEEEE /* Resources */,
			);
			name = "HBuilder-Hello";
		};
/* End PBXNativeTarget section */
/* Begin PBXResourcesBuildPhase section */
		EEEEEEEEEEEEEEEEEEEEEEEE /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			files = (
			);
		};
/* End PBXResourcesBuildPhase section */
buildSettings = {
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
			};
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "appid": "__UNI__FRV",
        "app-plus": {
            "modules": {
                "FacialRecognitionVerify": {}
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    let mut config = crate::commands::project::ProjectConfig::default();
    config.ios.dcloud_app_key = "app-key".into();
    config.ios.bundle_id = "com.example.frv".into();

    patch_info_plist(
        &project_root,
        &project_file,
        &config,
        "__UNI__FRV",
        Some(&info),
    )
    .unwrap();
    let integration =
        apply_ios_facial_recognition_verify_module(&project_root, &project_file, Some(&info))
            .unwrap()
            .unwrap();

    assert_eq!(integration.linked_count, 41);
    assert_eq!(integration.embedded_count, 2);
    assert_eq!(integration.resource_count, 3);
    assert_eq!(integration.removed_duplicate_count, 1);
    let plist = plist_crate::Value::from_file(&plist_path).unwrap();
    let dict = plist.as_dictionary().unwrap();
    assert_eq!(
        dict.get("NSCameraUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("我们需要使用摄像头进行人脸识别验证")
    );
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("DCUniBase.framework in Frameworks"));
    assert!(pbxproj.contains("DCUniBase.framework in Embed Frameworks"));
    assert!(pbxproj.contains("DCloudUTSFoundation.framework in Embed Frameworks"));
    assert!(pbxproj.contains("CodeSignOnCopy"));
    assert!(pbxproj.contains("uniFacialRecognitionVerify.framework in Frameworks"));
    assert!(pbxproj.contains("AVFoundation.framework in Frameworks"));
    assert!(pbxproj.contains("libc++abi.tbd in Frameworks"));
    assert!(pbxproj.contains("APBToygerFacade.bundle in Resources"));
    assert!(pbxproj.contains("lastKnownFileType = \"wrapper.plug-in\""));
    assert!(!pbxproj.contains("liblibPDRCore.a in Frameworks"));
    assert!(project_root
        .join("HBuilder-Hello/APBToygerFacade.bundle/marker.txt")
        .is_file());

    let integration =
        apply_ios_facial_recognition_verify_module(&project_root, &project_file, Some(&info))
            .unwrap()
            .unwrap();
    assert_eq!(integration.linked_count, 0);
    assert_eq!(integration.embedded_count, 0);
    assert_eq!(integration.removed_duplicate_count, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(
        pbxproj.matches("DCUniBase.framework in Frameworks").count(),
        2
    );
    assert_eq!(
        pbxproj
            .matches("DCUniBase.framework in Embed Frameworks")
            .count(),
        2
    );
    assert_eq!(
        pbxproj
            .matches("APBToygerFacade.bundle in Resources")
            .count(),
        2
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_push_module_skips_when_unipush_version_is_not_v2() {
    let root = std::env::temp_dir().join(format!("unipack-ios-push-skip-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let plist_path = project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
    let feature_plist_path = project_root.join("HBuilder-Hello/PandoraApi.bundle/feature.plist");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(feature_plist_path.parent().unwrap()).unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&plist_path)
        .unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&feature_plist_path)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXGroup section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* Frameworks */ = {
			isa = PBXGroup;
			children = (
			);
			name = Frameworks;
			sourceTree = "<group>";
		};
/* End PBXGroup section */
/* Begin PBXProject section */
		CCCCCCCCCCCCCCCCCCCCCCCC /* Project object */ = {
			isa = PBXProject;
			attributes = {
				TargetAttributes = {
					DDDDDDDDDDDDDDDDDDDDDDDD = {
					};
				};
			};
		};
/* End PBXProject section */
buildSettings = {
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
			};
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "appid": "__UNI__PUSH",
        "app-plus": {
            "modules": {
                "Push": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "push": {
                        "unipush": {
                            "version": "1",
                            "appid": "getui-ios-appid",
                            "appkey": "getui-ios-appkey",
                            "appsecret": "getui-ios-appsecret"
                        }
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    let mut config = crate::commands::project::ProjectConfig::default();
    config.ios.dcloud_app_key = "app-key".into();
    config.ios.bundle_id = "com.example.push".into();

    patch_info_plist(
        &project_root,
        &project_file,
        &config,
        "__UNI__PUSH",
        Some(&info),
    )
    .unwrap();
    let integration = apply_ios_push_module(&project_root, &project_file, Some(&info)).unwrap();

    assert!(integration.is_none());
    let plist = plist_crate::Value::from_file(&plist_path).unwrap();
    let dict = plist.as_dictionary().unwrap();
    assert!(dict.get("getui").is_none());
    assert!(dict.get("UIBackgroundModes").is_none());
    let feature_plist = plist_crate::Value::from_file(&feature_plist_path).unwrap();
    assert!(feature_plist.as_dictionary().unwrap().get("Push").is_none());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pbx_optional_framework_marks_existing_link_as_weak() {
    let root = std::env::temp_dir().join(format!("unipack-ios-weak-{}", uuid::Uuid::new_v4()));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* UserNotifications.framework in Frameworks */ = {isa = PBXBuildFile; fileRef = BBBBBBBBBBBBBBBBBBBBBBBB /* UserNotifications.framework */; };
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* UserNotifications.framework */ = {isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = UserNotifications.framework; path = System/Library/Frameworks/UserNotifications.framework; sourceTree = SDKROOT; };
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		CCCCCCCCCCCCCCCCCCCCCCCC /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
				AAAAAAAAAAAAAAAAAAAAAAAA /* UserNotifications.framework in Frameworks */,
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXGroup section */
		DDDDDDDDDDDDDDDDDDDDDDDD /* Frameworks */ = {
			isa = PBXGroup;
			children = (
				BBBBBBBBBBBBBBBBBBBBBBBB /* UserNotifications.framework */,
			);
			name = Frameworks;
			sourceTree = "<group>";
		};
/* End PBXGroup section */
"#,
    )
    .unwrap();

    let linked = register_pbx_linked_files(
        &project_file,
        &[IosPbxLinkedFile::optional_system_framework(
            "UserNotifications.framework",
        )],
    )
    .unwrap();

    assert_eq!(linked, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("settings = {ATTRIBUTES = (Weak, ); };"));
    assert_eq!(
        pbxproj
            .matches("UserNotifications.framework in Frameworks")
            .count(),
        2
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pbx_embedded_frameworks_create_copy_phase_and_codesign() {
    let root = std::env::temp_dir().join(format!("unipack-ios-embed-{}", uuid::Uuid::new_v4()));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXGroup section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* Frameworks */ = {
			isa = PBXGroup;
			children = (
			);
			name = Frameworks;
			sourceTree = "<group>";
		};
/* End PBXGroup section */
/* Begin PBXNativeTarget section */
		CCCCCCCCCCCCCCCCCCCCCCCC /* HBuilder-Hello */ = {
			isa = PBXNativeTarget;
			buildPhases = (
				AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */,
			);
			name = "HBuilder-Hello";
		};
/* End PBXNativeTarget section */
"#,
    )
    .unwrap();

    let embedded = register_pbx_embedded_frameworks(
        &project_file,
        &[IosPbxLinkedFile::local_framework("DCUniBase.framework")],
    )
    .unwrap();

    assert_eq!(embedded, 1);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("PBXCopyFilesBuildPhase"));
    assert!(pbxproj.contains("DCUniBase.framework in Embed Frameworks"));
    assert!(pbxproj.contains("CodeSignOnCopy"));
    assert!(pbxproj.contains("../SDK/Libs/DCUniBase.framework"));

    let embedded = register_pbx_embedded_frameworks(
        &project_file,
        &[IosPbxLinkedFile::local_framework("DCUniBase.framework")],
    )
    .unwrap();
    assert_eq!(embedded, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(
        pbxproj
            .matches("DCUniBase.framework in Embed Frameworks")
            .count(),
        2
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn info_plist_strings_value_is_replaced_and_escaped() {
    let content = "/* Localized */\nCFBundleDisplayName=\"HBuilder Hello\";\nOther=\"keep\";\n";

    let updated =
        set_info_plist_string_value(content, "CFBundleDisplayName", "My \"$App\" \\ Name");

    assert!(updated.contains("CFBundleDisplayName=\"My \\\"$App\\\" \\\\ Name\";"));
    assert!(updated.contains("Other=\"keep\";"));
}

#[test]
fn info_plist_strings_supports_quoted_keys_and_missing_keys() {
    let content =
        "/* Localized */\n\"CFBundleDisplayName\" = \"HBuilder Hello\";\nOther=\"keep\";\n";

    let updated = set_info_plist_string_value(content, "CFBundleDisplayName", "Manifest App");
    let appended = set_info_plist_string_value("Other=\"keep\";", "CFBundleDisplayName", "App");

    assert!(updated.contains("\"CFBundleDisplayName\" = \"Manifest App\";"));
    assert!(updated.contains("Other=\"keep\";"));
    assert!(appended.contains("Other=\"keep\";\n\"CFBundleDisplayName\" = \"App\";"));
}

#[test]
fn ios_privacy_descriptions_overlay_manifest_values_on_sdk_defaults() {
    let mut dict = plist_crate::Dictionary::new();
    dict.insert(
        "NSCameraUsageDescription".into(),
        plist_crate::Value::String("模板相机说明".into()),
    );
    dict.insert(
        "NSMicrophoneUsageDescription".into(),
        plist_crate::Value::String("模板麦克风说明".into()),
    );
    dict.insert(
        "NSLocationWhenInUseUsageDescription".into(),
        plist_crate::Value::String("".into()),
    );
    dict.insert(
        "NSLocationWhenInUseUsageDescription - 2".into(),
        plist_crate::Value::String("用户使用时期定位".into()),
    );
    dict.insert(
        "NSLocationWhenInUseDescription".into(),
        plist_crate::Value::String("".into()),
    );
    let descriptions = std::collections::BTreeMap::from([
        (
            "NSPhotoLibraryUsageDescription".to_string(),
            "在上传头像或发布内容时，开启相册权限便于您保存图片或选择图片上传".to_string(),
        ),
        (
            "NSPhotoLibraryAddUsageDescription".to_string(),
            "该应用需要读取您的相册，以便您使用应用生成海报时保存到相册".to_string(),
        ),
        (
            "NSCameraUsageDescription".to_string(),
            "在上传头像或发布内容时，开启相机权限便于您拍照上传图片".to_string(),
        ),
        (
            "NSLocalNetworkUsageDescription".to_string(),
            "请允许访问本地网络，以便更好的体验应用".to_string(),
        ),
        (
            "NSContactsUsageDescription".to_string(),
            "用于拨通客服热线".to_string(),
        ),
    ]);

    apply_ios_privacy_descriptions(&mut dict, &descriptions);

    assert_eq!(
        dict.get("NSCameraUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("在上传头像或发布内容时，开启相机权限便于您拍照上传图片")
    );
    assert_eq!(
        dict.get("NSPhotoLibraryAddUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("该应用需要读取您的相册，以便您使用应用生成海报时保存到相册")
    );
    assert_eq!(
        dict.get("NSMicrophoneUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("模板麦克风说明")
    );
    assert_eq!(
        dict.get("NSLocationWhenInUseUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用户使用时期定位")
    );
    assert_eq!(
        dict.get("NSLocalNetworkUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("请允许访问本地网络，以便更好的体验应用")
    );
    assert_eq!(
        dict.get("NSContactsUsageDescription")
            .and_then(plist_crate::Value::as_string),
        Some("用于拨通客服热线")
    );
    assert!(!dict.contains_key("NSLocationWhenInUseUsageDescription - 2"));
    assert!(!dict.contains_key("NSLocationWhenInUseDescription"));
}

#[test]
fn ios_manifest_universal_links_are_written_to_entitlements() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-entitlements-{}", uuid::Uuid::new_v4()));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    let entitlements = root.join("HBuilder/HBuilder.entitlements");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(entitlements.parent().unwrap()).unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&entitlements)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        "CODE_SIGN_ENTITLEMENTS = HBuilder/HBuilder.entitlements;",
    )
    .unwrap();
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "ios": {
                    "capabilities": {
                        "entitlements": {
                            "com.apple.developer.associated-domains": [
                                "applinks:www.hubeijianmeishiye.cn"
                            ]
                        }
                    }
                },
                "sdkConfigs": {
                    "share": {
                        "weixin": {
                            "UniversalLinks": "https://example.com/apple-app-site-association/"
                        }
                    },
                    "payment": {
                        "weixin": {
                            "UniversalLinks": "https://example.com/pay/"
                        }
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    assert_eq!(
        patch_ios_entitlements(&root, &project_file, Some(&info)).unwrap(),
        2
    );
    let value = plist_crate::Value::from_file(&entitlements).unwrap();
    let domains = value
        .as_dictionary()
        .unwrap()
        .get("com.apple.developer.associated-domains")
        .and_then(plist_crate::Value::as_array)
        .unwrap();
    assert!(domains.contains(&plist_crate::Value::String(
        "applinks:www.hubeijianmeishiye.cn".into()
    )));
    assert!(domains.contains(&plist_crate::Value::String("applinks:example.com".into())));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_storyboard_zip_replaces_launch_screen_and_registers_resources() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-storyboard-{}", uuid::Uuid::new_v4()));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    let source_dir = root.join("HBuilder-Hello");
    let launch_screen = source_dir.join("LaunchScreen.storyboard");
    let zip_path = root.join("storyboard.zip");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(&launch_screen, "old storyboard").unwrap();
    std::fs::write(source_dir.join("HBuilder-Hello-Info.plist"), "<plist/>").unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
		AAA /* Supporting Files */ = {
			isa = PBXGroup;
			children = (
			);
		};
		BBB /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			files = (
			);
		};
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
"#,
    )
    .unwrap();
    let zip_file = std::fs::File::create(&zip_path).unwrap();
    let mut writer = zip::ZipWriter::new(zip_file);
    let options = zip::write::SimpleFileOptions::default();
    writer
        .start_file("LaunchScreen.storyboard", options)
        .unwrap();
    std::io::Write::write_all(&mut writer, b"new storyboard").unwrap();
    writer
        .start_file("images/background@2x.png", options)
        .unwrap();
    std::io::Write::write_all(&mut writer, b"image").unwrap();
    writer.finish().unwrap();
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "splashscreen": {
                    "iosStyle": "storyboard",
                    "ios": {
                        "storyboard": zip_path.to_string_lossy()
                    }
                }
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    assert_eq!(
        apply_ios_splashscreen(&root, &project_file, Some(&info)).unwrap(),
        Some(1)
    );
    assert_eq!(
        std::fs::read_to_string(&launch_screen).unwrap(),
        "new storyboard"
    );
    assert!(source_dir.join("background@2x.png").is_file());
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("background@2x.png in Resources"));
    assert!(pbxproj.contains("lastKnownFileType = image.png"));

    apply_ios_splashscreen(&root, &project_file, Some(&info)).unwrap();
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(pbxproj.matches("background@2x.png in Resources").count(), 2);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn configured_ios_sdk_project_accepts_local_manifest_when_requested() {
    let Ok(sdk_project) = std::env::var("UNIPACK_TEST_IOS_SDK_PROJECT") else {
        return;
    };
    let Ok(local_project) = std::env::var("UNIPACK_TEST_UNIAPP_PROJECT") else {
        return;
    };
    let root =
        std::env::temp_dir().join(format!("unipack-ios-real-config-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    crate::utils::fs::copy_recursive(Path::new(&sdk_project), &project_root).unwrap();
    let project_file = find_xcodeproj(&project_root).unwrap();
    let mut config = crate::commands::project::ProjectConfig::default();
    config.local_path = local_project;
    config.ios.bundle_id = "com.example.unipack.verify".into();
    config.ios.team_id = "TEAM123".into();
    config.ios.dcloud_app_key = "verify-app-key".into();
    let info = resolve_ios_manifest_info(&config, None).unwrap().unwrap();

    patch_pbxproj(&project_file, &config, Some(&info)).unwrap();
    apply_ios_splashscreen(&project_root, &project_file, Some(&info)).unwrap();
    patch_info_plist(
        &project_root,
        &project_file,
        &config,
        info.app_id.as_deref().unwrap_or("__UNI__VERIFY"),
        Some(&info),
    )
    .unwrap();
    assert_eq!(
        patch_ios_entitlements(&project_root, &project_file, Some(&info)).unwrap(),
        1
    );
    generate_app_icons(&project_root, &config, Some(&info)).unwrap();

    let output = std::process::Command::new("xcodebuild")
        .args(["-list", "-project"])
        .arg(&project_file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("dc_launchscreen_portrait_background@2x.png in Resources"));
    assert!(project_root
        .join("HBuilder-Hello/dc_launchscreen_portrait_background@2x.png")
        .is_file());
    let plist = plist_crate::Value::from_file(
        project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist"),
    )
    .unwrap();
    let plist = plist.as_dictionary().unwrap();
    assert_eq!(
        plist
            .get("CFBundleDisplayName")
            .and_then(plist_crate::Value::as_string),
        Some("ccc222")
    );
    assert_eq!(
        plist
            .get("AMapApiKey")
            .and_then(plist_crate::Value::as_string),
        Some("e58f1b2f4c1e3d8a9b7c6d5e4f3a2b1c")
    );
    let entitlements =
        plist_crate::Value::from_file(project_root.join("HBuilder/HBuilder.entitlements")).unwrap();
    assert!(entitlements
        .as_dictionary()
        .unwrap()
        .contains_key("com.apple.developer.associated-domains"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pbx_setting_replaces_existing_value() {
    let content = "\t\t\t\tPRODUCT_BUNDLE_IDENTIFIER = io.dcloud.HBuilder;\n";
    let updated = set_pbx_build_setting(content, "PRODUCT_BUNDLE_IDENTIFIER", "com.example.app");
    assert!(updated.contains("PRODUCT_BUNDLE_IDENTIFIER = com.example.app;"));
}

#[test]
fn pbx_setting_inserts_into_build_settings_block() {
    let content = "buildSettings = {\n\tOTHER = value;\n};\n";
    let updated = set_pbx_build_setting(content, "DEVELOPMENT_TEAM", "TEAM123");
    assert!(updated.contains("DEVELOPMENT_TEAM = TEAM123;"));
}

#[test]
fn pbx_linker_flag_preserves_existing_flags_and_is_idempotent() {
    let content = "\t\t\t\tOTHER_LDFLAGS = \"-ObjC\";\n";
    let updated = append_pbx_build_setting_flag(content, "OTHER_LDFLAGS", "-ld_classic");
    let updated_again = append_pbx_build_setting_flag(&updated, "OTHER_LDFLAGS", "-ld_classic");

    assert!(updated.contains("OTHER_LDFLAGS = \"-ObjC -ld_classic\";"));
    assert_eq!(updated, updated_again);
}

#[test]
fn pbx_conditional_simulator_arch_is_quoted() {
    let content = "buildSettings = {\n\tOTHER = value;\n};\n";
    let updated = set_pbx_build_setting(content, "\"ARCHS[sdk=iphonesimulator*]\"", "x86_64");

    assert!(updated.contains("\"ARCHS[sdk=iphonesimulator*]\" = x86_64;"));
}

#[test]
fn legacy_framework_requires_x86_64_simulator_compatibility() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-legacy-sim-{}", uuid::Uuid::new_v4()));
    let project = root.join("HBuilder-Hello/HBuilder-Hello.xcodeproj");
    let framework = root.join("SDK/Libs/DCUniRecord.framework");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(&framework).unwrap();
    std::fs::write(framework.join("DCUniRecord"), "legacy").unwrap();

    assert!(legacy_simulator_x86_64_required(&project));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn scheme_name_uses_native_target_instead_of_project_file_name() {
    let root = std::env::temp_dir().join(format!("unipack-ios-scheme-{}", uuid::Uuid::new_v4()));
    let project = root.join("HBuilder-Hello.xcodeproj");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join("project.pbxproj"),
        r#"
				isa = PBXNativeTarget;
				buildConfigurationList = ABC;
				name = HBuilder;
				productName = "HBuilder-Hello";
			};
"#,
    )
    .unwrap();

    assert_eq!(find_scheme_name(&project).as_deref(), Some("HBuilder"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn runtime_layout_supports_nested_hbuilder_source_directory() {
    let root = std::env::temp_dir().join(format!("unipack-ios-layout-{}", uuid::Uuid::new_v4()));
    let source = root.join("HBuilder-Hello");
    let resource = root.join("resource");
    std::fs::create_dir_all(source.join("Pandora/apps")).unwrap();
    std::fs::create_dir_all(&resource).unwrap();
    std::fs::write(
        source.join("control.xml"),
        r#"<HBuilder><apps><app appid="__UNI__OLD"/></apps></HBuilder>"#,
    )
    .unwrap();
    std::fs::write(resource.join("manifest.json"), "{}").unwrap();

    let layout = resolve_ios_runtime_layout(&root).unwrap();
    import_app_resource(&layout.apps_dir, &resource, "__UNI__NEW").unwrap();
    patch_control_xml(&layout.control_xml, "__UNI__NEW").unwrap();

    assert_eq!(layout.control_xml, source.join("control.xml"));
    assert_eq!(layout.apps_dir, source.join("Pandora/apps"));
    assert!(source
        .join("Pandora/apps/__UNI__NEW/manifest.json")
        .is_file());
    assert!(!root.join("Pandora").exists());
    assert!(std::fs::read_to_string(source.join("control.xml"))
        .unwrap()
        .contains(r#"appid="__UNI__NEW""#));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn runtime_layout_supports_control_inside_pandora() {
    let root = std::env::temp_dir().join(format!("unipack-ios-layout-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("Pandora/apps")).unwrap();
    std::fs::write(root.join("Pandora/control.xml"), "<HBuilder />").unwrap();

    let layout = resolve_ios_runtime_layout(&root).unwrap();

    assert_eq!(layout.control_xml, root.join("Pandora/control.xml"));
    assert_eq!(layout.apps_dir, root.join("Pandora/apps"));
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn workspace_links_sibling_sdk_support_directory() {
    let root = std::env::temp_dir().join(format!("unipack-ios-support-{}", uuid::Uuid::new_v4()));
    let sdk_project = root.join("package/HBuilder-Hello");
    let support = root.join("package/SDK");
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&sdk_project).unwrap();
    std::fs::create_dir_all(&support).unwrap();
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(support.join("PrivacyInfo.xcprivacy"), "privacy").unwrap();

    let linked = link_ios_sdk_support(&sdk_project, &workspace)
        .unwrap()
        .unwrap();

    assert_eq!(linked, workspace.join("SDK"));
    assert!(linked.join("PrivacyInfo.xcprivacy").is_file());
    assert!(std::fs::symlink_metadata(&linked)
        .unwrap()
        .file_type()
        .is_symlink());
    std::fs::remove_dir_all(&workspace).unwrap();
    assert!(support.join("PrivacyInfo.xcprivacy").is_file());
    let _ = std::fs::remove_dir_all(root);
}
