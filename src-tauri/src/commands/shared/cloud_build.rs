use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio_util::io::ReaderStream;

use super::sdk_cache::SdkCacheReference;

const GITHUB_API: &str = "https://api.github.com";
const GITHUB_UPLOADS: &str = "https://uploads.github.com";
const TOKEN_ACCOUNT: &str = "github-cloud-build-token";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCloudBuildConfig {
    pub owner: String,
    pub repo: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub workflow_file: String,
}

impl Default for GithubCloudBuildConfig {
    fn default() -> Self {
        Self {
            owner: String::new(),
            repo: String::new(),
            ref_name: "main".to_string(),
            workflow_file: "cloud-build.yml".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCloudBuildConfigView {
    #[serde(flatten)]
    pub config: GithubCloudBuildConfig,
    pub has_token: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCloudBuildSecretStatus {
    pub has_token: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCloudBuildTestResult {
    pub ok: bool,
    pub repository_private: bool,
    pub workflow_found: bool,
    pub message: String,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCloudBuildRequest {
    pub project_id: String,
    pub platform: String,
    pub resource_path: String,
    pub build_id: Option<String>,
    pub manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    pub module_config: Option<HashMap<String, String>>,
    pub ios_packaging_mode: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudPayloadMetadata {
    version: u8,
    build_id: String,
    platform: String,
    project_id: String,
    project_config: crate::commands::project::ProjectConfig,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    module_config: Option<HashMap<String, String>>,
    ios_packaging_mode: Option<String>,
    sdk_cache: SdkCacheReference,
    signing: CloudPayloadSigning,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudPayloadSigning {
    android_keystore_file: Option<String>,
    android_store_password: Option<String>,
    android_key_password: Option<String>,
    ios_certificate_file: Option<String>,
    ios_certificate_password: Option<String>,
    ios_provisioning_profile_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRepo {
    private: bool,
    html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubWorkflow {
    path: Option<String>,
    html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    id: u64,
    upload_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingRelease {
    release_id: u64,
    tag: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    repo: String,
}

#[derive(Debug, Deserialize)]
struct WorkflowRuns {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Deserialize)]
struct WorkflowRun {
    id: u64,
    html_url: String,
    display_title: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkflowArtifacts {
    artifacts: Vec<WorkflowArtifact>,
}

#[derive(Debug, Deserialize)]
struct WorkflowArtifact {
    id: u64,
    name: String,
    expired: bool,
}

#[tauri::command]
pub async fn get_github_cloud_build_config() -> Result<GithubCloudBuildConfigView, String> {
    let config = load_config()?;
    let has_token = crate::utils::keychain::get_password(TOKEN_ACCOUNT)
        .map_err(|e| e.to_string())?
        .is_some();
    Ok(GithubCloudBuildConfigView { config, has_token })
}

#[tauri::command]
pub async fn save_github_cloud_build_config(config: GithubCloudBuildConfig) -> Result<(), String> {
    validate_config_shape(&config)?;
    save_config_sync(&config)
}

#[tauri::command]
pub async fn save_github_cloud_build_secret(token: String) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        crate::utils::keychain::delete_password(TOKEN_ACCOUNT)
            .map_err(|e| format!("清理 GitHub Token 失败: {}", e))?;
        return Ok(());
    }
    crate::utils::keychain::store_password(TOKEN_ACCOUNT, token)
        .map_err(|e| format!("保存 GitHub Token 失败: {}", e))
}

#[tauri::command]
pub async fn get_github_cloud_build_secret_status() -> Result<GithubCloudBuildSecretStatus, String>
{
    Ok(GithubCloudBuildSecretStatus {
        has_token: crate::utils::keychain::get_password(TOKEN_ACCOUNT)
            .map_err(|e| e.to_string())?
            .is_some(),
    })
}

#[tauri::command]
pub async fn test_github_cloud_build_config() -> Result<GithubCloudBuildTestResult, String> {
    let config = load_config()?;
    validate_config_ready(&config)?;
    let token = require_token()?;
    let client = GithubClient::new(token.clone())?;
    let repo = client.get_repo(&config).await?;
    let workflow = client.get_workflow(&config).await?;
    if !repo.private {
        return Ok(GithubCloudBuildTestResult {
            ok: false,
            repository_private: false,
            workflow_found: workflow.path.is_some(),
            message: "GitHub 云端打包需要私有仓库承载临时构建包".to_string(),
            html_url: repo.html_url,
        });
    }
    Ok(GithubCloudBuildTestResult {
        ok: true,
        repository_private: true,
        workflow_found: true,
        message: "GitHub 仓库和 workflow 校验通过".to_string(),
        html_url: workflow.html_url.or(repo.html_url),
    })
}

#[tauri::command]
pub async fn run_github_cloud_build(
    request: GithubCloudBuildRequest,
    window: tauri::Window,
) -> Result<crate::commands::android::types::BuildArtifact, String> {
    let config = load_config()?;
    validate_config_ready(&config)?;
    let token = require_token()?;
    let client = GithubClient::new(token.clone())?;
    let build_id = request
        .build_id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("github-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    validate_identifier("project_id", &request.project_id)?;
    validate_identifier("build_id", &build_id)?;
    let compile_sdk_version =
        crate::commands::project::load_project_config_sync(&request.project_id)?
            .android
            .compile_sdk_version;
    let platform = normalize_platform(&request.platform)?;
    emit_cloud_log(
        &window,
        &build_id,
        &platform,
        "info",
        "准备 GitHub 云端打包",
        Some(5),
    );

    let repo = client.get_repo(&config).await?;
    if !repo.private {
        return Err("GitHub 云端打包需要使用私有仓库，已拒绝上传构建包".to_string());
    }
    client.get_workflow(&config).await?;
    cleanup_pending_releases(
        &client,
        &config,
        &request.project_id,
        &window,
        &build_id,
        &platform,
    )
    .await?;

    let workspace = cloud_workspace(&request.project_id, &build_id)?;
    let _workspace_cleanup = CloudWorkspaceCleanup(workspace.clone());
    emit_cloud_log(
        &window,
        &build_id,
        &platform,
        "info",
        "校验并准备 DCloud 离线 SDK 云端缓存",
        Some(10),
    );
    let sdk_cache_result =
        super::sdk_cache::ensure_github_sdk_cache(&config, &token, &platform, &workspace).await;
    let _ = std::fs::remove_dir_all(workspace.join("sdk-cache-upload"));
    let sdk_cache = sdk_cache_result?;
    let sdk_fingerprint = sdk_cache.fingerprint.clone();
    let payload_zip = prepare_payload_zip(&workspace, &request, &build_id, &platform, sdk_cache)?;
    let tag = format!("unipack-cloud-build-{}", safe_ref_component(&build_id));
    let asset_name = format!("payload-{}.zip", safe_ref_component(&build_id));
    let artifact_name = format!("unipack-{}", build_id);
    let release = client
        .create_release(&config, &tag, &format!("UniPack cloud build {}", build_id))
        .await?;
    let cleanup = ReleaseCleanup {
        client: client.clone(),
        config: config.clone(),
        release_id: release.id,
        tag: tag.clone(),
        pending_path: pending_release_path(&request.project_id, &tag),
    };
    if let Err(error) = cleanup.persist() {
        cleanup.cleanup(&window, &build_id, &platform).await;
        return Err(error);
    }

    let result = async {
        emit_cloud_log(
            &window,
            &build_id,
            &platform,
            "info",
            "上传临时构建包到 GitHub Release",
            Some(18),
        );
        client
            .upload_release_asset(&release.upload_url, &asset_name, &payload_zip)
            .await?;
        cleanup_local_payload(&workspace);
        emit_cloud_log(
            &window,
            &build_id,
            &platform,
            "info",
            "触发 GitHub Actions workflow",
            Some(25),
        );
        client
            .dispatch_workflow(
                &config,
                &build_id,
                &platform,
                &tag,
                &asset_name,
                &artifact_name,
                &sdk_fingerprint,
                compile_sdk_version,
            )
            .await?;
        let run = client
            .wait_for_workflow_run(&config, &window, &build_id, &platform)
            .await?;
        emit_cloud_log(
            &window,
            &build_id,
            &platform,
            "info",
            &format!("GitHub Actions Run: {}", run.html_url),
            Some(70),
        );
        let cloud_run_url = run.html_url.clone();
        let conclusion = run.conclusion.unwrap_or_else(|| "unknown".to_string());
        if conclusion != "success" {
            let _ = client
                .download_run_logs(&config, run.id, &workspace.join("run-logs.zip"))
                .await
                .map(|_| {
                    emit_cloud_log(
                        &window,
                        &build_id,
                        &platform,
                        "warn",
                        "已下载 GitHub Actions 日志压缩包到云端工作区",
                        None,
                    )
                });
            return Err(format!("GitHub Actions 构建失败，状态: {}", conclusion));
        }
        emit_cloud_log(
            &window,
            &build_id,
            &platform,
            "info",
            "下载 GitHub Actions 构建产物",
            Some(88),
        );
        let artifact_zip = workspace.join("artifact.zip");
        let artifact = client
            .download_artifact(&config, run.id, &artifact_name, &artifact_zip)
            .await?;
        let mut artifact_delete_error = None;
        for attempt in 1..=3 {
            match client.delete_artifact(&config, artifact.id).await {
                Ok(()) => {
                    artifact_delete_error = None;
                    break;
                }
                Err(error) => {
                    artifact_delete_error = Some(error);
                    if attempt < 3 {
                        tokio::time::sleep(Duration::from_secs(attempt)).await;
                    }
                }
            }
        }
        if let Some(error) = artifact_delete_error {
            emit_cloud_log(
                &window,
                &build_id,
                &platform,
                "warn",
                &format!("清理 GitHub Actions artifact 失败: {}", error),
                None,
            );
        }
        let output_path =
            collect_downloaded_artifact(&request.project_id, &workspace, &artifact_zip, &platform)?;
        let size_bytes = std::fs::metadata(&output_path)
            .map(|meta| meta.len())
            .unwrap_or_default();
        emit_cloud_log(
            &window,
            &build_id,
            &platform,
            "success",
            &format!("GitHub 云端打包完成: {}", output_path.display()),
            Some(100),
        );
        Ok(crate::commands::android::types::BuildArtifact {
            platform: platform.clone(),
            path: output_path.to_string_lossy().to_string(),
            file_name: output_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&artifact.name)
                .to_string(),
            size_bytes,
            build_id: build_id.clone(),
            cloud_run_url: Some(cloud_run_url),
        })
    }
    .await;

    cleanup_local_payload(&workspace);
    cleanup.cleanup(&window, &build_id, &platform).await;
    result
}

fn config_path() -> PathBuf {
    crate::utils::fs::get_unipack_home().join("github-cloud-build.json")
}

pub(crate) fn load_config() -> Result<GithubCloudBuildConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(GithubCloudBuildConfig::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 GitHub 云端打包配置失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析 GitHub 云端打包配置失败: {}", e))
}

fn save_config_sync(config: &GithubCloudBuildConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| format!("保存 GitHub 云端打包配置失败: {}", e))
}

fn validate_config_shape(config: &GithubCloudBuildConfig) -> Result<(), String> {
    for (label, value) in [("owner", &config.owner), ("repo", &config.repo)] {
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(format!("{} 包含无效字符", label));
        }
    }
    if config.ref_name.chars().any(char::is_control)
        || config.ref_name.contains(['?', '#', '&'])
        || config.ref_name.split('/').any(|part| part == "..")
    {
        return Err("ref 包含无效字符".to_string());
    }
    if config.workflow_file.contains('/')
        || config.workflow_file.contains('\\')
        || !config
            .workflow_file
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err("workflowFile 必须是 workflow 文件名，不能包含路径".to_string());
    }
    Ok(())
}

pub(crate) fn validate_config_ready(config: &GithubCloudBuildConfig) -> Result<(), String> {
    validate_config_shape(config)?;
    if config.owner.trim().is_empty()
        || config.repo.trim().is_empty()
        || config.ref_name.trim().is_empty()
        || config.workflow_file.trim().is_empty()
    {
        return Err("请先完整配置 GitHub owner、repo、ref 和 workflow 文件名".to_string());
    }
    Ok(())
}

pub(crate) fn require_token() -> Result<String, String> {
    crate::utils::keychain::get_password(TOKEN_ACCOUNT)
        .map_err(|e| e.to_string())?
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| "请先保存 GitHub Token".to_string())
}

fn normalize_platform(platform: &str) -> Result<String, String> {
    match platform {
        "android" | "ios" => Ok(platform.to_string()),
        "harmony" => Err("HarmonyOS 暂不支持 GitHub 云端打包".to_string()),
        other => Err(format!("不支持的云端打包平台: {}", other)),
    }
}

fn validate_identifier(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 120
        || value == "."
        || value == ".."
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(format!("{} 格式无效", label));
    }
    Ok(())
}

fn cloud_workspace(project_id: &str, build_id: &str) -> Result<PathBuf, String> {
    let workspace = crate::utils::fs::get_project_config_dir(project_id)
        .join("cloud")
        .join(safe_ref_component(build_id));
    if workspace.exists() {
        std::fs::remove_dir_all(&workspace)
            .map_err(|e| format!("清理旧云端打包工作区失败: {}", e))?;
    }
    std::fs::create_dir_all(&workspace).map_err(|e| format!("创建云端打包工作区失败: {}", e))?;
    Ok(workspace)
}

fn pending_release_path(project_id: &str, tag: &str) -> PathBuf {
    crate::utils::fs::get_project_config_dir(project_id)
        .join("cloud-pending-releases")
        .join(format!("{}.json", safe_ref_component(tag)))
}

async fn cleanup_pending_releases(
    client: &GithubClient,
    config: &GithubCloudBuildConfig,
    project_id: &str,
    window: &tauri::Window,
    build_id: &str,
    platform: &str,
) -> Result<(), String> {
    let directory =
        crate::utils::fs::get_project_config_dir(project_id).join("cloud-pending-releases");
    if !directory.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(&directory)
        .map_err(|error| format!("读取待清理云构建 Release 失败: {}", error))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取待清理 Release 记录失败: {}", error))?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let pending: PendingRelease = match std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<PendingRelease>(&text).ok())
        {
            Some(value)
                if value.tag.starts_with("unipack-cloud-build-")
                    && safe_ref_component(&value.tag) == value.tag =>
            {
                value
            }
            _ => {
                let _ = std::fs::remove_file(&path);
                continue;
            }
        };
        if pending.owner.is_empty() || pending.repo.is_empty() {
            return Err(format!(
                "发现旧格式待清理 Release 记录，无法确认所属仓库: {}",
                path.display()
            ));
        }
        let mut recorded_config = config.clone();
        recorded_config.owner = pending.owner;
        recorded_config.repo = pending.repo;
        validate_config_shape(&recorded_config)?;
        let cleanup = ReleaseCleanup {
            client: client.clone(),
            config: recorded_config,
            release_id: pending.release_id,
            tag: pending.tag,
            pending_path: path,
        };
        if !cleanup.cleanup(window, build_id, platform).await {
            return Err("仍有上次中断的临时 GitHub Release 无法清理，请稍后重试".to_string());
        }
    }
    let _ = std::fs::remove_dir(&directory);
    Ok(())
}

fn prepare_payload_zip(
    workspace: &Path,
    request: &GithubCloudBuildRequest,
    build_id: &str,
    platform: &str,
    sdk_cache: SdkCacheReference,
) -> Result<PathBuf, String> {
    let payload_dir = workspace.join("payload");
    std::fs::create_dir_all(&payload_dir).map_err(|e| e.to_string())?;
    let resource_path = Path::new(&request.resource_path);
    validate_payload_resource_tree(resource_path)?;
    let resource_layout = payload_resource_layout(resource_path)?;

    let mut project_config =
        crate::commands::project::load_project_config_sync(&request.project_id)?;
    let signing = collect_signing_files(&payload_dir, &project_config, platform)?;
    let mut module_config = request.module_config.clone();
    let manifest_info =
        hydrate_payload_manifest_info(request.manifest_info.clone(), &resource_layout)?;
    let manifest_info = sanitize_payload_paths(
        &payload_dir,
        &mut project_config,
        manifest_info,
        &mut module_config,
        &signing,
        platform,
    )?;
    let resource_zip = payload_dir.join("resource.zip");
    write_sanitized_resource_zip(
        workspace,
        &resource_layout,
        manifest_info
            .as_ref()
            .and_then(|info| info.manifest_value.as_ref()),
        &resource_zip,
    )?;
    let metadata = CloudPayloadMetadata {
        version: 2,
        build_id: build_id.to_string(),
        platform: platform.to_string(),
        project_id: request.project_id.clone(),
        project_config,
        manifest_info,
        module_config,
        ios_packaging_mode: request.ios_packaging_mode.clone(),
        sdk_cache,
        signing,
    };
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("序列化云端打包 payload 失败: {}", e))?;
    std::fs::write(payload_dir.join("payload.json"), metadata_json)
        .map_err(|e| format!("写入云端打包 payload 失败: {}", e))?;

    let payload_zip = workspace.join("payload.zip");
    crate::utils::fs::zip_directory(&payload_dir, &payload_zip)
        .map_err(|e| format!("打包云端构建 payload 失败: {}", e))?;
    Ok(payload_zip)
}

#[derive(Debug)]
struct PayloadResourceLayout {
    root: PathBuf,
    app_root: PathBuf,
    manifest_paths: Vec<PathBuf>,
    manifest_target: PathBuf,
}

fn payload_resource_layout(root: &Path) -> Result<PayloadResourceLayout, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("读取导入资源目录失败 {}: {}", root.display(), error))?;
    let layout = crate::commands::shared::resource_scan::resolve_resource_layout(&root)?;
    let app_root = layout
        .app_resource_path
        .canonicalize()
        .map_err(|error| format!("读取应用资源目录失败: {}", error))?;
    let app_relative = app_root
        .strip_prefix(&root)
        .map_err(|_| "应用资源目录超出导入资源根目录".to_string())?;
    let direct = app_relative.join("manifest.json");
    let nested = app_relative.join("www/manifest.json");
    let manifest_paths = [direct.clone(), nested]
        .into_iter()
        .filter(|relative| root.join(relative).is_file())
        .collect::<Vec<_>>();
    let manifest_target = manifest_paths.first().cloned().unwrap_or(direct);
    Ok(PayloadResourceLayout {
        root,
        app_root,
        manifest_paths,
        manifest_target,
    })
}

