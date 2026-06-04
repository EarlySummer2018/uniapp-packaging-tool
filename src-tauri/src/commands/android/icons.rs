//! Android 图标生成 + 启动图处理

use crate::commands::android::types::emit_log;
use std::path::{Path, PathBuf};

pub fn generate_icons(
    config: &crate::commands::project::ProjectConfig,
    workspace: &Path,
    window: &tauri::Window,
) -> Result<(), String> {
    if config.app.icon1024.trim().is_empty() {
        emit_log(window, "warn", "未配置 1024 图标，跳过图标生成", None);
        return Ok(());
    }
    let source = PathBuf::from(&config.app.icon1024);
    if !source.exists() {
        emit_log(
            window,
            "warn",
            &format!("图标文件不存在: {}", source.display()),
            None,
        );
        return Ok(());
    }
    let img = image::open(&source)
        .map_err(|e| format!("读取图标失败: {}", e))?
        .to_rgba8();
    let res_dir = workspace
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/res");

    // 先清理 SDK 模板自带的旧图标文件（icon.png / push.png / splash.png），避免残留
    clean_drawable_files(&res_dir, &["icon.png", "push.png", "splash.png"])?;

    for (dir, size) in [
        ("drawable-ldpi", 36),
        ("drawable-mdpi", 48),
        ("drawable-hdpi", 72),
        ("drawable-xhdpi", 96),
        ("drawable-xxhdpi", 144),
        ("drawable-xxxhdpi", 192),
    ] {
        let out_dir = res_dir.join(dir);
        crate::utils::fs::ensure_directory(&out_dir).map_err(|e| e.to_string())?;
        for name in ["icon.png", "push.png", "splash.png"] {
            let resized = image::imageops::resize(&img, size, size, image::imageops::Lanczos3);
            resized
                .save(out_dir.join(name))
                .map_err(|e| format!("写入图标失败: {}", e))?;
        }
    }
    Ok(())
}

/// 清理 res 目录下所有 drawable 子目录中的指定文件名。
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
        // 只处理 drawable-* 目录和 drawable 目录
        if !name_str.starts_with("drawable") || !entry.path().is_dir() {
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
        .join(crate::utils::android_project_mod::MODULE_NAME)
        .join("src/main/res");

    // 先清理 SDK 模板自带的旧启动图和背景 XML，避免残留
    clean_drawable_files(&res_dir, &["unipack_splash_image.png"])?;
    // 清理旧的自定义背景 drawable XML
    let old_bg_xml = res_dir.join("drawable").join("unipack_splash.xml");
    if old_bg_xml.exists() {
        let _ = std::fs::remove_file(&old_bg_xml);
    }

    let Some(config) = splashscreen else {
        write_default_android_splashscreen(&res_dir)?;
        emit_log(
            window,
            "info",
            "manifest 未配置 Android 启动图，已使用默认白底",
            None,
        );
        return Ok(());
    };

    if config.android.is_empty() {
        write_default_android_splashscreen(&res_dir)?;
        emit_log(
            window,
            "info",
            "manifest 未提供 Android 启动图密度资源，已使用默认白底",
            None,
        );
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
            "unipack_splash_image.9.png"
        } else {
            "unipack_splash_image.png"
        };
        let target = res_dir.join(drawable_dir).join(target_name);
        crate::utils::fs::copy_file(&source_path, &target)
            .map_err(|e| format!("复制 Android 启动图失败 {}: {}", source_path.display(), e))?;
        copied += 1;
    }

    if copied > 0 {
        write_android_window_background(&res_dir, "@drawable/unipack_splash_image")?;
        emit_log(
            window,
            "info",
            &format!("已导入 {} 张 Android 自定义启动图", copied),
            None,
        );
    } else {
        write_default_android_splashscreen(&res_dir)?;
        emit_log(
            window,
            "warn",
            "Android 自定义启动图均未找到，已使用默认白底",
            None,
        );
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

pub fn write_default_android_splashscreen(res_dir: &Path) -> Result<(), String> {
    write_android_window_background(res_dir, "@android:color/white")?;
    Ok(())
}

pub fn write_android_window_background(res_dir: &Path, drawable_ref: &str) -> Result<(), String> {
    let drawable_dir = res_dir.join("drawable");
    crate::utils::fs::ensure_directory(&drawable_dir).map_err(|e| e.to_string())?;
    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<layer-list xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:drawable="{}" />
</layer-list>
"#,
        drawable_ref
    );
    crate::utils::fs::write_string_to_file(&drawable_dir.join("unipack_splash.xml"), &content)
        .map_err(|e| format!("写入 Android 启动背景资源失败: {}", e))
}
