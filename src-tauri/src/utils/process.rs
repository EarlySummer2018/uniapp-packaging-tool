#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Destination for build events.  The native build core uses this interface so
/// it can run both inside Tauri and in the headless GitHub Actions runner.
pub trait BuildEventSink: Send + Sync {
    fn send(&self, channel: &str, payload: serde_json::Value);
}

impl BuildEventSink for tauri::Window {
    fn send(&self, channel: &str, payload: serde_json::Value) {
        let _ = self.emit(channel, payload);
    }
}

impl BuildEventSink for tauri::AppHandle {
    fn send(&self, channel: &str, payload: serde_json::Value) {
        let _ = self.emit(channel, payload);
    }
}

/// Cloneable sink used by async command readers and headless build runtimes.
pub type SharedBuildEventSink = Arc<dyn BuildEventSink>;

#[derive(Debug, Default)]
pub struct JsonLineBuildEventSink {
    secrets: Vec<String>,
}

impl JsonLineBuildEventSink {
    pub fn new(secrets: impl IntoIterator<Item = String>) -> Self {
        Self {
            secrets: secrets
                .into_iter()
                .filter(|secret| !secret.is_empty())
                .collect(),
        }
    }

    fn redacted_event(&self, channel: &str, mut payload: serde_json::Value) -> serde_json::Value {
        redact_json_secrets(&mut payload);
        redact_json_values(&mut payload, &self.secrets);
        serde_json::json!({ "channel": channel, "event": payload })
    }
}

impl BuildEventSink for JsonLineBuildEventSink {
    fn send(&self, channel: &str, payload: serde_json::Value) {
        println!("{}", self.redacted_event(channel, payload));
    }
}

fn redact_json_values(value: &mut serde_json::Value, secrets: &[String]) {
    match value {
        serde_json::Value::String(text) => {
            for secret in secrets {
                *text = text.replace(secret, "***");
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                redact_json_values(value, secrets);
            }
        }
        serde_json::Value::Object(map) => {
            for value in map.values_mut() {
                redact_json_values(value, secrets);
            }
        }
        _ => {}
    }
}

/// Keep secrets out of CI logs even when a subprocess happens to echo its
/// command line or environment-derived configuration.
pub fn redact_json_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let lower = key.to_ascii_lowercase();
                if lower.contains("password") || lower.contains("token") || lower.contains("secret")
                {
                    *value = serde_json::Value::String("***".to_string());
                } else {
                    redact_json_secrets(value);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                redact_json_secrets(value);
            }
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StreamLogMeta {
    pub build_id: String,
    pub platform: String,
}

impl CommandOutput {
    pub fn from_exit_status(
        status: std::process::ExitStatus,
        stdout: String,
        stderr: String,
    ) -> Self {
        let success = status.success();
        let exit_code = status.code();
        let logs = parse_output_lines(&stdout, &stderr);
        Self {
            success,
            exit_code,
            stdout,
            stderr,
            logs,
        }
    }
}

pub async fn run_command(program: &str, args: &[&str], cwd: Option<&str>) -> Result<CommandOutput> {
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let child = cmd.spawn()?;
    let output = child.wait_with_output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(CommandOutput::from_exit_status(
        output.status,
        stdout,
        stderr,
    ))
}

pub async fn run_command_streaming(
    program: &str,
    args: &[String],
    cwd: &str,
    app_handle: tauri::AppHandle,
    channel: &str,
) -> Result<CommandOutput> {
    run_command_streaming_with_env(program, args, cwd, &[], app_handle, channel).await
}

pub async fn run_command_streaming_with_env(
    program: &str,
    args: &[String],
    cwd: &str,
    env_vars: &[(String, String)],
    app_handle: tauri::AppHandle,
    channel: &str,
) -> Result<CommandOutput> {
    run_command_streaming_with_env_internal(program, args, cwd, env_vars, app_handle, channel, None)
        .await
}

pub async fn run_command_streaming_with_env_tagged(
    program: &str,
    args: &[String],
    cwd: &str,
    env_vars: &[(String, String)],
    app_handle: tauri::AppHandle,
    channel: &str,
    meta: StreamLogMeta,
) -> Result<CommandOutput> {
    run_command_streaming_with_env_internal(
        program,
        args,
        cwd,
        env_vars,
        app_handle,
        channel,
        Some(meta),
    )
    .await
}

