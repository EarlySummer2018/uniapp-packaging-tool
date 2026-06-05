//! Android 图标生成 + 启动图处理

use crate::commands::android::types::emit_log;
use std::path::{Path, PathBuf};

pub fn generate_icons(
    android_icons: Option<&crate::commands::shared::resource::AndroidIconsConfig>,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let res_dir = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/res");

    // 清理 SDK 模板自带的旧图标文件，避免残留
    clean_drawable_files(&res_dir, &["icon.png", "push.png", "splash.png"])?;

    let Some(config) = android_icons else {
        emit_log(
            window,
            "warn",
            "manifest 未配置 Android 图标，跳过图标复制",
            None,
        );
        return Ok(());
    };

    if config.android.is_empty() {
        emit_log(
            window,
            "warn",
            "manifest 未提供 Android 图标密度资源，跳过",
            None,
        );
        return Ok(());
    }

    let density_map: &[(&str, &str)] = &[
        ("hdpi", "mipmap-hdpi"),
        ("xhdpi", "mipmap-xhdpi"),
        ("xxhdpi", "mipmap-xxhdpi"),
        ("xxxhdpi", "mipmap-xxxhdpi"),
    ];

    let mut copied = 0usize;
    for (density, source_path) in &config.android {
        let Some(res_subdir) = density_map
            .iter()
            .find(|(d, _)| *d == density.as_str())
            .map(|(_, r)| *r)
        else {
            emit_log(
                window,
                "warn",
                &format!("忽略不支持的 Android 图标密度: {}", density),
                None,
            );
            continue;
        };
        let source = PathBuf::from(source_path);
        if !source.exists() {
            emit_log(
                window,
                "warn",
                &format!("Android 图标不存在: {}", source.display()),
                None,
            );
            continue;
        }
        let target_dir = res_dir.join(res_subdir);
        crate::utils::fs::ensure_directory(&target_dir).map_err(|e| e.to_string())?;
        crate::utils::fs::copy_file(&source, &target_dir.join("icon.png"))
            .map_err(|e| format!("复制 Android 图标失败 {}: {}", source.display(), e))?;
        copied += 1;
    }

    if copied > 0 {
        set_android_launcher_icon_reference(workspace)?;
        emit_log(
            window,
            "success",
            &format!("已导入 {} 张 Android 自定义图标", copied),
            None,
        );
    } else {
        emit_log(window, "warn", "Android 自定义图标均未找到", None);
    }
    Ok(())
}

fn set_android_launcher_icon_reference(workspace: &Path) -> Result<(), String> {
    let manifest_path = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/AndroidManifest.xml");
    if !manifest_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 AndroidManifest.xml 失败: {}", e))?;
    let mut editor =
        crate::commands::android::project_mod::xml_editor::XmlManifestEditor::from_str(&content);
    editor.set_application_attr("android:icon", "@mipmap/icon")?;
    std::fs::write(&manifest_path, editor.as_str())
        .map_err(|e| format!("写入 AndroidManifest.xml 图标引用失败: {}", e))
}

pub fn apply_push_small_icon(
    push_icons: Option<&crate::commands::shared::resource::PushIconsConfig>,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let Some(config) = push_icons else {
        return Ok(());
    };
    if config.small.is_none() && config.small_densities.is_empty() {
        return Ok(());
    }

    let res_dir = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/res");
    clean_push_icon_files(&res_dir)?;

    let mut copied = 0usize;
    if let Some(source_path) = config.small.as_deref() {
        let drawable_dir = res_dir.join("drawable");
        copy_push_icon_file(source_path, &drawable_dir)?;
        copied += 1;
    }
    for (density, source_path) in &config.small_densities {
        let Some(drawable_dir) = push_icon_drawable_dir(density).map(|dir| res_dir.join(dir))
        else {
            emit_log(
                window,
                "warn",
                &format!("忽略不支持的 Push 小图标密度: {}", density),
                None,
            );
            continue;
        };
        copy_push_icon_file(source_path, &drawable_dir)?;
        copied += 1;
    }

    if copied == 0 {
        return Err("Push 小图标未导入：未找到支持的 Android 密度或本地图片".to_string());
    }

    emit_log(
        window,
        "success",
        &format!("已导入 {} 张 Push 小图标", copied),
        None,
    );
    Ok(())
}

