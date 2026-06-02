//! Android 工程补丁器。
//!
//! 构建流程会先从用户配置的 DCloud Android 离线 SDK 复制
//! `HBuilder-Integrate-AS` 到临时工作区，本模块只修改这份工作区副本。

use regex::Regex;
use std::path::{Path, PathBuf};

pub const MODULE_NAME: &str = "simpleDemo";

#[derive(Debug, Clone)]
pub struct BuildModificationContext {
    pub project_name: String,
    pub package_name: String,
    pub appid: String,
    pub dcloud_appkey: String,
    pub app_name: String,
    pub version_code: u32,
    pub version_name: String,
    pub compile_sdk: u32,
    pub target_sdk: u32,
    pub min_sdk: u32,
    pub keystore_path: String,
    pub key_alias: String,
    pub key_password: String,
    pub store_password: String,
    pub android_allow_backup: String,
    pub extra_repositories: Vec<String>,
    pub extra_dependencies: Vec<String>,
    pub plugin_includes: Vec<String>,
    pub plugin_project_dependencies: Vec<String>,
    pub module_permissions: Vec<String>,
    pub module_application_entries: Vec<String>,
    pub module_pandora_entry_intent_filters: Vec<String>,
    pub manifest_placeholders: String,
    pub dependency_excludes: String,
}

pub struct AndroidProjectModifier {
    workspace_dir: PathBuf,
}

impl AndroidProjectModifier {
    pub fn new(workspace_dir: PathBuf) -> Result<Self, String> {
        let simple_demo = workspace_dir.join(MODULE_NAME);
        if !simple_demo.is_dir() {
            return Err(format!(
                "HBuilder-Integrate-AS 工程缺少 {} 模块: {}",
                MODULE_NAME,
                simple_demo.display()
            ));
        }
        Ok(Self { workspace_dir })
    }

    pub fn apply_all_modifications(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        self.modify_settings_gradle(ctx)?;
        self.modify_root_build_gradle(ctx)?;
        self.modify_app_build_gradle(ctx)?;
        self.modify_android_manifest(ctx)?;
        self.modify_strings_xml(ctx)?;
        self.modify_dcloud_control(ctx)?;
        self.ensure_dcloud_properties()?;
        Ok(())
    }

    fn modify_settings_gradle(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self.workspace_dir.join("settings.gradle");
        let mut content = self.read_file(&path)?;

        // 确保 settings.gradle 包含 pluginManagement 块（用于 UTS 插件）
        if !ctx.plugin_includes.is_empty() {
            content = ensure_plugin_management_block(&content);
        }

        content = set_or_insert_root_project_name(&content, &ctx.project_name);
        for include in &ctx.plugin_includes {
            content = ensure_gradle_statement(&content, include);
        }
        content = ensure_gradle_statement(&content, "include ':simpleDemo'");

        self.write_file(&path, &content)
    }

    fn modify_root_build_gradle(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self.workspace_dir.join("build.gradle");
        if !path.exists() {
            return Ok(());
        }

        let mut content = self.read_file(&path)?;
        if !ctx.extra_repositories.is_empty() {
            content = ensure_repositories_in_allprojects(&content, &ctx.extra_repositories);
        }

        self.validate_gradle_syntax(&content, &path)?;
        self.write_file(&path, &content)
    }

