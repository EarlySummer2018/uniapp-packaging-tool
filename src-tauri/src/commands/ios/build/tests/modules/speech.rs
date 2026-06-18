use crate::commands::ios::modules::speech::apply_ios_speech_module;

#[test]
fn ios_speech_ignores_local_pod_and_requires_offline_sdk_files() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-speech-local-pod-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Speech": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "speech": {
                        "localPod": true,
                        "baidu": {
                            "appid": "baidu-speech"
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

    let error = apply_ios_speech_module(&project_root, &project_file, Some(&info)).unwrap_err();

    assert!(error.contains("iOS 语音输入模块缺少 SDK 依赖文件"));
    assert!(error.contains("liblibSpeech.a"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn ios_speech_ifly_auto_migration_requires_ifly_framework() {
    let root = std::env::temp_dir().join(format!(
        "unipack-ios-speech-ifly-manual-{}",
        uuid::Uuid::new_v4()
    ));
    let project_root = root.join("HBuilder-Hello");
    let project_file = project_root.join("HBuilder-Hello.xcodeproj");
    let libs_dir = root.join("SDK/Libs");
    std::fs::create_dir_all(&libs_dir).unwrap();
    std::fs::write(libs_dir.join("liblibSpeech.a"), "lib").unwrap();
    let manifest = serde_json::json!({
        "app-plus": {
            "modules": {
                "Speech": {}
            },
            "distribute": {
                "sdkConfigs": {
                    "speech": {
                        "ifly": {
                            "__platform__": ["ios"],
                            "appid": "ifly-app-id"
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

    let error = apply_ios_speech_module(&project_root, &project_file, Some(&info)).unwrap_err();

    assert!(error.contains("iOS 语音输入模块缺少 SDK 依赖文件"));
    assert!(error.contains("iflyMSC.framework"));
    let _ = std::fs::remove_dir_all(root);
}
