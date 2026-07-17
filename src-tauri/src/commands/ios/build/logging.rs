use std::path::Path;

pub(super) fn emit_ios_log(
    window: &dyn crate::utils::process::BuildEventSink,
    build_id: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    let event = crate::commands::android::BuildLogEvent {
        build_id: Some(build_id.to_string()),
        platform: "ios".to_string(),
        level: level.to_string(),
        message: message.to_string(),
        progress,
    };
    window.send(
        "build-log",
        serde_json::to_value(event).unwrap_or_else(|_| {
            serde_json::json!({
                "buildId": build_id,
                "platform": "ios",
                "level": level,
                "message": message,
                "progress": progress,
            })
        }),
    );
}

pub(super) fn emit_version_warning_if_needed(
    window: &dyn crate::utils::process::BuildEventSink,
    build_id: &str,
    scan: &crate::commands::shared::resource_scan::ResourceScanResult,
    sdk_project: &Path,
) {
    let Some(resource_version) = scan.hbuilderx_version.as_deref() else {
        return;
    };
    let Some(sdk_version) = detect_version_from_path(sdk_project) else {
        emit_ios_log(
            window,
            build_id,
            "warn",
            "无法从 iOS SDK 路径识别版本，请确认与 HBuilderX 导出资源版本一致",
            Some(12),
        );
        return;
    };
    if sdk_version != resource_version {
        emit_ios_log(
            window,
            build_id,
            "warn",
            &format!(
                "资源 HBuilderX 版本 ({}) 与 iOS SDK 路径版本 ({}) 不一致，请确认 SDK 选择正确",
                resource_version, sdk_version
            ),
            Some(12),
        );
    }
}

fn detect_version_from_path(path: &Path) -> Option<String> {
    let re = regex::Regex::new(r"\d+\.\d+(?:\.\d+)?").ok()?;
    path.ancestors().find_map(|ancestor| {
        ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| re.find(name).map(|m| m.as_str().to_string()))
    })
}
