//! Android 模块编排：Activity 源码复制、配置校验、Manifest 补丁生成、UTS 插件处理

#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

// ===== 模块 Activity 源文件复制 =====

/// 从 DCloud SDK 的 src 目录复制模块所需的 Activity Java 源文件到 Android 工作区。
pub fn copy_module_activity_sources(
    modules: &[crate::commands::resource::DetectedModule],
    sdk_src_dir: &Path,
    workspace: &Path,
    package_name: &str,
    window: &tauri::Window,
) -> Result<(), String> {
    use crate::commands::android::types::emit_log;

    let mut copied = Vec::new();
    // 去重：同一缺失文件只警告一次（多个模块可能引用同一个第三方 SDK 源码）
    let mut warned_missing: std::collections::HashSet<String> = std::collections::HashSet::new();
    let java_src_root = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/java");

    // 诊断日志：输出关键路径信息，便于排查路径解析问题
    emit_log(
        window,
        "info",
        &format!(
            "准备复制模块 Activity 源文件: workspace={} java_src_root={} package_name={}",
            workspace.display(),
            java_src_root.display(),
            package_name
        ),
        None,
    );

    for module in modules {
        let template_key = match super::artifacts::android_module_template_key(&module.name) {
            Some(k) => k,
            None => continue,
        };
        let template = match crate::commands::module::get_module_template_sync(template_key) {
            Ok(t) => t,
            Err(_) => continue,
        };

        for activity_class in &template.android_config.activities {
            // activity_class 格式如 ".wxapi.WXEntryActivity (微信登录回调)"
            // 或 "com.tencent.tauth.AuthActivity (QQ登录)" — 需要提取纯类名
            let class_name = match activity_class.find([' ', '(', '（']) {
                Some(pos) => &activity_class[..pos],
                None => activity_class.as_str(),
            };
            if class_name.is_empty() {
                continue;
            }
            let class_path = class_name.trim_start_matches('.');
            // 从类全限定名推导源文件相对路径：wxapi/WXEntryActivity.java
            let source_relative = format!("{}.java", class_path.replace('.', "/"));
            let source_file = sdk_src_dir.join(&source_relative);

            if !source_file.exists() {
                // 第三方 SDK（QQ、微博、微信支付等）的 Activity 源码已编译在 AAR 中，
                // 不以独立 .java 文件存在于 SDK/src 下，属于正常情况
                if warned_missing.insert(source_relative.clone()) {
                    emit_log(
                        window,
                        "info",
                        &format!(
                            "Activity 源文件不存在于 SDK/src 中（已跳过，通常为第三方 SDK 源码）: {}",
                            source_relative
                        ),
                        None,
                    );
                }
                continue;
            }

            // 目标路径：{workspace}/simpleDemo/src/main/java/{package_name}/{activity_subpath}
            let activity_subpath =
                source_relative[source_relative.find('/').unwrap_or(0)..].trim_start_matches('/');
            let dest_file = java_src_root
                .join(package_name.replace('.', "/"))
                .join(activity_subpath);

            // 安全守卫：目标路径不能落在根目录或非工作区位置
            if !dest_file.starts_with(workspace) {
                return Err(format!(
                    "模块 {} Activity 源文件目标路径逃逸了工作区: workspace={} java_src_root={} package_name={} class_path={} → dest={} (source_relative={})",
                    module.name,
                    workspace.display(),
                    java_src_root.display(),
                    package_name,
                    class_path,
                    dest_file.display(),
                    source_relative
                ));
            }

            crate::utils::fs::ensure_directory(dest_file.parent().unwrap()).map_err(|e| {
                format!(
                    "创建 Activity 源文件目录失败: {} (目标: {})",
                    e,
                    dest_file.display()
                )
            })?;

            // 使用 read + write 替代 std::fs::copy，避免部分文件系统上的 EROFS 问题
            let content = match std::fs::read(&source_file) {
                Ok(c) => c,
                Err(e) => {
                    emit_log(
                        window,
                        "warn",
                        &format!(
                            "读取模块 {} Activity 源文件失败 {}: {} (跳过)",
                            module.name, class_name, e
                        ),
                        None,
                    );
                    continue;
                }
            };
            if let Err(e) = std::fs::write(&dest_file, &content) {
                let abs_path = dest_file
                    .canonicalize()
                    .unwrap_or_else(|_| dest_file.clone());
                let parent_info = dest_file
                    .parent()
                    .and_then(|p| {
                        std::fs::metadata(p).ok().map(|m| {
                            let mode = m.permissions().mode();
                            format!("writable={} mode={:o}", mode & 0o200 != 0, mode)
                        })
                    })
                    .unwrap_or_else(|| "父目录不存在".to_string());
                return Err(format!(
                    "写入模块 {} Activity 源文件失败 {}: {} → {} (parent: {})",
                    module.name,
                    class_name,
                    e,
                    abs_path.display(),
                    parent_info
                ));
            }

            let file_name = dest_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            copied.push(format!("{} ({})", module.name, file_name));
        }
    }

    if !copied.is_empty() {
        emit_log(
            window,
            "success",
            &format!(
                "已复制 {} 个模块 Activity 源文件: {}",
                copied.len(),
                copied.join(", ")
            ),
            None,
        );
    }
    Ok(())
}

