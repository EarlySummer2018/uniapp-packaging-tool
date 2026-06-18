use super::super::splashscreen::apply_ios_splashscreen;

#[test]
fn ios_storyboard_zip_replaces_launch_screen_and_registers_resources() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-storyboard-{}", uuid::Uuid::new_v4()));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    let source_dir = root.join("HBuilder-Hello");
    let launch_screen = source_dir.join("LaunchScreen.storyboard");
    let zip_path = root.join("storyboard.zip");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(&launch_screen, "old storyboard").unwrap();
    std::fs::write(source_dir.join("HBuilder-Hello-Info.plist"), "<plist/>").unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"/* Begin PBXBuildFile section */
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
/* End PBXFileReference section */
		AAA /* Supporting Files */ = {
			isa = PBXGroup;
			children = (
			);
		};
		BBB /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			files = (
			);
		};
				INFOPLIST_FILE = "HBuilder-Hello/HBuilder-Hello-Info.plist";
"#,
    )
    .unwrap();
    let zip_file = std::fs::File::create(&zip_path).unwrap();
    let mut writer = zip::ZipWriter::new(zip_file);
    let options = zip::write::SimpleFileOptions::default();
    writer
        .start_file("LaunchScreen.storyboard", options)
        .unwrap();
    std::io::Write::write_all(&mut writer, b"new storyboard").unwrap();
    writer
        .start_file("images/background@2x.png", options)
        .unwrap();
    std::io::Write::write_all(&mut writer, b"image").unwrap();
    writer.finish().unwrap();
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "splashscreen": {
                    "iosStyle": "storyboard",
                    "ios": {
                        "storyboard": zip_path.to_string_lossy()
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

    assert_eq!(
        apply_ios_splashscreen(&root, &project_file, Some(&info)).unwrap(),
        Some(1)
    );
    assert_eq!(
        std::fs::read_to_string(&launch_screen).unwrap(),
        "new storyboard"
    );
    assert!(source_dir.join("background@2x.png").is_file());
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("background@2x.png in Resources"));
    assert!(pbxproj.contains("lastKnownFileType = image.png"));

    apply_ios_splashscreen(&root, &project_file, Some(&info)).unwrap();
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(pbxproj.matches("background@2x.png in Resources").count(), 2);
    let _ = std::fs::remove_dir_all(root);
}