    fn modify_app_build_gradle(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self.workspace_dir.join(MODULE_NAME).join("build.gradle");
        let original_content = self.read_file(&path)?;
        let mut content = original_content.clone();

        content = replace_or_insert_android_assignment(
            &content,
            &["namespace"],
            &format!(
                "namespace '{}'",
                escape_gradle_single_quoted(&ctx.package_name)
            ),
            InsertAndroidPosition::Top,
        )?;
        content = replace_or_insert_android_assignment(
            &content,
            &["compileSdk", "compileSdkVersion"],
            &format!("compileSdkVersion {}", ctx.compile_sdk),
            InsertAndroidPosition::Top,
        )?;
        content = replace_or_insert_default_config_assignment(
            &content,
            &["applicationId"],
            &format!(
                "applicationId \"{}\"",
                escape_gradle_double_quoted(&ctx.package_name)
            ),
        )?;
        content = replace_or_insert_default_config_assignment(
            &content,
            &["minSdk", "minSdkVersion"],
            &format!("minSdkVersion {}", ctx.min_sdk),
        )?;
        content = replace_or_insert_default_config_assignment(
            &content,
            &["targetSdk", "targetSdkVersion"],
            &format!("targetSdkVersion {}", ctx.target_sdk),
        )?;
        content = replace_or_insert_default_config_assignment(
            &content,
            &["versionCode"],
            &format!("versionCode {}", ctx.version_code),
        )?;
        content = replace_or_insert_default_config_assignment(
            &content,
            &["versionName"],
            &format!(
                "versionName \"{}\"",
                escape_gradle_double_quoted(&ctx.version_name)
            ),
        )?;

        content = set_manifest_placeholders(&content, &ctx.manifest_placeholders)?;
        content = ensure_android_block_content(&content, &render_signing_configs(ctx))?;
        content = ensure_build_type_signing_config(&content, "debug", "release")?;
        content = ensure_build_type_signing_config(&content, "release", "release")?;
        content = ensure_android_block_content(&content, &render_packaging_options())?;
        content = ensure_android_block_content(&content, &render_source_sets())?;

        if !ctx.dependency_excludes.trim().is_empty() {
            content = ensure_top_level_block_content(&content, ctx.dependency_excludes.trim())?;
        }
        content = ensure_dependencies_block_content(&content, &ctx.plugin_project_dependencies)?;
        content = ensure_dependencies_block_content(&content, &ctx.extra_dependencies)?;

        if !content.contains("com.android.application") {
            return Err(format!(
                "build.gradle 缺少 com.android.application 插件: {}",
                path.display()
            ));
        }
        self.validate_gradle_syntax(&content, &path)?;
        self.write_file(&path, &content)
    }

    fn validate_gradle_syntax(&self, content: &str, file_path: &Path) -> Result<(), String> {
        let mut brace_count = 0i32;
        let mut in_string = false;
        let mut in_char = false;
        let mut in_block_comment = false;

        for (line_idx, line) in content.lines().enumerate() {
            let chars = line.chars().collect::<Vec<_>>();
            let mut col_idx = 0;
            while col_idx < chars.len() {
                let ch = chars[col_idx];
                match ch {
                    '/' if !in_string && !in_char && !in_block_comment => {
                        if let Some(next_ch) = chars.get(col_idx + 1) {
                            match next_ch {
                                '/' => break,
                                '*' => {
                                    in_block_comment = true;
                                    col_idx += 1;
                                }
                                _ => {}
                            }
                        }
                    }
                    '*' if !in_string && !in_char && in_block_comment => {
                        if chars.get(col_idx + 1) == Some(&'/') {
                            in_block_comment = false;
                            col_idx += 1;
                        }
                    }
                    '\'' if !in_string && !in_block_comment => {
                        in_char = !in_char;
                    }
                    '"' if !in_char && !in_block_comment => {
                        in_string = !in_string;
                    }
                    '{' if !in_string && !in_char && !in_block_comment => {
                        brace_count += 1;
                    }
                    '}' if !in_string && !in_char && !in_block_comment => {
                        brace_count -= 1;
                        if brace_count < 0 {
                            return Err(format!(
                                "Gradle 语法错误: {} 第 {} 行有多余的 '}}'",
                                file_path.display(),
                                line_idx + 1
                            ));
                        }
                    }
                    _ => {}
                }
                col_idx += 1;
            }
        }

        if brace_count != 0 {
            Err(format!(
                "Gradle 语法错误: {} 花括号不匹配，缺少 {} 个 '}}' 或存在多余 '{{'",
                file_path.display(),
                brace_count.abs()
            ))
        } else {
            Ok(())
        }
    }

