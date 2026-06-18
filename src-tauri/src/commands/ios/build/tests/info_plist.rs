use ::plist as plist_crate;

use super::super::pbxproj::patch_pbxproj;
use super::super::plist::{
    apply_ios_privacy_descriptions, patch_info_plist, set_info_plist_string_value,
};

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
