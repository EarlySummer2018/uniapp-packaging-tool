use std::path::Path;

use super::config::{effective_app_name, effective_app_version, effective_app_version_code};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IosPbxLinkKind {
    LocalStaticLibrary,
    LocalFramework,
    LocalXcFramework,
    SystemFramework,
    SystemLibrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IosPbxLinkedFile {
    pub(crate) name: &'static str,
    kind: IosPbxLinkKind,
    weak: bool,
}

impl IosPbxLinkedFile {
    pub(crate) fn local_static(name: &'static str) -> Self {
        Self {
            name,
            kind: IosPbxLinkKind::LocalStaticLibrary,
            weak: false,
        }
    }

    pub(crate) fn local_framework(name: &'static str) -> Self {
        Self {
            name,
            kind: IosPbxLinkKind::LocalFramework,
            weak: false,
        }
    }

    pub(crate) fn local_xcframework(name: &'static str) -> Self {
        Self {
            name,
            kind: IosPbxLinkKind::LocalXcFramework,
            weak: false,
        }
    }

    pub(crate) fn system_framework(name: &'static str) -> Self {
        Self {
            name,
            kind: IosPbxLinkKind::SystemFramework,
            weak: false,
        }
    }

    pub(crate) fn optional_system_framework(name: &'static str) -> Self {
        Self {
            name,
            kind: IosPbxLinkKind::SystemFramework,
            weak: true,
        }
    }

    pub(crate) fn system_library(name: &'static str) -> Self {
        Self {
            name,
            kind: IosPbxLinkKind::SystemLibrary,
            weak: false,
        }
    }

    pub(crate) fn is_local(self) -> bool {
        matches!(
            self.kind,
            IosPbxLinkKind::LocalStaticLibrary
                | IosPbxLinkKind::LocalFramework
                | IosPbxLinkKind::LocalXcFramework
        )
    }

    fn last_known_file_type(self) -> &'static str {
        match self.kind {
            IosPbxLinkKind::LocalStaticLibrary => "archive.ar",
            IosPbxLinkKind::LocalFramework | IosPbxLinkKind::SystemFramework => "wrapper.framework",
            IosPbxLinkKind::LocalXcFramework => "wrapper.xcframework",
            IosPbxLinkKind::SystemLibrary => "\"sourcecode.text-based-dylib-definition\"",
        }
    }

    fn pbx_path(self) -> String {
        match self.kind {
            IosPbxLinkKind::LocalStaticLibrary
            | IosPbxLinkKind::LocalFramework
            | IosPbxLinkKind::LocalXcFramework => {
                format!("../SDK/Libs/{}", self.name)
            }
            IosPbxLinkKind::SystemFramework => {
                format!("System/Library/Frameworks/{}", self.name)
            }
            IosPbxLinkKind::SystemLibrary => format!("usr/lib/{}", self.name),
        }
    }

    fn source_tree(self) -> &'static str {
        match self.kind {
            IosPbxLinkKind::LocalStaticLibrary
            | IosPbxLinkKind::LocalFramework
            | IosPbxLinkKind::LocalXcFramework => "<group>",
            IosPbxLinkKind::SystemFramework | IosPbxLinkKind::SystemLibrary => "SDKROOT",
        }
    }

    fn build_settings(self) -> &'static str {
        if self.weak {
            " settings = {ATTRIBUTES = (Weak, ); };"
        } else {
            ""
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IosPbxFileSpec {
    pub(crate) name: String,
    path: String,
    last_known_file_type: &'static str,
    source_tree: &'static str,
}

impl IosPbxFileSpec {
    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn project_framework(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            last_known_file_type: "wrapper.framework",
            source_tree: "<group>",
        }
    }

    pub(crate) fn project_xcframework(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            last_known_file_type: "wrapper.xcframework",
            source_tree: "<group>",
        }
    }

    pub(crate) fn project_resource(name: impl Into<String>, path: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            last_known_file_type: pbx_resource_file_type(&name),
            name,
            path: path.into(),
            source_tree: "<group>",
        }
    }

    pub(crate) fn system_framework(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            path: format!("System/Library/Frameworks/{}", name),
            name,
            last_known_file_type: "wrapper.framework",
            source_tree: "SDKROOT",
        }
    }

    fn file_reference_line(&self, file_ref: &str) -> String {
        format!(
            "\t\t{} /* {} */ = {{isa = PBXFileReference; lastKnownFileType = {}; name = {}; path = {}; sourceTree = {}; }};\n",
            file_ref,
            self.name,
            self.last_known_file_type,
            render_pbx_value(&self.name),
            render_pbx_value(&self.path),
            render_pbx_value(self.source_tree)
        )
    }
}

