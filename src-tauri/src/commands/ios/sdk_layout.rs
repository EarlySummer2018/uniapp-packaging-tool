//! iOS 离线 SDK 布局解析。

use std::path::{Path, PathBuf};

pub fn resolve_ios_sdk_project(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("iOS SDK 路径不存在: {}", path.display()));
    }

    let mut checked = Vec::new();
    for candidate in sdk_candidates(path) {
        push_unique(&mut checked, candidate.clone());
        if is_hbuilder_hello_project(&candidate) {
            return Ok(canonicalize_or_self(&candidate));
        }
        if let Some(child) = find_hbuilder_hello_child(&candidate) {
            return Ok(canonicalize_or_self(&child));
        }
    }

    Err(format!(
        "DCloud iOS 离线 SDK 中未找到 HBuilder-Hello* Xcode 工程。已检查: {}",
        checked
            .iter()
            .take(12)
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

pub fn resolve_ios_sdk_root(path: &Path) -> Result<PathBuf, String> {
    let project = resolve_ios_sdk_project(path)?;
    let Some(root) = project.parent() else {
        return Err(format!(
            "DCloud iOS 离线 SDK 工程路径异常: {}",
            project.display()
        ));
    };
    let root = canonicalize_or_self(root);
    if !root.join("SDK").is_dir() {
        return Err(format!(
            "DCloud iOS 离线 SDK 缺少 SDK 支持目录: {}",
            root.join("SDK").display()
        ));
    }
    if !root.join("SDK/Libs").is_dir() {
        return Err(format!(
            "DCloud iOS 离线 SDK 缺少 SDK/Libs 目录: {}",
            root.join("SDK/Libs").display()
        ));
    }
    if !root.join("SDK/Bundles").is_dir() {
        return Err(format!(
            "DCloud iOS 离线 SDK 缺少 SDK/Bundles 目录: {}",
            root.join("SDK/Bundles").display()
        ));
    }
    Ok(root)
}

fn sdk_candidates(path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    push_unique(&mut candidates, path.to_path_buf());
    if let Some(parent) = path.parent() {
        push_unique(&mut candidates, parent.to_path_buf());
    }
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut children = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect::<Vec<_>>();
            children.sort();
            for child in children {
                push_unique(&mut candidates, child);
            }
        }
    }
    candidates
}

fn find_hbuilder_hello_child(root: &Path) -> Option<PathBuf> {
    let mut children = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    children.sort();
    children
        .into_iter()
        .find(|path| is_hbuilder_hello_project(path))
}

fn is_hbuilder_hello_project(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.starts_with("HBuilder-Hello") && contains_xcodeproj(path)
}

fn contains_xcodeproj(path: &Path) -> bool {
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("xcodeproj"))
}

fn canonicalize_or_self(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|item| item == &path) {
        paths.push(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_hbuilder_hello_under_sdk_root() {
        let root = std::env::temp_dir().join(format!("unipack-ios-sdk-{}", uuid::Uuid::new_v4()));
        let hello = root.join("HBuilder-HelloUniApp");
        std::fs::create_dir_all(hello.join("HBuilder.xcodeproj")).unwrap();

        let found = resolve_ios_sdk_project(&root).unwrap();

        assert_eq!(found, hello.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn normalizes_ios_sdk_child_project_to_sdk_root() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-sdk-root-{}", uuid::Uuid::new_v4()));
        let hello = root.join("HBuilder-HelloUniApp");
        std::fs::create_dir_all(hello.join("HBuilder.xcodeproj")).unwrap();
        std::fs::create_dir_all(root.join("SDK/Libs")).unwrap();
        std::fs::create_dir_all(root.join("SDK/Bundles")).unwrap();

        let normalized = resolve_ios_sdk_root(&hello).unwrap();

        assert_eq!(normalized, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn ios_sdk_root_requires_support_layout() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-sdk-missing-{}", uuid::Uuid::new_v4()));
        let hello = root.join("HBuilder-HelloUniApp");
        std::fs::create_dir_all(hello.join("HBuilder.xcodeproj")).unwrap();
        std::fs::create_dir_all(root.join("SDK/Libs")).unwrap();

        let err = resolve_ios_sdk_root(&root).unwrap_err();

        assert!(err.contains("SDK/Bundles"));
        let _ = std::fs::remove_dir_all(root);
    }
}
