use super::*;

#[test]
fn parse_manifest_reads_distribute_app_icons() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-app-icons-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "icons": {
                    "android": {
                        "hdpi": "unpackage/res/icons/72x72.png",
                        "xhdpi": "unpackage/res/icons/96x96.png"
                    },
                    "ios": {
                        "appstore": "unpackage/res/icons/1024x1024.png",
                        "iphone": {
                            "app@3x": "unpackage/res/icons/180x180.png"
                        },
                        "ipad": {
                            "proapp@2x": "unpackage/res/icons/167x167.png"
                        }
                    }
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let android = &info
        .android_icons
        .as_ref()
        .expect("Android icons should be parsed")
        .android;
    assert_eq!(
        android.get("xhdpi"),
        Some(
            &project_root
                .join("unpackage/res/icons/96x96.png")
                .to_string_lossy()
                .to_string()
        )
    );

    let ios = &info
        .ios_icons
        .as_ref()
        .expect("iOS icons should be parsed")
        .ios;
    assert_eq!(
        ios.get("appstore"),
        Some(
            &project_root
                .join("unpackage/res/icons/1024x1024.png")
                .to_string_lossy()
                .to_string()
        )
    );
    assert_eq!(
        ios.get("iphone.app@3x"),
        Some(
            &project_root
                .join("unpackage/res/icons/180x180.png")
                .to_string_lossy()
                .to_string()
        )
    );
    assert_eq!(
        ios.get("ipad.proapp@2x"),
        Some(
            &project_root
                .join("unpackage/res/icons/167x167.png")
                .to_string_lossy()
                .to_string()
        )
    );
}

#[test]
fn parse_manifest_reads_ios_storyboard_splashscreen() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-ios-splash-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "splashscreen": {
                    "iosStyle": "storyboard",
                    "ios": {
                        "storyboard": "static/storyboard/storyboard.zip"
                    }
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );
    let splash = info
        .splashscreen
        .expect("iOS splashscreen should be parsed");

    assert_eq!(splash.ios_style.as_deref(), Some("storyboard"));
    assert_eq!(
        splash.ios_storyboard.as_deref(),
        Some(
            project_root
                .join("static/storyboard/storyboard.zip")
                .to_string_lossy()
                .as_ref()
        )
    );
}

#[test]
fn parse_manifest_reads_all_non_empty_ios_privacy_descriptions() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-ios-privacy-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "ios": {
                    "privacyDescription": {
                        "NSPhotoLibraryUsageDescription": "在上传头像或发布内容时，开启相册权限便于您保存图片或选择图片上传",
                        "NSPhotoLibraryAddUsageDescription": "该应用需要读取您的相册，以便您使用应用生成海报时保存到相册",
                        "NSCameraUsageDescription": "在上传头像或发布内容时，开启相机权限便于您拍照上传图片",
                        "NSLocalNetworkUsageDescription": "请允许访问本地网络，以便更好的体验应用",
                        "NSContactsUsageDescription": "用于拨通客服热线",
                        "NSLocationWhenInUseUsageDescription": "",
                        "NSUnsupportedUsageDescription": "不应写入"
                    }
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSPhotoLibraryUsageDescription"),
        Some(&"在上传头像或发布内容时，开启相册权限便于您保存图片或选择图片上传".to_string())
    );
    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSPhotoLibraryAddUsageDescription"),
        Some(&"该应用需要读取您的相册，以便您使用应用生成海报时保存到相册".to_string())
    );
    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSCameraUsageDescription"),
        Some(&"在上传头像或发布内容时，开启相机权限便于您拍照上传图片".to_string())
    );
    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSLocalNetworkUsageDescription"),
        Some(&"请允许访问本地网络，以便更好的体验应用".to_string())
    );
    assert_eq!(
        info.ios_privacy_descriptions
            .get("NSContactsUsageDescription"),
        Some(&"用于拨通客服热线".to_string())
    );
    assert!(!info
        .ios_privacy_descriptions
        .contains_key("NSLocationWhenInUseUsageDescription"));
    assert!(!info
        .ios_privacy_descriptions
        .contains_key("NSUnsupportedUsageDescription"));
}