pub(crate) fn register_pbx_linked_files(
    project_file: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<usize, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let mut linked_count = 0usize;

    for file in files {
        if content.contains(&format!("/* {} in Frameworks */", file.name)) {
            if file.weak {
                content = ensure_pbx_build_file_weak_linked(&content, file.name);
            }
            continue;
        }

        let existing_file_ref = find_pbx_file_reference_id(&content, file.name);
        let file_ref = existing_file_ref.clone().unwrap_or_else(pbx_object_id);
        let build_ref = pbx_object_id();
        let build_line = format!(
            "\t\t{} /* {} in Frameworks */ = {{isa = PBXBuildFile; fileRef = {} /* {} */;{} }};\n",
            build_ref,
            file.name,
            file_ref,
            file.name,
            file.build_settings()
        );
        content = insert_after_marker(
            &content,
            "/* Begin PBXBuildFile section */\n",
            &build_line,
            "PBXBuildFile section",
        )?;

        if existing_file_ref.is_none() {
            let file_line = format!(
                "\t\t{} /* {} */ = {{isa = PBXFileReference; lastKnownFileType = {}; name = {}; path = {}; sourceTree = {}; }};\n",
                file_ref,
                file.name,
                file.last_known_file_type(),
                render_pbx_value(file.name),
                render_pbx_value(&file.pbx_path()),
                render_pbx_value(file.source_tree())
            );
            content = insert_after_marker(
                &content,
                "/* Begin PBXFileReference section */\n",
                &file_line,
                "PBXFileReference section",
            )?;
            content = insert_into_pbx_list(
                &content,
                r"(?s)(/\* Frameworks \*/ = \{\s*isa = PBXGroup;\s*children = \(\n)",
                &format!("\t\t\t\t{} /* {} */,\n", file_ref, file.name),
                "Frameworks group",
            )?;
        }

        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Frameworks \*/ = \{\s*isa = PBXFrameworksBuildPhase;.*?files = \(\n)",
            &format!("\t\t\t\t{} /* {} in Frameworks */,\n", build_ref, file.name),
            "PBXFrameworksBuildPhase",
        )?;
        linked_count += 1;
    }

    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(linked_count)
}

pub(crate) fn register_pbx_linked_file_specs(
    project_file: &Path,
    files: &[IosPbxFileSpec],
) -> Result<usize, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let mut linked_count = 0usize;

    for file in files {
        if content.contains(&format!("/* {} in Frameworks */", file.name)) {
            continue;
        }

        let existing_file_ref = find_pbx_file_reference_id(&content, &file.name);
        let file_ref = existing_file_ref.clone().unwrap_or_else(pbx_object_id);
        let build_ref = pbx_object_id();
        let build_line = format!(
            "\t\t{} /* {} in Frameworks */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; }};\n",
            build_ref, file.name, file_ref, file.name
        );
        content = insert_after_marker(
            &content,
            "/* Begin PBXBuildFile section */\n",
            &build_line,
            "PBXBuildFile section",
        )?;

        if existing_file_ref.is_none() {
            content = insert_after_marker(
                &content,
                "/* Begin PBXFileReference section */\n",
                &file.file_reference_line(&file_ref),
                "PBXFileReference section",
            )?;
            content = insert_into_pbx_list(
                &content,
                r"(?s)(/\* Frameworks \*/ = \{\s*isa = PBXGroup;\s*children = \(\n)",
                &format!("\t\t\t\t{} /* {} */,\n", file_ref, file.name),
                "Frameworks group",
            )?;
        }

        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Frameworks \*/ = \{\s*isa = PBXFrameworksBuildPhase;.*?files = \(\n)",
            &format!("\t\t\t\t{} /* {} in Frameworks */,\n", build_ref, file.name),
            "PBXFrameworksBuildPhase",
        )?;
        linked_count += 1;
    }

    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(linked_count)
}

pub(crate) fn register_pbx_embedded_frameworks(
    project_file: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<usize, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let (updated, copy_phase_id) = ensure_embed_frameworks_copy_phase(&content)?;
    content = updated;
    let mut embedded_count = 0usize;

    for file in files {
        if content.contains(&format!("/* {} in Embed Frameworks */", file.name)) {
            content = ensure_pbx_build_file_embed_signed(&content, file.name);
            continue;
        }

        let existing_file_ref = find_pbx_file_reference_id(&content, file.name);
        let file_ref = existing_file_ref.clone().unwrap_or_else(pbx_object_id);
        let build_ref = pbx_object_id();
        let build_line = format!(
            "\t\t{} /* {} in Embed Frameworks */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; settings = {{ATTRIBUTES = (CodeSignOnCopy, RemoveHeadersOnCopy, ); }}; }};\n",
            build_ref, file.name, file_ref, file.name
        );
        content = insert_after_marker(
            &content,
            "/* Begin PBXBuildFile section */\n",
            &build_line,
            "PBXBuildFile section",
        )?;

        if existing_file_ref.is_none() {
            let file_line = format!(
                "\t\t{} /* {} */ = {{isa = PBXFileReference; lastKnownFileType = {}; name = {}; path = {}; sourceTree = {}; }};\n",
                file_ref,
                file.name,
                file.last_known_file_type(),
                render_pbx_value(file.name),
                render_pbx_value(&file.pbx_path()),
                render_pbx_value(file.source_tree())
            );
            content = insert_after_marker(
                &content,
                "/* Begin PBXFileReference section */\n",
                &file_line,
                "PBXFileReference section",
            )?;
            content = insert_into_pbx_list(
                &content,
                r"(?s)(/\* Frameworks \*/ = \{\s*isa = PBXGroup;\s*children = \(\n)",
                &format!("\t\t\t\t{} /* {} */,\n", file_ref, file.name),
                "Frameworks group",
            )?;
        }

        content = insert_into_pbx_list(
            &content,
            &format!(
                r"(?s)({} /\* .*? \*/ = \{{\s*isa = PBXCopyFilesBuildPhase;.*?files = \(\n)",
                regex::escape(&copy_phase_id)
            ),
            &format!(
                "\t\t\t\t{} /* {} in Embed Frameworks */,\n",
                build_ref, file.name
            ),
            "Embed Frameworks build phase",
        )?;
        embedded_count += 1;
    }

    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(embedded_count)
}

