use std::path::{Path, PathBuf};

use crate::commands::shared::resource_scan::{
    ResourceScanResult, UtsCustomPlugin, UtsPluginScanResult,
};

fn parse_manifest(
    root: &Path,
    manifest: serde_json::Value,
) -> crate::commands::resource::UniappManifestInfo {
    crate::commands::shared::resource::parse_uniapp_manifest(
        &manifest,
        &root.join("manifest.json"),
        root,
        None,
    )
}

fn empty_scan(root: &Path) -> ResourceScanResult {
    ResourceScanResult {
        app_id: "__UNI__POD".into(),
        app_name: None,
        version_name: None,
        version_code: None,
        hbuilderx_version: None,
        source_path: root.to_string_lossy().to_string(),
        imported_path: root.to_string_lossy().to_string(),
        app_resource_path: root.to_string_lossy().to_string(),
        is_zip: false,
        manifest_path: None,
        splashscreen: None,
        detected_modules: Vec::new(),
        uts: UtsPluginScanResult::default(),
        warnings: Vec::new(),
    }
}

#[test]
fn podfile_renders_local_uniapp_subspecs() {
    let root = PathBuf::from("/tmp/HBuilder-Hello");
    let project = root.join("HBuilder-Hello.xcodeproj");
    let content = super::super::pod::render_ios_podfile(
        &project,
        &["Core".into(), "Payment-Wechat".into(), "Map-Gaode".into()],
    )
    .unwrap();

    assert!(content.contains("platform :ios, '13.0'"));
    assert!(content.contains("project 'HBuilder-Hello.xcodeproj'"));
    assert!(content.contains("pod 'uniapp', :path => '..', :subspecs => uniapp_subspecs"));
    assert!(content.contains("'Core'"));
    assert!(content.contains("'Payment-Wechat'"));
    assert!(content.contains("'Map-Gaode'"));
}

#[test]
fn podspec_copy_includes_license_file_when_present() {
    let root = std::env::temp_dir().join(format!("unipack-ios-pod-copy-{}", uuid::Uuid::new_v4()));
    let sdk = root.join("sdk");
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&sdk).unwrap();
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(sdk.join("uniapp.podspec"), "Pod::Spec.new").unwrap();
    std::fs::write(sdk.join("license.md"), "DCloud").unwrap();

    super::super::pod::copy_uniapp_podspec(&sdk, &workspace).unwrap();

    assert_eq!(
        std::fs::read_to_string(workspace.join("uniapp.podspec")).unwrap(),
        "Pod::Spec.new"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.join("license.md")).unwrap(),
        "DCloud"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pod_xcode_patch_adds_sdk_header_search_paths_once() {
    let content = r#"buildSettings = {
				HEADER_SEARCH_PATHS = (
					"$(inherited)",
					/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include,
				);
			};
buildSettings = {
				HEADER_SEARCH_PATHS = (
					"$(inherited)",
				);
			};"#;

    let (updated, patched_count) = super::super::pod_xcode::ensure_header_search_paths(
        content,
        &["$(SRCROOT)/../SDK/inc", "$(SRCROOT)/../SDK/inc/**"],
    );
    let (updated_again, patched_again) = super::super::pod_xcode::ensure_header_search_paths(
        &updated,
        &["$(SRCROOT)/../SDK/inc", "$(SRCROOT)/../SDK/inc/**"],
    );

    assert_eq!(patched_count, 2);
    assert_eq!(patched_again, 0);
    assert_eq!(updated, updated_again);
    assert_eq!(updated.matches("$(SRCROOT)/../SDK/inc\",").count(), 2);
    assert_eq!(updated.matches("$(SRCROOT)/../SDK/inc/**").count(), 2);
}

