use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::Deserialize;
use unipack_tool_lib::cloud_runner;

const PAYLOAD_VERSION: u8 = 2;
const CACHE_MANIFEST_VERSION: u8 = 1;
const SDK_CHUNK_SIZE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudPayloadMetadata {
    version: u8,
    build_id: String,
    platform: String,
    project_id: String,
    project_config: cloud_runner::ProjectConfig,
    #[serde(default)]
    manifest_info: Option<serde_json::Value>,
    #[serde(default)]
    module_config: Option<HashMap<String, String>>,
    #[serde(default)]
    ios_packaging_mode: Option<String>,
    sdk_cache: SdkCacheReference,
    #[serde(default)]
    signing: CloudPayloadSigning,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SdkCacheReference {
    platform: String,
    release_tag: String,
    manifest_asset: String,
    fingerprint: String,
    archive_format: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudPayloadSigning {
    android_keystore_file: Option<String>,
    android_store_password: Option<String>,
    android_key_password: Option<String>,
    ios_certificate_file: Option<String>,
    ios_certificate_password: Option<String>,
    ios_provisioning_profile_file: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SdkCacheManifest {
    version: u8,
    platform: String,
    fingerprint: String,
    original_size_bytes: u64,
    compressed_size_bytes: u64,
    created_at: String,
    parts: Vec<SdkCachePart>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SdkCachePart {
    index: u32,
    name: String,
    size_bytes: u64,
    sha256: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!(
            "{}",
            serde_json::json!({
                "channel": "build-log",
                "event": { "level": "error", "message": redact_text(&error) }
            })
        );
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let payload_zip = env::args()
        .nth(1)
        .ok_or_else(|| "usage: unipack-cloud-build <payload.zip>".to_string())?;
    let payload_zip = PathBuf::from(payload_zip);
    if !payload_zip.is_file() {
        return Err(format!("payload 不存在: {}", payload_zip.display()));
    }
    let work_dir = env::current_dir()
        .map_err(|e| e.to_string())?
        .join(".unipack-cloud-work")
        .join(format!(
            "{}-{}",
            chrono::Utc::now().timestamp(),
            std::process::id()
        ));
    fs::create_dir_all(&work_dir).map_err(|e| e.to_string())?;
    unzip(&payload_zip, &work_dir)?;

    let metadata_path = work_dir.join("payload.json");
    let metadata_text =
        fs::read_to_string(&metadata_path).map_err(|e| format!("读取 payload.json 失败: {}", e))?;
    let mut metadata = parse_payload_metadata(&metadata_text)?;
    validate_payload(&metadata)?;
    log(
        "info",
        &metadata.build_id,
        &metadata.platform,
        "payload v2 校验通过",
        Some(3),
    );

    let sdk_root = ensure_sdk_cache(&metadata.sdk_cache, &work_dir)?;
    // The repository token is only needed to fetch the private SDK cache. Never
    // expose it to Gradle, CocoaPods, Xcode, or project-provided build scripts.
    env::remove_var("GH_TOKEN");
    env::remove_var("GITHUB_TOKEN");
    let resource_zip = work_dir.join("resource.zip");
    if !resource_zip.is_file() {
        return Err("payload v2 缺少 resource.zip".to_string());
    }
    let resource_dir = work_dir.join("resource");
    unzip(&resource_zip, &resource_dir)?;
    let output_dir = env::current_dir()
        .map_err(|e| e.to_string())?
        .join("cloud-output");
    fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    rebase_payload_paths(&mut metadata, &work_dir)?;

    match metadata.platform.as_str() {
        "android" => run_android(metadata, &sdk_root, &resource_dir, &work_dir, &output_dir),
        "ios" => run_ios(metadata, &sdk_root, &resource_dir, &work_dir, &output_dir),
        other => Err(format!("不支持的云构建平台: {}", other)),
    }
}

fn parse_payload_metadata(text: &str) -> Result<CloudPayloadMetadata, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("解析 payload.json 失败: {}", e))?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "云构建 payload 缺少有效的 version，请升级桌面端后重试".to_string())?;
    if version != PAYLOAD_VERSION as u64 {
        return Err(format!(
            "不支持云构建 payload v{}；当前 Runner 只接受 v{}，请升级桌面端后重试",
            version, PAYLOAD_VERSION
        ));
    }
    serde_json::from_value(value).map_err(|e| format!("解析 payload v2 失败: {}", e))
}

fn validate_payload(payload: &CloudPayloadMetadata) -> Result<(), String> {
    if payload.version != PAYLOAD_VERSION {
        return Err(format!(
            "不支持云构建 payload v{}；当前 Runner 只接受 v{}，请升级桌面端后重试",
            payload.version, PAYLOAD_VERSION
        ));
    }
    if !matches!(payload.platform.as_str(), "android" | "ios") {
        return Err(format!("payload 平台无效: {}", payload.platform));
    }
    if payload.sdk_cache.platform != payload.platform {
        return Err("payload 平台与 SDK 缓存平台不一致".to_string());
    }
    if payload.sdk_cache.release_tag != "unipack-sdk-cache-v1" {
        return Err("SDK 缓存 Release tag 无效".to_string());
    }
    if payload.sdk_cache.archive_format != "tar.zst" {
        return Err(format!(
            "不支持 SDK 缓存格式: {}",
            payload.sdk_cache.archive_format
        ));
    }
    validate_asset_name(&payload.sdk_cache.manifest_asset)?;
    validate_fingerprint(&payload.sdk_cache.fingerprint)?;
    if payload.sdk_cache.manifest_asset
        != sdk_manifest_asset_name(&payload.platform, &payload.sdk_cache.fingerprint)
    {
        return Err("SDK 缓存 manifest asset 与平台/指纹不匹配".to_string());
    }
    if payload.build_id.trim().is_empty() || payload.project_id.trim().is_empty() {
        return Err("payload buildId/projectId 不能为空".to_string());
    }
    Ok(())
}

fn rebase_payload_paths(
    payload: &mut CloudPayloadMetadata,
    payload_root: &Path,
) -> Result<(), String> {
    payload.project_config.local_path.clear();
    payload.project_config.output_dir.clear();
    if payload
        .project_config
        .app
        .icon1024
        .starts_with("manifest-assets/")
    {
        payload.project_config.app.icon1024 =
            payload_path(payload_root, &payload.project_config.app.icon1024)?
                .to_string_lossy()
                .to_string();
    }
    if let Some(path) = payload.signing.android_keystore_file.as_deref() {
        payload.project_config.android.keystore.path = payload_path(payload_root, path)?
            .to_string_lossy()
            .to_string();
    }
    if let Some(path) = payload.signing.ios_certificate_file.as_deref() {
        payload.project_config.ios.certificate = payload_path(payload_root, path)?
            .to_string_lossy()
            .to_string();
    }
    if let Some(path) = payload.signing.ios_provisioning_profile_file.as_deref() {
        payload.project_config.ios.provisioning_profile = payload_path(payload_root, path)?
            .to_string_lossy()
            .to_string();
    }
    if let Some(manifest) = payload.manifest_info.as_mut() {
        rebase_json_paths(manifest, payload_root)?;
    }
    rebase_config_map_paths(
        &mut payload.project_config.android_module_config,
        payload_root,
    )?;
    rebase_config_map_paths(&mut payload.project_config.ios_module_config, payload_root)?;
    if let Some(config) = payload.module_config.as_mut() {
        rebase_config_map_paths(config, payload_root)?;
    }
    Ok(())
}

fn rebase_config_map_paths(
    config: &mut HashMap<String, String>,
    payload_root: &Path,
) -> Result<(), String> {
    for value in config.values_mut() {
        if value.starts_with("manifest-assets/") || value.starts_with("signing/") {
            *value = payload_path(payload_root, value)?
                .to_string_lossy()
                .to_string();
        }
    }
    Ok(())
}

fn rebase_json_paths(value: &mut serde_json::Value, payload_root: &Path) -> Result<(), String> {
    match value {
        serde_json::Value::Object(map) => {
            for value in map.values_mut() {
                rebase_json_paths(value, payload_root)?;
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                rebase_json_paths(value, payload_root)?;
            }
        }
        serde_json::Value::String(path)
            if path.starts_with("manifest-assets/") || path.starts_with("signing/") =>
        {
            *path = payload_path(payload_root, path)?
                .to_string_lossy()
                .to_string();
        }
        _ => {}
    }
    Ok(())
}

fn run_android(
    payload: CloudPayloadMetadata,
    sdk_root: &Path,
    resource_dir: &Path,
    work_dir: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    let manifest_info = payload
        .manifest_info
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| format!("解析 Android manifestInfo 失败: {}", e))?;
    let store_password = payload
        .signing
        .android_store_password
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Android 云构建缺少 Store 密码".to_string())?;
    let key_password = payload
        .signing
        .android_key_password
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Android 云构建缺少 Key 密码".to_string())?;
    if payload
        .project_config
        .android
        .keystore
        .path
        .trim()
        .is_empty()
    {
        return Err("Android 云构建缺少 Keystore 文件".to_string());
    }
    let java_home = required_env_path("JAVA_HOME")?;
    let android_home = env::var_os("ANDROID_HOME")
        .or_else(|| env::var_os("ANDROID_SDK_ROOT"))
        .map(PathBuf::from)
        .ok_or_else(|| "Runner 缺少 ANDROID_HOME/ANDROID_SDK_ROOT".to_string())?;
    let gradle_user_home = env::var_os("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| work_dir.join("gradle-home"));
    fs::create_dir_all(&gradle_user_home).map_err(|e| e.to_string())?;
    let log_secrets = vec![store_password.clone(), key_password.clone()];
    let mut project_config = payload.project_config;
    project_config.android.keystore.has_store_password = true;
    project_config.android.keystore.has_key_password = true;
    let runtime = cloud_runner::AndroidHeadlessRuntime {
        project_config,
        sdk_root: sdk_root.to_path_buf(),
        environment: cloud_runner::AndroidBuildEnvironment {
            gradle_bin: PathBuf::new(),
            java_home,
            android_home,
            gradle_user_home,
        },
        workspace: work_dir.join("android-workspace"),
        output_dir: output_dir.to_path_buf(),
        store_password,
        key_password,
    };
    let sink: cloud_runner::SharedBuildEventSink =
        Arc::new(cloud_runner::JsonLineBuildEventSink::new(log_secrets));
    let mut context = cloud_runner::BuildContext::new_headless(
        payload.project_id,
        resource_dir.to_string_lossy().to_string(),
        payload.build_id,
        manifest_info,
        payload.module_config,
        runtime,
        sink.as_ref(),
    )?;
    context.inject_base_aars(sink.as_ref())?;
    context.process_modules_and_uts(sink.as_ref())?;
    context.apply_manifest_modules(sink.as_ref())?;
    context.render_patches(sink.as_ref())?;
    context.apply_modifications(sink.as_ref(), false)?;
    context.import_resources(sink.as_ref())?;
    context.finalize(sink.as_ref())?;
    run_async(context.execute_gradle_and_collect_with_sink(sink))?;
    Ok(())
}

fn run_ios(
    payload: CloudPayloadMetadata,
    sdk_root: &Path,
    resource_dir: &Path,
    work_dir: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Err("iOS 云构建必须运行在 macOS Runner".to_string());
    }
    let manifest_info = payload
        .manifest_info
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| format!("解析 iOS manifestInfo 失败: {}", e))?;
    let packaging_mode = match payload
        .ios_packaging_mode
        .as_deref()
        .unwrap_or("autoMigration")
    {
        "autoMigration" => cloud_runner::IosPackagingMode::AutoMigration,
        "localPod" => cloud_runner::IosPackagingMode::LocalPod,
        other => return Err(format!("无效 iOS 打包方式: {}", other)),
    };
    let certificate_password = payload
        .signing
        .ios_certificate_password
        .ok_or_else(|| "iOS 云构建缺少 P12 证书密码".to_string())?;
    if payload.project_config.ios.certificate.trim().is_empty()
        || payload
            .project_config
            .ios
            .provisioning_profile
            .trim()
            .is_empty()
    {
        return Err("iOS 云构建缺少 P12 证书或 provisioning profile".to_string());
    }
    let keychain = TemporaryKeychain::create(work_dir)?;
    let sink: cloud_runner::SharedBuildEventSink =
        Arc::new(cloud_runner::JsonLineBuildEventSink::new([
            certificate_password.clone(),
            keychain.password.clone(),
        ]));
    let mut project_config = payload.project_config;
    project_config.ios.has_certificate_password = true;
    let result = run_async(cloud_runner::build_ios_ipa_headless(
        project_config,
        sdk_root.to_path_buf(),
        resource_dir.to_string_lossy().to_string(),
        payload.build_id,
        manifest_info,
        packaging_mode,
        work_dir.join("ios-workspace"),
        output_dir.to_path_buf(),
        certificate_password,
        keychain.path.clone(),
        keychain.password.clone(),
        sink,
    ));
    drop(keychain);
    result.map(|_| ())
}

