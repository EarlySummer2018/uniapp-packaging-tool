use ::plist as plist_crate;

use super::super::super::plist::patch_info_plist;
use crate::commands::ios::modules::bluetooth::apply_ios_bluetooth_module;
use crate::commands::ios::modules::ibeacon::apply_ios_ibeacon_module;

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
