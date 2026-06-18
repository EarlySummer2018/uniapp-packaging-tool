use ::plist as plist_crate;

use super::super::entitlements::patch_ios_entitlements;

#[test]
fn ios_manifest_universal_links_are_written_to_entitlements() {
    let root =
        std::env::temp_dir().join(format!("unipack-ios-entitlements-{}", uuid::Uuid::new_v4()));
    let project_file = root.join("HBuilder-Hello.xcodeproj");
    let entitlements = root.join("HBuilder/HBuilder.entitlements");
    std::fs::create_dir_all(&project_file).unwrap();
    std::fs::create_dir_all(entitlements.parent().unwrap()).unwrap();
    plist_crate::Value::Dictionary(plist_crate::Dictionary::new())
        .to_file_xml(&entitlements)
        .unwrap();
    std::fs::write(
        project_file.join("project.pbxproj"),
        "CODE_SIGN_ENTITLEMENTS = HBuilder/HBuilder.entitlements;",
    )
    .unwrap();
    let manifest = serde_json::json!({
        "app-plus": {
            "distribute": {
                "ios": {
                    "capabilities": {
                        "entitlements": {
                            "com.apple.developer.associated-domains": [
                                "applinks:www.hubeijianmeishiye.cn"
                            ]
                        }
                    }
                },
                "sdkConfigs": {
                    "share": {
                        "weixin": {
                            "UniversalLinks": "https://example.com/apple-app-site-association/"
                        }
                    },
                    "payment": {
                        "weixin": {
                            "UniversalLinks": "https://example.com/pay/"
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

    assert_eq!(
        patch_ios_entitlements(&root, &project_file, Some(&info)).unwrap(),
        2
    );
    let value = plist_crate::Value::from_file(&entitlements).unwrap();
    let domains = value
        .as_dictionary()
        .unwrap()
        .get("com.apple.developer.associated-domains")
        .and_then(plist_crate::Value::as_array)
        .unwrap();
    assert!(domains.contains(&plist_crate::Value::String(
        "applinks:www.hubeijianmeishiye.cn".into()
    )));
    assert!(domains.contains(&plist_crate::Value::String("applinks:example.com".into())));
    let _ = std::fs::remove_dir_all(root);
}