    fn modify_android_manifest(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self
            .workspace_dir
            .join(MODULE_NAME)
            .join("src/main/AndroidManifest.xml");
        let mut content = self.read_file(&path)?;

        content = ensure_manifest_permissions(&content, &ctx.module_permissions)?;
        content =
            set_application_attribute(&content, "android:allowBackup", &ctx.android_allow_backup)?;
        content = ensure_application_entry(
            &content,
            &format!(
                r#"<meta-data android:name="dcloud_appkey" android:value="{}" />"#,
                escape_xml_attr(&ctx.dcloud_appkey)
            ),
            EntryIdentity::MetaData("dcloud_appkey".to_string()),
        )?;
        content = set_meta_data_value(&content, "dcloud_appkey", &ctx.dcloud_appkey)?;
        for entry in &ctx.module_application_entries {
            content = ensure_application_entry(&content, entry, entry_identity(entry))?;
        }
        for filter in &ctx.module_pandora_entry_intent_filters {
            content = ensure_activity_child(
                &content,
                "io.dcloud.PandoraEntryActivity",
                filter,
                child_identity(filter),
            )
            .or_else(|_| {
                ensure_activity_child(
                    &content,
                    "io.dcloud.PandoraEntry",
                    filter,
                    child_identity(filter),
                )
            })?;
        }

        self.write_file(&path, &content)
    }

    fn modify_strings_xml(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self
            .workspace_dir
            .join(MODULE_NAME)
            .join("src/main/res/values/strings.xml");
        let content = self.read_file(&path)?;
        let modified = set_string_resource(&content, "app_name", &ctx.app_name)?;
        self.write_file(&path, &modified)
    }

    fn modify_dcloud_control(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self
            .workspace_dir
            .join(MODULE_NAME)
            .join("src/main/assets/data/dcloud_control.xml");
        let content = self.read_file(&path)?;
        let modified =
            crate::utils::xml::set_xml_attribute(&content, "/apps/app", "appid", &ctx.appid)
                .or_else(|_| {
                    crate::utils::xml::set_xml_attribute(&content, "/hbuilder", "appid", &ctx.appid)
                })
                .map_err(|e| format!("设置 dcloud_control.xml appid 失败: {}", e))?;
        self.write_file(&path, &modified)
    }

    fn ensure_dcloud_properties(&self) -> Result<(), String> {
        let path = self
            .workspace_dir
            .join(MODULE_NAME)
            .join("src/main/assets/data/dcloud_properties.xml");
        if path.exists() {
            return Ok(());
        }

        self.write_file(&path, "<properties>\n</properties>\n")
    }

    fn read_file(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
        }
        std::fs::write(path, content).map_err(|e| format!("写入文件失败 {}: {}", path.display(), e))
    }
}

#[derive(Clone, Copy)]
enum InsertAndroidPosition {
    Top,
}

enum EntryIdentity {
    MetaData(String),
    Component { tag: String, name: String },
    ProviderAuthority(String),
    Comment(String),
    Raw(String),
}

enum ChildIdentity {
    IntentFilterDataScheme(String),
    IntentFilterAction(String),
    Raw(String),
}

