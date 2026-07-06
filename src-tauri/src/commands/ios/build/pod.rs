use std::path::{Path, PathBuf};

use tauri::Manager;

use super::logging::emit_ios_log;
use super::pod_config::write_ios_pod_config;
use super::pod_subspecs::{resolve_ios_pod_subspecs, IosPodSubspecs};
use super::pod_xcode::{ensure_ios_pod_core_libraries, ensure_ios_pod_header_search_paths};
use crate::commands::ios::modules::uts_plugins::copy_ios_uts_plugins_for_pod;
use crate::commands::shared::resource_scan::ResourceScanResult;

#[derive(Debug, Clone)]
pub(super) struct IosPodIntegration {
    pub(super) workspace_file: PathBuf,
    pub(super) subspecs: Vec<String>,
}

pub(super) struct IosPodContext<'a> {
    pub(super) workspace: &'a Path,
    pub(super) project_root: &'a Path,
    pub(super) project_file: &'a Path,
    pub(super) sdk_root: &'a Path,
    pub(super) manifest_info: Option<&'a crate::commands::resource::UniappManifestInfo>,
    pub(super) scan: &'a ResourceScanResult,
    pub(super) window: &'a tauri::Window,
    pub(super) build_id: &'a str,
}

pub(super) async fn integrate_ios_pods(
    ctx: IosPodContext<'_>,
) -> Result<IosPodIntegration, String> {
    ensure_pod_ready_layout(&ctx)?;
    copy_uniapp_podspec(ctx.sdk_root, ctx.workspace)?;

    let copied_uts_count = copy_ios_uts_plugins_for_pod(ctx.project_root, &ctx.scan.uts)?;
    if copied_uts_count > 0 {
        emit_ios_log(
            ctx.window,
            ctx.build_id,
            "success",
            &format!(
                "已按 HBuilderX 5.13+ Pod 流程复制 iOS UTS 插件 app-ios 目录 {} 个",
                copied_uts_count
            ),
            Some(31),
        );
    }
    let uts_pod_count = ios_uts_plugin_pod_dependency_count(ctx.scan);
    if uts_pod_count > 0 {
        emit_ios_log(
            ctx.window,
            ctx.build_id,
            "info",
            &format!(
                "检测到 iOS UTS 插件声明 {} 个 Pod 依赖，将交由官方 Pod 脚本处理",
                uts_pod_count
            ),
            Some(31),
        );
    }

    let subspecs = resolve_ios_pod_subspecs(ctx.manifest_info, ctx.scan);
    write_ios_podfile(ctx.project_root, ctx.project_file, &subspecs.values)?;
    write_ios_pod_config(ctx.project_root, ctx.manifest_info)?;
    log_pod_selection(&ctx, &subspecs);
    log_manual_pod_followups(&ctx);

    run_pod_install(ctx.project_root, ctx.window, ctx.build_id).await?;
    let patched_header_search_paths = ensure_ios_pod_header_search_paths(ctx.project_file)?;
    if patched_header_search_paths > 0 {
        emit_ios_log(
            ctx.window,
            ctx.build_id,
            "success",
            "已为 Pod 模式补充 SDK/inc 头文件搜索路径",
            Some(44),
        );
    }
    let linked_core_libraries = ensure_ios_pod_core_libraries(ctx.project_file)?;
    if linked_core_libraries > 0 {
        emit_ios_log(
            ctx.window,
            ctx.build_id,
            "success",
            "已为 Pod 模式补充 DCloud Core 静态库链接",
            Some(44),
        );
    }
    let workspace_file = find_xcworkspace(ctx.project_root, ctx.project_file)?;
    emit_ios_log(
        ctx.window,
        ctx.build_id,
        "success",
        &format!("CocoaPods 已生成 workspace: {}", workspace_file.display()),
        Some(45),
    );

    Ok(IosPodIntegration {
        workspace_file,
        subspecs: subspecs.values,
    })
}

fn ensure_pod_ready_layout(ctx: &IosPodContext<'_>) -> Result<(), String> {
    let podspec = ctx.sdk_root.join("uniapp.podspec");
    if !podspec.is_file() {
        return Err(format!(
            "iOS 本地 Pod 模式需要 HBuilderX 5.13+ 离线 SDK，未找到 {}；请升级 iOS 离线 SDK 或改用自动迁移打包",
            podspec.display()
        ));
    }

    let script = ctx.project_root.join("scripts/uniapp_module_config.rb");
    let script_without_ext = ctx.project_root.join("scripts/uniapp_module_config");
    if !script.is_file() && !script_without_ext.is_file() {
        return Err(format!(
            "iOS 本地 Pod 模式需要 HBuilderX 5.13+ 示例工程脚本，未找到 {}；请升级 iOS 离线 SDK 或改用自动迁移打包",
            script.display()
        ));
    }
    Ok(())
}

