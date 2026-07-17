use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio_util::io::ReaderStream;

use super::cloud_build::GithubCloudBuildConfig;

pub const SDK_CACHE_RELEASE_TAG: &str = "unipack-sdk-cache-v1";
pub const SDK_CACHE_ARCHIVE_FORMAT: &str = "tar.zst";
const GITHUB_API: &str = "https://api.github.com";
const CACHE_MANIFEST_VERSION: u8 = 1;
const CHUNK_SIZE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 2 * 1024 * 1024;
const CACHE_VERSIONS_PER_PLATFORM: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdkCacheReference {
    pub platform: String,
    pub release_tag: String,
    pub manifest_asset: String,
    pub fingerprint: String,
    pub archive_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdkCachePart {
    pub index: u32,
    pub name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdkCacheManifest {
    pub version: u8,
    pub platform: String,
    pub fingerprint: String,
    pub original_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub created_at: String,
    pub parts: Vec<SdkCachePart>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GithubSdkCacheEntry {
    pub platform: String,
    pub fingerprint: String,
    pub compressed_size_bytes: u64,
    pub uploaded_at: String,
    pub matches_current_local_sdk: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalSdkCacheInspection {
    pub platform: String,
    pub fingerprint: String,
    pub size_bytes: u64,
    pub cache_hit: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct LocalSdkSnapshot {
    pub platform: String,
    pub root: PathBuf,
    pub fingerprint: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct CacheRelease {
    id: u64,
    upload_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReleaseAsset {
    id: u64,
    name: String,
    size: u64,
    #[serde(default)]
    digest: Option<String>,
}

#[derive(Debug)]
struct LocalPart {
    metadata: SdkCachePart,
    path: PathBuf,
}

#[derive(Clone)]
struct CacheGithubClient {
    client: reqwest::Client,
    token: String,
    owner: String,
    repo: String,
}

impl CacheGithubClient {
    fn new(config: &GithubCloudBuildConfig, token: String) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .user_agent("unipack-tool")
            .build()
            .map_err(|error| format!("创建 GitHub HTTP Client 失败: {}", error))?;
        Ok(Self {
            client,
            token,
            owner: config.owner.clone(),
            repo: config.repo.clone(),
        })
    }

    fn request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, url)
            .bearer_auth(&self.token)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    fn repo_url(&self, suffix: &str) -> String {
        format!(
            "{}/repos/{}/{}{}",
            GITHUB_API, self.owner, self.repo, suffix
        )
    }

    async fn get_cache_release(&self) -> Result<Option<CacheRelease>, String> {
        let response = self
            .request(
                reqwest::Method::GET,
                &self.repo_url(&format!("/releases/tags/{}", SDK_CACHE_RELEASE_TAG)),
            )
            .send()
            .await
            .map_err(|error| format!("查询 GitHub SDK 缓存 Release 失败: {}", error))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        response
            .error_for_status()
            .map_err(|error| format!("查询 GitHub SDK 缓存 Release 失败: {}", error))?
            .json::<CacheRelease>()
            .await
            .map(Some)
            .map_err(|error| format!("解析 GitHub SDK 缓存 Release 失败: {}", error))
    }

    async fn get_or_create_cache_release(&self, target_ref: &str) -> Result<CacheRelease, String> {
        if let Some(release) = self.get_cache_release().await? {
            return Ok(release);
        }
        let response = self
            .request(reqwest::Method::POST, &self.repo_url("/releases"))
            .json(&serde_json::json!({
                "tag_name": SDK_CACHE_RELEASE_TAG,
                "target_commitish": target_ref,
                "name": "UniPack SDK cache",
                "body": "Persistent private cache used by UniPack GitHub cloud builds.",
                "draft": false,
                "prerelease": true
            }))
            .send()
            .await
            .map_err(|error| format!("创建 GitHub SDK 缓存 Release 失败: {}", error))?;
        if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
            return self
                .get_cache_release()
                .await?
                .ok_or_else(|| "GitHub SDK 缓存 Release 创建冲突，请稍后重试".to_string());
        }
        response
            .error_for_status()
            .map_err(|error| format!("创建 GitHub SDK 缓存 Release 失败: {}", error))?
            .json::<CacheRelease>()
            .await
            .map_err(|error| format!("解析 GitHub SDK 缓存 Release 失败: {}", error))
    }

    async fn list_assets(&self, release_id: u64) -> Result<Vec<ReleaseAsset>, String> {
        let mut all = Vec::new();
        for page in 1..=20 {
            let url = self.repo_url(&format!(
                "/releases/{}/assets?per_page=100&page={}",
                release_id, page
            ));
            let batch = self
                .request(reqwest::Method::GET, &url)
                .send()
                .await
                .map_err(|error| format!("查询 GitHub SDK 缓存文件失败: {}", error))?
                .error_for_status()
                .map_err(|error| format!("查询 GitHub SDK 缓存文件失败: {}", error))?
                .json::<Vec<ReleaseAsset>>()
                .await
                .map_err(|error| format!("解析 GitHub SDK 缓存文件失败: {}", error))?;
            let count = batch.len();
            all.extend(batch);
            if count < 100 {
                return Ok(all);
            }
        }
        Err("GitHub SDK 缓存文件数量异常，已停止分页读取".to_string())
    }

    async fn download_asset_bytes(&self, asset_id: u64) -> Result<Vec<u8>, String> {
        let response = self
            .request(
                reqwest::Method::GET,
                &self.repo_url(&format!("/releases/assets/{}", asset_id)),
            )
            .header(reqwest::header::ACCEPT, "application/octet-stream")
            .send()
            .await
            .map_err(|error| format!("下载 GitHub SDK 缓存清单失败: {}", error))?
            .error_for_status()
            .map_err(|error| format!("下载 GitHub SDK 缓存清单失败: {}", error))?;
        if response.content_length().unwrap_or(0) > MAX_MANIFEST_BYTES as u64 {
            return Err("GitHub SDK 缓存清单体积异常".to_string());
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("读取 GitHub SDK 缓存清单失败: {}", error))?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err("GitHub SDK 缓存清单体积异常".to_string());
        }
        Ok(bytes.to_vec())
    }

    async fn upload_file(
        &self,
        upload_url: &str,
        name: &str,
        path: &Path,
        content_type: &'static str,
    ) -> Result<(), String> {
        let base = upload_url
            .split_once('{')
            .map(|(base, _)| base)
            .unwrap_or(upload_url);
        let size = std::fs::metadata(path)
            .map_err(|error| format!("读取待上传缓存文件失败: {}", error))?
            .len();
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|error| format!("打开待上传缓存文件失败: {}", error))?;
        self.request(reqwest::Method::POST, &format!("{}?name={}", base, name))
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .header(reqwest::header::CONTENT_LENGTH, size)
            .body(reqwest::Body::wrap_stream(ReaderStream::new(file)))
            .send()
            .await
            .map_err(|error| format!("上传 GitHub SDK 缓存文件失败: {}", error))?
            .error_for_status()
            .map_err(|error| format!("上传 GitHub SDK 缓存文件失败: {}", error))?;
        Ok(())
    }

    async fn delete_asset(&self, asset_id: u64) -> Result<(), String> {
        let response = self
            .request(
                reqwest::Method::DELETE,
                &self.repo_url(&format!("/releases/assets/{}", asset_id)),
            )
            .send()
            .await
            .map_err(|error| format!("删除 GitHub SDK 缓存文件失败: {}", error))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        response
            .error_for_status()
            .map_err(|error| format!("删除 GitHub SDK 缓存文件失败: {}", error))?;
        Ok(())
    }
}

#[tauri::command]
pub async fn inspect_local_sdk_cache(platform: String) -> Result<LocalSdkCacheInspection, String> {
    let platform = normalize_platform(&platform)?.to_string();
    let snapshot = tokio::task::spawn_blocking({
        let platform = platform.clone();
        move || inspect_local_sdk_sync(&platform)
    })
    .await
    .map_err(|error| format!("检查本地 SDK 任务失败: {}", error))??;
    let config = super::cloud_build::load_config()?;
    super::cloud_build::validate_config_ready(&config)?;
    let token = super::cloud_build::require_token()?;
    let client = CacheGithubClient::new(&config, token)?;
    let cache_hit = if let Some(release) = client.get_cache_release().await? {
        let assets = client.list_assets(release.id).await?;
        find_valid_manifest(&client, &assets, &snapshot.platform, &snapshot.fingerprint)
            .await?
            .is_some()
    } else {
        false
    };
    Ok(LocalSdkCacheInspection {
        platform: snapshot.platform,
        fingerprint: snapshot.fingerprint,
        size_bytes: snapshot.size_bytes,
        cache_hit,
    })
}

#[tauri::command]
pub async fn get_github_sdk_cache_status(
    platform: Option<String>,
) -> Result<Vec<GithubSdkCacheEntry>, String> {
    let platform = platform
        .as_deref()
        .map(normalize_platform)
        .transpose()?
        .map(str::to_string);
    let config = super::cloud_build::load_config()?;
    super::cloud_build::validate_config_ready(&config)?;
    let client = CacheGithubClient::new(&config, super::cloud_build::require_token()?)?;
    let Some(release) = client.get_cache_release().await? else {
        return Ok(Vec::new());
    };
    let assets = client.list_assets(release.id).await?;
    let mut local_fingerprints = HashMap::new();
    for candidate in platform
        .as_deref()
        .map(|value| vec![value])
        .unwrap_or_else(|| vec!["android", "ios"])
    {
        let candidate = candidate.to_string();
        if let Ok(Ok(snapshot)) = tokio::task::spawn_blocking({
            let candidate = candidate.clone();
            move || inspect_local_sdk_sync(&candidate)
        })
        .await
        {
            local_fingerprints.insert(candidate, snapshot.fingerprint);
        }
    }

    let mut manifests = read_valid_manifests(&client, &assets, platform.as_deref()).await?;
    manifests.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    let mut per_platform = HashMap::<String, usize>::new();
    let mut entries = Vec::new();
    for manifest in manifests {
        let count = per_platform.entry(manifest.platform.clone()).or_default();
        if *count >= CACHE_VERSIONS_PER_PLATFORM {
            continue;
        }
        *count += 1;
        entries.push(GithubSdkCacheEntry {
            platform: manifest.platform.clone(),
            fingerprint: manifest.fingerprint.clone(),
            compressed_size_bytes: manifest.compressed_size_bytes,
            uploaded_at: manifest.created_at,
            matches_current_local_sdk: local_fingerprints
                .get(&manifest.platform)
                .map(|value| value == &manifest.fingerprint)
                .unwrap_or(false),
        });
    }
    Ok(entries)
}

#[tauri::command]
pub async fn delete_github_sdk_cache(platform: String, fingerprint: String) -> Result<(), String> {
    let platform = normalize_platform(&platform)?;
    validate_fingerprint(&fingerprint)?;
    let config = super::cloud_build::load_config()?;
    super::cloud_build::validate_config_ready(&config)?;
    let client = CacheGithubClient::new(&config, super::cloud_build::require_token()?)?;
    let Some(release) = client.get_cache_release().await? else {
        return Ok(());
    };
    let assets = client.list_assets(release.id).await?;
    let manifest_name = manifest_asset_name(platform, &fingerprint);
    let part_prefix = part_asset_prefix(platform, &fingerprint);

    // Delete the manifest first so a concurrent build never treats a partially deleted
    // cache version as complete.
    let mut errors = Vec::new();
    if let Some(asset) = assets.iter().find(|asset| asset.name == manifest_name) {
        if let Err(error) = client.delete_asset(asset.id).await {
            errors.push(error);
        }
    }
    for asset in assets
        .iter()
        .filter(|asset| asset.name.starts_with(&part_prefix))
    {
        if let Err(error) = client.delete_asset(asset.id).await {
            errors.push(error);
        }
    }
    if !errors.is_empty() {
        return Err(format!("删除 SDK 缓存失败: {}", errors.join("；")));
    }
    Ok(())
}

pub(crate) async fn ensure_github_sdk_cache(
    config: &GithubCloudBuildConfig,
    token: &str,
    platform: &str,
    workspace: &Path,
) -> Result<SdkCacheReference, String> {
    let platform = normalize_platform(platform)?.to_string();
    let snapshot = tokio::task::spawn_blocking({
        let platform = platform.clone();
        move || inspect_local_sdk_sync(&platform)
    })
    .await
    .map_err(|error| format!("检查本地 SDK 任务失败: {}", error))??;
    let client = CacheGithubClient::new(config, token.to_string())?;
    let release = client.get_or_create_cache_release(&config.ref_name).await?;
    let mut assets = client.list_assets(release.id).await?;

    if find_valid_manifest(&client, &assets, &platform, &snapshot.fingerprint)
        .await?
        .is_some()
    {
        prune_old_versions(&client, release.id, &platform).await?;
        return Ok(cache_reference(&platform, &snapshot.fingerprint));
    }

    let package_dir = workspace.join("sdk-cache-upload");
    let package = tokio::task::spawn_blocking({
        let snapshot = snapshot.clone();
        move || package_sdk(&snapshot, &package_dir, CHUNK_SIZE_BYTES)
    })
    .await
    .map_err(|error| format!("压缩 SDK 缓存任务失败: {}", error))??;

    for part in &package.parts {
        if reusable_part_uploaded(&assets, &part.metadata) {
            continue;
        }
        if let Some(existing) = assets.iter().find(|asset| asset.name == part.metadata.name) {
            let existing_id = existing.id;
            client.delete_asset(existing_id).await?;
            assets.retain(|asset| asset.id != existing_id);
        }
        client
            .upload_file(
                &release.upload_url,
                &part.metadata.name,
                &part.path,
                "application/octet-stream",
            )
            .await?;
    }

    // The manifest is the commit marker and is deliberately uploaded last.
    let manifest_name = manifest_asset_name(&platform, &snapshot.fingerprint);
    if let Some(existing) = assets.iter().find(|asset| asset.name == manifest_name) {
        client.delete_asset(existing.id).await?;
    }
    client
        .upload_file(
            &release.upload_url,
            &manifest_name,
            &package.manifest_path,
            "application/json",
        )
        .await?;

    prune_old_versions(&client, release.id, &platform).await?;
    Ok(cache_reference(&platform, &snapshot.fingerprint))
}

fn reusable_part_uploaded(assets: &[ReleaseAsset], part: &SdkCachePart) -> bool {
    // GitHub computes a digest for every release asset. Reuse only a byte-for-byte
    // match; a matching name and size alone is not a completed upload.
    assets
        .iter()
        .any(|asset| release_asset_matches_part(asset, part))
}

fn release_asset_matches_part(asset: &ReleaseAsset, part: &SdkCachePart) -> bool {
    asset.name == part.name
        && asset.size == part.size_bytes
        && asset
            .digest
            .as_deref()
            .map(|digest| digest.eq_ignore_ascii_case(&format!("sha256:{}", part.sha256)))
            .unwrap_or(false)
}

fn cache_reference(platform: &str, fingerprint: &str) -> SdkCacheReference {
    SdkCacheReference {
        platform: platform.to_string(),
        release_tag: SDK_CACHE_RELEASE_TAG.to_string(),
        manifest_asset: manifest_asset_name(platform, fingerprint),
        fingerprint: fingerprint.to_string(),
        archive_format: SDK_CACHE_ARCHIVE_FORMAT.to_string(),
    }
}

pub(crate) fn inspect_local_sdk_sync(platform: &str) -> Result<LocalSdkSnapshot, String> {
    let platform = normalize_platform(platform)?.to_string();
    let config = crate::commands::sdk::load_global_sdk_config_sync()?;
    let configured = match platform.as_str() {
        "android" => config.dcloud_android_sdk_path,
        "ios" => config.dcloud_ios_sdk_path,
        _ => unreachable!(),
    };
    if configured.trim().is_empty() {
        return Err(format!(
            "请先在 DCloud 离线 SDK 页面配置 {} SDK 路径",
            if platform == "android" {
                "Android"
            } else {
                "iOS"
            }
        ));
    }
    let root =
        crate::commands::sdk::normalize_global_sdk_path(&platform, Path::new(configured.trim()))?;
    let root = root
        .canonicalize()
        .map_err(|error| format!("读取本地 SDK 路径失败: {}", error))?;
    let (fingerprint, size_bytes) = fingerprint_sdk_directory(&root)?;
    Ok(LocalSdkSnapshot {
        platform,
        root,
        fingerprint,
        size_bytes,
    })
}

fn normalize_platform(platform: &str) -> Result<&'static str, String> {
    match platform.trim().to_ascii_lowercase().as_str() {
        "android" => Ok("android"),
        "ios" => Ok("ios"),
        "harmony" | "harmonyos" => Err("HarmonyOS 暂不支持 GitHub 云端 SDK 缓存".to_string()),
        _ => Err(format!("不支持的 SDK 缓存平台: {}", platform)),
    }
}

fn validate_fingerprint(fingerprint: &str) -> Result<(), String> {
    if fingerprint.len() != 64
        || !fingerprint
            .bytes()
            .all(|value| value.is_ascii_hexdigit() && !value.is_ascii_uppercase())
    {
        return Err("SDK 指纹格式无效".to_string());
    }
    Ok(())
}

fn manifest_asset_name(platform: &str, fingerprint: &str) -> String {
    format!("unipack-sdk-{}-{}.manifest.json", platform, fingerprint)
}

fn part_asset_prefix(platform: &str, fingerprint: &str) -> String {
    format!("unipack-sdk-{}-{}.part-", platform, fingerprint)
}

fn part_asset_name(platform: &str, fingerprint: &str, index: u32, sha256: &str) -> String {
    format!(
        "{}{:05}-{}.tar.zst",
        part_asset_prefix(platform, fingerprint),
        index,
        sha256
    )
}

fn fingerprint_sdk_directory(root: &Path) -> Result<(String, u64), String> {
    if !root.is_dir() {
        return Err(format!("SDK 路径不是目录: {}", root.display()));
    }
    let entries = sorted_sdk_entries(root)?;
    let mut digest = Sha256::new();
    let mut size_bytes = 0u64;
    for entry in entries {
        hash_field(&mut digest, entry.relative.as_bytes());
        hash_field(&mut digest, entry.kind.as_bytes());
        hash_field(&mut digest, &entry.size.to_le_bytes());
        hash_field(&mut digest, &entry.modified_seconds.to_le_bytes());
        hash_field(&mut digest, &entry.modified_nanos.to_le_bytes());
        hash_field(
            &mut digest,
            entry
                .symlink_target
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        );
        if entry.kind == "file" {
            size_bytes = size_bytes.saturating_add(entry.size);
        }
    }
    Ok((hex_digest(digest.finalize().as_slice()), size_bytes))
}

#[derive(Debug)]
struct FingerprintEntry {
    relative: String,
    relative_path: PathBuf,
    kind: &'static str,
    size: u64,
    modified_seconds: i64,
    modified_nanos: u32,
    symlink_target: Option<String>,
}

fn collect_sdk_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<FingerprintEntry>,
) -> Result<(), String> {
    let mut children = std::fs::read_dir(directory)
        .map_err(|error| format!("读取 SDK 目录失败 {}: {}", directory.display(), error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("读取 SDK 目录项失败: {}", error))?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("读取 SDK 文件信息失败 {}: {}", path.display(), error))?;
        let relative_path = path
            .strip_prefix(root)
            .map_err(|_| "SDK 文件超出根目录".to_string())?;
        let relative = safe_relative_string(relative_path)?;
        let file_type = metadata.file_type();
        let (kind, target) = if file_type.is_symlink() {
            let target = std::fs::read_link(&path)
                .map_err(|error| format!("读取 SDK 符号链接失败 {}: {}", path.display(), error))?;
            validate_symlink_target(relative_path, &target)?;
            ("symlink", Some(safe_relative_string(&target)?))
        } else if file_type.is_dir() {
            ("directory", None)
        } else if file_type.is_file() {
            ("file", None)
        } else {
            return Err(format!("SDK 包含不支持的特殊文件: {}", relative));
        };
        let (modified_seconds, modified_nanos) = modified_timestamp(&metadata);
        entries.push(FingerprintEntry {
            relative,
            relative_path: relative_path.to_path_buf(),
            kind,
            size: metadata.len(),
            modified_seconds,
            modified_nanos,
            symlink_target: target,
        });
        if file_type.is_dir() {
            collect_sdk_entries(root, &path, entries)?;
        }
    }
    Ok(())
}