struct TemporaryKeychain {
    path: PathBuf,
    password: String,
}

impl TemporaryKeychain {
    fn create(work_dir: &Path) -> Result<Self, String> {
        let path = work_dir.join("unipack-build.keychain-db");
        let password = uuid::Uuid::new_v4().to_string();
        run_security(&[
            "create-keychain".into(),
            "-p".into(),
            password.clone(),
            path.to_string_lossy().to_string(),
        ])?;
        let keychain = Self { path, password };
        let setup = (|| {
            run_security(&[
                "set-keychain-settings".into(),
                "-lut".into(),
                "21600".into(),
                keychain.path.to_string_lossy().to_string(),
            ])?;
            run_security(&[
                "unlock-keychain".into(),
                "-p".into(),
                keychain.password.clone(),
                keychain.path.to_string_lossy().to_string(),
            ])?;
            run_security(&[
                "list-keychains".into(),
                "-d".into(),
                "user".into(),
                "-s".into(),
                keychain.path.to_string_lossy().to_string(),
            ])?;
            run_security(&[
                "default-keychain".into(),
                "-d".into(),
                "user".into(),
                "-s".into(),
                keychain.path.to_string_lossy().to_string(),
            ])
        })();
        if let Err(error) = setup {
            drop(keychain);
            return Err(error);
        }
        Ok(keychain)
    }
}