fn hydrate_payload_manifest_info(
    mut supplied: Option<crate::commands::resource::UniappManifestInfo>,
    layout: &PayloadResourceLayout,
) -> Result<Option<crate::commands::resource::UniappManifestInfo>, String> {
    let source_manifest = layout
        .manifest_paths
        .first()
        .map(|relative| layout.root.join(relative));
    let source_value = source_manifest
        .as_deref()
        .map(crate::commands::shared::resource::read_manifest_file)
        .transpose()?;

    if let Some(info) = supplied.as_mut() {
        if info.manifest_value.is_none() {
            info.manifest_value = source_value;
        }
        return Ok(supplied);
    }
    let (Some(path), Some(value)) = (source_manifest, source_value) else {
        return Ok(None);
    };
    Ok(Some(
        crate::commands::shared::resource::parse_uniapp_manifest(
            &value,
            &path,
            &layout.app_root,
            None,
        ),
    ))
}

fn write_sanitized_resource_zip(
    workspace: &Path,
    layout: &PayloadResourceLayout,
    manifest: Option<&serde_json::Value>,
    destination: &Path,
) -> Result<(), String> {
    if manifest.is_none() && !layout.manifest_paths.is_empty() {
        return Err("云构建无法生成脱敏后的 manifest.json，已拒绝上传原文件".to_string());
    }
    let staging = workspace.join("resource-payload");
    if staging.exists() {
        std::fs::remove_dir_all(&staging)
            .map_err(|error| format!("清理云构建资源副本失败: {}", error))?;
    }
    let result = (|| {
        copy_payload_resource_tree(&layout.root, &layout.root, &staging, &layout.manifest_paths)?;
        if let Some(value) = manifest {
            let targets = if layout.manifest_paths.is_empty() {
                std::slice::from_ref(&layout.manifest_target)
            } else {
                layout.manifest_paths.as_slice()
            };
            let bytes = serde_json::to_vec_pretty(value)
                .map_err(|error| format!("序列化脱敏 manifest.json 失败: {}", error))?;
            for relative in targets {
                let path = staging.join(relative);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|error| format!("创建脱敏 manifest 目录失败: {}", error))?;
                }
                // The original manifest is deliberately not hard-linked. Creating a
                // fresh file guarantees that writing the sanitized value cannot alter
                // the imported resource snapshot.
                std::fs::write(&path, &bytes)
                    .map_err(|error| format!("写入脱敏 manifest.json 失败: {}", error))?;
            }
        }
        crate::utils::fs::zip_directory(&staging, destination)
            .map_err(|error| format!("压缩导入资源失败: {}", error))
    })();
    let _ = std::fs::remove_dir_all(&staging);
    result
}

