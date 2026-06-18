use super::super::fs_utils::find_scheme_name;
use super::super::pbxproj::{
    append_pbx_build_setting_paths_to_content, legacy_simulator_x86_64_required,
    raise_pbx_ios_deployment_target, register_pbx_embedded_frameworks, register_pbx_linked_files,
    remove_pbx_build_setting_flag, set_pbx_build_setting, IosPbxLinkedFile,
};

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
fn pbx_linker_flag_remove_strips_ld_classic_and_keeps_other_flags() {
    let content = "\t\t\t\tOTHER_LDFLAGS = \"$(inherited) -ObjC -ld_classic\";\n";
    let updated = remove_pbx_build_setting_flag(content, "OTHER_LDFLAGS", "-ld_classic");
    let updated_again = remove_pbx_build_setting_flag(&updated, "OTHER_LDFLAGS", "-ld_classic");

    assert!(updated.contains("OTHER_LDFLAGS = \"$(inherited) -ObjC\";"));
    assert!(!updated.contains("-ld_classic"));
    assert_eq!(updated, updated_again);
}

#[test]
fn pbx_deployment_target_raise_preserves_higher_values() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-deployment-target-{}",
        uuid::Uuid::new_v4()
    ));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin XCBuildConfiguration section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				IPHONEOS_DEPLOYMENT_TARGET = 12.0;
			};
			name = Debug;
		};
		BBBBBBBBBBBBBBBBBBBBBBBB /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				IPHONEOS_DEPLOYMENT_TARGET = 14.0;
			};
			name = Release;
		};
/* End XCBuildConfiguration section */
"#,
    )
    .unwrap();

    assert!(raise_pbx_ios_deployment_target(&project_file, "13.0").unwrap());

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("IPHONEOS_DEPLOYMENT_TARGET = 13.0;"));
    assert!(pbxproj.contains("IPHONEOS_DEPLOYMENT_TARGET = 14.0;"));
    assert!(!pbxproj.contains("IPHONEOS_DEPLOYMENT_TARGET = 12.0;"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pbx_search_path_appends_to_array_and_is_idempotent() {
    let content = r#"buildSettings = {
				FRAMEWORK_SEARCH_PATHS = (
					"$(inherited)",
					"$(PROJECT_DIR)",
				);
};
"#;
    let paths = vec!["$(PROJECT_DIR)/UTSPlugins/Dingtalk-DingRTC".to_string()];

    let (updated, changed) =
        append_pbx_build_setting_paths_to_content(content, "FRAMEWORK_SEARCH_PATHS", &paths);
    let (updated_again, changed_again) =
        append_pbx_build_setting_paths_to_content(&updated, "FRAMEWORK_SEARCH_PATHS", &paths);

    assert_eq!(changed, 1);
    assert_eq!(changed_again, 0);
    assert_eq!(updated, updated_again);
    assert!(updated.contains("\"$(PROJECT_DIR)/UTSPlugins/Dingtalk-DingRTC\""));
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
