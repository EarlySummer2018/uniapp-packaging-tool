//! 自定义 UTS 插件处理主逻辑
//!
//! 包含自定义插件的资源复制、dcloud_uniplugins.json 生成、
//! AndroidManifest 清理等核心流程

use std::collections::BTreeSet;
use std::path::Path;

use crate::commands::android::types::{emit_log, UnpackedAarInfo};
use crate::commands::shared::resource_scan::UtsCustomPlugin;

use super::aar_unpack::{is_nonstandard_aar, unpack_nonstandard_aar};
use super::gradle::generate_uts_plugin_build_gradle;

/// 处理所有自定义 UTS 插件：复制源码、解包非标准 AAR、生成 build.gradle、收集依赖
#[allow(clippy::too_many_arguments)]
pub fn process_custom_uts_plugins_uniapp(
    custom_plugins: &[UtsCustomPlugin],
    workspace: &Path,
    main_libs: &Path,
    extra_repos: &mut BTreeSet<String>,
    extra_deps: &mut BTreeSet<String>,
    plugin_includes: &mut BTreeSet<String>,
    plugin_project_deps: &mut BTreeSet<String>,
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<(), String> {
    let custom_root = workspace.join("uts-modules");
    crate::utils::fs::ensure_directory(&custom_root).map_err(|e| e.to_string())?;

    for plugin in custom_plugins {
        let Some(android_dir) = &plugin.android_dir else {
            continue;
        };
        let module_dir = custom_root.join(&plugin.id);

        crate::utils::fs::copy_recursive(Path::new(android_dir), &module_dir)
            .map_err(|e| format!("复制 UTS 插件 {} 失败: {}", plugin.id, e))?;

        // 检测并预处理非标准 AAR：解包为散落文件以绕过 Jetifier 转换问题
        let mut unpacked_aars: Vec<UnpackedAarInfo> = Vec::new();
        let mut unpacked_names: Vec<String> = Vec::new();
        for libs_name in ["libs", "lib"] {
            let libs_src = module_dir.join(libs_name);
            if !libs_src.is_dir() {
                continue;
            }
            if let Ok(aar_files) = crate::utils::fs::find_files_by_extension(&libs_src, "aar") {
                for aar_path in &aar_files {
                    match is_nonstandard_aar(aar_path) {
                        Ok(true) => {
                            match unpack_nonstandard_aar(aar_path, &module_dir, main_libs, window) {
                                Ok(info) => {
                                    unpacked_names.push(info.original_name.clone());
                                    unpacked_aars.push(info);
                                }
                                Err(e) => {
                                    emit_log(
                                        window,
                                        "warn",
                                        &format!(
                                            "非标准AAR {} 解包失败，将原样使用: {}",
                                            aar_path.display(),
                                            e
                                        ),
                                        None,
                                    );
                                }
                            }
                        }
                        Ok(false) => {}
                        Err(e) => {
                            emit_log(
                                window,
                                "warn",
                                &format!("AAR 结构检测失败，跳过: {}", e),
                                None,
                            );
                        }
                    }
                }
            }
        }

        copy_uts_plugin_resources(plugin, &module_dir, main_libs, window, &unpacked_names)?;
        generate_uts_plugin_build_gradle(plugin, &module_dir, &unpacked_aars)?;

        if !plugin.gradle_plugins.is_empty() || !plugin.project_dependencies.is_empty() {
            emit_log(
                window,
                "info",
                &format!("插件 {} 需要项目级Gradle配置", plugin.id),
                None,
            );
        }
        if !plugin.dependencies.is_empty() {
            extra_repos.insert("maven { url 'https://jitpack.io' }".to_string());
        }

        plugin_includes.insert(format!(
            "include ':{0}'\nproject(':{0}').projectDir = file('uts-modules/{0}')",
            plugin.id
        ));
        plugin_project_deps.insert(format!("implementation project(':{}')", plugin.id));

        for dep in &plugin.android_deps {
            extra_deps.insert(dep.clone());
        }
    }

    emit_log(
        window,
        "success",
        &format!("已处理 {} 个UTS自定义插件", custom_plugins.len()),
        Some(26),
    );
    Ok(())
}

/// 复制 UTS 插件的本地库文件（jar/aar）、源码、assets、res 到模块目录和主模块
fn copy_uts_plugin_resources(
    plugin: &UtsCustomPlugin,
    module_dir: &Path,
    main_libs: &Path,
    window: &dyn crate::utils::process::BuildEventSink,
    unpacked_aar_names: &[String],
) -> Result<(), String> {
    let main = module_dir.join("src/main");
    let mut copied_libs: Vec<String> = Vec::new();

    for libs_name in ["libs", "lib"] {
        let libs_src = module_dir.join(libs_name);
        if !libs_src.is_dir() {
            continue;
        }
        let mod_libs = module_dir.join("libs");
        crate::utils::fs::ensure_directory(&mod_libs).map_err(|e| e.to_string())?;
        crate::utils::fs::ensure_directory(main_libs).map_err(|e| e.to_string())?;
        for ext in ["aar", "jar"] {
            for f in crate::utils::fs::find_files_by_extension(&libs_src, ext)
                .map_err(|e| format!("扫描{}库失败: {}", plugin.id, e))?
            {
                let name = f
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                // 跳过已解包的非标准 AAR（其内容已被拆散到 classes.jar / jniLibs / res 等）
                if unpacked_aar_names.iter().any(|n| n == &name) {
                    continue;
                }
                copied_libs.push(name.clone());
                let mod_dst = mod_libs.join(&name);
                // 跳过自拷贝（src 与 dst 为同一路径时 std::fs::copy 会截断文件为空）
                if f != mod_dst {
                    crate::utils::fs::copy_file(&f, &mod_dst)
                        .map_err(|e| format!("复制插件库失败: {}", e))?;
                }
                crate::utils::fs::copy_file(&f, &main_libs.join(&name))
                    .map_err(|e| format!("复制插件库到主模块失败: {}", e))?;
            }
        }
    }

    if !copied_libs.is_empty() {
        emit_log(
            window,
            "info",
            &format!(
                "UTS插件 {} 已复制本地依赖: {}",
                plugin.id,
                copied_libs.join(", ")
            ),
            None,
        );
    }

    if module_dir.join("assets").is_dir() {
        crate::utils::fs::copy_recursive(&module_dir.join("assets"), &main.join("assets"))
            .map_err(|e| format!("复制插件{} assets失败: {}", plugin.id, e))?;
    }

    if module_dir.join("res").is_dir() {
        crate::utils::fs::copy_recursive(&module_dir.join("res"), &main.join("res"))
            .map_err(|e| format!("复制插件{} res失败: {}", plugin.id, e))?;
    }

    copy_uts_plugin_source_files(plugin, module_dir, window)?;

    let mf = module_dir.join("AndroidManifest.xml");
    if mf.exists() && !main.join("AndroidManifest.xml").exists() {
        copy_and_clean_android_manifest(&mf, &main.join("AndroidManifest.xml"))?;
    }
    if main.join("AndroidManifest.xml").exists() {
        clean_android_manifest_package(&main.join("AndroidManifest.xml"))?;
    }

    Ok(())
}

/// 将 HBuilderX 生成的 UTS Kotlin/Java 源码复制到 Android 标准 source set。
fn copy_uts_plugin_source_files(
    plugin: &UtsCustomPlugin,
    module_dir: &Path,
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<(), String> {
    let src = module_dir.join("src");
    if !src.is_dir() {
        return Ok(());
    }

    let dst = module_dir.join("src/main/java");
    let mut copied = 0usize;
    copy_uts_source_tree(&src, &src, &dst, &mut copied)
        .map_err(|e| format!("复制插件{} src失败: {}", plugin.id, e))?;

    if copied > 0 {
        emit_log(
            window,
            "info",
            &format!("UTS插件 {} 已复制原生源码: {} 个文件", plugin.id, copied),
            None,
        );
    } else if crate::utils::fs::find_files_by_extension(module_dir, "uts")
        .map(|files| !files.is_empty())
        .unwrap_or(false)
    {
        emit_log(
            window,
            "warn",
            &format!(
                "UTS插件 {} 仅发现 .uts 源码，未发现 HBuilderX 编译后的 Kotlin/Java 产物",
                plugin.id
            ),
            None,
        );
    }

    Ok(())
}

fn copy_uts_source_tree(
    root: &Path,
    current: &Path,
    dst_root: &Path,
    copied: &mut usize,
) -> Result<(), String> {
    for entry in std::fs::read_dir(current).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let relative = path.strip_prefix(root).map_err(|e| e.to_string())?;

        if relative
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            == Some("main")
        {
            continue;
        }

        if path.is_dir() {
            copy_uts_source_tree(root, &path, dst_root, copied)?;
            continue;
        }

        let is_source = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "kt" | "java"))
            .unwrap_or(false);
        if !is_source {
            continue;
        }

        let dst = dst_root.join(relative);
        crate::utils::fs::copy_file(&path, &dst).map_err(|e| e.to_string())?;
        *copied += 1;
    }
    Ok(())
}

