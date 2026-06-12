use std::path::{Path, PathBuf};

use super::fs_utils::{find_file_named_skipping_bundles, find_info_plist};
use super::pbxproj::register_pbx_resources;

pub(super) fn apply_ios_splashscreen(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<usize>, String> {
    let Some(splashscreen) = manifest_info.and_then(|info| info.splashscreen.as_ref()) else {
        return Ok(None);
    };
    if splashscreen.ios_style.as_deref() != Some("storyboard") {
        return Ok(None);
    }
    let zip_path = splashscreen
        .ios_storyboard
        .as_deref()
        .map(PathBuf::from)
        .ok_or_else(|| {
            "manifest 已配置 iOS storyboard 启动界面，但未配置 app-plus.distribute.splashscreen.ios.storyboard"
                .to_string()
        })?;
    if !zip_path.is_file() {
        return Err(format!(
            "manifest 配置的 iOS storyboard zip 不存在: {}",
            zip_path.display()
        ));
    }

    let launch_screen = find_launch_screen_storyboard(project_root, project_file)
        .ok_or_else(|| "SDK 自带 iOS 工程中未找到 LaunchScreen.storyboard".to_string())?;
    let target_dir = launch_screen
        .parent()
        .ok_or_else(|| format!("启动界面目录异常: {}", launch_screen.display()))?;
    let file = std::fs::File::open(&zip_path)
        .map_err(|e| format!("打开 iOS storyboard zip 失败 {}: {}", zip_path.display(), e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("解析 iOS storyboard zip 失败: {}", e))?;
    let mut storyboard_candidates = Vec::new();
    let mut resource_names = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("读取 iOS storyboard zip 条目失败: {}", e))?;
        if entry.is_dir() {
            continue;
        }
        let Some(enclosed) = entry.enclosed_name() else {
            return Err(format!(
                "iOS storyboard zip 包含不安全路径: {}",
                entry.name()
            ));
        };
        if enclosed
            .components()
            .any(|component| component.as_os_str() == "__MACOSX")
        {
            continue;
        }
        let Some(file_name) = enclosed.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with('.') || file_name.contains("/*") || file_name.contains("*/") {
            continue;
        }
        if enclosed.extension().and_then(|ext| ext.to_str()) == Some("storyboard") {
            let mut content = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut content)
                .map_err(|e| format!("读取 iOS storyboard 失败: {}", e))?;
            let priority = if file_name.eq_ignore_ascii_case("LaunchScreen.storyboard") {
                0
            } else {
                1
            };
            storyboard_candidates.push((priority, file_name.to_string(), content));
            continue;
        }

        let output = target_dir.join(file_name);
        let mut output_file = std::fs::File::create(&output)
            .map_err(|e| format!("创建启动界面资源失败 {}: {}", output.display(), e))?;
        std::io::copy(&mut entry, &mut output_file)
            .map_err(|e| format!("复制启动界面资源失败 {}: {}", output.display(), e))?;
        resource_names.push(file_name.to_string());
    }

    storyboard_candidates.sort_by_key(|candidate| candidate.0);
    let (_, source_name, storyboard) =
        storyboard_candidates.into_iter().next().ok_or_else(|| {
            format!(
                "iOS storyboard zip 中未找到 .storyboard 文件: {}",
                zip_path.display()
            )
        })?;
    std::fs::write(&launch_screen, storyboard).map_err(|e| {
        format!(
            "写入 manifest 启动界面 {} -> {} 失败: {}",
            source_name,
            launch_screen.display(),
            e
        )
    })?;

    resource_names.sort();
    resource_names.dedup();
    register_pbx_resources(project_file, &resource_names)?;
    Ok(Some(resource_names.len()))
}

fn find_launch_screen_storyboard(project_root: &Path, project_file: &Path) -> Option<PathBuf> {
    if let Some(info_plist) = find_info_plist(project_root, project_file) {
        let candidate = info_plist.parent()?.join("LaunchScreen.storyboard");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    find_file_named_skipping_bundles(project_root, "LaunchScreen.storyboard")
}
