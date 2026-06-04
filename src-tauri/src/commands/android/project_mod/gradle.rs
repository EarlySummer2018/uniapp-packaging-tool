//! Gradle 文件操作：settings.gradle / build.gradle 修改、块解析、依赖校验。

use regex::Regex;
use std::path::Path;
use super::types::BuildModificationContext;

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

pub(crate) fn set_manifest_placeholders(content: &str, placeholders: &str) -> Result<String, String> {
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

pub(crate) fn ensure_signing_configs_block(content: &str, block_content: &str) -> Result<String, String> {
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

pub(crate) fn ensure_android_block_content(content: &str, block_content: &str) -> Result<String, String> {
    let block_content = block_content.trim_matches('\n');
    if block_content.trim().is_empty() {
        return Ok(content.to_string());
    }
    let android = find_required_block(content, "android")?;
    ensure_block_body_content(content, android, block_content)
}

pub(crate) fn ensure_top_level_block_content(content: &str, block_content: &str) -> Result<String, String> {
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

/// 校验 build.gradle 是否包含文档 4.2 节要求的基础依赖库。
/// 缺失时输出警告日志（不阻断构建，因为 SDK 模板可能已以不同方式包含这些依赖）。
pub(crate) fn validate_base_gradle_dependencies(content: &str, file_path: &Path) -> Result<(), String> {
    /// 文档 4.2 节 ① 列出的关键基础依赖（通过依赖坐标中的特征片段匹配）
    const REQUIRED_DEPS: &[(&str, &str)] = &[
        ("androidx.appcompat:appcompat", "AndroidX AppCompat"),
        (
            "androidx.recyclerview:recyclerview",
            "AndroidX RecyclerView",
        ),
        ("com.facebook.fresco:fresco", "Fresco 图片库"),
        ("com.github.bumptech.glide:glide", "Glide 图片加载"),
        ("com.alibaba:fastjson", "FastJSON"),
        ("androidx.webkit:webkit", "AndroidX WebKit"),
        ("net.lingala.zip4j:zip4j", "Zip4J 压缩库"),
        ("fileTree.*libs", "本地 libs 目录引用 (jar/aar)"),
    ];

    let mut warnings = Vec::new();
    for &(pattern, name) in REQUIRED_DEPS {
        if !content.contains(pattern) {
            warnings.push(name);
        }
    }

    if !warnings.is_empty() {
        eprintln!(
            "[WARN] {} 可能缺少以下 UniApp SDK 基础依赖（文档 4.2 节 ① 要求）: {}。\
             如果 SDK 模板已包含这些依赖则可忽略此警告",
            file_path.display(),
            warnings.join(", ")
        );
    }

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

pub(crate) fn find_named_block_from(content: &str, start_at: usize, name: &str) -> Option<GradleBlock> {
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