#[test]
fn read_uniapp_manifest_caches_raw_manifest_value() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-manifest-cache-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(
        project_root.join("manifest.json"),
        r#"{
                "appid": "__UNI__CACHE",
                "app-plus": {
                    "distribute": {
                        "icons": {
                            "android": {
                                "hdpi": "unpackage/res/icons/72x72.png"
                            }
                        }
                    }
                }
            }"#,
    )
    .unwrap();

    let info = read_uniapp_manifest_sync(project_root.to_str().unwrap()).unwrap();
    let cached = info
        .manifest_value
        .as_ref()
        .expect("raw manifest JSON should be cached");

    assert_eq!(
        cached.get("appid").and_then(|value| value.as_str()),
        Some("__UNI__CACHE")
    );
    assert_eq!(
        cached
            .pointer("/app-plus/distribute/icons/android/hdpi")
            .and_then(|value| value.as_str()),
        Some("unpackage/res/icons/72x72.png")
    );

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn parse_manifest_reads_android_distribute_build_fields() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-android-fields-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "android": {
                    "permissions": [
                        "<uses-permission android:name=\"android.permission.INTERNET\" />",
                        "android.permission.CAMERA"
                    ],
                    "excludePermissions": [
                        "<uses-permission android:name=\"android.permission.READ_LOGS\" />"
                    ],
                    "schemes": "comchatvivaus, demoapp",
                    "abiFilters": ["arm64-v8a"],
                    "minSdkVersion": 23
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    assert_eq!(
        info.android.permissions,
        vec![
            r#"<uses-permission android:name="android.permission.INTERNET" />"#.to_string(),
            "android.permission.CAMERA".to_string()
        ]
    );
    assert_eq!(
        info.android.exclude_permissions,
        vec![r#"<uses-permission android:name="android.permission.READ_LOGS" />"#.to_string()]
    );
    assert_eq!(
        info.android.schemes,
        vec!["comchatvivaus".to_string(), "demoapp".to_string()]
    );
    assert_eq!(info.android.abi_filters, vec!["arm64-v8a".to_string()]);
    assert_eq!(info.android.min_sdk_version, Some(23));
}

#[test]
fn parse_manifest_ignores_empty_android_distribute_build_fields() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-empty-android-fields-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "android": {
                    "permissions": "",
                    "excludePermissions": [],
                    "schemes": "",
                    "abiFilters": [],
                    "minSdkVersion": "",
                    "targetSdkVersion": ""
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    assert_eq!(info.android.min_sdk_version, Some(21));
    assert_eq!(info.android.target_sdk_version, None);
    assert!(info.android.permissions.is_empty());
    assert!(info.android.exclude_permissions.is_empty());
    assert!(info.android.schemes.is_empty());
    assert!(info.android.abi_filters.is_empty());
}

#[test]
fn parse_manifest_reads_push_small_icon_path() {
    let project_root =
        std::env::temp_dir().join(format!("unipack-push-icon-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "push": {
                        "icons": {
                            "small": "static/push_icon.png"
                        }
                    }
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let small_icon = info
        .push_icons
        .as_ref()
        .and_then(|icons| icons.small.as_ref())
        .expect("push small icon should be parsed");
    assert_eq!(
        small_icon,
        &project_root
            .join("static/push_icon.png")
            .to_string_lossy()
            .to_string()
    );
}

#[test]
fn parse_manifest_reads_nested_unipush_small_icon_densities() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-push-icon-density-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "sdkConfigs": {
                    "push": {
                        "unipush": {
                            "icons": {
                                "small": {
                                    "hdpi": "static/push/36x36.png",
                                    "xhdpi": "static/push/48x48.png"
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let densities = &info
        .push_icons
        .as_ref()
        .expect("push icons should be parsed")
        .small_densities;
    assert_eq!(
        densities.get("hdpi"),
        Some(
            &project_root
                .join("static/push/36x36.png")
                .to_string_lossy()
                .to_string()
        )
    );
    assert_eq!(
        densities.get("xhdpi"),
        Some(
            &project_root
                .join("static/push/48x48.png")
                .to_string_lossy()
                .to_string()
        )
    );
}

#[test]
fn parse_manifest_reads_distribute_push_unipush_small_icon_densities() {
    let project_root = std::env::temp_dir().join(format!(
        "unipack-push-icon-new-density-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "push": {
                    "unipush": {
                        "version": "2",
                        "offline": true,
                        "icons": {
                            "small": {
                                "hdpi": "static/push/36x36.png",
                                "xxhdpi": "static/push/72x72.png"
                            }
                        }
                    }
                }
            }
        }
    });

    let info = parse_uniapp_manifest(
        &manifest,
        &project_root.join("manifest.json"),
        &project_root,
        None,
    );

    let densities = &info
        .push_icons
        .as_ref()
        .expect("new push icons path should be parsed")
        .small_densities;
    assert_eq!(
        densities.get("hdpi"),
        Some(
            &project_root
                .join("static/push/36x36.png")
                .to_string_lossy()
                .to_string()
        )
    );
    assert_eq!(
        densities.get("xxhdpi"),
        Some(
            &project_root
                .join("static/push/72x72.png")
                .to_string_lossy()
                .to_string()
        )
    );
}
