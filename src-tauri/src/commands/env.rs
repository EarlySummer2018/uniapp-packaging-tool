use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

fn check_tool(name: &str, arg: &str) -> ToolInfo {
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

fn which_tool(name: &str) -> Option<String> {
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
fn read_android_studio_version(app_path: &PathBuf) -> Option<String> {
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
                    .last()?
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

fn detect_hbx_version(hb_path: &PathBuf) -> String {
    let plist_path = hb_path.join("Contents").join("Info.plist");
    if plist_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&plist_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.contains("CFBundleShortVersionString") {
                    if let Some(rest) = line.split('>').last() {
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

#[tauri::command]
pub async fn check_env() -> Result<bool, String> {
    let report = build_env_report().await;
    Ok(report.android.available || report.ios.available)
}

#[tauri::command]
pub async fn get_full_env_report() -> Result<EnvReport, String> {
    Ok(build_env_report().await)
}

async fn build_env_report() -> EnvReport {
    let android_env = check_android_platform().await;
    let ios_env = check_ios_platform().await;
    let harmony_env = check_harmony_platform().await;

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

async fn check_android_platform() -> PlatformEnv {
    let android_home = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_ROOT"))
        .ok();
    let mut issues = Vec::new();

    let (sdk_path, sdk_version) = match &android_home {
        Some(path) => {
            let p = std::path::Path::new(path);
            if !p.exists() {
                issues.push(format!(
                    "ANDROID_HOME points to non-existent path: {}",
                    path
                ));
            }
            let version = p.join("build-tools").read_dir().ok().and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .max()
            });
            (Some(path.clone()), version)
        }
        None => {
            issues.push("ANDROID_HOME environment variable is not set".to_string());
            (None, None)
        }
    };

    if !check_tool("java", "-version").installed {
        issues.push("Java/JDK is not installed or not in PATH".to_string());
    }

    PlatformEnv {
        available: android_home.is_some(),
        sdk_path,
        sdk_version,
        issues,
    }
}

async fn check_ios_platform() -> PlatformEnv {
    let mut issues = Vec::new();
    let xcode_path = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let xcode_version = std::process::Command::new("xcodebuild")
        .arg("-version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find(|l| l.contains("Xcode"))
                .map(|s| {
                    s.trim()
                        .replace("Xcode ", "")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string()
                })
        });

    match &xcode_path {
        Some(p) if p.is_empty() => {
            issues.push("Xcode Command Line Tools are not configured".to_string());
        }
        None => {
            issues.push("Xcode is not installed".to_string());
        }
        _ => {}
    }

    PlatformEnv {
        available: xcode_path.is_some() && xcode_path.as_deref().unwrap_or("").is_empty() == false,
        sdk_path: xcode_path,
        sdk_version: xcode_version.or(Some("detected".to_string())),
        issues,
    }
}

async fn check_harmony_platform() -> PlatformEnv {
    let mut issues = Vec::new();

    let ohpm_installed = std::process::Command::new("ohpm")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let hvigorw_path = which_tool("hvigorw");

    if !ohpm_installed && hvigorw_path.is_none() {
        issues
            .push("HarmonyOS SDK is not installed or DevEco Studio is not configured".to_string());
    }

    PlatformEnv {
        available: ohpm_installed || hvigorw_path.is_some(),
        sdk_path: hvigorw_path,
        sdk_version: if ohpm_installed {
            Some("detected".to_string())
        } else {
            None
        },
        issues,
    }
}

#[tauri::command]
pub async fn check_android_env() -> Result<PlatformEnv, String> {
    Ok(check_android_platform().await)
}

#[tauri::command]
pub async fn check_ios_env() -> Result<PlatformEnv, String> {
    Ok(check_ios_platform().await)
}

#[tauri::command]
pub async fn check_harmony_env() -> Result<PlatformEnv, String> {
    Ok(check_harmony_platform().await)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolValidationResult {
    pub valid: bool,
    pub tool_name: String,
    pub version: Option<String>,
    pub actual_path: String,
    pub details: Vec<String>,
    pub errors: Vec<String>,
    pub set_env_var: Option<String>,
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

fn normalize_macos_app_path(path: &PathBuf) -> PathBuf {
    let mut current = path.as_path();
    loop {
        if current.extension().and_then(|ext| ext.to_str()) == Some("app") {
            return current.to_path_buf();
        }
        let Some(parent) = current.parent() else {
            return path.clone();
        };
        current = parent;
    }
}

fn is_macos_app_path(path: &PathBuf) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("app")
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

fn extract_version_from_output(output: &[u8], keyword: &str) -> Option<String> {
    String::from_utf8_lossy(output)
        .lines()
        .find(|l| l.contains(keyword))
        .and_then(|l| {
            l.split_whitespace()
                .find(|w| {
                    w.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                        || w.starts_with('v') && w.len() > 1
                })
                .or_else(|| l.rsplit(' ').next())
                .map(|s| s.trim().to_string())
        })
}

#[tauri::command]
pub async fn validate_tool_path(
    tool_name: String,
    path: String,
) -> Result<ToolValidationResult, String> {
    let input = path.trim();
    if input.is_empty() {
        return Ok(ToolValidationResult {
            valid: false,
            tool_name: tool_name.clone(),
            version: None,
            actual_path: input.to_string(),
            details: vec![],
            errors: vec!["路径不能为空".to_string()],
            set_env_var: None,
        });
    }

    let mut p = PathBuf::from(input);
    if matches!(tool_name.as_str(), "xcode" | "android_studio" | "hbuilderx") {
        p = normalize_macos_app_path(&p);
    }
    let mut details = Vec::new();
    let mut errors = Vec::new();
    let mut version = None;
    let mut set_env_var = None;

    match tool_name.as_str() {
        "android_sdk" => {
            set_env_var = Some("ANDROID_HOME".to_string());
            if !p.exists() {
                errors.push(format!("目录不存在: {}", input));
            } else {
                details.push(format!("✓ 目录存在"));
                let platforms = p.join("platforms");
                if platforms.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&platforms) {
                        let apis: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect();
                        if apis.is_empty() {
                            errors.push("platforms/ 目录为空，未找到 Android API 平台".to_string());
                        } else {
                            details.push(format!("✓ 检测到 {} 个API平台", apis.len()));
                        }
                    }
                } else {
                    errors.push("缺少 platforms/ 子目录".to_string());
                }

                let bt = p.join("build-tools");
                if bt.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&bt) {
                        let versions: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect();
                        if versions.is_empty() {
                            errors.push("build-tools/ 目录为空".to_string());
                        } else {
                            details.push(format!("✓ Build Tools: {}", versions.join(", ")));
                        }
                    }
                } else {
                    errors.push("缺少 build-tools/ 子目录".to_string());
                }

                if p.join("platform-tools").exists() || p.join("platform-tools/adb").exists() {
                    details.push("✓ platform-tools 存在".to_string());
                }

                if errors.is_empty() {
                    version = p
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string());
                }
            }
        }

        "java" => {
            set_env_var = Some("JAVA_HOME".to_string());
            let java_bin = resolve_bin_path(input, "java");
            match java_bin {
                Some(ref bin) => {
                    details.push(format!("✓ 找到 java: {}", bin.display()));
                    if let Ok(output) = std::process::Command::new(bin).arg("-version").output() {
                        if output.status.success() {
                            let v = extract_version_from_output(&output.stdout, "\"")
                                .or_else(|| extract_version_from_output(&output.stderr, "\""))
                                .or_else(|| {
                                    String::from_utf8_lossy(&output.stderr)
                                        .lines()
                                        .find(|l| l.contains("version"))
                                        .and_then(|l| l.split('"').nth(1).map(|s| s.to_string()))
                                });
                            if let Some(ref v) = v {
                                version = Some(v.clone());
                                details.push(format!("✓ 版本: {}", v));
                            } else {
                                details.push("✓ java 可执行但无法解析版本".to_string());
                            }
                        } else {
                            errors.push("java -version 执行失败".to_string());
                        }
                    }
                }
                None => {
                    errors.push(format!("未找到 java 可执行文件: {}", input));
                }
            }
        }

        "xcode" => {
            set_env_var = Some("DEVELOPER_DIR".to_string());
            if is_macos_app_path(&p) && p.exists() {
                details.push(format!("✓ Xcode.app 存在"));
                let xcodebuild = p
                    .join("Contents")
                    .join("Developer")
                    .join("usr")
                    .join("bin")
                    .join("xcodebuild");
                if xcodebuild.exists() {
                    details.push("✓ xcodebuild 存在".to_string());
                    if let Ok(output) = std::process::Command::new(&xcodebuild)
                        .arg("-version")
                        .output()
                    {
                        if output.status.success() {
                            let v = extract_version_from_output(&output.stdout, "Xcode");
                            if let Some(v) = v {
                                version = Some(v.clone());
                                details.push(format!("✓ Xcode 版本: {}", v));
                            }
                        }
                    }
                } else {
                    errors.push("xcodebuild 未找到".to_string());
                }
            } else {
                let xcodebuild = resolve_bin_path(input, "xcodebuild");
                match xcodebuild {
                    Some(ref bin) => {
                        details.push(format!("✓ 找到 xcodebuild: {}", bin.display()));
                        if let Ok(output) = std::process::Command::new(bin).arg("-version").output()
                        {
                            if output.status.success() {
                                let v = extract_version_from_output(&output.stdout, "Xcode");
                                if let Some(v) = v {
                                    version = Some(v.clone());
                                    details.push(format!("✓ 版本: {}", v));
                                }
                            }
                        }
                    }
                    None => {
                        errors.push(format!("无效的 Xcode 路径: {}", input));
                    }
                }
            }
        }

        "ndk" => {
            set_env_var = Some("NDK_HOME".to_string());
            if p.exists() {
                details.push("✓ NDK 目录存在".to_string());
                let source_props = p.join("source.properties");
                if source_props.exists() {
                    if let Ok(content) = std::fs::read_to_string(&source_props) {
                        for line in content.lines() {
                            if line.starts_with("Pkg.Revision") {
                                if let Some(v) = line.split('=').nth(1) {
                                    version = Some(v.trim().to_string());
                                    details.push(format!("✓ NDK 版本: {}", v.trim()));
                                }
                            }
                        }
                    }
                }
                let ndk_build = p.join("ndk-build");
                if ndk_build.exists() {
                    details.push("✓ ndk-build 存在".to_string());
                }
                let toolchains = p.join("toolchains");
                if toolchains.is_dir() {
                    details.push("✓ toolchains 目录存在".to_string());
                }
                if version.is_none() && (ndk_build.exists() || toolchains.is_dir()) {
                    details.push("✓ 看起来是有效的 NDK 目录（版本未知）".to_string());
                }
                if !ndk_build.exists() && !source_props.exists() && !toolchains.is_dir() {
                    errors.push("目录存在但未找到 NDK 特征文件".to_string());
                }
            } else {
                errors.push(format!("NDK 路径不存在: {}", input));
            }
        }

        "android_studio" => {
            if is_macos_app_path(&p) && p.exists() {
                details.push(format!("✓ Android Studio.app 存在"));
                let plist = p.join("Contents").join("Info.plist");
                if plist.exists() {
                    if let Ok(content) = std::fs::read_to_string(&plist) {
                        if let Some(v) = content
                            .lines()
                            .find(|l| l.contains("CFBundleShortVersionString"))
                            .and_then(|l| l.split('>').last()?.split('<').next())
                        {
                            version = Some(v.to_string());
                            details.push(format!("✓ 版本: {}", v));
                        }
                    }
                }
            } else {
                errors.push(format!("Android Studio 路径无效（应为 .app）: {}", input));
            }
        }

        "gradle" => {
            let gradle_bin = resolve_bin_path(input, "gradle").or_else(|| {
                let gw = p.join("gradlew");
                if gw.exists() {
                    Some(gw)
                } else {
                    None
                }
            });
            match gradle_bin {
                Some(ref bin) => {
                    details.push(format!("✓ 找到 gradle: {}", bin.display()));
                    if let Ok(output) = std::process::Command::new(bin).arg("-v").output() {
                        if output.status.success() {
                            let v = extract_version_from_output(&output.stdout, "Gradle");
                            if let Some(v) = v {
                                version = Some(v.clone());
                                details.push(format!("✓ Gradle 版本: {}", v));
                            }
                        }
                    }
                }
                None => {
                    errors.push(format!("未找到 gradle 或 gradlew: {}", input));
                }
            }
        }

        "cocoapods" => {
            let pod_bin = resolve_bin_path(input, "pod");
            match pod_bin {
                Some(ref bin) => {
                    details.push(format!("✓ 找到 pod: {}", bin.display()));
                    if let Ok(output) = std::process::Command::new(bin).arg("--version").output() {
                        if output.status.success() {
                            let v = String::from_utf8_lossy(&output.stdout)
                                .lines()
                                .next()
                                .map(|s| s.trim().to_string());
                            if let Some(v) = v {
                                version = Some(v.clone());
                                details.push(format!("✓ CocoaPods 版本: {}", v));
                            }
                        }
                    }
                }
                None => {
                    errors.push(format!("未找到 pod: {}", input));
                }
            }
        }

        "hbuilderx" => {
            if is_macos_app_path(&p) && p.exists() {
                details.push("✓ HBuilderX.app 存在".to_string());
                let plist = p.join("Contents").join("Info.plist");
                if plist.exists() {
                    if let Ok(content) = std::fs::read_to_string(&plist) {
                        if let Some(v) = content
                            .lines()
                            .find(|l| l.contains("CFBundleShortVersionString"))
                            .and_then(|l| l.split('>').last()?.split('<').next())
                        {
                            version = Some(v.to_string());
                            details.push(format!("✓ HBuilderX 版本: {}", v));
                        }
                    }
                }
                if p.join("plugins").join("uniapp-cli").exists() {
                    details.push("✓ uniapp-cli 插件就绪".to_string());
                }
            } else {
                errors.push(format!("HBuilderX 路径无效（应为 .app）: {}", input));
            }
        }

        "node" => {
            let node_bin = resolve_bin_path(input, "node");
            match node_bin {
                Some(ref bin) => {
                    details.push(format!("✓ 找到 node: {}", bin.display()));
                    if let Ok(output) = std::process::Command::new(bin).arg("--version").output() {
                        if output.status.success() {
                            let v = String::from_utf8_lossy(&output.stdout)
                                .lines()
                                .next()
                                .map(|s| s.trim().to_string());
                            if let Some(v) = v {
                                version = Some(v.clone());
                                details.push(format!("✓ Node.js 版本: {}", v));
                            }
                        }
                    }
                }
                None => {
                    errors.push(format!("未找到 node: {}", input));
                }
            }
        }

        "git" => {
            let git_bin = resolve_bin_path(input, "git");
            match git_bin {
                Some(ref bin) => {
                    details.push(format!("✓ 找到 git: {}", bin.display()));
                    if let Ok(output) = std::process::Command::new(bin).arg("--version").output() {
                        if output.status.success() {
                            let v = String::from_utf8_lossy(&output.stdout)
                                .lines()
                                .next()
                                .map(|s| s.trim().to_string());
                            if let Some(v) = v {
                                version = Some(v.clone());
                                details.push(format!("✓ Git 版本: {}", v));
                            }
                        }
                    }
                }
                None => {
                    errors.push(format!("未找到 git: {}", input));
                }
            }
        }

        _ => {
            return Err(format!("不支持的工具类型: {}，支持的类型: android_sdk, java, xcode, ndk, android_studio, gradle, cocoapods, hbuilderx, node, git", tool_name));
        }
    }

    Ok(ToolValidationResult {
        valid: errors.is_empty(),
        tool_name,
        version,
        actual_path: p.canonicalize().unwrap_or(p).to_string_lossy().to_string(),
        details,
        errors,
        set_env_var,
    })
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