fn copy_payload_resource_tree(
    root: &Path,
    source: &Path,
    destination: &Path,
    skipped_manifests: &[PathBuf],
) -> Result<(), String> {
    std::fs::create_dir_all(destination)
        .map_err(|error| format!("创建云构建资源副本失败: {}", error))?;
    let mut entries = std::fs::read_dir(source)
        .map_err(|error| format!("读取导入资源失败 {}: {}", source.display(), error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("读取导入资源目录项失败: {}", error))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let source_path = entry.path();
        let relative = source_path
            .strip_prefix(root)
            .map_err(|_| "导入资源文件超出根目录".to_string())?;
        let metadata = std::fs::symlink_metadata(&source_path)
            .map_err(|error| format!("读取导入资源信息失败: {}", error))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "导入资源包含符号链接，云端打包为防止路径逃逸已拒绝: {}",
                source_path.display()
            ));
        }
        if metadata.is_dir() {
            copy_payload_resource_tree(
                root,
                &source_path,
                &destination.join(entry.file_name()),
                skipped_manifests,
            )?;
        } else if metadata.is_file() {
            if skipped_manifests.iter().any(|path| path == relative) {
                continue;
            }
            let destination_path = destination.join(entry.file_name());
            if std::fs::hard_link(&source_path, &destination_path).is_err() {
                std::fs::copy(&source_path, &destination_path).map_err(|error| {
                    format!(
                        "复制云构建资源失败 {} -> {}: {}",
                        source_path.display(),
                        destination_path.display(),
                        error
                    )
                })?;
            }
        } else {
            return Err(format!(
                "导入资源包含不支持的特殊文件: {}",
                source_path.display()
            ));
        }
    }
    Ok(())
}

fn cleanup_local_payload(workspace: &Path) {
    let _ = std::fs::remove_file(workspace.join("payload.zip"));
    let _ = std::fs::remove_dir_all(workspace.join("payload"));
    let _ = std::fs::remove_dir_all(workspace.join("sdk-cache-upload"));
}

