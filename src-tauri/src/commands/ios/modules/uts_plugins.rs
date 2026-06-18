use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::commands::ios::build::fs_utils::find_info_plist;
use crate::commands::ios::build::pbxproj::{
    append_pbx_build_setting_paths, register_pbx_embedded_file_specs,
    register_pbx_linked_file_specs, register_pbx_resource_file_specs,
    remove_pbx_linked_or_embedded_files, IosPbxFileSpec,
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

#[derive(Debug, Clone, Default)]
pub(crate) struct IosUtsPluginIntegration {
    pub(crate) linked_count: usize,
    pub(crate) embedded_count: usize,
    pub(crate) system_framework_count: usize,
    pub(crate) custom_framework_count: usize,
    pub(crate) resource_count: usize,
    pub(crate) plist_count: usize,
    pub(crate) pod_dependency_count: usize,
    pub(crate) removed_duplicate_count: usize,
}

pub(crate) fn apply_ios_uts_plugins(
    project_root: &Path,
    project_file: &Path,
    scan: &UtsPluginScanResult,
) -> Result<Option<IosUtsPluginIntegration>, String> {
    if !scan.has_ios_uts_plugins {
        return Ok(None);
    }

    let mut integration = IosUtsPluginIntegration::default();
    integration.removed_duplicate_count =
        remove_pbx_linked_or_embedded_files(project_file, IOS_UTS_DUPLICATE_LINKED_FILES)?;

    let runtime_specs = vec![
        sdk_framework_spec(project_root, "DCUniBase")?,
        sdk_framework_spec(project_root, "DCloudUTSFoundation")?,
    ];
    integration.linked_count += register_pbx_linked_file_specs(project_file, &runtime_specs)?;
    integration.embedded_count += register_pbx_embedded_file_specs(project_file, &runtime_specs)?;

    let ios_custom_plugins = scan
        .custom_plugins
        .iter()
        .filter(|plugin| plugin.ios_dir.is_some())
        .collect::<Vec<_>>();
    if ios_custom_plugins.is_empty() {
        return Ok(Some(integration));
    }

    let custom_framework_specs = copy_ios_uts_frameworks(project_root, &ios_custom_plugins)?;
    let framework_search_paths = ios_uts_framework_search_paths(&custom_framework_specs);
    append_pbx_build_setting_paths(
        project_file,
        "FRAMEWORK_SEARCH_PATHS",
        &framework_search_paths,
    )?;
    integration.custom_framework_count = custom_framework_specs.len();
    integration.linked_count +=
        register_pbx_linked_file_specs(project_file, &custom_framework_specs)?;
    integration.embedded_count +=
        register_pbx_embedded_file_specs(project_file, &custom_framework_specs)?;

    let system_framework_specs = ios_custom_plugins
        .iter()
        .flat_map(|plugin| plugin.ios_system_frameworks.iter())
        .filter_map(|name| normalize_system_framework_name(name))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(IosPbxFileSpec::system_framework)
        .collect::<Vec<_>>();
    integration.system_framework_count = system_framework_specs.len();
    integration.linked_count +=
        register_pbx_linked_file_specs(project_file, &system_framework_specs)?;

    let resource_specs = copy_ios_uts_resources(project_root, &ios_custom_plugins)?;
    integration.resource_count = resource_specs.len();
    register_pbx_resource_file_specs(project_file, &resource_specs)?;

    integration.plist_count =
        merge_ios_uts_plists(project_root, project_file, &ios_custom_plugins)?;
    integration.pod_dependency_count = ios_custom_plugins
        .iter()
        .map(|plugin| plugin.ios_dependencies_pods.len())
        .sum();

    Ok(Some(integration))
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
        "iOS UTS 插件缺少 SDK 依赖文件: {} 或 {}",
        framework.display(),
        xcframework.display()
    ))
}