pub(crate) fn register_pbx_embedded_file_specs(
    project_file: &Path,
    files: &[IosPbxFileSpec],
) -> Result<usize, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let (updated, copy_phase_id) = ensure_embed_frameworks_copy_phase(&content)?;
    content = updated;
    let mut embedded_count = 0usize;

    for file in files {
        if content.contains(&format!("/* {} in Embed Frameworks */", file.name)) {
            content = ensure_pbx_build_file_embed_signed(&content, &file.name);
            continue;
        }

        let existing_file_ref = find_pbx_file_reference_id(&content, &file.name);
        let file_ref = existing_file_ref.clone().unwrap_or_else(pbx_object_id);
        let build_ref = pbx_object_id();
        let build_line = format!(
            "\t\t{} /* {} in Embed Frameworks */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; settings = {{ATTRIBUTES = (CodeSignOnCopy, RemoveHeadersOnCopy, ); }}; }};\n",
            build_ref, file.name, file_ref, file.name
        );
        content = insert_after_marker(
            &content,
            "/* Begin PBXBuildFile section */\n",
            &build_line,
            "PBXBuildFile section",
        )?;

        if existing_file_ref.is_none() {
            content = insert_after_marker(
                &content,
                "/* Begin PBXFileReference section */\n",
                &file.file_reference_line(&file_ref),
                "PBXFileReference section",
            )?;
            content = insert_into_pbx_list(
                &content,
                r"(?s)(/\* Frameworks \*/ = \{\s*isa = PBXGroup;\s*children = \(\n)",
                &format!("\t\t\t\t{} /* {} */,\n", file_ref, file.name),
                "Frameworks group",
            )?;
        }

        content = insert_into_pbx_list(
            &content,
            &format!(
                r"(?s)({} /\* .*? \*/ = \{{\s*isa = PBXCopyFilesBuildPhase;.*?files = \(\n)",
                regex::escape(&copy_phase_id)
            ),
            &format!(
                "\t\t\t\t{} /* {} in Embed Frameworks */,\n",
                build_ref, file.name
            ),
            "Embed Frameworks build phase",
        )?;
        embedded_count += 1;
    }

    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(embedded_count)
}

pub(crate) fn remove_pbx_linked_or_embedded_files(
    project_file: &Path,
    names: &[&str],
) -> Result<usize, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let original = content.clone();
    let mut removed_count = 0usize;

    for name in names {
        let build_file_pattern = regex::Regex::new(&format!(
            r"(?m)^\s*([A-Za-z0-9]{{24}}) /\* {} in (?:Frameworks|Embed Frameworks) \*/ = \{{isa = PBXBuildFile;[^\n]*\}};\n?",
            regex::escape(name)
        ))
        .map_err(|e| e.to_string())?;
        let build_ids = build_file_pattern
            .captures_iter(&content)
            .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
            .collect::<Vec<_>>();
        if build_ids.is_empty() {
            continue;
        }
        removed_count += build_ids.len();
        content = build_file_pattern.replace_all(&content, "").into_owned();

        for build_id in build_ids {
            let phase_ref_pattern = regex::Regex::new(&format!(
                r"(?m)^\s*{} /\* {} in (?:Frameworks|Embed Frameworks) \*/,\n?",
                regex::escape(&build_id),
                regex::escape(name)
            ))
            .map_err(|e| e.to_string())?;
            content = phase_ref_pattern.replace_all(&content, "").into_owned();
        }
    }

    if content != original {
        std::fs::write(&pbxproj, content)
            .map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    }
    Ok(removed_count)
}

pub(crate) fn append_pbx_build_setting_paths(
    project_file: &Path,
    key: &str,
    paths: &[String],
) -> Result<usize, String> {
    if paths.is_empty() {
        return Ok(0);
    }

    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let (updated, changed_count) = append_pbx_build_setting_paths_to_content(&content, key, paths);
    if changed_count > 0 {
        std::fs::write(&pbxproj, updated)
            .map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    }
    Ok(changed_count)
}

