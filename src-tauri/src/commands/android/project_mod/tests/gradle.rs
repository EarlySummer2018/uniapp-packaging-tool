use super::{test_context, write_official_like_project};
use crate::commands::android::project_mod::gradle::{
    ensure_android_gradle_plugin_supports_kotlin_22, set_or_insert_androidx_version_extra,
    set_or_insert_root_project_name,
};
use crate::commands::android::project_mod::{AndroidProjectModifier, MODULE_NAME};

#[test]
fn android_gradle_plugin_is_raised_for_kotlin_22_metadata() {
    let old = r#"buildscript {
    dependencies {
        classpath 'com.android.tools.build:gradle:8.7.3'
    }
}
"#;
    let patched = ensure_android_gradle_plugin_supports_kotlin_22(old);
    assert!(patched.contains("classpath 'com.android.tools.build:gradle:8.10.0'"));

    let current = r#"buildscript {
    dependencies {
        classpath 'com.android.tools.build:gradle:8.12.0'
    }
}
"#;
    let unchanged = ensure_android_gradle_plugin_supports_kotlin_22(current);
    assert!(unchanged.contains("classpath 'com.android.tools.build:gradle:8.12.0'"));
}

#[test]
fn root_gradle_missing_androidx_version_gets_configured_extra_property() {
    let root_gradle = r#"buildscript {
    repositories {
        google()
    }
}
"#;
    let patched = set_or_insert_androidx_version_extra(root_gradle, "1.6.1");
    assert!(patched.contains("androidxVersion = '1.6.1'"));

    let patched_twice = set_or_insert_androidx_version_extra(&patched, "1.6.1");
    assert_eq!(patched_twice.matches("androidxVersion").count(), 1);
}

#[test]
fn existing_androidx_version_extra_property_is_updated() {
    let root_gradle = r#"ext {
    androidxVersion = '1.6.1'
}
"#;
    let patched = set_or_insert_androidx_version_extra(root_gradle, "1.7.0");
    assert!(patched.contains("androidxVersion = '1.7.0'"));
    assert!(!patched.contains("androidxVersion = '1.6.1'"));
}

#[test]
fn root_project_name_is_inserted_after_complete_plugin_management_block() {
    let settings = r#"pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.PREFER_SETTINGS)
    repositories {
        google()
        mavenCentral()
    }
}

include ':simpleDemo'
"#;

    let patched = set_or_insert_root_project_name(settings, "Demo App");
    let name_idx = patched.find("rootProject.name = 'Demo App'").unwrap();
    let dependency_idx = patched.find("dependencyResolutionManagement").unwrap();

    assert!(name_idx < dependency_idx);
    assert!(patched.contains("gradlePluginPortal()\n    }\n}\nrootProject.name"));
    assert!(!patched.contains("gradlePluginPortal()\n    }\nrootProject.name"));
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
fn google_oauth_injects_google_services_buildscript_configuration() {
    let workspace =
        std::env::temp_dir().join(format!("unipack-google-oauth-{}", uuid::Uuid::new_v4()));
    write_official_like_project(&workspace);
    std::fs::write(
        workspace.join("build.gradle"),
        r#"buildscript {
    repositories {
        mavenCentral()
    }
    dependencies {
        classpath 'com.android.tools.build:gradle:8.7.3'
    }
}
"#,
    )
    .unwrap();
    let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
    let mut ctx = test_context();
    ctx.project_buildscript_dependencies =
        vec!["classpath 'com.google.gms:google-services:4.2.0'".to_string()];

    modifier.apply_all_modifications(&ctx).unwrap();

    let root_gradle = std::fs::read_to_string(workspace.join("build.gradle")).unwrap();
    assert!(root_gradle.contains("google()"));
    assert!(root_gradle.contains("classpath 'com.google.gms:google-services:4.2.0'"));

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
    assert!(root_gradle.contains("classpath 'com.android.tools.build:gradle:8.10.0'"));
    assert!(!root_gradle.contains("allprojects"));
    assert_eq!(root_gradle.matches("mavenCentral()").count(), 0);
    assert_eq!(root_gradle.matches("https://jitpack.io").count(), 0);

    let _ = std::fs::remove_dir_all(workspace);
}
