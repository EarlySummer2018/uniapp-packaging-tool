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
    assert!(!content.contains("source 'https://github.com/CocoaPods/Specs.git'"));
    assert!(!content.contains("source 'https://github.com/volcengine/volcengine-specs.git'"));
    assert!(content.contains("project 'HBuilder-Hello.xcodeproj'"));
    assert!(content.contains("use_frameworks! :linkage => :static"));
    assert!(content.contains("require_relative 'scripts/uniapp_module_config'"));
    assert!(content.contains("require_relative 'scripts/uniapp_uts_plugins'"));
    assert!(content.contains("pod 'uniapp', :path => '..', :subspecs => uniapp_subspecs"));
    assert!(content.contains("UniAppUTSPlugins.prepare!("));
    assert!(content.contains("unipack_normalize_uts_plugin_podspecs!(uts_plugins)"));
    assert!(content.contains("s.homepage = 'https://dcloud.io/'"));
    assert!(content.contains("s.license = { :type => 'DCloud' }"));
    assert!(content.contains("s.authors = { 'DCloud' => 'https://dcloud.io/' }"));
    assert!(content.contains("pod plugin[:pod_name], :path => plugin[:pod_path]"));
    assert!(content.contains("uts_plugins: uts_plugins"));
    assert!(content.contains("'Core'"));
    assert!(content.contains("'Payment-Wechat'"));
    assert!(content.contains("'Map-Gaode'"));
}

#[test]
fn podfile_adds_official_sources_for_uni_ad() {
    let root = PathBuf::from("/tmp/HBuilder-Hello");
    let project = root.join("HBuilder-Hello.xcodeproj");
    let content =
        super::super::pod::render_ios_podfile(&project, &["Core".into(), "UniAd-GDT".into()])
            .unwrap();

    assert!(content.contains("source 'https://github.com/CocoaPods/Specs.git'"));
    assert!(content.contains("source 'https://github.com/volcengine/volcengine-specs.git'"));
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
fn pod_integration_verification_accepts_cocoapods_markers() {
    let root = std::env::temp_dir().join(format!("unipack-ios-pod-check-{}", uuid::Uuid::new_v4()));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let support = project_root.join("Pods/Target Support Files/Pods-HBuilder");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&support).unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        r#"
10C3DAEF31AC8F49429C9BF2 /* Pods_HBuilder.framework in Frameworks */ = {isa = PBXBuildFile; };
baseConfigurationReference = CCB08588010615606572E49E /* Pods-HBuilder.release.xcconfig */;
shellScript = ""${PODS_ROOT}/Target Support Files/Pods-HBuilder/Pods-HBuilder-frameworks.sh"\n";
shellScript = ""${PODS_ROOT}/Target Support Files/Pods-HBuilder/Pods-HBuilder-resources.sh"\n";
"#,
    )
    .unwrap();
    std::fs::write(
        support.join("Pods-HBuilder.release.xcconfig"),
        r#"HEADER_SEARCH_PATHS = $(inherited) "${PODS_ROOT}/Headers/Public/uniapp"
OTHER_LDFLAGS = $(inherited) -framework "DCUniBase"
"#,
    )
    .unwrap();

    super::super::pod::verify_cocoapods_project_integration(&project_root, &project_file).unwrap();
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn pod_integration_verification_rejects_missing_cocoapods_markers() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-pod-check-missing-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let support = project_root.join("Pods/Target Support Files/Pods-HBuilder");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(&support).unwrap();
    std::fs::write(project_file.join("project.pbxproj"), "/* empty */").unwrap();
    std::fs::write(support.join("Pods-HBuilder.release.xcconfig"), "").unwrap();

    let err = super::super::pod::verify_cocoapods_project_integration(&project_root, &project_file)
        .unwrap_err();

    assert!(err.contains("Pods_HBuilder.framework"));
    assert!(err.contains("DCUniBase"));
    assert!(err.contains(".xcworkspace"));
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