pub(super) fn append_pbx_build_setting_paths_to_content(
    content: &str,
    key: &str,
    paths: &[String],
) -> (String, usize) {
    let paths = paths
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return (content.to_string(), 0);
    }

    let mut updated = content.to_string();
    let mut changed_count = 0usize;
    for path in paths {
        let (next, changed) = append_pbx_build_setting_path_to_content(&updated, key, path);
        updated = next;
        changed_count += changed;
    }
    (updated, changed_count)
}

fn append_pbx_build_setting_path_to_content(
    content: &str,
    key: &str,
    path: &str,
) -> (String, usize) {
    let array_pattern = regex::Regex::new(&format!(
        r"(?ms)^([ \t]*{}\s*=\s*\(\n)(.*?)(^[ \t]*\);\n?)",
        regex::escape(key)
    ))
    .expect("valid pbx array setting regex");
    let mut matched_array = false;
    let mut changed_count = 0usize;
    let updated = array_pattern
        .replace_all(content, |caps: &regex::Captures| {
            matched_array = true;
            let full = caps.get(0).map_or("", |value| value.as_str());
            let body = caps.get(2).map_or("", |value| value.as_str());
            if pbx_setting_value_contains_path(body, path) {
                return full.to_string();
            }
            let key_line = caps.get(1).map_or("", |value| value.as_str());
            let indent = key_line
                .chars()
                .take_while(|ch| matches!(ch, ' ' | '\t'))
                .collect::<String>();
            let item_indent = format!("{}\t", indent);
            changed_count += 1;
            format!(
                "{}{}{}{},\n{}",
                key_line,
                body,
                item_indent,
                render_pbx_value(path),
                caps.get(3).map_or("", |value| value.as_str())
            )
        })
        .into_owned();
    if matched_array {
        return (updated, changed_count);
    }

    let line_pattern = regex::Regex::new(&format!(
        r"(?m)^([ \t]*{}[ \t]*=[ \t]*)([^;\n]*)(;\n?)",
        regex::escape(key)
    ))
    .expect("valid pbx line setting regex");
    let mut matched_line = false;
    let mut changed_count = 0usize;
    let updated = line_pattern
        .replace_all(content, |caps: &regex::Captures| {
            matched_line = true;
            let full = caps.get(0).map_or("", |value| value.as_str());
            let existing = caps.get(2).map_or("", |value| value.as_str()).trim();
            if pbx_setting_value_contains_path(existing, path) {
                return full.to_string();
            }
            let prefix = caps.get(1).map_or("", |value| value.as_str());
            let indent = prefix
                .chars()
                .take_while(|ch| matches!(ch, ' ' | '\t'))
                .collect::<String>();
            let item_indent = format!("{}\t", indent);
            changed_count += 1;
            let mut body = String::new();
            if !existing.is_empty() {
                body.push_str(&format!("{}{},\n", item_indent, existing));
            }
            body.push_str(&format!("{}{},\n", item_indent, render_pbx_value(path)));
            format!(
                "{}(\n{}{}){}",
                prefix,
                body,
                indent,
                caps.get(3).map_or(";", |value| value.as_str())
            )
        })
        .into_owned();
    if matched_line {
        return (updated, changed_count);
    }

    insert_pbx_build_setting_path_when_missing(content, key, path)
}

fn insert_pbx_build_setting_path_when_missing(
    content: &str,
    key: &str,
    path: &str,
) -> (String, usize) {
    let mut output = String::with_capacity(content.len() + key.len() + path.len() + 96);
    let mut in_build_settings = false;
    let mut changed_count = 0usize;

    for line in content.lines() {
        if in_build_settings && line.trim() == "};" {
            let indent = line
                .chars()
                .take_while(|ch| matches!(ch, ' ' | '\t'))
                .collect::<String>();
            let item_indent = format!("{}\t", indent);
            output.push_str(&format!("{}{} = (\n", indent, key));
            output.push_str(&format!("{}\"$(inherited)\",\n", item_indent));
            output.push_str(&format!("{}{},\n", item_indent, render_pbx_value(path)));
            output.push_str(&format!("{});\n", indent));
            changed_count += 1;
            in_build_settings = false;
        }
        output.push_str(line);
        output.push('\n');
        if line.contains("buildSettings = {") {
            in_build_settings = true;
        }
    }

    if changed_count == 0 {
        (content.to_string(), 0)
    } else {
        (output, changed_count)
    }
}

fn pbx_setting_value_contains_path(value: &str, path: &str) -> bool {
    let rendered = render_pbx_value(path);
    value.lines().any(|line| {
        let token = line.trim().trim_end_matches(',').trim();
        token == path || token == rendered
    }) || value
        .split_whitespace()
        .map(|token| token.trim().trim_end_matches(',').trim())
        .any(|token| token == path || token == rendered)
}