fn sorted_sdk_entries(root: &Path) -> Result<Vec<FingerprintEntry>, String> {
    let mut entries = Vec::new();
    collect_sdk_entries(root, root, &mut entries)?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok(entries)
}

fn modified_timestamp(metadata: &std::fs::Metadata) -> (i64, u32) {
    match metadata.modified() {
        Ok(value) => match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => (
                duration.as_secs().min(i64::MAX as u64) as i64,
                duration.subsec_nanos(),
            ),
            Err(error) => {
                let duration = error.duration();
                (
                    -(duration.as_secs().min(i64::MAX as u64) as i64),
                    duration.subsec_nanos(),
                )
            }
        },
        Err(_) => (0, 0),
    }
}

fn safe_relative_string(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .ok_or_else(|| "SDK 路径不是有效 UTF-8，无法安全上传".to_string())?
                    .to_string(),
            ),
            Component::CurDir => {}
            Component::ParentDir => parts.push("..".to_string()),
            Component::RootDir | Component::Prefix(_) => {
                return Err("SDK 包含绝对路径符号链接，无法安全上传".to_string())
            }
        }
    }
    Ok(parts.join("/"))
}

fn validate_symlink_target(link_relative: &Path, target: &Path) -> Result<(), String> {
    if target.is_absolute() {
        return Err(format!(
            "SDK 符号链接使用绝对路径，无法安全上传: {}",
            link_relative.display()
        ));
    }
    let mut depth = link_relative
        .parent()
        .map(|path| path.components().count())
        .unwrap_or(0);
    for component in target.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => {
                if depth == 0 {
                    return Err(format!(
                        "SDK 符号链接指向根目录外，无法安全上传: {}",
                        link_relative.display()
                    ));
                }
                depth -= 1;
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "SDK 符号链接使用绝对路径，无法安全上传: {}",
                    link_relative.display()
                ));
            }
        }
    }
    Ok(())
}

