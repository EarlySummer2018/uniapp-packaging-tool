use super::{test_context, write_official_like_project};
use crate::commands::android::project_mod::{AndroidProjectModifier, MODULE_NAME};

#[test]
fn official_project_patch_is_idempotent_without_template_markers() {
    let workspace =
        std::env::temp_dir().join(format!("unipack-android-mod-{}", uuid::Uuid::new_v4()));
    write_official_like_project(&workspace);
    let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
    let ctx = test_context();

    modifier.apply_all_modifications(&ctx).unwrap();
    modifier.apply_all_modifications(&ctx).unwrap();

    let build_gradle =
        std::fs::read_to_string(workspace.join(MODULE_NAME).join("build.gradle")).unwrap();
    assert!(build_gradle.contains("namespace 'com.example.test'"));
    assert!(build_gradle.contains("compileSdkVersion 36"));
    assert!(build_gradle.contains("applicationId \"com.example.test\""));
    assert!(build_gradle.contains("minSdkVersion 21"));
    assert!(build_gradle.contains("targetSdkVersion 34"));
    assert!(build_gradle.contains("versionCode 178"));
    assert!(build_gradle.contains("versionName \"1.7.8\""));
    assert!(build_gradle.contains("abiFilters 'arm64-v8a', 'armeabi-v7a'"));
    assert!(build_gradle.contains(
        "buildConfigField 'String[]', 'UTSHooksClassArray', '{\"uts.sdk.modules.demo.DemoHook\"}'"
    ));
    assert!(build_gradle.contains("signingConfig signingConfigs.release"));
    assert!(build_gradle.contains("storeFile file('/tmp/test-release.keystore')"));
    assert_eq!(
        build_gradle
            .matches("implementation project(':demo-plugin')")
            .count(),
        1
    );
    assert_eq!(build_gradle.matches("manifestPlaceholders").count(), 1);

    let strings = std::fs::read_to_string(
        workspace
            .join(MODULE_NAME)
            .join("src/main/res/values/strings.xml"),
    )
    .unwrap();
    assert!(strings.contains(r#"<string name="facebook_app_id">123456</string>"#));
    assert!(strings.contains(r#"<string name="fb_login_protocol_scheme">fb123456</string>"#));
    assert!(strings.contains(r#"<string name="facebook_client_token">client-token</string>"#));

    let root_gradle = std::fs::read_to_string(workspace.join("build.gradle")).unwrap();
    assert!(root_gradle.contains("classpath 'com.android.tools.build:gradle:8.10.0'"));
    assert!(root_gradle.contains("androidxVersion = '1.0.0'"));
    assert!(root_gradle.contains("classpath 'com.example:demo-gradle-plugin:1.0.0'"));
    assert!(!root_gradle.contains("allprojects"));

    let manifest = std::fs::read_to_string(
        workspace
            .join(MODULE_NAME)
            .join("src/main/AndroidManifest.xml"),
    )
    .unwrap();
    assert!(manifest.contains(r#"android:allowBackup="false""#));
    assert!(manifest.contains(r#"xmlns:tools="http://schemas.android.com/tools""#));
    assert!(manifest.contains(r#"tools:replace="android:allowBackup""#));
    assert_eq!(manifest.matches("android:allowBackup").count(), 2);
    assert!(manifest.contains(r#"android:value="test-app-key""#));
    assert_eq!(
        manifest
            .matches("android.permission.ACCESS_BACKGROUND_LOCATION")
            .count(),
        1
    );
    assert_eq!(manifest.matches(r#"android:name="GETUI_APPID""#).count(), 1);
    assert_eq!(manifest.matches(r#"android:scheme="unipush""#).count(), 1);
    // 验证权限被正确包裹为 <uses-permission> 标签（不是裸字符串）
    assert!(
        manifest.contains(
            r#"<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION" />"#
        ),
        "权限应被包裹在 <uses-permission> 标签中"
    );
    // 验证 XML 格式未被压成单行（set_application_attr 不应重写整个文档）
    assert!(
        manifest.contains('\n'),
        "AndroidManifest.xml 应保留换行格式"
    );

    let settings = std::fs::read_to_string(workspace.join("settings.gradle")).unwrap();
    assert!(settings.contains("rootProject.name = 'Test App'"));
    assert_eq!(settings.matches("include ':demo-plugin'").count(), 1);

    let dcloud = std::fs::read_to_string(
        workspace
            .join(MODULE_NAME)
            .join("src/main/assets/data/dcloud_control.xml"),
    )
    .unwrap();
    assert!(dcloud.contains(r#"appid="__UNI__TEST""#));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn manifest_android_distribute_fields_are_applied_with_excludes_taking_priority() {
    let workspace = std::env::temp_dir().join(format!(
        "unipack-android-distribute-{}",
        uuid::Uuid::new_v4()
    ));
    write_official_like_project(&workspace);
    let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
    let mut ctx = test_context();
    ctx.android_abi_filters = vec!["arm64-v8a".to_string()];
    ctx.android_permissions = vec![
        r#"<uses-permission android:name="android.permission.INTERNET" />"#.to_string(),
        r#"<uses-feature android:name="android.hardware.camera" />"#.to_string(),
    ];
    ctx.android_exclude_permissions = vec![
        r#"<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION" />"#
            .to_string(),
        "android.hardware.camera".to_string(),
    ];
    ctx.android_schemes = vec!["comchatvivaus".to_string()];

    modifier.apply_all_modifications(&ctx).unwrap();
    modifier.apply_all_modifications(&ctx).unwrap();

    let build_gradle =
        std::fs::read_to_string(workspace.join(MODULE_NAME).join("build.gradle")).unwrap();
    assert!(build_gradle.contains("abiFilters 'arm64-v8a'"));
    assert!(!build_gradle.contains("armeabi-v7a"));

    let manifest = std::fs::read_to_string(
        workspace
            .join(MODULE_NAME)
            .join("src/main/AndroidManifest.xml"),
    )
    .unwrap();
    assert_eq!(manifest.matches("android.permission.INTERNET").count(), 1);
    assert!(!manifest.contains("android.permission.ACCESS_BACKGROUND_LOCATION"));
    assert!(!manifest.contains("android.hardware.camera"));
    assert_eq!(
        manifest
            .matches(r#"android:scheme="comchatvivaus""#)
            .count(),
        1
    );
    assert!(manifest.contains(r#"android.intent.category.BROWSABLE"#));

    let _ = std::fs::remove_dir_all(workspace);
}
