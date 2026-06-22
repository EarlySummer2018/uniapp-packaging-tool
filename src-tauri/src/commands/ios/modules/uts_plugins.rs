use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::commands::ios::build::fs_utils::find_info_plist;
use crate::commands::ios::build::pbxproj::{
    append_pbx_build_setting_paths, raise_pbx_ios_deployment_target,
    register_pbx_embedded_file_specs, register_pbx_linked_file_specs,
    register_pbx_resource_file_specs, register_pbx_source_file_specs,
    remove_pbx_linked_or_embedded_files, IosPbxFileSpec, IosPbxSourceFileSpec,
};
use crate::commands::shared::resource_scan::{UtsCustomPlugin, UtsPluginScanResult};

const IOS_UTS_DUPLICATE_LINKED_FILES: &[&str] = &[
    "liblibPDRCore.a",
    "liblibWeex.a",
    "libcoreSupport.a",
    "storage.framework",
    "libSDWebImage.a",
    "KSCrash.framework",
];

#[derive(Debug, Clone)]
pub(crate) struct IosUtsBaseIntegration {
    pub(crate) linked_count: usize,
    pub(crate) embedded_count: usize,
    pub(crate) ext_api_count: usize,
    pub(crate) removed_duplicate_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct IosUtsPluginIntegration {
    pub(crate) copied_plugin_count: usize,
    pub(crate) source_count: usize,
    pub(crate) local_framework_count: usize,
    pub(crate) static_library_count: usize,
    pub(crate) system_framework_count: usize,
    pub(crate) resource_count: usize,
    pub(crate) plist_count: usize,
    pub(crate) pod_dependency_count: usize,
    pub(crate) deployment_target_update_count: usize,
    pub(crate) linked_count: usize,
    pub(crate) embedded_count: usize,
    pub(crate) framework_search_path_count: usize,
    pub(crate) library_search_path_count: usize,
    pub(crate) header_search_path_count: usize,
}

pub(crate) fn apply_ios_uts_base_module(
    project_root: &Path,
    project_file: &Path,
    include_ext_api: bool,
) -> Result<IosUtsBaseIntegration, String> {
    let mut specs = vec![
        sdk_framework_spec(project_root, "DCUniBase")?,
        sdk_framework_spec(project_root, "DCloudUTSFoundation")?,
    ];
    if include_ext_api {
        specs.push(sdk_framework_spec(project_root, "DCloudUTSExtAPI")?);
    }

    let removed_duplicate_count =
        remove_pbx_linked_or_embedded_files(project_file, IOS_UTS_DUPLICATE_LINKED_FILES)?;
    let linked_count = register_pbx_linked_file_specs(project_file, &specs)?;
    let embedded_count = register_pbx_embedded_file_specs(project_file, &specs)?;

    Ok(IosUtsBaseIntegration {
        linked_count,
        embedded_count,
        ext_api_count: usize::from(include_ext_api),
        removed_duplicate_count,
    })
}

pub(crate) fn apply_ios_uts_plugins(
    project_root: &Path,
    project_file: &Path,
    scan: &UtsPluginScanResult,
) -> Result<Option<IosUtsPluginIntegration>, String> {
    if !scan.has_ios_uts_plugins {
        return Ok(None);
    }

    let ios_custom_plugins = scan
        .custom_plugins
        .iter()
        .filter(|plugin| plugin.ios_dir.is_some())
        .collect::<Vec<_>>();
    if ios_custom_plugins.is_empty() {
        return Ok(None);
    }

    let copied_plugins = copy_ios_uts_plugin_app_ios_dirs(project_root, &ios_custom_plugins)?;
    if copied_plugins.is_empty() {
        return Ok(None);
    }

    let artifacts = collect_ios_uts_plugin_artifacts(project_root, &copied_plugins)?;
    let framework_search_path_count = append_pbx_build_setting_paths(
        project_file,
        "FRAMEWORK_SEARCH_PATHS",
        &artifacts.framework_search_paths,
    )?;
    let library_search_path_count = append_pbx_build_setting_paths(
        project_file,
        "LIBRARY_SEARCH_PATHS",
        &artifacts.library_search_paths,
    )?;
    let header_search_path_count = append_pbx_build_setting_paths(
        project_file,
        "HEADER_SEARCH_PATHS",
        &artifacts.header_search_paths,
    )?;

    let source_count = register_pbx_source_file_specs(project_file, &artifacts.source_specs)?;
    let mut linked_count =
        register_pbx_linked_file_specs(project_file, &artifacts.local_framework_specs)?;
    linked_count += register_pbx_linked_file_specs(project_file, &artifacts.static_library_specs)?;
    linked_count +=
        register_pbx_linked_file_specs(project_file, &artifacts.system_framework_specs)?;
    let embedded_count =
        register_pbx_embedded_file_specs(project_file, &artifacts.local_framework_specs)?;
    let resource_count = register_pbx_resource_file_specs(project_file, &artifacts.resource_specs)?;
    let plist_count = merge_ios_uts_plists(project_root, project_file, &copied_plugins)?;
    let deployment_target_update_count =
        apply_ios_uts_deployment_targets(project_file, &copied_plugins)?;
    let pod_dependency_count = copied_plugins
        .iter()
        .map(|plugin| plugin.ios_dependencies_pod_count)
        .sum();

    Ok(Some(IosUtsPluginIntegration {
        copied_plugin_count: copied_plugins.len(),
        source_count,
        local_framework_count: artifacts.local_framework_specs.len(),
        static_library_count: artifacts.static_library_specs.len(),
        system_framework_count: artifacts.system_framework_specs.len(),
        resource_count,
        plist_count,
        pod_dependency_count,
        deployment_target_update_count,
        linked_count,
        embedded_count,
        framework_search_path_count,
        library_search_path_count,
        header_search_path_count,
    }))
}

fn sdk_framework_spec(project_root: &Path, base_name: &str) -> Result<IosPbxFileSpec, String> {
    let libs_dir = project_root
        .parent()
        .map(|workspace| workspace.join("SDK/Libs"))
        .ok_or_else(|| format!("iOS 工程路径异常: {}", project_root.display()))?;
    let framework_name = format!("{}.framework", base_name);
    let framework = libs_dir.join(&framework_name);
    if framework.exists() {
        return Ok(IosPbxFileSpec::project_framework(
            framework_name.clone(),
            format!("../SDK/Libs/{}", framework_name),
        ));
    }

    let xcframework_name = format!("{}.xcframework", base_name);
    let xcframework = libs_dir.join(&xcframework_name);
    if xcframework.exists() {
        return Ok(IosPbxFileSpec::project_xcframework(
            xcframework_name.clone(),
            format!("../SDK/Libs/{}", xcframework_name),
        ));
    }

    Err(format!(
        "iOS UTS 基础模块缺少 SDK 依赖文件: {} 或 {}",
        framework.display(),
        xcframework.display()
    ))
}

fn copy_ios_uts_plugin_app_ios_dirs(
    project_root: &Path,
    plugins: &[&UtsCustomPlugin],
) -> Result<Vec<CopiedIosUtsPlugin>, String> {
    let root = project_root.join("UTSPlugins");
    crate::utils::fs::ensure_directory(&root).map_err(|e| e.to_string())?;
    let mut copied_plugins = Vec::new();

    for plugin in plugins {
        let Some(ios_dir) = plugin.ios_dir.as_deref() else {
            continue;
        };
        let ios_dir = Path::new(ios_dir);
        let plugin_dir_name = ios_uts_plugin_dir_name(plugin, ios_dir);
        let target = root.join(&plugin_dir_name).join("app-ios");
        replace_path(ios_dir, &target).map_err(|e| {
            format!(
                "复制 iOS UTS 插件 {} app-ios 失败 {} -> {}: {}",
                plugin.id,
                ios_dir.display(),
                target.display(),
                e
            )
        })?;
        copied_plugins.push(CopiedIosUtsPlugin {
            id: plugin.id.clone(),
            dir_name: plugin_dir_name,
            app_ios_dir: target,
            ios_system_frameworks: plugin.ios_system_frameworks.clone(),
            ios_plists: plugin.ios_plists.clone(),
            ios_dependencies_pod_count: plugin.ios_dependencies_pods.len(),
        });
    }
    Ok(copied_plugins)
}

#[derive(Debug, Clone)]
struct CopiedIosUtsPlugin {
    id: String,
    dir_name: String,
    app_ios_dir: PathBuf,
    ios_system_frameworks: Vec<String>,
    ios_plists: BTreeMap<String, String>,
    ios_dependencies_pod_count: usize,
}

#[derive(Debug, Default)]
struct IosUtsPluginArtifacts {
    source_specs: Vec<IosPbxSourceFileSpec>,
    local_framework_specs: Vec<IosPbxFileSpec>,
    static_library_specs: Vec<IosPbxFileSpec>,
    system_framework_specs: Vec<IosPbxFileSpec>,
    resource_specs: Vec<IosPbxFileSpec>,
    framework_search_paths: Vec<String>,
    library_search_paths: Vec<String>,
    header_search_paths: Vec<String>,
}

fn collect_ios_uts_plugin_artifacts(
    project_root: &Path,
    plugins: &[CopiedIosUtsPlugin],
) -> Result<IosUtsPluginArtifacts, String> {
    let mut artifacts = IosUtsPluginArtifacts::default();
    let mut seen_source_paths = BTreeSet::new();
    let mut seen_local_framework_paths = BTreeSet::new();
    let mut seen_static_library_paths = BTreeSet::new();
    let mut seen_system_frameworks = BTreeSet::new();
    let mut seen_resource_paths = BTreeSet::new();
    let mut framework_search_paths = BTreeSet::new();
    let mut library_search_paths = BTreeSet::new();
    let mut header_search_paths = BTreeSet::new();

    for plugin in plugins {
        let source_dir = plugin.app_ios_dir.join("src");
        if source_dir.is_dir() {
            header_search_paths.insert(format!(
                "$(PROJECT_DIR)/{}/**",
                project_relative_pbx_path(project_root, &source_dir)?
            ));
        }
        for source in collect_files_by_extensions(
            &source_dir,
            &["swift", "m", "mm", "c", "cc", "cpp", "cxx", "metal"],
        )? {
            let pbx_path = project_relative_pbx_path(project_root, &source)?;
            if !seen_source_paths.insert(pbx_path.clone()) {
                continue;
            }
            let name = ios_uts_artifact_display_name(plugin, &source)?;
            if let Some(spec) = IosPbxSourceFileSpec::project_source(name, pbx_path) {
                artifacts.source_specs.push(spec);
            }
        }

        for framework in collect_xcode_package_dirs(
            &plugin.app_ios_dir.join("Frameworks"),
            &["framework", "xcframework"],
        )? {
            let Some(name) = framework.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let pbx_path = project_relative_pbx_path(project_root, &framework)?;
            if !seen_local_framework_paths.insert(pbx_path.clone()) {
                continue;
            }
            add_parent_search_path(&mut framework_search_paths, &pbx_path);
            let spec = if name.ends_with(".xcframework") {
                IosPbxFileSpec::project_xcframework(name.to_string(), pbx_path)
            } else {
                IosPbxFileSpec::project_framework(name.to_string(), pbx_path)
            };
            artifacts.local_framework_specs.push(spec);
        }

        let libs_dir = plugin.app_ios_dir.join("Libs");
        for library in collect_files_by_extensions(&libs_dir, &["a"])? {
            let Some(name) = library.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let pbx_path = project_relative_pbx_path(project_root, &library)?;
            if !seen_static_library_paths.insert(pbx_path.clone()) {
                continue;
            }
            add_parent_search_path(&mut library_search_paths, &pbx_path);
            artifacts
                .static_library_specs
                .push(IosPbxFileSpec::project_static_library(
                    name.to_string(),
                    pbx_path,
                ));
        }
        if libs_dir.is_dir() {
            header_search_paths.insert(format!(
                "$(PROJECT_DIR)/{}/**",
                project_relative_pbx_path(project_root, &libs_dir)?
            ));
        }

        for framework in &plugin.ios_system_frameworks {
            let Some(name) = normalize_system_framework_name(framework) else {
                continue;
            };
            if seen_system_frameworks.insert(name.clone()) {
                artifacts
                    .system_framework_specs
                    .push(IosPbxFileSpec::system_framework(name));
            }
        }

        for resource in collect_ios_uts_resource_sources(&plugin.app_ios_dir)? {
            let pbx_path = project_relative_pbx_path(project_root, &resource)?;
            if !seen_resource_paths.insert(pbx_path.clone()) {
                continue;
            }
            let name = ios_uts_artifact_display_name(plugin, &resource)?;
            artifacts
                .resource_specs
                .push(IosPbxFileSpec::project_resource_source_root(name, pbx_path));
        }
    }

    artifacts.framework_search_paths = framework_search_paths.into_iter().collect();
    artifacts.library_search_paths = library_search_paths.into_iter().collect();
    artifacts.header_search_paths = header_search_paths.into_iter().collect();
    Ok(artifacts)
}

fn merge_ios_uts_plists(
    project_root: &Path,
    project_file: &Path,
    plugins: &[CopiedIosUtsPlugin],
) -> Result<usize, String> {
    let has_plist_values = plugins.iter().any(|plugin| {
        !plugin.ios_plists.is_empty() || plugin.app_ios_dir.join("Info.plist").is_file()
    });
    if !has_plist_values {
        return Ok(0);
    }

    let plist_path = find_info_plist(project_root, project_file)
        .ok_or_else(|| "未找到主工程 Info.plist，无法合并 UTS 插件 plist".to_string())?;
    let mut value =
        plist::Value::from_file(&plist_path).map_err(|e| format!("解析 Info.plist 失败: {}", e))?;
    let dict = value
        .as_dictionary_mut()
        .ok_or_else(|| "Info.plist 不是 dictionary".to_string())?;
    let mut merged_count = 0usize;

    for plugin in plugins {
        for (key, value) in &plugin.ios_plists {
            if dict.get(key).is_none() {
                dict.insert(key.clone(), plist::Value::String(value.clone()));
                merged_count += 1;
            }
        }

        let plugin_plist_path = plugin.app_ios_dir.join("Info.plist");
        if !plugin_plist_path.is_file() {
            continue;
        }
        let plugin_plist = plist::Value::from_file(&plugin_plist_path)
            .map_err(|e| format!("解析 iOS UTS 插件 {} Info.plist 失败: {}", plugin.id, e))?;
        let Some(plugin_dict) = plugin_plist.as_dictionary() else {
            continue;
        };
        for (key, plugin_value) in plugin_dict {
            if dict.get(key).is_none() {
                dict.insert(key.clone(), plugin_value.clone());
                merged_count += 1;
            }
        }
    }

    if merged_count > 0 {
        value
            .to_file_xml(&plist_path)
            .map_err(|e| format!("写入 Info.plist 失败: {}", e))?;
    }
    Ok(merged_count)
}

fn apply_ios_uts_deployment_targets(
    project_file: &Path,
    plugins: &[CopiedIosUtsPlugin],
) -> Result<usize, String> {
    let mut changed_count = 0usize;
    for plugin in plugins {
        let Some(target) = ios_uts_deployment_target(&plugin.app_ios_dir) else {
            continue;
        };
        if raise_pbx_ios_deployment_target(project_file, &target)? {
            changed_count += 1;
        }
    }
    Ok(changed_count)
}

fn ios_uts_deployment_target(app_ios_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(app_ios_dir.join("config.json")).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    let target = value.get("deploymentTarget")?;
    if let Some(target) = target.as_str() {
        let target = target.trim();
        return (!target.is_empty()).then(|| target.to_string());
    }
    target.as_f64().map(|target| target.to_string())
}

fn collect_ios_uts_resource_sources(app_ios_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::new();
    let resources_dir = app_ios_dir.join("Resources");
    if resources_dir.is_dir() {
        for entry in std::fs::read_dir(&resources_dir).map_err(|e| e.to_string())? {
            sources.push(entry.map_err(|e| e.to_string())?.path());
        }
    }
    sources.extend(collect_xcode_package_dirs(
        app_ios_dir,
        &["bundle", "xcassets"],
    )?);
    let privacy = app_ios_dir.join("PrivacyInfo.xcprivacy");
    if privacy.is_file() {
        sources.push(privacy);
    }
    Ok(sources)
}

fn normalize_system_framework_name(name: &str) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some(if name.ends_with(".framework") {
        name.to_string()
    } else {
        format!("{}.framework", name)
    })
}

