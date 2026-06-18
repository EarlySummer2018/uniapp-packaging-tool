use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvReport {
    pub os: OsInfo,
    pub java: ToolInfo,
    pub node: ToolInfo,
    pub python: ToolInfo,
    pub git: ToolInfo,
    pub gradle: ToolInfo,
    pub cocoapods: ToolInfo,
    pub android: PlatformEnv,
    pub ios: PlatformEnv,
    pub harmony: PlatformEnv,
    pub disk_space: DiskSpaceInfo,
    pub android_studio: ToolInfo,
    pub ndk: ToolInfo,
    pub hbuilderx: Option<HBuilderXEnvInfo>,
    pub command_line_tools: ToolInfo,
    pub sdk_build_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HBuilderXEnvInfo {
    pub installed: bool,
    pub version: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEnv {
    pub available: bool,
    pub sdk_path: Option<String>,
    pub sdk_version: Option<String>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSpaceInfo {
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f64,
}

pub(crate) fn check_tool(name: &str, arg: &str) -> ToolInfo {
    match std::process::Command::new(name).arg(arg).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(|s| s.trim().to_string());
            ToolInfo {
                installed: true,
                version,
                path: which_tool(name),
            }
        }
        _ => ToolInfo {
            installed: false,
            version: None,
            path: None,
        },
    }
}

pub(crate) fn which_tool(name: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn get_os_info() -> OsInfo {
    OsInfo {
        name: std::env::consts::OS.to_string(),
        version: get_os_version(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn get_os_version() -> String {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("PRETTY_NAME="))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|s| s.trim_matches('"').to_string())
            })
            .unwrap_or_default()
    }
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        "Unknown".to_string()
    }
}

fn get_disk_info() -> DiskSpaceInfo {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    match sys_disk_info(&home) {
        Some((total, free)) => {
            let used = total.saturating_sub(free);
            let usage_percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            DiskSpaceInfo {
                total_gb: total as f64 / (1024.0 * 1024.0 * 1024.0),
                free_gb: free as f64 / (1024.0 * 1024.0 * 1024.0),
                used_gb: used as f64 / (1024.0 * 1024.0 * 1024.0),
                usage_percent,
            }
        }
        None => DiskSpaceInfo {
            total_gb: 0.0,
            free_gb: 0.0,
            used_gb: 0.0,
            usage_percent: 0.0,
        },
    }
}

#[cfg(unix)]
fn sys_disk_info(path: &std::path::Path) -> Option<(u64, u64)> {
    unsafe {
        let mut statvfs_out: libc::statvfs = std::mem::zeroed();
        let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_ref()).ok()?;
        if libc::statvfs(path_cstr.as_ptr(), &mut statvfs_out) == 0 && statvfs_out.f_blocks > 0 {
            let total = statvfs_out.f_blocks as u64 * statvfs_out.f_frsize as u64;
            let free = statvfs_out.f_bavail as u64 * statvfs_out.f_frsize as u64;
            Some((total, free))
        } else {
            None
        }
    }
}

#[cfg(not(unix))]
fn sys_disk_info(_path: &std::path::Path) -> Option<(u64, u64)> {
    None
}

