use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::sdk_alignment::{
    ios_static_library_needs_alignment_repair, repair_ios_static_library_alignment,
};

#[derive(Debug, Clone)]
pub(super) struct IosSdkSupportPreparation {
    pub(super) path: PathBuf,
    pub(super) copied_for_repair: bool,
    pub(super) repaired_libraries: Vec<String>,
    pub(super) logs: Vec<IosSdkSupportLog>,
}

#[derive(Debug, Clone)]
pub(super) struct IosSdkSupportLog {
    pub(super) level: &'static str,
    pub(super) message: String,
}

impl IosSdkSupportLog {
    pub(super) fn info(message: impl Into<String>) -> Self {
        Self {
            level: "info",
            message: message.into(),
        }
    }

    pub(super) fn success(message: impl Into<String>) -> Self {
        Self {
            level: "success",
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct IosProjectDependencyReferences {
    sdk_static_libraries: Vec<PathBuf>,
    sdk_binary_dependencies: Vec<String>,
    project_binary_dependencies: Vec<String>,
    resources: Vec<String>,
    system_dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
struct PbxFileReference {
    name: String,
    path: String,
    source_tree: String,
}

pub(super) fn link_ios_sdk_support(
    sdk_project: &Path,
    workspace: &Path,
) -> Result<Option<PathBuf>, String> {
    let Some(sdk_root) = sdk_project.parent() else {
        return Ok(None);
    };
    let support_source = sdk_root.join("SDK");
    if !support_source.is_dir() {
        return Ok(None);
    }

    let support_dest = workspace.join("SDK");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&support_source, &support_dest).map_err(|e| {
        format!(
            "关联 iOS SDK 支持目录失败 {} -> {}: {}",
            support_dest.display(),
            support_source.display(),
            e
        )
    })?;
    #[cfg(not(unix))]
    crate::utils::fs::copy_recursive(&support_source, &support_dest).map_err(|e| {
        format!(
            "复制 iOS SDK 支持目录失败 {} -> {}: {}",
            support_source.display(),
            support_dest.display(),
            e
        )
    })?;

    Ok(Some(support_dest))
}

pub(super) fn prepare_ios_sdk_support(
    sdk_project: &Path,
    workspace: &Path,
) -> Result<Option<IosSdkSupportPreparation>, String> {
    let Some(sdk_root) = sdk_project.parent() else {
        return Ok(None);
    };
    let support_source = sdk_root.join("SDK");
    if !support_source.is_dir() {
        return Ok(None);
    }

    let path = link_ios_sdk_support(sdk_project, workspace)?
        .ok_or_else(|| "iOS SDK 支持目录不存在".to_string())?;
    Ok(Some(IosSdkSupportPreparation {
        path,
        copied_for_repair: false,
        repaired_libraries: Vec::new(),
        logs: vec![IosSdkSupportLog::info(
            "已准备 iOS SDK 支持目录，将在工程配置完成后按实际引用检测 alignment 修复需求",
        )],
    }))
}

pub(super) fn materialize_ios_sdk_support_for_pod(
    sdk_project: &Path,
    workspace: &Path,
) -> Result<Option<IosSdkSupportPreparation>, String> {
    let Some(sdk_root) = sdk_project.parent() else {
        return Ok(None);
    };
    let support_source = sdk_root.join("SDK");
    if !support_source.is_dir() {
        return Ok(None);
    }
    let support_dest = workspace.join("SDK");
    let mut logs = vec![IosSdkSupportLog::info(
        "Pod 模式按 HBuilderX 官方目录结构使用 workspace 内 SDK 实体目录，确保 CocoaPods 能识别 uniapp.podspec 中的本地库",
    )];
    ensure_ios_sdk_support_workspace_copy(
        &support_source,
        &support_dest,
        &mut logs,
        "已复制 iOS SDK 支持目录到 workspace 副本用于 Pod 集成",
    )?;
    Ok(Some(IosSdkSupportPreparation {
        path: support_dest,
        copied_for_repair: true,
        repaired_libraries: Vec::new(),
        logs,
    }))
}

pub(super) fn repair_ios_sdk_support_alignment_for_project(
    sdk_project: &Path,
    workspace: &Path,
    project_file: &Path,
) -> Result<Option<IosSdkSupportPreparation>, String> {
    let Some(sdk_root) = sdk_project.parent() else {
        return Ok(None);
    };
    let support_source = sdk_root.join("SDK");
    if !support_source.is_dir() {
        return Ok(None);
    }
    let support_dest = workspace.join("SDK");

    let dependencies = ios_project_dependency_references(project_file)?;
    let mut logs = vec![IosSdkSupportLog::info(
        "检测 iOS 工程实际引用的依赖库、系统库和资源",
    )];
    logs.push(IosSdkSupportLog::info(format!(
        "iOS 工程依赖扫描完成: SDK 静态库 {} 项，SDK framework/xcframework {} 项，工程内依赖库 {} 项，资源 {} 项，系统库/framework {} 项",
        dependencies.sdk_static_libraries.len(),
        dependencies.sdk_binary_dependencies.len(),
        dependencies.project_binary_dependencies.len(),
        dependencies.resources.len(),
        dependencies.system_dependencies.len()
    )));
    if dependencies.sdk_static_libraries.is_empty() {
        logs.push(IosSdkSupportLog::info(
            "未发现工程实际引用的 iOS SDK 静态库，跳过 alignment 修复检测",
        ));
        return Ok(Some(IosSdkSupportPreparation {
            path: support_dest,
            copied_for_repair: false,
            repaired_libraries: Vec::new(),
            logs,
        }));
    }
    logs.push(IosSdkSupportLog::info(format!(
        "检测 iOS SDK 静态库 8-byte alignment 修复需求: {}",
        summarize_paths(&dependencies.sdk_static_libraries)
    )));

    let repair_candidates = ios_static_libraries_requiring_alignment_repair(
        &support_dest,
        &dependencies.sdk_static_libraries,
    )?;
    if repair_candidates.is_empty() {
        logs.push(IosSdkSupportLog::info(
            "未检测到工程实际使用的 iOS SDK 静态库需要修复，跳过 alignment 修复流程",
        ));
        return Ok(Some(IosSdkSupportPreparation {
            path: support_dest,
            copied_for_repair: false,
            repaired_libraries: Vec::new(),
            logs,
        }));
    }

    logs.push(IosSdkSupportLog::info(format!(
        "检测到 {} 个工程实际使用的 iOS SDK 静态库需要修复: {}",
        repair_candidates.len(),
        summarize_paths(&repair_candidates)
    )));
    ensure_ios_sdk_support_workspace_copy(
        &support_source,
        &support_dest,
        &mut logs,
        "已复制 iOS SDK 支持目录到 workspace 副本用于修复",
    )?;

    let mut repaired_libraries = Vec::new();
    for source in repair_candidates {
        let relative = source.strip_prefix(&support_dest).map_err(|e| {
            format!(
                "计算 iOS SDK 静态库相对路径失败 {}: {}",
                source.display(),
                e
            )
        })?;
        let target = support_dest.join(relative);
        let library_name = target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        repair_ios_static_library_alignment(&target, &mut logs)?;
        repaired_libraries.push(library_name);
    }

    logs.push(IosSdkSupportLog::success(format!(
        "iOS SDK 静态库 8-byte alignment 修复完成: {}",
        repaired_libraries.join("、")
    )));
    Ok(Some(IosSdkSupportPreparation {
        path: support_dest,
        copied_for_repair: true,
        repaired_libraries,
        logs,
    }))
}

fn ios_static_libraries_requiring_alignment_repair(
    support_root: &Path,
    referenced_libraries: &[PathBuf],
) -> Result<Vec<PathBuf>, String> {
    let mut libraries = Vec::new();
    for relative in referenced_libraries {
        let path = resolve_sdk_relative_path(support_root, relative);
        if path.is_file() && ios_static_library_needs_alignment_repair(&path)? {
            libraries.push(path);
        }
    }
    Ok(libraries)
}

fn ensure_ios_sdk_support_workspace_copy(
    support_source: &Path,
    support_dest: &Path,
    logs: &mut Vec<IosSdkSupportLog>,
    copy_message: &str,
) -> Result<(), String> {
    match std::fs::symlink_metadata(support_dest) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            std::fs::remove_file(support_dest).map_err(|e| {
                format!(
                    "移除 iOS SDK 支持目录符号链接失败 {}: {}",
                    support_dest.display(),
                    e
                )
            })?;
            copy_ios_sdk_support_directory(support_source, support_dest)?;
            logs.push(IosSdkSupportLog::info(format!(
                "{}: {}",
                copy_message,
                support_dest.display()
            )));
        }
        Ok(metadata) if metadata.is_dir() => {
            logs.push(IosSdkSupportLog::info(format!(
                "使用 iOS SDK 支持目录 workspace 副本执行修复: {}",
                support_dest.display()
            )));
        }
        Ok(_) => {
            std::fs::remove_file(support_dest).map_err(|e| {
                format!(
                    "清理 iOS SDK 支持目录占位文件失败 {}: {}",
                    support_dest.display(),
                    e
                )
            })?;
            copy_ios_sdk_support_directory(support_source, support_dest)?;
            logs.push(IosSdkSupportLog::info(format!(
                "{}: {}",
                copy_message,
                support_dest.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            copy_ios_sdk_support_directory(support_source, support_dest)?;
            logs.push(IosSdkSupportLog::info(format!(
                "{}: {}",
                copy_message,
                support_dest.display()
            )));
        }
        Err(error) => {
            return Err(format!(
                "读取 iOS SDK 支持目录状态失败 {}: {}",
                support_dest.display(),
                error
            ));
        }
    }

    crate::utils::fs::ensure_writable_tree(support_dest).map_err(|e| {
        format!(
            "设置 iOS SDK 支持目录可写失败 {}: {}",
            support_dest.display(),
            e
        )
    })
}

fn copy_ios_sdk_support_directory(
    support_source: &Path,
    support_dest: &Path,
) -> Result<(), String> {
    if let Some(parent) = support_dest.parent() {
        crate::utils::fs::ensure_directory(parent)
            .map_err(|e| format!("创建 iOS SDK 支持目录父级失败 {}: {}", parent.display(), e))?;
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/bin/cp")
            .arg("-R")
            .arg("-c")
            .arg(support_source)
            .arg(support_dest)
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                return Ok(());
            }
        }
    }

    crate::utils::fs::copy_recursive(support_source, support_dest).map_err(|e| {
        format!(
            "复制 iOS SDK 支持目录失败 {} -> {}: {}",
            support_source.display(),
            support_dest.display(),
            e
        )
    })
}

fn ios_project_dependency_references(
    project_file: &Path,
) -> Result<IosProjectDependencyReferences, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    Ok(ios_project_dependency_references_from_content(&content))
}

fn ios_project_dependency_references_from_content(content: &str) -> IosProjectDependencyReferences {
    let file_references = parse_pbx_file_references(content);
    let build_file_refs = parse_pbx_build_file_refs(content);
    let mut sdk_static_libraries = BTreeSet::new();
    let mut sdk_binary_dependencies = BTreeSet::new();
    let mut project_binary_dependencies = BTreeSet::new();
    let mut resources = BTreeSet::new();
    let mut system_dependencies = BTreeSet::new();

    for file_ref_id in build_file_refs {
        let Some(file_ref) = file_references.get(&file_ref_id) else {
            continue;
        };
        let path = file_ref.path.trim();
        let display_name = if file_ref.name.trim().is_empty() {
            path.to_string()
        } else {
            file_ref.name.clone()
        };

        if let Some(relative) = sdk_relative_path(path) {
            if is_static_library_path(&relative) {
                sdk_static_libraries.insert(PathBuf::from(relative));
            } else if is_binary_dependency_path(&relative) {
                sdk_binary_dependencies.insert(display_name);
            } else {
                resources.insert(display_name);
            }
            continue;
        }

        if is_system_dependency(path, &file_ref.source_tree) {
            system_dependencies.insert(display_name);
        } else if is_binary_dependency_path(path) {
            project_binary_dependencies.insert(display_name);
        } else {
            resources.insert(display_name);
        }
    }

    IosProjectDependencyReferences {
        sdk_static_libraries: sdk_static_libraries.into_iter().collect(),
        sdk_binary_dependencies: sdk_binary_dependencies.into_iter().collect(),
        project_binary_dependencies: project_binary_dependencies.into_iter().collect(),
        resources: resources.into_iter().collect(),
        system_dependencies: system_dependencies.into_iter().collect(),
    }
}

fn parse_pbx_file_references(content: &str) -> BTreeMap<String, PbxFileReference> {
    let pattern = regex::Regex::new(
        r#"(?m)^\s*([A-Za-z0-9]{24}) /\* (.*?) \*/ = \{isa = PBXFileReference;([^\n]*?)\};"#,
    )
    .expect("valid PBXFileReference regex");
    pattern
        .captures_iter(content)
        .filter_map(|captures| {
            let id = captures.get(1)?.as_str().to_string();
            let comment_name = captures.get(2)?.as_str().trim().to_string();
            let body = captures.get(3)?.as_str();
            let path = pbx_field_value(body, "path").unwrap_or_else(|| comment_name.clone());
            let name = pbx_field_value(body, "name").unwrap_or(comment_name);
            let source_tree = pbx_field_value(body, "sourceTree").unwrap_or_default();
            Some((
                id,
                PbxFileReference {
                    name,
                    path,
                    source_tree,
                },
            ))
        })
        .collect()
}

fn parse_pbx_build_file_refs(content: &str) -> BTreeSet<String> {
    let pattern = regex::Regex::new(
        r#"(?m)^\s*[A-Za-z0-9]{24} /\* .*? \*/ = \{isa = PBXBuildFile; fileRef = ([A-Za-z0-9]{24}) /\* .*? \*/;"#,
    )
    .expect("valid PBXBuildFile regex");
    pattern
        .captures_iter(content)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect()
}

fn pbx_field_value(body: &str, key: &str) -> Option<String> {
    let pattern = regex::Regex::new(&format!(
        r#"\b{}\s*=\s*("[^"]*"|[^;]+);"#,
        regex::escape(key)
    ))
    .ok()?;
    let value = pattern.captures(body)?.get(1)?.as_str().trim();
    Some(unquote_pbx_value(value))
}

fn unquote_pbx_value(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
        .replace("\\\"", "\"")
}

fn sdk_relative_path(path: &str) -> Option<String> {
    let normalized = normalize_pbx_path(path);
    normalized
        .strip_prefix("../SDK/")
        .or_else(|| normalized.strip_prefix("SDK/"))
        .or_else(|| normalized.split_once("/SDK/").map(|(_, rest)| rest))
        .filter(|rest| !rest.is_empty())
        .map(ToString::to_string)
}

fn normalize_pbx_path(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('\\', "/")
        .replace("$(SRCROOT)/", "")
        .replace("${SRCROOT}/", "")
}

fn is_static_library_path(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".a")
}