fn copy_push_icon_file(source_path: &str, target_dir: &Path) -> Result<(), String> {
    let source_path = source_path.trim();
    if source_path.contains("://") || source_path.starts_with("data:") {
        return Err("Push 小图标必须是本地图片文件，不能使用远程 URL 或 data URI".to_string());
    }

    let source = PathBuf::from(source_path);
    if !source.exists() {
        return Err(format!("Push 小图标不存在: {}", source.display()));
    }

    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "png".to_string());

    crate::utils::fs::ensure_directory(target_dir).map_err(|e| e.to_string())?;
    let target = target_dir.join(format!("push_icon.{}", extension));
    crate::utils::fs::copy_file(&source, &target)
        .map_err(|e| format!("复制 Push 小图标失败 {}: {}", source.display(), e))?;
    Ok(())
}

fn push_icon_drawable_dir(density: &str) -> Option<&'static str> {
    match density {
        "ldpi" => Some("drawable-ldpi"),
        "mdpi" => Some("drawable-mdpi"),
        "hdpi" => Some("drawable-hdpi"),
        "xhdpi" => Some("drawable-xhdpi"),
        "xxhdpi" => Some("drawable-xxhdpi"),
        "xxxhdpi" => Some("drawable-xxxhdpi"),
        _ => None,
    }
}

fn clean_push_icon_files(res_dir: &Path) -> Result<(), String> {
    if !res_dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(res_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with("drawable") || !entry.path().is_dir() {
            continue;
        }
        for file in std::fs::read_dir(entry.path()).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            let path = file.path();
            let is_push_icon = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| stem == "push_icon")
                .unwrap_or(false);
            if is_push_icon && path.is_file() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("清理旧 Push 小图标失败 {}: {}", path.display(), e))?;
            }
        }
    }
    Ok(())
}

/// 清理 res 目录下所有 drawable/mipmap 子目录中的指定文件名。
///
/// 用于在写入用户自定义资源（图标、启动图等）之前，
/// 删除 SDK 模板自带的同名默认文件，避免新旧文件共存。
pub fn clean_drawable_files(res_dir: &Path, file_names: &[&str]) -> Result<(), String> {
    let Ok(entries) = std::fs::read_dir(res_dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(n) => n,
            None => continue,
        };
        // 只处理 drawable/mipmap 资源目录
        if !(name_str.starts_with("drawable") || name_str.starts_with("mipmap"))
            || !entry.path().is_dir()
        {
            continue;
        }
        for &file_name in file_names {
            let target = entry.path().join(file_name);
            if target.exists() {
                if let Err(e) = std::fs::remove_file(&target) {
                    // 删除失败不中断构建，仅记录
                    eprintln!(
                        "[WARN] 清理旧 drawable 文件失败: {} ({})",
                        target.display(),
                        e
                    );
                }
            }
            // 同时清理 .9.png 变体（如 splash.9.png）
            let target_9 = entry.path().join(format!("{}.9", file_name));
            if target_9.exists() {
                if let Err(e) = std::fs::remove_file(&target_9) {
                    eprintln!(
                        "[WARN] 清理旧 drawable 文件失败: {} ({})",
                        target_9.display(),
                        e
                    );
                }
            }
        }
    }
    Ok(())
}