#[test]
fn pod_xcode_patch_adds_core_libraries_once() {
    let root = std::env::temp_dir().join(format!("unipack-ios-pod-xcode-{}", uuid::Uuid::new_v4()));
    let (_project_root, project_file, _libs_dir) =
        super::support::prepare_ios_payment_alipay_project(&root);

    let linked = super::super::pod_xcode::ensure_ios_pod_core_libraries(&project_file).unwrap();
    let linked_again =
        super::super::pod_xcode::ensure_ios_pod_core_libraries(&project_file).unwrap();

    let pbxproj = std::fs::read_to_string(project_file.join("project.pbxproj")).unwrap();
    assert_eq!(linked, 2);
    assert_eq!(linked_again, 0);
    assert!(pbxproj.contains("liblibPDRCore.a in Frameworks"));
    assert!(pbxproj.contains("libcoreSupport.a in Frameworks"));
    assert!(pbxproj.contains("../SDK/Libs/liblibPDRCore.a"));
    assert!(pbxproj.contains("../SDK/Libs/libcoreSupport.a"));
    assert_eq!(pbxproj.matches("liblibPDRCore.a in Frameworks").count(), 2);
    assert_eq!(pbxproj.matches("libcoreSupport.a in Frameworks").count(), 2);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pod_config_renders_manifest_business_values() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-pod-config-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": { "Payment": {}, "Map": {}, "Statistic": {}, "UniAD": {} },
            "distribute": {
                "ios": {
                    "marketChannel": "io.dcloud.HBuilder|__UNI__POD|adid|apple",
                    "dcloudAdId": "adid"
                },
                "sdkConfigs": {
                    "payment": {
                        "weixin": {
                            "__platform__": ["ios"],
                            "appid": "wx-app",
                            "UniversalLinks": "https://example.com/app/"
                        },
                        "alipay": {
                            "__platform__": ["ios"],
                            "appid": "20240001"
                        }
                    },
                    "maps": {
                        "amap": {
                            "__platform__": ["ios"],
                            "appkey_ios": "amap-key"
                        }
                    },
                    "statistic": {
                        "umeng": {
                            "__platform__": ["ios"],
                            "appkey_ios": "umeng-key",
                            "channelid_ios": "App Store"
                        }
                    }
                }
            }
        }
    });
    let info = parse_manifest(&root, manifest);
    let content = super::super::pod_config::render_ios_pod_config(Some(&info));

    assert!(content.contains("payment_wechat"));
    assert!(content.contains("appid: 'wx-app'"));
    assert!(content.contains("universal_links: 'https://example.com/app/'"));
    assert!(content.contains("payment_alipay"));
    assert!(content.contains("scheme: 'ap20240001'"));
    assert!(content.contains("map_gaode"));
    assert!(content.contains("appkey: 'amap-key'"));
    assert!(content.contains("statistic_umeng"));
    assert!(content.contains("channel: 'App Store'"));
    assert!(content.contains("market_channel: 'io.dcloud.HBuilder|__UNI__POD|adid|apple'"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pod_subspecs_follow_manifest_and_uts_scan() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-pod-subspecs-{}", uuid::Uuid::new_v4()));
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Payment": {},
                "Map": {},
                "Statistic": {},
                "UniAD": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "payment": {
                        "weixin": { "__platform__": ["ios"], "appid": "wx-app" },
                        "alipay": { "__platform__": ["ios"], "appid": "ali-app" }
                    },
                    "maps": {
                        "amap": { "__platform__": ["ios"], "appkey_ios": "amap-key" }
                    },
                    "statistic": {
                        "umeng": { "__platform__": ["ios"], "appkey_ios": "umeng-key" }
                    },
                    "ad": {
                        "gdt": { "__platform__": ["ios"], "appid": "gdt-app" }
                    }
                }
            }
        }
    });
    let info = parse_manifest(&root, manifest);
    let mut scan = empty_scan(&root);
    scan.uts.has_uts_plugins = true;
    scan.uts.has_ios_uts_plugins = true;
    scan.uts.custom_plugins.push(UtsCustomPlugin {
        id: "uts-demo".into(),
        android_dir: None,
        ios_dir: Some(
            root.join("uni_modules/uts-demo/utssdk/app-ios")
                .to_string_lossy()
                .to_string(),
        ),
        android_deps: Vec::new(),
        ios_frameworks: Vec::new(),
        ios_system_frameworks: Vec::new(),
        ios_plists: Default::default(),
        ios_provider: None,
        ios_dependencies_pods: Default::default(),
        abis: None,
        min_sdk_version: None,
        dependencies: Vec::new(),
        components: Vec::new(),
        hooks_class: None,
        gradle_plugins: Vec::new(),
        project_dependencies: Vec::new(),
    });

    let subspecs = super::super::pod_subspecs::resolve_ios_pod_subspecs(Some(&info), &scan);

    assert_eq!(subspecs.values.first().map(String::as_str), Some("Core"));
    for expected in [
        "Payment-AliPay",
        "Payment-Wechat",
        "Map-Gaode",
        "Statistic-Umeng",
        "UTS",
        "UniAd-GDT",
    ] {
        assert!(
            subspecs.values.iter().any(|value| value == expected),
            "{expected}"
        );
    }
    assert!(subspecs.warnings.is_empty());
    let _ = std::fs::remove_dir_all(root);
}
