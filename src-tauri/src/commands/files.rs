use serde::Serialize;
use std::path::{Component, Path, PathBuf};

#[tauri::command]
pub async fn read_text_file(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("不是文件: {}", path.display()));
    }
    std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))
}

#[tauri::command]
pub async fn append_build_log(
    project_id: String,
    build_id: String,
    lines: Vec<String>,
) -> Result<String, String> {
    let log_path = build_log_path(&project_id, &build_id)?;
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建日志目录失败: {}", e))?;
    }
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(content.as_bytes())
        })
        .map_err(|e| format!("写入构建日志失败: {}", e))?;
    Ok(log_path.to_string_lossy().to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupBuildTempResult {
    pub items: Vec<CleanupBuildTempItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupBuildTempItem {
    pub label: String,
    pub path: String,
    pub status: String,
    pub message: String,
}

#[tauri::command]
pub async fn cleanup_build_temporary_files(
    project_id: String,
    build_id: Option<String>,
    resource_path: Option<String>,
) -> Result<CleanupBuildTempResult, String> {
    let project_dir = crate::utils::fs::get_project_config_dir(&project_id);
    let mut items = Vec::new();

    if let Some(build_id) = build_id.as_deref().filter(|id| !id.trim().is_empty()) {
        let workspace = project_dir.join("workspace").join(safe_file_name(build_id));
        items.push(remove_cleanup_target("构建工作区", &workspace));
    }

    if let Some(resource_path) = resource_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        let resource_path = PathBuf::from(resource_path);
        if !resource_path.exists() {
            items.push(CleanupBuildTempItem {
                label: "导入资源临时目录".to_string(),
                path: resource_path.to_string_lossy().to_string(),
                status: "skipped".to_string(),
                message: "导入资源临时目录不存在，已跳过".to_string(),
            });
        } else {
            match resource_import_cleanup_root(&project_dir, &resource_path) {
                Ok(Some(cleanup_root)) => {
                    items.push(remove_cleanup_target("导入资源临时目录", &cleanup_root));
                }
                Ok(None) => {
                    items.push(CleanupBuildTempItem {
                        label: "导入资源临时目录".to_string(),
                        path: resource_path.to_string_lossy().to_string(),
                        status: "skipped".to_string(),
                        message: "未找到可清理的导入资源临时目录，已跳过".to_string(),
                    });
                }
                Err(error) => {
                    items.push(CleanupBuildTempItem {
                        label: "导入资源临时目录".to_string(),
                        path: resource_path.to_string_lossy().to_string(),
                        status: "failed".to_string(),
                        message: error,
                    });
                }
            }
        }
    }

    Ok(CleanupBuildTempResult { items })
}

pub fn build_log_path(project_id: &str, build_id: &str) -> Result<PathBuf, String> {
    let safe_build_id = safe_file_name(build_id);
    if safe_build_id.is_empty() {
        return Err("build_id 不能为空".to_string());
    }
    Ok(crate::utils::fs::get_project_config_dir(project_id)
        .join("logs")
        .join(format!("{}.log", safe_build_id)))
}

fn remove_cleanup_target(label: &str, path: &Path) -> CleanupBuildTempItem {
    let path_text = path.to_string_lossy().to_string();
    if !path.exists() {
        return CleanupBuildTempItem {
            label: label.to_string(),
            path: path_text,
            status: "skipped".to_string(),
            message: format!("{}不存在，已跳过", label),
        };
    }

    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };

    match result {
        Ok(()) => CleanupBuildTempItem {
            label: label.to_string(),
            path: path_text,
            status: "removed".to_string(),
            message: format!("已删除{}", label),
        },
        Err(error) => CleanupBuildTempItem {
            label: label.to_string(),
            path: path_text,
            status: "failed".to_string(),
            message: format!("删除{}失败: {}", label, error),
        },
    }
}

fn resource_import_cleanup_root(
    project_dir: &Path,
    resource_path: &Path,
) -> Result<Option<PathBuf>, String> {
    let resources_dir = project_dir.join("resources");
    if !resources_dir.exists() || !resource_path.exists() {
        return Ok(None);
    }

    let resources_dir = resources_dir
        .canonicalize()
        .map_err(|e| format!("解析项目资源目录失败: {}", e))?;
    let resource_path = resource_path
        .canonicalize()
        .map_err(|e| format!("解析导入资源路径失败: {}", e))?;

    if !resource_path.starts_with(&resources_dir) {
        return Err(format!(
            "拒绝清理项目资源目录之外的路径: {}",
            resource_path.display()
        ));
    }

    let relative = resource_path
        .strip_prefix(&resources_dir)
        .map_err(|e| format!("解析导入资源相对路径失败: {}", e))?;
    let Some(first_component) = relative.components().next() else {
        return Err("拒绝清理整个项目资源目录".to_string());
    };
    let Component::Normal(import_dir) = first_component else {
        return Err("导入资源路径不合法".to_string());
    };

    Ok(Some(resources_dir.join(import_dir)))
}

fn safe_file_name(value: &str) -> String {
    value
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_log_path_sanitizes_build_id() {
        let path = build_log_path("project-a", "android:bad/name").unwrap();
        assert!(path.ends_with("android_bad_name.log"));
        assert!(path.to_string_lossy().contains(".unipack"));
    }

    #[test]
    fn build_log_path_rejects_empty_id() {
        assert!(build_log_path("project-a", "   ").is_err());
    }

    #[test]
    fn resource_cleanup_root_uses_import_batch_dir() {
        let root = std::env::temp_dir().join(format!("unipack-cleanup-{}", uuid::Uuid::new_v4()));
        let project_dir = root.join("project");
        let imported = project_dir.join("resources/20260529-120000/resources/__UNI__AA97490");
        std::fs::create_dir_all(&imported).unwrap();

        let cleanup_root = resource_import_cleanup_root(&project_dir, &imported)
            .unwrap()
            .unwrap();

        assert_eq!(
            cleanup_root,
            project_dir
                .join("resources/20260529-120000")
                .canonicalize()
                .unwrap()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resource_cleanup_root_rejects_external_paths() {
        let root = std::env::temp_dir().join(format!("unipack-cleanup-{}", uuid::Uuid::new_v4()));
        let project_dir = root.join("project");
        let resources_dir = project_dir.join("resources/20260529-120000");
        let external = root.join("external/resources");
        std::fs::create_dir_all(&resources_dir).unwrap();
        std::fs::create_dir_all(&external).unwrap();

        let result = resource_import_cleanup_root(&project_dir, &external);

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(root);
    }
}
