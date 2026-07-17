//! Android 构建类型定义

use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    pub cloud_run_url: Option<String>,
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
    let format_pattern = "%Y%m%d-%H%M%S";
    chrono::Local::now().format(format_pattern).to_string()
}

pub fn emit_log(
    sink: &dyn crate::utils::process::BuildEventSink,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    emit_log_with_build_id(sink, None, "android", level, message, progress);
}

pub fn emit_log_for_build(
    sink: &dyn crate::utils::process::BuildEventSink,
    build_id: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    emit_log_with_build_id(sink, Some(build_id), "android", level, message, progress);
}

pub fn emit_log_with_build_id(
    sink: &dyn crate::utils::process::BuildEventSink,
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
    sink.send(
        "build-log",
        serde_json::to_value(event).unwrap_or_else(|_| {
            serde_json::json!({
                "platform": platform,
                "level": level,
                "message": message,
                "progress": progress,
            })
        }),
    );
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
        if i == 0 && result.chars().next().is_some_and(|c| c.is_ascii_digit()) {
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
        _ if dep.contains("${") => {
            format!("    implementation \"{}\"", dep)
        }
        _ if dep.starts_with("com.amap.api:3dmap-location-search:") => {
            format!("    implementation \"{}\"", dep)
        }
        _ if dep.starts_with("com.tencent.map.geolocation:TencentLocationSdk-openplatform:") => {
            format!("    implementation('{}')", dep)
        }
        "com.getui:gtsdk:3.3.7.0" => {
            "    implementation('com.getui:gtsdk:3.3.7.0'){ exclude(group: 'com.getui') }"
                .to_string()
        }
        "com.getui:gysdk:3.1.7.0" => {
            "    implementation('com.getui:gysdk:3.1.7.0'){ exclude(group: 'com.getui') }"
                .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amap_combined_dependency_uses_double_quoted_implementation() {
        assert_eq!(
            render_gradle_dependency_line(
                "com.amap.api:3dmap-location-search:10.0.700_loc6.4.5_sea9.7.2"
            ),
            "    implementation \"com.amap.api:3dmap-location-search:10.0.700_loc6.4.5_sea9.7.2\""
        );
    }

    #[test]
    fn android_x_dependency_using_root_project_version_keeps_interpolation() {
        assert_eq!(
            render_gradle_dependency_line(
                "androidx.appcompat:appcompat:${rootProject.ext.androidxVersion}"
            ),
            "    implementation \"androidx.appcompat:appcompat:${rootProject.ext.androidxVersion}\""
        );
    }

    #[test]
    fn univerify_dependency_excludes_getui_group_as_officially_documented() {
        assert_eq!(
            render_gradle_dependency_line("com.getui:gysdk:3.1.7.0"),
            "    implementation('com.getui:gysdk:3.1.7.0'){ exclude(group: 'com.getui') }"
        );
    }

    #[test]
    fn tencent_location_dependency_uses_parenthesized_implementation() {
        assert_eq!(
            render_gradle_dependency_line(
                "com.tencent.map.geolocation:TencentLocationSdk-openplatform:2.3.1"
            ),
            "    implementation('com.tencent.map.geolocation:TencentLocationSdk-openplatform:2.3.1')"
        );
    }
}