impl Drop for TemporaryKeychain {
    fn drop(&mut self) {
        let _ = Command::new("security")
            .arg("delete-keychain")
            .arg(&self.path)
            .status();
    }
}

fn run_security(args: &[String]) -> Result<(), String> {
    let output = Command::new("security")
        .args(args)
        .output()
        .map_err(|e| format!("启动 security 失败: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "配置临时 Keychain 失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn run_async<F, T>(future: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("创建异步运行时失败: {}", e))?
        .block_on(future)
}

fn ensure_sdk_cache(reference: &SdkCacheReference, work_dir: &Path) -> Result<PathBuf, String> {
    let cache_base = env::var_os("UNIPACK_SDK_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| work_dir.join("sdk-cache"));
    let cache_entry = cache_base
        .join(&reference.platform)
        .join(&reference.fingerprint);
    let sdk_root = cache_entry.join("sdk");
    let marker = cache_entry.join("manifest.json");
    if cache_marker_matches(&marker, reference)?
        && validate_extracted_tree(&sdk_root).is_ok()
        && validate_sdk_layout(&reference.platform, &sdk_root).is_ok()
    {
        log(
            "info",
            "sdk-cache",
            &reference.platform,
            &format!("命中 SDK 缓存 {}", reference.fingerprint),
            Some(8),
        );
        return Ok(sdk_root);
    }

    if cache_entry.exists() {
        fs::remove_dir_all(&cache_entry).map_err(|e| format!("清理无效 SDK 缓存失败: {}", e))?;
    }
    fs::create_dir_all(&cache_entry).map_err(|e| e.to_string())?;
    let download_dir = work_dir.join("sdk-download");
    if download_dir.exists() {
        fs::remove_dir_all(&download_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    download_release_asset(
        &reference.release_tag,
        &reference.manifest_asset,
        &download_dir,
    )?;
    let manifest_path = download_dir.join(&reference.manifest_asset);
    let mut manifest: SdkCacheManifest = serde_json::from_str(
        &fs::read_to_string(&manifest_path)
            .map_err(|e| format!("读取 SDK 缓存 manifest 失败: {}", e))?,
    )
    .map_err(|e| format!("解析 SDK 缓存 manifest 失败: {}", e))?;
    validate_sdk_manifest(&manifest, reference)?;
    manifest.parts.sort_by_key(|part| part.index);
    let archive = download_dir.join("sdk.tar.zst");
    let mut archive_file = File::create(&archive).map_err(|e| e.to_string())?;
    for part in &manifest.parts {
        download_release_asset(&reference.release_tag, &part.name, &download_dir)?;
        let path = download_dir.join(&part.name);
        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
        if metadata.len() != part.size_bytes {
            return Err(format!("SDK 分片大小校验失败: {}", part.name));
        }
        let actual = sha256_file(&path)?;
        if !actual.eq_ignore_ascii_case(&part.sha256) {
            return Err(format!("SDK 分片 SHA-256 校验失败: {}", part.name));
        }
        let mut input = File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut input, &mut archive_file).map_err(|e| e.to_string())?;
    }
    archive_file.flush().map_err(|e| e.to_string())?;
    drop(archive_file);
    if fs::metadata(&archive).map_err(|e| e.to_string())?.len() != manifest.compressed_size_bytes {
        return Err("SDK 合并压缩包大小与 manifest 不一致".to_string());
    }
    extract_tar_zst_safely(&archive, &sdk_root)?;
    validate_extracted_tree(&sdk_root)?;
    validate_sdk_layout(&reference.platform, &sdk_root)?;
    fs::write(
        &marker,
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    log(
        "success",
        "sdk-cache",
        &reference.platform,
        &format!("SDK 缓存准备完成 {}", reference.fingerprint),
        Some(15),
    );
    Ok(sdk_root)
}

fn validate_sdk_layout(platform: &str, root: &Path) -> Result<(), String> {
    match platform {
        "android" => cloud_runner::resolve_android_sdk_layout(root)
            .map(|_| ())
            .map_err(|error| format!("Android SDK 缓存布局无效: {}", error)),
        "ios" => {
            let root = cloud_runner::resolve_ios_sdk_root(root)
                .map_err(|error| format!("iOS SDK 缓存根目录无效: {}", error))?;
            cloud_runner::resolve_ios_sdk_project(&root)
                .map(|_| ())
                .map_err(|error| format!("iOS SDK 缓存布局无效: {}", error))
        }
        other => Err(format!("不支持的 SDK 缓存平台: {}", other)),
    }
}

fn cache_marker_matches(path: &Path, reference: &SdkCacheReference) -> Result<bool, String> {
    if !path.is_file() {
        return Ok(false);
    }
    let manifest: SdkCacheManifest = match fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
    {
        Some(manifest) => manifest,
        None => return Ok(false),
    };
    Ok(validate_sdk_manifest(&manifest, reference).is_ok())
}

fn validate_sdk_manifest(
    manifest: &SdkCacheManifest,
    reference: &SdkCacheReference,
) -> Result<(), String> {
    if manifest.version != CACHE_MANIFEST_VERSION
        || manifest.platform != reference.platform
        || manifest.fingerprint != reference.fingerprint
    {
        return Err("SDK 缓存 manifest 与 payload 引用不匹配".to_string());
    }
    if manifest.parts.is_empty() {
        return Err("SDK 缓存 manifest 没有分片".to_string());
    }
    let mut expected_index = 0u32;
    let mut total = 0u64;
    let mut names = HashSet::new();
    let mut parts = manifest.parts.iter().collect::<Vec<_>>();
    parts.sort_by_key(|part| part.index);
    for part in parts {
        if part.index != expected_index
            || part.size_bytes == 0
            || part.size_bytes > SDK_CHUNK_SIZE_BYTES
        {
            return Err("SDK 缓存分片 index 或大小无效".to_string());
        }
        expected_index += 1;
        validate_asset_name(&part.name)?;
        validate_fingerprint(&part.sha256)?;
        if part.name
            != sdk_part_asset_name(
                &manifest.platform,
                &manifest.fingerprint,
                part.index,
                &part.sha256,
            )
            || !names.insert(part.name.clone())
        {
            return Err("SDK 缓存分片名称与清单不匹配".to_string());
        }
        total = total
            .checked_add(part.size_bytes)
            .ok_or_else(|| "SDK 缓存分片总大小溢出".to_string())?;
    }
    if total != manifest.compressed_size_bytes {
        return Err("SDK 缓存分片总大小与 manifest 不一致".to_string());
    }
    Ok(())
}

fn sdk_manifest_asset_name(platform: &str, fingerprint: &str) -> String {
    format!("unipack-sdk-{}-{}.manifest.json", platform, fingerprint)
}

fn sdk_part_asset_name(platform: &str, fingerprint: &str, index: u32, sha256: &str) -> String {
    format!(
        "unipack-sdk-{}-{}.part-{:05}-{}.tar.zst",
        platform, fingerprint, index, sha256
    )
}

fn download_release_asset(tag: &str, asset: &str, destination: &Path) -> Result<(), String> {
    validate_asset_name(asset)?;
    let repository =
        env::var("GITHUB_REPOSITORY").map_err(|_| "Runner 缺少 GITHUB_REPOSITORY".to_string())?;
    let token = env::var("GH_TOKEN")
        .or_else(|_| env::var("GITHUB_TOKEN"))
        .map_err(|_| "Runner 缺少 GH_TOKEN/GITHUB_TOKEN".to_string())?;
    let status = Command::new("gh")
        .args([
            "release",
            "download",
            tag,
            "--repo",
            &repository,
            "--pattern",
            asset,
            "--dir",
        ])
        .arg(destination)
        .arg("--clobber")
        .env("GH_TOKEN", token)
        .status()
        .map_err(|e| format!("启动 gh 下载 SDK 缓存失败: {}", e))?;
    if !status.success() {
        return Err(format!("下载 SDK 缓存 Release asset 失败: {}", asset));
    }
    Ok(())
}

fn extract_tar_zst_safely(archive: &Path, destination: &Path) -> Result<(), String> {
    validate_tar_archive(archive)?;
    if destination.exists() {
        fs::remove_dir_all(destination).map_err(|e| format!("清理 SDK 解压目录失败: {}", e))?;
    }
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    let decoder = zstd::stream::read::Decoder::new(
        File::open(archive).map_err(|e| format!("打开 SDK tar.zst 失败: {}", e))?,
    )
    .map_err(|e| format!("读取 SDK zstd 流失败: {}", e))?;
    let mut tar = tar::Archive::new(decoder);
    tar.set_preserve_permissions(true);
    tar.set_preserve_mtime(true);
    let entries = tar
        .entries()
        .map_err(|e| format!("读取 SDK tar 条目失败: {}", e))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("读取 SDK tar 条目失败: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("读取 SDK tar 路径失败: {}", e))?
            .into_owned();
        if !entry
            .unpack_in(destination)
            .map_err(|e| format!("解压 SDK tar 条目 {} 失败: {}", path.display(), e))?
        {
            return Err(format!("SDK tar 条目超出目标目录: {}", path.display()));
        }
    }
    validate_extracted_tree(destination)
}

fn validate_tar_archive(archive: &Path) -> Result<(), String> {
    let decoder = zstd::stream::read::Decoder::new(
        File::open(archive).map_err(|e| format!("打开 SDK tar.zst 失败: {}", e))?,
    )
    .map_err(|e| format!("读取 SDK zstd 流失败: {}", e))?;
    let mut tar = tar::Archive::new(decoder);
    let entries = tar
        .entries()
        .map_err(|e| format!("读取 SDK tar 条目失败: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取 SDK tar 条目失败: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("读取 SDK tar 路径失败: {}", e))?
            .into_owned();
        validate_archive_path(&path)?;
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            let target = entry
                .link_name()
                .map_err(|e| format!("读取 SDK tar 链接目标失败: {}", e))?
                .ok_or_else(|| format!("SDK tar 链接缺少目标: {}", path.display()))?;
            validate_archive_link_target(&path, &target, entry_type.is_symlink())?;
        } else if !entry_type.is_file() && !entry_type.is_dir() {
            return Err(format!("SDK tar 包含不支持的特殊条目: {}", path.display()));
        }
    }
    Ok(())
}

fn validate_archive_path(path: &Path) -> Result<(), String> {
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(format!("SDK 压缩包包含不安全路径: {}", path.display()));
    }
    Ok(())
}

fn validate_archive_link_target(
    entry_path: &Path,
    target: &Path,
    target_is_relative_to_parent: bool,
) -> Result<(), String> {
    if target.is_absolute() {
        return Err(format!(
            "SDK tar 链接使用绝对目标: {} -> {}",
            entry_path.display(),
            target.display()
        ));
    }
    let base = if target_is_relative_to_parent {
        entry_path.parent().unwrap_or_else(|| Path::new(""))
    } else {
        Path::new("")
    };
    let mut depth = base
        .components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count();
    for component in target.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => {
                if depth == 0 {
                    return Err(format!(
                        "SDK tar 链接目标逃逸: {} -> {}",
                        entry_path.display(),
                        target.display()
                    ));
                }
                depth -= 1;
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "SDK tar 链接目标不安全: {} -> {}",
                    entry_path.display(),
                    target.display()
                ));
            }
        }
    }
    Ok(())
}