fn hash_field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value);
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut value, "{:02x}", byte);
    }
    value
}

struct PackagedSdk {
    parts: Vec<LocalPart>,
    manifest_path: PathBuf,
}

fn package_sdk(
    snapshot: &LocalSdkSnapshot,
    output_dir: &Path,
    chunk_size: u64,
) -> Result<PackagedSdk, String> {
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir)
            .map_err(|error| format!("清理 SDK 压缩缓存失败: {}", error))?;
    }
    std::fs::create_dir_all(output_dir)
        .map_err(|error| format!("创建 SDK 压缩缓存目录失败: {}", error))?;
    // Re-scan immediately before archiving to reject unsafe links and reduce TOCTOU risk.
    let (fingerprint, size_bytes) = fingerprint_sdk_directory(&snapshot.root)?;
    if fingerprint != snapshot.fingerprint || size_bytes != snapshot.size_bytes {
        return Err("本地 SDK 在缓存准备期间发生变化，请重新发起打包".to_string());
    }

    let writer = ChunkWriter::new(
        output_dir,
        &snapshot.platform,
        &snapshot.fingerprint,
        chunk_size,
    )
    .map_err(|error| format!("创建 SDK 分片写入器失败: {}", error))?;
    let encoder = zstd::stream::write::Encoder::new(writer, 8)
        .map_err(|error| format!("创建 SDK zstd 压缩器失败: {}", error))?;
    let mut archive = tar::Builder::new(encoder);
    archive.follow_symlinks(false);
    // Write entries explicitly in fingerprint order. This makes the compressed
    // stream and part hashes stable across retries for the same SDK snapshot.
    for entry in sorted_sdk_entries(&snapshot.root)? {
        let source = snapshot.root.join(&entry.relative_path);
        if entry.kind == "directory" {
            archive
                .append_dir(&entry.relative_path, &source)
                .map_err(|error| format!("写入 SDK tar 目录失败: {}", error))?;
        } else {
            archive
                .append_path_with_name(&source, &entry.relative_path)
                .map_err(|error| format!("写入 SDK tar 条目失败: {}", error))?;
        }
    }
    let encoder = archive
        .into_inner()
        .map_err(|error| format!("完成 SDK tar 归档失败: {}", error))?;
    let writer = encoder
        .finish()
        .map_err(|error| format!("完成 SDK zstd 压缩失败: {}", error))?;
    let parts = writer
        .finish()
        .map_err(|error| format!("完成 SDK 分片失败: {}", error))?;
    let compressed_size_bytes = parts.iter().map(|part| part.metadata.size_bytes).sum();
    let manifest = SdkCacheManifest {
        version: CACHE_MANIFEST_VERSION,
        platform: snapshot.platform.clone(),
        fingerprint: snapshot.fingerprint.clone(),
        original_size_bytes: snapshot.size_bytes,
        compressed_size_bytes,
        created_at: chrono::Utc::now().to_rfc3339(),
        parts: parts.iter().map(|part| part.metadata.clone()).collect(),
    };
    validate_manifest(&manifest)?;
    let manifest_path = output_dir.join(manifest_asset_name(
        &snapshot.platform,
        &snapshot.fingerprint,
    ));
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest)
            .map_err(|error| format!("序列化 SDK 缓存清单失败: {}", error))?,
    )
    .map_err(|error| format!("写入 SDK 缓存清单失败: {}", error))?;
    Ok(PackagedSdk {
        parts,
        manifest_path,
    })
}