fn check_android_studio() -> ToolInfo {
    #[cfg(target_os = "macos")]
    {
        let apps_dir = PathBuf::from("/Applications");
        if let Ok(entries) = std::fs::read_dir(&apps_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("Android Studio") && name.ends_with(".app") {
                    let path = entry.path();
                    let version = read_android_studio_version(&path);
                    return ToolInfo {
                        installed: true,
                        version,
                        path: Some(path.to_string_lossy().to_string()),
                    };
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        let program_files = std::env::var("ProgramFiles").unwrap_or_default();
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        for base in [program_files, local_app_data] {
            let as_path = PathBuf::from(&base).join("Android").join("Android Studio");
            if as_path.exists() {
                return ToolInfo {
                    installed: true,
                    version: None,
                    path: Some(as_path.to_string_lossy().to_string()),
                };
            }
        }
    }
    ToolInfo {
        installed: false,
        version: None,
        path: None,
    }
}

#[cfg(target_os = "macos")]
fn read_android_studio_version(app_path: &Path) -> Option<String> {
    let plist = app_path.join("Contents").join("Info.plist");
    if !plist.exists() {
        return None;
    }
    std::fs::read_to_string(&plist).ok().and_then(|content| {
        content
            .lines()
            .find(|l| l.contains("CFBundleShortVersionString"))
            .and_then(|line| {
                line.split('>')
                    .next_back()?
                    .split('<')
                    .next()
                    .map(|v| v.trim().to_string())
            })
    })
}

fn check_ndk() -> ToolInfo {
    let android_home = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_ROOT"))
        .ok();

    let ndk_home = android_home
        .as_ref()
        .map(|h| PathBuf::from(h).join("ndk"))
        .or_else(|| std::env::var("NDK_HOME").ok().map(PathBuf::from));

    match ndk_home {
        Some(ndk_path) if ndk_path.exists() => {
            if let Ok(entries) = std::fs::read_dir(&ndk_path) {
                for entry in entries.flatten() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    if dir_name.starts_with("android-ndk-")
                        || dir_name.parse::<f64>().is_ok()
                        || dir_name.contains('.')
                    {
                        return ToolInfo {
                            installed: true,
                            version: Some(dir_name),
                            path: Some(entry.path().to_string_lossy().to_string()),
                        };
                    }
                }
            }
            ToolInfo {
                installed: true,
                version: None,
                path: Some(ndk_path.to_string_lossy().to_string()),
            }
        }
        _ => ToolInfo {
            installed: false,
            version: None,
            path: None,
        },
    }
}

fn check_hbuilderx_env() -> Option<HBuilderXEnvInfo> {
    let candidates: Vec<PathBuf> = vec![PathBuf::from("/Applications/HBuilderX.app")];
    let mut paths = candidates;
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Applications").join("HBuilderX.app"));
    }

    for hb_path in paths {
        if hb_path.exists() {
            let version = detect_hbx_version(&hb_path);
            return Some(HBuilderXEnvInfo {
                installed: true,
                version,
                path: hb_path.to_string_lossy().to_string(),
            });
        }
    }
    None
}

fn detect_hbx_version(hb_path: &Path) -> String {
    let plist_path = hb_path.join("Contents").join("Info.plist");
    if plist_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&plist_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.contains("CFBundleShortVersionString") {
                    if let Some(rest) = line.split('>').next_back() {
                        if let Some(v) = rest.split('<').next() {
                            let v = v.trim().trim_matches('"');
                            if !v.is_empty() {
                                return v.to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    let plugins_dir = hb_path.join("plugins");
    if !plugins_dir.exists() {
        return String::new();
    }

    let uniapp_cli = plugins_dir.join("uniapp-cli");
    if uniapp_cli.exists() {
        if let Ok(content) = std::fs::read_to_string(uniapp_cli.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ver) = json.get("version").and_then(|v| v.as_str()) {
                    return ver.to_string();
                }
            }
        }
    }

    String::new()
}

fn check_command_line_tools() -> ToolInfo {
    let output = std::process::Command::new("xcode-select")
        .arg("-p")
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let is_clt_only = path.contains("CommandLineTools")
                || path.contains("Library/Developer/CommandLineTools");

            let version_output = std::process::Command::new("xcodebuild")
                .arg("-version")
                .output();
            let version = version_output
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .find(|l| l.contains("Xcode") || l.contains("Command Line Tools"))
                        .map(|s| s.trim().to_string())
                });

            ToolInfo {
                installed: true,
                version: version.or_else(|| {
                    Some(if is_clt_only {
                        "CLT only".to_string()
                    } else {
                        "via Xcode".to_string()
                    })
                }),
                path: Some(path),
            }
        }
        _ => ToolInfo {
            installed: false,
            version: None,
            path: None,
        },
    }
}

