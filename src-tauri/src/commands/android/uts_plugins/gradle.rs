//! UTS 插件 build.gradle 生成与 Kotlin 插件版本处理
//!
//! 负责为每个自定义 UTS 插件生成标准的 build.gradle 文件，
//! 并修补 Kotlin Android 插件的 ID 和版本号

use std::path::Path;

use crate::commands::android::types::{
    sanitize_java_identifier, UnpackedAarInfo, UTS_KOTLIN_PLUGIN_VERSION,
};
use crate::commands::shared::resource_scan::UtsCustomPlugin;

/// 为单个 UTS 插件生成 build.gradle 文件
///
/// 如果已存在包含 `io.dcloud.uts.kotlin` 的 build.gradle，则仅修补版本号；
/// 否则生成完整的标准模板。
pub fn generate_uts_plugin_build_gradle(
    plugin: &UtsCustomPlugin,
    module_dir: &Path,
    unpacked_aars: &[UnpackedAarInfo],
) -> Result<(), String> {
    let path = module_dir.join("build.gradle");
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        if content.contains("io.dcloud.uts.kotlin") {
            let patched = patch_uts_kotlin_plugin_versions(&content);
            if patched != content {
                std::fs::write(&path, patched)
                    .map_err(|e| format!("修补 {} build.gradle 失败: {}", plugin.id, e))?;
            }
            return Ok(());
        }
    }

    let namespace = extract_namespace_from_manifest(module_dir)
        .unwrap_or_else(|| format!("uts.sdk.modules.{}", sanitize_java_identifier(&plugin.id)));

    let ndk_block = match &plugin.abis {
        Some(abis) if !abis.is_empty() => format!(
            "\n        ndk {{\n            abiFilters {}\n        }}",
            abis.iter()
                .map(|a| format!("'{}'", a))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => String::new(),
    };

    let min_sdk = plugin
        .min_sdk_version
        .map(|v| format!("\n        minSdk {}", v))
        .unwrap_or_default();

    let custom_deps: String = plugin
        .dependencies
        .iter()
        .filter_map(|d| d.source.as_deref().or(d.value.as_deref()))
        .filter(|s| !s.is_empty())
        .map(|d| format!("    implementation '{}'", d))
        .collect::<Vec<_>>()
        .join("\n");
    let custom_deps = if custom_deps.is_empty() {
        String::new()
    } else {
        format!("\n{}", custom_deps)
    };

    let extra_plugins: String = plugin
        .gradle_plugins
        .iter()
        .map(|p| render_uts_gradle_plugin_line(p))
        .collect::<Vec<_>>()
        .join("\n");
    let extra_plugins = if extra_plugins.is_empty() {
        String::new()
    } else {
        format!("\n{}", extra_plugins)
    };

    // 非标准 AAR 已解包为散落文件，生成显式依赖替代 fileTree 中的 .aar 引用
    let unpacked_deps: String = unpacked_aars
        .iter()
        .flat_map(|info| {
            let mut deps = Vec::new();
            deps.push(format!("    compileOnly files('libs/classes.jar')"));
            for jar in &info.extra_jars {
                deps.push(format!("    compileOnly files('libs/{}')", jar));
            }
            deps
        })
        .collect::<Vec<_>>()
        .join("\n");
    let unpacked_deps = if unpacked_deps.is_empty() {
        String::new()
    } else {
        format!("{}\n", unpacked_deps)
    };

    let content = format!(
        r#"plugins {{
    id 'com.android.library'
    id 'org.jetbrains.kotlin.android' version '{UTS_KOTLIN_PLUGIN_VERSION}'
{extra_plugins}}}

android {{
    namespace '{namespace}'
    compileSdk 35
    defaultConfig {{{ndk_block}{min_sdk}
    }}
}}

