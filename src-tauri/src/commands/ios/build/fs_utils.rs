use std::path::{Path, PathBuf};

pub(super) fn clean_copied_project(project_root: &Path) -> Result<(), String> {
    for path in [
        project_root.join("build"),
        project_root.join("DerivedData"),
        project_root.join(".build"),
    ] {
        if path.exists() {
            std::fs::remove_dir_all(&path)
                .map_err(|e| format!("清理 iOS 工程旧构建产物失败 {}: {}", path.display(), e))?;
        }
    }
    Ok(())
}

pub(super) fn find_xcodeproj(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("xcodeproj"))
}

pub(super) fn find_scheme_name(project_file: &Path) -> Option<String> {
    let content = std::fs::read_to_string(project_file.join("project.pbxproj")).ok()?;
    let pattern =
        regex::Regex::new(r#"(?s)isa = PBXNativeTarget;.*?\n\s*name = "?([^";]+)"?;"#).ok()?;
    pattern
        .captures(&content)
        .and_then(|captures| captures.get(1))
        .map(|name| name.as_str().trim().to_string())
        .filter(|name| !name.is_empty())
}

pub(crate) fn find_info_plist(project_root: &Path, project_file: &Path) -> Option<PathBuf> {
    let pbxproj = project_file.join("project.pbxproj");
    if let Ok(content) = std::fs::read_to_string(pbxproj) {
        let re = regex::Regex::new(r#"INFOPLIST_FILE = "?([^";]+)"?;"#).ok()?;
        for cap in re.captures_iter(&content) {
            let rel = cap
                .get(1)?
                .as_str()
                .replace("$(SRCROOT)/", "")
                .replace("${SRCROOT}/", "");
            let candidate = project_root.join(rel.trim_matches('"'));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    find_file_named_skipping_bundles(project_root, "Info.plist")
}

pub(super) fn find_file_named(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_named(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

pub(super) fn find_file_named_skipping_bundles(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_xcode_package_dir(&path) {
                continue;
            }
            if let Some(found) = find_file_named_skipping_bundles(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

pub(super) fn collect_files_named_skipping_bundles(
    dir: &Path,
    name: &str,
    output: &mut Vec<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if !is_xcode_package_dir(&path) {
                collect_files_named_skipping_bundles(&path, name, output);
            }
        } else if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            output.push(path);
        }
    }
}

pub(super) fn find_file_with_ext_skipping_bundles(dir: &Path, ext: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_xcode_package_dir(&path) {
                continue;
            }
            if let Some(found) = find_file_with_ext_skipping_bundles(&path, ext) {
                return Some(found);
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some(ext) {
            return Some(path);
        }
    }
    None
}

pub(super) fn find_file_with_ext(dir: &Path, ext: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_with_ext(&path, ext) {
                return Some(found);
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some(ext) {
            return Some(path);
        }
    }
    None
}

fn is_xcode_package_dir(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("framework" | "xcframework" | "bundle" | "xcodeproj" | "xcworkspace")
    )
}

pub(super) fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }
    PathBuf::from(path)
}

pub(super) fn safe_file_name(value: &str) -> String {
    let cleaned = value.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    if cleaned.trim().is_empty() {
        "ios-build".to_string()
    } else {
        cleaned
    }
}
