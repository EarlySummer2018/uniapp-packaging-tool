use ::plist as plist_crate;

use super::super::super::plist::patch_info_plist;
use crate::commands::ios::modules::facial_recognition_verify::apply_ios_facial_recognition_verify_module;

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