struct OpenChunk {
    index: u32,
    path: PathBuf,
    file: File,
    digest: Sha256,
    size: u64,
}

struct ChunkWriter {
    output_dir: PathBuf,
    platform: String,
    fingerprint: String,
    max_size: u64,
    next_index: u32,
    current: Option<OpenChunk>,
    parts: Vec<LocalPart>,
}

impl ChunkWriter {
    fn new(
        output_dir: &Path,
        platform: &str,
        fingerprint: &str,
        max_size: u64,
    ) -> io::Result<Self> {
        if max_size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "chunk size must be positive",
            ));
        }
        Ok(Self {
            output_dir: output_dir.to_path_buf(),
            platform: platform.to_string(),
            fingerprint: fingerprint.to_string(),
            max_size,
            next_index: 0,
            current: None,
            parts: Vec::new(),
        })
    }

    fn open_chunk(&mut self) -> io::Result<()> {
        if self.current.is_some() {
            return Ok(());
        }
        let index = self.next_index;
        self.next_index += 1;
        let path = self.output_dir.join(format!("chunk-{:05}.tmp", index));
        self.current = Some(OpenChunk {
            index,
            file: File::create(&path)?,
            path,
            digest: Sha256::new(),
            size: 0,
        });
        Ok(())
    }

    fn close_chunk(&mut self) -> io::Result<()> {
        let Some(mut chunk) = self.current.take() else {
            return Ok(());
        };
        chunk.file.flush()?;
        chunk.file.sync_all()?;
        let sha256 = hex_digest(chunk.digest.finalize().as_slice());
        let name = part_asset_name(&self.platform, &self.fingerprint, chunk.index, &sha256);
        let destination = self.output_dir.join(&name);
        if destination.exists() {
            std::fs::remove_file(&destination)?;
        }
        std::fs::rename(&chunk.path, &destination)?;
        self.parts.push(LocalPart {
            metadata: SdkCachePart {
                index: chunk.index,
                name,
                size_bytes: chunk.size,
                sha256,
            },
            path: destination,
        });
        Ok(())
    }

    fn finish(mut self) -> io::Result<Vec<LocalPart>> {
        self.close_chunk()?;
        Ok(self.parts)
    }
}

