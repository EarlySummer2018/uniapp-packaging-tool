//! Android 模块编排：Activity 源码复制、配置校验、Manifest 补丁生成、UTS 插件处理

#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::commands::module::{payment_provider_enabled_for_platform, PaymentProvider};

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
    let mut logged_prepare = false;
    let java_src_root = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/java");

    for module in modules {
        if !crate::commands::shared::module::templates::module_applies_to_android(&module.platforms)
        {
            continue;
        }
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
            if !should_copy_activity_source(template_key, class_name) {
                continue;
            }
            if !logged_prepare {
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
                logged_prepare = true;
            }
            let is_relative_activity = class_name.starts_with('.');
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

            let dest_file = activity_destination_file(
                &java_src_root,
                package_name,
                class_name,
                &source_relative,
            );

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
            let mut content = match std::fs::read(&source_file) {
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
            if is_relative_activity {
                let target_package = relative_activity_package_name(package_name, class_path);
                content = rewrite_java_package_declaration_bytes(&content, &target_package);
            }
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

fn should_copy_activity_source(template_key: &str, class_name: &str) -> bool {
    matches!(
        (template_key, class_name),
        ("login", ".wxapi.WXEntryActivity") | ("payment", ".wxapi.WXPayEntryActivity")
    )
}

fn activity_destination_file(
    java_src_root: &Path,
    package_name: &str,
    class_name: &str,
    source_relative: &str,
) -> std::path::PathBuf {
    if class_name.starts_with('.') {
        java_src_root
            .join(package_name.replace('.', "/"))
            .join(source_relative)
    } else {
        java_src_root.join(source_relative)
    }
}

fn relative_activity_package_name(package_name: &str, class_path: &str) -> String {
    match class_path.rsplit_once('.') {
        Some((subpackage, _)) if !subpackage.is_empty() => {
            format!("{}.{}", package_name, subpackage)
        }
        _ => package_name.to_string(),
    }
}

fn rewrite_java_package_declaration_bytes(content: &[u8], target_package: &str) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(content) else {
        return content.to_vec();
    };
    rewrite_java_package_declaration(text, target_package).into_bytes()
}

fn rewrite_java_package_declaration(content: &str, target_package: &str) -> String {
    let replacement = format!("package {};", target_package);
    let package_re = regex::Regex::new(
        r"(?m)^\s*package\s+[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*\s*;",
    )
    .expect("valid Java package regex");
    if package_re.is_match(content) {
        package_re.replacen(content, 1, replacement).to_string()
    } else {
        format!("{}\n\n{}", replacement, content)
    }
}

#[cfg(test)]
mod activity_source_tests {
    use super::*;

    #[test]
    fn relative_wx_activity_keeps_wxapi_subdirectory() {
        let root = Path::new("/tmp/java");
        let dest = activity_destination_file(
            root,
            "com.example.demo",
            ".wxapi.WXEntryActivity",
            "wxapi/WXEntryActivity.java",
        );

        assert_eq!(
            dest,
            Path::new("/tmp/java/com/example/demo/wxapi/WXEntryActivity.java")
        );
    }

    #[test]
    fn activity_source_copy_is_limited_to_wechat_login_and_payment() {
        assert!(should_copy_activity_source(
            "login",
            ".wxapi.WXEntryActivity"
        ));
        assert!(should_copy_activity_source(
            "payment",
            ".wxapi.WXPayEntryActivity"
        ));

        assert!(!should_copy_activity_source(
            "push",
            "com.tencent.tauth.AuthActivity"
        ));
        assert!(!should_copy_activity_source(
            "push",
            "cn.sharesdk.wechat.friends.WXFriendActivity"
        ));
        assert!(!should_copy_activity_source(
            "login",
            "com.tencent.tauth.AuthActivity"
        ));
        assert!(!should_copy_activity_source(
            "share",
            ".wxapi.WXEntryActivity"
        ));
    }

    #[test]
    fn relative_activity_package_is_rewritten_to_app_wxapi_package() {
        let content = "\n\npackage io.dcloud.HBuilder.wxapi;\n\npublic class WXEntryActivity {}\n";
        let rewritten = rewrite_java_package_declaration(content, "com.example.demo.wxapi");

        assert!(rewritten.contains("package com.example.demo.wxapi;"));
        assert!(!rewritten.contains("package io.dcloud.HBuilder.wxapi;"));
    }
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
#[allow(clippy::too_many_arguments)]
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
    let configurable_modules = report
        .modules
        .iter()
        .filter(|module| !module.fields.is_empty())
        .collect::<Vec<_>>();
    if configurable_modules.is_empty() {
        return;
    }

    let mut configured_fields = 0usize;
    let mut total_fields = 0usize;
    let mut missing_optional = Vec::new();
    let mut missing_required = Vec::new();
    for module in &configurable_modules {
        total_fields += module.fields.len();
        for field in &module.fields {
            let configured = field
                .value
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false);
            if configured {
                configured_fields += 1;
            } else if field.required {
                missing_required.push(format!("{} / {}", module.name, field.label));
            } else {
                missing_optional.push(format!("{} / {}", module.name, field.label));
            }
        }
    }
    emit_log(
        window,
        "info",
        &format!(
            "Android 模块配置已校验: {} 个模块，{}/{} 项已填写",
            configurable_modules.len(),
            configured_fields,
            total_fields
        ),
        None,
    );
    for item in missing_required {
        emit_log(
            window,
            "warn",
            &format!("缺失 Android 必填配置: {}", item),
            None,
        );
    }
    if !missing_optional.is_empty() {
        emit_log(
            window,
            "info",
            &format!("未填写可选配置: {}", missing_optional.join("、")),
            None,
        );
    }
}