fn is_binary_dependency_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".a")
        || lower.ends_with(".framework")
        || lower.ends_with(".xcframework")
        || lower.ends_with(".tbd")
        || lower.ends_with(".dylib")
}

fn is_system_dependency(path: &str, source_tree: &str) -> bool {
    let normalized = normalize_pbx_path(path);
    source_tree == "SDKROOT"
        || normalized.starts_with("System/Library/Frameworks/")
        || normalized.starts_with("usr/lib/")
}

fn resolve_sdk_relative_path(support_root: &Path, relative: &Path) -> PathBuf {
    let direct = support_root.join(relative);
    if direct.exists() {
        return direct;
    }

    let relative_text = relative.to_string_lossy().replace('\\', "/");
    let swapped = if let Some(rest) = relative_text.strip_prefix("Libs/") {
        Some(format!("libs/{}", rest))
    } else {
        relative_text
            .strip_prefix("libs/")
            .map(|rest| format!("Libs/{}", rest))
    };
    swapped
        .map(|path| support_root.join(path))
        .filter(|path| path.exists())
        .unwrap_or(direct)
}

fn summarize_paths(paths: &[PathBuf]) -> String {
    let names = paths
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(ToString::to_string)
                .unwrap_or_else(|| path.display().to_string())
        })
        .collect::<Vec<_>>();
    summarize_names(&names)
}