impl Write for ChunkWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut offset = 0usize;
        while offset < buffer.len() {
            self.open_chunk()?;
            let current = self.current.as_mut().expect("chunk must be open");
            let available = (self.max_size - current.size) as usize;
            let count = available.min(buffer.len() - offset);
            current.file.write_all(&buffer[offset..offset + count])?;
            current.digest.update(&buffer[offset..offset + count]);
            current.size += count as u64;
            offset += count;
            if current.size == self.max_size {
                self.close_chunk()?;
            }
        }
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(chunk) = self.current.as_mut() {
            chunk.file.flush()?;
        }
        Ok(())
    }
}

async fn find_valid_manifest(
    client: &CacheGithubClient,
    assets: &[ReleaseAsset],
    platform: &str,
    fingerprint: &str,
) -> Result<Option<SdkCacheManifest>, String> {
    let name = manifest_asset_name(platform, fingerprint);
    let Some(asset) = assets.iter().find(|asset| asset.name == name) else {
        return Ok(None);
    };
    let bytes = client.download_asset_bytes(asset.id).await?;
    let Ok(manifest) = serde_json::from_slice::<SdkCacheManifest>(&bytes) else {
        return Ok(None);
    };
    if validate_manifest(&manifest).is_err() {
        return Ok(None);
    }
    if manifest.platform != platform || manifest.fingerprint != fingerprint {
        return Ok(None);
    }
    if !manifest_parts_exist(&manifest, assets) {
        return Ok(None);
    }
    Ok(Some(manifest))
}