dependencies {{
    compileOnly fileTree(include: ['*.aar'], dir: '../app/libs')
    compileOnly fileTree(include: ['*.aar'], dir: './libs')
{unpacked_deps}    compileOnly 'com.alibaba:fastjson:1.1.46.android'
    compileOnly 'org.jetbrains.kotlin:kotlin-gradle-plugin:1.5.10'
    compileOnly 'androidx.core:core-ktx:1.6.0'
    compileOnly 'org.jetbrains.kotlin:kotlin-stdlib-jdk7:1.6.0'
    compileOnly 'org.jetbrains.kotlin:kotlin-reflect:1.6.0'
    compileOnly 'org.jetbrains.kotlinx:kotlinx-coroutines-core:1.3.8'
    compileOnly 'org.jetbrains.kotlinx:kotlinx-coroutines-android:1.3.8'
{custom_deps}
}}
"#
    );

    std::fs::write(&path, content)
        .map_err(|e| format!("写入 {} build.gradle 失败: {}", plugin.id, e))
}

/// 将原始 Gradle 插件声明字符串渲染为标准格式
///
/// - 空字符串 → 空字符串
/// - 已含 `id ` 前缀 → 直接使用（附带版本修补）
/// - kotlin-android 短名 → 补全命名空间 + 版本号
/// - 其他 → 包装为 `id 'xxx'`
fn render_uts_gradle_plugin_line(plugin: &str) -> String {
    let plugin = plugin.trim();
    if plugin.is_empty() {
        return String::new();
    }
    if plugin.starts_with("id ") {
        return format!("    {}", patch_uts_kotlin_plugin_versions(plugin));
    }
    if is_kotlin_android_plugin_id(plugin) {
        return format!(
            "    id '{}' version '{}'",
            plugin, UTS_KOTLIN_PLUGIN_VERSION
        );
    }
    format!("    id '{}'", plugin)
}

/// 修补 Kotlin Android 插件声明：
/// 1. 将短名称 `kotlin-android` 替换为完整命名空间 `org.jetbrains.kotlin.android`
/// 2. 为缺少版本号的 Kotlin Android 插件添加版本号
pub fn patch_uts_kotlin_plugin_versions(content: &str) -> String {
    let mut result = content.to_string();

    // 1. 首先修正插件 ID：将短名称替换为完整命名空间
    result = result.replace("id 'kotlin-android'", "id 'org.jetbrains.kotlin.android'");
    result = result.replace(
        r#"id "kotlin-android""#,
        r#"id "org.jetbrains.kotlin.android""#,
    );

    // 2. 然后添加版本号（如果缺失）
    for plugin_id in ["org.jetbrains.kotlin.android"] {
        let re = regex::Regex::new(&format!(
            r#"(?m)^([ \t]*id\s+['"]{}['"])([ \t]*(?://.*)?$)"#,
            regex::escape(plugin_id)
        ))
        .expect("valid kotlin plugin regex");
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                if caps[2].contains("version") {
                    format!("{}{}", &caps[1], &caps[2])
                } else {
                    format!(
                        "{} version '{}'{}",
                        &caps[1], UTS_KOTLIN_PLUGIN_VERSION, &caps[2]
                    )
                }
            })
            .to_string();
    }
    result
}

/// 判断插件 ID 是否为 Kotlin Android 插件（支持短名和全限定名）
pub fn is_kotlin_android_plugin_id(id: &str) -> bool {
    matches!(id, "kotlin-android" | "org.jetbrains.kotlin.android")
}

/// 从模块目录下的 AndroidManifest.xml 中提取 namespace（package 属性值）
pub fn extract_namespace_from_manifest(module_dir: &Path) -> Option<String> {
    let manifest_path = module_dir.join("src/main/AndroidManifest.xml");
    if !manifest_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    content.find("package=\"").and_then(|start| {
        let pkg_start = start + "package=\"".len();
        content[pkg_start..]
            .find('"')
            .map(|len| content[pkg_start..pkg_start + len].to_string())
    })
}
