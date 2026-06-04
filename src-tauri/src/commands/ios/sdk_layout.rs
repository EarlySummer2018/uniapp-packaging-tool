//! iOS SDK 布局解析与 Xcode 工程检测

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// 解析 iOS 离线 SDK 根目录（HBuilder-Hello* 工程的父目录）
pub fn resolve_ios_sdk_root(path: &Path) -> Result<PathBuf, String> {
    let project = resolve_ios_sdk_project(path)?;
    let root = project.parent().unwrap_or(&project);
    Ok(canonicalize_or_self(root))
}

/// 解析 iOS 离线 SDK 中的 HBuilder-Hello* Xcode 工程路径
pub fn resolve_ios_sdk_project(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    let mut checked = Vec::new();
    for root in ios_sdk_root_candidates(path) {
        if is_ios_hello_project(&root) {
            return Ok(canonicalize_or_self(&root));
        }
        push_unique_path(&mut checked, root.clone());

        if let Some(project) = find_ios_hello_project_child(&root) {
            return Ok(canonicalize_or_self(&project));
        }
        push_unique_path(&mut checked, root.join("HBuilder-Hello*"));
    }

    Err(format!(
        "DCloud iOS 离线 SDK 中未找到 HBuilder-Hello* Xcode 工程。已检查: {}",
        format_path_list(&checked)
    ))
}

fn ios_sdk_root_candidates(path: &Path) -> Vec<PathBuf> {
    generic_root_candidates(path)
}

fn generic_root_candidates(path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    push_unique_path(&mut candidates, path.to_path_buf());

    if let Some(parent) = path.parent() {
        push_unique_path(&mut candidates, parent.to_path_buf());
        if let Some(grandparent) = parent.parent() {
            push_unique_path(&mut candidates, grandparent.to_path_buf());
        }
    }

    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut children = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|child| child.is_dir())
                .collect::<Vec<_>>();
            children.sort();
            for child in children {
                push_unique_path(&mut candidates, child);
            }
        }
    }

    candidates
}

fn find_ios_hello_project_child(root: &Path) -> Option<PathBuf> {
    let mut children = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|child| child.is_dir())
        .collect::<Vec<_>>();
    children.sort();
    children
        .into_iter()
        .find(|child| is_ios_hello_project(child))
}

fn is_ios_hello_project(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.starts_with("HBuilder-Hello") && has_xcode_project(path)
}

fn has_xcode_project(path: &Path) -> bool {
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            let entry_path = entry.path();
            matches!(
                entry_path.extension().and_then(|ext| ext.to_str()),
                Some("xcodeproj" | "xcworkspace")
            )
        })
}

fn canonicalize_or_self(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn format_path_list(paths: &[PathBuf]) -> String {
    let mut labels = paths
        .iter()
        .take(12)
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    if paths.len() > 12 {
        labels.push(format!("另有 {} 个目录", paths.len() - 12));
    }
    labels.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_parent_selection_finds_hello_project() {
        let root = unique_temp_dir("unipack-ios-sdk");
        let hello = root.join("HBuilder-HelloUniApp");
        std::fs::create_dir_all(hello.join("Demo.xcodeproj")).unwrap();

        let found = resolve_ios_sdk_project(&root).unwrap();
        let saved_root = resolve_ios_sdk_root(&root).unwrap();

        assert_eq!(found, hello.canonicalize().unwrap());
        assert_eq!(saved_root, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }
}