fn get_sdk_build_tools_versions() -> Vec<String> {
    let android_home = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_ROOT"))
        .ok();

    match android_home {
        Some(home) => {
            let bt_dir = PathBuf::from(&home).join("build-tools");
            if bt_dir.is_dir() {
                return std::fs::read_dir(&bt_dir)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect();
            }
            vec![]
        }
        None => vec![],
    }
}

/// Build env report by calling platform-specific check functions.
/// The platform check functions are provided by each platform module.
pub async fn build_env_report(
    android_env: PlatformEnv,
    ios_env: PlatformEnv,
    harmony_env: PlatformEnv,
) -> EnvReport {
    EnvReport {
        os: get_os_info(),
        java: check_tool("java", "-version"),
        node: check_tool("node", "--version"),
        python: check_tool("python3", "--version"),
        git: check_tool("git", "--version"),
        gradle: check_gradle(),
        cocoapods: check_cocoapods(),
        android: android_env,
        ios: ios_env,
        harmony: harmony_env,
        disk_space: get_disk_info(),
        android_studio: check_android_studio(),
        ndk: check_ndk(),
        hbuilderx: check_hbuilderx_env(),
        command_line_tools: check_command_line_tools(),
        sdk_build_tools: get_sdk_build_tools_versions(),
    }
}

fn check_gradle() -> ToolInfo {
    let paths = [("gradle", "-v"), ("./gradlew", "-v")];

    for (cmd, arg) in &paths {
        if let Ok(output) = std::process::Command::new(cmd).arg(arg).output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .find(|l| l.contains("Gradle"))
                    .map(|s| s.trim().to_string());
                return ToolInfo {
                    installed: true,
                    version,
                    path: which_tool(cmd),
                };
            }
        }
    }

    ToolInfo {
        installed: false,
        version: None,
        path: None,
    }
}

fn check_cocoapods() -> ToolInfo {
    match std::process::Command::new("pod").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(|s| s.trim().to_string());
            ToolInfo {
                installed: true,
                version,
                path: which_tool("pod"),
            }
        }
        _ => ToolInfo {
            installed: false,
            version: None,
            path: None,
        },
    }
}

// --- validate_tool_path is in env_validate.rs, use that path directly ---

#[tauri::command]
pub async fn check_env() -> Result<EnvReport, String> {
    let android_env = PlatformEnv {
        available: false,
        sdk_path: None,
        sdk_version: None,
        issues: vec![],
    };
    let ios_env = PlatformEnv {
        available: false,
        sdk_path: None,
        sdk_version: None,
        issues: vec![],
    };
    let harmony_env = PlatformEnv {
        available: false,
        sdk_path: None,
        sdk_version: None,
        issues: vec![],
    };
    Ok(build_env_report(android_env, ios_env, harmony_env).await)
}

#[tauri::command]
pub async fn get_full_env_report() -> Result<EnvReport, String> {
    check_env().await
}

fn resolve_bin_path(input_path: &str, bin_name: &str) -> Option<PathBuf> {
    let p = PathBuf::from(input_path);
    if p.is_file() {
        return Some(p);
    }
    let direct = p.join(bin_name);
    if direct.exists() {
        return Some(direct);
    }
    let bin_dir = p.join("bin").join(bin_name);
    if bin_dir.exists() {
        return Some(bin_dir);
    }
    None
}

pub fn resolve_configured_tool_bin(tool_name: &str, bin_name: &str) -> Result<PathBuf, String> {
    let record = get_env_override_sync(tool_name)?.ok_or_else(|| {
        format!(
            "请先在 SDK & 环境管理中配置 {}",
            tool_display_name(tool_name)
        )
    })?;
    resolve_tool_bin_from_path(tool_name, &record.actual_path, bin_name)
        .or_else(|| resolve_tool_bin_from_path(tool_name, &record.path, bin_name))
        .ok_or_else(|| {
            format!(
                "SDK & 环境管理中配置的 {} 无效，未找到 {}: {}",
                tool_display_name(tool_name),
                bin_name,
                record.actual_path
            )
        })
}

