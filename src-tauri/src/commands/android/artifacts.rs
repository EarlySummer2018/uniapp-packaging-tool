//! Android AAR 操作与模块产物管理

use crate::commands::android::types::emit_log;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn copy_required_aars(
    sdk_libs: &Path,
    libs_dst: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    for requirement in crate::commands::android::sdk_layout::ANDROID_REQUIRED_AARS {
        let src = crate::commands::android::sdk_layout::resolve_android_required_aar(
            sdk_libs,
            requirement,
        )
        .ok_or_else(|| format!("SDK 缺少必需 AAR: {}", requirement.display_name))?;
        let file_name = src
            .file_name()
            .ok_or_else(|| format!("SDK AAR 文件名无效: {}", src.display()))?;
        crate::utils::fs::copy_file(&src, &libs_dst.join(file_name))
            .map_err(|e| format!("复制 {} 失败: {}", requirement.display_name, e))?;
    }
    emit_log(
        window,
        "info",
        &format!(
            "已复制 {} 个基础 AAR",
            crate::commands::android::sdk_layout::ANDROID_REQUIRED_AARS.len()
        ),
        None,
    );
    Ok(())
}

pub fn copy_optional_aar(
    sdk_libs: &Path,
    libs_dst: &Path,
    aar_name: &str,
    window: &tauri::Window,
) -> Result<(), String> {
    let src = sdk_libs.join(aar_name);
    if src.exists() {
        crate::utils::fs::copy_file(&src, &libs_dst.join(aar_name))
            .map_err(|e| format!("复制 {} 失败: {}", aar_name, e))?;
    } else {
        emit_log(
            window,
            "warn",
            &format!("SDK 中未找到可选 AAR: {}", aar_name),
            None,
        );
    }
    Ok(())
}

/// 从构建参数中的模块配置提取华为推送的 agconnect-services.json 并写入 Android 工作区
pub fn inject_huawei_agconnect_json(
    module_config: &Option<std::collections::HashMap<String, String>>,
    workspace: &std::path::Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let Some(config) = module_config else {
        return Ok(());
    };

    let Some(base64_content) = config.get("HUAWEI_AGCONNECT_JSON") else {
        // 未配置文件，检查是否启用了华为推送（有 HUAWEI_APP_ID 说明启用了）
        if config.contains_key("HUAWEI_APP_ID") {
            emit_log(
                window,
                "warn",
                "华为推送已启用但未配置 agconnect-services.json，可能导致推送功能异常",
                None,
            );
        }
        return Ok(());
    };

    let base64_content = base64_content.trim();
    if base64_content.is_empty() {
        return Ok(());
    }

    // 自动检测输入格式：原始 JSON 直接使用，base64 编码则先解码
    let json_bytes: Vec<u8> = if base64_content.starts_with('{') {
        // 用户直接粘贴了原始 JSON 内容
        base64_content.as_bytes().to_vec()
    } else {
        super::manifest_modules::decode_base64(base64_content)
            .map_err(|e| format!("解码 agconnect-services.json 失败: {}", e))?
    };

    // 验证是合法 JSON
    let _: serde_json::Value = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("agconnect-services.json 不是有效的 JSON: {}", e))?;

    let dest = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src")
        .join("main")
        .join("agconnect-services.json");

    std::fs::write(&dest, &json_bytes)
        .map_err(|e| format!("写入 agconnect-services.json 失败: {}", e))?;

    emit_log(window, "info", "华为 agconnect-services.json 已注入", None);
    Ok(())
}