/// 从报告构建模块配置树
pub fn module_config_tree_for_android_build(
    modules: &[crate::commands::resource::DetectedModule],
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    manifest: Option<&serde_json::Value>,
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
        if login_provider_enabled_for_manifest(manifest, "微信登录")
            || report_value(report, "login", "WX_APPID").is_some()
        {
            let mut config = HashMap::new();
            if let Some(wx_appid) = report_value(report, "login", "WX_APPID") {
                config.insert("WX_APPID".to_string(), wx_appid);
            }
            if let Some(secret) = report_value(report, "login", "WX_SECRET") {
                config.insert("WX_SECRET".to_string(), secret);
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "weixin".to_string(),
                enabled: true,
                config,
            });
        }
        if login_provider_enabled_for_manifest(manifest, "QQ登录")
            || report_value(report, "login", "QQ_APPID").is_some()
        {
            let mut config = HashMap::new();
            if let Some(qq_appid) = report_value(report, "login", "QQ_APPID") {
                config.insert("QQ_APPID".to_string(), qq_appid);
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "qq".to_string(),
                enabled: true,
                config,
            });
        }
        if login_provider_enabled_for_manifest(manifest, "一键登录")
            || report_value(report, "login", "GY_APP_ID").is_some()
        {
            let mut config = HashMap::new();
            if let Some(gy_appid) = report_value(report, "login", "GY_APP_ID") {
                config.insert("GY_APP_ID".to_string(), gy_appid);
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "univerify".to_string(),
                enabled: true,
                config,
            });
        }
        if login_provider_enabled_for_manifest(manifest, "微博登录")
            || report_value(report, "login", "SINA_APPKEY").is_some()
        {
            let mut config = HashMap::new();
            for key in ["SINA_APPKEY", "SINA_REDIRECT_URI"] {
                if let Some(value) = report_value(report, "login", key) {
                    config.insert(key.to_string(), value);
                }
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "sina".to_string(),
                enabled: true,
                config,
            });
        }
        if login_provider_enabled_for_manifest(manifest, "小米登录")
            || report_value(report, "login", "MIUI_APPID").is_some()
        {
            let mut config = HashMap::new();
            for key in ["MIUI_APPID", "MIUI_APPSECRET", "MIUI_REDIRECT_URI"] {
                if let Some(value) = report_value(report, "login", key) {
                    config.insert(key.to_string(), value);
                }
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "miui".to_string(),
                enabled: true,
                config,
            });
        }
        if login_provider_enabled_for_manifest(manifest, "Google登录") {
            providers.push(crate::commands::module::LoginProvider {
                name: "google".to_string(),
                enabled: true,
                config: HashMap::new(),
            });
        }
        if login_provider_enabled_for_manifest(manifest, "Facebook登录")
            || report_value(report, "login", "FACEBOOK_APP_ID").is_some()
        {
            let mut config = HashMap::new();
            for key in ["FACEBOOK_APP_ID", "FACEBOOK_CLIENT_TOKEN"] {
                if let Some(value) = report_value(report, "login", key) {
                    config.insert(key.to_string(), value);
                }
            }
            providers.push(crate::commands::module::LoginProvider {
                name: "facebook".to_string(),
                enabled: true,
                config,
            });
        }
        if !providers.is_empty() {
            login.providers = providers;
        }
    }

    if let Some(ref mut payment) = tree.payment {
        if manifest.is_some() {
            payment.alipay =
                payment_provider_enabled_for_manifest(manifest, PaymentProvider::Alipay)
                    .then(HashMap::new);
            payment.weixin =
                payment_provider_enabled_for_manifest(manifest, PaymentProvider::Weixin)
                    .then(HashMap::new);
            payment.paypal =
                payment_provider_enabled_for_manifest(manifest, PaymentProvider::Paypal)
                    .then(HashMap::new);
            payment.stripe =
                payment_provider_enabled_for_manifest(manifest, PaymentProvider::Stripe)
                    .then(HashMap::new);
            payment.google =
                payment_provider_enabled_for_manifest(manifest, PaymentProvider::Google)
                    .then(HashMap::new);
        }
        if payment.weixin.is_some() {
            if let Some(wx_appid) = report_value(report, "payment", "WX_APPID") {
                payment.weixin = Some(HashMap::from([("WX_APPID".to_string(), wx_appid)]));
            }
        }
        if payment.paypal.is_some() {
            if let Some(return_scheme) = report_value(report, "payment", "PAYPAL_RETURN_SCHEME") {
                payment.paypal = Some(HashMap::from([(
                    "PAYPAL_RETURN_SCHEME".to_string(),
                    return_scheme,
                )]));
            }
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

fn login_provider_enabled_for_manifest(
    manifest: Option<&serde_json::Value>,
    provider_note: &str,
) -> bool {
    manifest
        .map(|manifest| {
            crate::commands::module::android_module_artifact_enabled_for_manifest(
                "login",
                &format!("oauth-provider.aar ({})", provider_note),
                Some(manifest),
            )
        })
        .unwrap_or(false)
}

fn payment_provider_enabled_for_manifest(
    manifest: Option<&serde_json::Value>,
    provider: PaymentProvider,
) -> bool {
    manifest
        .map(|manifest| payment_provider_enabled_for_platform(manifest, provider, "android"))
        .unwrap_or(false)
}

pub fn android_module_string_resources(
    report: Option<&crate::commands::module::AndroidModuleConfigReport>,
) -> Vec<(String, String)> {
    let Some(report) = report else {
        return Vec::new();
    };
    let Some(app_id) = report_value(report, "login", "FACEBOOK_APP_ID") else {
        return Vec::new();
    };

    let mut resources = vec![
        ("facebook_app_id".to_string(), app_id.clone()),
        (
            "fb_login_protocol_scheme".to_string(),
            format!("fb{}", app_id),
        ),
    ];
    if let Some(client_token) = report_value(report, "login", "FACEBOOK_CLIENT_TOKEN") {
        resources.push(("facebook_client_token".to_string(), client_token));
    }
    resources
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

#[cfg(test)]
mod oauth_config_tests {
    use super::*;
    use crate::commands::shared::module::types::{
        AndroidModuleConfigField, AndroidModuleConfigModule, AndroidModuleConfigReport,
    };

    fn field(key: &str, value: &str) -> AndroidModuleConfigField {
        AndroidModuleConfigField {
            key: key.to_string(),
            value: Some(value.to_string()),
            ..Default::default()
        }
    }

    fn oauth_report() -> AndroidModuleConfigReport {
        AndroidModuleConfigReport {
            modules: vec![AndroidModuleConfigModule {
                name: "OAuth".to_string(),
                template_key: "login".to_string(),
                category: "oauth".to_string(),
                platforms: vec!["android".to_string()],
                source: "manifest.json".to_string(),
                fields: vec![
                    field("WX_APPID", "wx-id"),
                    field("WX_SECRET", "wx-secret"),
                    field("QQ_APPID", "qq-id"),
                    field("GY_APP_ID", "gy-id"),
                    field("SINA_APPKEY", "sina-key"),
                    field("SINA_REDIRECT_URI", "https://example.com/sina"),
                    field("MIUI_APPID", "miui-id"),
                    field("MIUI_APPSECRET", "miui-secret"),
                    field("MIUI_REDIRECT_URI", "https://example.com/miui"),
                    field("FACEBOOK_APP_ID", "facebook-id"),
                    field("FACEBOOK_CLIENT_TOKEN", "facebook-token"),
                ],
            }],
            all_configured: true,
            ..Default::default()
        }
    }

    #[test]
    fn all_selected_oauth_providers_are_kept_for_dcloud_properties() {
        let modules = vec![crate::commands::resource::DetectedModule {
            name: "OAuth".to_string(),
            category: "oauth".to_string(),
            platforms: vec!["android".to_string()],
            configured: true,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        }];
        let manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "oauth": {
                            "weixin": {},
                            "qq": {},
                            "univerify": {},
                            "sinaweibo": {},
                            "miui": {},
                            "google": {},
                            "facebook": {}
                        }
                    }
                }
            }
        });
        let report = oauth_report();

        let tree = module_config_tree_for_android_build(&modules, Some(&report), Some(&manifest));
        let provider_names = tree
            .login
            .expect("login config")
            .providers
            .into_iter()
            .map(|provider| provider.name)
            .collect::<Vec<_>>();

        assert_eq!(
            provider_names,
            vec![
                "weixin",
                "qq",
                "univerify",
                "sina",
                "miui",
                "google",
                "facebook"
            ]
        );
    }

    #[test]
    fn facebook_login_generates_official_string_resources() {
        let report = oauth_report();

        assert_eq!(
            android_module_string_resources(Some(&report)),
            vec![
                ("facebook_app_id".to_string(), "facebook-id".to_string()),
                (
                    "fb_login_protocol_scheme".to_string(),
                    "fbfacebook-id".to_string()
                ),
                (
                    "facebook_client_token".to_string(),
                    "facebook-token".to_string()
                ),
            ]
        );
    }
}