/// 复制 AndroidManifest.xml 并移除 package 声明以避免合并冲突
fn copy_and_clean_android_manifest(src: &Path, dst: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(src).map_err(|e| e.to_string())?;
    let cleaned = clean_android_manifest_package_content(&content);
    std::fs::write(dst, cleaned.trim()).map_err(|e| e.to_string())
}

fn clean_android_manifest_package(path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cleaned = clean_android_manifest_package_content(&content);
    if cleaned != content {
        std::fs::write(path, cleaned.trim()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn clean_android_manifest_package_content(content: &str) -> String {
    extract_package(content)
        .map(|package_name| content.replace(&format!("package=\"{}\"", package_name), ""))
        .unwrap_or_else(|| content.to_string())
}

/// 从 AndroidManifest XML 内容中提取 package 属性值
fn extract_package(manifest_content: &str) -> Option<String> {
    manifest_content.find("package=\"").and_then(|start| {
        let s = start + "package=\"".len();
        manifest_content[s..]
            .find('"')
            .map(|e| manifest_content[s..s + e].to_string())
    })
}

/// 生成 dcloud_uniplugins.json 文件，注册 UTS 原生组件
pub fn generate_dcloud_uniplugins_json(
    plugins: &[UtsCustomPlugin],
    workspace: &Path,
) -> Result<(), String> {
    let all_components: Vec<&crate::commands::resource::UtsComponent> =
        plugins.iter().flat_map(|p| p.components.iter()).collect();

    if all_components.is_empty() {
        return Ok(());
    }

    let native_plugins: Vec<serde_json::Value> = plugins
        .iter()
        .filter(|p| !p.components.is_empty())
        .map(|p| {
            let plugin_components: Vec<serde_json::Value> = p
                .components
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "type": "component",
                        "name": c.name,
                        "class": c.class
                    })
                })
                .collect();
            serde_json::json!({ "plugins": plugin_components })
        })
        .collect();

    let json = serde_json::json!({ "nativePlugins": native_plugins });
    let path = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/assets/dcloud_uniplugins.json");

    std::fs::write(
        &path,
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )
    .map_err(|e| format!("写入 dcloud_uniplugins.json 失败: {}", e))
}
