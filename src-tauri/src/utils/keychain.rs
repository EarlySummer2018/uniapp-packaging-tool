#![allow(dead_code)]
use anyhow::Result;

const SERVICE_NAME: &str = "unipack-tool";

#[cfg(target_os = "macos")]
pub fn store_password(account: &str, password: &str) -> Result<()> {
    run_security_cmd(
        "add-generic-password",
        &["-a", account, "-s", SERVICE_NAME, "-w", password, "-U"],
    )?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn get_password(account: &str) -> Result<Option<String>> {
    let output = run_security_cmd(
        "find-generic-password",
        &["-a", account, "-s", SERVICE_NAME, "-w"],
    )?;

    if !output.status.success() {
        return Ok(None);
    }

    let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if password.is_empty() {
        Ok(None)
    } else {
        Ok(Some(password))
    }
}

#[cfg(target_os = "macos")]
pub fn delete_password(account: &str) -> Result<bool> {
    match run_security_cmd(
        "delete-generic-password",
        &["-a", account, "-s", SERVICE_NAME],
    ) {
        Ok(output) => Ok(output.status.success()),
        Err(e) if e.to_string().contains("could not be found") => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn store_password(account: &str, password: &str) -> Result<()> {
    let path = secret_path(account);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, password)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn get_password(account: &str) -> Result<Option<String>> {
    let path = secret_path(account);
    if !path.exists() {
        return Ok(None);
    }
    let password = std::fs::read_to_string(path)?.trim().to_string();
    if password.is_empty() {
        Ok(None)
    } else {
        Ok(Some(password))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn delete_password(account: &str) -> Result<bool> {
    let path = secret_path(account);
    if !path.exists() {
        return Ok(false);
    }
    std::fs::remove_file(path)?;
    Ok(true)
}

#[cfg(not(target_os = "macos"))]
fn secret_path(account: &str) -> std::path::PathBuf {
    let safe = account.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    crate::utils::fs::get_unipack_home()
        .join("secrets")
        .join(format!("{}.secret", safe))
}

#[cfg(target_os = "macos")]
struct CmdOutput {
    stdout: Vec<u8>,
    status: std::process::ExitStatus,
}

#[cfg(target_os = "macos")]
fn run_security_cmd(command: &str, args: &[&str]) -> Result<CmdOutput> {
    let output = std::process::Command::new("security")
        .arg(command)
        .args(args)
        .output()?;

    if !output.status.success()
        && !command.contains("find-generic-password")
        && !command.contains("delete-generic-password")
    {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "security {} failed: {}",
            command,
            err_msg.trim()
        ));
    }

    Ok(CmdOutput {
        stdout: output.stdout,
        status: output.status,
    })
}