struct CloudWorkspaceCleanup(PathBuf);

impl Drop for CloudWorkspaceCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn validate_payload_resource_tree(root: &Path) -> Result<(), String> {
    let root_metadata = std::fs::symlink_metadata(root)
        .map_err(|error| format!("读取导入资源信息失败 {}: {}", root.display(), error))?;
    if root_metadata.file_type().is_symlink() {
        return Err(format!(
            "导入资源根目录是符号链接，云端打包为防止路径逃逸已拒绝: {}",
            root.display()
        ));
    }
    if !root_metadata.is_dir() {
        return Err(format!("导入资源目录不存在: {}", root.display()));
    }
    fn visit(directory: &Path) -> Result<(), String> {
        let entries = std::fs::read_dir(directory)
            .map_err(|error| format!("读取导入资源失败 {}: {}", directory.display(), error))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("读取导入资源失败: {}", error))?;
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .map_err(|error| format!("读取导入资源信息失败 {}: {}", path.display(), error))?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "导入资源包含符号链接，云端打包为防止路径逃逸已拒绝: {}",
                    path.display()
                ));
            }
            if metadata.is_dir() {
                visit(&path)?;
            } else if !metadata.is_file() {
                return Err(format!("导入资源包含不支持的特殊文件: {}", path.display()));
            }
        }
        Ok(())
    }
    visit(root)
}

fn collect_signing_files(
    payload_dir: &Path,
    config: &crate::commands::project::ProjectConfig,
    platform: &str,
) -> Result<CloudPayloadSigning, String> {
    let signing_dir = payload_dir.join("signing");
    std::fs::create_dir_all(&signing_dir).map_err(|e| e.to_string())?;
    let mut signing = CloudPayloadSigning::default();
    if platform == "android" {
        if let Some(file_name) = copy_optional_file(
            &config.android.keystore.path,
            &signing_dir,
            "android-keystore",
        )? {
            signing.android_keystore_file = Some(file_name);
        }
        signing.android_store_password =
            crate::utils::keychain::get_password(&format!("{}-android-store-password", config.id))
                .map_err(|e| e.to_string())?;
        signing.android_key_password =
            crate::utils::keychain::get_password(&format!("{}-android-key-password", config.id))
                .map_err(|e| e.to_string())?;
    } else if platform == "ios" {
        if let Some(file_name) =
            copy_optional_file(&config.ios.certificate, &signing_dir, "ios-certificate")?
        {
            signing.ios_certificate_file = Some(file_name);
        }
        if let Some(file_name) = copy_optional_file(
            &config.ios.provisioning_profile,
            &signing_dir,
            "ios-mobileprovision",
        )? {
            signing.ios_provisioning_profile_file = Some(file_name);
        }
        signing.ios_certificate_password = crate::utils::keychain::get_password(&format!(
            "{}-ios-certificate-password",
            config.id
        ))
        .map_err(|e| e.to_string())?;
    }
    Ok(signing)
}

fn copy_optional_file(path: &str, dst_dir: &Path, prefix: &str) -> Result<Option<String>, String> {
    let path = path.trim();
    if path.is_empty() {
        return Ok(None);
    }
    let src = PathBuf::from(path);
    if !src.exists() || !src.is_file() {
        return Ok(None);
    }
    let ext = src
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default();
    let file_name = format!("{}{}", prefix, ext);
    std::fs::copy(&src, dst_dir.join(&file_name))
        .map_err(|e| format!("复制签名文件失败 {}: {}", src.display(), e))?;
    Ok(Some(format!("signing/{}", file_name)))
}

fn sanitize_payload_paths(
    payload_dir: &Path,
    project: &mut crate::commands::project::ProjectConfig,
    mut manifest: Option<crate::commands::resource::UniappManifestInfo>,
    module_config: &mut Option<HashMap<String, String>>,
    signing: &CloudPayloadSigning,
    platform: &str,
) -> Result<Option<crate::commands::resource::UniappManifestInfo>, String> {
    let mut assets = PayloadAssetCopier::new(payload_dir.join("manifest-assets"));
    project.local_path.clear();
    project.output_dir = "cloud-output".to_string();
    project.android.keystore.path = signing.android_keystore_file.clone().unwrap_or_default();
    project.ios.certificate = signing.ios_certificate_file.clone().unwrap_or_default();
    project.ios.provisioning_profile = signing
        .ios_provisioning_profile_file
        .clone()
        .unwrap_or_default();
    project.harmony.signing_config.store_file.clear();
    project.harmony.signing_config.key_alias.clear();
    project.harmony.signing_config.has_store_password = false;
    project.harmony.signing_config.has_key_password = false;
    project.harmony.enabled = false;
    project.harmony.bundle_name.clear();
    project.harmony.runtime_version.clear();
    project.app.icon1024 = assets.copy_reference(&project.app.icon1024, "app-icon")?;
    if platform == "android" {
        project.ios.enabled = false;
        project.ios.dcloud_app_key.clear();
        project.ios.bundle_id.clear();
        project.ios.team_id.clear();
        project.ios.export_method.clear();
        project.ios.has_certificate_password = false;
        project.ios_module_config.clear();
        sanitize_config_map_paths(
            &mut project.android_module_config,
            &mut assets,
            "android-module",
        )?;
        if let Some(config) = module_config.as_mut() {
            sanitize_config_map_paths(config, &mut assets, "build-module")?;
        }
    } else {
        project.android.enabled = false;
        project.android.dcloud_app_key.clear();
        project.android.package_name.clear();
        project.android.keystore.alias.clear();
        project.android.keystore.has_store_password = false;
        project.android.keystore.has_key_password = false;
        project.android_module_config.clear();
        sanitize_config_map_paths(&mut project.ios_module_config, &mut assets, "ios-module")?;
        *module_config = None;
    }

    if let Some(info) = manifest.as_mut() {
        prune_manifest_for_platform(info, platform);
        // Never copy the original local manifest into payload assets. It may still
        // contain another platform's credentials or absolute machine paths.
        info.manifest_path.clear();
        info.project_root.clear();
        info.warnings.clear();
        if platform == "android" {
            info.ios_icons = None;
            info.ios_privacy_descriptions.clear();
            if let Some(config) = info.android_icons.as_mut() {
                for (density, path) in config.android.iter_mut() {
                    *path = assets.copy_reference(path, &format!("android-icon-{}", density))?;
                }
            }
            if let Some(config) = info.push_icons.as_mut() {
                if let Some(path) = config.small.as_mut() {
                    *path = assets.copy_reference(path, "push-icon")?;
                }
                for (density, path) in config.small_densities.iter_mut() {
                    *path = assets.copy_reference(path, &format!("push-icon-{}", density))?;
                }
            }
            if let Some(config) = info.splashscreen.as_mut() {
                config.ios_style = None;
                config.ios_storyboard = None;
                for (density, path) in config.android.iter_mut() {
                    *path = assets.copy_reference(path, &format!("splash-{}", density))?;
                }
            }
        } else {
            info.android_icons = None;
            info.push_icons = None;
            if let Some(config) = info.ios_icons.as_mut() {
                for (slot, path) in config.ios.iter_mut() {
                    *path = assets.copy_reference(path, &format!("ios-icon-{}", slot))?;
                }
            }
            if let Some(config) = info.splashscreen.as_mut() {
                config.android_style = None;
                config.android.clear();
                if let Some(path) = config.ios_storyboard.as_mut() {
                    *path = assets.copy_reference(path, "ios-storyboard")?;
                }
            }
        }
        if let Some(value) = info.manifest_value.as_mut() {
            sanitize_json_absolute_paths(value, &mut assets)?;
            std::fs::create_dir_all(&assets.directory)
                .map_err(|error| format!("创建脱敏 manifest 目录失败: {}", error))?;
            let manifest_asset = assets.directory.join("platform-manifest.json");
            std::fs::write(
                &manifest_asset,
                serde_json::to_vec_pretty(value)
                    .map_err(|error| format!("序列化脱敏 manifest 失败: {}", error))?,
            )
            .map_err(|error| format!("写入脱敏 manifest 失败: {}", error))?;
            info.manifest_path = "manifest-assets/platform-manifest.json".to_string();
        }
    }
    Ok(manifest)
}

