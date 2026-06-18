use super::super::support::{
    contains_code_signature_dir, create_placeholder_xcframework_with_signatures,
    prepare_ios_payment_alipay_project,
};
use crate::commands::ios::modules::oauth::{
    apply_ios_oauth_module, ios_oauth_providers, IosOauthProvider,
};

#[test]
fn ios_oauth_detects_enabled_providers_from_manifest() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-oauth-providers-{}",
        uuid::Uuid::new_v4()
    ));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "OAuth": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "univerify": {},
                        "sinaweibo": {},
                        "qq": {},
                        "weixin": {},
                        "apple": {},
                        "google": {},
                        "facebook": {}
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
        ios_oauth_providers(Some(&info)).unwrap(),
        vec![
            IosOauthProvider::Univerify,
            IosOauthProvider::Sina,
            IosOauthProvider::Qq,
            IosOauthProvider::Weixin,
            IosOauthProvider::Apple,
            IosOauthProvider::Google,
            IosOauthProvider::Facebook,
        ]
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_oauth_sina_uses_weibo_sdk_library_name_from_offline_sdk() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-oauth-sina-{}", uuid::Uuid::new_v4()));
    let (project_root, project_file, libs_dir) = prepare_ios_payment_alipay_project(&root);
    std::fs::create_dir_all(root.join("SDK/Bundles/WeiboSDK.bundle")).unwrap();
    std::fs::write(libs_dir.join("liblibOauth.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libSinaWBOauth.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libWeiboSDK.a"), "lib").unwrap();
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "OAuth": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "sinaweibo": {
                            "__platform__": ["ios"]
                        }
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

    let integration = apply_ios_oauth_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .expect("Sina OAuth should be applied");

    assert_eq!(integration.providers, vec![IosOauthProvider::Sina]);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("libWeiboSDK.a in Frameworks"));
    assert!(!pbxproj.contains("liblWeiboSDK.a"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_oauth_facebook_uses_sanitized_xcframework_copies() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-oauth-facebook-{}",
        uuid::Uuid::new_v4()
    ));
    let (project_root, project_file, libs_dir) = prepare_ios_payment_alipay_project(&root);
    std::fs::write(libs_dir.join("liblibOauth.a"), "lib").unwrap();
    std::fs::write(libs_dir.join("libFBOauth.a"), "lib").unwrap();
    for name in [
        "FBSDKCoreKit.xcframework",
        "FBAEMKit.xcframework",
        "FBSDKCoreKit_Basics.xcframework",
        "FBSDKLoginKit.xcframework",
    ] {
        create_placeholder_xcframework_with_signatures(&libs_dir, name);
    }
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "OAuth": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "facebook": {
                            "__platform__": ["ios"],
                            "appid": "fb-ios"
                        }
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

    let integration = apply_ios_oauth_module(&project_root, &project_file, Some(&info))
        .unwrap()
        .expect("Facebook OAuth should be applied");

    assert_eq!(integration.providers, vec![IosOauthProvider::Facebook]);
    assert_eq!(integration.facebook_compat_xcframework_count, 4);
    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert!(pbxproj.contains("UniPackSanitizedSDK/FBSDKCoreKit.xcframework"));
    assert!(pbxproj.contains("UniPackSanitizedSDK/FBAEMKit.xcframework"));
    assert!(!pbxproj.contains("../SDK/Libs/FBSDKCoreKit.xcframework"));

    let sanitized = project_root.join("UniPackSanitizedSDK/FBSDKCoreKit.xcframework");
    assert!(sanitized.is_dir());
    assert!(!contains_code_signature_dir(&sanitized));
    assert!(contains_code_signature_dir(
        &libs_dir.join("FBSDKCoreKit.xcframework")
    ));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_oauth_ignores_local_pod_and_requires_offline_sdk_files() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-oauth-local-pod-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "OAuth": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "localPod": true,
                        "weixin": {
                            "appid": "wx-oauth"
                        }
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

    let error = apply_ios_oauth_module(&project_root, &project_file, Some(&info)).unwrap_err();

    assert!(error.contains("iOS Oauth 模块缺少 SDK 依赖文件"));
    assert!(error.contains("liblibOauth.a"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_oauth_manual_integration_requires_offline_sdk_files() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-oauth-missing-sdk-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "OAuth": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "oauth": {
                        "weixin": {
                            "appid": "wx-oauth"
                        }
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

    let error = apply_ios_oauth_module(&project_root, &project_file, Some(&info)).unwrap_err();

    assert!(error.contains("iOS Oauth 模块缺少 SDK 依赖文件"));
    assert!(error.contains("liblibOauth.a"));
    let _ = std::fs::remove_dir_all(root);
}
