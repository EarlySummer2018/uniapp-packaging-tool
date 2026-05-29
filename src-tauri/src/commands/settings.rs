use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub cache_dir: String,
    pub default_cache_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationResult {
    pub success: bool,
    pub cache_dir: String,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationLogEvent {
    pub level: String,
    pub message: String,
}

#[tauri::command]
pub async fn get_app_settings() -> Result<AppSettings, String> {
    Ok(AppSettings {
        cache_dir: crate::utils::fs::get_unipack_home()
            .to_string_lossy()
            .to_string(),
        default_cache_dir: crate::utils::fs::default_unipack_home()
            .to_string_lossy()
            .to_string(),
    })
}

#[tauri::command]
pub async fn migrate_cache_dir(
    new_cache_dir: String,
    window: tauri::Window,
) -> Result<MigrationResult, String> {
    let target = crate::utils::fs::expand_home(new_cache_dir.trim());
    if target.as_os_str().is_empty() {
        return Ok(MigrationResult {
            success: false,
            cache_dir: String::new(),
            logs: vec!["缓存路径不能为空".to_string()],
            error: Some("缓存路径不能为空".to_string()),
        });
    }

    let current = crate::utils::fs::get_unipack_home();
    let mut logs = Vec::new();
    push_log(
        &mut logs,
        Some(&window),
        "info",
        format!("当前缓存目录: {}", current.display()),
    );
    push_log(
        &mut logs,
        Some(&window),
        "info",
        format!("目标缓存目录: {}", target.display()),
    );

    match migrate_cache_dir_sync(&current, &target, &mut logs, Some(&window)) {
        Ok(()) => {
            crate::utils::fs::save_configured_unipack_home(&target)
                .map_err(|e| format!("保存设置失败: {}", e))?;
            push_log(
                &mut logs,
                Some(&window),
                "success",
                "已保存新的缓存目录设置",
            );
            Ok(MigrationResult {
                success: true,
                cache_dir: target.to_string_lossy().to_string(),
                logs,
                error: None,
            })
        }
        Err(error) => {
            push_log(
                &mut logs,
                Some(&window),
                "error",
                format!("迁移失败: {}", error),
            );
            Ok(MigrationResult {
                success: false,
                cache_dir: target.to_string_lossy().to_string(),
                logs,
                error: Some(error),
            })
        }
    }
}

fn migrate_cache_dir_sync(
    current: &Path,
    target: &Path,
    logs: &mut Vec<String>,
    window: Option<&tauri::Window>,
) -> Result<(), String> {
    if paths_equal(current, target) {
        push_log(logs, window, "info", "目标目录与当前目录一致，无需迁移");
        return Ok(());
    }

    reject_nested_migration(current, target)?;
    std::fs::create_dir_all(target)
        .map_err(|e| format!("创建目标目录失败 {}: {}", target.display(), e))?;
    push_log(logs, window, "info", "目标目录已准备");

    if current.exists() {
        copy_dir_contents(current, target, logs, window)?;
        push_log(logs, window, "success", "缓存数据复制完成");
    } else {
        push_log(logs, window, "info", "当前缓存目录不存在，将仅保存新路径");
    }

    Ok(())
}

fn push_log(
    logs: &mut Vec<String>,
    window: Option<&tauri::Window>,
    level: &str,
    message: impl Into<String>,
) {
    let message = message.into();
    logs.push(message.clone());
    if let Some(window) = window {
        let _ = window.emit(
            "settings-migration-log",
            MigrationLogEvent {
                level: level.to_string(),
                message,
            },
        );
    }
}

fn reject_nested_migration(current: &Path, target: &Path) -> Result<(), String> {
    if !current.exists() {
        return Ok(());
    }
    let current = current
        .canonicalize()
        .map_err(|e| format!("解析当前缓存目录失败: {}", e))?;
    let target_for_compare = if target.exists() {
        target
            .canonicalize()
            .map_err(|e| format!("解析目标缓存目录失败: {}", e))?
    } else {
        target.to_path_buf()
    };
    if target_for_compare.starts_with(&current) {
        return Err("目标目录不能位于当前缓存目录内部，请选择独立目录".to_string());
    }
    Ok(())
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn copy_dir_contents(
    src: &Path,
    dst: &Path,
    logs: &mut Vec<String>,
    window: Option<&tauri::Window>,
) -> Result<(), String> {
    for entry in
        std::fs::read_dir(src).map_err(|e| format!("读取目录失败 {}: {}", src.display(), e))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path)
                .map_err(|e| format!("创建目录失败 {}: {}", dst_path.display(), e))?;
            push_log(
                logs,
                window,
                "info",
                format!("复制目录: {}", src_path.display()),
            );
            copy_dir_contents(&src_path, &dst_path, logs, window)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
            }
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制文件失败 {}: {}", src_path.display(), e))?;
            push_log(
                logs,
                window,
                "info",
                format!("复制文件: {}", src_path.display()),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_cache_dir_copies_contents() {
        let root = std::env::temp_dir().join(format!("unipack-settings-{}", uuid::Uuid::new_v4()));
        let src = root.join("old");
        let dst = root.join("new");
        std::fs::create_dir_all(src.join("projects/p1")).unwrap();
        std::fs::write(src.join("projects/p1/config.json"), "{}").unwrap();

        let mut logs = Vec::new();
        migrate_cache_dir_sync(&src, &dst, &mut logs, None).unwrap();

        assert!(dst.join("projects/p1/config.json").exists());
        assert!(logs.iter().any(|line| line.contains("缓存数据复制完成")));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn migrate_cache_dir_rejects_nested_target() {
        let root = std::env::temp_dir().join(format!("unipack-settings-{}", uuid::Uuid::new_v4()));
        let src = root.join("old");
        let dst = src.join("nested");
        std::fs::create_dir_all(&src).unwrap();

        let mut logs = Vec::new();
        let result = migrate_cache_dir_sync(&src, &dst, &mut logs, None);

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(root);
    }
}