fn ensure_pbx_build_file_weak_linked(content: &str, name: &str) -> String {
    let pattern = regex::Regex::new(&format!(
        r"(?m)^(\s*[A-Za-z0-9]{{24}} /\* {} in Frameworks \*/ = \{{isa = PBXBuildFile; fileRef = [A-Za-z0-9]{{24}} /\* {} \*/;)([^\n]*)(\}};)$",
        regex::escape(name),
        regex::escape(name)
    ))
    .expect("valid PBXBuildFile regex");
    pattern
        .replace_all(content, |caps: &regex::Captures| {
            let line = caps.get(0).map_or("", |value| value.as_str());
            if line.contains("Weak") {
                return line.to_string();
            }
            format!(
                "{} settings = {{ATTRIBUTES = (Weak, ); }}; {}",
                caps.get(1).map_or("", |value| value.as_str()),
                caps.get(4).map_or("", |value| value.as_str())
            )
        })
        .into_owned()
}

fn ensure_pbx_build_file_embed_signed(content: &str, name: &str) -> String {
    let pattern = regex::Regex::new(&format!(
        r"(?m)^(\s*[A-Za-z0-9]{{24}} /\* {} in Embed Frameworks \*/ = \{{isa = PBXBuildFile; fileRef = [A-Za-z0-9]{{24}} /\* {} \*/;)([^\n]*)(\}};)$",
        regex::escape(name),
        regex::escape(name)
    ))
    .expect("valid PBXBuildFile regex");
    pattern
        .replace_all(content, |caps: &regex::Captures| {
            let line = caps.get(0).map_or("", |value| value.as_str());
            if line.contains("CodeSignOnCopy") {
                return line.to_string();
            }
            format!(
                "{} settings = {{ATTRIBUTES = (CodeSignOnCopy, RemoveHeadersOnCopy, ); }}; {}",
                caps.get(1).map_or("", |value| value.as_str()),
                caps.get(4).map_or("", |value| value.as_str())
            )
        })
        .into_owned()
}

fn ensure_embed_frameworks_copy_phase(content: &str) -> Result<(String, String), String> {
    if let Some(id) = find_embed_frameworks_copy_phase_id(content) {
        return Ok((content.to_string(), id));
    }

    let phase_id = pbx_object_id();
    let phase_block = format!(
        "/* Begin PBXCopyFilesBuildPhase section */\n\t\t{} /* Embed Frameworks */ = {{\n\t\t\tisa = PBXCopyFilesBuildPhase;\n\t\t\tbuildActionMask = 2147483647;\n\t\t\tdstPath = \"\";\n\t\t\tdstSubfolderSpec = 10;\n\t\t\tfiles = (\n\t\t\t);\n\t\t\tname = \"Embed Frameworks\";\n\t\t\trunOnlyForDeploymentPostprocessing = 0;\n\t\t}};\n/* End PBXCopyFilesBuildPhase section */\n",
        phase_id
    );
    let mut updated = if content.contains("/* Begin PBXCopyFilesBuildPhase section */\n") {
        insert_after_marker(
            content,
            "/* Begin PBXCopyFilesBuildPhase section */\n",
            &format!(
                "\t\t{} /* Embed Frameworks */ = {{\n\t\t\tisa = PBXCopyFilesBuildPhase;\n\t\t\tbuildActionMask = 2147483647;\n\t\t\tdstPath = \"\";\n\t\t\tdstSubfolderSpec = 10;\n\t\t\tfiles = (\n\t\t\t);\n\t\t\tname = \"Embed Frameworks\";\n\t\t\trunOnlyForDeploymentPostprocessing = 0;\n\t\t}};\n",
                phase_id
            ),
            "PBXCopyFilesBuildPhase section",
        )?
    } else {
        insert_before_first_existing_marker(
            content,
            &[
                "/* Begin PBXFileReference section */\n",
                "/* Begin PBXFrameworksBuildPhase section */\n",
                "/* Begin PBXGroup section */\n",
            ],
            &phase_block,
            "PBXCopyFilesBuildPhase insertion point",
        )?
    };

    updated = insert_into_pbx_list(
        &updated,
        r"(?s)(/\* Begin PBXNativeTarget section \*/.*?isa = PBXNativeTarget;.*?buildPhases = \(\n)",
        &format!("\t\t\t\t{} /* Embed Frameworks */,\n", phase_id),
        "PBXNativeTarget buildPhases",
    )?;

    Ok((updated, phase_id))
}

fn find_embed_frameworks_copy_phase_id(content: &str) -> Option<String> {
    let comment_pattern = regex::Regex::new(
        r#"(?s)([A-Za-z0-9]{24}) /\* Embed Frameworks \*/ = \{\s*isa = PBXCopyFilesBuildPhase;.*?\};"#,
    )
    .ok()?;
    if let Some(captures) = comment_pattern.captures(content) {
        return captures.get(1).map(|value| value.as_str().to_string());
    }

    let name_pattern = regex::Regex::new(
        r#"(?s)([A-Za-z0-9]{24}) /\* .*? \*/ = \{\s*isa = PBXCopyFilesBuildPhase;.*?name = "?Embed Frameworks"?;.*?\};"#,
    )
    .ok()?;
    name_pattern
        .captures(content)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
}