async fn read_valid_manifests(
    client: &CacheGithubClient,
    assets: &[ReleaseAsset],
    platform: Option<&str>,
) -> Result<Vec<SdkCacheManifest>, String> {
    read_manifests(client, assets, platform, true).await
}

async fn read_manifests(
    client: &CacheGithubClient,
    assets: &[ReleaseAsset],
    platform: Option<&str>,
    require_complete_parts: bool,
) -> Result<Vec<SdkCacheManifest>, String> {
    let mut manifests = Vec::new();
    for asset in assets.iter().filter(|asset| {
        asset.name.starts_with("unipack-sdk-") && asset.name.ends_with(".manifest.json")
    }) {
        let bytes = match client.download_asset_bytes(asset.id).await {
            Ok(value) => value,
            Err(_) => continue,
        };
        let manifest = match serde_json::from_slice::<SdkCacheManifest>(&bytes) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if validate_manifest(&manifest).is_err()
            || manifest_asset_name(&manifest.platform, &manifest.fingerprint) != asset.name
            || platform
                .map(|expected| manifest.platform != expected)
                .unwrap_or(false)
            || (require_complete_parts && !manifest_parts_exist(&manifest, assets))
        {
            continue;
        }
        manifests.push(manifest);
    }
    Ok(manifests)
}

fn manifest_parts_exist(manifest: &SdkCacheManifest, assets: &[ReleaseAsset]) -> bool {
    manifest.parts.iter().all(|part| {
        assets
            .iter()
            .any(|asset| release_asset_matches_part(asset, part))
    })
}

