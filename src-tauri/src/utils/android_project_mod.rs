//! Android 工程修改器（V2 模板核心组件）
//!
//! 本模块实现了基于 **方式二（导入工程）** 的 Android 项目动态修改能力。
//! 通过解析 `.template-config.json` 配置文件，对 HBuilder-Integrate-AS 工程模板进行
//! 运行时修改，包括：
//!
//! - Gradle 构建配置修改（applicationId、dependencies、signingConfigs 等）
//! - AndroidManifest.xml 注入（权限、Activity、Service、Meta-data 等）
//! - 资源文件更新（strings.xml、dcloud_control.xml、dcloud_properties.xml）
//! - UTS 插件集成（settings.gradle include 追加）
//!
//! ## 使用示例
//! ```ignore
//! let modifier = AndroidProjectModifier::new(template_dir, workspace_dir)?;
//! modifier.apply_all_modifications(&ctx)?;
//! ```
//!
//! ## 配置驱动
//! 所有修改规则声明在模板目录的 `.template-config.json` 文件中，
//! 支持的修改类型包括：
//! - `ReplaceField`: 替换指定字段值
//! - `InjectPermissions`: 注入权限声明
//! - `AppendDependencies`: 追加依赖项
//! - `ConfigureSigning`: 配置签名信息
//! - 等等

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MODULE_NAME: &str = "simpleDemo";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateConfig {
    #[serde(default)]
    pub template_version: String,
    #[serde(default)]
    pub base_project: String,
    #[serde(default)]
    pub module_name: String,
    pub modifications: ModificationsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificationsConfig {
    #[serde(rename = "settings.gradle")]
    pub settings_gradle: FileModificationConfig,
    #[serde(rename = "simpleDemo/build.gradle")]
    pub app_build_gradle: FileModificationConfig,
    #[serde(rename = "simpleDemo/src/main/AndroidManifest.xml")]
    pub android_manifest: FileModificationConfig,
    #[serde(rename = "simpleDemo/src/main/res/values/strings.xml")]
    pub strings_xml: FileModificationConfig,
    #[serde(rename = "simpleDemo/src/main/assets/data/dcloud_control.xml")]
    pub dcloud_control: FileModificationConfig,
    #[serde(rename = "simpleDemo/src/main/assets/data/dcloud_properties.xml")]
    pub dcloud_properties: FileModificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModificationConfig {
    pub actions: Vec<ModificationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ModificationAction {
    ReplaceField {
        field: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        replacement: Option<String>,
        marker: Option<String>,
    },
    ConfigureSigning {
        config: SigningConfig,
    },
    InjectBlock {
        block: String,
        marker: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "contentPrefix")]
        content: Option<String>,
    },
    AppendRepositories {
        marker: String,
    },
    AppendDependencies {
        marker: String,
    },
    AppendPluginDeps {
        marker: String,
    },
    AppendContent {
        position: String,
        marker: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "contentPrefix")]
        content_prefix: Option<String>,
    },
    InjectPermissions {
        position: String,
    },
    InjectApplicationEntries {
        position: String,
    },
    ReplaceMetaData {
        name: String,
        value: String,
    },
    ReplaceAttribute {
        xpath: String,
        attribute: String,
        value: String,
    },
    ReplaceStringResource {
        name: String,
        value: String,
    },
    SetXmlAttribute {
        xpath: String,
        attribute: String,
        value: String,
    },
    GenerateFromModules {
        modules_source: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    pub release: SigningEntry,
    pub debug: SigningEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigningEntry {
    pub store_file_pattern: String,
    pub key_alias_pattern: String,
    pub key_password_marker: String,
    pub store_password_marker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    config: TemplateConfig,
    workspace_dir: PathBuf,
}

impl AndroidProjectModifier {
    pub fn new(template_dir: PathBuf, workspace_dir: PathBuf) -> Result<Self, String> {
        let config_path = template_dir.join(".template-config.json");
        if !config_path.exists() {
            return Err(format!("模板配置文件不存在: {}", config_path.display()));
        }

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        let config: TemplateConfig = serde_json::from_str(&config_content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        Ok(Self {
            config,
            workspace_dir,
        })
    }

    pub fn apply_all_modifications(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        self.modify_settings_gradle(ctx)?;
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

        for action in &self.config.modifications.settings_gradle.actions {
            match action {
                ModificationAction::ReplaceField { field, pattern, .. } => {
                    if field == "rootProject.name" {
                        let re = Regex::new(r"rootProject\.name\s*=\s*'[^']*'").unwrap();
                        content = re
                            .replace(
                                &content,
                                format!("rootProject.name = '{}'", ctx.project_name),
                            )
                            .to_string();
                    } else if let Some(pat) = pattern {
                        let re = Regex::new(pat).map_err(|e| format!("正则错误: {}", e))?;
                        if let Some(repl) = self.get_replacement(field, ctx) {
                            content = re.replace(&content, repl.as_str()).to_string();
                        }
                    }
                }
                ModificationAction::AppendContent {
                    marker,
                    content_prefix,
                    ..
                } => {
                    if !ctx.plugin_includes.is_empty() {
                        let includes = ctx
                            .plugin_includes
                            .iter()
                            .map(|inc| {
                                format!("{}{}", content_prefix.as_deref().unwrap_or(""), inc)
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        content = content.replace(marker, &includes);
                    }
                }
                _ => {}
            }
        }

        self.write_file(&path, &content)
    }

    fn modify_app_build_gradle(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self.workspace_dir.join(MODULE_NAME).join("build.gradle");
        let original_content = self.read_file(&path)?;
        let mut content = original_content.clone();

        for action in &self.config.modifications.app_build_gradle.actions {
            match action {
                ModificationAction::ReplaceField {
                    field,
                    pattern,
                    replacement,
                    ..
                } => {
                    let replacement = replacement
                        .as_deref()
                        .map(|template| self.render_template(template, ctx))
                        .or_else(|| self.get_replacement(field, ctx));
                    if let Some(replacement) = replacement {
                        if let Some(pat) = pattern {
                            let re = Regex::new(pat)
                                .map_err(|e| format!("正则错误[{}]: {}", field, e))?;
                            content = re.replace(&content, replacement.as_str()).to_string();
                            if content.len() < 100 {
                                return Err(format!(
                                    "build.gradle 替换后内容过短(字段:{}), 可能被清空:\n{}",
                                    field, content
                                ));
                            }
                        }
                    }
                }
                ModificationAction::ConfigureSigning { config } => {
                    content = self.apply_signing_config(&content, &config.release, ctx, false)?;
                    content = self.apply_signing_config(&content, &config.debug, ctx, true)?;
                }
                ModificationAction::InjectBlock {
                    marker,
                    content: block_content,
                    ..
                } => {
                    if let Some(block_content) = block_content {
                        let rendered = self.render_template(block_content, ctx);
                        content = content.replace(marker, &rendered);
                    } else {
                        content = content.replace(marker, "");
                    }
                }
                ModificationAction::AppendRepositories { marker } => {
                    if !ctx.extra_repositories.is_empty() {
                        let repos = ctx
                            .extra_repositories
                            .iter()
                            .map(|r| format!("        {}", r))
                            .collect::<Vec<_>>()
                            .join("\n");
                        content = content.replace(marker, &repos);
                    } else {
                        content = content.replace(marker, "");
                    }
                }
                ModificationAction::AppendDependencies { marker } => {
                    if !ctx.extra_dependencies.is_empty() {
                        let deps = ctx
                            .extra_dependencies
                            .iter()
                            .map(|d| format!("    {}", d))
                            .collect::<Vec<_>>()
                            .join("\n");
                        content = content.replace(marker, &deps);
                    } else {
                        content = content.replace(marker, "");
                    }
                }
                ModificationAction::AppendPluginDeps { marker } => {
                    if !ctx.plugin_project_dependencies.is_empty() {
                        let deps = ctx
                            .plugin_project_dependencies
                            .iter()
                            .map(|d| format!("    {}", d))
                            .collect::<Vec<_>>()
                            .join("\n");
                        content = content.replace(marker, &deps);
                    } else {
                        content = content.replace(marker, "");
                    }
                }
                _ => {}
            }

            if !ctx.dependency_excludes.is_empty() {
                content =
                    content.replace("// UNIPACK_DEPENDENCY_EXCLUDES", &ctx.dependency_excludes);
            }
        }

        // 验证 plugins 块是否完整
        if !content.contains("plugins {") || !content.contains("id 'com.android.application'") {
            return Err(format!(
                "build.gradle 的 plugins 块被破坏！\n前50字符:\n{}\n...\n原始前50字符:\n{}",
                &content[..content.len().min(50)],
                &original_content[..original_content.len().min(50)]
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
            for (col_idx, ch) in line.chars().enumerate() {
                match ch {
                    '/' if !in_string && !in_char && !in_block_comment => {
                        if col_idx + 1 < line.len() {
                            let next_ch = line.chars().nth(col_idx + 1).unwrap_or('\0');
                            match next_ch {
                                '/' => break,
                                '*' => {
                                    in_block_comment = true;
                                }
                                _ => {}
                            }
                        }
                    }
                    '*' if !in_string && !in_char && in_block_comment => {
                        if col_idx + 1 < line.len() && line.chars().nth(col_idx + 1) == Some('/') {
                            in_block_comment = false;
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
            }
        }

        if brace_count != 0 {
            Err(format!(
                "Gradle 语法错误: {} 花括号不匹配，缺少 {} 个 '}}' (或多余 {{)",
                file_path.display(),
                brace_count.abs()
            ))
        } else {
            Ok(())
        }
    }

    fn apply_signing_config(
        &self,
        content: &str,
        entry: &SigningEntry,
        ctx: &BuildModificationContext,
        _is_debug: bool,
    ) -> Result<String, String> {
        let mut result = content.to_string();

        let store_file_re = Regex::new(&entry.store_file_pattern).unwrap();
        let key_alias_re = Regex::new(&entry.key_alias_pattern).unwrap();

        result = store_file_re
            .replace(
                &result,
                format!(
                    "storeFile file('{}')",
                    escape_gradle_single_quoted(&ctx.keystore_path)
                ),
            )
            .to_string();

        result = key_alias_re
            .replace(
                &result,
                format!("keyAlias '{}'", escape_gradle_single_quoted(&ctx.key_alias)),
            )
            .to_string();

        let key_pw_re = Regex::new(&format!(
            r"keyPassword\s+''\s+//\s*{}",
            regex::escape(&entry.key_password_marker)
        ))
        .unwrap();
        let store_pw_re = Regex::new(&format!(
            r"storePassword\s+''\s+//\s*{}",
            regex::escape(&entry.store_password_marker)
        ))
        .unwrap();

        result = key_pw_re
            .replace(
                &result,
                format!(
                    "keyPassword '{}'",
                    escape_gradle_single_quoted(&ctx.key_password)
                ),
            )
            .to_string();
        result = store_pw_re
            .replace(
                &result,
                format!(
                    "storePassword '{}'",
                    escape_gradle_single_quoted(&ctx.store_password)
                ),
            )
            .to_string();

        Ok(result)
    }

    fn modify_android_manifest(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self
            .workspace_dir
            .join(MODULE_NAME)
            .join("src/main/AndroidManifest.xml");
        let mut content = self.read_file(&path)?;

        for action in &self.config.modifications.android_manifest.actions {
            match action {
                ModificationAction::InjectPermissions { .. } => {
                    if !ctx.module_permissions.is_empty() {
                        let permissions = ctx
                            .module_permissions
                            .iter()
                            .map(|p| format!("    {}", p))
                            .collect::<Vec<_>>()
                            .join("\n");
                        content = content.replace(
                            "<!-- UNIPACK_INJECT_PERMISSIONS: Module-specific permissions will be injected here -->",
                            &format!(
                                "<!-- UNIPACK_INJECT_PERMISSIONS -->\n{}",
                                permissions
                            ),
                        );
                    }
                }
                ModificationAction::InjectApplicationEntries { .. } => {
                    if !ctx.module_application_entries.is_empty() {
                        let entries = ctx.module_application_entries.join("\n");
                        content = content.replace(
                            "<!-- UNIPACK_INJECT_APPLICATION_ENTRIES: Module activities/services/receivers/providers will be injected here -->",
                            &format!(
                                "<!-- UNIPACK_INJECT_APPLICATION_ENTRIES -->\n{}",
                                entries
                            ),
                        );
                    }
                }
                ModificationAction::ReplaceMetaData { name, .. } => {
                    if name == "dcloud_appkey" {
                        content = content.replace(
                            "android:value=\"\" />  <!-- UNIPACK_REPLACE:dcloud_appkey -->",
                            &format!("android:value=\"{}\" />", ctx.dcloud_appkey),
                        );
                    }
                }
                ModificationAction::ReplaceAttribute { attribute, .. } => {
                    if attribute == "allowBackup" {
                        content = content.replace(
                            "android:allowBackup=\"true\"  <!-- UNIPACK_REPLACE:allowBackup -->",
                            &format!(
                                "android:allowBackup=\"{}\"  <!-- UNIPACK_REPLACE:allowBackup -->",
                                ctx.android_allow_backup
                            ),
                        );
                    }
                }
                _ => {}
            }

            if !ctx.module_pandora_entry_intent_filters.is_empty() {
                let filters = ctx.module_pandora_entry_intent_filters.join("\n");
                content = content.replace(
                    "<!-- UNIPACK_INJECT_PANDORA_ENTRY_INTENT_FILTERS: Additional intent filters will be injected here -->",
                    &format!(
                        "<!-- UNIPACK_INJECT_PANDORA_ENTRY_INTENT_FILTERS -->\n{}",
                        filters
                    ),
                );
            }
        }

        self.write_file(&path, &content)
    }

    fn modify_strings_xml(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self
            .workspace_dir
            .join(MODULE_NAME)
            .join("src/main/res/values/strings.xml");
        let content = self.read_file(&path)?;

        let modified = content.replace(
            "<string name=\"app_name\">UniApp</string>",
            &format!("<string name=\"app_name\">{}</string>", ctx.app_name),
        );

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
                .unwrap_or(content);

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

        let properties_content = r#"<properties>
</properties>"#;

        self.write_file(&path, properties_content)
    }

    fn get_replacement(&self, field: &str, ctx: &BuildModificationContext) -> Option<String> {
        match field {
            "namespace" | "package_name" => Some(format!(
                "namespace '{}'",
                escape_gradle_single_quoted(&ctx.package_name)
            )),
            "applicationId" => Some(format!(
                "applicationId \"{}\"",
                escape_gradle_double_quoted(&ctx.package_name)
            )),
            "compileSdk" | "compile_sdk" => Some(format!("compileSdk {}", ctx.compile_sdk)),
            "minSdk" | "min_sdk" => Some(format!("minSdk {}", ctx.min_sdk)),
            "targetSdk" | "target_sdk" => Some(format!("targetSdk {}", ctx.target_sdk)),
            "versionCode" | "version_code" => Some(format!("versionCode {}", ctx.version_code)),
            "versionName" | "version_name" => Some(format!(
                "versionName \"{}\"",
                escape_gradle_double_quoted(&ctx.version_name)
            )),
            "project_name" => Some(format!(
                "'{}'",
                escape_gradle_single_quoted(&ctx.project_name)
            )),
            _ => None,
        }
    }

    fn render_template(&self, template: &str, ctx: &BuildModificationContext) -> String {
        template
            .replace("{{package_name}}", &ctx.package_name)
            .replace("{{compile_sdk}}", &ctx.compile_sdk.to_string())
            .replace("{{target_sdk}}", &ctx.target_sdk.to_string())
            .replace("{{min_sdk}}", &ctx.min_sdk.to_string())
            .replace("{{version_code}}", &ctx.version_code.to_string())
            .replace("{{version_name}}", &ctx.version_name)
            .replace("{{keystore_path}}", &ctx.keystore_path)
            .replace("{{key_alias}}", &ctx.key_alias)
            .replace("{{key_password}}", &ctx.key_password)
            .replace("{{store_password}}", &ctx.store_password)
            .replace("{{dcloud_appkey}}", &ctx.dcloud_appkey)
            .replace("{{appid}}", &ctx.appid)
            .replace("{{app_name}}", &ctx.app_name)
            .replace("{{android_allow_backup}}", &ctx.android_allow_backup)
            .replace(
                "{{manifest_placeholders_block}}",
                &ctx.manifest_placeholders,
            )
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

fn escape_gradle_single_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn escape_gradle_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
            android_allow_backup: "true".to_string(),
            extra_repositories: vec![],
            extra_dependencies: vec![],
            plugin_includes: vec![],
            plugin_project_dependencies: vec![],
            module_permissions: vec![],
            module_application_entries: vec![],
            module_pandora_entry_intent_filters: vec![],
            manifest_placeholders: String::new(),
            dependency_excludes: String::new(),
        }
    }

    #[test]
    fn v2_app_build_gradle_renders_valid_gradle_keywords() {
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("bundled/android-template-v2");
        assert!(template_dir.exists());

        let workspace =
            std::env::temp_dir().join(format!("unipack-android-mod-{}", uuid::Uuid::new_v4()));
        crate::utils::fs::copy_recursive(&template_dir, &workspace).unwrap();
        let modifier = AndroidProjectModifier::new(template_dir, workspace.clone()).unwrap();
        modifier.apply_all_modifications(&test_context()).unwrap();

        let build_gradle =
            std::fs::read_to_string(workspace.join(MODULE_NAME).join("build.gradle")).unwrap();

        assert!(build_gradle.contains("android {"));
        assert!(build_gradle.contains("namespace 'com.example.test'"));
        assert!(build_gradle.contains("compileSdk 35"));
        assert!(build_gradle.contains("applicationId \"com.example.test\""));
        assert!(build_gradle.contains("minSdk 21"));
        assert!(build_gradle.contains("targetSdk 34"));
        assert!(build_gradle.contains("versionCode 178"));
        assert!(build_gradle.contains("versionName \"1.7.8\""));
        assert!(build_gradle.contains("storeFile file('/tmp/test-release.keystore')"));
        assert!(build_gradle.contains("keyAlias 'release'"));
        assert!(build_gradle.contains("keyPassword 'keypass'"));
        assert!(build_gradle.contains("storePassword 'storepass'"));

        let _ = std::fs::remove_dir_all(workspace);
    }
}
