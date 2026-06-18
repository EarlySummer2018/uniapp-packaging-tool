use super::super::super::plist::ios_geolocation_provider_value;
use crate::commands::ios::modules::geolocation::{
    apply_ios_geolocation_module, ios_geolocation_providers, IosGeolocationProvider,
};

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