/// 应用 Android manifest 模块（内部实现，供 manifest_modules.rs 公开接口调用）
pub(crate) fn apply_android_manifest_modules_internal(
    modules: &[crate::commands::resource::DetectedModule],
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    manifest: Option<&serde_json::Value>,
    sdk_libs: &Path,
    libs_dst: &Path,
    workspace: &Path,
    extra_repos: &mut BTreeSet<String>,
    extra_deps: &mut BTreeSet<String>,
    window: &tauri::Window,
) -> Result<(), String> {
    let supported = modules
        .iter()
        .filter(|module| android_module_template_key(&module.name).is_some())
        .collect::<Vec<_>>();
    if supported.is_empty() {
        emit_log(
            window,
            "info",
            "manifest 中未检测到需要迁移的 Android UniApp 模块",
            None,
        );
        return Ok(());
    }

    let config =
        super::manifest_modules::module_config_tree_for_android_build(modules, config_report);
    let properties_path = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/assets/data/dcloud_properties.xml");
    crate::commands::module::generate_dcloud_properties(&properties_path, &config)?;

    for module in supported {
        emit_log(
            window,
            "info",
            &format!("检测到 {} 模块", module.name),
            None,
        );
        emit_log(
            window,
            "info",
            &format!("开始打包 {} 模块", module.name),
            None,
        );
        emit_log(
            window,
            "info",
            &format!("开始拷贝 {} 模块涉及的 aar 文件", module.name),
            None,
        );

        let template_key = android_module_template_key(&module.name)
            .expect("supported modules are filtered by template key");
        let template = crate::commands::module::get_module_template_sync(template_key)?;
        copy_android_module_artifacts(
            &module.name,
            template_key,
            &template.android_config.required_aars,
            manifest,
            sdk_libs,
            libs_dst,
            window,
        )?;
        // 复制厂商推送 SDK 的本地 AAR（仅当用户配置了对应厂商时才复制）
        if !template.android_config.vendor_aars.is_empty() {
            copy_android_module_artifacts(
                &module.name,
                template_key,
                &template.android_config.vendor_aars,
                manifest,
                sdk_libs,
                libs_dst,
                window,
            )?;
        }
        for repo in crate::commands::module::android_module_gradle_repositories_for_manifest(
            template_key,
            manifest,
        ) {
            extra_repos.insert(repo.to_string());
        }
        for dep in android_gradle_dependencies(
            template_key,
            &template.android_config.gradle_dependencies,
            manifest,
        ) {
            extra_deps.insert(dep);
        }

        emit_log(
            window,
            "info",
            &format!("{} 模块涉及的 aar 文件 拷贝完成", module.name),
            None,
        );
        emit_log(
            window,
            "info",
            &format!("开始修改 {} 模块需要的配置项", module.name),
            None,
        );
        emit_log(
            window,
            "success",
            &format!("{} 模块配置/打包完成", module.name),
            None,
        );
    }
    Ok(())
}

pub fn android_module_template_key(module_name: &str) -> Option<&'static str> {
    crate::commands::module::android_module_template_key(module_name)
}

fn copy_android_module_artifacts(
    module_name: &str,
    template_key: &str,
    required_artifacts: &[String],
    manifest: Option<&serde_json::Value>,
    sdk_libs: &Path,
    libs_dst: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let mut copied_names: Vec<String> = Vec::new();
    for artifact in required_artifacts {
        if !crate::commands::module::android_module_artifact_enabled_for_manifest(
            template_key,
            artifact,
            manifest,
        ) {
            continue;
        }
        let Some(pattern) = clean_android_artifact_pattern(artifact) else {
            continue;
        };
        let Some(src) = find_android_sdk_artifact(sdk_libs, &pattern) else {
            emit_log(
                window,
                "warn",
                &format!("{} 模块未找到可选依赖文件: {}", module_name, pattern),
                None,
            );
            continue;
        };
        let Some(file_name) = src.file_name() else {
            continue;
        };
        crate::utils::fs::copy_file(&src, &libs_dst.join(file_name)).map_err(|e| {
            format!(
                "复制 {} 模块依赖 {} 失败: {}",
                module_name,
                src.display(),
                e
            )
        })?;
        copied_names.push(file_name.to_string_lossy().to_string());
    }
    if !copied_names.is_empty() {
        emit_log(
            window,
            "info",
            &format!(
                "{} 模块已复制本地依赖: {}",
                module_name,
                copied_names.join(", ")
            ),
            None,
        );
    }
    Ok(())
}

pub fn clean_android_artifact_pattern(raw: &str) -> Option<String> {
    let name = raw.split_whitespace().next()?.trim();
    if name.is_empty() || name.starts_with('(') {
        return None;
    }
    if !(name.ends_with(".aar") || name.ends_with(".jar")) {
        return None;
    }
    Some(name.to_string())
}

pub fn find_android_sdk_artifact(sdk_libs: &Path, artifact_pattern: &str) -> Option<PathBuf> {
    let direct = sdk_libs.join(artifact_pattern);
    if direct.exists() {
        return Some(direct);
    }
    let stem = android_artifact_search_stem(artifact_pattern);
    let mut matches = std::fs::read_dir(sdk_libs)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let ext = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default();
            if ext != "aar" && ext != "jar" {
                return false;
            }
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            crate::commands::sdk::android_artifact_name_matches(artifact_pattern, file_name)
                || (!stem.is_empty() && (file_name.starts_with(&stem) || file_name.contains(&stem)))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

pub fn android_artifact_search_stem(pattern: &str) -> String {
    crate::commands::sdk::android_artifact_versionless_stem(pattern)
}

pub fn android_gradle_dependencies(
    template_key: &str,
    raw_deps: &[String],
    manifest: Option<&serde_json::Value>,
) -> Vec<String> {
    raw_deps
        .iter()
        .filter(|dep| {
            crate::commands::module::android_module_gradle_dependency_enabled_for_manifest(
                template_key,
                dep,
                manifest,
            )
        })
        .filter_map(|dep| dep.split_whitespace().next())
        .map(str::trim)
        .filter(|dep| dep.matches(':').count() >= 2)
        .map(ToString::to_string)
        .collect()
}