fn prune_manifest_for_platform(
    info: &mut crate::commands::resource::UniappManifestInfo,
    platform: &str,
) {
    info.detected_modules.retain(|module| {
        module.platforms.is_empty()
            || module
                .platforms
                .iter()
                .any(|value| value == platform || value == "all")
    });
    if platform == "android" {
        info.package_names.ios_bundle_id = None;
        info.package_names.harmony_bundle = None;
    } else {
        info.package_names.android_package = None;
        info.package_names.harmony_bundle = None;
        info.android.package_name = None;
        info.android.min_sdk_version = None;
        info.android.target_sdk_version = None;
        info.android.compile_sdk_version = None;
        info.android.permissions.clear();
        info.android.exclude_permissions.clear();
        info.android.schemes.clear();
        info.android.abi_filters.clear();
    }
    if let Some(value) = info.manifest_value.as_mut() {
        prune_manifest_json_platforms(value, platform);
    }
}

fn prune_manifest_json_platforms(value: &mut serde_json::Value, platform: &str) {
    match value {
        serde_json::Value::Object(map) => {
            let excluded = if platform == "android" {
                ["ios", "harmony", "app-harmony"]
            } else {
                ["android", "harmony", "app-harmony"]
            };
            for key in excluded {
                map.remove(key);
            }
            for child in map.values_mut() {
                prune_manifest_json_platforms(child, platform);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                prune_manifest_json_platforms(child, platform);
            }
        }
        _ => {}
    }
}

fn sanitize_config_map_paths(
    config: &mut HashMap<String, String>,
    assets: &mut PayloadAssetCopier,
    label: &str,
) -> Result<(), String> {
    for (key, value) in config.iter_mut() {
        if Path::new(value).is_absolute() {
            *value = assets.copy_reference(value, &format!("{}-{}", label, key))?;
        }
    }
    Ok(())
}

struct PayloadAssetCopier {
    directory: PathBuf,
    copied: HashMap<PathBuf, String>,
    next_index: usize,
}

impl PayloadAssetCopier {
    fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            copied: HashMap::new(),
            next_index: 0,
        }
    }

    fn copy_reference(&mut self, value: &str, label: &str) -> Result<String, String> {
        let value = value.trim();
        if value.is_empty() || value.contains("://") || value.starts_with("data:") {
            return Ok(value.to_string());
        }
        let source = PathBuf::from(value);
        if !source.is_absolute() {
            // Relative manifest values are already portable and refer to resource.zip.
            return Ok(value.replace('\\', "/"));
        }
        if !source.is_file() {
            return Err(format!("云端打包引用的资源不存在: {}", source.display()));
        }
        let canonical = source
            .canonicalize()
            .map_err(|error| format!("读取云端打包资源失败 {}: {}", source.display(), error))?;
        if let Some(existing) = self.copied.get(&canonical) {
            return Ok(existing.clone());
        }
        std::fs::create_dir_all(&self.directory)
            .map_err(|error| format!("创建云端资源目录失败: {}", error))?;
        let safe_label = safe_ref_component(label);
        let extension = canonical
            .extension()
            .and_then(|value| value.to_str())
            .filter(|value| {
                value.len() <= 12 && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
            })
            .map(|value| format!(".{}", value.to_ascii_lowercase()))
            .unwrap_or_default();
        let file_name = format!(
            "{:04}-{}{}",
            self.next_index,
            if safe_label.is_empty() {
                "asset"
            } else {
                &safe_label
            },
            extension
        );
        self.next_index += 1;
        std::fs::copy(&canonical, self.directory.join(&file_name))
            .map_err(|error| format!("复制云端打包资源失败 {}: {}", canonical.display(), error))?;
        let reference = format!("manifest-assets/{}", file_name);
        self.copied.insert(canonical, reference.clone());
        Ok(reference)
    }
}

fn sanitize_json_absolute_paths(
    value: &mut serde_json::Value,
    assets: &mut PayloadAssetCopier,
) -> Result<(), String> {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                sanitize_json_absolute_paths(value, assets)?;
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                sanitize_json_absolute_paths(value, assets)?;
            }
        }
        serde_json::Value::String(text) if Path::new(text).is_absolute() => {
            if Path::new(text).is_file() {
                *text = assets.copy_reference(text, "manifest-asset")?;
            } else {
                // Never serialize a local absolute path into the cloud payload.
                text.clear();
            }
        }
        _ => {}
    }
    Ok(())
}

fn collect_downloaded_artifact(
    project_id: &str,
    workspace: &Path,
    artifact_zip: &Path,
    platform: &str,
) -> Result<PathBuf, String> {
    let extracted = workspace.join("artifact");
    crate::utils::fs::unzip_file(artifact_zip, &extracted)
        .map_err(|e| format!("解压云端打包产物失败: {}", e))?;
    let expected_exts: &[&str] = if platform == "ios" {
        &["ipa"]
    } else {
        &["apk", "aab"]
    };
    let artifact = find_first_with_extensions(&extracted, expected_exts)
        .ok_or_else(|| format!("云端构建产物中未找到 {}", expected_exts.join(" 或 ")))?;
    let project = crate::commands::project::load_project_config_sync(project_id)?;
    let output_dir = crate::utils::fs::expand_home(&project.output_dir);
    crate::utils::fs::ensure_directory(&output_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;
    let dest_name = artifact
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(if platform == "ios" {
            "app.ipa"
        } else {
            "app.apk"
        });
    let dest = output_dir.join(dest_name);
    std::fs::copy(&artifact, &dest)
        .map_err(|e| format!("复制云端构建产物失败 {}: {}", artifact.display(), e))?;
    Ok(dest)
}

fn find_first_with_extensions(root: &Path, exts: &[&str]) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_first_with_extensions(&path, exts) {
                return Some(found);
            }
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                exts.iter()
                    .any(|candidate| ext.eq_ignore_ascii_case(candidate))
            })
            .unwrap_or(false)
        {
            return Some(path);
        }
    }
    None
}

fn safe_ref_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn emit_cloud_log(
    window: &tauri::Window,
    build_id: &str,
    platform: &str,
    level: &str,
    message: &str,
    progress: Option<u8>,
) {
    let _ = window.emit(
        "build-log",
        crate::commands::android::types::BuildLogEvent {
            build_id: Some(build_id.to_string()),
            platform: platform.to_string(),
            level: level.to_string(),
            message: message.to_string(),
            progress,
        },
    );
}

#[derive(Clone)]
struct GithubClient {
    client: reqwest::Client,
    token: String,
}