fn validate_extracted_tree(root: &Path) -> Result<(), String> {
    if !root.is_dir() {
        return Err("SDK 缓存目录不存在".to_string());
    }
    validate_tree_recursive(root, root)
}

fn validate_tree_recursive(root: &Path, dir: &Path) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(&path).map_err(|e| e.to_string())?;
            if target.is_absolute() {
                return Err(format!("SDK 缓存包含绝对符号链接: {}", path.display()));
            }
            let parent = path.parent().unwrap_or(root);
            let joined = normalize_lexically(&parent.join(target))?;
            let root = normalize_lexically(root)?;
            if !joined.starts_with(&root) {
                return Err(format!("SDK 缓存符号链接逃逸: {}", path.display()));
            }
        } else if metadata.is_dir() {
            validate_tree_recursive(root, &path)?;
        }
    }
    Ok(())
}

fn normalize_lexically(path: &Path) -> Result<PathBuf, String> {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return Err("路径逃逸".to_string());
                }
            }
            other => result.push(other.as_os_str()),
        }
    }
    Ok(result)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let output = if cfg!(target_os = "macos") {
        Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
    } else {
        Command::new("sha256sum").arg(path).output()
    }
    .map_err(|e| format!("计算 SHA-256 失败: {}", e))?;
    if !output.status.success() {
        return Err(format!("计算 SHA-256 失败: {}", path.display()));
    }
    let digest = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    validate_sha256(&digest)?;
    Ok(digest)
}