fn copy_ios_uts_frameworks(
    project_root: &Path,
    plugins: &[&UtsCustomPlugin],
) -> Result<Vec<IosPbxFileSpec>, String> {
    let mut specs = Vec::new();
    let root = project_root.join("UTSPlugins");
    crate::utils::fs::ensure_directory(&root).map_err(|e| e.to_string())?;

    for plugin in plugins {
        let Some(ios_dir) = plugin.ios_dir.as_deref() else {
            continue;
        };
        let ios_dir = Path::new(ios_dir);
        let plugin_dir_name = safe_path_component(&plugin.id);
        let target_dir = root.join(&plugin_dir_name);
        crate::utils::fs::ensure_directory(&target_dir).map_err(|e| e.to_string())?;

        for framework in collect_xcode_package_dirs(ios_dir, &["framework", "xcframework"])? {
            let Some(name) = framework.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let target = target_dir.join(name);
            replace_path(&framework, &target).map_err(|e| {
                format!(
                    "复制 iOS UTS 插件 {} framework 失败 {} -> {}: {}",
                    plugin.id,
                    framework.display(),
                    target.display(),
                    e
                )
            })?;
            let path = format!("UTSPlugins/{}/{}", plugin_dir_name, name);
            let spec = if name.ends_with(".xcframework") {
                IosPbxFileSpec::project_xcframework(name.to_string(), path)
            } else {
                IosPbxFileSpec::project_framework(name.to_string(), path)
            };
            if !specs
                .iter()
                .any(|existing: &IosPbxFileSpec| existing.name == spec.name)
            {
                specs.push(spec);
            }
        }
    }

    Ok(specs)
}

fn ios_uts_framework_search_paths(framework_specs: &[IosPbxFileSpec]) -> Vec<String> {
    framework_specs
        .iter()
        .filter_map(|spec| spec.path().rsplit_once('/').map(|(parent, _)| parent))
        .filter(|parent| !parent.is_empty())
        .map(|parent| format!("$(PROJECT_DIR)/{}", parent))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn copy_ios_uts_resources(
    project_root: &Path,
    plugins: &[&UtsCustomPlugin],
) -> Result<Vec<IosPbxFileSpec>, String> {
    let mut specs = Vec::new();
    let resources_root = project_root.join("HBuilder-Hello/UTSResources");
    crate::utils::fs::ensure_directory(&resources_root).map_err(|e| e.to_string())?;

    for plugin in plugins {
        let Some(ios_dir) = plugin.ios_dir.as_deref() else {
            continue;
        };
        let ios_dir = Path::new(ios_dir);
        let plugin_dir_name = safe_path_component(&plugin.id);
        let target_dir = resources_root.join(&plugin_dir_name);
        crate::utils::fs::ensure_directory(&target_dir).map_err(|e| e.to_string())?;

        let mut resource_sources = Vec::new();
        let resources_dir = ios_dir.join("Resources");
        if resources_dir.is_dir() {
            for entry in std::fs::read_dir(&resources_dir).map_err(|e| e.to_string())? {
                resource_sources.push(entry.map_err(|e| e.to_string())?.path());
            }
        }
        resource_sources.extend(collect_xcode_package_dirs(ios_dir, &["bundle"])?);
        let privacy = ios_dir.join("PrivacyInfo.xcprivacy");
        if privacy.is_file() {
            resource_sources.push(privacy);
        }

        for source in resource_sources {
            let Some(name) = source.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let target = target_dir.join(name);
            replace_path(&source, &target).map_err(|e| {
                format!(
                    "复制 iOS UTS 插件 {} 资源失败 {} -> {}: {}",
                    plugin.id,
                    source.display(),
                    target.display(),
                    e
                )
            })?;
            let spec_name = format!("{}-{}", plugin_dir_name, name);
            let path = format!("HBuilder-Hello/UTSResources/{}/{}", plugin_dir_name, name);
            if !specs
                .iter()
                .any(|existing: &IosPbxFileSpec| existing.name == spec_name)
            {
                specs.push(IosPbxFileSpec::project_resource(spec_name, path));
            }
        }
    }

    Ok(specs)
}

fn merge_ios_uts_plists(
    project_root: &Path,
    project_file: &Path,
    plugins: &[&UtsCustomPlugin],
) -> Result<usize, String> {
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
    }

    if merged_count > 0 {
        value
            .to_file_xml(&plist_path)
            .map_err(|e| format!("写入 Info.plist 失败: {}", e))?;
    }
    Ok(merged_count)
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
