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
    let content = if classic_linker_available() {
        append_pbx_build_setting_flag(&content, "OTHER_LDFLAGS", "-ld_classic")
    } else {
        content
    };
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

fn classic_linker_available() -> bool {
    std::process::Command::new("xcrun")
        .args(["--find", "ld-classic"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub(super) fn append_pbx_build_setting_flag(content: &str, key: &str, flag: &str) -> String {
    let pattern = regex::Regex::new(&format!(
        r"(?m)^(\s*{}\s*=\s*)([^;]*)(;)",
        regex::escape(key)
    ))
    .expect("valid pbx setting regex");
    pattern
        .replace_all(content, |caps: &regex::Captures| {
            let value = caps.get(2).map(|value| value.as_str()).unwrap_or_default();
            if pbx_value_contains_flag(value, flag) {
                return caps
                    .get(0)
                    .map(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
            let trimmed = value.trim();
            let updated = if trimmed.starts_with('"') && trimmed.ends_with('"') {
                format!("\"{} {}\"", &trimmed[1..trimmed.len() - 1], flag)
            } else if trimmed.is_empty() {
                render_pbx_value(flag)
            } else {
                render_pbx_value(&format!("{} {}", trimmed, flag))
            };
            format!("{}{}{}", &caps[1], updated, &caps[3])
        })
        .into_owned()
}

fn pbx_value_contains_flag(value: &str, flag: &str) -> bool {
    value
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '"' | ',' | '(' | ')'))
        .any(|token| token == flag)
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

pub(super) fn register_pbx_resources(
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
        _ => "file",
    }
}