fn validate_asset_name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 200
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
    {
        return Err(format!("Release asset 名称不安全: {}", value));
    }
    Ok(())
}

fn validate_fingerprint(value: &str) -> Result<(), String> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err("SDK fingerprint 必须是小写 SHA-256".to_string())
    }
}

fn validate_sha256(value: &str) -> Result<(), String> {
    if value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("无效 SHA-256".to_string())
    }
}

fn payload_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(format!("payload 包含不安全路径: {}", relative.display()));
    }
    let path = root.join(relative);
    if !path.is_file() {
        return Err(format!("payload 引用文件不存在: {}", relative.display()));
    }
    Ok(path)
}

fn required_env_path(name: &str) -> Result<PathBuf, String> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .ok_or_else(|| format!("Runner 缺少有效的 {}", name))
}

fn log(level: &str, build_id: &str, platform: &str, message: &str, progress: Option<u8>) {
    println!(
        "{}",
        serde_json::json!({
            "channel": "build-log",
            "event": {
                "buildId": build_id,
                "platform": platform,
                "level": level,
                "message": redact_text(message),
                "progress": progress,
            }
        })
    );
}

fn redact_text(value: &str) -> String {
    let mut redacted = value.to_string();
    for secret in [env::var("GH_TOKEN").ok(), env::var("GITHUB_TOKEN").ok()]
        .into_iter()
        .flatten()
        .filter(|secret| !secret.is_empty())
    {
        redacted = redacted.replace(&secret, "***");
    }
    redacted
}