pub(crate) fn enable_pbx_system_capability(
    project_file: &Path,
    capability: &str,
) -> Result<bool, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let (updated, changed) = enable_pbx_system_capability_in_content(&content, capability)?;
    if changed {
        std::fs::write(&pbxproj, updated)
            .map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    }
    Ok(changed)
}

fn enable_pbx_system_capability_in_content(
    content: &str,
    capability: &str,
) -> Result<(String, bool), String> {
    let capability_pattern = regex::Regex::new(&format!(
        r"(?s)({}\s*=\s*\{{.*?enabled\s*=\s*)\d(\s*;.*?\}};)",
        regex::escape(capability)
    ))
    .map_err(|e| e.to_string())?;
    if capability_pattern.is_match(content) {
        let updated = capability_pattern
            .replace(content, |caps: &regex::Captures| {
                format!(
                    "{}1{}",
                    caps.get(1).map_or("", |value| value.as_str()),
                    caps.get(2).map_or("", |value| value.as_str())
                )
            })
            .into_owned();
        return Ok((updated.clone(), updated != content));
    }

    let capability_block = format!(
        "\t\t\t\t\t{} = {{\n\t\t\t\t\t\tenabled = 1;\n\t\t\t\t\t}};\n",
        capability
    );
    if let Some(matched) = regex::Regex::new(r"(?s)SystemCapabilities\s*=\s*\{\n")
        .map_err(|e| e.to_string())?
        .find(content)
    {
        let mut updated = String::with_capacity(content.len() + capability_block.len());
        updated.push_str(&content[..matched.end()]);
        updated.push_str(&capability_block);
        updated.push_str(&content[matched.end()..]);
        return Ok((updated, true));
    }

    let target_attributes =
        regex::Regex::new(r"(?s)(TargetAttributes\s*=\s*\{\s*[A-Za-z0-9]{24}\s*=\s*\{\n)")
            .map_err(|e| e.to_string())?;
    let Some(matched) = target_attributes.find(content) else {
        return Err("project.pbxproj 缺少 TargetAttributes，无法开启 Xcode 能力".to_string());
    };
    let system_capabilities = format!(
        "\t\t\t\tSystemCapabilities = {{\n{}\t\t\t\t}};\n",
        capability_block
    );
    let mut updated = String::with_capacity(content.len() + system_capabilities.len());
    updated.push_str(&content[..matched.end()]);
    updated.push_str(&system_capabilities);
    updated.push_str(&content[matched.end()..]);
    Ok((updated, true))
}

fn find_pbx_file_reference_id(content: &str, name: &str) -> Option<String> {
    let pattern = regex::Regex::new(&format!(
        r"(?m)^\s*([A-Za-z0-9]{{24}}) /\* {} \*/ = \{{isa = PBXFileReference;",
        regex::escape(name)
    ))
    .ok()?;
    pattern
        .captures(content)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
}

pub(super) fn patch_pbxproj(
    project_file: &Path,
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<bool, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let content = content.replace("io.dcloud.HBuilder", &config.ios.bundle_id);
    let content =
        set_pbx_build_setting(&content, "PRODUCT_BUNDLE_IDENTIFIER", &config.ios.bundle_id);
    let content = set_pbx_build_setting(&content, "DEVELOPMENT_TEAM", &config.ios.team_id);
    let content = set_pbx_build_setting(
        &content,
        "INFOPLIST_KEY_CFBundleDisplayName",
        &effective_app_name(config, manifest_info),
    );
    let content = set_pbx_build_setting(
        &content,
        "MARKETING_VERSION",
        &effective_app_version(config, manifest_info),
    );
    let content = set_pbx_build_setting(
        &content,
        "CURRENT_PROJECT_VERSION",
        &effective_app_version_code(config, manifest_info).to_string(),
    );
    let content = remove_pbx_build_setting_flag(&content, "OTHER_LDFLAGS", "-ld_classic");
    let uses_legacy_simulator_arch = legacy_simulator_x86_64_required(project_file);
    let content = if uses_legacy_simulator_arch {
        set_pbx_build_setting(&content, "\"ARCHS[sdk=iphonesimulator*]\"", "x86_64")
    } else {
        content
    };
    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(uses_legacy_simulator_arch)
}

pub(super) fn legacy_simulator_x86_64_required(project_file: &Path) -> bool {
    project_file
        .parent()
        .and_then(Path::parent)
        .map(|workspace| {
            workspace
                .join("SDK/Libs/DCUniRecord.framework/DCUniRecord")
                .is_file()
        })
        .unwrap_or(false)
}