fn resolve_tool_bin_from_path(tool_name: &str, path: &str, bin_name: &str) -> Option<PathBuf> {
    if tool_name == "xcode" {
        let p = PathBuf::from(path);
        for candidate in [
            p.join("Contents")
                .join("Developer")
                .join("usr")
                .join("bin")
                .join(bin_name),
            p.join("usr").join("bin").join(bin_name),
        ] {
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    resolve_bin_path(path, bin_name)
}

pub fn resolve_configured_tool_bin_with_candidates(
    tool_name: &str,
    bin_names: &[&str],
) -> Result<PathBuf, String> {
    let record = get_env_override_sync(tool_name)?.ok_or_else(|| {
        format!(
            "请先在 SDK & 环境管理中配置 {}",
            tool_display_name(tool_name)
        )
    })?;
    for bin_name in bin_names {
        if let Some(path) = resolve_tool_bin_from_path(tool_name, &record.actual_path, bin_name)
            .or_else(|| resolve_tool_bin_from_path(tool_name, &record.path, bin_name))
        {
            return Ok(path);
        }
    }
    Err(format!(
        "SDK & 环境管理中配置的 {} 无效，未找到 {}: {}",
        tool_display_name(tool_name),
        bin_names.join(" / "),
        record.actual_path
    ))
}

pub fn require_configured_tool_path(tool_name: &str) -> Result<PathBuf, String> {
    let record = get_env_override_sync(tool_name)?.ok_or_else(|| {
        format!(
            "请先在 SDK & 环境管理中配置 {}",
            tool_display_name(tool_name)
        )
    })?;
    let path = PathBuf::from(&record.actual_path);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "SDK & 环境管理中配置的 {} 路径不存在: {}",
            tool_display_name(tool_name),
            record.actual_path
        ))
    }
}

fn tool_display_name(tool_name: &str) -> &'static str {
    match tool_name {
        "android_sdk" => "Android SDK",
        "java" => "JDK (Java)",
        "gradle" => "Gradle",
        "xcode" => "Xcode",
        "cocoapods" => "CocoaPods",
        "harmony" => "DevEco Studio / Harmony 工具",
        _ => "对应工具",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvOverrideRecord {
    pub tool_name: String,
    pub path: String,
    pub version: Option<String>,
    pub actual_path: String,
    pub set_at: String,
}

fn get_env_overrides_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
        .join("env-overrides.json")
}

pub fn get_env_override_sync(tool_name: &str) -> Result<Option<EnvOverrideRecord>, String> {
    let overrides_file = get_env_overrides_path();
    if !overrides_file.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&overrides_file).map_err(|e| format!("读取环境覆盖配置失败: {}", e))?;
    let overrides: Vec<EnvOverrideRecord> =
        serde_json::from_str(&content).map_err(|e| format!("解析环境覆盖配置失败: {}", e))?;
    Ok(overrides
        .into_iter()
        .rev()
        .find(|record| record.tool_name == tool_name))
}

#[tauri::command]
pub async fn save_env_override(
    tool_name: String,
    path: String,
    actual_path: String,
    version: Option<String>,
) -> Result<(), String> {
    let overrides_file = get_env_overrides_path();
    let mut overrides: Vec<EnvOverrideRecord> = if overrides_file.exists() {
        let content = fs::read_to_string(&overrides_file)
            .map_err(|e| format!("读取环境覆盖配置失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    overrides.retain(|r| r.tool_name != tool_name);

    overrides.push(EnvOverrideRecord {
        tool_name,
        path,
        actual_path,
        version,
        set_at: chrono::Utc::now().to_rfc3339(),
    });

    if let Some(parent) = overrides_file.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let json =
        serde_json::to_string_pretty(&overrides).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&overrides_file, json).map_err(|e| format!("写入环境覆盖配置失败: {}", e))
}

#[tauri::command]
pub async fn get_env_overrides() -> Result<Vec<EnvOverrideRecord>, String> {
    let overrides_file = get_env_overrides_path();
    if !overrides_file.exists() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(&overrides_file).map_err(|e| format!("读取环境覆盖配置失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析环境覆盖配置失败: {}", e))
}