pub(super) fn copy_uniapp_podspec(sdk_root: &Path, workspace: &Path) -> Result<(), String> {
    let source = sdk_root.join("uniapp.podspec");
    let target = workspace.join("uniapp.podspec");
    std::fs::copy(&source, &target).map_err(|e| {
        format!(
            "复制 uniapp.podspec 到 workspace 失败 {} -> {}: {}",
            source.display(),
            target.display(),
            e
        )
    })?;
    let license_source = sdk_root.join("license.md");
    if license_source.is_file() {
        let license_target = workspace.join("license.md");
        std::fs::copy(&license_source, &license_target).map_err(|e| {
            format!(
                "复制 license.md 到 workspace 失败 {} -> {}: {}",
                license_source.display(),
                license_target.display(),
                e
            )
        })?;
    }
    Ok(())
}

pub(super) fn write_ios_podfile(
    project_root: &Path,
    project_file: &Path,
    subspecs: &[String],
) -> Result<(), String> {
    let content = render_ios_podfile(project_file, subspecs)?;
    std::fs::write(project_root.join("Podfile"), content)
        .map_err(|e| format!("写入 Podfile 失败: {}", e))
}

pub(super) fn render_ios_podfile(
    project_file: &Path,
    subspecs: &[String],
) -> Result<String, String> {
    let project_name = project_file
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("无法识别 Xcode 工程名: {}", project_file.display()))?;
    let mut content = String::new();
    content.push_str("platform :ios, '13.0'\n");
    content.push_str(&format!(
        "project '{}'\n\n",
        ruby_single_quoted(project_name)
    ));
    content.push_str("require_relative 'scripts/uniapp_module_config'\n");
    content.push_str("require_relative 'uniapp_config' if File.exist?(File.join(__dir__, 'uniapp_config.rb'))\n\n");
    content.push_str("uniapp_subspecs = [\n");
    for subspec in subspecs {
        content.push_str(&format!("  '{}',\n", ruby_single_quoted(subspec)));
    }
    content.push_str("]\n\n");
    content.push_str("target 'HBuilder' do\n");
    content.push_str("  pod 'uniapp', :path => '..', :subspecs => uniapp_subspecs\n");
    content.push_str("end\n\n");
    content.push_str("post_install do |_installer|\n");
    content.push_str("  UniAppModuleConfig.apply(\n");
    content.push_str("    uniapp_subspecs,\n");
    content
        .push_str("    plist_values: defined?(UNIAPP_PLIST_VALUES) ? UNIAPP_PLIST_VALUES : {}\n");
    content.push_str("  )\n");
    content.push_str("end\n");
    Ok(content)
}

fn log_pod_selection(ctx: &IosPodContext<'_>, subspecs: &IosPodSubspecs) {
    emit_ios_log(
        ctx.window,
        ctx.build_id,
        "info",
        &format!("iOS 本地 Pod subspec: {}", subspecs.values.join("、")),
        Some(32),
    );
    for warning in &subspecs.warnings {
        emit_ios_log(ctx.window, ctx.build_id, "warn", warning, Some(33));
    }
}

fn log_manual_pod_followups(ctx: &IosPodContext<'_>) {
    for message in [
        "Pod 模式已跳过旧式手动链接/Embed 模块迁移，避免与 uniapp.podspec 重复链接",
        "证书/Profile、三方平台后台配置、AppDelegate 回调仍需按官方文档确认",
        "如启用 Firebase 统计或 FCM，请确认 GoogleService-Info.plist 已按官方文档放入工程",
        "如启用 UniAd-WM，请按 uni-AD 官方文档手工确认微信相关参数",
    ] {
        emit_ios_log(ctx.window, ctx.build_id, "info", message, Some(34));
    }
}

async fn run_pod_install(
    project_root: &Path,
    window: &tauri::Window,
    build_id: &str,
) -> Result<(), String> {
    emit_ios_log(
        window,
        build_id,
        "info",
        "执行 pod install --no-repo-update",
        Some(40),
    );
    let output = crate::utils::process::run_command_streaming_with_env_tagged(
        "pod",
        &["install".into(), "--no-repo-update".into()],
        &project_root.to_string_lossy(),
        &[],
        window.app_handle().clone(),
        "build-log",
        crate::utils::process::StreamLogMeta {
            build_id: build_id.to_string(),
            platform: "ios".to_string(),
        },
    )
    .await
    .map_err(|e| {
        format!(
            "执行 pod install --no-repo-update 失败，请确认已安装 CocoaPods: {}",
            e
        )
    })?;
    if output.success {
        Ok(())
    } else {
        Err(format!(
            "pod install --no-repo-update 失败，退出码: {:?}",
            output.exit_code
        ))
    }
}

fn find_xcworkspace(project_root: &Path, project_file: &Path) -> Result<PathBuf, String> {
    if let Some(stem) = project_file.file_stem().and_then(|name| name.to_str()) {
        let candidate = project_root.join(format!("{}.xcworkspace", stem));
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    std::fs::read_dir(project_root)
        .map_err(|e| format!("读取 iOS 工程目录失败 {}: {}", project_root.display(), e))?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("xcworkspace"))
        .ok_or_else(|| {
            format!(
                "pod install 完成后未找到 .xcworkspace，请检查 Pod 输出: {}",
                project_root.display()
            )
        })
}

fn ios_uts_plugin_pod_dependency_count(scan: &ResourceScanResult) -> usize {
    scan.uts
        .custom_plugins
        .iter()
        .map(|plugin| plugin.ios_dependencies_pods.len())
        .sum()
}

fn ruby_single_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}