pub(super) fn remove_pbx_build_setting_flag(content: &str, key: &str, flag: &str) -> String {
    let pattern = regex::Regex::new(&format!(
        r"(?m)^(\s*{}\s*=\s*)([^;]*)(;)",
        regex::escape(key)
    ))
    .expect("valid pbx setting regex");
    pattern
        .replace_all(content, |caps: &regex::Captures| {
            let value = caps.get(2).map(|value| value.as_str()).unwrap_or_default();
            if !pbx_value_contains_flag(value, flag) {
                return caps
                    .get(0)
                    .map(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
            let flags = pbx_flag_tokens(value)
                .into_iter()
                .filter(|token| *token != flag)
                .collect::<Vec<_>>();
            let updated = if flags.is_empty() {
                "\"\"".to_string()
            } else {
                render_pbx_value(&flags.join(" "))
            };
            format!("{}{}{}", &caps[1], updated, &caps[3])
        })
        .into_owned()
}

fn pbx_value_contains_flag(value: &str, flag: &str) -> bool {
    pbx_flag_tokens(value)
        .into_iter()
        .any(|token| token == flag)
}

fn pbx_flag_tokens(value: &str) -> Vec<&str> {
    value
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '"' | ','))
        .filter(|token| !token.is_empty() && *token != "(" && *token != ")")
        .collect()
}

pub(super) fn set_pbx_build_setting(content: &str, key: &str, value: &str) -> String {
    let rendered = render_pbx_value(value);
    let pattern = regex::Regex::new(&format!(r"(?m)^(\s*{}\s*=\s*)[^;]*;", regex::escape(key)))
        .expect("valid pbx setting regex");
    if pattern.is_match(content) {
        return pattern
            .replace_all(content, |caps: &regex::Captures| {
                format!(
                    "{}{};",
                    caps.get(1).map_or("", |value| value.as_str()),
                    rendered
                )
            })
            .into_owned();
    }

    let mut output = String::with_capacity(content.len() + key.len() + value.len() + 64);
    let mut in_build_settings = false;
    for line in content.lines() {
        if in_build_settings && line.trim() == "};" {
            output.push_str(&format!("\t\t\t\t{} = {};\n", key, rendered));
            in_build_settings = false;
        }
        output.push_str(line);
        output.push('\n');
        if line.contains("buildSettings = {") {
            in_build_settings = true;
        }
    }
    output
}

pub(crate) fn raise_pbx_ios_deployment_target(
    project_file: &Path,
    minimum: &str,
) -> Result<bool, String> {
    let pbxproj = project_file.join("project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let pattern = regex::Regex::new(r"(?m)^(\s*IPHONEOS_DEPLOYMENT_TARGET\s*=\s*)([^;]+)(;)")
        .expect("valid deployment target regex");
    let minimum_version = parse_ios_deployment_version(minimum)
        .ok_or_else(|| format!("无效的 iOS 最低版本: {}", minimum))?;
    let mut found = false;
    let mut changed = false;
    let updated = pattern
        .replace_all(&content, |captures: &regex::Captures| {
            found = true;
            let value = captures
                .get(2)
                .map(|value| value.as_str())
                .unwrap_or_default()
                .trim()
                .trim_matches('"');
            let should_raise = parse_ios_deployment_version(value)
                .map(|version| compare_ios_deployment_version(&version, &minimum_version).is_lt())
                .unwrap_or(true);
            if !should_raise {
                return captures
                    .get(0)
                    .map(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
            changed = true;
            format!(
                "{}{}{}",
                captures.get(1).map_or("", |value| value.as_str()),
                render_pbx_value(minimum),
                captures.get(3).map_or("", |value| value.as_str())
            )
        })
        .into_owned();
    if found {
        if !changed {
            return Ok(false);
        }
        std::fs::write(&pbxproj, updated)
            .map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
        return Ok(true);
    }
    let updated = set_pbx_build_setting(&content, "IPHONEOS_DEPLOYMENT_TARGET", minimum);
    if updated == content {
        return Ok(false);
    }
    std::fs::write(&pbxproj, updated).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(true)
}

fn parse_ios_deployment_version(value: &str) -> Option<Vec<u32>> {
    let parts = value
        .split('.')
        .map(str::trim)
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if parts.is_empty() {
        return None;
    }
    Some(parts)
}

fn compare_ios_deployment_version(left: &[u32], right: &[u32]) -> std::cmp::Ordering {
    let max_len = left.len().max(right.len());
    for index in 0..max_len {
        let left_part = left.get(index).copied().unwrap_or_default();
        let right_part = right.get(index).copied().unwrap_or_default();
        match left_part.cmp(&right_part) {
            std::cmp::Ordering::Equal => {}
            ordering => return ordering,
        }
    }
    std::cmp::Ordering::Equal
}

fn render_pbx_value(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '*'))
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}

pub(crate) fn register_pbx_resources(
    project_file: &Path,
    resource_names: &[String],
) -> Result<(), String> {
    if resource_names.is_empty() {
        return Ok(());
    }
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    for name in resource_names {
        if content.contains(&format!("/* {} in Resources */", name)) {
            continue;
        }
        let file_ref = pbx_object_id();
        let build_ref = pbx_object_id();
        let file_type = pbx_resource_file_type(name);
        let path = render_pbx_value(name);
        let build_line = format!(
            "\t\t{} /* {} in Resources */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; }};\n",
            build_ref, name, file_ref, name
        );
        let file_line = format!(
            "\t\t{} /* {} */ = {{isa = PBXFileReference; lastKnownFileType = {}; path = {}; sourceTree = \"<group>\"; }};\n",
            file_ref, name, file_type, path
        );
        content = insert_after_marker(
            &content,
            "/* Begin PBXBuildFile section */\n",
            &build_line,
            "PBXBuildFile section",
        )?;
        content = insert_after_marker(
            &content,
            "/* Begin PBXFileReference section */\n",
            &file_line,
            "PBXFileReference section",
        )?;
        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Supporting Files \*/ = \{\s*isa = PBXGroup;\s*children = \(\n)",
            &format!("\t\t\t\t{} /* {} */,\n", file_ref, name),
            "Supporting Files group",
        )?;
        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Resources \*/ = \{\s*isa = PBXResourcesBuildPhase;.*?files = \(\n)",
            &format!("\t\t\t\t{} /* {} in Resources */,\n", build_ref, name),
            "PBXResourcesBuildPhase",
        )?;
    }
    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))
}