fn collect_xcode_package_dirs(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    collect_xcode_package_dirs_inner(root, extensions, &mut dirs)?;
    Ok(dirs)
}

fn collect_xcode_package_dirs_inner(
    current: &Path,
    extensions: &[&str],
    dirs: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(current).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !path.is_dir() {
            continue;
        }
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase);
        if extension
            .as_deref()
            .is_some_and(|extension| extensions.iter().any(|target| extension == *target))
        {
            dirs.push(path);
            continue;
        }
        collect_xcode_package_dirs_inner(&path, extensions, dirs)?;
    }
    Ok(())
}

fn collect_files_by_extensions(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_files_by_extensions_inner(root, extensions, &mut files)?;
    Ok(files)
}

fn collect_files_by_extensions_inner(
    current: &Path,
    extensions: &[&str],
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(current).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_files_by_extensions_inner(&path, extensions, files)?;
            continue;
        }
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase);
        if extension
            .as_deref()
            .is_some_and(|extension| extensions.iter().any(|target| extension == *target))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn add_parent_search_path(search_paths: &mut BTreeSet<String>, pbx_path: &str) {
    if let Some((parent, _)) = pbx_path.rsplit_once('/') {
        if !parent.is_empty() {
            search_paths.insert(format!("$(PROJECT_DIR)/{}", parent));
        }
    }
}

