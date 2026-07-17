//! 非标准 AAR 解包逻辑
//!
//! 某些第三方 UTS 插件的 AAR 文件内部结构不符合 Android 标准规范
//! （如内嵌 JAR、根目录 ABI 目录等），会导致 Jetifier 转换失败。
//! 本模块检测此类 AAR 并将其解包为散落文件，避免构建错误。

use std::path::Path;

use crate::commands::android::types::{emit_log, UnpackedAarInfo};

/// 检测 AAR 是否具有可能导致 Jetifier 转换失败的非标准内部结构。
///
/// 非标准特征（如 dingrtcbasic.aar）：
/// - 内嵌 JAR/AAR 文件（`libs/*.jar` 或 `libs/*.aar`）
/// - 根目录下存在 ABI 目录（`arm64-v8a/` 等，正常应仅在 `jni/` 下）
pub fn is_nonstandard_aar(aar_path: &Path) -> Result<bool, String> {
    let file = std::fs::File::open(aar_path)
        .map_err(|e| format!("无法打开 AAR 文件 {}: {}", aar_path.display(), e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("AAR 不是有效的 ZIP: {}", e))?;

    const ABI_DIRS: &[&str] = &["arm64-v8a/", "armeabi-v7a/", "armeabi/", "x86/", "x86_64/"];

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name();

        // 内嵌 JAR/AAR（非标准：标准 AAR 的 libs/ 不含编译产物）
        if name.starts_with("libs/") && (name.ends_with(".jar") || name.ends_with(".aar")) {
            return Ok(true);
        }

        // 根目录下的 ABI 目录（非标准：SO 应在 jni/{abi}/ 下）
        for abi in ABI_DIRS {
            if let Some(rest) = name.strip_prefix(abi) {
                if !rest.is_empty() && !name.starts_with("jni/") {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// 将非标准结构的 AAR 解包为散落文件，避免 Jetifier 转换失败。
///
/// 解包映射：
///   `classes.jar`          → `module_dir/libs/classes.jar`          (+ `main_libs/`)
///   `jni/**/*.so`          → `module_dir/src/main/jniLibs/{abi}/*.so` (+ main jniLibs)
///   `res/**`               → `module_dir/src/main/res/`
///   `libs/*.jar`           → `module_dir/libs/{name}.jar`           (+ `main_libs/`)
///   `AndroidManifest.xml`   → `module_dir/src/main/`（仅当不存在时）
pub fn unpack_nonstandard_aar(
    aar_path: &Path,
    module_dir: &Path,
    main_libs: &Path,
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<UnpackedAarInfo, String> {
    let mut extra_jars = Vec::new();
    let original_name = aar_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    emit_log(
        window,
        "warn",
        &format!(
            "检测到非标准AAR结构 ({}), 自动解包以绕过Jetifier转换",
            original_name
        ),
        None,
    );

    let file = std::fs::File::open(aar_path).map_err(|e| format!("无法打开 AAR: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("AAR ZIP 解析失败: {}", e))?;

    // 主模块的 jniLibs 路径（main_libs 是 app/libs/，其父级的 src/main/jniLibs 才是 SO 目标）
    let main_jnilibs = main_libs
        .parent()
        .unwrap_or(main_libs)
        .join("src")
        .join("main")
        .join("jniLibs");

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        // 跳过目录条目
        if name.ends_with('/') {
            continue;
        }

        match name.as_str() {
            "classes.jar" => {
                let dst = module_dir.join("libs").join("classes.jar");
                crate::utils::fs::ensure_directory(dst.parent().unwrap())
                    .map_err(|e| e.to_string())?;
                let mut out = std::fs::File::create(&dst).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                crate::utils::fs::copy_file(&dst, main_libs.join("classes.jar").as_path())
                    .map_err(|e| e.to_string())?;
            }
            n if n.starts_with("jni/") && n.ends_with(".so") => {
                let rest = n.strip_prefix("jni/").unwrap();
                let dst = module_dir.join("src/main/jniLibs").join(rest);
                crate::utils::fs::ensure_directory(dst.parent().unwrap())
                    .map_err(|e| e.to_string())?;
                let mut out = std::fs::File::create(&dst).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                // 同步到主模块
                let main_dst = main_jnilibs.join(rest);
                crate::utils::fs::ensure_directory(main_dst.parent().unwrap())
                    .map_err(|e| e.to_string())?;
                crate::utils::fs::copy_file(&dst, &main_dst).map_err(|e| e.to_string())?;
            }
            n if n.starts_with("res/") => {
                let rest = n.strip_prefix("res/").unwrap();
                let dst = module_dir.join("src/main/res").join(rest);
                crate::utils::fs::ensure_directory(dst.parent().unwrap())
                    .map_err(|e| e.to_string())?;
                let mut out = std::fs::File::create(&dst).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
            n if n.starts_with("libs/") && (n.ends_with(".jar") || n.ends_with(".aar")) => {
                let jar_name = n.strip_prefix("libs/").unwrap();
                let dst = module_dir.join("libs").join(jar_name);
                crate::utils::fs::ensure_directory(dst.parent().unwrap())
                    .map_err(|e| e.to_string())?;
                let mut out = std::fs::File::create(&dst).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                crate::utils::fs::copy_file(&dst, main_libs.join(jar_name).as_path())
                    .map_err(|e| e.to_string())?;
                extra_jars.push(jar_name.to_string());
            }
            "AndroidManifest.xml" => {
                let dst = module_dir.join("src/main/AndroidManifest.xml");
                if !dst.exists() {
                    crate::utils::fs::ensure_directory(dst.parent().unwrap())
                        .map_err(|e| e.to_string())?;
                    let mut out = std::fs::File::create(&dst).map_err(|e| e.to_string())?;
                    std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                }
            }
            _ => {}
        }
    }

    std::fs::remove_file(aar_path)
        .map_err(|e| format!("移除已解包的原始 AAR {} 失败: {}", aar_path.display(), e))?;

    emit_log(
        window,
        "info",
        &format!(
            "非标准AAR {} 已解包: classes.jar + {} 个内嵌库 + native SO",
            original_name,
            extra_jars.len()
        ),
        None,
    );

    Ok(UnpackedAarInfo {
        original_name,
        extra_jars,
    })
}
