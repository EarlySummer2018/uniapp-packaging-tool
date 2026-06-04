//! Android 构建环境解析

use crate::commands::android::types::AndroidBuildEnvironment;

pub fn resolve_android_build_environment() -> Result<AndroidBuildEnvironment, String> {
    let gradle_bin = crate::commands::shared::env::resolve_configured_tool_bin_with_candidates(
        "gradle",
        gradle_bin_names(),
    )?;
    let java_bin = crate::commands::shared::env::resolve_configured_tool_bin_with_candidates(
        "java",
        java_bin_names(),
    )?;
    let android_home = crate::commands::shared::env::require_configured_tool_path("android_sdk")?;
    let java_home = java_bin
        .parent()
        .and_then(|bin| bin.parent())
        .ok_or_else(|| {
            format!(
                "无法从 Java 可执行文件推导 JAVA_HOME: {}",
                java_bin.display()
            )
        })?
        .to_path_buf();
    let gradle_user_home = crate::utils::fs::get_unipack_home().join("gradle-home");
    crate::utils::fs::ensure_directory(&gradle_user_home)
        .map_err(|e| format!("创建 Gradle 用户目录失败: {}", e))?;

    Ok(AndroidBuildEnvironment {
        gradle_bin,
        java_home,
        android_home,
        gradle_user_home,
    })
}

pub fn android_process_env(env: &AndroidBuildEnvironment) -> Vec<(String, String)> {
    vec![
        (
            "JAVA_HOME".to_string(),
            env.java_home.to_string_lossy().to_string(),
        ),
        (
            "ANDROID_HOME".to_string(),
            env.android_home.to_string_lossy().to_string(),
        ),
        (
            "ANDROID_SDK_ROOT".to_string(),
            env.android_home.to_string_lossy().to_string(),
        ),
        (
            "GRADLE_USER_HOME".to_string(),
            env.gradle_user_home.to_string_lossy().to_string(),
        ),
    ]
}

pub fn gradle_bin_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["gradle.bat", "gradle"]
    } else {
        &["gradle"]
    }
}

pub fn java_bin_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["java.exe", "java"]
    } else {
        &["java"]
    }
}

pub fn find_apk_in_workspace(workspace: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    find_apks_recursive(
        &workspace
            .join(crate::utils::android_project_mod::MODULE_NAME)
            .join("build/outputs"),
        &mut results,
    );
    results.sort_by(|a, b| {
        let time_a = a.metadata().ok().and_then(|m| m.modified().ok());
        let time_b = b.metadata().ok().and_then(|m| m.modified().ok());
        time_b.cmp(&time_a)
    });
    results
}

fn find_apks_recursive(dir: &std::path::Path, results: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_apks_recursive(&path, results);
            } else if path.extension().map(|e| e == "apk").unwrap_or(false) {
                results.push(path);
            }
        }
    }
}

pub fn expand_home(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(rest);
    }
    std::path::PathBuf::from(path)
}

pub fn safe_file_name(value: &str) -> String {
    let cleaned = value.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    if cleaned.trim().is_empty() {
        "UniApp".to_string()
    } else {
        cleaned
    }
}

pub fn android_build_requires_allow_backup_false(
    extra_dependencies: &std::collections::BTreeSet<String>,
) -> bool {
    extra_dependencies
        .iter()
        .any(|dep| dep == "com.getui:gysdk:3.1.7.0")
}
