use ::plist as plist_crate;

use super::super::support::prepare_ios_uts_project;
use crate::commands::ios::modules::uts_plugins::apply_ios_uts_plugins;
use crate::commands::shared::resource_scan::scan_uts_plugins;

#[test]
fn ios_uts_plugins_copy_and_register_ios_only_plugin_assets() {
    let root = std::env::temp_dir().join(format!("unipack-ios-uts-{}", uuid::Uuid::new_v4()));
    let (project_root, project_file) = prepare_ios_uts_project(&root);
    let resource_root = root.join("resource");
    let ios_dir = resource_root.join("uni_modules/demo-uts/utssdk/app-iOS");
    std::fs::create_dir_all(ios_dir.join("Frameworks/DemoUTS.framework")).unwrap();
    std::fs::create_dir_all(ios_dir.join("Resources/DemoUTS.bundle")).unwrap();
    std::fs::write(
        ios_dir.join("config.json"),
        r#"{
            "frameworks":["CoreLocation"],
            "plists":{"NSCameraUsageDescription":"UTS camera"},
            "hooksClass":"DemoHook",
            "provider":"DemoProvider",
            "components":[{"name":"demo-view","class":"DemoView"}],
            "dependencies-pods":{"Alamofire":"~> 5.0"}
        }"#,
    )
    .unwrap();
    let builtin_ios = resource_root.join("uni_modules/uni-getNetworkType/utssdk/app-ios");
    std::fs::create_dir_all(&builtin_ios).unwrap();

    let scan = scan_uts_plugins(&resource_root);
    assert!(scan.has_ios_uts_plugins);
    assert!(!scan.has_android_uts_plugins);

    let integration = apply_ios_uts_plugins(&project_root, &project_file, &scan)
        .unwrap()
        .expect("iOS UTS plugins should be applied");

    assert_eq!(integration.custom_framework_count, 1);
    assert_eq!(integration.system_framework_count, 1);
    assert_eq!(integration.resource_count, 1);
    assert_eq!(integration.plist_count, 1);
    assert_eq!(integration.pod_dependency_count, 1);
    assert_eq!(integration.removed_duplicate_count, 1);
    assert!(project_root
        .join("UTSPlugins/demo-uts/DemoUTS.framework")
        .is_dir());
    assert!(project_root
        .join("HBuilder-Hello/UTSResources/demo-uts/DemoUTS.bundle")
        .is_dir());

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("DCUniBase.framework in Embed Frameworks"));
    assert!(pbxproj.contains("DCloudUTSFoundation.framework in Embed Frameworks"));
    assert!(!pbxproj.contains("DCloudUTSExtAPI.framework"));
    assert!(pbxproj.contains("DemoUTS.framework in Embed Frameworks"));
    assert!(pbxproj.contains("\"$(PROJECT_DIR)/UTSPlugins/demo-uts\""));
    assert!(pbxproj.contains("CoreLocation.framework in Frameworks"));
    assert!(pbxproj.contains("demo-uts-DemoUTS.bundle in Resources"));
    assert!(!pbxproj.contains("uts-config.json"));
    assert!(!pbxproj.contains("liblibPDRCore.a in Frameworks"));

    let plist = plist_crate::Value::from_file(
        project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist"),
    )
    .unwrap();
    assert_eq!(
        plist
            .as_dictionary()
            .and_then(|dict| dict.get("NSCameraUsageDescription"))
            .and_then(plist_crate::Value::as_string),
        Some("UTS camera")
    );
    assert!(!project_root.join("HBuilder-Hello/uts-config.json").exists());

    let _ = std::fs::remove_dir_all(root);
}
