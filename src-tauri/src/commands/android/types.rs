//! Android 构建类型定义

use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidBuildOptions {
    pub project_path: String,
    pub variant: Option<String>,
    pub clean: Option<bool>,
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub logs: Vec<String>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildArtifact {
    pub platform: String,
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub build_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildLogEvent {
    pub build_id: Option<String>,
    pub platform: String,
    pub level: String,
    pub message: String,
    pub progress: Option<u8>,
}

pub struct AppState {}

pub const UTS_RUNTIME_DEPS: &[&str] = &[
    "com.squareup.okhttp3:okhttp:3.12.12",
    "androidx.core:core-ktx:1.6.0",
    "org.jetbrains.kotlin:kotlin-stdlib:2.2.0",
    "org.jetbrains.kotlin:kotlin-reflect:2.2.0",
    "org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1",
    "org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1",
    "com.github.getActivity:XXPermissions:18.63",
];

// DCloud Android SDK 5.07 ships UTS runtime artifacts with Kotlin 2.2 metadata.
// The UTS Android library modules must use a compiler that can read that metadata.
pub const UTS_KOTLIN_PLUGIN_VERSION: &str = "2.2.0";

#[derive(Debug, Clone)]
pub struct AndroidBuildEnvironment {
    pub gradle_bin: std::path::PathBuf,
    pub java_home: std::path::PathBuf,
    pub android_home: std::path::PathBuf,
    pub gradle_user_home: std::path::PathBuf,
}

#[derive(Default)]
pub struct AndroidManifestPatches {
    pub permissions: String,
    pub application_entries: String,
    pub pandora_entry_intent_filters: String,
}

pub fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

pub fn emit_log(window: &tauri::Window, level: &str, message: &str, progress: Option<u8>) {
    emit_log_with_build_id(window, None, "android", level, message, progress);
}

pub fn emit_log_for_build(
    window: &tauri::Window,
    build_id: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    emit_log_with_build_id(window, Some(build_id), "android", level, message, progress);
}

pub fn emit_log_with_build_id(
    window: &tauri::Window,
    build_id: Option<&str>,
    platform: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    let event = BuildLogEvent {
        build_id: build_id.map(|id| id.to_string()),
        platform: platform.to_string(),
        level: level.to_string(),
        message: message.to_string(),
        progress,
    };
    let _ = window.emit("build-log", event);
}

/// 非 AAR 解包后的信息，用于生成正确的 build.gradle 依赖声明
pub struct UnpackedAarInfo {
    pub original_name: String,
    pub extra_jars: Vec<String>,
}

// ===== 通用工具函数（跨模块使用） =====

pub fn sanitize_java_identifier(id: &str) -> String {
    let mut result = String::with_capacity(id.len());
    for (i, c) in id.chars().enumerate() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
            result.push(c);
        } else {
            result.push('_');
        }
        if i == 0 && result.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            result.insert(0, '_');
        }
    }
    if result.is_empty() {
        "unknown".to_string()
    } else {
        result
    }
}

pub fn render_gradle_dependency_line(dep: &str) -> String {
    let dep = dep.trim();
    let known_gradle_configurations = [
        "implementation ",
        "api ",
        "compileOnly ",
        "runtimeOnly ",
        "debugImplementation ",
        "releaseImplementation ",
        "kapt ",
        "annotationProcessor ",
    ];
    if known_gradle_configurations
        .iter()
        .any(|configuration| dep.starts_with(configuration))
    {
        return format!("    {}", dep);
    }

    match dep {
        "com.getui:gtsdk:3.3.7.0" => {
            "    implementation('com.getui:gtsdk:3.3.7.0'){ exclude(group: 'com.getui') }"
                .to_string()
        }
        "com.getui:gysdk:3.1.7.0" => {
            "    implementation('com.getui:gysdk:3.1.7.0'){ exclude(group: 'com.getui', module: 'gtc') }".to_string()
        }
        _ => format!("    implementation '{}'", dep),
    }
}

pub fn render_dependency_excludes(_extra_dependencies: &str) -> String {
    String::new()
}

pub fn prefix_if_nonempty(value: String, prefix: &str) -> String {
    if value.is_empty() {
        value
    } else {
        format!("{}{}", prefix, value)
    }
}

pub fn escape_gradle_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn indent_manifest_fragment(fragment: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    fragment
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line.trim())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
