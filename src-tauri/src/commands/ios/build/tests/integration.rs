use std::path::Path;

use ::plist as plist_crate;

use super::super::super::icons::generate_app_icons;
use super::super::config::resolve_ios_manifest_info;
use super::super::entitlements::patch_ios_entitlements;
use super::super::fs_utils::find_xcodeproj;
use super::super::pbxproj::patch_pbxproj;
use super::super::plist::patch_info_plist;
use super::super::splashscreen::apply_ios_splashscreen;

#[test]
fn configured_ios_sdk_project_accepts_local_manifest_when_requested() {
    let Ok(sdk_project) = std::env::var("UNIPACK_TEST_IOS_SDK_PROJECT") else {
        return;
    };
    let Ok(local_project) = std::env::var("UNIPACK_TEST_UNIAPP_PROJECT") else {
        return;
    };
    let root =
        std::env::temp_dir().join(format!("unipack-ios-real-config-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    crate::utils::fs::copy_recursive(Path::new(&sdk_project), &project_root).unwrap();
    let project_file = find_xcodeproj(&project_root).unwrap();
    let mut config = crate::commands::project::ProjectConfig::default();
    config.local_path = local_project;
    config.ios.bundle_id = "com.example.unipack.verify".into();
    config.ios.team_id = "TEAM123".into();
    config.ios.dcloud_app_key = "verify-app-key".into();
    let info = resolve_ios_manifest_info(&config, None).unwrap().unwrap();

    patch_pbxproj(&project_file, &config, Some(&info)).unwrap();
    apply_ios_splashscreen(&project_root, &project_file, Some(&info)).unwrap();
    patch_info_plist(
        &project_root,
        &project_file,
        &config,
        info.app_id.as_deref().unwrap_or("__UNI__VERIFY"),
        Some(&info),
    )
    .unwrap();
    assert_eq!(
        patch_ios_entitlements(&project_root, &project_file, Some(&info)).unwrap(),
        1
    );
    generate_app_icons(&project_root, &config, Some(&info)).unwrap();

    let output = std::process::Command::new("xcodebuild")
        .args(["-list", "-project"])
        .arg(&project_file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("dc_launchscreen_portrait_background@2x.png in Resources"));
    assert!(project_root
        .join("HBuilder-Hello/dc_launchscreen_portrait_background@2x.png")
        .is_file());
    let plist = plist_crate::Value::from_file(
        project_root.join("HBuilder-Hello/HBuilder-Hello-Info.plist"),
    )
    .unwrap();
    let plist = plist.as_dictionary().unwrap();
    assert_eq!(
        plist
            .get("CFBundleDisplayName")
            .and_then(plist_crate::Value::as_string),
        Some("ccc222")
    );
    assert_eq!(
        plist
            .get("AMapApiKey")
            .and_then(plist_crate::Value::as_string),
        Some("e58f1b2f4c1e3d8a9b7c6d5e4f3a2b1c")
    );
    let entitlements =
        plist_crate::Value::from_file(project_root.join("HBuilder/HBuilder.entitlements")).unwrap();
    assert!(entitlements
        .as_dictionary()
        .unwrap()
        .contains_key("com.apple.developer.associated-domains"));
    let _ = std::fs::remove_dir_all(root);
}