fn set_or_insert_root_project_name(content: &str, project_name: &str) -> String {
    let escaped = escape_gradle_single_quoted(project_name);
    let replacement = format!("rootProject.name = '{}'", escaped);
    let re =
        Regex::new(r#"(?m)^\s*rootProject\.name\s*=\s*['"][^'"]*['"]\s*$"#).expect("valid regex");
    if re.is_match(content) {
        re.replace(content, replacement).to_string()
    } else {
        format!("{}\n{}", replacement, content.trim_start_matches('\n'))
    }
}

fn ensure_gradle_statement(content: &str, statement: &str) -> String {
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

fn ensure_repositories_in_allprojects(content: &str, repositories: &[String]) -> String {
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

fn replace_or_insert_android_assignment(
    content: &str,
    keys: &[&str],
    replacement: &str,
    _position: InsertAndroidPosition,
) -> Result<String, String> {
    let android = find_required_block(content, "android")?;
    replace_or_insert_assignment_in_block(content, android, keys, replacement)
}

fn replace_or_insert_default_config_assignment(
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

fn set_manifest_placeholders(content: &str, placeholders: &str) -> Result<String, String> {
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

fn ensure_android_block_content(content: &str, block_content: &str) -> Result<String, String> {
    let block_content = block_content.trim_matches('\n');
    if block_content.trim().is_empty() {
        return Ok(content.to_string());
    }
    let android = find_required_block(content, "android")?;
    ensure_block_body_content(content, android, block_content)
}

fn ensure_top_level_block_content(content: &str, block_content: &str) -> Result<String, String> {
    if content.contains(block_content) {
        Ok(content.to_string())
    } else {
        Ok(append_statement(content, block_content))
    }
}

fn ensure_dependencies_block_content(
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

fn ensure_build_type_signing_config(
    content: &str,
    build_type: &str,
    signing_config: &str,
) -> Result<String, String> {
    let block = find_required_block(content, build_type)?;
    let replacement = format!("signingConfig signingConfigs.{}", signing_config);
    replace_or_insert_assignment_in_block(content, block, &["signingConfig"], &replacement)
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

fn render_signing_configs(ctx: &BuildModificationContext) -> String {
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

fn render_packaging_options() -> String {
    r#"packagingOptions {
    pickFirst '**/libc++_shared.so'
    pickFirst '**/libjsc.so'
    jniLibs {
        useLegacyPackaging true
    }
}"#
    .to_string()
}

fn render_source_sets() -> String {
    r#"sourceSets {
    main {
        jniLibs.srcDirs = ['libs']
        assets.srcDirs = ['src/main/assets']
    }
}"#
    .to_string()
}

#[derive(Clone, Copy)]
struct GradleBlock {
    open_brace: usize,
    close_brace: usize,
}

fn find_required_block(content: &str, name: &str) -> Result<GradleBlock, String> {
    find_named_block(content, name, 0).ok_or_else(|| format!("Gradle 文件缺少 {} {{ }} 块", name))
}

fn find_named_block(content: &str, name: &str, start_at: usize) -> Option<GradleBlock> {
    let re = Regex::new(&format!(r#"(?m)\b{}\b\s*\{{"#, regex::escape(name))).ok()?;
    let mat = re.find(&content[start_at..])?;
    let open_brace = start_at + mat.end() - 1;
    let close_brace = find_matching_brace(content, open_brace)?;
    Some(GradleBlock {
        open_brace,
        close_brace,
    })
}

fn find_matching_brace(content: &str, open_brace: usize) -> Option<usize> {
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

fn child_indent(content: &str, block: GradleBlock) -> String {
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

fn indent_block(block: &str, indent: &str) -> String {
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

fn insert_after_index(content: &str, index: usize, insertion: &str) -> String {
    let mut result = String::with_capacity(content.len() + insertion.len());
    result.push_str(&content[..index]);
    result.push_str(insertion);
    result.push_str(&content[index..]);
    result
}

fn insert_before_index(content: &str, index: usize, insertion: &str) -> String {
    insert_after_index(content, index, insertion)
}

fn ensure_manifest_permissions(content: &str, permissions: &[String]) -> Result<String, String> {
    let manifest_tag = Regex::new(r#"(?s)<manifest\b[^>]*>"#).unwrap();
    let manifest = manifest_tag
        .find(content)
        .ok_or_else(|| "AndroidManifest.xml 缺少 manifest 节点".to_string())?;
    let mut result = content.to_string();
    let missing = permissions
        .iter()
        .map(|permission| permission.trim())
        .filter(|permission| !permission.is_empty())
        .filter(|permission| !permission_fragment_exists(&result, permission))
        .map(|permission| permission.to_string())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(result);
    }
    let insertion = missing
        .iter()
        .map(|permission| format!("    {}", permission))
        .collect::<Vec<_>>()
        .join("\n");
    result = insert_after_index(&result, manifest.end(), &format!("\n{}", insertion));
    Ok(result)
}

fn permission_fragment_exists(content: &str, permission: &str) -> bool {
    if let Some(name) = android_attr_value(permission, "android:name") {
        content.contains(&format!(r#"android:name="{}""#, name))
    } else {
        content.contains(permission)
    }
}

fn set_application_attribute(
    content: &str,
    attr_name: &str,
    attr_value: &str,
) -> Result<String, String> {
    let app = Regex::new(r#"(?s)<application\b[^>]*>"#)
        .unwrap()
        .find(content)
        .ok_or_else(|| "AndroidManifest.xml 缺少 application 节点".to_string())?;
    let tag = app.as_str();
    let attr_re = Regex::new(&format!(r#"{}\s*=\s*"[^"]*""#, regex::escape(attr_name))).unwrap();
    let updated_tag = if attr_re.is_match(tag) {
        attr_re
            .replace(
                tag,
                format!(r#"{}="{}""#, attr_name, escape_xml_attr(attr_value)),
            )
            .to_string()
    } else {
        let end = tag.trim_end_matches('>').trim_end();
        format!(
            "{}\n        {}=\"{}\">",
            end,
            attr_name,
            escape_xml_attr(attr_value)
        )
    };
    Ok(replace_range(content, app.start(), app.end(), &updated_tag))
}

fn ensure_application_entry(
    content: &str,
    entry: &str,
    identity: EntryIdentity,
) -> Result<String, String> {
    if application_entry_exists(content, identity) {
        return Ok(content.to_string());
    }
    let close = content
        .rfind("</application>")
        .ok_or_else(|| "AndroidManifest.xml 缺少 </application>".to_string())?;
    Ok(insert_before_index(
        content,
        close,
        &format!("\n{}\n", indent_xml_fragment(entry, 8)),
    ))
}

fn application_entry_exists(content: &str, identity: EntryIdentity) -> bool {
    match identity {
        EntryIdentity::MetaData(name) => {
            content.contains("<meta-data")
                && content.contains(&format!(r#"android:name="{}""#, name))
        }
        EntryIdentity::Component { tag, name } => {
            content.contains(&format!("<{}", tag))
                && content.contains(&format!(r#"android:name="{}""#, name))
        }
        EntryIdentity::ProviderAuthority(authority) => {
            content.contains("<provider")
                && content.contains(&format!(r#"android:authorities="{}""#, authority))
        }
        EntryIdentity::Comment(text) => content.contains(&text),
        EntryIdentity::Raw(fragment) => content.contains(fragment.trim()),
    }
}

fn set_meta_data_value(content: &str, name: &str, value: &str) -> Result<String, String> {
    let Some(mat) = find_xml_start_tag_with_attr(content, "meta-data", "android:name", name) else {
        return Ok(content.to_string());
    };
    let tag = &content[mat.start..mat.end];
    let value_re = Regex::new(r#"android:value\s*=\s*"[^"]*""#).unwrap();
    let updated = if value_re.is_match(tag) {
        value_re
            .replace(
                tag,
                format!(r#"android:value="{}""#, escape_xml_attr(value)),
            )
            .to_string()
    } else {
        tag.trim_end_matches("/>")
            .trim_end_matches('>')
            .trim_end()
            .to_string()
            + &format!(r#" android:value="{}" />"#, escape_xml_attr(value))
    };
    Ok(replace_range(content, mat.start, mat.end, &updated))
}

fn ensure_activity_child(
    content: &str,
    activity_name: &str,
    child: &str,
    identity: ChildIdentity,
) -> Result<String, String> {
    if activity_child_exists(content, activity_name, identity) {
        return Ok(content.to_string());
    }
    let activity = find_manifest_component(content, "activity", activity_name)
        .ok_or_else(|| format!("AndroidManifest.xml 缺少 Activity: {}", activity_name))?;
    Ok(insert_before_index(
        content,
        activity.end_close_start,
        &format!("\n{}\n", indent_xml_fragment(child, 12)),
    ))
}

fn activity_child_exists(content: &str, activity_name: &str, identity: ChildIdentity) -> bool {
    let Some(activity) = find_manifest_component(content, "activity", activity_name) else {
        return false;
    };
    let body = &content[activity.start..activity.end];
    match identity {
        ChildIdentity::IntentFilterDataScheme(scheme) => {
            body.contains("<intent-filter")
                && body.contains(&format!(r#"android:scheme="{}""#, scheme))
        }
        ChildIdentity::IntentFilterAction(action) => {
            body.contains("<intent-filter")
                && body.contains(&format!(r#"android:name="{}""#, action))
        }
        ChildIdentity::Raw(fragment) => body.contains(fragment.trim()),
    }
}

struct ManifestComponent {
    start: usize,
    end: usize,
    end_close_start: usize,
}

fn find_manifest_component(content: &str, tag: &str, name: &str) -> Option<ManifestComponent> {
    let start = find_xml_start_tag_with_attr(content, tag, "android:name", name)?;
    let start_tag = &content[start.start..start.end];
    if start_tag.trim_end().ends_with("/>") {
        return Some(ManifestComponent {
            start: start.start,
            end: start.end,
            end_close_start: start.end - 2,
        });
    }
    let end_tag = format!("</{}>", tag);
    let close_rel = content[start.end..].find(&end_tag)?;
    let end_close_start = start.end + close_rel;
    Some(ManifestComponent {
        start: start.start,
        end: end_close_start + end_tag.len(),
        end_close_start,
    })
}

fn entry_identity(entry: &str) -> EntryIdentity {
    if let Some(name) = android_attr_value(entry, "android:name") {
        if entry.contains("<meta-data") {
            return EntryIdentity::MetaData(name);
        }
        for tag in ["activity", "service", "receiver", "provider"] {
            if entry.contains(&format!("<{}", tag)) {
                return EntryIdentity::Component {
                    tag: tag.to_string(),
                    name,
                };
            }
        }
    }
    if let Some(authority) = android_attr_value(entry, "android:authorities") {
        return EntryIdentity::ProviderAuthority(authority);
    }
    if entry.trim_start().starts_with("<!--") {
        return EntryIdentity::Comment(entry.trim().to_string());
    }
    EntryIdentity::Raw(entry.trim().to_string())
}

fn child_identity(child: &str) -> ChildIdentity {
    if let Some(scheme) = android_attr_value(child, "android:scheme") {
        return ChildIdentity::IntentFilterDataScheme(scheme);
    }
    if let Some(action) = android_attr_value(child, "android:name") {
        return ChildIdentity::IntentFilterAction(action);
    }
    ChildIdentity::Raw(child.trim().to_string())
}

struct XmlTagRange {
    start: usize,
    end: usize,
}

fn find_xml_start_tag_with_attr(
    content: &str,
    tag: &str,
    attr: &str,
    value: &str,
) -> Option<XmlTagRange> {
    let re = Regex::new(&format!(r#"(?s)<{}\b[^>]*>"#, regex::escape(tag))).ok()?;
    for mat in re.find_iter(content) {
        let fragment = mat.as_str();
        if android_attr_value(fragment, attr).as_deref() == Some(value) {
            return Some(XmlTagRange {
                start: mat.start(),
                end: mat.end(),
            });
        }
    }
    None
}

fn android_attr_value(fragment: &str, attr: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"{}\s*=\s*"([^"]*)""#, regex::escape(attr))).ok()?;
    re.captures(fragment)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
}

fn set_string_resource(content: &str, name: &str, value: &str) -> Result<String, String> {
    let re = Regex::new(&format!(
        r#"(?s)<string\s+name="{}">.*?</string>"#,
        regex::escape(name)
    ))
    .unwrap();
    let replacement = format!(
        r#"<string name="{}">{}</string>"#,
        name,
        escape_xml_text(value)
    );
    if re.is_match(content) {
        Ok(re.replace(content, replacement).to_string())
    } else {
        let close = content
            .rfind("</resources>")
            .ok_or_else(|| "strings.xml 缺少 </resources>".to_string())?;
        Ok(insert_before_index(
            content,
            close,
            &format!("    {}\n", replacement),
        ))
    }
}

fn replace_range(content: &str, start: usize, end: usize, replacement: &str) -> String {
    let mut result = String::with_capacity(content.len() + replacement.len());
    result.push_str(&content[..start]);
    result.push_str(replacement);
    result.push_str(&content[end..]);
    result
}

fn indent_xml_fragment(fragment: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    fragment
        .trim()
        .lines()
        .map(|line| format!("{}{}", indent, line.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_gradle_single_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn escape_gradle_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 确保 settings.gradle 包含 pluginManagement 块
fn ensure_plugin_management_block(content: &str) -> String {
    if content.contains("pluginManagement") {
        return content.to_string();
    }

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

    format!("{}{}", plugin_mgmt, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context() -> BuildModificationContext {
        BuildModificationContext {
            project_name: "Test App".to_string(),
            package_name: "com.example.test".to_string(),
            appid: "__UNI__TEST".to_string(),
            dcloud_appkey: "test-app-key".to_string(),
            app_name: "Test App".to_string(),
            version_code: 178,
            version_name: "1.7.8".to_string(),
            compile_sdk: 35,
            target_sdk: 34,
            min_sdk: 21,
            keystore_path: "/tmp/test-release.keystore".to_string(),
            key_alias: "release".to_string(),
            key_password: "keypass".to_string(),
            store_password: "storepass".to_string(),
            android_allow_backup: "false".to_string(),
            extra_repositories: vec!["maven { url 'https://jitpack.io' }".to_string()],
            extra_dependencies: vec!["implementation 'androidx.core:core:1.12.0'".to_string()],
            plugin_includes: vec![
                "include ':demo-plugin'\nproject(':demo-plugin').projectDir = file('uts-modules/demo-plugin')"
                    .to_string(),
            ],
            plugin_project_dependencies: vec![
                "implementation project(':demo-plugin')".to_string(),
            ],
            module_permissions: vec![
                r#"<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION" />"#
                    .to_string(),
            ],
            module_application_entries: vec![
                r#"<meta-data android:name="GETUI_APPID" android:value="${GETUI_APPID}" />"#.to_string(),
            ],
            module_pandora_entry_intent_filters: vec![
                r#"<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:host="io.dcloud.unipush" android:path="/" android:scheme="unipush" />
</intent-filter>"#
                    .to_string(),
            ],
            manifest_placeholders:
                "\n        manifestPlaceholders = [\n            \"GETUI_APPID\": \"demo\"\n        ]"
                    .to_string(),
            dependency_excludes: String::new(),
        }
    }

    fn write_official_like_project(workspace: &Path) {
        std::fs::create_dir_all(workspace.join("simpleDemo/src/main/assets/data")).unwrap();
        std::fs::create_dir_all(workspace.join("simpleDemo/src/main/res/values")).unwrap();
        std::fs::write(workspace.join("settings.gradle"), "include ':simpleDemo'\n").unwrap();
        std::fs::write(
            workspace.join("build.gradle"),
            r#"buildscript {
    repositories {
        google()
    }
}

allprojects {
    repositories {
        google()
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            workspace.join("simpleDemo/build.gradle"),
            r#"apply plugin: 'com.android.application'

android {
    compileSdkVersion 35
    buildToolsVersion '35.0.0'
    namespace 'com.android.simple'
    defaultConfig {
        applicationId "com.android.simple"
        minSdkVersion 21
        targetSdkVersion 33
        versionCode 1
        versionName "1.0"
        multiDexEnabled true
    }
    signingConfigs {
        config {
            keyAlias 'key0'
            keyPassword '123456'
            storeFile file('test.jks')
            storePassword '123456'
        }
    }
    buildTypes {
        debug {
            signingConfig signingConfigs.config
        }
        release {
            signingConfig signingConfigs.config
        }
    }
}

dependencies {
    implementation fileTree(dir: 'libs', include: ['*.aar', '*.jar'], exclude: [])
}
"#,
        )
        .unwrap();
        std::fs::write(
            workspace.join("simpleDemo/src/main/AndroidManifest.xml"),
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application
        android:allowBackup="true"
        android:label="@string/app_name">
        <activity
            android:name="io.dcloud.PandoraEntryActivity"
            android:exported="true">
        </activity>
        <meta-data
            android:name="dcloud_appkey"
            android:value="placeholder" />
    </application>
</manifest>
"#,
        )
        .unwrap();
        std::fs::write(
            workspace.join("simpleDemo/src/main/res/values/strings.xml"),
            r#"<resources>
    <string name="app_name">UniApp</string>
</resources>
"#,
        )
        .unwrap();
        std::fs::write(
            workspace.join("simpleDemo/src/main/assets/data/dcloud_control.xml"),
            r#"<hbuilder>
<apps>
    <app appid="__UNI__A" appver=""/>
</apps>
</hbuilder>
"#,
        )
        .unwrap();
    }

    #[test]
    fn official_project_patch_is_idempotent_without_template_markers() {
        let workspace =
            std::env::temp_dir().join(format!("unipack-android-mod-{}", uuid::Uuid::new_v4()));
        write_official_like_project(&workspace);
        let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
        let ctx = test_context();

        modifier.apply_all_modifications(&ctx).unwrap();
        modifier.apply_all_modifications(&ctx).unwrap();

        let build_gradle =
            std::fs::read_to_string(workspace.join(MODULE_NAME).join("build.gradle")).unwrap();
        assert!(build_gradle.contains("namespace 'com.example.test'"));
        assert!(build_gradle.contains("compileSdkVersion 35"));
        assert!(build_gradle.contains("applicationId \"com.example.test\""));
        assert!(build_gradle.contains("minSdkVersion 21"));
        assert!(build_gradle.contains("targetSdkVersion 34"));
        assert!(build_gradle.contains("versionCode 178"));
        assert!(build_gradle.contains("versionName \"1.7.8\""));
        assert!(build_gradle.contains("signingConfig signingConfigs.release"));
        assert!(build_gradle.contains("storeFile file('/tmp/test-release.keystore')"));
        assert_eq!(
            build_gradle
                .matches("implementation project(':demo-plugin')")
                .count(),
            1
        );
        assert_eq!(build_gradle.matches("manifestPlaceholders").count(), 1);

        let manifest = std::fs::read_to_string(
            workspace
                .join(MODULE_NAME)
                .join("src/main/AndroidManifest.xml"),
        )
        .unwrap();
        assert!(manifest.contains(r#"android:allowBackup="false""#));
        assert!(manifest.contains(r#"android:value="test-app-key""#));
        assert_eq!(
            manifest
                .matches("android.permission.ACCESS_BACKGROUND_LOCATION")
                .count(),
            1
        );
        assert_eq!(manifest.matches(r#"android:name="GETUI_APPID""#).count(), 1);
        assert_eq!(manifest.matches(r#"android:scheme="unipush""#).count(), 1);

        let settings = std::fs::read_to_string(workspace.join("settings.gradle")).unwrap();
        assert!(settings.contains("rootProject.name = 'Test App'"));
        assert_eq!(settings.matches("include ':demo-plugin'").count(), 1);

        let dcloud = std::fs::read_to_string(
            workspace
                .join(MODULE_NAME)
                .join("src/main/assets/data/dcloud_control.xml"),
        )
        .unwrap();
        assert!(dcloud.contains(r#"appid="__UNI__TEST""#));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn local_downloaded_official_project_can_be_patched_when_present() {
        let sdk_root =
            PathBuf::from("/Users/huangxiangrui/Downloads/5.07/Android-SDK@5.07.82603_20260414");
        let source = sdk_root.join("HBuilder-Integrate-AS");
        if !source.exists() {
            return;
        }

        let workspace =
            std::env::temp_dir().join(format!("unipack-android-real-mod-{}", uuid::Uuid::new_v4()));
        crate::utils::fs::copy_recursive(&source, &workspace).unwrap();
        let modifier = AndroidProjectModifier::new(workspace.clone()).unwrap();
        let ctx = test_context();

        modifier.apply_all_modifications(&ctx).unwrap();
        modifier.apply_all_modifications(&ctx).unwrap();

        let build_gradle =
            std::fs::read_to_string(workspace.join(MODULE_NAME).join("build.gradle")).unwrap();
        assert!(build_gradle.contains("namespace 'com.example.test'"));
        assert!(build_gradle.contains("applicationId \"com.example.test\""));
        assert_eq!(
            build_gradle
                .matches("implementation project(':demo-plugin')")
                .count(),
            1
        );

        let manifest = std::fs::read_to_string(
            workspace
                .join(MODULE_NAME)
                .join("src/main/AndroidManifest.xml"),
        )
        .unwrap();
        assert!(manifest.contains(r#"android:allowBackup="false""#));
        assert!(manifest.contains(r#"android:value="test-app-key""#));
        assert_eq!(manifest.matches(r#"android:name="GETUI_APPID""#).count(), 1);

        let root_gradle = std::fs::read_to_string(workspace.join("build.gradle")).unwrap();
        assert_eq!(root_gradle.matches("https://jitpack.io").count(), 1);

        let _ = std::fs::remove_dir_all(workspace);
    }
}