fn summarize_names(names: &[String]) -> String {
    const MAX_INLINE_NAMES: usize = 12;
    if names.is_empty() {
        return "无".to_string();
    }
    if names.len() <= MAX_INLINE_NAMES {
        return names.join("、");
    }
    format!(
        "{} 等 {} 项",
        names[..MAX_INLINE_NAMES].join("、"),
        names.len()
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ios_project_dependency_references_from_content, sdk_relative_path};

    #[test]
    fn parses_actual_project_dependency_references_from_build_files() {
        let content = r#"/* Begin PBXBuildFile section */
		AAAAAAAAAAAAAAAAAAAAAAAA /* libUsed.a in Frameworks */ = {isa = PBXBuildFile; fileRef = BBBBBBBBBBBBBBBBBBBBBBBB /* libUsed.a */; };
		CCCCCCCCCCCCCCCCCCCCCCCC /* DCUniBase.framework in Frameworks */ = {isa = PBXBuildFile; fileRef = DDDDDDDDDDDDDDDDDDDDDDDD /* DCUniBase.framework */; };
		EEEEEEEEEEEEEEEEEEEEEEEE /* UserNotifications.framework in Frameworks */ = {isa = PBXBuildFile; fileRef = FFFFFFFFFFFFFFFFFFFFFFFF /* UserNotifications.framework */; };
		111111111111111111111111 /* Plugin.framework in Embed Frameworks */ = {isa = PBXBuildFile; fileRef = 222222222222222222222222 /* Plugin.framework */; };
		333333333333333333333333 /* SDKResource.bundle in Resources */ = {isa = PBXBuildFile; fileRef = 444444444444444444444444 /* SDKResource.bundle */; };
/* End PBXBuildFile section */
/* Begin PBXFileReference section */
		BBBBBBBBBBBBBBBBBBBBBBBB /* libUsed.a */ = {isa = PBXFileReference; lastKnownFileType = archive.ar; name = libUsed.a; path = ../SDK/Libs/libUsed.a; sourceTree = "<group>"; };
		999999999999999999999999 /* libUnused.a */ = {isa = PBXFileReference; lastKnownFileType = archive.ar; name = libUnused.a; path = ../SDK/Libs/libUnused.a; sourceTree = "<group>"; };
		DDDDDDDDDDDDDDDDDDDDDDDD /* DCUniBase.framework */ = {isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = DCUniBase.framework; path = ../SDK/Libs/DCUniBase.framework; sourceTree = "<group>"; };
		FFFFFFFFFFFFFFFFFFFFFFFF /* UserNotifications.framework */ = {isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = UserNotifications.framework; path = System/Library/Frameworks/UserNotifications.framework; sourceTree = SDKROOT; };
		222222222222222222222222 /* Plugin.framework */ = {isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = Plugin.framework; path = UTSPlugins/demo/Plugin.framework; sourceTree = "<group>"; };
		444444444444444444444444 /* SDKResource.bundle */ = {isa = PBXFileReference; lastKnownFileType = "wrapper.plug-in"; name = SDKResource.bundle; path = ../SDK/Bundles/SDKResource.bundle; sourceTree = "<group>"; };
/* End PBXFileReference section */
"#;

        let references = ios_project_dependency_references_from_content(content);

        assert_eq!(
            references.sdk_static_libraries,
            vec![PathBuf::from("Libs/libUsed.a")]
        );
        assert_eq!(
            references.sdk_binary_dependencies,
            vec!["DCUniBase.framework".to_string()]
        );
        assert_eq!(
            references.project_binary_dependencies,
            vec!["Plugin.framework".to_string()]
        );
        assert_eq!(references.resources, vec!["SDKResource.bundle".to_string()]);
        assert_eq!(
            references.system_dependencies,
            vec!["UserNotifications.framework".to_string()]
        );
    }

    #[test]
    fn normalizes_sdk_relative_paths() {
        assert_eq!(
            sdk_relative_path("../SDK/Libs/libA.a").as_deref(),
            Some("Libs/libA.a")
        );
        assert_eq!(
            sdk_relative_path("$(SRCROOT)/../SDK/libs/libB.a").as_deref(),
            Some("libs/libB.a")
        );
        assert_eq!(sdk_relative_path("UTSPlugins/Plugin.framework"), None);
    }
}