// ===== 配置校验与合并 =====

pub fn validate_android_config(
    config: &crate::commands::project::ProjectConfig,
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<(), String> {
    if sdk_config.dcloud_android_sdk_path.trim().is_empty() {
        return Err("请先在 SDK & 环境管理中配置 DCloud Android 离线 SDK 路径".to_string());
    }
    if config.android.package_name.trim().is_empty() {
        return Err("请先配置 Android 包名".to_string());
    }
    if config.android.dcloud_app_key.trim().is_empty() {
        return Err("请先配置 Android DCloud AppKey".to_string());
    }
    if config.android.keystore.path.trim().is_empty()
        || config.android.keystore.alias.trim().is_empty()
        || !config.android.keystore.has_store_password
        || !config.android.keystore.has_key_password
    {
        return Err(
            "Android release 构建需要完整 Keystore 路径、Alias、Store 密码和 Key 密码".to_string(),
        );
    }
    Ok(())
}

pub fn merged_android_module_config(
    project: &crate::commands::project::ProjectConfig,
    module_config: Option<HashMap<String, String>>,
) -> Option<HashMap<String, String>> {
    let mut merged = project.android_module_config.clone();
    if let Some(module_config) = module_config {
        for (key, value) in module_config {
            if value.trim().is_empty() {
                merged.remove(&key);
            } else {
                merged.insert(key, value);
            }
        }
    }
    (!merged.is_empty()).then_some(merged)
}

// ===== Base64 解码 =====

/// 简易 base64 解码（标准字母表）
pub fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("base64 输入为空".to_string());
    }

    let mut decoded = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer: u32 = 0;
    let mut bits_collected: u32 = 0;

    for ch in input.bytes() {
        let val = match ch {
            b'A'..=b'Z' => ch - b'A',
            b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                break;
            }
            _ => continue,
        };
        buffer = (buffer << 6) | u32::from(val);
        bits_collected += 6;
        if bits_collected >= 8 {
            bits_collected -= 8;
            decoded.push(((buffer >> bits_collected) & 0xFF) as u8);
        }
    }
    if decoded.is_empty() {
        return Err("base64 解码结果为空".to_string());
    }
    Ok(decoded)
}

// ===== Manifest 模块应用与补丁渲染 =====

/// 应用 Android manifest 模块（公开接口，供 commands.rs 调用）
pub fn apply_android_manifest_modules(
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
    super::artifacts::apply_android_manifest_modules_internal(
        modules,
        config_report,
        manifest,
        sdk_libs,
        libs_dst,
        workspace,
        extra_repos,
        extra_deps,
        window,
    )
}

/// 渲染 Android 模块 manifest 补丁（公开接口）
pub fn render_android_module_manifest_patches(
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    package_name: &str,
    app_id: &str,
) -> (
    crate::commands::android::types::AndroidManifestPatches,
    Vec<crate::commands::android::project_mod::ManifestPatchGroup>,
) {
    super::manifest_patches_render::render_android_module_manifest_patches_impl(
        report,
        package_name,
        app_id,
    )
}

/// 发送模块配置报告日志
pub fn emit_android_module_config_report(
    window: &tauri::Window,
    report: &crate::commands::module::AndroidModuleConfigReport,
) {
    use crate::commands::android::types::emit_log;
    if report.modules.is_empty() {
        return;
    }

    emit_log(
        window,
        "info",
        &format!(
            "Android 模块配置清单: {} 个模块需要配置项",
            report.modules.len()
        ),
        None,
    );
    for module in &report.modules {
        emit_log(
            window,
            "info",
            &format!("模块 {} 需要配置 {} 项", module.name, module.fields.len()),
            None,
        );
        for field in &module.fields {
            let status = if field
                .value
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
            {
                match field.value_source.as_deref() {
                    Some("manifest") => "已从 manifest 读取",
                    Some("user") => "已在构建中心填写",
                    _ => "已填写",
                }
            } else if field.required {
                "缺失必填"
            } else {
                "未填写可选"
            };
            emit_log(
                window,
                if field.required && status == "缺失必填" {
                    "warn"
                } else {
                    "info"
                },
                &format!("  - {} ({})", field.label, status),
                None,
            );
        }
    }
}

