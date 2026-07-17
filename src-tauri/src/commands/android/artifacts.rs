//! Android AAR 操作与模块产物管理

use crate::commands::android::types::emit_log;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn copy_required_aars(
    sdk_libs: &Path,
    libs_dst: &Path,
    window: &dyn crate::utils::process::BuildEventSink,
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
    window: &dyn crate::utils::process::BuildEventSink,
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
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<(), String> {
    let Some(config) = module_config else {
        return Ok(());
    };

    let Some(base64_content) = config
        .get("push.HUAWEI_AGCONNECT_JSON")
        .or_else(|| config.get("HUAWEI_AGCONNECT_JSON"))
    else {
        // 未配置文件，检查是否启用了华为推送（有 HUAWEI_APP_ID 说明启用了）
        if config.contains_key("push.HUAWEI_APP_ID") || config.contains_key("HUAWEI_APP_ID") {
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_android_manifest_modules_internal(
    modules: &[crate::commands::resource::DetectedModule],
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    manifest: Option<&serde_json::Value>,
    sdk_libs: &Path,
    libs_dst: &Path,
    workspace: &Path,
    extra_repos: &mut BTreeSet<String>,
    extra_deps: &mut BTreeSet<String>,
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<(), String> {
    let supported = modules
        .iter()
        .filter(|module| {
            crate::commands::shared::module::templates::module_applies_to_android(&module.platforms)
                && android_module_template_key(&module.name).is_some()
        })
        .filter(|module| {
            android_module_template_key(&module.name) != Some("push")
                || manifest.is_some_and(|manifest| {
                    crate::commands::module::manifest_push_unipush_v2_enabled(manifest)
                })
        })
        .cloned()
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
    if crate::commands::module::android_amap_map_enabled(manifest)
        && crate::commands::module::android_amap_geolocation_enabled(manifest)
    {
        emit_log(
            window,
            "info",
            "同时检测到高德地图与高德定位，按离线 SDK 要求跳过独立高德定位 SDK，复用高德地图定位能力",
            None,
        );
    }

    let config = super::manifest_modules::module_config_tree_for_android_build(
        &supported,
        config_report,
        manifest,
    );

    // 构建白名单：只在 detected_modules 中且 template key 匹配的模块名
    let enabled_module_names: Vec<String> = supported.iter().map(|m| m.name.clone()).collect();

    let properties_path = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/assets/data/dcloud_properties.xml");
    crate::commands::module::generate_dcloud_properties(
        &properties_path,
        &config,
        &enabled_module_names,
    )?;

    let mut processed_modules = 0usize;
    let mut copied_artifacts = 0usize;
    for module in &supported {
        let template_key = android_module_template_key(&module.name)
            .expect("supported modules are filtered by template key");
        let template = crate::commands::module::get_module_template_sync(template_key)?;

        let required_artifacts = enabled_android_artifact_patterns(
            template_key,
            &template.android_config.required_aars,
            manifest,
            config_report,
        );
        let vendor_artifacts = enabled_android_artifact_patterns(
            template_key,
            &template.android_config.vendor_aars,
            manifest,
            config_report,
        );
        let gradle_dependencies = android_gradle_dependencies(
            template_key,
            &template.android_config.gradle_dependencies,
            manifest,
        );
        let gradle_dependencies = android_gradle_dependencies_for_report(
            template_key,
            gradle_dependencies,
            config_report,
        );
        if gradle_dependencies
            .iter()
            .any(|dependency| dependency.starts_with("com.amap.api:"))
        {
            remove_conflicting_amap_sdk_aars(libs_dst, window)?;
        }
        copied_artifacts += copy_android_module_artifacts(
            &module.name,
            &required_artifacts,
            sdk_libs,
            libs_dst,
            window,
        )?;
        // 复制厂商推送 SDK 的本地 AAR（仅当用户配置了对应厂商时才复制）
        if !vendor_artifacts.is_empty() {
            copied_artifacts += copy_android_module_artifacts(
                &module.name,
                &vendor_artifacts,
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
        for dep in gradle_dependencies {
            insert_gradle_dependency(extra_deps, dep);
        }
        processed_modules += 1;
    }
    emit_log(
        window,
        "success",
        &format!(
            "已完成 {} 个 Android 模块处理，复制 {} 个本地依赖",
            processed_modules, copied_artifacts
        ),
        None,
    );
    Ok(())
}

pub fn android_module_template_key(module_name: &str) -> Option<&'static str> {
    crate::commands::module::android_module_template_key(module_name)
}

fn copy_android_module_artifacts(
    module_name: &str,
    artifact_patterns: &[String],
    sdk_libs: &Path,
    libs_dst: &Path,
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<usize, String> {
    let mut copied_count = 0usize;
    for pattern in artifact_patterns {
        let Some(src) = find_android_sdk_artifact(sdk_libs, pattern) else {
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
        copied_count += 1;
    }
    Ok(copied_count)
}

fn enabled_android_artifact_patterns(
    template_key: &str,
    artifacts: &[String],
    manifest: Option<&serde_json::Value>,
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
) -> Vec<String> {
    artifacts
        .iter()
        .filter(|artifact| {
            crate::commands::module::android_module_artifact_enabled_for_manifest(
                template_key,
                artifact,
                manifest,
            )
        })
        .filter(|artifact| {
            android_module_artifact_enabled_for_report(template_key, artifact, config_report)
        })
        .filter_map(|artifact| clean_android_artifact_pattern(artifact))
        .collect()
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

fn android_gradle_dependencies_for_report(
    template_key: &str,
    dependencies: Vec<String>,
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
) -> Vec<String> {
    match template_key {
        "map" => {
            let Some(amap_version) = report_field_value(config_report, "map", "AMAP_SDK_VERSION")
            else {
                return dependencies;
            };

            let mut inserted_amap_map_dependency = false;
            dependencies
                .into_iter()
                .filter_map(|dependency| {
                    if dependency.starts_with("com.amap.api:3dmap-location-search:")
                        || dependency.starts_with("com.amap.api:3dmap:")
                        || dependency.starts_with("com.amap.api:search:")
                    {
                        if inserted_amap_map_dependency {
                            None
                        } else {
                            inserted_amap_map_dependency = true;
                            Some(format!(
                                "com.amap.api:3dmap-location-search:{}",
                                amap_version
                            ))
                        }
                    } else {
                        Some(dependency)
                    }
                })
                .collect()
        }
        "geolocation" => {
            let Some(tencent_version) =
                report_field_value(config_report, "geolocation", "TENCENT_LOCATION_SDK_VERSION")
            else {
                return dependencies;
            };
            dependencies
                .into_iter()
                .map(|dependency| {
                    if dependency
                        .starts_with("com.tencent.map.geolocation:TencentLocationSdk-openplatform:")
                    {
                        format!(
                            "com.tencent.map.geolocation:TencentLocationSdk-openplatform:{}",
                            tencent_version
                        )
                    } else {
                        dependency
                    }
                })
                .collect()
        }
        _ => dependencies,
    }
}

fn insert_gradle_dependency(extra_deps: &mut BTreeSet<String>, dependency: String) {
    if dependency.starts_with("com.tencent.map.geolocation:TencentLocationSdk-openplatform:") {
        extra_deps.retain(|existing| {
            !existing.starts_with("com.tencent.map.geolocation:TencentLocationSdk-openplatform:")
        });
    }
    extra_deps.insert(dependency);
}

fn report_field_value(
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    template_key: &str,
    field_key: &str,
) -> Option<String> {
    config_report.and_then(|report| {
        report
            .modules
            .iter()
            .find(|module| module.template_key == template_key)
            .and_then(|module| {
                module
                    .fields
                    .iter()
                    .find(|field| field.key == field_key)
                    .and_then(|field| field.value.as_deref())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
            })
    })
}

fn android_module_artifact_enabled_for_report(
    template_key: &str,
    artifact: &str,
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
) -> bool {
    if template_key != "map" {
        return true;
    }

    let note = android_entry_provider_note(artifact);
    if !android_entry_mentions_any(&note, &["amap", "gaode", "高德"]) {
        return true;
    }

    let page_type = map_page_type_for_report(config_report);
    let normalized_note = note.to_ascii_lowercase();
    if normalized_note.contains("nvue") {
        return page_type == "nvue";
    }
    if normalized_note.contains("vue") {
        return page_type == "vue";
    }
    true
}

fn map_page_type_for_report(
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
) -> &'static str {
    config_report
        .and_then(|report| {
            report
                .modules
                .iter()
                .find(|module| module.template_key == "map")
                .and_then(|module| {
                    module
                        .fields
                        .iter()
                        .find(|field| field.key == "MAP_PAGE_TYPE")
                        .and_then(|field| field.value.as_deref())
                        .map(str::trim)
                })
        })
        .filter(|value| value.eq_ignore_ascii_case("nvue"))
        .map(|_| "nvue")
        .unwrap_or("vue")
}

fn android_entry_provider_note(entry: &str) -> String {
    let mut notes = Vec::new();
    let mut rest = entry;
    while let Some(start) = rest.find('(') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find(')') else {
            break;
        };
        notes.push(after_start[..end].trim());
        rest = &after_start[end + 1..];
    }
    if notes.is_empty() {
        entry.to_string()
    } else {
        notes.join(" ")
    }
}

fn android_entry_mentions_any(text: &str, markers: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    let normalized = crate::commands::shared::module::parsing::normalize_config_key(text);
    markers.iter().any(|marker| {
        let marker_lower = marker.to_ascii_lowercase();
        let marker_normalized =
            crate::commands::shared::module::parsing::normalize_config_key(marker);
        lower.contains(&marker_lower)
            || (!marker_normalized.is_empty() && normalized.contains(&marker_normalized))
    })
}

fn remove_conflicting_amap_sdk_aars(
    libs_dst: &Path,
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<(), String> {
    if !libs_dst.is_dir() {
        return Ok(());
    }

    let mut removed = Vec::new();
    let entries = std::fs::read_dir(libs_dst)
        .map_err(|e| format!("扫描 Android 工程 libs 失败 {}: {}", libs_dst.display(), e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !is_conflicting_amap_sdk_aar(file_name) {
            continue;
        }
        std::fs::remove_file(&path)
            .map_err(|e| format!("删除冲突的高德本地 SDK {} 失败: {}", path.display(), e))?;
        removed.push(file_name.to_string());
    }

    if !removed.is_empty() {
        removed.sort();
        emit_log(
            window,
            "info",
            &format!(
                "已删除与 Maven 高德 SDK 冲突的本地 AAR: {}",
                removed.join(", ")
            ),
            None,
        );
    }
    Ok(())
}

fn is_conflicting_amap_sdk_aar(file_name: &str) -> bool {
    if !file_name.to_ascii_lowercase().ends_with(".aar") {
        return false;
    }
    let normalized = file_name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>();
    if !normalized.contains("amap") {
        return false;
    }
    !["geolocationamap", "mapamap", "weexamap"]
        .iter()
        .any(|wrapper| normalized.contains(wrapper))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amap_wrapper_aars_are_preserved_while_vendor_sdk_aars_are_removed() {
        assert!(!is_conflicting_amap_sdk_aar("geolocation-amap-release.aar"));
        assert!(!is_conflicting_amap_sdk_aar("map-amap-release.aar"));
        assert!(!is_conflicting_amap_sdk_aar("weex_amap-release.aar"));
        assert!(is_conflicting_amap_sdk_aar("AMap_Location_V6.4.5.aar"));
        assert!(is_conflicting_amap_sdk_aar("amap-libs-release.aar"));
    }

    #[test]
    fn amap_map_artifacts_follow_configured_page_type() {
        let report = crate::commands::module::AndroidModuleConfigReport {
            modules: vec![crate::commands::module::AndroidModuleConfigModule {
                name: "Maps".to_string(),
                template_key: "map".to_string(),
                category: "map".to_string(),
                platforms: vec!["android".to_string()],
                source: "test".to_string(),
                fields: vec![
                    crate::commands::shared::module::types::AndroidModuleConfigField {
                        key: "MAP_PAGE_TYPE".to_string(),
                        label: "地图页面类型".to_string(),
                        required: false,
                        secret: false,
                        value: Some("nvue".to_string()),
                        value_source: Some("user".to_string()),
                        placeholder: String::new(),
                        field_type: "select".to_string(),
                    },
                ],
            }],
            missing_required: Vec::new(),
            all_configured: true,
        };
        let artifacts = vec![
            "weex_amap-release.aar (高德 nvue 页面)".to_string(),
            "map-amap-release.aar (高德 vue 页面)".to_string(),
        ];

        let enabled = enabled_android_artifact_patterns("map", &artifacts, None, Some(&report));

        assert_eq!(enabled, vec!["weex_amap-release.aar"]);
    }

    #[test]
    fn amap_map_dependency_uses_user_configured_combined_version() {
        let report = crate::commands::module::AndroidModuleConfigReport {
            modules: vec![crate::commands::module::AndroidModuleConfigModule {
                name: "Maps".to_string(),
                template_key: "map".to_string(),
                category: "map".to_string(),
                platforms: vec!["android".to_string()],
                source: "test".to_string(),
                fields: vec![
                    crate::commands::shared::module::types::AndroidModuleConfigField {
                        key: "AMAP_SDK_VERSION".to_string(),
                        label: "高德地图 SDK 版本".to_string(),
                        required: false,
                        secret: false,
                        value: Some("10.0.700_loc6.4.5_sea9.7.2".to_string()),
                        value_source: Some("user".to_string()),
                        placeholder: String::new(),
                        field_type: "text".to_string(),
                    },
                ],
            }],
            missing_required: Vec::new(),
            all_configured: true,
        };
        let deps = vec![
            "com.amap.api:3dmap:latest.release".to_string(),
            "com.amap.api:search:latest.release".to_string(),
            "com.google.android.gms:play-services-maps:18.0.1".to_string(),
        ];

        let resolved = android_gradle_dependencies_for_report("map", deps, Some(&report));

        assert_eq!(
            resolved,
            vec![
                "com.amap.api:3dmap-location-search:10.0.700_loc6.4.5_sea9.7.2",
                "com.google.android.gms:play-services-maps:18.0.1",
            ]
        );
    }

    #[test]
    fn tencent_location_dependency_uses_user_configured_version() {
        let report = crate::commands::module::AndroidModuleConfigReport {
            modules: vec![crate::commands::module::AndroidModuleConfigModule {
                name: "Geolocation".to_string(),
                template_key: "geolocation".to_string(),
                category: "geolocation".to_string(),
                platforms: vec!["android".to_string()],
                source: "test".to_string(),
                fields: vec![
                    crate::commands::shared::module::types::AndroidModuleConfigField {
                        key: "TENCENT_LOCATION_SDK_VERSION".to_string(),
                        label: "腾讯定位 SDK 版本".to_string(),
                        required: false,
                        secret: false,
                        value: Some("2.3.1".to_string()),
                        value_source: Some("user".to_string()),
                        placeholder: String::new(),
                        field_type: "text".to_string(),
                    },
                ],
            }],
            missing_required: Vec::new(),
            all_configured: true,
        };
        let deps =
            vec!["com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8".to_string()];

        let resolved = android_gradle_dependencies_for_report("geolocation", deps, Some(&report));

        assert_eq!(
            resolved,
            vec!["com.tencent.map.geolocation:TencentLocationSdk-openplatform:2.3.1"]
        );
    }

    #[test]
    fn inserting_tencent_location_dependency_removes_other_versions() {
        let mut deps = BTreeSet::from([
            "com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8".to_string(),
            "androidx.core:core-ktx:1.6.0".to_string(),
        ]);

        insert_gradle_dependency(
            &mut deps,
            "com.tencent.map.geolocation:TencentLocationSdk-openplatform:2.3.1".to_string(),
        );

        assert!(deps.contains("com.tencent.map.geolocation:TencentLocationSdk-openplatform:2.3.1"));
        assert!(
            !deps.contains("com.tencent.map.geolocation:TencentLocationSdk-openplatform:7.5.4.8")
        );
        assert!(deps.contains("androidx.core:core-ktx:1.6.0"));
    }
}