fn unzip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let enclosed = file
            .enclosed_name()
            .ok_or_else(|| format!("payload 包含不安全路径: {}", file.name()))?
            .to_path_buf();
        let out_path = dest_dir.join(enclosed);
        if file.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut output = File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut output).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            if let Some(mode) = file.unix_mode() {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode & 0o777))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_old_payload_version_message() {
        let error = parse_payload_metadata(r#"{"version":1,"buildId":"legacy"}"#).unwrap_err();
        assert!(error.contains("payload v1"));
        assert!(error.contains("升级桌面端"));
    }

    #[test]
    fn rejects_release_asset_path_traversal() {
        assert!(validate_asset_name("../part").is_err());
        assert!(validate_asset_name("parts/part-0").is_err());
        assert!(validate_asset_name("part-0000.zst").is_ok());
    }

    #[test]
    fn archive_paths_cannot_escape_destination() {
        assert!(validate_archive_path(Path::new("SDK/lib.a")).is_ok());
        assert!(validate_archive_path(Path::new("../outside")).is_err());
        assert!(validate_archive_path(Path::new("/tmp/outside")).is_err());
    }

    #[test]
    fn archive_link_targets_cannot_escape_destination() {
        assert!(validate_archive_link_target(
            Path::new("SDK/Versions/Current"),
            Path::new("A"),
            true,
        )
        .is_ok());
        assert!(validate_archive_link_target(
            Path::new("SDK/link"),
            Path::new("../../outside"),
            true,
        )
        .is_err());
        assert!(validate_archive_link_target(
            Path::new("SDK/hard-link"),
            Path::new("../outside"),
            false,
        )
        .is_err());
    }

    #[test]
    fn manifest_asset_paths_are_rebased_only_inside_payload() {
        let root = env::temp_dir().join(format!("unipack-rebase-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("manifest-assets")).unwrap();
        fs::write(root.join("manifest-assets/icon.png"), b"png").unwrap();
        let mut value = serde_json::json!({
            "icon": "manifest-assets/icon.png",
            "url": "https://example.com/icon.png"
        });
        rebase_json_paths(&mut value, &root).unwrap();
        assert!(value["icon"]
            .as_str()
            .unwrap()
            .starts_with(root.to_str().unwrap()));
        assert_eq!(value["url"], "https://example.com/icon.png");
        let mut config = HashMap::from([
            ("file".to_string(), "manifest-assets/icon.png".to_string()),
            ("remote".to_string(), "https://example.com/file".to_string()),
        ]);
        rebase_config_map_paths(&mut config, &root).unwrap();
        assert!(config["file"].starts_with(root.to_str().unwrap()));
        assert_eq!(config["remote"], "https://example.com/file");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn runner_manifest_rejects_parts_not_bound_to_fingerprint_and_hash() {
        let fingerprint = "a".repeat(64);
        let sha256 = "b".repeat(64);
        let reference = SdkCacheReference {
            platform: "android".to_string(),
            release_tag: "unipack-sdk-cache-v1".to_string(),
            manifest_asset: sdk_manifest_asset_name("android", &fingerprint),
            fingerprint: fingerprint.clone(),
            archive_format: "tar.zst".to_string(),
        };
        let valid_name = sdk_part_asset_name("android", &fingerprint, 0, &sha256);
        let mut manifest = SdkCacheManifest {
            version: CACHE_MANIFEST_VERSION,
            platform: "android".to_string(),
            fingerprint,
            original_size_bytes: 1,
            compressed_size_bytes: 1,
            created_at: "2026-07-13T00:00:00Z".to_string(),
            parts: vec![SdkCachePart {
                index: 0,
                name: valid_name,
                size_bytes: 1,
                sha256,
            }],
        };
        assert!(validate_sdk_manifest(&manifest, &reference).is_ok());
        manifest.parts[0].name = "unrelated-safe-name.tar.zst".to_string();
        assert!(validate_sdk_manifest(&manifest, &reference).is_err());
        manifest.parts[0].size_bytes = 0;
        assert!(validate_sdk_manifest(&manifest, &reference).is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn temporary_keychain_runs_fake_security_lifecycle() {
        use std::os::unix::fs::PermissionsExt;

        let root = env::temp_dir().join(format!("unipack-security-{}", uuid::Uuid::new_v4()));
        let bin = root.join("bin");
        let log_path = root.join("security.log");
        fs::create_dir_all(&bin).unwrap();
        let security = bin.join("security");
        fs::write(
            &security,
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$FAKE_SECURITY_LOG\"\n",
        )
        .unwrap();
        fs::set_permissions(&security, fs::Permissions::from_mode(0o755)).unwrap();

        let original_path = env::var_os("PATH");
        let mut paths = vec![bin.clone()];
        if let Some(path) = original_path.as_deref() {
            paths.extend(env::split_paths(path));
        }
        env::set_var("PATH", env::join_paths(paths).unwrap());
        env::set_var("FAKE_SECURITY_LOG", &log_path);

        let keychain = TemporaryKeychain::create(&root).unwrap();
        assert!(keychain.path.ends_with("unipack-build.keychain-db"));
        drop(keychain);

        if let Some(path) = original_path {
            env::set_var("PATH", path);
        } else {
            env::remove_var("PATH");
        }
        env::remove_var("FAKE_SECURITY_LOG");

        let log = fs::read_to_string(&log_path).unwrap();
        for command in [
            "create-keychain",
            "set-keychain-settings",
            "unlock-keychain",
            "list-keychains",
            "default-keychain",
            "delete-keychain",
        ] {
            assert!(log.contains(command), "missing security command: {command}");
        }
        let _ = fs::remove_dir_all(root);
    }
}
