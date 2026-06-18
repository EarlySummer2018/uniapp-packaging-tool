//! Android SDK 布局解析与 AAR 匹配

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct AndroidRequiredAar {
    pub display_name: &'static str,
    pub exact_names: &'static [&'static str],
    pub versionless_prefixes: &'static [&'static str],
}

pub const ANDROID_REQUIRED_AARS: &[AndroidRequiredAar] = &[
    AndroidRequiredAar {
        display_name: "lib.5plus.base",
        exact_names: &["lib.5plus.base-release.aar"],
        versionless_prefixes: &["lib.5plus.base"],
    },
    AndroidRequiredAar {
        display_name: "android-gif-drawable",
        exact_names: &["lib.android-gif-drawable-release.aar"],
        versionless_prefixes: &["android-gif-drawable", "lib.android-gif-drawable"],
    },
    AndroidRequiredAar {
        display_name: "uniapp-v8",
        exact_names: &["uniapp-v8-release.aar"],
        versionless_prefixes: &["uniapp-v8"],
    },
    AndroidRequiredAar {
        display_name: "oaid",
        exact_names: &["lib.oaid.release.aar"],
        versionless_prefixes: &["oaid_sdk_", "lib.oaid"],
    },
    AndroidRequiredAar {
        display_name: "install-apk",
        exact_names: &["install-apk-release.aar"],
        versionless_prefixes: &["install-apk"],
    },
    AndroidRequiredAar {
        display_name: "breakpad",
        exact_names: &["lib.breakpad-release.aar"],
        versionless_prefixes: &["breakpad-build", "lib.breakpad"],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidSdkLayout {
    pub root: PathBuf,
    pub integrate_project_dir: PathBuf,
    pub libs_dir: PathBuf,
    pub assets_dir: PathBuf,
    /// SDK/src 目录，包含各模块的 Java 源文件（如 wxapi/WXEntryActivity.java）
    pub src_dir: PathBuf,
}

/// 解析 Android 离线 SDK 布局，验证 HBuilder-Integrate-AS 和 SDK/libs 完整性
pub fn resolve_android_sdk_layout(path: &Path) -> Result<AndroidSdkLayout, String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    let candidates = android_sdk_root_candidates(path);
    let mut checked_integrate_projects = Vec::new();
    let mut checked_libs = Vec::new();
    let mut missing_reports = Vec::new();

    for root in candidates {
        push_unique_path(
            &mut checked_integrate_projects,
            root.join("HBuilder-Integrate-AS"),
        );
        if let Some(layout) = android_layout_from_root(&root) {
            push_unique_path(
                &mut checked_integrate_projects,
                layout.integrate_project_dir.clone(),
            );
            push_unique_path(&mut checked_libs, layout.libs_dir.clone());
            let missing = missing_android_required_aars(&layout.libs_dir);
            if missing.is_empty() {
                return Ok(AndroidSdkLayout {
                    root: canonicalize_or_self(&layout.root),
                    integrate_project_dir: canonicalize_or_self(&layout.integrate_project_dir),
                    libs_dir: canonicalize_or_self(&layout.libs_dir),
                    assets_dir: canonicalize_or_self(&layout.assets_dir),
                    src_dir: canonicalize_or_self(&layout.src_dir),
                });
            }
            missing_reports.push(format!(
                "{} 缺少 {}",
                layout.libs_dir.display(),
                format_missing_android_aars(&missing)
            ));
        } else {
            push_unique_path(&mut checked_libs, root.join("SDK").join("libs"));
            push_unique_path(&mut checked_libs, root.join("libs"));
        }
    }

    if missing_reports.is_empty() {
        Err(format!(
            "未找到完整的 DCloud Android 离线 SDK。需要同时包含 HBuilder-Integrate-AS/simpleDemo 和 SDK/libs。已检查工程: {}。已检查 libs: {}",
            format_path_list(&checked_integrate_projects),
            format_path_list(&checked_libs)
        ))
    } else {
        Err(format!(
            "DCloud Android 离线 SDK 缺少核心 AAR。已检查工程: {}。已检查 libs: {}。缺少: {}",
            format_path_list(&checked_integrate_projects),
            format_path_list(&checked_libs),
            missing_reports.join("; ")
        ))
    }
}

/// 在 libs 目录中查找匹配指定要求的 AAR 文件
pub fn resolve_android_required_aar(
    libs_dir: &Path,
    requirement: &AndroidRequiredAar,
) -> Option<PathBuf> {
    let mut matches = std::fs::read_dir(libs_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            if path.extension().and_then(|ext| ext.to_str()) != Some("aar") {
                return false;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return false;
            };
            if requirement
                .exact_names
                .iter()
                .any(|pattern| android_artifact_name_matches(pattern, name))
            {
                return true;
            }
            requirement
                .versionless_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

/// 判断两个 artifact 名称是否匹配（忽略版本号差异）
pub fn android_artifact_name_matches(pattern: &str, candidate: &str) -> bool {
    if pattern == candidate {
        return true;
    }

    let Some(pattern_ext) = android_artifact_extension(pattern) else {
        return false;
    };
    if android_artifact_extension(candidate) != Some(pattern_ext) {
        return false;
    }

    let Some(pattern_stem) = android_artifact_stem(pattern) else {
        return false;
    };
    let Some(candidate_stem) = android_artifact_stem(candidate) else {
        return false;
    };

    let Some(parts) = versionless_artifact_parts(&pattern_stem) else {
        return false;
    };
    versionless_artifact_parts_match(&parts, &candidate_stem)
}

/// 提取 artifact 的无版本前缀 stem
pub fn android_artifact_versionless_stem(pattern: &str) -> String {
    let stem = android_artifact_stem(pattern).unwrap_or_else(|| pattern.to_string());
    versionless_artifact_parts(&stem)
        .map(|parts| {
            if parts.suffix.is_empty() {
                parts.prefix
            } else if parts.prefix.is_empty() {
                parts.suffix
            } else {
                format!("{}-{}", parts.prefix, parts.suffix)
            }
        })
        .unwrap_or_else(|| stem.trim_end_matches("-release").to_string())
}

// ---- 内部辅助函数 ----

fn android_sdk_root_candidates(path: &Path) -> Vec<PathBuf> {
    let mut candidates = generic_root_candidates(path);
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

fn android_layout_from_root(root: &Path) -> Option<AndroidSdkLayout> {
    let package_root = android_package_root_from_candidate(root);
    let integrate_project_dir = package_root.join("HBuilder-Integrate-AS");
    if !integrate_project_dir.join("simpleDemo").is_dir() {
        return None;
    }

    let sdk_dir = package_root.join("SDK");
    let libs_dir = sdk_dir.join("libs");
    if !libs_dir.is_dir() {
        return None;
    }

    Some(AndroidSdkLayout {
        root: package_root,
        integrate_project_dir,
        libs_dir,
        assets_dir: sdk_dir.join("assets"),
        src_dir: sdk_dir.join("src"),
    })
}

fn android_package_root_from_candidate(candidate: &Path) -> PathBuf {
    if candidate.file_name().and_then(|name| name.to_str()) == Some("libs") {
        if let Some(sdk_dir) = candidate.parent() {
            if sdk_dir.file_name().and_then(|name| name.to_str()) == Some("SDK") {
                if let Some(package_root) = sdk_dir.parent() {
                    return package_root.to_path_buf();
                }
            }
        }
    }

    if candidate.file_name().and_then(|name| name.to_str()) == Some("SDK") {
        if let Some(package_root) = candidate.parent() {
            return package_root.to_path_buf();
        }
    }

    candidate.to_path_buf()
}

struct AndroidArtifactVersionlessParts {
    prefix: String,
    suffix: String,
    wildcard: bool,
}

fn versionless_artifact_parts(stem: &str) -> Option<AndroidArtifactVersionlessParts> {
    if let Some((index, marker)) = first_wildcard_marker(stem) {
        let prefix = trim_artifact_separators(&stem[..index]).to_ascii_lowercase();
        let suffix = trim_artifact_separators(&stem[index + marker.len()..]).to_ascii_lowercase();
        return (!prefix.is_empty() || !suffix.is_empty()).then_some(
            AndroidArtifactVersionlessParts {
                prefix,
                suffix,
                wildcard: true,
            },
        );
    }

    let (start, end) = version_span(stem)?;
    let prefix = trim_artifact_separators(&stem[..start]).to_ascii_lowercase();
    let suffix = stable_suffix_after_version(&stem[end..]);
    (!prefix.is_empty() || !suffix.is_empty()).then_some(AndroidArtifactVersionlessParts {
        prefix,
        suffix,
        wildcard: false,
    })
}

fn versionless_artifact_parts_match(
    parts: &AndroidArtifactVersionlessParts,
    candidate_stem: &str,
) -> bool {
    let candidate = candidate_stem.to_ascii_lowercase();
    if !parts.prefix.is_empty() {
        let Some(rest) = candidate.strip_prefix(&parts.prefix) else {
            return false;
        };
        if !rest.is_empty()
            && !rest
                .chars()
                .next()
                .map(is_artifact_separator)
                .unwrap_or(false)
        {
            return false;
        }
        if parts.suffix.is_empty() && !parts.wildcard {
            return rest
                .chars()
                .find(|ch| !is_artifact_separator(*ch))
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(true);
        }
    }

    if !parts.suffix.is_empty() {
        let Some(before_suffix) = candidate.strip_suffix(&parts.suffix) else {
            return false;
        };
        if !before_suffix.is_empty()
            && !before_suffix
                .chars()
                .last()
                .map(is_artifact_separator)
                .unwrap_or(false)
        {
            return false;
        }
    }

    true
}

fn first_wildcard_marker(stem: &str) -> Option<(usize, &'static str)> {
    ["*", "XXX", "xxx", "x.x", "vx", "Vx", "+"]
        .iter()
        .filter_map(|marker| stem.find(marker).map(|index| (index, *marker)))
        .min_by_key(|(index, _)| *index)
}

fn version_span(stem: &str) -> Option<(usize, usize)> {
    let bytes = stem.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let ch = bytes[index] as char;
        if !ch.is_ascii_digit() || (index > 0 && !is_artifact_separator(bytes[index - 1] as char)) {
            index += 1;
            continue;
        }

        let mut end = index;
        while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
            end += 1;
        }

        let mut groups = 0usize;
        loop {
            let separator_start = end;
            while end < bytes.len() && is_version_separator(bytes[end] as char) {
                end += 1;
            }
            let digits_start = end;
            while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                end += 1;
            }
            if digits_start == end {
                end = separator_start;
                break;
            }
            groups += 1;
        }

        let has_prefix = !trim_artifact_separators(&stem[..index]).is_empty();
        if groups > 0 || has_prefix {
            return Some((index, end));
        }

        index = end.max(index + 1);
    }
    None
}

fn stable_suffix_after_version(raw_suffix: &str) -> String {
    trim_artifact_separators(raw_suffix)
        .split(is_artifact_separator)
        .filter(|token| !token.is_empty())
        .filter(|token| token.chars().any(|ch| ch.is_ascii_alphabetic()))
        .filter(|token| !token.chars().any(|ch| ch.is_ascii_digit()))
        .collect::<Vec<_>>()
        .join("-")
        .to_ascii_lowercase()
}

fn android_artifact_extension(name: &str) -> Option<String> {
    name.rsplit_once('.').and_then(|(_, ext)| match ext {
        "aar" | "jar" => Some(ext.to_string()),
        _ => None,
    })
}

fn android_artifact_stem(name: &str) -> Option<String> {
    name.rsplit_once('.').and_then(|(stem, ext)| {
        if matches!(ext, "aar" | "jar") {
            Some(stem.to_string())
        } else {
            None
        }
    })
}

fn trim_artifact_separators(value: &str) -> &str {
    value.trim_matches(is_artifact_separator)
}

fn is_version_separator(ch: char) -> bool {
    matches!(ch, '.' | '_' | '-')
}

fn is_artifact_separator(ch: char) -> bool {
    matches!(ch, '-' | '_' | '.' | '@' | '+')
}

fn missing_android_required_aars(libs_dir: &Path) -> Vec<&'static AndroidRequiredAar> {
    ANDROID_REQUIRED_AARS
        .iter()
        .filter(|requirement| resolve_android_required_aar(libs_dir, requirement).is_none())
        .collect()
}