pub fn apply_android_splashscreen(
    splashscreen: Option<&crate::commands::resource::SplashscreenConfig>,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    let res_dir = workspace
        .join(crate::commands::android::project_mod::MODULE_NAME)
        .join("src/main/res");

    // 先清理旧的启动图残留（包括之前可能写入的 unipack_splash_image.* 和 splash.*）
    clean_drawable_files(&res_dir, &["splash.png", "unipack_splash_image.png"])?;

    let Some(config) = splashscreen else {
        // 无 splashscreen 配置 → 使用 SDK 默认（通用启动界面），不做任何处理
        return Ok(());
    };

    // 判断是否为自定义启动图模式：androidStyle 为 "default"/空/缺失 → 自定义模式
    // androidStyle 为 "common" → 通用启动界面模式（使用 SDK 渲染逻辑）
    let is_custom_style = config
        .android_style
        .as_deref()
        .map(|s| s == "default" || s.is_empty())
        .unwrap_or(true);

    if config.android.is_empty() {
        // 无图片资源，使用 SDK 默认启动界面
        return Ok(());
    }

    let mut copied = 0usize;
    for (density, source) in &config.android {
        let Some(drawable_dir) = android_splash_drawable_dir(density) else {
            emit_log(
                window,
                "warn",
                &format!("忽略不支持的 Android 启动图密度: {}", density),
                None,
            );
            continue;
        };
        let source_path = PathBuf::from(source);
        if !source_path.exists() {
            emit_log(
                window,
                "warn",
                &format!("Android 启动图不存在: {}", source_path.display()),
                None,
            );
            continue;
        }
        let target_name = if source_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.ends_with(".9.png"))
            .unwrap_or(false)
        {
            "splash.9.png"
        } else {
            "splash.png"
        };
        let target = res_dir.join(drawable_dir).join(target_name);
        crate::utils::fs::copy_file(&source_path, &target)
            .map_err(|e| format!("复制 Android 启动图失败 {}: {}", source_path.display(), e))?;
        copied += 1;
    }

    if copied > 0 {
        if is_custom_style {
            // 按官方指南配置全屏启动图：写入 AppTheme.Splash style + 设置主 Activity theme
            setup_splash_screen(&res_dir)?;
        }
        emit_log(
            window,
            "info",
            &format!("已导入 {} 张自定义启动图", copied),
            None,
        );
    } else {
        emit_log(window, "warn", "Android 自定义启动图均未找到", None);
    }
    Ok(())
}

pub fn android_splash_drawable_dir(density: &str) -> Option<&'static str> {
    match density.to_ascii_lowercase().as_str() {
        "ldpi" => Some("drawable-ldpi"),
        "mdpi" => Some("drawable-mdpi"),
        "hdpi" => Some("drawable-hdpi"),
        "xhdpi" => Some("drawable-xhdpi"),
        "xxhdpi" => Some("drawable-xxhdpi"),
        "xxxhdpi" => Some("drawable-xxxhdpi"),
        _ => None,
    }
}

/// 按官方 launch-config.md 文档配置全屏启动图：
/// 1. 在 values/styles.xml 中写入 AppTheme.Splash（windowBackground=@drawable/splash）
/// 2. 找到 LAUNCHER Activity，将其 android:theme 设为 @style/AppTheme.Splash
///
/// Manifest 修改采用与 XmlManifestEditor::set_application_attr() 一致的正则模式。
fn setup_splash_screen(res_dir: &Path) -> Result<(), String> {
    // === Part A: 写入 AppTheme.Splash 到 styles.xml ===
    let styles_path = res_dir.join("values").join("styles.xml");
    if styles_path.exists() {
        let content = std::fs::read_to_string(&styles_path)
            .map_err(|e| format!("读取 styles.xml 失败: {}", e))?;

        if !content.contains("AppTheme.Splash") {
            const SPLASH_STYLE: &str = r#"
    <style name="AppTheme.Splash" parent="Theme.AppCompat.Light.NoActionBar">
        <item name="windowNoTitle">true</item>
        <item name="windowActionBar">false</item>
        <item name="android:windowContentOverlay">@null</item>
        <item name="android:windowFullscreen">true</item>
        <item name="android:windowBackground">@drawable/splash</item>
    </style>"#;
            let merged =
                content.replace("</resources>", &format!("{}\n</resources>", SPLASH_STYLE));
            crate::utils::fs::write_string_to_file(&styles_path, &merged)
                .map_err(|e| format!("写入 styles.xml 失败: {}", e))?;
        }
    }

    // === Part B: 修改 AndroidManifest.xml — 正则模式设置 LAUNCHER Activity theme ===
    let manifest_path = res_dir
        .parent()
        .ok_or("res 目录无父目录".to_string())?
        .join("AndroidManifest.xml");
    if !manifest_path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 AndroidManifest.xml 失败: {}", e))?;

    let modified = set_launcher_theme_attr(&content, "@style/TranslucentTheme");

    if modified != content {
        crate::utils::fs::write_string_to_file(&manifest_path, &modified)
            .map_err(|e| format!("写入 AndroidManifest.xml 失败: {}", e))?;
    }
    Ok(())
}

