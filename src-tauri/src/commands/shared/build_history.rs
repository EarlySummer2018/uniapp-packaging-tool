use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRecord {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
    pub platform: String,
    pub status: String,
    pub artifact_path: Option<String>,
    pub artifact_size_mb: Option<f64>,
    pub version_name: String,
    pub version_code: u32,
    pub build_mode: String,
    #[serde(default)]
    pub build_source: Option<String>,
    #[serde(default)]
    pub cloud_run_url: Option<String>,
    pub duration_secs: u64,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub error_message: Option<String>,
    pub log_path: Option<String>,
    pub resource_path: Option<String>,
}

fn get_history_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
        .join("build-history.json")
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    Ok(())
}

fn load_history() -> Result<Vec<BuildRecord>, String> {
    let path = get_history_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read build history: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse build history JSON: {}", e))
}

fn save_history(records: &[BuildRecord]) -> Result<(), String> {
    let path = get_history_file_path();
    ensure_parent_dir(&path)?;
    let content = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize build history: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write build history: {}", e))
}

#[tauri::command]
pub async fn get_build_history(project_id: Option<String>) -> Result<Vec<BuildRecord>, String> {
    let mut records = load_history()?;

    if let Some(pid) = project_id {
        records.retain(|r| r.project_id == pid);
    }

    records.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(records)
}

#[tauri::command]
pub async fn add_build_record(record: BuildRecord) -> Result<String, String> {
    let mut records = load_history()?;
    records.push(record);
    save_history(&records)?;
    Ok(records.last().map(|r| r.id.clone()).unwrap_or_default())
}

#[tauri::command]
pub async fn update_build_record(id: String, update: serde_json::Value) -> Result<(), String> {
    let mut records = load_history()?;

    if let Some(ref mut record) = records.iter_mut().find(|r| r.id == id) {
        if let Some(status) = update.get("status").and_then(|v| v.as_str()) {
            record.status = status.to_string();
        }
        if let Some(artifact_path) = update.get("artifact_path").and_then(|v| v.as_str()) {
            record.artifact_path = Some(artifact_path.to_string());
        }
        if let Some(artifact_size) = update.get("artifact_size_mb").and_then(|v| v.as_f64()) {
            record.artifact_size_mb = Some(artifact_size);
        }
        if let Some(finished_at) = update.get("finished_at").and_then(|v| v.as_str()) {
            record.finished_at = Some(finished_at.to_string());
        }
        if let Some(error_message) = update.get("error_message").and_then(|v| v.as_str()) {
            record.error_message = Some(error_message.to_string());
        }
        if let Some(log_path) = update.get("log_path").and_then(|v| v.as_str()) {
            record.log_path = Some(log_path.to_string());
        }
        if let Some(duration_secs) = update.get("duration_secs").and_then(|v| v.as_u64()) {
            record.duration_secs = duration_secs;
        }
        if let Some(resource_path) = update.get("resource_path").and_then(|v| v.as_str()) {
            record.resource_path = Some(resource_path.to_string());
        }
        if let Some(build_source) = update.get("build_source").and_then(|v| v.as_str()) {
            record.build_source = Some(build_source.to_string());
        }
        if let Some(cloud_run_url) = update.get("cloud_run_url").and_then(|v| v.as_str()) {
            record.cloud_run_url = Some(cloud_run_url.to_string());
        }

        save_history(&records)?;
        Ok(())
    } else {
        Err(format!("Build record with id {} not found", id))
    }
}

fn clean_record_resources(record: &BuildRecord) {
    if let Some(log_path) = &record.log_path {
        let log_path = PathBuf::from(log_path);
        if log_path.exists() {
            let _ = fs::remove_file(&log_path);
        }
    }

    if let Some(resource_path) = &record.resource_path {
        let resource_path = PathBuf::from(resource_path);
        if resource_path.exists() {
            let timestamp_dir = resource_path.parent().and_then(|p| p.parent());
            if let Some(timestamp_dir) = timestamp_dir {
                let _ = fs::remove_dir_all(timestamp_dir);
            }
        }
    }

    let workspace_path = crate::utils::fs::get_project_config_dir(&record.project_id)
        .join("workspace")
        .join(&record.id);
    if workspace_path.exists() {
        let _ = fs::remove_dir_all(&workspace_path);
    }
}

#[tauri::command]
pub async fn clear_build_history(project_id: Option<String>) -> Result<(), String> {
    let records = load_history()?;

    if let Some(pid) = project_id {
        let to_delete: Vec<BuildRecord> = records
            .iter()
            .filter(|r| r.project_id == pid)
            .cloned()
            .collect();

        for record in to_delete {
            clean_record_resources(&record);
        }

        let mut records = records;
        records.retain(|r| r.project_id != pid);
        save_history(&records)?;
    } else {
        for record in records {
            clean_record_resources(&record);
        }

        let path = get_history_file_path();
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove build history file: {}", e))?;
        }
    }
    Ok(())
}
