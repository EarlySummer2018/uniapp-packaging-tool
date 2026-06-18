use ::plist as plist_crate;

use super::super::super::plist::patch_info_plist;
use crate::commands::ios::modules::push::apply_ios_push_module;

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