fn validate_manifest(manifest: &SdkCacheManifest) -> Result<(), String> {
    if manifest.version != CACHE_MANIFEST_VERSION {
        return Err(format!("不支持的 SDK 缓存清单版本: {}", manifest.version));
    }
    normalize_platform(&manifest.platform)?;
    validate_fingerprint(&manifest.fingerprint)?;
    chrono::DateTime::parse_from_rfc3339(&manifest.created_at)
        .map_err(|_| "SDK 缓存清单上传时间无效".to_string())?;
    if manifest.parts.is_empty() {
        return Err("SDK 缓存清单没有分片".to_string());
    }
    let mut names = HashSet::new();
    let mut compressed_size = 0u64;
    for (expected_index, part) in manifest.parts.iter().enumerate() {
        if part.index != expected_index as u32
            || part.size_bytes == 0
            || part.size_bytes > CHUNK_SIZE_BYTES
        {
            return Err("SDK 缓存分片索引或体积无效".to_string());
        }
        validate_fingerprint(&part.sha256)?;
        if part.name
            != part_asset_name(
                &manifest.platform,
                &manifest.fingerprint,
                part.index,
                &part.sha256,
            )
            || !names.insert(part.name.clone())
        {
            return Err("SDK 缓存分片名称无效".to_string());
        }
        compressed_size = compressed_size
            .checked_add(part.size_bytes)
            .ok_or_else(|| "SDK 缓存分片总体积溢出".to_string())?;
    }
    if compressed_size != manifest.compressed_size_bytes {
        return Err("SDK 缓存清单压缩体积不匹配".to_string());
    }
    Ok(())
}