/// 从报告构建模块配置树
pub fn module_config_tree_for_android_build(
    modules: &[crate::commands::resource::DetectedModule],
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
) -> crate::commands::module::ModuleConfigTree {
    let mut tree = crate::commands::module::module_config_from_detected_modules(modules);
    let Some(report) = report else {
        return tree;
    };

    if let Some(ref mut push) = tree.push {
        push.unipush_appid = report_value(report, "push", "GETUI_APPID");
        push.unipush_appkey = report_value(report, "push", "plus.unipush.appkey");
        push.unipush_appsecret = report_value(report, "push", "plus.unipush.appsecret");
    }

    if let Some(ref mut geolocation) = tree.geolocation {
        geolocation.baidu_ak = report_value(report, "geolocation", "BAIDU_MAP_AK");
        geolocation.amap_key = report_value(report, "geolocation", "AMAP_KEY");
        geolocation.engine = if geolocation.baidu_ak.is_some() {
            "baidu".to_string()
        } else if geolocation.amap_key.is_some() {
            "amap".to_string()
        } else {
            "system".to_string()
        };
    }

    if let Some(ref mut share) = tree.share {
        if let Some(wx_appid) = report_value(report, "share", "WX_APPID") {
            let mut value = HashMap::new();
            value.insert("WX_APPID".to_string(), wx_appid);
            if let Some(secret) = report_value(report, "share", "WX_SECRET") {
                value.insert("WX_SECRET".to_string(), secret);
            }
            share.weixin = Some(value);
        }
        if let Some(qq_appid) = report_value(report, "share", "QQ_APPID") {
            share.qq = Some(HashMap::from([("QQ_APPID".to_string(), qq_appid)]));
        }
        if let Some(sina_appkey) = report_value(report, "share", "SINA_APPKEY") {
            let mut value = HashMap::from([("SINA_APPKEY".to_string(), sina_appkey)]);
            if let Some(secret) = report_value(report, "share", "SINA_SECRET") {
                value.insert("SINA_SECRET".to_string(), secret);
            }
            if let Some(uri) = report_value(report, "share", "SINA_REDIRECT_URI") {
                value.insert("SINA_REDIRECT_URI".to_string(), uri);
            }
            share.sina = Some(value);
        }
    }

    if let Some(ref mut login) = tree.login {
        let mut providers = Vec::new();
        if let Some(wx_appid) = report_value(report, "login", "WX_APPID") {
            let mut config = HashMap::from([("WX_APPID".to_string(), wx_appid)]);
            if let Some(secret) = report_value(report, "login", "WX_SECRET") {
                config.insert("WX_SECRET".to_string(), secret);
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "weixin".to_string(),
                enabled: true,
                config,
            });
        }
        if let Some(qq_appid) = report_value(report, "login", "QQ_APPID") {
            providers.push(crate::commands::module::LoginProvider {
                name: "qq".to_string(),
                enabled: true,
                config: HashMap::from([("QQ_APPID".to_string(), qq_appid)]),
            });
        }
        if let Some(gy_appid) = report_value(report, "login", "GY_APP_ID") {
            providers.push(crate::commands::module::LoginProvider {
                name: "univerify".to_string(),
                enabled: true,
                config: HashMap::from([("GY_APP_ID".to_string(), gy_appid)]),
            });
        }
        if !providers.is_empty() {
            login.providers = providers;
        }
    }

    if let Some(ref mut payment) = tree.payment {
        if let Some(wx_appid) = report_value(report, "payment", "WX_APPID") {
            payment.weixin = Some(HashMap::from([("WX_APPID".to_string(), wx_appid)]));
        }
    }

    if let Some(ref mut map) = tree.map {
        map.baidu_map_ak = report_value(report, "map", "BAIDU_MAP_AK");
        map.amap_key = report_value(report, "map", "AMAP_KEY");
        map.tencent_map_key = report_value(report, "map", "TENCENT_MAP_KEY");
        map.google_maps_api_key = report_value(report, "map", "GOOGLE_MAPS_API_KEY");
        map.engine = if map.baidu_map_ak.is_some() {
            "baidu".to_string()
        } else if map.tencent_map_key.is_some() {
            "tencent".to_string()
        } else if map.google_maps_api_key.is_some() {
            "google".to_string()
        } else {
            "amap".to_string()
        };
    }

    if let Some(ref mut statistic) = tree.statistic {
        if let Some(appkey) = report_value(report, "statistic", "UMENG_APPKEY") {
            statistic.umeng = Some(HashMap::from([
                ("UMENG_APPKEY".to_string(), appkey),
                (
                    "UMENG_CHANNEL".to_string(),
                    report_value(report, "statistic", "UMENG_CHANNEL").unwrap_or_default(),
                ),
            ]));
            statistic.provider = "umeng".to_string();
        }
    }

    if let Some(ref mut livepusher) = tree.livepusher {
        livepusher.license_url = report_value(report, "livepusher", "LIVEPUSH_LICENSE_URL");
        livepusher.license_key = report_value(report, "livepusher", "LIVEPUSH_LICENSE_KEY");
    }

    tree
}

fn report_value(
    report: &crate::commands::module::AndroidModuleConfigReport,
    template_key: &str,
    key: &str,
) -> Option<String> {
    report
        .modules
        .iter()
        .find(|module| module.template_key == template_key)
        .and_then(|module| module.fields.iter().find(|field| field.key == key))
        .and_then(|field| field.value.clone())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