/// 在 AndroidManifest.xml 中找到包含 LAUNCHER intent-filter 的 <activity>，
/// 设置其 android:theme 属性。
///
/// 实现方式与 XmlManifestEditor::set_application_attr() 完全一致：
/// - 正则定位 target tag 的 opening tag 范围
/// - 正则匹配/替换属性值或在 > 前插入新属性
/// - 从后往前替换避免字节偏移问题
fn set_launcher_theme_attr(xml: &str, theme_value: &str) -> String {
    let Some((tag_start, gt_pos)) = find_launcher_activity_opening_tag(xml) else {
        return xml.to_string();
    };

    // tag_content = <activity ... （不含 >）
    let tag_content = &xml[tag_start..gt_pos];
    let escaped_value = escape_xml_attr(theme_value);

    // 用正则检查是否已有 android:theme 属性
    let theme_re =
        regex::Regex::new(r#"\s*android:theme\s*=\s*"[^"]*""#).expect("theme 属性正则编译失败");

    if theme_re.is_match(tag_content) {
        // 已有 → 替换所有出现（从后往前避免偏移）
        let mut result = xml.to_string();
        let mut matches: Vec<_> = theme_re
            .find_iter(tag_content)
            .map(|m| (m.start() + tag_start, m.end() + tag_start))
            .collect();
        matches.sort_by(|a, b| b.0.cmp(&a.0));
        for (start, end) in matches {
            result.replace_range(
                start..end,
                &format!(r#" android:theme="{}""#, escaped_value),
            );
        }
        result
    } else {
        // 没有 → 在 > 前插入
        format!(
            "{} android:theme=\"{}\">{}",
            tag_content.trim_end(),
            escaped_value,
            &xml[gt_pos + 1..]
        )
    }
}

/// 找到 LAUNCHER Activity 的 opening tag 范围。
///
/// 返回 `(tag_start, gt_pos)` 其中 gt_pos 是 '>' 的位置（不含）。
fn find_launcher_activity_opening_tag(xml: &str) -> Option<(usize, usize)> {
    let activity_re = regex::Regex::new(r#"<activity\b[^>]*>"#).expect("activity 标签正则编译失败");

    for mat in activity_re.find_iter(xml) {
        let tag_start = mat.start();
        let after_gt = mat.end(); // > 之后的位置

        // 向后查找对应的 </activity> 确定这个 activity 块的范围
        let block_end = xml[after_gt..].find("</activity>")?;
        let block = &xml[tag_start..after_gt + block_end + "</activity>".len()];

        // 检查是否为 LAUNCHER Activity
        if block.contains("android.intent.action.MAIN")
            && block.contains("android.intent.category.LAUNCHER")
        {
            return Some((tag_start, after_gt - 1)); // > 的位置
        }
    }

    None
}

/// 转义 XML 属性值中的特殊字符。
fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_icon_reference_uses_mipmap_icon() {
        let workspace =
            std::env::temp_dir().join(format!("unipack-android-icon-{}", uuid::Uuid::new_v4()));
        let manifest_dir = workspace
            .join(crate::commands::android::project_mod::MODULE_NAME)
            .join("src/main");
        std::fs::create_dir_all(&manifest_dir).unwrap();
        let manifest_path = manifest_dir.join("AndroidManifest.xml");
        std::fs::write(
            &manifest_path,
            r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application android:icon="@drawable/icon" android:label="@string/app_name">
    </application>
</manifest>
"#,
        )
        .unwrap();

        set_android_launcher_icon_reference(&workspace).unwrap();

        let manifest = std::fs::read_to_string(manifest_path).unwrap();
        assert!(manifest.contains(r#"android:icon="@mipmap/icon""#));
        assert!(!manifest.contains(r#"android:icon="@drawable/icon""#));

        let _ = std::fs::remove_dir_all(workspace);
    }
}