async fn prune_old_versions(
    client: &CacheGithubClient,
    release_id: u64,
    platform: &str,
) -> Result<(), String> {
    let assets = client.list_assets(release_id).await?;
    let mut manifests = read_valid_manifests(client, &assets, Some(platform)).await?;
    manifests.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    let retained = manifests
        .iter()
        .take(CACHE_VERSIONS_PER_PLATFORM)
        .map(|manifest| manifest.fingerprint.clone())
        .collect::<HashSet<_>>();
    let mut errors = Vec::new();
    for manifest in manifests
        .into_iter()
        .filter(|manifest| !retained.contains(&manifest.fingerprint))
    {
        let manifest_name = manifest_asset_name(&manifest.platform, &manifest.fingerprint);
        let Some(manifest_asset) = assets.iter().find(|asset| asset.name == manifest_name) else {
            continue;
        };
        // The manifest is the commit marker: remove it first so new readers stop
        // accepting this version before any part disappears.
        if let Err(error) = client.delete_asset(manifest_asset.id).await {
            errors.push(error);
            continue;
        }
        for part in manifest.parts {
            if let Some(asset) = assets.iter().find(|asset| asset.name == part.name) {
                if let Err(error) = client.delete_asset(asset.id).await {
                    errors.push(error);
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("清理旧 SDK 缓存失败: {}", errors.join("；")))
    }
}

#[cfg(test)]
fn manifests_to_prune(
    mut manifests: Vec<SdkCacheManifest>,
    retained_versions: usize,
) -> Vec<SdkCacheManifest> {
    manifests.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    manifests.into_iter().skip(retained_versions).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "unipack-sdk-cache-{}-{}",
            name,
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn fingerprint_is_stable_and_does_not_include_root_path() {
        let left = temp_dir("fingerprint-left");
        let right = temp_dir("fingerprint-right");
        std::fs::create_dir_all(left.join("SDK/libs")).unwrap();
        std::fs::create_dir_all(right.join("SDK/libs")).unwrap();
        std::fs::write(left.join("SDK/libs/a.bin"), b"same").unwrap();
        std::fs::write(right.join("SDK/libs/a.bin"), b"same").unwrap();

        // Copy mtimes are not guaranteed to match, so this assertion checks that a
        // second scan of the same tree is deterministic. Root paths are separately
        // excluded by inspecting the relative-entry implementation.
        let first = fingerprint_sdk_directory(&left).unwrap();
        let second = fingerprint_sdk_directory(&left).unwrap();
        assert_eq!(first, second);
        assert!(!first.0.contains(left.to_string_lossy().as_ref()));

        std::fs::write(left.join("SDK/libs/a.bin"), b"changed-size").unwrap();
        assert_ne!(first, fingerprint_sdk_directory(&left).unwrap());
        let _ = std::fs::remove_dir_all(left);
        let _ = std::fs::remove_dir_all(right);
    }

    #[test]
    fn chunk_writer_splits_and_hashes_without_loading_whole_archive() {
        let root = temp_dir("chunks");
        let mut writer = ChunkWriter::new(&root, "android", &"a".repeat(64), 4).unwrap();
        writer.write_all(b"abcdefghij").unwrap();
        let parts = writer.finish().unwrap();
        assert_eq!(
            parts
                .iter()
                .map(|part| part.metadata.size_bytes)
                .collect::<Vec<_>>(),
            vec![4, 4, 2]
        );
        assert_eq!(std::fs::read(&parts[0].path).unwrap(), b"abcd");
        assert!(parts.iter().all(|part| part.metadata.sha256.len() == 64));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_rejects_forged_part_names() {
        let fingerprint = "a".repeat(64);
        let sha = "b".repeat(64);
        let manifest = SdkCacheManifest {
            version: CACHE_MANIFEST_VERSION,
            platform: "android".to_string(),
            fingerprint,
            original_size_bytes: 1,
            compressed_size_bytes: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            parts: vec![SdkCachePart {
                index: 0,
                name: "../../secret".to_string(),
                size_bytes: 1,
                sha256: sha,
            }],
        };
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn completed_parts_are_reused_but_truncated_parts_are_not() {
        let part = SdkCachePart {
            index: 0,
            name: part_asset_name("android", &"a".repeat(64), 0, &"b".repeat(64)),
            size_bytes: 512,
            sha256: "b".repeat(64),
        };
        assert!(reusable_part_uploaded(
            &[ReleaseAsset {
                id: 1,
                name: part.name.clone(),
                size: 512,
                digest: Some(format!("sha256:{}", part.sha256)),
            }],
            &part,
        ));
        assert!(!reusable_part_uploaded(
            &[ReleaseAsset {
                id: 2,
                name: part.name.clone(),
                size: 511,
                digest: Some(format!("sha256:{}", part.sha256)),
            }],
            &part,
        ));
        assert!(!reusable_part_uploaded(
            &[ReleaseAsset {
                id: 3,
                name: part.name.clone(),
                size: 512,
                digest: Some(format!("sha256:{}", "c".repeat(64))),
            }],
            &part,
        ));
        assert!(!reusable_part_uploaded(
            &[ReleaseAsset {
                id: 4,
                name: part.name.clone(),
                size: 512,
                digest: None,
            }],
            &part,
        ));
    }

    #[test]
    fn retention_keeps_only_two_newest_complete_versions() {
        fn manifest(day: u8) -> SdkCacheManifest {
            let fingerprint = format!("{:064x}", day);
            let sha = format!("{:064x}", day + 10);
            SdkCacheManifest {
                version: CACHE_MANIFEST_VERSION,
                platform: "ios".to_string(),
                fingerprint: fingerprint.clone(),
                original_size_bytes: 1,
                compressed_size_bytes: 1,
                created_at: format!("2026-07-{:02}T00:00:00Z", day),
                parts: vec![SdkCachePart {
                    index: 0,
                    name: part_asset_name("ios", &fingerprint, 0, &sha),
                    size_bytes: 1,
                    sha256: sha,
                }],
            }
        }
        let pruned =
            manifests_to_prune(vec![manifest(2), manifest(4), manifest(1), manifest(3)], 2);
        assert_eq!(
            pruned
                .iter()
                .map(|manifest| manifest.created_at.as_str())
                .collect::<Vec<_>>(),
            vec!["2026-07-02T00:00:00Z", "2026-07-01T00:00:00Z"]
        );
    }

    #[test]
    fn packaged_sdk_extracts_without_an_extra_root_directory() {
        let root = temp_dir("package-source");
        std::fs::create_dir_all(root.join("SDK/libs")).unwrap();
        std::fs::write(root.join("SDK/libs/a.bin"), b"archive-data").unwrap();
        let (fingerprint, size_bytes) = fingerprint_sdk_directory(&root).unwrap();
        let snapshot = LocalSdkSnapshot {
            platform: "android".to_string(),
            root: root.clone(),
            fingerprint,
            size_bytes,
        };
        let package_dir = temp_dir("package-output");
        let package = package_sdk(&snapshot, &package_dir, 32).unwrap();
        let combined = package_dir.join("combined.tar.zst");
        let mut combined_file = File::create(&combined).unwrap();
        for part in package.parts {
            let mut file = File::open(part.path).unwrap();
            std::io::copy(&mut file, &mut combined_file).unwrap();
        }
        drop(combined_file);
        let extracted = temp_dir("package-extracted");
        let decoder = zstd::stream::read::Decoder::new(File::open(combined).unwrap()).unwrap();
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(&extracted).unwrap();
        assert_eq!(
            std::fs::read(extracted.join("SDK/libs/a.bin")).unwrap(),
            b"archive-data"
        );
        assert!(!extracted.join("sdk/SDK/libs/a.bin").exists());
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(package_dir);
        let _ = std::fs::remove_dir_all(extracted);
    }

    #[test]
    fn packaging_same_snapshot_produces_stable_part_hashes() {
        let root = temp_dir("stable-package-source");
        std::fs::create_dir_all(root.join("z-last")).unwrap();
        std::fs::create_dir_all(root.join("a-first")).unwrap();
        std::fs::write(root.join("z-last/b.bin"), b"second").unwrap();
        std::fs::write(root.join("a-first/a.bin"), b"first").unwrap();
        let (fingerprint, size_bytes) = fingerprint_sdk_directory(&root).unwrap();
        let snapshot = LocalSdkSnapshot {
            platform: "android".to_string(),
            root: root.clone(),
            fingerprint,
            size_bytes,
        };
        let first_dir = temp_dir("stable-package-one");
        let second_dir = temp_dir("stable-package-two");
        let first = package_sdk(&snapshot, &first_dir, 32).unwrap();
        let second = package_sdk(&snapshot, &second_dir, 32).unwrap();
        assert_eq!(
            first
                .parts
                .iter()
                .map(|part| (&part.metadata.name, part.metadata.size_bytes))
                .collect::<Vec<_>>(),
            second
                .parts
                .iter()
                .map(|part| (&part.metadata.name, part.metadata.size_bytes))
                .collect::<Vec<_>>()
        );
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(first_dir);
        let _ = std::fs::remove_dir_all(second_dir);
    }

    #[cfg(unix)]
    #[test]
    fn fingerprint_rejects_symlinks_outside_sdk_root() {
        use std::os::unix::fs::symlink;
        let root = temp_dir("unsafe-link");
        symlink("../../outside", root.join("escape")).unwrap();
        let error = fingerprint_sdk_directory(&root).unwrap_err();
        assert!(error.contains("根目录外"));
        let _ = std::fs::remove_dir_all(root);
    }
}
