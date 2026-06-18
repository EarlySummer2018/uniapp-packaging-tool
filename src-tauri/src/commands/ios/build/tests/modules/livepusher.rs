use crate::commands::ios::modules::livepusher::apply_ios_livepusher_module;

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
