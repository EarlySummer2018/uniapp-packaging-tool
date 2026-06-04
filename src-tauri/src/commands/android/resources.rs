//! Android 资源导入

use crate::commands::android::types::emit_log;
use std::path::Path;

pub fn import_uniapp_assets(
    resource_dir: &Path,
    workspace: &Path,
    app_id: &str,
) -> Result<(), String> {
    let apps_root = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/assets/apps");
    // 先清空 SDK 模板自带的默认 apps 目录（如 __UNI__DEMO__），再完整复制用户资源
    if apps_root.exists() {
        std::fs::remove_dir_all(&apps_root).map_err(|e| format!("清空 apps 目录失败: {}", e))?;
    }
    crate::utils::fs::ensure_directory(&apps_root).map_err(|e| e.to_string())?;
    let dest = apps_root.join(app_id);
    crate::utils::fs::copy_recursive(resource_dir, &dest)
        .map_err(|e| format!("导入 UniApp 资源失败: {}", e))
}

pub fn update_dcloud_control(workspace: &Path, app_id: &str) -> Result<(), String> {
    let path = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/assets/data/dcloud_control.xml");
    if !path.exists() {
        return Err(format!("dcloud_control.xml 不存在: {}", path.display()));
    }
    let content = crate::utils::fs::read_file_to_string(&path).map_err(|e| e.to_string())?;
    let updated = crate::utils::xml::set_xml_attribute(&content, "/apps/app", "appid", app_id)
        .or_else(|_| crate::utils::xml::set_xml_attribute(&content, "/hbuilder", "appid", app_id))
        .map_err(|e| format!("设置 dcloud_control.xml appid 失败: {}", e))?;
    crate::utils::fs::write_string_to_file(&path, &updated).map_err(|e| e.to_string())
}

pub fn copy_sdk_assets(
    sdk_assets: &Path,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let src = sdk_assets.join("data");
    let dst = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/assets/data");
    if src.exists() {
        crate::utils::fs::copy_recursive(&src, &dst)
            .map_err(|e| format!("复制 SDK assets/data 失败: {}", e))?;
    } else {
        emit_log(
            window,
            "warn",
            &format!("SDK assets/data 不存在: {}", src.display()),
            None,
        );
    }
    Ok(())
}
