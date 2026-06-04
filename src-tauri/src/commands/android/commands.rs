//! Android Tauri Command 入口 + UTS 插件处理

use std::collections::HashMap;
use std::path::Path;

use crate::commands::android::types::emit_log;

// ===== Tauri Command 入口 =====

#[tauri::command]
pub async fn prepare_android_build(
    options: super::types::AndroidBuildOptions,
) -> Result<super::types::BuildResult, String> {
    let env = super::environment::resolve_android_build_environment()?;
    let project_dir = Path::new(&options.project_path);
    if !project_dir.exists() {
        return Err(format!(
            "Project path does not exist: {}",
            options.project_path
        ));
    }
    Ok(super::types::BuildResult {
        success: true,
        output_path: None,
        logs: vec![
            "[prepare] Android build environment verified".to_string(),
            format!("[prepare] Gradle: {}", env.gradle_bin.display()),
            format!("[prepare] JAVA_HOME: {}", env.java_home.display()),
            format!("[prepare] ANDROID_HOME: {}", env.android_home.display()),
        ],
        duration_ms: 0,
        error: None,
    })
}

#[tauri::command]
pub async fn run_android_build(
    options: super::types::AndroidBuildOptions,
    app_handle: tauri::AppHandle,
) -> Result<super::types::BuildResult, String> {
    use crate::commands::android::environment::android_process_env;

    let start = std::time::Instant::now();
    let project_dir = Path::new(&options.project_path);
    let env = super::environment::resolve_android_build_environment()?;
    let mut args = Vec::new();
    if options.clean.unwrap_or(false) {
        args.push("clean".to_string());
    }
    args.push(format!(
        "assemble{}",
        options.variant.unwrap_or_else(|| "Debug".to_string())
    ));
    if let Some(extra) = options.extra_args {
        args.extend(extra);
    }
    let output = crate::utils::process::run_command_streaming_with_env(
        &env.gradle_bin.to_string_lossy(),
        &args,
        &project_dir.to_string_lossy(),
        &android_process_env(&env),
        app_handle,
        "android-build",
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(super::types::BuildResult {
        success: output.success,
        output_path: None,
        logs: output.logs,
        duration_ms: start.elapsed().as_millis() as u64,
        error: (!output.success).then(|| "Build failed".to_string()),
    })
}

/// 构建 Android APK（完整流程：准备工程 + Gradle 构建 + 收集产物）
#[tauri::command]
pub async fn build_android_apk(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    module_config: Option<HashMap<String, String>>,
    window: tauri::Window,
    _state: tauri::State<'_, super::types::AppState>,
) -> Result<super::types::BuildArtifact, String> {
    use super::build_pipeline::BuildContext;

    let mut ctx = BuildContext::new(
        project_id,
        resource_path,
        build_id,
        manifest_info,
        module_config,
        &window,
        /*resolve_env=*/ true,
    )?;
    ctx.inject_base_aars(&window)?;
    ctx.process_modules_and_uts(&window)?;
    ctx.apply_manifest_modules(&window)?;
    ctx.render_patches(&window)?;
    ctx.apply_modifications(&window, /*tolerant_passwords=*/ false)?;
    ctx.import_resources(&window)?;
    ctx.finalize(&window)?;
    ctx.execute_gradle_and_collect(&window).await
}

/// 生成安卓工程（仅准备工程，不执行 Gradle 构建）
#[tauri::command]
pub async fn generate_android_project(
    project_id: String,
    resource_path: String,
    build_id: Option<String>,
    manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    module_config: Option<HashMap<String, String>>,
    window: tauri::Window,
    _state: tauri::State<'_, super::types::AppState>,
) -> Result<String, String> {
    use super::build_pipeline::BuildContext;

    let mut ctx = BuildContext::new(
        project_id,
        resource_path,
        build_id,
        manifest_info,
        module_config,
        &window,
        /*resolve_env=*/ false,
    )?;
    ctx.inject_base_aars(&window)?;
    ctx.process_modules_and_uts(&window)?;
    ctx.apply_manifest_modules(&window)?;
    ctx.render_patches(&window)?;
    ctx.apply_modifications(&window, /*tolerant_passwords=*/ true)?;
    ctx.import_resources(&window)?;
    ctx.finalize(&window)?;

    let workspace_display = ctx.workspace_path();
    emit_log(
        &window,
        "success",
        &format!("Android 工程已生成: {}", workspace_display),
        Some(100),
    );
    Ok(workspace_display)
}

// Manifest 补丁渲染已提取到独立兄弟模块（在 android/mod.rs 中声明）
