use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

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
    fn info(message: impl Into<String>) -> Self {
        Self {
            level: "info",
            message: message.into(),
        }
    }

    fn success(message: impl Into<String>) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct LipoArchitectureSlice {
    name: String,
    offset: usize,
}

pub(super) fn clean_copied_project(project_root: &Path) -> Result<(), String> {
    for path in [
        project_root.join("build"),
        project_root.join("DerivedData"),
        project_root.join(".build"),
    ] {
        if path.exists() {
            std::fs::remove_dir_all(&path)
                .map_err(|e| format!("清理 iOS 工程旧构建产物失败 {}: {}", path.display(), e))?;
        }
    }
    Ok(())
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
    ensure_ios_sdk_support_workspace_copy(&support_source, &support_dest, &mut logs)?;

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
            crate::utils::fs::copy_recursive(support_source, support_dest).map_err(|e| {
                format!(
                    "复制 iOS SDK 支持目录失败 {} -> {}: {}",
                    support_source.display(),
                    support_dest.display(),
                    e
                )
            })?;
            logs.push(IosSdkSupportLog::info(format!(
                "已复制 iOS SDK 支持目录到 workspace 副本用于修复: {}",
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
            crate::utils::fs::copy_recursive(support_source, support_dest).map_err(|e| {
                format!(
                    "复制 iOS SDK 支持目录失败 {} -> {}: {}",
                    support_source.display(),
                    support_dest.display(),
                    e
                )
            })?;
            logs.push(IosSdkSupportLog::info(format!(
                "已复制 iOS SDK 支持目录到 workspace 副本用于修复: {}",
                support_dest.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            crate::utils::fs::copy_recursive(support_source, support_dest).map_err(|e| {
                format!(
                    "复制 iOS SDK 支持目录失败 {} -> {}: {}",
                    support_source.display(),
                    support_dest.display(),
                    e
                )
            })?;
            logs.push(IosSdkSupportLog::info(format!(
                "已复制 iOS SDK 支持目录到 workspace 副本用于修复: {}",
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

fn ios_static_library_needs_alignment_repair(path: &Path) -> Result<bool, String> {
    let slices = lipo_architecture_slices(path)?;
    let work_dir = std::env::temp_dir().join(format!(
        "unipack-ios-align-check-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&work_dir).map_err(|e| {
        format!(
            "创建 iOS 静态库检测临时目录失败 {}: {}",
            work_dir.display(),
            e
        )
    })?;
    let result = (|| {
        if slices.is_empty() {
            return archive_has_unaligned_macho_members(path);
        }
        for slice in slices {
            let thin = work_dir.join(format!("{}.a", slice.name));
            run_command(
                Command::new("xcrun")
                    .arg("lipo")
                    .arg(path)
                    .arg("-thin")
                    .arg(&slice.name)
                    .arg("-output")
                    .arg(&thin),
                &format!("提取 iOS 静态库架构 {} 失败", slice.name),
            )?;
            if archive_has_unaligned_macho_members_with_base_offset(&thin, slice.offset)? {
                return Ok(true);
            }
        }
        Ok(false)
    })();
    let _ = std::fs::remove_dir_all(&work_dir);
    result
}

fn repair_ios_static_library_alignment(
    path: &Path,
    logs: &mut Vec<IosSdkSupportLog>,
) -> Result<(), String> {
    let library_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_string();
    logs.push(IosSdkSupportLog::info(format!(
        "开始修复 iOS 静态库对齐: {}",
        library_name
    )));
    make_file_writable(path)?;

    let backup = path.with_extension("a.bak");
    if backup.exists() {
        logs.push(IosSdkSupportLog::info(format!(
            "{} 备份已存在，跳过备份",
            library_name
        )));
    } else {
        std::fs::copy(path, &backup).map_err(|e| {
            format!(
                "备份 iOS 静态库失败 {} -> {}: {}",
                path.display(),
                backup.display(),
                e
            )
        })?;
        logs.push(IosSdkSupportLog::info(format!(
            "已备份 workspace 副本: {}",
            backup.display()
        )));
    }

    let archs = lipo_architectures(path)?;
    if archs.is_empty() {
        return Err(format!(
            "无法识别 iOS 静态库架构，跳过修复: {}",
            path.display()
        ));
    }
    logs.push(IosSdkSupportLog::info(format!(
        "{} 包含架构: {}",
        library_name,
        archs.join("、")
    )));

    let work_dir = std::env::temp_dir().join(format!(
        "unipack-ios-align-fix-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&work_dir).map_err(|e| {
        format!(
            "创建 iOS 静态库修复临时目录失败 {}: {}",
            work_dir.display(),
            e
        )
    })?;
    let result = (|| {
        let mut fixed_libraries = Vec::new();
        for arch in &archs {
            logs.push(IosSdkSupportLog::info(format!(
                "{} 处理架构: {}",
                library_name, arch
            )));
            let arch_dir = work_dir.join(arch);
            std::fs::create_dir_all(&arch_dir).map_err(|e| {
                format!(
                    "创建 iOS 静态库架构工作目录失败 {}: {}",
                    arch_dir.display(),
                    e
                )
            })?;
            let thin = arch_dir.join("thin.a");
            run_command(
                Command::new("xcrun")
                    .arg("lipo")
                    .arg(path)
                    .arg("-thin")
                    .arg(arch)
                    .arg("-output")
                    .arg(&thin),
                &format!("提取 iOS 静态库架构 {} 失败", arch),
            )?;
            run_command(
                Command::new("xcrun")
                    .arg("ar")
                    .arg("x")
                    .arg("thin.a")
                    .current_dir(&arch_dir),
                &format!("解包 iOS 静态库架构 {} 失败", arch),
            )?;
            let _ = std::fs::remove_file(&thin);
            let _ = std::fs::remove_file(arch_dir.join("__.SYMDEF"));
            let _ = std::fs::remove_file(arch_dir.join("__.SYMDEF SORTED"));

            let object_files = sorted_object_files(&arch_dir)?;
            if object_files.is_empty() {
                return Err(format!(
                    "iOS 静态库架构 {} 未解出 .o 文件: {}",
                    arch,
                    path.display()
                ));
            }
            let fixed = arch_dir.join("fixed.a");
            let libtool_result = run_libtool_static(&fixed, &object_files);
            if libtool_result.is_err() {
                run_ar_static(&fixed, &object_files)?;
            }
            fixed_libraries.push(fixed);
        }

        run_lipo_create(&fixed_libraries, path)?;
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&work_dir);
    result?;

    if ios_static_library_needs_alignment_repair(path)? {
        return Err(format!(
            "iOS 静态库修复后仍存在 not 8-byte aligned 风险: {}",
            path.display()
        ));
    }

    logs.push(IosSdkSupportLog::success(format!(
        "{} 修复完成",
        library_name
    )));
    Ok(())
}

fn make_file_writable(path: &Path) -> Result<(), String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("读取文件权限失败 {}: {}", path.display(), e))?;
    let mut permissions = metadata.permissions();
    if permissions.readonly() {
        permissions.set_readonly(false);
        std::fs::set_permissions(path, permissions)
            .map_err(|e| format!("设置文件可写失败 {}: {}", path.display(), e))?;
    }
    Ok(())
}

fn lipo_architectures(path: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("xcrun")
        .arg("lipo")
        .arg("-info")
        .arg(path)
        .output()
        .map_err(|e| format!("执行 xcrun lipo -info 失败 {}: {}", path.display(), e))?;
    if !output.status.success() {
        return Err(format!(
            "读取 iOS 静态库架构失败 {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_lipo_architectures(&text))
}

fn parse_lipo_architectures(text: &str) -> Vec<String> {
    let line = text.trim();
    let archs = line
        .split(" are: ")
        .nth(1)
        .or_else(|| line.split(" is architecture: ").nth(1))
        .unwrap_or_default();
    archs
        .split_whitespace()
        .map(str::trim)
        .filter(|arch| !arch.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn lipo_architecture_slices(path: &Path) -> Result<Vec<LipoArchitectureSlice>, String> {
    let output = Command::new("xcrun")
        .arg("lipo")
        .arg("-detailed_info")
        .arg(path)
        .output()
        .map_err(|e| {
            format!(
                "执行 xcrun lipo -detailed_info 失败 {}: {}",
                path.display(),
                e
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "读取 iOS 静态库架构详情失败 {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut slices = parse_lipo_architecture_slices(&text);
    if slices.is_empty() {
        slices = parse_lipo_architectures(&text)
            .into_iter()
            .map(|name| LipoArchitectureSlice { name, offset: 0 })
            .collect();
    }
    Ok(slices)
}

fn parse_lipo_architecture_slices(text: &str) -> Vec<LipoArchitectureSlice> {
    let mut slices = Vec::new();
    let mut current_arch: Option<String> = None;
    for line in text.lines().map(str::trim) {
        if let Some(arch) = line.strip_prefix("architecture ") {
            current_arch = Some(arch.trim().to_string());
            continue;
        }
        let Some(offset_text) = line.strip_prefix("offset ") else {
            continue;
        };
        let Some(name) = current_arch.take() else {
            continue;
        };
        if let Ok(offset) = offset_text.trim().parse::<usize>() {
            slices.push(LipoArchitectureSlice { name, offset });
        }
    }
    slices
}

fn archive_has_unaligned_macho_members(path: &Path) -> Result<bool, String> {
    archive_has_unaligned_macho_members_with_base_offset(path, 0)
}

fn archive_has_unaligned_macho_members_with_base_offset(
    path: &Path,
    base_offset: usize,
) -> Result<bool, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("读取 iOS 静态库失败 {}: {}", path.display(), e))?;
    if !bytes.starts_with(b"!<arch>\n") {
        return Ok(false);
    }

    let mut offset = 8usize;
    while offset + 60 <= bytes.len() {
        let header = &bytes[offset..offset + 60];
        if &header[58..60] != b"`\n" {
            return Err(format!("iOS 静态库 ar header 异常: {}", path.display()));
        }
        let size_text = std::str::from_utf8(&header[48..58])
            .map_err(|e| format!("解析 iOS 静态库 member size 失败 {}: {}", path.display(), e))?
            .trim();
        let size = size_text
            .parse::<usize>()
            .map_err(|e| format!("解析 iOS 静态库 member size 失败 {}: {}", path.display(), e))?;

        let data_start = offset + 60;
        if data_start + size > bytes.len() {
            return Err(format!("iOS 静态库 member 越界: {}", path.display()));
        }

        let name = std::str::from_utf8(&header[..16])
            .unwrap_or_default()
            .trim();
        let name_len = name
            .strip_prefix("#1/")
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or_default();
        let object_start = data_start + name_len;
        if object_start + 4 <= data_start + size
            && is_64_bit_macho_magic(&bytes[object_start..object_start + 4])
            && (base_offset + object_start) % 8 != 0
        {
            return Ok(true);
        }

        offset = data_start + size;
        if offset % 2 == 1 {
            offset += 1;
        }
    }
    Ok(false)
}

fn is_64_bit_macho_magic(bytes: &[u8]) -> bool {
    matches!(bytes, [0xfe, 0xed, 0xfa, 0xcf] | [0xcf, 0xfa, 0xed, 0xfe])
}

fn sorted_object_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = std::fs::read_dir(dir)
        .map_err(|e| format!("读取 iOS 静态库工作目录失败 {}: {}", dir.display(), e))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("o"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn run_libtool_static(output: &Path, objects: &[PathBuf]) -> Result<(), String> {
    let mut command = Command::new("xcrun");
    command.arg("libtool").arg("-static").arg("-o").arg(output);
    for object in objects {
        command.arg(object);
    }
    run_command(&mut command, "使用 libtool 重建 iOS 静态库失败")
}

fn run_ar_static(output: &Path, objects: &[PathBuf]) -> Result<(), String> {
    let mut command = Command::new("xcrun");
    command.arg("ar").arg("rcs").arg(output);
    for object in objects {
        command.arg(object);
    }
    run_command(&mut command, "使用 ar 重建 iOS 静态库失败")
}

fn run_lipo_create(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    let mut command = Command::new("xcrun");
    command.arg("lipo").arg("-create");
    for input in inputs {
        command.arg(input);
    }
    command.arg("-output").arg(output);
    run_command(&mut command, "合并 iOS 静态库架构失败")
}

fn run_command(command: &mut Command, context: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|e| format!("{}: {}", context, e))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("{}: {}", context, stderr.trim()))
}

#[cfg(test)]
mod alignment_tests {
    use std::path::PathBuf;

    use super::{
        archive_has_unaligned_macho_members, archive_has_unaligned_macho_members_with_base_offset,
        ios_project_dependency_references_from_content, parse_lipo_architecture_slices,
        parse_lipo_architectures, sdk_relative_path, LipoArchitectureSlice,
    };

    #[test]
    fn parses_lipo_architectures_for_fat_and_thin_libraries() {
        assert_eq!(
            parse_lipo_architectures(
                "Architectures in the fat file: lib.a are: armv7 x86_64 arm64"
            ),
            vec!["armv7", "x86_64", "arm64"]
        );
        assert_eq!(
            parse_lipo_architectures("Non-fat file: lib.a is architecture: arm64"),
            vec!["arm64"]
        );
    }

    #[test]
    fn parses_lipo_detailed_info_offsets() {
        let slices = parse_lipo_architecture_slices(
            r#"Fat header in: lib.a
fat_magic 0xcafebabe
nfat_arch 2
architecture x86_64
    cputype CPU_TYPE_X86_64
    offset 787976
    size 293328
    align 2^3 (8)
architecture arm64
    cputype CPU_TYPE_ARM64
    offset 1081304
    size 320032
    align 2^3 (8)
"#,
        );

        assert_eq!(
            slices,
            vec![
                LipoArchitectureSlice {
                    name: "x86_64".to_string(),
                    offset: 787976
                },
                LipoArchitectureSlice {
                    name: "arm64".to_string(),
                    offset: 1081304
                }
            ]
        );
    }

    #[test]
    fn detects_unaligned_macho_member_in_archive() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-unaligned-ar-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let archive = root.join("libBroken.a");
        let mut bytes = b"!<arch>\n".to_vec();
        bytes.extend_from_slice(ar_header("#1/3", 7).as_bytes());
        bytes.extend_from_slice(b"foo");
        bytes.extend_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
        if bytes.len() % 2 == 1 {
            bytes.push(b'\n');
        }
        std::fs::write(&archive, bytes).unwrap();

        assert!(archive_has_unaligned_macho_members(&archive).unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn detects_fat_slice_base_offset_alignment_for_64_bit_members() {
        let root = std::env::temp_dir().join(format!(
            "unipack-ios-fat-offset-ar-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let archive = root.join("libFatOffset.a");
        let mut bytes = b"!<arch>\n".to_vec();
        bytes.extend_from_slice(ar_header("#1/20", 24).as_bytes());
        bytes.extend_from_slice(b"AlignedObjectFile.o\0");
        bytes.extend_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
        std::fs::write(&archive, bytes).unwrap();

        assert!(!archive_has_unaligned_macho_members_with_base_offset(&archive, 0).unwrap());
        assert!(archive_has_unaligned_macho_members_with_base_offset(&archive, 4).unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

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

    fn ar_header(name: &str, size: usize) -> String {
        format!(
            "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
            name, 0, 0, 0, 0o100644, size
        )
    }
}

pub(super) fn find_xcodeproj(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("xcodeproj"))
}

pub(super) fn find_scheme_name(project_file: &Path) -> Option<String> {
    let content = std::fs::read_to_string(project_file.join("project.pbxproj")).ok()?;
    let pattern =
        regex::Regex::new(r#"(?s)isa = PBXNativeTarget;.*?\n\s*name = "?([^";]+)"?;"#).ok()?;
    pattern
        .captures(&content)
        .and_then(|captures| captures.get(1))
        .map(|name| name.as_str().trim().to_string())
        .filter(|name| !name.is_empty())
}

pub(crate) fn find_info_plist(project_root: &Path, project_file: &Path) -> Option<PathBuf> {
    let pbxproj = project_file.join("project.pbxproj");
    if let Ok(content) = std::fs::read_to_string(pbxproj) {
        let re = regex::Regex::new(r#"INFOPLIST_FILE = "?([^";]+)"?;"#).ok()?;
        for cap in re.captures_iter(&content) {
            let rel = cap
                .get(1)?
                .as_str()
                .replace("$(SRCROOT)/", "")
                .replace("${SRCROOT}/", "");
            let candidate = project_root.join(rel.trim_matches('"'));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    find_file_named_skipping_bundles(project_root, "Info.plist")
}

pub(super) fn find_file_named(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_named(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

pub(super) fn find_file_named_skipping_bundles(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_xcode_package_dir(&path) {
                continue;
            }
            if let Some(found) = find_file_named_skipping_bundles(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

pub(super) fn collect_files_named_skipping_bundles(
    dir: &Path,
    name: &str,
    output: &mut Vec<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if !is_xcode_package_dir(&path) {
                collect_files_named_skipping_bundles(&path, name, output);
            }
        } else if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            output.push(path);
        }
    }
}

pub(super) fn find_file_with_ext_skipping_bundles(dir: &Path, ext: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_xcode_package_dir(&path) {
                continue;
            }
            if let Some(found) = find_file_with_ext_skipping_bundles(&path, ext) {
                return Some(found);
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some(ext) {
            return Some(path);
        }
    }
    None
}

pub(super) fn find_file_with_ext(dir: &Path, ext: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_with_ext(&path, ext) {
                return Some(found);
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some(ext) {
            return Some(path);
        }
    }
    None
}

fn is_xcode_package_dir(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("framework" | "xcframework" | "bundle" | "xcodeproj" | "xcworkspace")
    )
}

pub(super) fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    PathBuf::from(path)
}

pub(super) fn safe_file_name(value: &str) -> String {
    let cleaned = value.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    if cleaned.trim().is_empty() {
        "ios-build".to_string()
    } else {
        cleaned
    }
}
