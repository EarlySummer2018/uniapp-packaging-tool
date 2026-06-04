use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub name: String,
    pub app_id: String,
    #[serde(default, skip_serializing)]
    pub dcloud_app_key: String,
    pub version: String,
    pub version_code: u32,
    pub icon1024: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidConfig {
    pub enabled: bool,
    #[serde(default)]
    pub dcloud_app_key: String,
    pub package_name: String,
    pub min_sdk_version: u32,
    pub target_sdk_version: u32,
    pub compile_sdk_version: u32,
    pub keystore: AndroidKeystoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidKeystoreConfig {
    pub path: String,
    pub alias: String,
    pub has_store_password: bool,
    pub has_key_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IosConfig {
    pub enabled: bool,
    #[serde(default)]
    pub dcloud_app_key: String,
    pub bundle_id: String,
    pub team_id: String,
    pub provisioning_profile: String,
    pub certificate: String,
    pub export_method: String,
    pub has_certificate_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonyConfig {
    pub enabled: bool,
    pub bundle_name: String,
    pub runtime_version: String,
    pub signing_config: HarmonySigningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonySigningConfig {
    pub store_file: String,
    pub key_alias: String,
    pub has_store_password: bool,
    pub has_key_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub local_path: String,
    pub app: AppConfig,
    pub android: AndroidConfig,
    pub ios: IosConfig,
    pub harmony: HarmonyConfig,
    #[serde(default)]
    pub android_module_config: HashMap<String, String>,
    pub output_dir: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            name: String::new(),
            description: String::new(),
            local_path: String::new(),
            app: AppConfig {
                name: String::new(),
                app_id: String::new(),
                dcloud_app_key: String::new(),
                version: "1.0.0".to_string(),
                version_code: 1,
                icon1024: String::new(),
            },
            android: AndroidConfig {
                enabled: true,
                dcloud_app_key: String::new(),
                package_name: String::new(),
                min_sdk_version: 21,
                target_sdk_version: 34,
                compile_sdk_version: 35,
                keystore: AndroidKeystoreConfig {
                    path: String::new(),
                    alias: String::new(),
                    has_store_password: false,
                    has_key_password: false,
                },
            },
            ios: IosConfig {
                enabled: cfg!(target_os = "macos"),
                dcloud_app_key: String::new(),
                bundle_id: String::new(),
                team_id: String::new(),
                provisioning_profile: String::new(),
                certificate: String::new(),
                export_method: "app-store".to_string(),
                has_certificate_password: false,
            },
            harmony: HarmonyConfig {
                enabled: false,
                bundle_name: String::new(),
                runtime_version: String::new(),
                signing_config: HarmonySigningConfig {
                    store_file: String::new(),
                    key_alias: String::new(),
                    has_store_password: false,
                    has_key_password: false,
                },
            },
            android_module_config: HashMap::new(),
            output_dir: dirs::desktop_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
                .join("unipack-output")
                .to_string_lossy()
                .to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

fn projects_dir() -> PathBuf {
    crate::utils::fs::get_unipack_home().join("projects")
}

fn project_dir(id: &str) -> PathBuf {
    projects_dir().join(id)
}

pub fn project_file_path(id: &str) -> PathBuf {
    project_dir(id).join("config.json")
}

pub fn load_project_config_sync(project_id: &str) -> Result<ProjectConfig, String> {
    let path = project_file_path(project_id);
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        return serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e));
    }

    let legacy = crate::utils::fs::get_legacy_project_file(project_id);
    if legacy.exists() {
        let content = std::fs::read_to_string(&legacy)
            .map_err(|e| format!("Failed to read legacy config {}: {}", legacy.display(), e))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse legacy config {}: {}", legacy.display(), e))?;
        let migrated = migrate_legacy_value(value);
        save_project_config_sync(&migrated)?;
        return Ok(migrated);
    }

    Err(format!("Project not found: {}", project_id))
}

pub fn save_project_config_sync(config: &ProjectConfig) -> Result<(), String> {
    let dir = project_dir(&config.id);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(project_file_path(&config.id), json).map_err(|e| e.to_string())
}

fn migrate_legacy_value(value: serde_json::Value) -> ProjectConfig {
    let mut project = ProjectConfig::default();
    project.id = value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or(&project.id)
        .to_string();
    project.name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    project.description = value
        .get("description")
        .or_else(|| value.get("app").and_then(|v| v.get("description")))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    project.local_path = value
        .get("localPath")
        .or_else(|| value.get("local_path"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    if let Some(app) = value.get("app") {
        project.app.name = app
            .get("name")
            .or_else(|| app.get("displayName"))
            .and_then(|v| v.as_str())
            .unwrap_or(&project.name)
            .to_string();
        project.app.app_id = app
            .get("appId")
            .or_else(|| app.get("app_id"))
            .or_else(|| app.get("packageId"))
            .or_else(|| app.get("package_id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        project.app.dcloud_app_key = app
            .get("dcloudAppKey")
            .or_else(|| app.get("dcloud_app_key"))
            .or_else(|| app.get("appKey"))
            .or_else(|| app.get("app_key"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        project.app.version = app
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();
        project.app.version_code = app
            .get("versionCode")
            .or_else(|| app.get("version_code"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
    }

    if let Some(android) = value.get("android") {
        project.android.dcloud_app_key = android
            .get("dcloudAppKey")
            .or_else(|| android.get("dcloud_app_key"))
            .or_else(|| android.get("appKey"))
            .or_else(|| android.get("app_key"))
            .and_then(|v| v.as_str())
            .unwrap_or(&project.app.dcloud_app_key)
            .to_string();
        project.android.package_name = android
            .get("packageName")
            .or_else(|| android.get("applicationId"))
            .or_else(|| android.get("application_id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        project.android.min_sdk_version = android
            .get("minSdkVersion")
            .or_else(|| android.get("min_sdk"))
            .and_then(|v| v.as_u64())
            .unwrap_or(21) as u32;
        project.android.target_sdk_version = android
            .get("targetSdkVersion")
            .or_else(|| android.get("target_sdk"))
            .and_then(|v| v.as_u64())
            .unwrap_or(34) as u32;
        project.android.compile_sdk_version = android
            .get("compileSdkVersion")
            .or_else(|| android.get("compile_sdk"))
            .and_then(|v| v.as_u64())
            .unwrap_or(35) as u32;
    }

    if let Some(ios) = value.get("ios") {
        project.ios.dcloud_app_key = ios
            .get("dcloudAppKey")
            .or_else(|| ios.get("dcloud_app_key"))
            .or_else(|| ios.get("appKey"))
            .or_else(|| ios.get("app_key"))
            .and_then(|v| v.as_str())
            .unwrap_or(&project.app.dcloud_app_key)
            .to_string();
        project.ios.bundle_id = ios
            .get("bundleId")
            .or_else(|| ios.get("bundleIdentifier"))
            .or_else(|| ios.get("bundle_identifier"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        project.ios.team_id = ios
            .get("teamId")
            .or_else(|| ios.get("team_id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
    }

    if project.app.name.is_empty() {
        project.app.name = project.name.clone();
    }
    if project.android.package_name.is_empty() {
        project.android.package_name = project.app.app_id.clone();
    }
    if project.ios.bundle_id.is_empty() {
        project.ios.bundle_id = project.android.package_name.clone();
    }
    if project.android.dcloud_app_key.is_empty() {
        project.android.dcloud_app_key = project.app.dcloud_app_key.clone();
    }
    if project.ios.dcloud_app_key.is_empty() {
        project.ios.dcloud_app_key = project.app.dcloud_app_key.clone();
    }
    project.app.dcloud_app_key.clear();
    project
}

#[tauri::command]
pub async fn create_project(
    name: String,
    description: Option<String>,
    config: Option<ProjectConfig>,
) -> Result<ProjectConfig, String> {
    let mut project = config.unwrap_or_default();
    if project.id.is_empty() {
        project.id = uuid::Uuid::new_v4().to_string();
    }
    let now = chrono::Utc::now().to_rfc3339();
    project.name = name.clone();
    project.description = description.unwrap_or_default();
    if project.app.name.is_empty() {
        project.app.name = name;
    }
    project.created_at = now.clone();
    project.updated_at = now;
    save_project_config_sync(&project)?;
    Ok(project)
}

#[tauri::command]
pub async fn get_project(project_id: String) -> Result<ProjectConfig, String> {
    load_project_config_sync(&project_id)
}

#[tauri::command]
pub async fn update_project(
    project_id: String,
    updates: serde_json::Value,
) -> Result<ProjectConfig, String> {
    let mut project = load_project_config_sync(&project_id)?;
    let mut project_value = serde_json::to_value(&project).map_err(|e| e.to_string())?;
    merge_json_value(&mut project_value, &updates);
    project = serde_json::from_value(project_value).map_err(|e| e.to_string())?;
    migrate_platform_app_keys(&mut project);
    project.updated_at = chrono::Utc::now().to_rfc3339();
    save_project_config_sync(&project)?;
    Ok(project)
}

#[tauri::command]
pub async fn delete_project(project_id: String) -> Result<(), String> {
    let dir = project_dir(&project_id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    crate::commands::build_history::clear_build_history(Some(project_id)).await?;
    Ok(())
}

#[tauri::command]
pub async fn list_projects() -> Result<Vec<ProjectConfig>, String> {
    let dir = projects_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    migrate_legacy_projects().ok();

    let mut projects = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().join("config.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(project) = serde_json::from_str::<ProjectConfig>(&content) {
                    projects.push(project);
                }
            }
        }
    }

    projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(projects)
}

#[tauri::command]
pub async fn save_project_config(
    project_id: String,
    mut config: ProjectConfig,
) -> Result<ProjectConfig, String> {
    config.id = project_id;
    migrate_platform_app_keys(&mut config);
    config.updated_at = chrono::Utc::now().to_rfc3339();
    if config.created_at.is_empty() {
        config.created_at = config.updated_at.clone();
    }
    save_project_config_sync(&config)?;
    Ok(config)
}

fn migrate_platform_app_keys(project: &mut ProjectConfig) {
    if !project.app.dcloud_app_key.is_empty() {
        if project.android.dcloud_app_key.is_empty() {
            project.android.dcloud_app_key = project.app.dcloud_app_key.clone();
        }
        if project.ios.dcloud_app_key.is_empty() {
            project.ios.dcloud_app_key = project.app.dcloud_app_key.clone();
        }
        project.app.dcloud_app_key.clear();
    }
}

#[tauri::command]
pub async fn save_signing_secret(
    project_id: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let account = format!("{}-{}", project_id, key);
    crate::utils::keychain::store_password(&account, &value)
        .map_err(|e| format!("保存密钥失败: {}", e))
}

#[tauri::command]
pub async fn delete_signing_secret(project_id: String, key: String) -> Result<bool, String> {
    let account = format!("{}-{}", project_id, key);
    crate::utils::keychain::delete_password(&account).map_err(|e| format!("删除密钥失败: {}", e))
}

#[tauri::command]
pub async fn get_signing_secret_status(
    project_id: String,
    keys: Vec<String>,
) -> Result<Vec<(String, bool)>, String> {
    let mut statuses = Vec::new();
    for key in keys {
        let account = format!("{}-{}", project_id, key);
        let has_value = crate::utils::keychain::get_password(&account)
            .map_err(|e| format!("读取密钥状态失败: {}", e))?
            .is_some();
        statuses.push((key, has_value));
    }
    Ok(statuses)
}

fn migrate_legacy_projects() -> Result<(), String> {
    let legacy_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unipack-tool")
        .join("projects");
    if !legacy_dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(&legacy_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                let project = migrate_legacy_value(value);
                if !project_file_path(&project.id).exists() {
                    save_project_config_sync(&project)?;
                }
            }
        }
    }
    Ok(())
}

fn merge_json_value(target: &mut serde_json::Value, source: &serde_json::Value) {
    if let (Some(t), Some(s)) = (target.as_object_mut(), source.as_object()) {
        for (k, v) in s {
            if t.contains_key(k) {
                merge_json_value(&mut t[k], v);
            } else {
                t.insert(k.clone(), v.clone());
            }
        }
    } else {
        *target = source.clone();
    }
}
