//! Gradle 文件操作：settings.gradle / build.gradle 修改、块解析、依赖校验。

use super::types::BuildModificationContext;
use regex::Regex;
use std::path::Path;

// ============================================================================
// Gradle 转义工具函数
// ============================================================================

pub(crate) fn escape_gradle_single_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

pub(crate) fn escape_gradle_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

// ============================================================================
// settings.gradle 操作 + 通用辅助
// ============================================================================

pub(crate) fn set_or_insert_root_project_name(content: &str, project_name: &str) -> String {
    let escaped = escape_gradle_single_quoted(project_name);
    let replacement = format!("rootProject.name = '{}'", escaped);
    let re =
        Regex::new(r#"(?m)^\s*rootProject\.name\s*=\s*['"][^'"]*['"]\s*$"#).expect("valid regex");
    if re.is_match(content) {
        re.replace(content, replacement).to_string()
    } else if content.contains("pluginManagement") {
        let plugin_mgmt_end = content
            .find('}')
            .and_then(|close_brace| {
                content[close_brace..]
                    .find('\n')
                    .map(|nl_offset| close_brace + nl_offset + 1)
            })
            .unwrap_or(0);
        let (before, after) = content.split_at(plugin_mgmt_end);
        format!(
            "{}{}\n{}",
            before,
            replacement,
            after.trim_start_matches('\n')
        )
    } else {
        format!("{}\n{}", replacement, content.trim_start_matches('\n'))
    }
}

pub(crate) fn ensure_gradle_statement(content: &str, statement: &str) -> String {
    let statement = statement.trim();
    if statement.is_empty() {
        return content.to_string();
    }
    if gradle_statement_exists(content, statement) {
        return content.to_string();
    }
    append_statement(content, statement)
}

fn gradle_statement_exists(content: &str, statement: &str) -> bool {
    statement.lines().all(|line| {
        content
            .lines()
            .any(|existing| existing.trim() == line.trim())
    })
}

fn append_statement(content: &str, statement: &str) -> String {
    let mut result = content.trim_end().to_string();
    if !result.is_empty() {
        result.push('\n');
    }
    result.push_str(statement);
    result.push('\n');
    result
}

/// 将仓库注入到 settings.gradle 的 dependencyResolutionManagement.repositories 块中。
/// PREFER_SETTINGS 模式下，Gradle 只从此处读取仓库声明，忽略 build.gradle 中的 allprojects。
pub(crate) fn ensure_repositories_in_drm(content: &str, repositories: &[String]) -> String {
    let Some(drm) = find_named_block(content, "dependencyResolutionManagement", 0) else {
        // 不存在 dependencyResolutionManagement 块，创建一个完整的块并追加到文件开头
        let repos = repositories
            .iter()
            .map(|repo| format!("        {}", repo.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        let block = format!(
            "dependencyResolutionManagement {{\n    repositoriesMode.set(RepositoriesMode.PREFER_SETTINGS)\n    repositories {{\n{}\n    }}\n}}\n",
            repos
        );
        return prepend_statement(content, &block);
    };

    let Some(repositories_block) = find_named_block(content, "repositories", drm.open_brace) else {
        // dependencyResolutionManagement 存在但内部没有 repositories 块，插入一个
        let repos = repositories
            .iter()
            .map(|repo| format!("    {}", repo.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        return insert_before_index(
            content,
            drm.close_brace,
            &format!("\n    repositories {{\n{}\n    }}\n", repos),
        );
    };

    // 已有 repositories 块，补充缺失的仓库声明
    let existing_body = &content[repositories_block.open_brace + 1..repositories_block.close_brace];
    let missing = repositories
        .iter()
        .map(|repo| repo.trim())
        .filter(|repo| !repo.is_empty())
        .filter(|repo| !existing_body.contains(*repo))
        .map(|repo| format!("        {}", repo))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        content.to_string()
    } else {
        insert_before_index(
            content,
            repositories_block.close_brace,
            &format!("\n{}", missing.join("\n")),
        )
    }
}

fn prepend_statement(content: &str, statement: &str) -> String {
    format!("{}\n{}", statement, content)
}

pub(crate) fn ensure_repositories_in_allprojects(content: &str, repositories: &[String]) -> String {
    let Some(allprojects) = find_named_block(content, "allprojects", 0) else {
        let repos = repositories
            .iter()
            .map(|repo| format!("        {}", repo.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        let block = format!(
            "allprojects {{\n    repositories {{\n{}\n    }}\n}}\n",
            repos
        );
        return append_statement(content, &block);
    };

    let Some(repositories_block) =
        find_named_block(content, "repositories", allprojects.open_brace)
    else {
        let block = repositories
            .iter()
            .map(|repo| format!("    {}", repo.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        return insert_before_index(
            content,
            allprojects.close_brace,
            &format!("\n    repositories {{\n{}\n    }}\n", block),
        );
    };

    let existing_body = &content[repositories_block.open_brace + 1..repositories_block.close_brace];
    let missing = repositories
        .iter()
        .map(|repo| repo.trim())
        .filter(|repo| !repo.is_empty())
        .filter(|repo| !existing_body.contains(*repo))
        .map(|repo| format!("        {}", repo))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        content.to_string()
    } else {
        insert_before_index(
            content,
            repositories_block.close_brace,
            &format!("\n{}", missing.join("\n")),
        )
    }
}

pub(crate) fn remove_allprojects_repositories(content: &str) -> String {
    let mut result = content.to_string();
    loop {
        let Some(allprojects) = find_named_block(&result, "allprojects", 0) else {
            break;
        };
        let repositories = find_named_block(&result, "repositories", allprojects.open_brace)
            .filter(|block| block.open_brace < allprojects.close_brace);
        let Some(repositories) = repositories else {
            break;
        };

        result = remove_named_gradle_block(&result, "repositories", repositories);
        if let Some(updated_allprojects) = find_named_block(&result, "allprojects", 0) {
            let body = &result[updated_allprojects.open_brace + 1..updated_allprojects.close_brace];
            if body.trim().is_empty() {
                result = remove_named_gradle_block(&result, "allprojects", updated_allprojects);
            }
        }
    }
    result
}

fn remove_named_gradle_block(content: &str, name: &str, block: GradleBlock) -> String {
    let keyword_start = content[..block.open_brace]
        .rfind(name)
        .unwrap_or(block.open_brace);
    let start = content[..keyword_start]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let mut end = block.close_brace + 1;
    if content.as_bytes().get(end) == Some(&b'\n') {
        end += 1;
    }
    replace_range(content, start, end, "")
}

pub(crate) fn ensure_buildscript_repository(content: &str, repository: &str) -> String {
    let repository = repository.trim();
    if repository.is_empty() {
        return content.to_string();
    }

    let Some(buildscript) = find_named_block(content, "buildscript", 0) else {
        let block = format!(
            "buildscript {{\n    repositories {{\n        {}\n    }}\n}}\n",
            repository
        );
        return prepend_statement(content, &block);
    };

    let repositories_block = find_named_block(content, "repositories", buildscript.open_brace)
        .filter(|block| block.open_brace < buildscript.close_brace);
    let Some(repositories_block) = repositories_block else {
        return insert_before_index(
            content,
            buildscript.close_brace,
            &format!("\n    repositories {{\n        {}\n    }}\n", repository),
        );
    };

    let existing_body = &content[repositories_block.open_brace + 1..repositories_block.close_brace];
    if existing_body.contains(repository) {
        content.to_string()
    } else {
        insert_before_index(
            content,
            repositories_block.close_brace,
            &format!("\n        {}", repository),
        )
    }
}

pub(crate) fn ensure_buildscript_dependency(content: &str, dependency: &str) -> String {
    let dependency = dependency.trim();
    if dependency.is_empty() {
        return content.to_string();
    }

    let Some(buildscript) = find_named_block(content, "buildscript", 0) else {
        let block = format!(
            "buildscript {{\n    dependencies {{\n        {}\n    }}\n}}\n",
            dependency
        );
        return prepend_statement(content, &block);
    };

    let dependencies_block = find_named_block(content, "dependencies", buildscript.open_brace)
        .filter(|block| block.open_brace < buildscript.close_brace);
    let Some(dependencies_block) = dependencies_block else {
        return insert_before_index(
            content,
            buildscript.close_brace,
            &format!("\n    dependencies {{\n        {}\n    }}\n", dependency),
        );
    };

    let existing_body = &content[dependencies_block.open_brace + 1..dependencies_block.close_brace];
    if existing_body.contains(dependency) {
        content.to_string()
    } else {
        insert_before_index(
            content,
            dependencies_block.close_brace,
            &format!("\n        {}", dependency),
        )
    }
}

pub(crate) fn ensure_android_gradle_plugin_supports_kotlin_22(content: &str) -> String {
    const MIN_AGP_FOR_KOTLIN_22: &str = "8.10.0";
    let re = Regex::new(r#"classpath\s+['"]com\.android\.tools\.build:gradle:([^'"]+)['"]"#)
        .expect("valid Android Gradle Plugin regex");

    re.replace_all(content, |caps: &regex::Captures| {
        let full = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
        let current = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        if version_is_less_than(current, MIN_AGP_FOR_KOTLIN_22) {
            full.replace(current, MIN_AGP_FOR_KOTLIN_22)
        } else {
            full.to_string()
        }
    })
    .to_string()
}

fn version_is_less_than(current: &str, minimum: &str) -> bool {
    let current_parts = parse_version_parts(current);
    let minimum_parts = parse_version_parts(minimum);
    for idx in 0..minimum_parts.len().max(current_parts.len()) {
        let current_part = current_parts.get(idx).copied().unwrap_or(0);
        let minimum_part = minimum_parts.get(idx).copied().unwrap_or(0);
        if current_part != minimum_part {
            return current_part < minimum_part;
        }
    }
    false
}

fn parse_version_parts(version: &str) -> Vec<u32> {
    version
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(0)
        })
        .collect()
}

pub(crate) fn ensure_apply_plugin_after_android_application(
    content: &str,
    plugin_id: &str,
) -> String {
    let statement = format!("apply plugin: '{}'", escape_gradle_single_quoted(plugin_id));
    if content.contains(&statement) {
        return content.to_string();
    }

    let android_plugin_re =
        Regex::new(r#"(?m)^\s*apply\s+plugin:\s*['"]com\.android\.application['"]\s*$"#)
            .expect("valid apply plugin regex");
    if let Some(mat) = android_plugin_re.find(content) {
        insert_after_index(content, mat.end(), &format!("\n{}", statement))
    } else {
        prepend_statement(content, &statement)
    }
}

// ============================================================================
// build.gradle 赋值辅助
// ============================================================================

pub(crate) fn replace_or_insert_android_assignment(
    content: &str,
    keys: &[&str],
    replacement: &str,
    _position: super::types::InsertAndroidPosition,
) -> Result<String, String> {
    let android = find_required_block(content, "android")?;
    replace_or_insert_assignment_in_block(content, android, keys, replacement)
}

pub(crate) fn replace_or_insert_default_config_assignment(
    content: &str,
    keys: &[&str],
    replacement: &str,
) -> Result<String, String> {
    let default_config = find_required_block(content, "defaultConfig")?;
    replace_or_insert_assignment_in_block(content, default_config, keys, replacement)
}

fn replace_or_insert_assignment_in_block(
    content: &str,
    block: GradleBlock,
    keys: &[&str],
    replacement: &str,
) -> Result<String, String> {
    let body_start = block.open_brace + 1;
    let body = &content[body_start..block.close_brace];
    let re = Regex::new(&format!(
        r#"(?m)^([ \t]*)({})\s+(.*)$"#,
        keys.iter()
            .map(|key| regex::escape(key))
            .collect::<Vec<_>>()
            .join("|")
    ))
    .map_err(|e| e.to_string())?;

    if let Some(mat) = re.find(body) {
        let line = &body[mat.start()..mat.end()];
        let indent = line
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .collect::<String>();
        let replacement_line = format!("{}{}", indent, replacement);
        let start = body_start + mat.start();
        let end = body_start + mat.end();
        let mut result = String::new();
        result.push_str(&content[..start]);
        result.push_str(&replacement_line);
        result.push_str(&content[end..]);
        return Ok(result);
    }

    let indent = child_indent(content, block);
    Ok(insert_after_index(
        content,
        block.open_brace + 1,
        &format!("\n{}{}", indent, replacement),
    ))
}

// ============================================================================
// build.gradle 块内容操作 + 渲染器
// ============================================================================

pub(crate) fn set_manifest_placeholders(
    content: &str,
    placeholders: &str,
) -> Result<String, String> {
    let placeholders = placeholders.trim_matches('\n');
    if placeholders.trim().is_empty() {
        return Ok(content.to_string());
    }

    let default_config = find_required_block(content, "defaultConfig")?;
    let body_start = default_config.open_brace + 1;
    let body = &content[body_start..default_config.close_brace];
    let re = Regex::new(r#"(?s)\n?[ \t]*manifestPlaceholders\s*=\s*\[.*?\]"#).unwrap();
    if let Some(mat) = re.find(body) {
        let start = body_start + mat.start();
        let end = body_start + mat.end();
        return Ok(replace_range(
            content,
            start,
            end,
            &format!("\n{}", indent_block(placeholders, "")),
        ));
    }

    ensure_block_body_content(content, default_config, placeholders)
}

pub(crate) fn ensure_default_config_ndk_abi_filters(
    content: &str,
    abis: &[String],
) -> Result<String, String> {
    let abis = normalized_gradle_values(abis);
    if abis.is_empty() {
        return Ok(content.to_string());
    }

    let abi_line = format!(
        "abiFilters {}",
        abis.iter()
            .map(|abi| format!("'{}'", escape_gradle_single_quoted(abi)))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let default_config = find_required_block(content, "defaultConfig")?;
    let ndk_block = find_named_block(content, "ndk", default_config.open_brace)
        .filter(|block| block.open_brace < default_config.close_brace);

    if let Some(ndk_block) = ndk_block {
        let body_start = ndk_block.open_brace + 1;
        let body = &content[body_start..ndk_block.close_brace];
        let re = Regex::new(r#"(?m)^([ \t]*)abiFilters\s+.*$"#).unwrap();
        if let Some(mat) = re.find(body) {
            let line = &body[mat.start()..mat.end()];
            let indent = line
                .chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>();
            return Ok(replace_range(
                content,
                body_start + mat.start(),
                body_start + mat.end(),
                &format!("{}{}", indent, abi_line),
            ));
        }
        return Ok(insert_before_index(
            content,
            ndk_block.close_brace,
            &format!("\n{}{}", child_indent(content, ndk_block), abi_line),
        ));
    }

    ensure_block_body_content(
        content,
        default_config,
        &format!("ndk {{\n    {}\n}}", abi_line),
    )
}

pub(crate) fn ensure_uts_hooks_class_array(
    content: &str,
    hooks: &[String],
) -> Result<String, String> {
    let mut hooks = normalized_gradle_values(hooks);
    if hooks.is_empty() {
        return Ok(content.to_string());
    }

    let default_config = find_required_block(content, "defaultConfig")?;
    let body_start = default_config.open_brace + 1;
    let body = &content[body_start..default_config.close_brace];
    let re = Regex::new(
        r#"(?m)^([ \t]*)buildConfigField\s+['"]String\[\]['"]\s*,\s*['"]UTSHooksClassArray['"]\s*,\s*.*$"#,
    )
    .unwrap();

    if let Some(caps) = re.captures(body) {
        let mat = caps.get(0).unwrap();
        for hook in parse_hooks_class_array(mat.as_str()) {
            if !hooks.iter().any(|item| item == &hook) {
                hooks.push(hook);
            }
        }
        hooks.sort();
        hooks.dedup();
        let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let replacement = format!("{}{}", indent, render_uts_hooks_class_array_line(&hooks));
        return Ok(replace_range(
            content,
            body_start + mat.start(),
            body_start + mat.end(),
            &replacement,
        ));
    }

    ensure_block_body_content(
        content,
        default_config,
        &render_uts_hooks_class_array_line(&hooks),
    )
}

fn normalized_gradle_values(values: &[String]) -> Vec<String> {
    let mut result = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();
    result.sort();
    result.dedup();
    result
}

fn render_uts_hooks_class_array_line(hooks: &[String]) -> String {
    let value = hooks
        .iter()
        .map(|hook| format!("\"{}\"", escape_gradle_single_quoted(hook)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "buildConfigField 'String[]', 'UTSHooksClassArray', '{{{}}}'",
        value
    )
}

fn parse_hooks_class_array(value: &str) -> Vec<String> {
    let normalized = value.replace("\\\"", "\"");
    let re = Regex::new(r#""([^"]+)""#).unwrap();
    re.captures_iter(&normalized)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|hook| !hook.is_empty())
        .collect()
}

pub(crate) fn ensure_signing_configs_block(
    content: &str,
    block_content: &str,
) -> Result<String, String> {
    let block_content = block_content.trim_matches('\n');
    if block_content.trim().is_empty() {
        return Ok(content.to_string());
    }
    let android = find_required_block(content, "android")?;
    let android_body = &content[android.open_brace + 1..android.close_brace];

    if android_body.contains("signingConfigs") {
        let re = Regex::new(r"(?s)(\s*)signingConfigs\s*\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}")
            .expect("valid regex");
        if re.is_match(content) {
            let indented = indent_block(block_content, &child_indent(content, android));
            return Ok(re
                .replace(content, format!("{}{}", "$1", indented))
                .to_string());
        }
    }

    ensure_block_body_content(content, android, block_content)
}

pub(crate) fn ensure_android_block_content(
    content: &str,
    block_content: &str,
) -> Result<String, String> {
    let block_content = block_content.trim_matches('\n');
    if block_content.trim().is_empty() {
        return Ok(content.to_string());
    }
    let android = find_required_block(content, "android")?;
    ensure_block_body_content(content, android, block_content)
}

pub(crate) fn ensure_top_level_block_content(
    content: &str,
    block_content: &str,
) -> Result<String, String> {
    if content.contains(block_content) {
        Ok(content.to_string())
    } else {
        Ok(append_statement(content, block_content))
    }
}

pub(crate) fn ensure_dependencies_block_content(
    content: &str,
    dependencies: &[String],
) -> Result<String, String> {
    let dependencies = dependencies
        .iter()
        .map(|dep| dep.trim())
        .filter(|dep| !dep.is_empty())
        .filter(|dep| !content.contains(*dep))
        .map(|dep| dep.to_string())
        .collect::<Vec<_>>();
    if dependencies.is_empty() {
        return Ok(content.to_string());
    }

    let Some(block) = find_named_block(content, "dependencies", 0) else {
        let body = dependencies
            .iter()
            .map(|dep| format!("    {}", dep))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(append_statement(
            content,
            &format!("dependencies {{\n{}\n}}\n", body),
        ));
    };

    let lines = dependencies
        .iter()
        .map(|dep| format!("    {}", dep))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(insert_before_index(
        content,
        block.close_brace,
        &format!("\n{}", lines),
    ))
}

pub(crate) fn ensure_build_type_signing_config(
    content: &str,
    build_type: &str,
    signing_config: &str,
) -> Result<String, String> {
    let android = find_required_block(content, "android")?;
    let android_body = &content[android.open_brace + 1..android.close_brace];

    let build_types_re = Regex::new(r"(?m)\bbuildTypes\b\s*\{").expect("valid regex");

    let Some(build_types_match) = build_types_re.find(android_body) else {
        return Ok(content.to_string());
    };

    let build_types_start = android.open_brace + 1 + build_types_match.start();
    let build_types_block = find_named_block_from(content, build_types_start, build_type)
        .ok_or_else(|| format!("buildTypes 中缺少 {} {{ }} 块", build_type))?;

    let replacement = format!("signingConfig signingConfigs.{}", signing_config);
    replace_or_insert_assignment_in_block(
        content,
        build_types_block,
        &["signingConfig"],
        &replacement,
    )
}

fn ensure_block_body_content(
    content: &str,
    block: GradleBlock,
    block_content: &str,
) -> Result<String, String> {
    if content[block.open_brace + 1..block.close_brace].contains(block_content.trim()) {
        return Ok(content.to_string());
    }

    let indented = indent_block(block_content, &child_indent(content, block));
    Ok(insert_before_index(
        content,
        block.close_brace,
        &format!("\n{}", indented),
    ))
}

pub(crate) fn render_signing_configs(ctx: &BuildModificationContext) -> String {
    format!(
        r#"signingConfigs {{
    release {{
        storeFile file('{}')
        keyAlias '{}'
        keyPassword '{}'
        storePassword '{}'
        v1SigningEnabled true
        v2SigningEnabled true
    }}
}}"#,
        escape_gradle_single_quoted(&ctx.keystore_path),
        escape_gradle_single_quoted(&ctx.key_alias),
        escape_gradle_single_quoted(&ctx.key_password),
        escape_gradle_single_quoted(&ctx.store_password)
    )
}

pub(crate) fn render_packaging_options() -> String {
    r#"packagingOptions {
    pickFirst '**/libc++_shared.so'
    pickFirst '**/libjsc.so'
    jniLibs {
        useLegacyPackaging true
    }
}"#
    .to_string()
}

pub(crate) fn render_source_sets() -> String {
    r#"sourceSets {
    main {
        jniLibs.srcDirs = ['libs']
        assets.srcDirs = ['src/main/assets']
    }
}"#
    .to_string()
}

pub(crate) fn render_aapt_options() -> String {
    r#"aaptOptions {
    additionalParameters '--auto-add-overlay'
    ignoreAssetsPattern "!.svn:!.git:.*:!CVS:!thumbs.db:!picasa.ini:!*.scc:*~"
}"#
    .to_string()
}

// ============================================================================
// 依赖校验
// ============================================================================

/// 保留基础依赖校验入口以兼容调用链。
/// 依赖可能由 SDK 模板、本地 libs 或模块逻辑间接提供，这里不再输出非阻断警告。
pub(crate) fn validate_base_gradle_dependencies(
    _content: &str,
    _file_path: &Path,
) -> Result<(), String> {
    Ok(())
}

// ============================================================================
// Gradle 块解析
// ============================================================================

#[derive(Clone, Copy)]
pub(crate) struct GradleBlock {
    open_brace: usize,
    close_brace: usize,
}

pub(crate) fn find_required_block(content: &str, name: &str) -> Result<GradleBlock, String> {
    find_named_block(content, name, 0).ok_or_else(|| format!("Gradle 文件缺少 {} {{ }} 块", name))
}

pub(crate) fn find_named_block(content: &str, name: &str, start_at: usize) -> Option<GradleBlock> {
    let re = Regex::new(&format!(r#"(?m)\b{}\b\s*\{{"#, regex::escape(name))).ok()?;
    let mat = re.find(&content[start_at..])?;
    let open_brace = start_at + mat.end() - 1;
    let close_brace = find_matching_brace(content, open_brace)?;
    Some(GradleBlock {
        open_brace,
        close_brace,
    })
}

pub(crate) fn find_named_block_from(
    content: &str,
    start_at: usize,
    name: &str,
) -> Option<GradleBlock> {
    let re = Regex::new(&format!(r#"(?m)\b{}\b\s*\{{"#, regex::escape(name))).ok()?;
    let mat = re.find(&content[start_at..])?;
    let open_brace = start_at + mat.end() - 1;
    let close_brace = find_matching_brace(content, open_brace)?;
    Some(GradleBlock {
        open_brace,
        close_brace,
    })
}

pub(crate) fn find_matching_brace(content: &str, open_brace: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut idx = open_brace;

    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        let next = bytes.get(idx + 1).map(|b| *b as char);
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            idx += 1;
            continue;
        }
        if in_block_comment {
            if ch == '*' && next == Some('/') {
                in_block_comment = false;
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }
        if !in_single && !in_double && ch == '/' && next == Some('/') {
            in_line_comment = true;
            idx += 2;
            continue;
        }
        if !in_single && !in_double && ch == '/' && next == Some('*') {
            in_block_comment = true;
            idx += 2;
            continue;
        }
        if !in_double && ch == '\'' {
            in_single = !in_single;
            idx += 1;
            continue;
        }
        if !in_single && ch == '"' {
            in_double = !in_double;
            idx += 1;
            continue;
        }
        if !in_single && !in_double {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
        }
        idx += 1;
    }
    None
}

pub(crate) fn child_indent(content: &str, block: GradleBlock) -> String {
    let line_start = content[..block.open_brace]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let parent_indent = content[line_start..block.open_brace]
        .chars()
        .take_while(|ch| ch.is_whitespace())
        .collect::<String>();
    format!("{}    ", parent_indent)
}

pub(crate) fn indent_block(block: &str, indent: &str) -> String {
    block
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn insert_after_index(content: &str, index: usize, insertion: &str) -> String {
    let mut result = String::with_capacity(content.len() + insertion.len());
    result.push_str(&content[..index]);
    result.push_str(insertion);
    result.push_str(&content[index..]);
    result
}

pub(crate) fn insert_before_index(content: &str, index: usize, insertion: &str) -> String {
    insert_after_index(content, index, insertion)
}

// ============================================================================
// 内部工具：字符串范围替换
// ============================================================================

pub(crate) fn replace_range(content: &str, start: usize, end: usize, replacement: &str) -> String {
    let mut result = String::with_capacity(content.len() + replacement.len());
    result.push_str(&content[..start]);
    result.push_str(replacement);
    result.push_str(&content[end..]);
    result
}

// ============================================================================
// settings.gradle pluginManagement 块确保
// ============================================================================

/// 确保 settings.gradle 包含 pluginManagement 块（必须在文件最前面）
pub(crate) fn ensure_plugin_management_block(content: &str) -> String {
    if content.contains("pluginManagement") {
        return content.to_string();
    }

    // 清理原始内容的前导空白行和 BOM，确保 pluginManagement 是第 1 行
    let trimmed_content = content.trim_start_matches('\u{FEFF}').trim_start();

    let plugin_mgmt = r#"pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.PREFER_SETTINGS)
    repositories {
        google()
        mavenCentral()
    }
}

"#;

    format!("{}{}", plugin_mgmt, trimmed_content)
}
