//! iOS xcodebuild 环境解析与命令执行。

use std::path::{Path, PathBuf};

use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct IosBuildEnvironment {
    pub(crate) xcodebuild_bin: PathBuf,
    pub(crate) developer_dir: PathBuf,
}

pub(super) async fn run_xcodebuild(
    args: &[String],
    cwd: &Path,
    window: &tauri::Window,
    env: &IosBuildEnvironment,
    build_id: &str,
) -> Result<(), String> {
    run_xcodebuild_with_sink(args, cwd, Arc::new(window.clone()), env, build_id).await
}

pub(crate) async fn run_xcodebuild_with_sink(
    args: &[String],
    cwd: &Path,
    sink: crate::utils::process::SharedBuildEventSink,
    env: &IosBuildEnvironment,
    build_id: &str,
) -> Result<(), String> {
    let output = crate::utils::process::run_command_streaming_with_env_sink(
        &env.xcodebuild_bin.to_string_lossy(),
        args,
        &cwd.to_string_lossy(),
        &ios_process_env(env),
        sink,
        "build-log",
        Some(crate::utils::process::StreamLogMeta {
            build_id: build_id.to_string(),
            platform: "ios".to_string(),
        }),
    )
    .await
    .map_err(|e| e.to_string())?;
    if output.success {
        Ok(())
    } else {
        Err(format!("xcodebuild 失败，退出码: {:?}", output.exit_code))
    }
}

pub(crate) fn resolve_ios_ci_environment() -> Result<IosBuildEnvironment, String> {
    let output = std::process::Command::new("xcrun")
        .args(["--find", "xcodebuild"])
        .output()
        .map_err(|e| format!("执行 xcrun --find xcodebuild 失败: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "xcrun 未找到 xcodebuild: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let xcodebuild_bin = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    let developer_dir = std::env::var_os("DEVELOPER_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let output = std::process::Command::new("xcode-select")
                .arg("-p")
                .output()
                .ok()?;
            output
                .status
                .success()
                .then(|| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
        })
        .ok_or_else(|| "无法解析 Runner 的 DEVELOPER_DIR".to_string())?;
    if !xcodebuild_bin.is_file() || !developer_dir.is_dir() {
        return Err("Runner 的 Xcode 环境无效".to_string());
    }
    Ok(IosBuildEnvironment {
        xcodebuild_bin,
        developer_dir,
    })
}

pub(super) fn resolve_ios_build_environment() -> Result<IosBuildEnvironment, String> {
    let xcodebuild_bin =
        crate::commands::shared::env::resolve_configured_tool_bin("xcode", "xcodebuild")?;
    let developer_dir = xcodebuild_bin
        .parent()
        .and_then(|bin| bin.parent())
        .and_then(|usr| usr.parent())
        .and_then(|developer| {
            (developer.file_name().and_then(|n| n.to_str()) == Some("Developer"))
                .then(|| developer.to_path_buf())
        })
        .or_else(|| {
            let configured =
                crate::commands::shared::env::require_configured_tool_path("xcode").ok()?;
            if configured.extension().and_then(|ext| ext.to_str()) == Some("app") {
                Some(configured.join("Contents/Developer"))
            } else {
                configured.parent().map(|p| p.to_path_buf())
            }
        })
        .ok_or_else(|| {
            format!(
                "无法从 xcodebuild 路径推导 DEVELOPER_DIR: {}",
                xcodebuild_bin.display()
            )
        })?;
    if !developer_dir.exists() {
        return Err(format!(
            "Xcode DEVELOPER_DIR 不存在: {}",
            developer_dir.display()
        ));
    }
    Ok(IosBuildEnvironment {
        xcodebuild_bin,
        developer_dir,
    })
}

fn ios_process_env(env: &IosBuildEnvironment) -> Vec<(String, String)> {
    vec![(
        "DEVELOPER_DIR".into(),
        env.developer_dir.to_string_lossy().to_string(),
    )]
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::utils::process::{BuildEventSink, SharedBuildEventSink};
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct RecordingSink {
        events: Mutex<Vec<(String, serde_json::Value)>>,
    }

    impl BuildEventSink for RecordingSink {
        fn send(&self, channel: &str, payload: serde_json::Value) {
            self.events
                .lock()
                .unwrap()
                .push((channel.to_string(), payload));
        }
    }

    #[tokio::test]
    async fn fake_xcodebuild_runs_headless_with_explicit_developer_dir() {
        let root =
            std::env::temp_dir().join(format!("unipack-fake-xcodebuild-{}", uuid::Uuid::new_v4()));
        let developer_dir = root.join("FakeXcode.app/Contents/Developer");
        std::fs::create_dir_all(&developer_dir).unwrap();
        let script = root.join("xcodebuild");
        std::fs::write(
            &script,
            r#"#!/bin/sh
set -eu
printf 'developer=%s\n' "$DEVELOPER_DIR"
printf 'args=%s\n' "$*"
"#,
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&script, permissions).unwrap();

        let env = IosBuildEnvironment {
            xcodebuild_bin: script,
            developer_dir: developer_dir.clone(),
        };
        let sink = Arc::new(RecordingSink::default());
        let shared_sink: SharedBuildEventSink = sink.clone();

        run_xcodebuild_with_sink(
            &["-scheme".into(), "FakeApp".into(), "archive".into()],
            &root,
            shared_sink,
            &env,
            "ios-headless-smoke",
        )
        .await
        .unwrap();

        let events = sink.events.lock().unwrap();
        assert!(events.iter().all(|(channel, _)| channel == "build-log"));
        assert!(events.iter().all(|(_, event)| {
            event["buildId"] == "ios-headless-smoke" && event["platform"] == "ios"
        }));
        assert!(events.iter().any(|(_, event)| {
            event["line"]
                .as_str()
                .is_some_and(|line| line == format!("developer={}", developer_dir.display()))
        }));
        assert!(events.iter().any(|(_, event)| {
            event["line"] == "args=-scheme FakeApp archive" && event["type"] == "stdout"
        }));

        drop(events);
        let _ = std::fs::remove_dir_all(root);
    }
}
