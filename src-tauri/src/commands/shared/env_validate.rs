use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

fn normalize_macos_app_path(path: &Path) -> PathBuf {
    let mut current = path;
    loop {
        if current.extension().and_then(|ext| ext.to_str()) == Some("app") {
            return current.to_path_buf();
        }
        let Some(parent) = current.parent() else {
            return path.to_path_buf();
        };
        current = parent;
    }
}

fn is_macos_app_path(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("app")
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
    if matches!(
        tool_name.as_str(),
        "xcode" | "android_studio" | "hbuilderx" | "harmony"
    ) {
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
                details.push("✓ 目录存在".to_string());
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
                details.push("✓ Xcode.app 存在".to_string());
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
                details.push("✓ Android Studio.app 存在".to_string());
                let plist = p.join("Contents").join("Info.plist");
                if plist.exists() {
                    if let Ok(content) = std::fs::read_to_string(&plist) {
                        if let Some(v) = content
                            .lines()
                            .find(|l| l.contains("CFBundleShortVersionString"))
                            .and_then(|l| l.split('>').next_back()?.split('<').next())
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
                            .and_then(|l| l.split('>').next_back()?.split('<').next())
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

        "harmony" => {
            if is_macos_app_path(&p) && p.exists() {
                details.push("✓ DevEco Studio.app 存在".to_string());
                let plist = p.join("Contents").join("Info.plist");
                if plist.exists() {
                    if let Ok(content) = std::fs::read_to_string(&plist) {
                        if let Some(v) = content
                            .lines()
                            .find(|l| l.contains("CFBundleShortVersionString"))
                            .and_then(|l| l.split('>').next_back()?.split('<').next())
                        {
                            version = Some(v.to_string());
                            details.push(format!("✓ 版本: {}", v));
                        }
                    }
                }
            } else {
                errors.push(format!("DevEco Studio 路径无效（应为 .app）: {}", input));
            }
        }

        _ => {
            return Err(format!("不支持的工具类型: {}，支持的类型: android_sdk, java, xcode, ndk, android_studio, gradle, cocoapods, hbuilderx, harmony, node, git", tool_name));
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
