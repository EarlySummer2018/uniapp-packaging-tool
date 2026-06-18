use super::super::config::{
    effective_app_name, effective_app_version, effective_app_version_code,
    resolve_ios_manifest_info, validate_ios_app_id,
};

#[test]
fn ios_build_reloads_manifest_from_configured_local_project() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-local-manifest-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(root.join("unpackage/res/icons")).unwrap();
    std::fs::write(root.join("unpackage/res/icons/1024.png"), "icon").unwrap();
    std::fs::write(
        root.join("manifest.json"),
        r#"{
                "name": "Manifest App",
                "appid": "__UNI__MANIFEST",
                "versionName": "2.3.4",
                "versionCode": "234",
                "app-plus": {
                    "distribute": {
                        "ios": {
                            "privacyDescription": {
                                "NSCameraUsageDescription": "用于扫码"
                            }
                        },
                        "icons": {
                            "ios": {
                                "appstore": "unpackage/res/icons/1024.png"
                            }
                        }
                    }
                }
            }"#,
    )
    .unwrap();
    let mut config = crate::commands::project::ProjectConfig::default();
    config.local_path = root.to_string_lossy().to_string();
    config.app.name = "Config App".into();
    config.app.version = "1.0.0".into();
    config.app.version_code = 1;

    let info = resolve_ios_manifest_info(&config, None).unwrap().unwrap();

    assert_eq!(effective_app_name(&config, Some(&info)), "Manifest App");
    assert_eq!(effective_app_version(&config, Some(&info)), "2.3.4");
    assert_eq!(effective_app_version_code(&config, Some(&info)), 234);
    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSCameraUsageDescription")
            .map(String::as_str),
        Some("用于扫码")
    );
    let expected_icon = root
        .join("unpackage/res/icons/1024.png")
        .to_string_lossy()
        .to_string();
    assert_eq!(
        info.ios_icons
            .as_ref()
            .and_then(|icons| icons.ios.get("appstore"))
            .map(String::as_str),
        Some(expected_icon.as_str())
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_build_center_overrides_survive_local_manifest_reload() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-build-center-overrides-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let manifest = serde_json::json!({
        "name": "Manifest App",
        "appid": "__UNI__MANIFEST",
        "app-plus": {
            "modules": {
                "Geolocation": {}
            },
            "distribute": {
                "ios": {
                    "UIBackgroundModes": ["bluetooth-central"]
                },
                "sdkConfigs": {
                    "geolocation": {
                        "baidu": {
                            "__platform__": ["ios"],
                            "appkey_ios": "local-baidu-key"
                        }
                    }
                }
            }
        }
    });
    std::fs::write(
        root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let mut config = crate::commands::project::ProjectConfig::default();
    config.local_path = root.to_string_lossy().to_string();

    let mut supplied = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );
    supplied.ios_privacy_descriptions.insert(
        "NSLocationWhenInUseUsageDescription".into(),
        "构建中心定位说明".into(),
    );
    supplied.manifest_value.as_mut().unwrap()["app-plus"]["distribute"]["sdkConfigs"]
        ["geolocation"]["baidu"]["appkey_ios"] =
        serde_json::Value::String("build-center-baidu-key".into());
    supplied.manifest_value.as_mut().unwrap()["app-plus"]["distribute"]["ios"]
        ["UIBackgroundModes"] = serde_json::Value::Array(Vec::new());

    let info = resolve_ios_manifest_info(&config, Some(&supplied))
        .unwrap()
        .unwrap();

    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSLocationWhenInUseUsageDescription")
            .map(String::as_str),
        Some("构建中心定位说明")
    );
    assert_eq!(
        info.manifest_value.as_ref().and_then(|manifest| {
            manifest
                .get("app-plus")?
                .get("distribute")?
                .get("sdkConfigs")?
                .get("geolocation")?
                .get("baidu")?
                .get("appkey_ios")?
                .as_str()
        }),
        Some("build-center-baidu-key")
    );
    assert_eq!(
        info.manifest_value.as_ref().and_then(|manifest| {
            manifest
                .get("app-plus")?
                .get("distribute")?
                .get("ios")?
                .get("UIBackgroundModes")?
                .as_array()
                .map(Vec::len)
        }),
        Some(0)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_build_rejects_manifest_and_resource_app_id_mismatch() {
    let manifest = serde_json::json!({ "appid": "__UNI__MANIFEST" });
    let root = std::env::temp_dir().join(format!("unipack-ios-appid-{}", uuid::Uuid::new_v4()));
    let info = crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        &root,
        None,
    );

    let error = validate_ios_app_id("__UNI__RESOURCE", Some(&info)).unwrap_err();

    assert!(error.contains("__UNI__MANIFEST"));
    assert!(error.contains("__UNI__RESOURCE"));
}
