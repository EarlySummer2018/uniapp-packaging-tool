//! Android 资源导入

use crate::commands::android::types::emit_log;
use std::path::Path;

const DCLOUD_PROPERTIES_FILE: &str = "dcloud_properties.xml";
const DCLOUD_PROPERTIES_SCAFFOLD: &str =
    "<properties>\n\t<features>\n\t</features>\n</properties>\n";

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
        copy_sdk_data_preserving_properties(&src, &dst)
            .map_err(|e| format!("复制 SDK assets/data 失败: {}", e))?;
    } else {
        emit_log(
            window,
            "warn",
            &format!("SDK assets/data 不存在: {}", src.display()),
            None,
        );
    }
    ensure_dcloud_properties_scaffold(&dst)?;
    Ok(())
}

fn copy_sdk_data_preserving_properties(src: &Path, dst: &Path) -> Result<(), String> {
    crate::utils::fs::ensure_directory(dst).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let is_root_properties = src_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == DCLOUD_PROPERTIES_FILE)
            .unwrap_or(false);
        if is_root_properties {
            continue;
        }

        if src_path.is_dir() {
            crate::utils::fs::copy_recursive(&src_path, &dst_path).map_err(|e| e.to_string())?;
        } else {
            crate::utils::fs::copy_file(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn ensure_dcloud_properties_scaffold(data_dir: &Path) -> Result<(), String> {
    let properties = data_dir.join(DCLOUD_PROPERTIES_FILE);
    if properties.exists() {
        return Ok(());
    }
    crate::utils::fs::write_string_to_file(&properties, DCLOUD_PROPERTIES_SCAFFOLD)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_data_copy_preserves_existing_dcloud_properties() {
        let root = std::env::temp_dir().join(format!("unipack-sdk-data-{}", uuid::Uuid::new_v4()));
        let src = root.join("sdk/data");
        let dst = root.join("workspace/simpleDemo/src/main/assets/data");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&dst).unwrap();
        std::fs::write(src.join(DCLOUD_PROPERTIES_FILE), "<full/>").unwrap();
        std::fs::write(src.join("dcloud_control.xml"), "<control/>").unwrap();
        std::fs::write(dst.join(DCLOUD_PROPERTIES_FILE), "<template/>").unwrap();

        copy_sdk_data_preserving_properties(&src, &dst).unwrap();
        ensure_dcloud_properties_scaffold(&dst).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst.join(DCLOUD_PROPERTIES_FILE)).unwrap(),
            "<template/>"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("dcloud_control.xml")).unwrap(),
            "<control/>"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn sdk_data_copy_creates_properties_scaffold_when_missing() {
        let root = std::env::temp_dir().join(format!("unipack-sdk-data-{}", uuid::Uuid::new_v4()));
        let src = root.join("sdk/data");
        let dst = root.join("workspace/simpleDemo/src/main/assets/data");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join(DCLOUD_PROPERTIES_FILE), "<full/>").unwrap();

        copy_sdk_data_preserving_properties(&src, &dst).unwrap();
        ensure_dcloud_properties_scaffold(&dst).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst.join(DCLOUD_PROPERTIES_FILE)).unwrap(),
            DCLOUD_PROPERTIES_SCAFFOLD
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
