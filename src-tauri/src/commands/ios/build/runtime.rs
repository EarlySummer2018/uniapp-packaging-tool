use std::path::{Path, PathBuf};

use super::fs_utils::{
    collect_files_named_skipping_bundles, find_file_named_skipping_bundles,
    find_file_with_ext_skipping_bundles,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IosRuntimeLayout {
    pub(super) control_xml: PathBuf,
    pub(super) apps_dir: PathBuf,
}

pub(super) fn import_app_resource(
    apps_dir: &Path,
    resource_dir: &Path,
    app_id: &str,
) -> Result<(), String> {
    if apps_dir.exists() {
        std::fs::remove_dir_all(apps_dir)
            .map_err(|e| format!("清理旧 Pandora/apps 失败 {}: {}", apps_dir.display(), e))?;
    }
    crate::utils::fs::ensure_directory(apps_dir).map_err(|e| e.to_string())?;
    crate::utils::fs::copy_recursive(resource_dir, &apps_dir.join(app_id))
        .map_err(|e| format!("复制 UniApp iOS 资源失败: {}", e))
}

pub(super) fn patch_control_xml(control_xml: &Path, app_id: &str) -> Result<(), String> {
    if !control_xml.exists() {
        return Err(format!("未找到 control.xml: {}", control_xml.display()));
    }
    let content = std::fs::read_to_string(control_xml)
        .map_err(|e| format!("读取 control.xml 失败: {}", e))?;
    let updated = crate::utils::xml::set_xml_attribute(&content, "/apps/app", "appid", app_id)
        .or_else(|_| crate::utils::xml::set_xml_attribute(&content, "/hbuilder", "appid", app_id))
        .map_err(|e| format!("写入 control.xml appid 失败: {}", e))?;
    std::fs::write(control_xml, updated).map_err(|e| format!("写入 control.xml 失败: {}", e))
}

pub(super) fn resolve_ios_runtime_layout(project_root: &Path) -> Result<IosRuntimeLayout, String> {
    let mut controls: Vec<PathBuf> = Vec::new();
    collect_files_named_skipping_bundles(project_root, "control.xml", &mut controls);
    controls.sort();

    for control_xml in &controls {
        let Some(parent) = control_xml.parent() else {
            continue;
        };
        let pandora = parent.join("Pandora");
        if pandora.is_dir() {
            return Ok(IosRuntimeLayout {
                control_xml: control_xml.clone(),
                apps_dir: pandora.join("apps"),
            });
        }
        if parent.file_name().and_then(|name| name.to_str()) == Some("Pandora") {
            return Ok(IosRuntimeLayout {
                control_xml: control_xml.clone(),
                apps_dir: parent.join("apps"),
            });
        }
    }

    let checked = if controls.is_empty() {
        project_root.display().to_string()
    } else {
        controls
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    Err(format!(
        "未找到有效的 iOS control.xml 与 Pandora/apps 布局。已检查: {}",
        checked
    ))
}

pub(super) fn verify_privacy_manifest(workspace: &Path, project_file: &Path) -> Result<(), String> {
    let privacy = find_privacy_manifest(workspace)?;
    let name = privacy
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("隐私清单文件名异常: {}", privacy.display()))?;
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    if !content.contains(name) {
        return Err(format!(
            "隐私清单 {} 未纳入 Xcode 工程，请检查 iOS SDK 自带 HBuilder-Hello",
            name
        ));
    }
    Ok(())
}

pub(super) fn verify_pod_privacy_manifest(
    workspace: &Path,
    project_root: &Path,
) -> Result<(), String> {
    find_privacy_manifest(workspace)?;
    let resources_script = find_file_named_skipping_bundles(
        project_root,
        "Pods-HBuilder-resources.sh",
    )
    .ok_or_else(|| {
        "Pod 模式未找到 CocoaPods 资源脚本 Pods-HBuilder-resources.sh，请检查 pod install 输出"
            .to_string()
    })?;
    let content = std::fs::read_to_string(&resources_script).map_err(|e| {
        format!(
            "读取 CocoaPods 资源脚本失败 {}: {}",
            resources_script.display(),
            e
        )
    })?;
    if !content.contains("SDK/PrivacyInfo.xcprivacy") {
        return Err(format!(
            "Pod 模式未将 PrivacyInfo.xcprivacy 纳入 CocoaPods 资源脚本，请检查 {}",
            resources_script.display()
        ));
    }
    Ok(())
}

fn find_privacy_manifest(workspace: &Path) -> Result<PathBuf, String> {
    let sdk_privacy = workspace.join("SDK/PrivacyInfo.xcprivacy");
    sdk_privacy
        .is_file()
        .then_some(sdk_privacy)
        .or_else(|| find_file_named_skipping_bundles(workspace, "PrivacyInfo.xcprivacy"))
        .or_else(|| find_file_with_ext_skipping_bundles(workspace, "xcprivacy"))
        .ok_or_else(|| {
            "iOS SDK 工程缺少 .xcprivacy 隐私清单，请确认使用 HBuilderX 5.0+ 对应的 iOS 离线 SDK"
                .to_string()
        })
}