async fn run_command_streaming_with_env_internal(
    program: &str,
    args: &[String],
    cwd: &str,
    env_vars: &[(String, String)],
    app_handle: tauri::AppHandle,
    channel: &str,
    meta: Option<StreamLogMeta>,
) -> Result<CommandOutput> {
    run_command_streaming_with_env_sink(
        program,
        args,
        cwd,
        env_vars,
        Arc::new(app_handle),
        channel,
        meta,
    )
    .await
}

pub async fn run_command_streaming_with_env_sink(
    program: &str,
    args: &[String],
    cwd: &str,
    env_vars: &[(String, String)],
    sink: SharedBuildEventSink,
    channel: &str,
    meta: Option<StreamLogMeta>,
) -> Result<CommandOutput> {
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(cwd);
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

    let sink_clone = sink.clone();
    let channel_stdout = channel.to_string();
    let stdout_meta = meta.clone();
    let stdout_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let payload = if let Some(meta) = &stdout_meta {
                serde_json::json!({
                    "type": "stdout",
                    "line": line,
                    "level": "info",
                    "buildId": meta.build_id,
                    "platform": meta.platform,
                })
            } else {
                serde_json::json!({ "type": "stdout", "line": line })
            };
            sink_clone.send(&channel_stdout, payload);
        }
    });

    let sink_clone2 = sink.clone();
    let channel_stderr = channel.to_string();
    let stderr_meta = meta.clone();
    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let level = stderr_line_level(&line);
            let payload = if let Some(meta) = &stderr_meta {
                serde_json::json!({
                    "type": "stderr",
                    "line": line,
                    "level": level,
                    "buildId": meta.build_id,
                    "platform": meta.platform,
                })
            } else {
                serde_json::json!({ "type": "stderr", "line": line })
            };
            sink_clone2.send(&channel_stderr, payload);
        }
    });

    let status = child.wait().await?;
    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    Ok(CommandOutput::from_exit_status(
        status,
        String::new(),
        String::new(),
    ))
}

pub async fn run_command_with_env(
    program: &str,
    args: &[&str],
    env_vars: &[(&str, &str)],
    cwd: Option<&str>,
) -> Result<CommandOutput> {
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    for &(key, val) in env_vars {
        cmd.env(key, val);
    }

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let child = cmd.spawn()?;
    let output = child.wait_with_output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(CommandOutput::from_exit_status(
        output.status,
        stdout,
        stderr,
    ))
}

pub fn kill_process(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
    }
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status();
    }
    Ok(())
}

fn parse_output_lines(stdout: &str, stderr: &str) -> Vec<String> {
    let mut logs = Vec::new();
    for line in stdout.lines() {
        logs.push(line.to_string());
    }
    for line in stderr.lines() {
        logs.push(format!("[ERR] {}", line));
    }
    logs
}

