//! 鸿蒙(HarmonyOS) SDK 布局解析与模板检测

use std::path::{Path, PathBuf};

/// 解析鸿蒙工程模板根目录，通过查找 hvigorw / hvigorw.bat 确认有效性
pub fn resolve_harmony_template_root(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    let mut checked = Vec::new();
    for root in generic_root_candidates(path) {
        if has_harmony_wrapper(&root) {
            return Ok(canonicalize_or_self(&root));
        }
        push_unique_path(&mut checked, root.join(harmony_wrapper_name()));
    }

    Err(format!(
        "Harmony 工程模板中未找到 {}。已检查: {}",
        harmony_wrapper_name(),
        format_path_list(&checked)
    ))
}

fn has_harmony_wrapper(root: &Path) -> bool {
    root.join(harmony_wrapper_name()).exists()
}

fn harmony_wrapper_name() -> &'static str {
    if cfg!(windows) {
        "hvigorw.bat"
    } else {
        "hvigorw"
    }
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
    fn harmony_parent_selection_finds_template() {
        let parent = unique_temp_dir("unipack-harmony-template-parent");
        let template = parent.join("HarmonyTemplate");
        std::fs::create_dir_all(&template).unwrap();
        std::fs::write(template.join(harmony_wrapper_name()), b"#!/bin/sh").unwrap();

        let found = resolve_harmony_template_root(&parent).unwrap();

        assert_eq!(found, template.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(parent);
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }
}