impl GithubClient {
    fn new(token: String) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .user_agent("unipack-tool")
            .build()
            .map_err(|e| format!("创建 GitHub HTTP Client 失败: {}", e))?;
        Ok(Self { client, token })
    }

    async fn get_repo(&self, config: &GithubCloudBuildConfig) -> Result<GithubRepo, String> {
        self.get_json(&format!(
            "{}/repos/{}/{}",
            GITHUB_API, config.owner, config.repo
        ))
        .await
    }

    async fn get_workflow(
        &self,
        config: &GithubCloudBuildConfig,
    ) -> Result<GithubWorkflow, String> {
        self.get_json(&format!(
            "{}/repos/{}/{}/actions/workflows/{}",
            GITHUB_API, config.owner, config.repo, config.workflow_file
        ))
        .await
    }

    async fn create_release(
        &self,
        config: &GithubCloudBuildConfig,
        tag: &str,
        name: &str,
    ) -> Result<GithubRelease, String> {
        let url = format!(
            "{}/repos/{}/{}/releases",
            GITHUB_API, config.owner, config.repo
        );
        self.request(reqwest::Method::POST, &url)
            .json(&serde_json::json!({
                "tag_name": tag,
                "name": name,
                "draft": false,
                "prerelease": true
            }))
            .send()
            .await
            .map_err(|e| format!("创建 GitHub Release 失败: {}", e))?
            .error_for_status()
            .map_err(|e| format!("创建 GitHub Release 失败: {}", e))?
            .json::<GithubRelease>()
            .await
            .map_err(|e| format!("解析 GitHub Release 响应失败: {}", e))
    }

    async fn get_release_by_tag(
        &self,
        config: &GithubCloudBuildConfig,
        tag: &str,
    ) -> Result<Option<GithubRelease>, String> {
        let response = self
            .request(
                reqwest::Method::GET,
                &format!(
                    "{}/repos/{}/{}/releases/tags/{}",
                    GITHUB_API, config.owner, config.repo, tag
                ),
            )
            .send()
            .await
            .map_err(|error| format!("查询待清理 GitHub Release 失败: {}", error))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        response
            .error_for_status()
            .map_err(|error| format!("查询待清理 GitHub Release 失败: {}", error))?
            .json::<GithubRelease>()
            .await
            .map(Some)
            .map_err(|error| format!("解析待清理 GitHub Release 失败: {}", error))
    }

    async fn upload_release_asset(
        &self,
        upload_url: &str,
        name: &str,
        path: &Path,
    ) -> Result<(), String> {
        let upload_url = upload_url
            .split_once('{')
            .map(|(base, _)| base)
            .unwrap_or(upload_url);
        let url = format!("{}?name={}", upload_url, name);
        let size = std::fs::metadata(path)
            .map_err(|e| format!("读取 payload 失败: {}", e))?
            .len();
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|e| format!("打开 payload 失败: {}", e))?;
        self.request(reqwest::Method::POST, &url)
            .header(reqwest::header::CONTENT_TYPE, "application/zip")
            .header(reqwest::header::CONTENT_LENGTH, size)
            .body(reqwest::Body::wrap_stream(ReaderStream::new(file)))
            .send()
            .await
            .map_err(|e| format!("上传 GitHub Release Asset 失败: {}", e))?
            .error_for_status()
            .map_err(|e| format!("上传 GitHub Release Asset 失败: {}", e))?;
        Ok(())
    }

    async fn dispatch_workflow(
        &self,
        config: &GithubCloudBuildConfig,
        build_id: &str,
        platform: &str,
        tag: &str,
        asset_name: &str,
        artifact_name: &str,
        sdk_fingerprint: &str,
        compile_sdk_version: u32,
    ) -> Result<(), String> {
        let url = format!(
            "{}/repos/{}/{}/actions/workflows/{}/dispatches",
            GITHUB_API, config.owner, config.repo, config.workflow_file
        );
        self.request(reqwest::Method::POST, &url)
            .json(&serde_json::json!({
                "ref": config.ref_name,
                "inputs": {
                    "build_id": build_id,
                    "platform": platform,
                    "payload_release_tag": tag,
                    "payload_asset_name": asset_name,
                    "artifact_name": artifact_name,
                    "sdk_fingerprint": sdk_fingerprint,
                    "compile_sdk_version": compile_sdk_version.to_string()
                }
            }))
            .send()
            .await
            .map_err(|e| format!("触发 GitHub Actions 失败: {}", e))?
            .error_for_status()
            .map_err(|e| format!("触发 GitHub Actions 失败: {}", e))?;
        Ok(())
    }

    async fn wait_for_workflow_run(
        &self,
        config: &GithubCloudBuildConfig,
        window: &tauri::Window,
        build_id: &str,
        platform: &str,
    ) -> Result<WorkflowRun, String> {
        let started = Instant::now();
        let mut last_status = String::new();
        loop {
            if let Some(run) = self.find_workflow_run(config, build_id).await? {
                let status = run.status.clone().unwrap_or_else(|| "unknown".to_string());
                if status != last_status {
                    last_status = status.clone();
                    emit_cloud_log(
                        window,
                        build_id,
                        platform,
                        "info",
                        &format!("GitHub Actions 状态: {}", status),
                        Some(if status == "completed" { 70 } else { 45 }),
                    );
                }
                if status == "completed" {
                    return Ok(run);
                }
            }
            if started.elapsed() > Duration::from_secs(180 * 60) {
                return Err("等待 GitHub Actions 超时".to_string());
            }
            tokio::time::sleep(Duration::from_secs(8)).await;
        }
    }

    async fn find_workflow_run(
        &self,
        config: &GithubCloudBuildConfig,
        build_id: &str,
    ) -> Result<Option<WorkflowRun>, String> {
        for page in 1..=10 {
            let runs = self.list_runs_page(config, page).await?.workflow_runs;
            let count = runs.len();
            if let Some(run) = runs
                .into_iter()
                .find(|run| run.display_title.as_deref() == Some(build_id))
            {
                return Ok(Some(run));
            }
            if count < 100 {
                break;
            }
        }
        Ok(None)
    }

    async fn list_runs_page(
        &self,
        config: &GithubCloudBuildConfig,
        page: u32,
    ) -> Result<WorkflowRuns, String> {
        let mut url = reqwest::Url::parse(&format!(
            "{}/repos/{}/{}/actions/workflows/{}/runs",
            GITHUB_API, config.owner, config.repo, config.workflow_file
        ))
        .map_err(|e| format!("创建 GitHub Actions 查询地址失败: {}", e))?;
        url.query_pairs_mut()
            .append_pair("event", "workflow_dispatch")
            .append_pair("branch", &config.ref_name)
            .append_pair("per_page", "100")
            .append_pair("page", &page.to_string());
        self.get_json(url.as_str()).await
    }

    async fn download_artifact(
        &self,
        config: &GithubCloudBuildConfig,
        run_id: u64,
        artifact_name: &str,
        output: &Path,
    ) -> Result<WorkflowArtifact, String> {
        let artifacts: WorkflowArtifacts = self
            .get_json(&format!(
                "{}/repos/{}/{}/actions/runs/{}/artifacts",
                GITHUB_API, config.owner, config.repo, run_id
            ))
            .await?;
        let artifact = artifacts
            .artifacts
            .into_iter()
            .find(|artifact| !artifact.expired && artifact.name == artifact_name)
            .ok_or_else(|| format!("未找到 GitHub Actions artifact: {}", artifact_name))?;
        self.download_to(
            &format!(
                "{}/repos/{}/{}/actions/artifacts/{}/zip",
                GITHUB_API, config.owner, config.repo, artifact.id
            ),
            output,
        )
        .await?;
        Ok(artifact)
    }

    async fn download_run_logs(
        &self,
        config: &GithubCloudBuildConfig,
        run_id: u64,
        output: &Path,
    ) -> Result<(), String> {
        self.download_to(
            &format!(
                "{}/repos/{}/{}/actions/runs/{}/logs",
                GITHUB_API, config.owner, config.repo, run_id
            ),
            output,
        )
        .await
    }

    async fn delete_artifact(
        &self,
        config: &GithubCloudBuildConfig,
        artifact_id: u64,
    ) -> Result<(), String> {
        let response = self
            .request(
                reqwest::Method::DELETE,
                &format!(
                    "{}/repos/{}/{}/actions/artifacts/{}",
                    GITHUB_API, config.owner, config.repo, artifact_id
                ),
            )
            .send()
            .await
            .map_err(|e| format!("删除 GitHub Actions artifact 失败: {}", e))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        response
            .error_for_status()
            .map_err(|e| format!("删除 GitHub Actions artifact 失败: {}", e))?;
        Ok(())
    }

    async fn delete_release(
        &self,
        config: &GithubCloudBuildConfig,
        release_id: u64,
    ) -> Result<(), String> {
        let response = self
            .request(
                reqwest::Method::DELETE,
                &format!(
                    "{}/repos/{}/{}/releases/{}",
                    GITHUB_API, config.owner, config.repo, release_id
                ),
            )
            .send()
            .await
            .map_err(|e| format!("删除 GitHub Release 失败: {}", e))?;
        if response.status() != reqwest::StatusCode::NOT_FOUND {
            response
                .error_for_status()
                .map_err(|e| format!("删除 GitHub Release 失败: {}", e))?;
        }
        Ok(())
    }

    async fn delete_tag(&self, config: &GithubCloudBuildConfig, tag: &str) -> Result<(), String> {
        let response = self
            .request(
                reqwest::Method::DELETE,
                &format!(
                    "{}/repos/{}/{}/git/refs/tags/{}",
                    GITHUB_API, config.owner, config.repo, tag
                ),
            )
            .send()
            .await
            .map_err(|e| format!("删除 GitHub Tag 失败: {}", e))?;
        if response.status() != reqwest::StatusCode::NOT_FOUND {
            response
                .error_for_status()
                .map_err(|e| format!("删除 GitHub Tag 失败: {}", e))?;
        }
        Ok(())
    }

    async fn download_to(&self, url: &str, output: &Path) -> Result<(), String> {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = self
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .map_err(|e| format!("下载 GitHub 文件失败: {}", e))?
            .error_for_status()
            .map_err(|e| format!("下载 GitHub 文件失败: {}", e))?
            .bytes()
            .await
            .map_err(|e| format!("读取 GitHub 下载内容失败: {}", e))?;
        std::fs::File::create(output)
            .and_then(|mut file| file.write_all(&bytes))
            .map_err(|e| format!("写入 GitHub 下载内容失败: {}", e))
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, String> {
        self.request(reqwest::Method::GET, url)
            .send()
            .await
            .map_err(|e| format!("请求 GitHub API 失败: {}", e))?
            .error_for_status()
            .map_err(|e| format!("请求 GitHub API 失败: {}", e))?
            .json::<T>()
            .await
            .map_err(|e| format!("解析 GitHub API 响应失败: {}", e))
    }

    fn request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let url = if let Some(rest) = url.strip_prefix(GITHUB_UPLOADS) {
            format!("{}{}", GITHUB_UPLOADS, rest)
        } else {
            url.to_string()
        };
        self.client
            .request(method, url)
            .bearer_auth(&self.token)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }
}

