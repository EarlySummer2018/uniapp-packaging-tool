use super::super::fs_utils::link_ios_sdk_support;
use super::super::runtime::{import_app_resource, patch_control_xml, resolve_ios_runtime_layout};

#[test]
fn runtime_layout_supports_nested_hbuilder_source_directory() {
    let root = std::env::temp_dir().join(format!("unipack-ios-layout-{}", uuid::Uuid::new_v4()));
    let source = root.join("HBuilder-Hello");
    let resource = root.join("resource");
    std::fs::create_dir_all(source.join("Pandora/apps")).unwrap();
    std::fs::create_dir_all(&resource).unwrap();
    std::fs::write(
        source.join("control.xml"),
        r#"<HBuilder><apps><app appid="__UNI__OLD"/></apps></HBuilder>"#,
    )
    .unwrap();
    std::fs::write(resource.join("manifest.json"), "{}").unwrap();

    let layout = resolve_ios_runtime_layout(&root).unwrap();
    import_app_resource(&layout.apps_dir, &resource, "__UNI__NEW").unwrap();
    patch_control_xml(&layout.control_xml, "__UNI__NEW").unwrap();

    assert_eq!(layout.control_xml, source.join("control.xml"));
    assert_eq!(layout.apps_dir, source.join("Pandora/apps"));
    assert!(source
        .join("Pandora/apps/__UNI__NEW/manifest.json")
        .is_file());
    assert!(!root.join("Pandora").exists());
    assert!(std::fs::read_to_string(source.join("control.xml"))
        .unwrap()
        .contains(r#"appid="__UNI__NEW""#));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn runtime_layout_supports_control_inside_pandora() {
    let root = std::env::temp_dir().join(format!("unipack-ios-layout-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("Pandora/apps")).unwrap();
    std::fs::write(root.join("Pandora/control.xml"), "<HBuilder />").unwrap();

    let layout = resolve_ios_runtime_layout(&root).unwrap();

    assert_eq!(layout.control_xml, root.join("Pandora/control.xml"));
    assert_eq!(layout.apps_dir, root.join("Pandora/apps"));
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn workspace_links_sibling_sdk_support_directory() {
    let root = std::env::temp_dir().join(format!("unipack-ios-support-{}", uuid::Uuid::new_v4()));
    let sdk_project = root.join("package/HBuilder-Hello");
    let support = root.join("package/SDK");
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&sdk_project).unwrap();
    std::fs::create_dir_all(&support).unwrap();
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(support.join("PrivacyInfo.xcprivacy"), "privacy").unwrap();

    let linked = link_ios_sdk_support(&sdk_project, &workspace)
        .unwrap()
        .unwrap();

    assert_eq!(linked, workspace.join("SDK"));
    assert!(linked.join("PrivacyInfo.xcprivacy").is_file());
    assert!(std::fs::symlink_metadata(&linked)
        .unwrap()
        .file_type()
        .is_symlink());
    std::fs::remove_dir_all(&workspace).unwrap();
    assert!(support.join("PrivacyInfo.xcprivacy").is_file());
    let _ = std::fs::remove_dir_all(root);
}
