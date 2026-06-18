//! android_project_mod 单元测试。

use std::path::Path;

use crate::commands::android::project_mod::BuildModificationContext;

mod gradle;
mod manifest_xml;
mod project_patch;
mod xml_editor;

fn test_context() -> BuildModificationContext {
    BuildModificationContext {
        project_name: "Test App".to_string(),
        package_name: "com.example.test".to_string(),
        appid: "__UNI__TEST".to_string(),
        dcloud_appkey: "test-app-key".to_string(),
        app_name: "Test App".to_string(),
        string_resources: vec![
            ("facebook_app_id".to_string(), "123456".to_string()),
            (
                "fb_login_protocol_scheme".to_string(),
                "fb123456".to_string(),
            ),
            (
                "facebook_client_token".to_string(),
                "client-token".to_string(),
            ),
        ],
        version_code: 178,
        version_name: "1.7.8".to_string(),
        compile_sdk: 36,
        target_sdk: 34,
        min_sdk: 21,
        keystore_path: "/tmp/test-release.keystore".to_string(),
        key_alias: "release".to_string(),
        key_password: "keypass".to_string(),
        store_password: "storepass".to_string(),
        android_allow_backup: "false".to_string(),
        androidx_version: Some("1.0.0".to_string()),
        extra_repositories: vec!["maven { url 'https://jitpack.io' }".to_string()],
        extra_dependencies: vec!["implementation 'androidx.core:core:1.12.0'".to_string()],
        project_buildscript_dependencies: vec![
            "classpath 'com.example:demo-gradle-plugin:1.0.0'".to_string(),
        ],
        plugin_includes: vec![
            "include ':demo-plugin'\nproject(':demo-plugin').projectDir = file('uts-modules/demo-plugin')"
                .to_string(),
        ],
        plugin_project_dependencies: vec![
            "implementation project(':demo-plugin')".to_string(),
        ],
        uts_abi_filters: vec!["armeabi-v7a".to_string(), "arm64-v8a".to_string()],
        android_abi_filters: vec![],
        android_permissions: vec![],
        android_exclude_permissions: vec![],
        android_schemes: vec![],
        uts_hooks_classes: vec!["uts.sdk.modules.demo.DemoHook".to_string()],
        module_permissions: vec![
            r#"<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION" />"#
                .to_string(),
        ],
        module_application_entries: vec![
            r#"<meta-data android:name="GETUI_APPID" android:value="${GETUI_APPID}" />"#.to_string(),
        ],
        module_pandora_entry_intent_filters: vec![
            r#"<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:host="io.dcloud.unipush" android:path="/" android:scheme="unipush" />
</intent-filter>"#
                .to_string(),
        ],
        manifest_placeholders:
            "\n        manifestPlaceholders = [\n            \"GETUI_APPID\": \"demo\"\n        ]"
                .to_string(),
        dependency_excludes: String::new(),
        module_patch_groups: vec![],
    }
}

fn write_official_like_project(workspace: &Path) {
    std::fs::create_dir_all(workspace.join("simpleDemo/src/main/assets/data")).unwrap();
    std::fs::create_dir_all(workspace.join("simpleDemo/src/main/res/values")).unwrap();
    std::fs::write(workspace.join("settings.gradle"), "include ':simpleDemo'\n").unwrap();
    std::fs::write(
        workspace.join("build.gradle"),
        r#"buildscript {
    repositories {
        google()
    }
    dependencies {
        classpath 'com.android.tools.build:gradle:8.7.3'
    }
}

allprojects {
    repositories {
        google()
    }
}
"#,
    )
    .unwrap();
    std::fs::write(
        workspace.join("simpleDemo/build.gradle"),
        r#"apply plugin: 'com.android.application'

android {
    compileSdkVersion 35
    buildToolsVersion '35.0.0'
    namespace 'com.android.simple'
    defaultConfig {
        applicationId "com.android.simple"
        minSdkVersion 21
        targetSdkVersion 33
        versionCode 1
        versionName "1.0"
        multiDexEnabled true
    }
    signingConfigs {
        config {
            keyAlias 'key0'
            keyPassword '123456'
            storeFile file('test.jks')
            storePassword '123456'
        }
    }
    buildTypes {
        debug {
            signingConfig signingConfigs.config
        }
        release {
            signingConfig signingConfigs.config
        }
    }
}

dependencies {
    implementation fileTree(dir: 'libs', include: ['*.aar', '*.jar'], exclude: [])
}
"#,
    )
    .unwrap();
    std::fs::write(
        workspace.join("simpleDemo/src/main/AndroidManifest.xml"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application
        android:allowBackup="true"
        android:label="@string/app_name">
        <activity
            android:name="io.dcloud.PandoraEntryActivity"
            android:exported="true">
        </activity>
        <meta-data
            android:name="dcloud_appkey"
            android:value="placeholder" />
    </application>
</manifest>
"#,
    )
    .unwrap();
    std::fs::write(
        workspace.join("simpleDemo/src/main/res/values/strings.xml"),
        r#"<resources>
    <string name="app_name">UniApp</string>
</resources>
"#,
    )
    .unwrap();
    std::fs::write(
        workspace.join("simpleDemo/src/main/assets/data/dcloud_control.xml"),
        r#"<hbuilder>
<apps>
    <app appid="__UNI__A" appver=""/>
</apps>
</hbuilder>
"#,
    )
    .unwrap();
}
