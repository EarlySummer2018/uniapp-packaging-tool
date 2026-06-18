use crate::commands::ios::modules::ui_webview::apply_ios_ui_webview_module;

#[test]
fn ios_ui_webview_requires_offline_sdk_library() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-uiwebview-missing-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "UIWebview": {}
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    let error = apply_ios_ui_webview_module(&project_root, &project_file, Some(&info)).unwrap_err();

    assert!(error.contains("iOS UIWebview 模块缺少 SDK 依赖文件"));
    assert!(error.contains("libH5WEUIWebview.a"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_ui_webview_links_documented_dependencies_idempotently() {
    let root = std::env::temp_dir().join(format!("unipack-ios-uiwebview-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let libs_dir = root.join("SDK/Libs");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&libs_dir).unwrap();
    std::fs::write(libs_dir.join("libH5WEUIWebview.a"), "lib").unwrap();
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
                "UIWebview": {}
            }
        }
    });
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    let integration = apply_ios_ui_webview_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();

    assert_eq!(integration.linked_count, 4);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("libH5WEUIWebview.a in Frameworks"));
    assert!(pbxproj.contains("../SDK/Libs/libH5WEUIWebview.a"));
    assert!(pbxproj.contains("JavaScriptCore.framework in Frameworks"));
    assert!(pbxproj.contains("Foundation.framework in Frameworks"));
    assert!(pbxproj.contains("UIKit.framework in Frameworks"));

    let integration = apply_ios_ui_webview_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .unwrap();
    assert_eq!(integration.linked_count, 0);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(
        pbxproj.matches("libH5WEUIWebview.a in Frameworks").count(),
        2
    );
    let _ = std::fs::remove_dir_all(root);
}
