use std::path::Path;

use super::pbxproj::{register_pbx_linked_files, IosPbxLinkedFile};

const POD_SDK_HEADER_PATHS: [&str; 2] = ["$(SRCROOT)/../SDK/inc", "$(SRCROOT)/../SDK/inc/**"];

pub(super) fn ensure_ios_pod_core_libraries(project_file: &Path) -> Result<usize, String> {
    let libraries = [
        IosPbxLinkedFile::local_static("liblibPDRCore.a"),
        IosPbxLinkedFile::local_static("libcoreSupport.a"),
    ];
    register_pbx_linked_files(project_file, &libraries)
}

pub(super) fn ensure_ios_pod_header_search_paths(project_file: &Path) -> Result<usize, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let (updated, patched_count) = ensure_header_search_paths(&content, &POD_SDK_HEADER_PATHS);
    if patched_count > 0 {
        std::fs::write(&pbxproj, updated)
            .map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    }
    Ok(patched_count)
}

pub(crate) fn ensure_header_search_paths(content: &str, paths: &[&str]) -> (String, usize) {
    let pattern = regex::Regex::new(r"(?s)(HEADER_SEARCH_PATHS\s*=\s*\(\n)(?P<body>.*?)(\n\s*\);)")
        .expect("valid header search paths regex");
    let mut patched_count = 0usize;
    let updated = pattern
        .replace_all(content, |captures: &regex::Captures| {
            let body = captures.name("body").map_or("", |value| value.as_str());
            let missing = paths
                .iter()
                .copied()
                .filter(|path| !body.contains(path))
                .collect::<Vec<_>>();
            if missing.is_empty() {
                return captures
                    .get(0)
                    .map_or(String::new(), |value| value.as_str().to_string());
            }
            patched_count += 1;
            let mut insert = String::new();
            for path in missing {
                insert.push_str(&format!("\t\t\t\t\t{},\n", quoted_pbx_value(path)));
            }
            format!(
                "{}{}{}{}",
                captures.get(1).map_or("", |value| value.as_str()),
                body,
                insert,
                captures.get(3).map_or("", |value| value.as_str())
            )
        })
        .into_owned();
    (updated, patched_count)
}

fn quoted_pbx_value(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\\\""))
}