fn format_missing_android_aars(missing: &[&AndroidRequiredAar]) -> String {
    missing
        .iter()
        .map(|requirement| {
            if requirement.versionless_prefixes.is_empty() {
                requirement.display_name.to_string()
            } else {
                format!(
                    "{}(文件名前缀: {})",
                    requirement.display_name,
                    requirement.versionless_prefixes.join(" 或 ")
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
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

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }

    fn write_required_aars(libs_dir: &Path, legacy_names: bool) {
        std::fs::create_dir_all(libs_dir).unwrap();
        for requirement in ANDROID_REQUIRED_AARS {
            let name = test_required_aar_name(requirement, legacy_names);
            std::fs::write(libs_dir.join(name), b"aar").unwrap();
        }
    }

    fn write_integrate_project(root: &Path) {
        let integrate = root.join("HBuilder-Integrate-AS");
        std::fs::create_dir_all(integrate.join("simpleDemo")).unwrap();
        std::fs::write(integrate.join("settings.gradle"), "include ':simpleDemo'\n").unwrap();
        std::fs::write(integrate.join("simpleDemo/build.gradle"), "").unwrap();
    }

    fn test_required_aar_name(
        requirement: &AndroidRequiredAar,
        legacy_names: bool,
    ) -> &'static str {
        match requirement.display_name {
            "android-gif-drawable" if !legacy_names => "android-gif-drawable-release@2.3.45.aar",
            "oaid" if !legacy_names => "oaid_sdk_2.3.45.aar",
            "breakpad" if !legacy_names => "breakpad-build-release@2.3.45.aar",
            "android-gif-drawable" => "lib.android-gif-drawable-release.aar",
            "oaid" => "lib.oaid.release.aar",
            "breakpad" => "lib.breakpad-release.aar",
            _ => requirement
                .exact_names
                .first()
                .copied()
                .or_else(|| requirement.versionless_prefixes.first().copied())
                .expect("test requirement should have a name"),
        }
    }

    #[test]
    fn android_package_root_with_sdk_libs_is_supported() {
        let root = unique_temp_dir("unipack-android-sdk-root");
        let libs = root.join("SDK/libs");
        write_integrate_project(&root);
        write_required_aars(&libs, false);
        std::fs::create_dir_all(root.join("SDK/assets/data")).unwrap();

        let layout = resolve_android_sdk_layout(&root).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        assert_eq!(
            layout.integrate_project_dir,
            root.join("HBuilder-Integrate-AS").canonicalize().unwrap()
        );
        assert_eq!(layout.libs_dir, libs.canonicalize().unwrap());
        assert_eq!(
            layout.assets_dir,
            root.join("SDK/assets").canonicalize().unwrap()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_legacy_aar_names_are_supported() {
        let root = unique_temp_dir("unipack-android-sdk-legacy");
        write_integrate_project(&root);
        write_required_aars(&root.join("SDK/libs"), true);
        std::fs::create_dir_all(root.join("SDK/assets")).unwrap();

        let layout = resolve_android_sdk_layout(&root).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_versioned_aar_names_are_supported() {
        let root = unique_temp_dir("unipack-android-sdk-versioned");
        let libs = root.join("SDK/libs");
        write_integrate_project(&root);
        write_required_aars(&libs, false);
        std::fs::create_dir_all(root.join("SDK/assets")).unwrap();

        let layout = resolve_android_sdk_layout(&root).unwrap();
        let gif =
            resolve_android_required_aar(&layout.libs_dir, &ANDROID_REQUIRED_AARS[1]).unwrap();
        let oaid =
            resolve_android_required_aar(&layout.libs_dir, &ANDROID_REQUIRED_AARS[3]).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        assert_eq!(
            gif.file_name().and_then(|n| n.to_str()),
            Some("android-gif-drawable-release@2.3.45.aar")
        );
        assert_eq!(
            oaid.file_name().and_then(|n| n.to_str()),
            Some("oaid_sdk_2.3.45.aar")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_artifact_name_matching_ignores_versions() {
        assert!(android_artifact_name_matches(
            "gtc-3.2.16.0.aar",
            "gtc-3.5.1.0.aar"
        ));
        assert!(android_artifact_name_matches(
            "open_sdk_3.5.12.2_r97423a8_lite.jar",
            "open_sdk_3.5.0.0_dawfrwafr_lite.jar"
        ));
        assert!(android_artifact_name_matches(
            "Android-7.0.1.20230914.jiagu.aar",
            "Android-8.2.0.20260101.jiagu.aar"
        ));
        assert!(!android_artifact_name_matches(
            "open_sdk_3.5.12.2_r97423a8_lite.jar",
            "open_sdk_3.5.0.0_dawfrwafr_full.jar"
        ));
        assert!(!android_artifact_name_matches(
            "gtc-3.2.16.0.aar",
            "gtsdk-3.5.1.0.aar"
        ));
    }

    #[test]
    fn android_artifact_name_matching_supports_wildcard_versions() {
        assert!(android_artifact_name_matches(
            "open_sdk_*_lite.jar",
            "open_sdk_3.5.12.2_r97423a8_lite.jar"
        ));
        assert!(android_artifact_name_matches(
            "openDefault-*.aar",
            "openDefault-12.5.0.aar"
        ));
        assert!(android_artifact_name_matches(
            "aliyun-base-*.aar",
            "aliyun-base-2.3.4.aar"
        ));
        assert!(android_artifact_name_matches(
            "Android-*.jiagu.aar",
            "Android-7.0.1.20230914.jiagu.aar"
        ));
        assert!(!android_artifact_name_matches(
            "open_sdk_*_lite.jar",
            "open_sdk_3.5.12.2_r97423a8_full.jar"
        ));
    }

    #[test]
    fn android_sdk_child_selection_is_supported() {
        let root = unique_temp_dir("unipack-android-sdk-child");
        let sdk = root.join("SDK");
        write_integrate_project(&root);
        write_required_aars(&sdk.join("libs"), false);
        std::fs::create_dir_all(sdk.join("assets")).unwrap();

        let layout = resolve_android_sdk_layout(&sdk).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        assert_eq!(layout.libs_dir, sdk.join("libs").canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_libs_child_selection_is_supported() {
        let root = unique_temp_dir("unipack-android-sdk-libs");
        let sdk = root.join("SDK");
        let libs = sdk.join("libs");
        write_integrate_project(&root);
        write_required_aars(&libs, false);
        std::fs::create_dir_all(sdk.join("assets")).unwrap();

        let layout = resolve_android_sdk_layout(&libs).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        assert_eq!(layout.libs_dir, libs.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_parent_selection_searches_children() {
        let parent = unique_temp_dir("unipack-android-sdk-parent");
        let root = parent.join("Android-SDK@20260414");
        write_integrate_project(&root);
        write_required_aars(&root.join("SDK/libs"), false);
        std::fs::create_dir_all(root.join("SDK/assets")).unwrap();

        let layout = resolve_android_sdk_layout(&parent).unwrap();

        assert_eq!(layout.root, root.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(parent);
    }

    #[test]
    fn android_missing_aar_error_lists_checked_dir_and_candidates() {
        let root = unique_temp_dir("unipack-android-sdk-missing");
        write_integrate_project(&root);
        std::fs::create_dir_all(root.join("SDK/libs")).unwrap();

        let err = resolve_android_sdk_layout(&root).unwrap_err();

        assert!(err.contains(&root.join("SDK/libs").display().to_string()));
        assert!(err.contains("android-gif-drawable"));
        assert!(err.contains("文件名前缀"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn android_missing_integrate_project_error_lists_checked_paths() {
        let root = unique_temp_dir("unipack-android-sdk-no-integrate");
        write_required_aars(&root.join("SDK/libs"), false);

        let err = resolve_android_sdk_layout(&root).unwrap_err();

        assert!(err.contains("HBuilder-Integrate-AS"));
        assert!(err.contains("SDK/libs"));
        let _ = std::fs::remove_dir_all(root);
    }
}
