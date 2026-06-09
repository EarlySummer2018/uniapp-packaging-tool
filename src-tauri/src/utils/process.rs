#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};

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
    run_command_streaming_with_env_internal(
        program, args, cwd, env_vars, app_handle, channel, None,
    )
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

    let app_clone = app_handle.clone();
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
            let _ = app_clone
                .emit(&channel_stdout, payload)
                .map_err(|e| eprintln!("emit error: {}", e));
        }
    });

    let app_clone2 = app_handle.clone();
    let channel_stderr = channel.to_string();
    let stderr_meta = meta.clone();
    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let payload = if let Some(meta) = &stderr_meta {
                serde_json::json!({
                    "type": "stderr",
                    "line": line,
                    "level": "error",
                    "buildId": meta.build_id,
                    "platform": meta.platform,
                })
            } else {
                serde_json::json!({ "type": "stderr", "line": line })
            };
            let _ = app_clone2
                .emit(&channel_stderr, payload)
                .map_err(|e| eprintln!("emit error: {}", e));
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
