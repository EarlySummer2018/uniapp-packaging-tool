#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppStorageSettings {
    cache_dir: String,
}

fn settings_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("UniPack")
        .join("settings.json")
}

pub fn default_unipack_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".unipack")
}

pub fn read_configured_unipack_home() -> PathBuf {
    let path = settings_file();
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str::<AppStorageSettings>(&content) {
            if !settings.cache_dir.trim().is_empty() {
                return expand_home(&settings.cache_dir);
            }
        }
    }
    default_unipack_home()
}

pub fn save_configured_unipack_home(path: &Path) -> Result<()> {
    let settings = AppStorageSettings {
        cache_dir: path.to_string_lossy().to_string(),
    };
    if let Some(parent) = settings_file().parent() {
        ensure_directory(parent)?;
    }
    let json = serde_json::to_string_pretty(&settings)?;
    std::fs::write(settings_file(), json)?;
    Ok(())
}

pub fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    PathBuf::from(path)
}

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        ensure_directory(parent)?;
    }
    std::fs::copy(src, dst)?;
    Ok(())
}

pub fn copy_recursive(src: &Path, dst: &Path) -> Result<()> {
    ensure_directory(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_recursive(&src_path, &dst_path)?;
        } else {
            copy_file(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// 递归确保目录树中所有目录均可写。
///
/// 从 zip/压缩包解压的 SDK 工程文件可能带有只读权限，
/// std::fs::copy 会保留这些权限，导致后续写入新文件时触发 EROFS。
/// 本函数在 copy_recursive 之后调用，将目标目录树中的所有目录设为可写。
pub fn ensure_writable_tree(root: &Path) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }

    let mut dirs_to_fix = Vec::new();

    // 先收集所有目录路径（避免迭代时修改）
    fn collect_dirs(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_dirs(&path, out)?;
                    out.push(path);
                }
            }
        }
        // 将当前目录也加入列表（子目录优先，父目录后处理）
        if dir.is_dir() {
            out.push(dir.to_path_buf());
        }
        Ok(())
    }

    collect_dirs(root, &mut dirs_to_fix)?;

    for dir in &dirs_to_fix {
        let _ = make_directory_writable(dir);
    }

    Ok(())
}

#[cfg(unix)]
fn make_directory_writable(dir: &Path) -> std::io::Result<()> {
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn make_directory_writable(dir: &Path) -> std::io::Result<()> {
    let mut permissions = std::fs::metadata(dir)?.permissions();
    if permissions.readonly() {
        permissions.set_readonly(false);
        std::fs::set_permissions(dir, permissions)?;
    }
    Ok(())
}

pub fn remove_recursive(path: &Path) -> Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            remove_recursive(&entry?.path())?;
        }
        std::fs::remove_dir(path)?;
    } else if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn read_file_to_string(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

pub fn write_string_to_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

pub fn compute_sha256_file(path: &Path) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = DefaultHasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        buffer[..n].hash(&mut hasher);
    }
    Ok(format!("{:x}", hasher.finish()))
}

pub fn find_files_by_extension(dir: &Path, extension: &str) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(find_files_by_extension(&path, extension)?);
        } else if path
            .extension()
            .map(|e| e.to_string_lossy() == extension)
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
    Ok(files)
}

pub fn unzip_file(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    ensure_directory(dest_dir)?;
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = dest_dir.join(file.name());
        if file.name().ends_with('/') {
            ensure_directory(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                ensure_directory(p)?;
            }
            let mut outfile = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

pub fn zip_directory(src_dir: &Path, output_zip: &Path) -> Result<()> {
    let file = std::fs::File::create(output_zip)?;
    let mut writer = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    fn add_to_zip(
        writer: &mut zip::ZipWriter<std::fs::File>,
        base: &Path,
        dir: &Path,
        options: zip::write::SimpleFileOptions,
    ) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path.strip_prefix(base)?.to_path_buf();
            if path.is_dir() {
                writer.add_directory(relative.to_string_lossy(), options)?;
                add_to_zip(writer, base, &path, options)?;
            } else {
                writer.start_file(relative.to_string_lossy(), options)?;
                let mut f = std::fs::File::open(&path)?;
                std::io::copy(&mut f, writer)?;
            }
        }
        Ok(())
    }

    add_to_zip(&mut writer, src_dir, src_dir, options)?;
    writer.finish()?;
    Ok(())
}

pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

pub fn is_file_exists(path: &Path) -> bool {
    path.exists()
}

pub fn get_unipack_home() -> std::path::PathBuf {
    read_configured_unipack_home()
}

pub fn get_project_config_dir(project_id: &str) -> std::path::PathBuf {
    get_unipack_home().join("projects").join(project_id)
}

pub fn get_legacy_project_file(project_id: &str) -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("unipack-tool")
        .join("projects")
        .join(format!("{}.json", project_id))
}