struct ReleaseCleanup {
    client: GithubClient,
    config: GithubCloudBuildConfig,
    release_id: u64,
    tag: String,
    pending_path: PathBuf,
}

impl ReleaseCleanup {
    fn persist(&self) -> Result<(), String> {
        if let Some(parent) = self.pending_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("创建待清理 Release 目录失败: {}", error))?;
        }
        let value = PendingRelease {
            release_id: self.release_id,
            tag: self.tag.clone(),
            owner: self.config.owner.clone(),
            repo: self.config.repo.clone(),
        };
        std::fs::write(
            &self.pending_path,
            serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?,
        )
        .map_err(|error| format!("记录待清理 GitHub Release 失败: {}", error))
    }

    async fn cleanup(&self, window: &tauri::Window, build_id: &str, platform: &str) -> bool {
        let mut cleaned = true;
        let release_identity_safe = match self
            .client
            .get_release_by_tag(&self.config, &self.tag)
            .await
        {
            Ok(Some(release)) if release.id == self.release_id => {
                if let Err(error) = self
                    .client
                    .delete_release(&self.config, self.release_id)
                    .await
                {
                    cleaned = false;
                    emit_cloud_log(
                        window,
                        build_id,
                        platform,
                        "warn",
                        &format!("清理 GitHub Release 失败: {}", error),
                        None,
                    );
                    false
                } else {
                    true
                }
            }
            Ok(Some(release)) => {
                cleaned = false;
                emit_cloud_log(
                    window,
                    build_id,
                    platform,
                    "warn",
                    &format!(
                        "待清理 Release 标识已变化，拒绝删除（记录 {}，当前 {}）",
                        self.release_id, release.id
                    ),
                    None,
                );
                false
            }
            Ok(None) => true,
            Err(error) => {
                cleaned = false;
                emit_cloud_log(window, build_id, platform, "warn", &error, None);
                false
            }
        };
        if release_identity_safe {
            if let Err(error) = self.client.delete_tag(&self.config, &self.tag).await {
                cleaned = false;
                emit_cloud_log(
                    window,
                    build_id,
                    platform,
                    "warn",
                    &format!("清理 GitHub Tag 失败: {}", error),
                    None,
                );
            }
        }
        if cleaned {
            let _ = std::fs::remove_file(&self.pending_path);
        }
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_only_contains_repository_fields() {
        let config = GithubCloudBuildConfig::default();
        assert_eq!(config.workflow_file, "cloud-build.yml");
        let value = serde_json::to_value(config).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 4);
        assert!(value.get("androidDefaultMode").is_none());
        assert!(value.get("androidSdkUrl").is_none());
    }

    #[test]
    fn legacy_config_fields_are_ignored_and_removed_when_serialized() {
        let config: GithubCloudBuildConfig = serde_json::from_value(serde_json::json!({
            "owner": "owner",
            "repo": "repo",
            "ref": "main",
            "workflowFile": "cloud-build.yml",
            "androidDefaultMode": "auto",
            "iosDefaultMode": "github",
            "androidSdkUrl": "https://example.invalid/android.zip",
            "iosSdkUrl": "https://example.invalid/ios.zip"
        }))
        .unwrap();
        let serialized = serde_json::to_value(config).unwrap();
        assert_eq!(serialized.as_object().unwrap().len(), 4);
        assert!(serialized.get("iosDefaultMode").is_none());
        assert!(serialized.get("iosSdkUrl").is_none());
    }

    #[test]
    fn safe_ref_component_removes_unsafe_chars() {
        assert_eq!(safe_ref_component("build/a:b c"), "build-a-b-c");
    }

    #[test]
    fn cloud_identifiers_reject_path_components_and_unsafe_characters() {
        assert!(validate_identifier("build_id", "build-123").is_ok());
        assert!(validate_identifier("build_id", "../build").is_err());
        assert!(validate_identifier("project_id", "project/name").is_err());
        assert!(validate_identifier("project_id", "..").is_err());
    }

    #[test]
    fn pending_release_journal_binds_cleanup_to_original_repository() {
        let root =
            std::env::temp_dir().join(format!("unipack-pending-release-{}", uuid::Uuid::new_v4()));
        let pending_path = root.join("pending.json");
        let cleanup = ReleaseCleanup {
            client: GithubClient::new("test-token".to_string()).unwrap(),
            config: GithubCloudBuildConfig {
                owner: "original-owner".to_string(),
                repo: "original-repo".to_string(),
                ..GithubCloudBuildConfig::default()
            },
            release_id: 42,
            tag: "unipack-cloud-build-build-42".to_string(),
            pending_path: pending_path.clone(),
        };
        cleanup.persist().unwrap();
        let pending: PendingRelease =
            serde_json::from_str(&std::fs::read_to_string(&pending_path).unwrap()).unwrap();
        assert_eq!(pending.owner, "original-owner");
        assert_eq!(pending.repo, "original-repo");
        assert_eq!(pending.release_id, 42);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn cloud_resource_validation_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let root =
            std::env::temp_dir().join(format!("unipack-cloud-resource-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        symlink("../../outside", root.join("escape")).unwrap();
        let error = validate_payload_resource_tree(&root).unwrap_err();
        assert!(error.contains("符号链接"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn payload_asset_reference_never_contains_source_path() {
        let root =
            std::env::temp_dir().join(format!("unipack-cloud-assets-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("private/icon.png");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, b"icon").unwrap();
        let mut copier = PayloadAssetCopier::new(root.join("payload/manifest-assets"));
        let reference = copier
            .copy_reference(source.to_str().unwrap(), "app-icon")
            .unwrap();
        assert!(reference.starts_with("manifest-assets/"));
        assert!(!reference.contains(root.to_str().unwrap()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn payload_sanitization_only_copies_assets_for_current_platform() {
        let root =
            std::env::temp_dir().join(format!("unipack-platform-assets-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut ios_icons = crate::commands::shared::resource::IosIconsConfig::default();
        ios_icons.ios.insert(
            "iphone.app@3x".to_string(),
            "/missing/ios-only-icon.png".to_string(),
        );
        let mut splash = crate::commands::shared::resource::SplashscreenConfig::default();
        splash.ios_storyboard = Some("/missing/ios-only.storyboard".to_string());
        let manifest = crate::commands::resource::UniappManifestInfo {
            app_name: None,
            app_id: None,
            version_name: None,
            version_code: None,
            hbuilderx_version: None,
            android_icons: None,
            ios_icons: Some(ios_icons),
            push_icons: None,
            splashscreen: Some(splash),
            ios_privacy_descriptions: Default::default(),
            manifest_value: Some(serde_json::json!({
                "app-plus": {
                    "distribute": {
                        "android": { "packageName": "com.example.android" },
                        "ios": { "secretProviderKey": "must-not-upload" },
                        "harmony": { "bundleName": "must-not-upload" }
                    }
                },
                "app-harmony": { "secretProviderKey": "must-not-upload" }
            })),
            manifest_path: "manifest.json".to_string(),
            project_root: "/private/local/project".to_string(),
            android: crate::commands::shared::resource::AndroidManifestConfig {
                package_name: None,
                min_sdk_version: None,
                target_sdk_version: None,
                compile_sdk_version: None,
                permissions: Vec::new(),
                exclude_permissions: Vec::new(),
                schemes: Vec::new(),
                abi_filters: Vec::new(),
            },
            package_names: crate::commands::shared::resource::PlatformPackages {
                android_package: None,
                ios_bundle_id: Some("com.example.ios".to_string()),
                harmony_bundle: Some("com.example.harmony".to_string()),
            },
            detected_modules: Vec::new(),
            warnings: vec!["local warning".to_string()],
        };
        let mut project = crate::commands::project::ProjectConfig::default();
        project.ios_module_config.insert(
            "ios.file".to_string(),
            "/missing/ios-only.plist".to_string(),
        );
        let mut module_config = Some(HashMap::new());
        let sanitized = sanitize_payload_paths(
            &root,
            &mut project,
            Some(manifest),
            &mut module_config,
            &CloudPayloadSigning::default(),
            "android",
        )
        .unwrap()
        .unwrap();
        assert!(sanitized.ios_icons.is_none());
        assert!(sanitized
            .splashscreen
            .as_ref()
            .unwrap()
            .ios_storyboard
            .is_none());
        assert!(project.ios_module_config.is_empty());
        assert!(!project.ios.enabled);
        assert!(sanitized.package_names.ios_bundle_id.is_none());
        assert!(sanitized.package_names.harmony_bundle.is_none());
        let raw = sanitized.manifest_value.as_ref().unwrap();
        assert!(raw["app-plus"]["distribute"].get("android").is_some());
        assert!(raw["app-plus"]["distribute"].get("ios").is_none());
        assert!(raw["app-plus"]["distribute"].get("harmony").is_none());
        assert!(raw.get("app-harmony").is_none());
        assert!(sanitized.project_root.is_empty());
        assert!(sanitized.warnings.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resource_zip_replaces_original_manifests_with_platform_sanitized_value() {
        let root = std::env::temp_dir().join(format!(
            "unipack-sanitized-resource-{}",
            uuid::Uuid::new_v4()
        ));
        let resource_root = root.join("resources");
        let app_root = resource_root.join("__UNI__SANITIZED");
        std::fs::create_dir_all(app_root.join("www")).unwrap();
        std::fs::write(app_root.join("www/index.js"), b"console.log('ok')").unwrap();
        let local_absolute_path = "/Users/private-machine/another-platform-secret.plist";
        let original = serde_json::json!({
            "appid": "__UNI__SANITIZED",
            "app-plus": {
                "distribute": {
                    "android": {
                        "packageName": "com.example.android",
                        "localOnlyPath": local_absolute_path
                    },
                    "ios": {
                        "providerSecret": "ios-secret-must-not-upload",
                        "certificate": local_absolute_path
                    },
                    "harmony": {
                        "providerSecret": "harmony-secret-must-not-upload"
                    }
                }
            },
            "app-harmony": {
                "providerSecret": "app-harmony-secret-must-not-upload"
            }
        });
        let original_bytes = serde_json::to_vec_pretty(&original).unwrap();
        let direct_manifest = app_root.join("manifest.json");
        let nested_manifest = app_root.join("www/manifest.json");
        std::fs::write(&direct_manifest, &original_bytes).unwrap();
        // A secondary manifest must not survive as an unsanitized duplicate.
        std::fs::write(&nested_manifest, &original_bytes).unwrap();

        validate_payload_resource_tree(&resource_root).unwrap();
        let layout = payload_resource_layout(&resource_root).unwrap();
        let manifest_info = crate::commands::shared::resource::parse_uniapp_manifest(
            &original,
            &direct_manifest,
            &app_root,
            None,
        );
        let payload_dir = root.join("payload");
        std::fs::create_dir_all(&payload_dir).unwrap();
        let mut project = crate::commands::project::ProjectConfig::default();
        let mut module_config = None;
        let sanitized = sanitize_payload_paths(
            &payload_dir,
            &mut project,
            Some(manifest_info),
            &mut module_config,
            &CloudPayloadSigning::default(),
            "android",
        )
        .unwrap()
        .unwrap();
        let resource_zip = root.join("resource.zip");
        write_sanitized_resource_zip(
            &root.join("workspace"),
            &layout,
            sanitized.manifest_value.as_ref(),
            &resource_zip,
        )
        .unwrap();

        let extracted = root.join("extracted");
        crate::utils::fs::unzip_file(&resource_zip, &extracted).unwrap();
        for manifest in [
            extracted.join("__UNI__SANITIZED/manifest.json"),
            extracted.join("__UNI__SANITIZED/www/manifest.json"),
        ] {
            let text = std::fs::read_to_string(&manifest).unwrap();
            let value: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert!(value["app-plus"]["distribute"].get("android").is_some());
            assert!(value["app-plus"]["distribute"].get("ios").is_none());
            assert!(value["app-plus"]["distribute"].get("harmony").is_none());
            assert!(value.get("app-harmony").is_none());
            assert!(!text.contains("ios-secret-must-not-upload"));
            assert!(!text.contains("harmony-secret-must-not-upload"));
            assert!(!text.contains(local_absolute_path));
        }
        assert_eq!(
            std::fs::read(&direct_manifest).unwrap(),
            original_bytes,
            "sanitizing a hard-linked staging tree must not mutate imported resources"
        );
        let sanitized_asset =
            std::fs::read_to_string(payload_dir.join("manifest-assets/platform-manifest.json"))
                .unwrap();
        assert!(!sanitized_asset.contains("ios-secret-must-not-upload"));
        assert!(!sanitized_asset.contains(local_absolute_path));
        assert_eq!(
            std::fs::read_to_string(extracted.join("__UNI__SANITIZED/www/index.js")).unwrap(),
            "console.log('ok')"
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
