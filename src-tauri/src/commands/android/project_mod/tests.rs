//! android_project_mod 单元测试。

#[cfg(test)]
mod tests {
    use crate::commands::android::project_mod::manifest::fix_manifest_xml_structure;
    use crate::commands::android::project_mod::*;
    use std::path::{Path, PathBuf};

    fn test_context() -> BuildModificationContext {
        BuildModificationContext {
            project_name: "Test App".to_string(),
            package_name: "com.example.test".to_string(),
            appid: "__UNI__TEST".to_string(),
            dcloud_appkey: "test-app-key".to_string(),
            app_name: "Test App".to_string(),
            version_code: 178,
            version_name: "1.7.8".to_string(),
            compile_sdk: 35,
            target_sdk: 34,
            min_sdk: 21,
            keystore_path: "/tmp/test-release.keystore".to_string(),
            key_alias: "release".to_string(),
            key_password: "keypass".to_string(),
            store_password: "storepass".to_string(),
            android_allow_backup: "false".to_string(),
            extra_repositories: vec!["maven { url 'https://jitpack.io' }".to_string()],
            extra_dependencies: vec!["implementation 'androidx.core:core:1.12.0'".to_string()],
            plugin_includes: vec![
                "include ':demo-plugin'\nproject(':demo-plugin').projectDir = file('uts-modules/demo-plugin')"
                    .to_string(),
            ],
            plugin_project_dependencies: vec![
                "implementation project(':demo-plugin')".to_string(),
            ],
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
        assert!(build_gradle.contains("compileSdkVersion 35"));
        assert!(build_gradle.contains("applicationId \"com.example.test\""));
        assert!(build_gradle.contains("minSdkVersion 21"));
        assert!(build_gradle.contains("targetSdkVersion 34"));
        assert!(build_gradle.contains("versionCode 178"));
        assert!(build_gradle.contains("versionName \"1.7.8\""));
        assert!(build_gradle.contains("signingConfig signingConfigs.release"));
        assert!(build_gradle.contains("storeFile file('/tmp/test-release.keystore')"));
        assert_eq!(
            build_gradle
                .matches("implementation project(':demo-plugin')")
                .count(),
            1
        );
        assert_eq!(build_gradle.matches("manifestPlaceholders").count(), 1);

        let manifest = std::fs::read_to_string(
            workspace
                .join(MODULE_NAME)
                .join("src/main/AndroidManifest.xml"),
        )
        .unwrap();
        assert!(manifest.contains(r#"android:allowBackup="false""#));
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
            manifest.contains(r#"<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION" />"#),
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
    fn huawei_push_injects_agconnect_gradle_plugin() {
        let workspace =
            std::env::temp_dir().join(format!("unipack-huawei-push-{}", uuid::Uuid::new_v4()));
        write_official_like_project(&workspace);
        let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
        let mut ctx = test_context();
        ctx.extra_repositories
            .push("maven { url 'https://developer.huawei.com/repo/' }".to_string());
        ctx.extra_dependencies
            .push("implementation 'com.huawei.hms:push:6.11.0.300'".to_string());

        modifier.apply_all_modifications(&ctx).unwrap();

        let root_gradle = std::fs::read_to_string(workspace.join("build.gradle")).unwrap();
        assert!(root_gradle.contains("maven { url 'https://developer.huawei.com/repo/' }"));
        assert!(root_gradle.contains("classpath 'com.huawei.agconnect:agcp:1.9.1.301'"));

        let app_gradle =
            std::fs::read_to_string(workspace.join(MODULE_NAME).join("build.gradle")).unwrap();
        assert!(app_gradle.contains("apply plugin: 'com.huawei.agconnect'"));
        assert!(app_gradle.contains("implementation 'com.huawei.hms:push:6.11.0.300'"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn extra_repositories_keep_default_dependency_repositories_without_plugins() {
        let workspace =
            std::env::temp_dir().join(format!("unipack-extra-repos-{}", uuid::Uuid::new_v4()));
        write_official_like_project(&workspace);
        let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
        let mut ctx = test_context();
        ctx.plugin_includes.clear();
        ctx.plugin_project_dependencies.clear();

        modifier.apply_all_modifications(&ctx).unwrap();
        modifier.apply_all_modifications(&ctx).unwrap();

        let settings = std::fs::read_to_string(workspace.join("settings.gradle")).unwrap();
        assert!(settings.contains("dependencyResolutionManagement"));
        assert_eq!(settings.matches("google()").count(), 1);
        assert_eq!(settings.matches("mavenCentral()").count(), 1);
        assert_eq!(settings.matches("https://jitpack.io").count(), 1);

        let root_gradle = std::fs::read_to_string(workspace.join("build.gradle")).unwrap();
        assert!(root_gradle.contains("google()"));
        assert_eq!(root_gradle.matches("mavenCentral()").count(), 1);
        assert_eq!(root_gradle.matches("https://jitpack.io").count(), 1);

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn local_downloaded_official_project_can_be_patched_when_present() {
        let sdk_root =
            PathBuf::from("/Users/huangxiangrui/Downloads/5.07/Android-SDK@5.07.82603_20260414");
        let source = sdk_root.join("HBuilder-Integrate-AS");
        if !source.exists() {
            return;
        }

        let workspace =
            std::env::temp_dir().join(format!("unipack-android-real-mod-{}", uuid::Uuid::new_v4()));
        crate::utils::fs::copy_recursive(&source, &workspace).unwrap();
        let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
        let ctx = test_context();

        modifier.apply_all_modifications(&ctx).unwrap();
        modifier.apply_all_modifications(&ctx).unwrap();

        let build_gradle =
            std::fs::read_to_string(workspace.join(MODULE_NAME).join("build.gradle")).unwrap();
        assert!(build_gradle.contains("namespace 'com.example.test'"));
        assert!(build_gradle.contains("applicationId \"com.example.test\""));
        assert_eq!(
            build_gradle
                .matches("implementation project(':demo-plugin')")
                .count(),
            1
        );

        let manifest = std::fs::read_to_string(
            workspace
                .join(MODULE_NAME)
                .join("src/main/AndroidManifest.xml"),
        )
        .unwrap();
        assert!(manifest.contains(r#"android:allowBackup="false""#));
        assert!(manifest.contains(r#"android:value="test-app-key""#));
        assert_eq!(manifest.matches(r#"android:name="GETUI_APPID""#).count(), 1);

        let root_gradle = std::fs::read_to_string(workspace.join("build.gradle")).unwrap();
        assert_eq!(root_gradle.matches("https://jitpack.io").count(), 1);

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn fix_manifest_xml_structure_passes_through_well_formed_xml() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application android:label="Test">
        <activity android:name=".MainActivity" android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
            </intent-filter>
        </activity>
        <meta-data android:name="key" android:value="val" />
    </application>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        assert_eq!(result, xml, "格式正确的 XML 不应被修改");
    }

    #[test]
    fn fix_manifest_xml_structure_auto_closes_unclosed_activity() {
        let xml = r#"<manifest>
    <application>
        <activity android:name=".Main">
    </application>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        assert!(
            result.contains("</activity>"),
            "修复后应包含自动补全的 </activity>"
        );
        // 验证修复后的 XML 可以再次通过校验（幂等性）
        let re_check = fix_manifest_xml_structure(&result).unwrap();
        assert_eq!(re_check, result, "修复结果应幂等，二次调用不再修改");
    }

    #[test]
    fn fix_manifest_xml_structure_fixes_mismatched_tags() {
        // 交叉嵌套：<manifest><a><b></a></b></manifest>
        let xml = r#"<manifest>
    <a><b></a></b>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        // 应在 </a> 前插入 </b>，在末尾补 </a>
        assert!(result.contains("</b>"), "应补全缺失的 </b>");
        // 验证幂等性
        let re_check = fix_manifest_xml_structure(&result).unwrap();
        assert_eq!(re_check, result, "修复结果应幂等");
    }

    #[test]
    fn fix_manifest_xml_structure_preserves_self_closing_tags() {
        let xml = r#"<manifest>
    <application>
        <meta-data android:name="k" android:value="v" />
        <uses-permission android:name="p" />
    </application>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        assert_eq!(result, xml, "含自闭合标签的正确 XML 不应被修改");
    }
}