fn project_relative_pbx_path(project_root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(project_root)
        .map_err(|_| format!("iOS UTS 插件文件不在工程目录内: {}", path.display()))?;
    path_to_pbx_string(relative)
}

fn path_to_pbx_string(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in path.components() {
        let std::path::Component::Normal(value) = component else {
            continue;
        };
        let Some(value) = value.to_str() else {
            return Err(format!("路径包含非 UTF-8 内容: {}", path.display()));
        };
        parts.push(value.to_string());
    }
    Ok(parts.join("/"))
}

fn ios_uts_artifact_display_name(
    plugin: &CopiedIosUtsPlugin,
    path: &Path,
) -> Result<String, String> {
    let relative = path
        .strip_prefix(&plugin.app_ios_dir)
        .map_err(|_| format!("iOS UTS 插件文件路径异常: {}", path.display()))?;
    let relative = path_to_pbx_string(relative)?;
    Ok(safe_path_component(&format!(
        "{}-{}",
        plugin.dir_name,
        relative.replace('/', "-")
    )))
}

fn replace_path(source: &Path, target: &Path) -> Result<(), String> {
    if target.is_dir() {
        std::fs::remove_dir_all(target).map_err(|e| e.to_string())?;
    } else if target.exists() {
        std::fs::remove_file(target).map_err(|e| e.to_string())?;
    }
    if source.is_dir() {
        crate::utils::fs::copy_recursive(source, target).map_err(|e| e.to_string())
    } else {
        crate::utils::fs::copy_file(source, target).map_err(|e| e.to_string())
    }
}

fn ios_uts_plugin_dir_name(plugin: &UtsCustomPlugin, ios_dir: &Path) -> String {
    let source_plugin_name = ios_dir
        .parent()
        .and_then(|parent| {
            if parent
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("utssdk"))
            {
                parent.parent()
            } else {
                Some(parent)
            }
        })
        .and_then(|plugin_root| plugin_root.file_name())
        .and_then(|name| name.to_str())
        .map(String::from);
    safe_path_component(source_plugin_name.as_deref().unwrap_or(&plugin.id))
}

fn safe_path_component(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if cleaned.trim_matches('_').is_empty() {
        "uts-plugin".to_string()
    } else {
        cleaned
    }
}