fn stderr_line_level(line: &str) -> &'static str {
    let lower = line.to_ascii_lowercase();
    if line.contains("IDEDistributionLogging _createLoggingBundleAtPath:")
        || line.contains("Created bundle at path")
    {
        "info"
    } else if line.contains(
        "IDERunDestination: Supported platforms for the buildables in the current scheme is empty.",
    ) || line.contains("Command line name \"app-store\" is deprecated")
        || lower.contains("warning:")
        || lower.contains("ld: warning")
    {
        "warn"
    } else {
        "error"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        run_command_streaming_with_env_sink, stderr_line_level, BuildEventSink,
        JsonLineBuildEventSink, SharedBuildEventSink, StreamLogMeta,
    };

    #[cfg(unix)]
    use std::path::{Path, PathBuf};
    #[cfg(unix)]
    use std::sync::{Arc, Mutex};

    #[cfg(unix)]
    struct RecordingJsonLineSink {
        formatter: JsonLineBuildEventSink,
        events: Mutex<Vec<serde_json::Value>>,
    }

    #[cfg(unix)]
    impl RecordingJsonLineSink {
        fn new(secrets: impl IntoIterator<Item = String>) -> Self {
            Self {
                formatter: JsonLineBuildEventSink::new(secrets),
                events: Mutex::new(Vec::new()),
            }
        }

        fn events(&self) -> Vec<serde_json::Value> {
            self.events.lock().unwrap().clone()
        }
    }

    #[cfg(unix)]
    impl BuildEventSink for RecordingJsonLineSink {
        fn send(&self, channel: &str, payload: serde_json::Value) {
            self.events
                .lock()
                .unwrap()
                .push(self.formatter.redacted_event(channel, payload));
        }
    }

    #[cfg(unix)]
    fn write_executable_script(root: &Path, name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        std::fs::create_dir_all(root).unwrap();
        let path = root.join(name);
        std::fs::write(&path, body).unwrap();
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[test]
    fn structured_logs_redact_secret_fields_and_values() {
        let sink = JsonLineBuildEventSink::new(["value-secret".to_string()]);
        let event = sink.redacted_event(
            "build-log",
            serde_json::json!({
                "password": "field-secret",
                "line": "gradle echoed value-secret",
            }),
        );
        assert_eq!(event["channel"], "build-log");
        assert_eq!(event["event"]["password"], "***");
        assert_eq!(event["event"]["line"], "gradle echoed ***");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn headless_process_streams_tagged_redacted_logs_with_explicit_env() {
        let root =
            std::env::temp_dir().join(format!("unipack-headless-process-{}", uuid::Uuid::new_v4()));
        let script = write_executable_script(
            &root,
            "fake-build-tool",
            r#"#!/bin/sh
set -eu
printf 'cwd=%s env=%s secret=%s\n' "$PWD" "$UNIPACK_TEST_ENV" 'runner-secret'
printf 'warning: fake stderr runner-secret\n' >&2
"#,
        );
        let sink = Arc::new(RecordingJsonLineSink::new(["runner-secret".to_string()]));
        let shared_sink: SharedBuildEventSink = sink.clone();

        let output = run_command_streaming_with_env_sink(
            &script.to_string_lossy(),
            &["assembleRelease".to_string()],
            &root.to_string_lossy(),
            &[("UNIPACK_TEST_ENV".to_string(), "explicit-value".to_string())],
            shared_sink,
            "build-log",
            Some(StreamLogMeta {
                build_id: "headless-smoke".to_string(),
                platform: "android".to_string(),
            }),
        )
        .await
        .unwrap();

        assert!(output.success);
        let events = sink.events();
        let stdout = events
            .iter()
            .find(|event| event["event"]["type"] == "stdout")
            .expect("stdout event");
        assert_eq!(stdout["channel"], "build-log");
        assert_eq!(stdout["event"]["buildId"], "headless-smoke");
        assert_eq!(stdout["event"]["platform"], "android");
        assert_eq!(stdout["event"]["level"], "info");
        let stdout_line = stdout["event"]["line"].as_str().unwrap();
        assert!(stdout_line.contains("env=explicit-value"));
        assert!(stdout_line.contains("secret=***"));
        assert!(!stdout_line.contains("runner-secret"));

        let stderr = events
            .iter()
            .find(|event| event["event"]["type"] == "stderr")
            .expect("stderr event");
        assert_eq!(stderr["event"]["level"], "warn");
        assert_eq!(stderr["event"]["line"], "warning: fake stderr ***");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn xcode_run_destination_noise_is_warning() {
        assert_eq!(
            stderr_line_level(
                "2026-06-18 16:48:24.450 xcodebuild[60339:4176221] [MT] IDERunDestination: Supported platforms for the buildables in the current scheme is empty."
            ),
            "warn"
        );
        assert_eq!(
            stderr_line_level("ld: framework not found DingRTC"),
            "error"
        );
    }

    #[test]
    fn xcode_distribution_noise_is_not_error() {
        assert_eq!(
            stderr_line_level(
                "2026-06-18 18:21:19.333 xcodebuild[68876:4849206] [MT] IDEDistribution: -[IDEDistributionLogging _createLoggingBundleAtPath:]: Created bundle at path \"/tmp/HBuilder.xcdistributionlogs\"."
            ),
            "info"
        );
        assert_eq!(
            stderr_line_level(
                "2026-06-18 18:21:19.582 xcodebuild[68876:4849206] [MT] IDEDistribution: Command line name \"app-store\" is deprecated. Use \"app-store-connect\" instead."
            ),
            "warn"
        );
        assert_eq!(
            stderr_line_level(
                "ld: warning: no platform load command found in 'lib.a', assuming: iOS"
            ),
            "warn"
        );
    }
}