#[cfg(test)]
mod payment_config_tests {
    use super::*;
    use crate::commands::shared::module::types::{
        AndroidModuleConfigField, AndroidModuleConfigModule, AndroidModuleConfigReport,
    };

    #[test]
    fn all_selected_payment_providers_are_kept_for_dcloud_properties() {
        let modules = vec![crate::commands::resource::DetectedModule {
            name: "Payment".to_string(),
            category: "payment".to_string(),
            platforms: vec!["android".to_string()],
            configured: true,
            required_keys: vec![],
            source: "manifest.json".to_string(),
        }];
        let manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Payment": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "payment": {
                            "alipay": { "__platform__": ["android"] },
                            "weixin": { "__platform__": ["android"] },
                            "paypal": { "__platform__": ["android"] },
                            "stripe": { "__platform__": ["android"] },
                            "googlepay": {}
                        }
                    }
                }
            }
        });
        let report = AndroidModuleConfigReport {
            modules: vec![AndroidModuleConfigModule {
                name: "Payment".to_string(),
                template_key: "payment".to_string(),
                category: "payment".to_string(),
                platforms: vec!["android".to_string()],
                source: "manifest.json".to_string(),
                fields: vec![
                    AndroidModuleConfigField {
                        key: "WX_APPID".to_string(),
                        value: Some("wx-pay".to_string()),
                        ..Default::default()
                    },
                    AndroidModuleConfigField {
                        key: "PAYPAL_RETURN_SCHEME".to_string(),
                        value: Some("paypal-demo".to_string()),
                        ..Default::default()
                    },
                ],
            }],
            all_configured: true,
            ..Default::default()
        };

        let payment =
            module_config_tree_for_android_build(&modules, Some(&report), Some(&manifest))
                .payment
                .expect("payment config");

        assert!(payment.alipay.is_some());
        assert!(payment.weixin.is_some());
        assert!(payment.paypal.is_some());
        assert!(payment.stripe.is_some());
        assert!(payment.google.is_some());
    }
}
