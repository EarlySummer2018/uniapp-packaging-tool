use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tauri::Emitter;

const GITHUB_API: &str = "https://api.github.com";
const GITHUB_UPLOADS: &str = "https://uploads.github.com";
const TOKEN_ACCOUNT: &str = "github-cloud-build-token";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BuildExecutionMode {
    Auto,
    Local,
    Github,
}

impl Default for BuildExecutionMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCloudBuildConfig {
    pub owner: String,
    pub repo: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub workflow_file: String,
    pub android_default_mode: BuildExecutionMode,
    pub ios_default_mode: BuildExecutionMode,
    pub android_sdk_url: String,
    pub ios_sdk_url: String,
}

impl Default for GithubCloudBuildConfig {
    fn default() -> Self {
        Self {
            owner: String::new(),
            repo: String::new(),
            ref_name: "main".to_string(),
            workflow_file: "cloud-build.yml".to_string(),
            android_default_mode: BuildExecutionMode::Auto,
            ios_default_mode: BuildExecutionMode::Auto,
            android_sdk_url: String::new(),
            ios_sdk_url: String::new(),
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
    global_sdk_config: crate::commands::sdk::GlobalSdkConfig,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    module_config: Option<HashMap<String, String>>,
    ios_packaging_mode: Option<String>,
    android_sdk_url: String,
    ios_sdk_url: String,
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

#[derive(Debug, Deserialize)]
struct WorkflowRuns {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Deserialize)]
struct WorkflowRun {
    id: u64,
    html_url: String,
    status: Option<String>,
    conclusion: Option<String>,
    created_at: Option<String>,
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
pub async fn get_github_cloud_build_secret_status() -> Result<GithubCloudBuildSecretStatus, String> {
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
    let client = GithubClient::new(token)?;
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
    let client = GithubClient::new(token)?;
    let build_id = request
        .build_id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("github-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
    let platform = normalize_platform(&request.platform)?;
    emit_cloud_log(&window, &build_id, &platform, "info", "准备 GitHub 云端打包", Some(5));

    let repo = client.get_repo(&config).await?;
    if !repo.private {
        return Err("GitHub 云端打包需要使用私有仓库，已拒绝上传构建包".to_string());
    }
    client.get_workflow(&config).await?;

    let workspace = cloud_workspace(&request.project_id, &build_id)?;
    let payload_zip = prepare_payload_zip(&workspace, &request, &config, &build_id, &platform)?;
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
    };

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
        emit_cloud_log(
            &window,
            &build_id,
            &platform,
            "info",
            "触发 GitHub Actions workflow",
            Some(25),
        );
        let dispatched_at = SystemTime::now();
        client
            .dispatch_workflow(
                &config,
                &build_id,
                &platform,
                &tag,
                &asset_name,
                &artifact_name,
            )
            .await?;
        let run = client
            .wait_for_workflow_run(&config, dispatched_at, &window, &build_id, &platform)
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
        let output_path = collect_downloaded_artifact(&request.project_id, &workspace, &artifact_zip, &platform)?;
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

    cleanup.cleanup(&window, &build_id, &platform).await;
    result
}

fn config_path() -> PathBuf {
    crate::utils::fs::get_unipack_home().join("github-cloud-build.json")
}

fn load_config() -> Result<GithubCloudBuildConfig, String> {
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
    for (label, value) in [
        ("owner", &config.owner),
        ("repo", &config.repo),
        ("ref", &config.ref_name),
        ("workflowFile", &config.workflow_file),
    ] {
        if value.contains('/') && label != "workflowFile" {
            return Err(format!("{} 不能包含 /", label));
        }
    }
    Ok(())
}

fn validate_config_ready(config: &GithubCloudBuildConfig) -> Result<(), String> {
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

fn require_token() -> Result<String, String> {
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

fn prepare_payload_zip(
    workspace: &Path,
    request: &GithubCloudBuildRequest,
    config: &GithubCloudBuildConfig,
    build_id: &str,
    platform: &str,
) -> Result<PathBuf, String> {
    let payload_dir = workspace.join("payload");
    std::fs::create_dir_all(&payload_dir).map_err(|e| e.to_string())?;
    let resource_zip = payload_dir.join("resource.zip");
    crate::utils::fs::zip_directory(Path::new(&request.resource_path), &resource_zip)
        .map_err(|e| format!("压缩导入资源失败: {}", e))?;

    let project_config = crate::commands::project::load_project_config_sync(&request.project_id)?;
    let global_sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
    let signing = collect_signing_files(&payload_dir, &project_config, platform)?;
    let metadata = CloudPayloadMetadata {
        version: 1,
        build_id: build_id.to_string(),
        platform: platform.to_string(),
        project_id: request.project_id.clone(),
        project_config,
        global_sdk_config,
        manifest_info: request.manifest_info.clone(),
        module_config: request.module_config.clone(),
        ios_packaging_mode: request.ios_packaging_mode.clone(),
        android_sdk_url: config.android_sdk_url.clone(),
        ios_sdk_url: config.ios_sdk_url.clone(),
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

fn collect_signing_files(
    payload_dir: &Path,
    config: &crate::commands::project::ProjectConfig,
    platform: &str,
) -> Result<CloudPayloadSigning, String> {
    let signing_dir = payload_dir.join("signing");
    std::fs::create_dir_all(&signing_dir).map_err(|e| e.to_string())?;
    let mut signing = CloudPayloadSigning::default();
    if platform == "android" {
        if let Some(file_name) =
            copy_optional_file(&config.android.keystore.path, &signing_dir, "android-keystore")?
        {
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
        signing.ios_certificate_password =
            crate::utils::keychain::get_password(&format!("{}-ios-certificate-password", config.id))
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
        .unwrap_or(if platform == "ios" { "app.ipa" } else { "app.apk" });
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
            .map(|ext| exts.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)))
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

    async fn get_workflow(&self, config: &GithubCloudBuildConfig) -> Result<GithubWorkflow, String> {
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
        let url = format!("{}/repos/{}/{}/releases", GITHUB_API, config.owner, config.repo);
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
        let bytes = std::fs::read(path).map_err(|e| format!("读取 payload 失败: {}", e))?;
        self.request(reqwest::Method::POST, &url)
            .header(reqwest::header::CONTENT_TYPE, "application/zip")
            .body(bytes)
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
                    "artifact_name": artifact_name
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
        dispatched_at: SystemTime,
        window: &tauri::Window,
        build_id: &str,
        platform: &str,
    ) -> Result<WorkflowRun, String> {
        let started = Instant::now();
        let mut last_status = String::new();
        loop {
            let runs = self.list_runs(config).await?;
            if let Some(run) = runs
                .workflow_runs
                .into_iter()
                .filter(|run| run.created_at.as_deref().is_some())
                .find(|run| run_is_after_dispatch(run, dispatched_at))
            {
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
            if started.elapsed() > Duration::from_secs(60 * 60) {
                return Err("等待 GitHub Actions 超时".to_string());
            }
            tokio::time::sleep(Duration::from_secs(8)).await;
        }
    }

    async fn list_runs(&self, config: &GithubCloudBuildConfig) -> Result<WorkflowRuns, String> {
        let url = format!(
            "{}/repos/{}/{}/actions/workflows/{}/runs?event=workflow_dispatch&branch={}&per_page=10",
            GITHUB_API, config.owner, config.repo, config.workflow_file, config.ref_name
        );
        self.get_json(&url).await
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

    async fn delete_release(&self, config: &GithubCloudBuildConfig, release_id: u64) -> Result<(), String> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "{}/repos/{}/{}/releases/{}",
                GITHUB_API, config.owner, config.repo, release_id
            ),
        )
        .send()
        .await
        .map_err(|e| format!("删除 GitHub Release 失败: {}", e))?
        .error_for_status()
        .map_err(|e| format!("删除 GitHub Release 失败: {}", e))?;
        Ok(())
    }

    async fn delete_tag(&self, config: &GithubCloudBuildConfig, tag: &str) -> Result<(), String> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "{}/repos/{}/{}/git/refs/tags/{}",
                GITHUB_API, config.owner, config.repo, tag
            ),
        )
        .send()
        .await
        .map_err(|e| format!("删除 GitHub Tag 失败: {}", e))?
        .error_for_status()
        .map_err(|e| format!("删除 GitHub Tag 失败: {}", e))?;
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
}

impl ReleaseCleanup {
    async fn cleanup(&self, window: &tauri::Window, build_id: &str, platform: &str) {
        if let Err(error) = self.client.delete_release(&self.config, self.release_id).await {
            emit_cloud_log(
                window,
                build_id,
                platform,
                "warn",
                &format!("清理 GitHub Release 失败: {}", error),
                None,
            );
        }
        if let Err(error) = self.client.delete_tag(&self.config, &self.tag).await {
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
}

fn run_is_after_dispatch(run: &WorkflowRun, dispatched_at: SystemTime) -> bool {
    let Some(created_at) = run.created_at.as_deref() else {
        return true;
    };
    let Ok(created_at) = chrono::DateTime::parse_from_rfc3339(created_at) else {
        return true;
    };
    let dispatch_at: chrono::DateTime<chrono::Utc> = dispatched_at.into();
    created_at.with_timezone(&chrono::Utc) + chrono::Duration::seconds(30) >= dispatch_at
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_auto_modes() {
        let config = GithubCloudBuildConfig::default();
        assert_eq!(config.android_default_mode, BuildExecutionMode::Auto);
        assert_eq!(config.ios_default_mode, BuildExecutionMode::Auto);
        assert_eq!(config.workflow_file, "cloud-build.yml");
    }

    #[test]
    fn safe_ref_component_removes_unsafe_chars() {
        assert_eq!(safe_ref_component("build/a:b c"), "build-a-b-c");
    }
}
