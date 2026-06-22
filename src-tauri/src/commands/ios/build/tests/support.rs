use std::path::{Path, PathBuf};

use ::plist as plist_crate;

pub(super) fn ios_payment_alipay_manifest(
    root: &Path,
) -> crate::commands::resource::UniappManifestInfo {
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Payment": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "alipay": {
                            "__platform__": ["ios"],
                            "appid": "ali-ios"
                        }
                    }
                }
            }
        }
    });
    crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        root,
        None,
    )
}

pub(super) fn prepare_ios_payment_alipay_project(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let libs_dir = root.join("SDK/Libs");
    let bundles_dir = root.join("SDK/Bundles");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&libs_dir).unwrap();
    std::fs::create_dir_all(bundles_dir.join("AlipaySDK.bundle")).unwrap();
    std::fs::write(libs_dir.join("liblibPayment.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libalixpayment.a"), "lib").unwrap();
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
		CCCCCCCCCCCCCCCCCCCCCCCC /* Supporting Files */ = {
			isa = PBXGroup;
			children = (
			);
			name = "Supporting Files";
			sourceTree = "<group>";
		};
/* End PBXGroup section */
/* Begin PBXResourcesBuildPhase section */
		DDDDDDDDDDDDDDDDDDDDDDDD /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			files = (
			);
		};
/* End PBXResourcesBuildPhase section */
/* Begin XCBuildConfiguration section */
		FFFFFFFFFFFFFFFFFFFFFFFF /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				IPHONEOS_DEPLOYMENT_TARGET = 12.0;
			};
			name = Release;
		};
/* End XCBuildConfiguration section */
"#,
    )
    .unwrap();
    (project_root, project_file, libs_dir)
}

pub(super) fn prepare_ios_uts_project(root: &Path) -> (PathBuf, PathBuf) {
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let libs_dir = root.join("SDK/Libs");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&libs_dir).unwrap();
    for framework in ["DCUniBase.framework", "DCloudUTSFoundation.framework"] {
        std::fs::create_dir_all(libs_dir.join(framework)).unwrap();
    }
    let plist_path = project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist");
    std::fs::create_dir_all(plist_path.parent().unwrap()).unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&plist_path)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
		111111111111111111111111 /* liblibPDRCore.a in Frameworks */ = {isa = PBXBuildFile; fileRef = 222222222222222222222222 /* liblibPDRCore.a */; };
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
		222222222222222222222222 /* liblibPDRCore.a */ = {isa = PBXFileReference; lastKnownFileType = archive.ar; name = liblibPDRCore.a; path = ../SDK/Libs/liblibPDRCore.a; sourceTree = "<group>"; };
/* End PBXFileReference section */
/* Begin PBXFrameworksBuildPhase section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			files = (
				111111111111111111111111 /* liblibPDRCore.a in Frameworks */,
			);
		};
/* End PBXFrameworksBuildPhase section */
/* Begin PBXSourcesBuildPhase section */
		999999999999999999999999 /* Sources */ = {
			isa = PBXSourcesBuildPhase;
			files = (
			);
		};
/* End PBXSourcesBuildPhase section */
/* Begin PBXGroup section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* Frameworks */ = {
			isa = PBXGroup;
			children = (
				222222222222222222222222 /* liblibPDRCore.a */,
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
/* Begin PBXResourcesBuildPhase section */
		DDDDDDDDDDDDDDDDDDDDDDDD /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			files = (
			);
		};
/* End PBXResourcesBuildPhase section */
/* Begin PBXNativeTarget section */
		EEEEEEEEEEEEEEEEEEEEEEEE /* HBuilder-Hello */ = {
			isa = PBXNativeTarget;
			buildPhases = (
				999999999999999999999999 /* Sources */,
				AAAAAAAAAAAAAAAAAAAAAAAA /* Frameworks */,
				DDDDDDDDDDDDDDDDDDDDDDDD /* Resources */,
			);
			name = "HBuilder-Hello";
		};
/* End PBXNativeTarget section */
/* Begin XCBuildConfiguration section */
		FFFFFFFFFFFFFFFFFFFFFFFF /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
			};
			name = Release;
		};
/* End XCBuildConfiguration section */
"#,
    )
    .unwrap();
    (project_root, project_file)
}

pub(super) fn create_placeholder_xcframework_with_signatures(libs_dir: &Path, name: &str) {
    let framework = name
        .strip_suffix(".xcframework")
        .expect("test xcframework name");
    let root = libs_dir.join(name);
    for dir in [
        root.join("_CodeSignature"),
        root.join(format!("ios-arm64/{}.framework/_CodeSignature", framework)),
        root.join(format!(
            "ios-arm64_x86_64-simulator/{}.framework/_CodeSignature",
            framework
        )),
    ] {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("CodeResources"), "signature").unwrap();
    }
    std::fs::write(
        root.join(format!("ios-arm64/{}.framework/{}", framework, framework)),
        "archive",
    )
    .unwrap();
    std::fs::write(root.join("Info.plist"), "<plist/>").unwrap();
}

pub(super) fn contains_code_signature_dir(root: &Path) -> bool {
    if !root.is_dir() {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        entry
            .file_type()
            .map(|file_type| {
                file_type.is_dir()
                    && (path.file_name().and_then(|name| name.to_str()) == Some("_CodeSignature")
                        || contains_code_signature_dir(&path))
            })
            .unwrap_or(false)
    })
}