pub(crate) fn register_pbx_resource_file_specs(
    project_file: &Path,
    resources: &[IosPbxFileSpec],
) -> Result<usize, String> {
    if resources.is_empty() {
        return Ok(0);
    }
    let pbxproj = project_file.join("project.pbxproj");
    let mut content = std::fs::read_to_string(&pbxproj)
        .map_err(|e| format!("读取 project.pbxproj 失败: {}", e))?;
    let mut registered_count = 0usize;

    for resource in resources {
        if content.contains(&format!("/* {} in Resources */", resource.name)) {
            continue;
        }
        let file_ref = pbx_object_id();
        let build_ref = pbx_object_id();
        let build_line = format!(
            "\t\t{} /* {} in Resources */ = {{isa = PBXBuildFile; fileRef = {} /* {} */; }};\n",
            build_ref, resource.name, file_ref, resource.name
        );
        content = insert_after_marker(
            &content,
            "/* Begin PBXBuildFile section */\n",
            &build_line,
            "PBXBuildFile section",
        )?;
        content = insert_after_marker(
            &content,
            "/* Begin PBXFileReference section */\n",
            &resource.file_reference_line(&file_ref),
            "PBXFileReference section",
        )?;
        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Supporting Files \*/ = \{\s*isa = PBXGroup;\s*children = \(\n)",
            &format!("\t\t\t\t{} /* {} */,\n", file_ref, resource.name),
            "Supporting Files group",
        )?;
        content = insert_into_pbx_list(
            &content,
            r"(?s)(/\* Resources \*/ = \{\s*isa = PBXResourcesBuildPhase;.*?files = \(\n)",
            &format!(
                "\t\t\t\t{} /* {} in Resources */,\n",
                build_ref, resource.name
            ),
            "PBXResourcesBuildPhase",
        )?;
        registered_count += 1;
    }

    std::fs::write(&pbxproj, content).map_err(|e| format!("写入 project.pbxproj 失败: {}", e))?;
    Ok(registered_count)
}

fn insert_after_marker(
    content: &str,
    marker: &str,
    value: &str,
    description: &str,
) -> Result<String, String> {
    let index = content
        .find(marker)
        .ok_or_else(|| format!("project.pbxproj 缺少 {}", description))?
        + marker.len();
    let mut result = String::with_capacity(content.len() + value.len());
    result.push_str(&content[..index]);
    result.push_str(value);
    result.push_str(&content[index..]);
    Ok(result)
}

fn insert_before_first_existing_marker(
    content: &str,
    markers: &[&str],
    value: &str,
    description: &str,
) -> Result<String, String> {
    let index = markers
        .iter()
        .filter_map(|marker| content.find(marker))
        .min()
        .ok_or_else(|| format!("project.pbxproj 缺少 {}", description))?;
    let mut result = String::with_capacity(content.len() + value.len());
    result.push_str(&content[..index]);
    result.push_str(value);
    result.push_str(&content[index..]);
    Ok(result)
}

fn insert_into_pbx_list(
    content: &str,
    pattern: &str,
    value: &str,
    description: &str,
) -> Result<String, String> {
    let regex = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
    let matched = regex
        .find(content)
        .ok_or_else(|| format!("project.pbxproj 缺少 {}", description))?;
    let mut result = String::with_capacity(content.len() + value.len());
    result.push_str(&content[..matched.end()]);
    result.push_str(value);
    result.push_str(&content[matched.end()..]);
    Ok(result)
}

fn pbx_object_id() -> String {
    uuid::Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(24)
        .collect::<String>()
        .to_uppercase()
}

fn pbx_resource_file_type(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image.png",
        Some("jpg" | "jpeg") => "image.jpeg",
        Some("pdf") => "image.pdf",
        Some("json") => "text.json",
        Some("bundle") => "\"wrapper.plug-in\"",
        _ => "file",
    }
}
