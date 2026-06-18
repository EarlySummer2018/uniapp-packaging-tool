use ::plist as plist_crate;

use crate::commands::ios::modules::statistic::{apply_ios_statistic_module, IosStatisticProvider};

#[test]
fn ios_statistic_ignores_local_pod_and_requires_offline_sdk_files() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-statistic-local-pod-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Statistic": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "statistic": {
                        "localPod": true,
                        "firebase": {}
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

    let error = apply_ios_statistic_module(&project_root, &project_file, Some(&info)).unwrap_err();

    assert!(error.contains("iOS 统计模块缺少 SDK 依赖文件"));
    assert!(error.contains("liblibStatistic.a"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_statistic_manual_integration_updates_feature_plist() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-statistic-feature-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let libs_dir = root.join("SDK/Libs");
    let feature_plist_path = project_root.join("HBuilder-Hello/PandoraApi.bundle/feature.plist");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&libs_dir).unwrap();
    std::fs::create_dir_all(feature_plist_path.parent().unwrap()).unwrap();
    std::fs::write(libs_dir.join("liblibStatistic.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libUmengStatistic.a"), "lib").unwrap();
    std::fs::create_dir_all(libs_dir.join("UMDevice.xcframework")).unwrap();
    std::fs::create_dir_all(libs_dir.join("UMCommon.xcframework")).unwrap();
    std::fs::create_dir_all(libs_dir.join("UMAPM.framework")).unwrap();
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
"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Statistic": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "statistic": {
                        "umeng": {
                            "__platform__": ["ios"],
                            "appkey_ios": "umeng-ios-key"
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

    let integration = apply_ios_statistic_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();

    assert_eq!(integration.providers, vec![IosStatisticProvider::Umeng]);
    assert!(integration.linked_count > 0);
    let feature_plist = plist_crate::Value::from_file(&feature_plist_path).unwrap();
    let statistic = feature_plist
        .as_dictionary()
        .and_then(|dict| dict.get("Statistic"))
        .and_then(plist_crate::Value::as_dictionary)
        .unwrap();
    let server = statistic
        .get("server")
        .and_then(plist_crate::Value::as_dictionary)
        .unwrap();
    assert_eq!(
        statistic
            .get("class")
            .and_then(plist_crate::Value::as_string),
        Some("UmengStatistic")
    );
    assert_eq!(
        server
            .get("identifier")
            .and_then(plist_crate::Value::as_string),
        Some("com.umeng.startup")
    );
    let _ = std::fs::remove_dir_all(root);
}
