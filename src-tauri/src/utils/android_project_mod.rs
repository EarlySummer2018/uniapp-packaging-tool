//! Android 工程补丁器。
//!
//! 构建流程会先从用户配置的 DCloud Android 离线 SDK 复制
//! `HBuilder-Integrate-AS` 到临时工作区，本模块只修改这份工作区副本。

use regex::Regex;
use std::path::{Path, PathBuf};

pub const MODULE_NAME: &str = "simpleDemo";

/// 按模块分组的 Manifest 补丁组，用于逐模块插入。
///
/// 每个模块（如 push、login、share）的权限、application 子元素和 intent-filter
/// 被归为一组，在 modify_android_manifest 中按顺序逐组处理，
/// 确保一个模块的所有条目完全插入成功后再进入下一个。
#[derive(Debug, Clone)]
pub struct ManifestPatchGroup {
    pub module_name: String,
    pub permissions: Vec<String>,
    pub application_entries: Vec<String>,
    pub intent_filters: Vec<String>,
}

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
    /// 旧字段（扁平化），保留向后兼容
    pub module_permissions: Vec<String>,
    pub module_application_entries: Vec<String>,
    pub module_pandora_entry_intent_filters: Vec<String>,
    /// 新字段：按模块分组的补丁，用于逐模块安全插入
    pub module_patch_groups: Vec<ManifestPatchGroup>,
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

        // 将额外仓库注入到 dependencyResolutionManagement.repositories
        // （PREFER_SETTINGS 模式下，build.gradle 的 allprojects 声明会被忽略）
        if !ctx.extra_repositories.is_empty() {
            content = ensure_repositories_in_drm(&content, &ctx.extra_repositories);
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
        content = ensure_signing_configs_block(&content, &render_signing_configs(ctx))?;
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

    /// 修改 AndroidManifest.xml：逐条安全插入 + 每步结构校验 + 自动修复重试。
    ///
    /// 将原来的批量 for 循环改为逐条插入模式，每条 entry 插入后立即调用
    /// validate_manifest_structure 校验结构合法性。
    /// 校验失败时先尝试 fix_manifest_xml_structure 自动修复并重新校验，
    /// 仅在自动修复也无法解决时才跳过该条目。
    fn modify_android_manifest(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self
            .workspace_dir
            .join(MODULE_NAME)
            .join("src/main/AndroidManifest.xml");
        let content = self.read_file(&path)?;

        let mut editor = XmlManifestEditor::from_str(&content);

        // === Phase 0: 基础设置（低风险，一次性完成）===
        editor.set_application_attr("android:allowBackup", &ctx.android_allow_backup)?;

        let appkey_entry = format!(
            r#"<meta-data android:name="dcloud_appkey" android:value="{}" />"#,
            escape_xml_attr(&ctx.dcloud_appkey)
        );
        editor.add_application_entry(
            &appkey_entry,
            &EntryIdentity::MetaData("dcloud_appkey".to_string()),
        )?;
        // 更新已存在的 meta-data 值（模板可能已有 placeholder）
        let current = editor.as_str().to_string();
        if let Ok(updated) = set_meta_data_value(&current, "dcloud_appkey", &ctx.dcloud_appkey) {
            editor.replace_content(updated);
        }

        // === Phase 1-N: 逐模块插入（核心改动：一个模块完全插入成功后再处理下一个）===
        // 如果 module_patch_groups 为空（旧代码路径或测试），从扁平字段合成一个默认组
        let groups: Vec<ManifestPatchGroup> = if ctx.module_patch_groups.is_empty() {
            vec![ManifestPatchGroup {
                module_name: "legacy".to_string(),
                permissions: ctx.module_permissions.clone(),
                application_entries: ctx.module_application_entries.clone(),
                intent_filters: ctx.module_pandora_entry_intent_filters.clone(),
            }]
        } else {
            ctx.module_patch_groups.clone()
        };

        for (_group_idx, group) in groups.iter().enumerate() {
            eprintln!(
                "[INFO] 正在插入模块 {} 的 Manifest 条目...",
                group.module_name
            );

            // 1) 添加该模块的权限
            if !group.permissions.is_empty() {
                editor.add_permissions(&group.permissions)?;
            }

            // 2) 逐条添加该模块的 application_entries
            for (entry_idx, entry) in group.application_entries.iter().enumerate() {
                let identity = entry_identity(entry);
                match editor.add_application_entry(entry, &identity) {
                    Ok(true) => {}  // 插入成功
                    Ok(false) => {} // 已存在，跳过
                    Err(e) => {
                        let description = format_entry_description(entry, &identity);
                        // 单条失败 → 校验 → 尝试修复 → 重验 → 仍失败才跳过
                        if let Err(_v_err) = editor.validate_structure() {
                            // 尝试用旧的正则修复器做最后努力
                            if let Ok(fixed_content) = fix_manifest_xml_structure(editor.as_str()) {
                                let fixed_editor = XmlManifestEditor::from_str(&fixed_content);
                                if fixed_editor.validate_structure().is_ok() {
                                    // 编辑器替换为修复后的版本，继续处理后续条目
                                    // （注意：editor 被 move，需要重新绑定）
                                    drop(editor);
                                    editor = fixed_editor;
                                    continue;
                                }
                            }
                            eprintln!(
                                "[WARN] 模块 '{}' 条目 #{} ({}) 插入失败: {}",
                                group.module_name,
                                entry_idx + 1,
                                description,
                                e
                            );
                        }
                    }
                }
            }

            // 3) 逐条添加该模块的 intent_filters
            for (filter_idx, filter) in group.intent_filters.iter().enumerate() {
                let identity = child_identity(filter);

                // 先尝试 PandoraEntryActivity，再 fallback 到 PandoraEntry
                let result = editor
                    .add_activity_child("io.dcloud.PandoraEntryActivity", filter, &identity)
                    .or_else(|_| {
                        editor.add_activity_child("io.dcloud.PandoraEntry", filter, &identity)
                    });

                match result {
                    Ok(true) => {}
                    Ok(false) => {}
                    Err(e) => {
                        // 始终输出警告（不再静默吞掉错误）
                        eprintln!(
                            "[WARN] 模块 '{}' Intent-filter #{} 插入失败: {}",
                            group.module_name,
                            filter_idx + 1,
                            e
                        );
                        // 尝试结构校验与修复
                        if let Err(v_err) = editor.validate_structure() {
                            eprintln!("[WARN] 结构校验也失败: {}", v_err);
                            if let Ok(fixed_content) = fix_manifest_xml_structure(editor.as_str()) {
                                let fixed_editor = XmlManifestEditor::from_str(&fixed_content);
                                if fixed_editor.validate_structure().is_ok() {
                                    drop(editor);
                                    editor = fixed_editor;
                                    // 重试当前 intent-filter
                                    let retry_identity = child_identity(filter);
                                    let retry_result = editor
                                        .add_activity_child(
                                            "io.dcloud.PandoraEntryActivity",
                                            filter,
                                            &retry_identity,
                                        )
                                        .or_else(|_| {
                                            editor.add_activity_child(
                                                "io.dcloud.PandoraEntry",
                                                filter,
                                                &retry_identity,
                                            )
                                        });
                                    if retry_result.is_ok() {
                                        continue; // 重试成功，处理下一条
                                    }
                                    eprintln!(
                                        "[WARN] 模块 '{}' Intent-filter #{} 修复后重试仍然失败",
                                        group.module_name,
                                        filter_idx + 1
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // 4) 本模块全部插入完毕 → 做一次完整性校验
            if let Err(e) = editor.validate_structure() {
                eprintln!(
                    "[WARN] 模块 '{}' 插入完成后结构校验未通过: {}，尝试自动修复",
                    group.module_name, e
                );
                let fixed_content = fix_manifest_xml_structure(editor.as_str())
                    .map_err(|fe| format!("自动修复失败: {} (原始错误: {})", fe, e))?;
                let fixed_editor = XmlManifestEditor::from_str(&fixed_content);
                fixed_editor.validate_structure()?; // 修复后再验，仍失败则中断
                drop(editor);
                editor = fixed_editor;
            }

            eprintln!("[INFO] 模块 {} Manifest 条目插入完成", group.module_name);
        }

        // === Post-processing: 清理模板遗留的无效内容 ===
        {
            let current = editor.as_str().to_string();
            let mut cleaned = current;

            // Fix 1: 移除空 scheme 的 <data /> 占位符
            {
                let re = Regex::new(r#"<data\s[^>]*android:scheme="\s*"[^>]*/\s*>"#).unwrap();
                if re.is_match(&cleaned) {
                    let n = re.find_iter(&cleaned).count();
                    eprintln!("[INFO] 已清理 {} 个空 scheme <data> 占位符", n);
                    cleaned = re.replace_all(&cleaned, "").to_string();
                }
            }

            // Fix 2: 去重 <application> 标签中的重复属性（如 allowBackup 出现两次）
            {
                let app_re = Regex::new(r#"(?s)(<application\b[^>]*?)(>)"#).unwrap();
                if let Some(caps) = app_re.captures(&cleaned) {
                    let tag_start = caps.get(1).unwrap().start();
                    let tag_end = caps.get(2).unwrap().end();
                    let tag_slice = &cleaned[tag_start..tag_end];

                    let pat = r#"(\w[\w:-]*)="[^"]*""#;
                    let ar = Regex::new(pat).unwrap();
                    let mut last_pos: std::collections::BTreeMap<String, usize> =
                        std::collections::BTreeMap::new();
                    for m in ar.find_iter(tag_slice) {
                        let c = Regex::new(pat).unwrap();
                        if let Some(cc) = c.captures(m.as_str()) {
                            last_pos.insert(cc[1].to_string(), m.start());
                        }
                    }

                    let total = ar.find_iter(tag_slice).count();
                    if total > last_pos.len() {
                        let mut removals: Vec<usize> = Vec::new();
                        for m in ar.find_iter(tag_slice) {
                            let c = Regex::new(pat).unwrap();
                            if let Some(cc) = c.captures(m.as_str()) {
                                if let Some(&lp) = last_pos.get(&cc[1].to_string()) {
                                    if m.start() != lp {
                                        removals.push(m.start() + tag_start);
                                    }
                                }
                            }
                        }
                        removals.sort_by(|a, b| b.cmp(a));
                        for pos in removals {
                            if let Some(m) = ar.find_at(&cleaned, pos) {
                                cleaned.replace_range(m.start()..m.end(), "");
                            }
                        }
                        eprintln!(
                            "[INFO] 已去重 <application> 中 {} 个重复属性",
                            total - last_pos.len()
                        );
                    }
                }
            }

            // Fix 3: 去重 activity configChanges 中的重复值（如 screenSize 出现两次）
            {
                let cfg_re =
                    Regex::new(r#"(?s)(<activity\b[^>]*android:configChanges=")([^"]*)(")"#)
                        .unwrap();
                cleaned = cfg_re
                    .replace_all(&cleaned, |caps: &regex::Captures| {
                        let vals: Vec<&str> = caps[2]
                            .split('|')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect::<std::collections::BTreeSet<_>>()
                            .into_iter()
                            .collect();
                        format!("{}{}{}", &caps[1], vals.join("|"), &caps[3])
                    })
                    .to_string();
            }

            // Fix 4: 清理移除空 data 后变为无效的 intent-filter（仅含 VIEW action 无有效 data）
            {
                let re = Regex::new(
                    r#"(?s)<intent-filter>\s*(?:<category\s[^>]*/>\s*)*<action\s[^>]*android:name="android\.intent\.action\.VIEW"\s*/?\s*</\s*intent-filter\s*>"#
                ).unwrap();
                if re.is_match(&cleaned) {
                    let n = re.find_iter(&cleaned).count();
                    eprintln!("[INFO] 已清理 {} 个无效 intent-filter", n);
                    cleaned = re.replace_all(&cleaned, "").to_string();
                }
            }

            editor.replace_content(cleaned);
        }

        // === 最终结构校验 + 自动修复 ===
        if let Err(e) = editor.validate_structure() {
            eprintln!(
                "[WARN] AndroidManifest.xml 最终校验异常: {}, 尝试自动修复...",
                e
            );
            if let Ok(fixed) = fix_manifest_xml_structure(editor.as_str()) {
                editor.replace_content(fixed);
                // 修复后再验一次
                if let Err(e2) = editor.validate_structure() {
                    eprintln!("[WARN] 自动修复后仍有异常: {}", e2);
                } else {
                    eprintln!("[INFO] AndroidManifest.xml 结构修复成功");
                }
            }
        }

        // === 最终写入 ===
        self.write_file(&path, editor.as_str())
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

#[derive(Clone)]
pub enum EntryIdentity {
    MetaData(String),
    Component { tag: String, name: String },
    ProviderAuthority(String),
    Comment(String),
    Raw(String),
}

#[derive(Clone)]
pub enum ChildIdentity {
    IntentFilterDataScheme(String),
    IntentFilterAction(String),
    Raw(String),
}

/// 判断 AndroidManifest.xml 中 child 标签是否可以作为 parent 的合法子元素。
///
/// 完整的 AndroidManifest 层级规则，用于 validate_manifest_structure 检测非法嵌套。
fn can_be_child_of(child: &str, parent: &str) -> bool {
    match parent {
        "activity" => matches!(child, "intent-filter" | "meta-data" | "layout"),
        "intent-filter" => {
            matches!(child, "action" | "category" | "data" | "mime-type")
        }
        "application" => matches!(
            child,
            "activity"
                | "service"
                | "receiver"
                | "provider"
                | "meta-data"
                | "uses-library"
                | "property"
        ),
        "manifest" => matches!(
            child,
            "application"
                | "permission"
                | "uses-permission"
                | "uses-sdk"
                | "query"
                | "supports-screens"
        ),
        _ => true, // 其他父标签不做限制
    }
}

/// 自动修复 XML 内容中未正确闭合或嵌套的标签。
///
/// 在写入 AndroidManifest.xml 前调用，自动补全缺失的 </activity> 等结束标记，
/// 避免 Gradle ManifestMerger2 报出 SAXParseException。
///
/// 修复策略：
/// - 遇到不匹配的结束标签（如 `</application>` 但栈顶是 `<activity>`），
///   在该结束标签前插入缺失的 `</activity>`
/// - 扫描结束后栈中残留的未闭合标签，按逆序在末尾补充结束标记
fn fix_manifest_xml_structure(content: &str) -> Result<String, String> {
    let tag_re = Regex::new(r#"<(/?)([a-zA-Z][\w-]*)[^>]*?(/?)>"#)
        .map_err(|e| format!("XML 校验正则编译失败: {}", e))?;

    /// 栈中记录的未闭合开始标签
    struct OpenTag {
        name: String,
    }

    let mut stack: Vec<OpenTag> = Vec::new();
    let mut fixes: Vec<(usize, String)> = Vec::new(); // (插入位置, 要插入的内容)

    for mat in tag_re.find_iter(content) {
        let caps = tag_re
            .captures(mat.as_str())
            .expect("正则匹配成功，captures 必定存在");
        let is_close = &caps[1] == "/";
        let is_self_closing = caps.get(3).map_or(false, |m| m.as_str() == "/");
        let name = caps[2].to_string();

        if is_close {
            match stack.last() {
                Some(top) if top.name == name => {
                    stack.pop();
                }
                Some(top) => {
                    // 结束标签与栈顶不匹配 → 在当前位置前插入缺失的结束标记
                    let insertion = format!("    </{}>\n", top.name);
                    fixes.push((mat.start(), insertion));
                    stack.pop();
                    // 弹出后继续检查是否还有其他不匹配的
                    let mut retry = true;
                    while retry {
                        match stack.last() {
                            Some(t) if t.name == name => {
                                stack.pop();
                                retry = false;
                            }
                            Some(t) => {
                                let ins = format!("    </{}>\n", t.name);
                                fixes.push((mat.start(), ins));
                                stack.pop();
                            }
                            None => {
                                retry = false;
                            }
                        }
                    }
                }
                None => {
                    // 多余的结束标签，忽略（不删除，保持原样）
                }
            }
        } else if !is_self_closing {
            // 检查 Android Manifest 合法性：某些标签不能作为 <activity> 的子元素
            // 如果栈顶是 <activity> 而当前标签不合法，先闭合 <activity>
            while let Some(top) = stack.last() {
                if !can_be_child_of(&name, &top.name) {
                    let ins = format!("    </{}>\n", top.name);
                    fixes.push((mat.start(), ins));
                    stack.pop();
                } else {
                    break;
                }
            }
            stack.push(OpenTag { name });
        }
    }

    // 栈中剩余的未闭合标签：按逆序在内容末尾补充结束标记
    if !stack.is_empty() {
        let mut tail_fixes = String::new();
        for tag in stack.iter().rev() {
            tail_fixes.push_str(&format!("    </{}>\n", tag.name));
        }
        fixes.push((content.len(), tail_fixes));
    }

    if fixes.is_empty() {
        return Ok(content.to_string());
    }

    // 按位置从后往前应用修复（避免偏移量影响）
    let mut result = content.to_string();
    fixes.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));
    for (pos, insertion) in &fixes {
        result.insert_str(*pos, insertion);
    }

    Ok(result)
}

/// 在所有工程修改完成后、Gradle 构建前，
/// 对最终 AndroidManifest.xml 进行结构校验和自动修复。
pub fn validate_and_fix_final_manifest(workspace: &std::path::Path) -> Result<(), String> {
    let path = workspace
        .join(MODULE_NAME)
        .join("src/main/AndroidManifest.xml");

    let modifier = AndroidProjectModifier::new(workspace.to_path_buf())?;
    let content = modifier.read_file(&path)?;
    let fixed = fix_manifest_xml_structure(&content)?;
    if fixed != content {
        // 有修改才写入，避免不必要的 IO
        modifier.write_file(&path, &fixed)?;
    }
    Ok(())
}

fn set_or_insert_root_project_name(content: &str, project_name: &str) -> String {
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

/// 将仓库注入到 settings.gradle 的 dependencyResolutionManagement.repositories 块中。
/// PREFER_SETTINGS 模式下，Gradle 只从此处读取仓库声明，忽略 build.gradle 中的 allprojects。
fn ensure_repositories_in_drm(content: &str, repositories: &[String]) -> String {
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

fn ensure_signing_configs_block(content: &str, block_content: &str) -> Result<String, String> {
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

fn find_named_block_from(content: &str, start_at: usize, name: &str) -> Option<GradleBlock> {
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

    // 使用深度匹配定位正确的闭合标签，避免多同名标签时匹配错误位置
    let end_tag = format!("</{}>", tag);
    let open_pattern = format!("<{}", tag);

    let mut depth = 1; // 已进入目标开始标签
    let mut search_pos = start.end;

    while search_pos < content.len() {
        if let Some(rel) = content[search_pos..].find(&end_tag) {
            let close_pos = search_pos + rel;

            // 统计从 search_pos 到 close_pos 之间有多少个同名的开标签
            let segment = &content[search_pos..close_pos];
            let open_count = count_tag_open_occurrences(segment, &open_pattern);

            // 空片段（如 <activity></activity>）不应有嵌套标签
            let actual_open_count = if segment.is_empty() { 0 } else { open_count };

            if depth - actual_open_count == 1 {
                // 这个 </tag> 是正确的配对
                return Some(ManifestComponent {
                    start: start.start,
                    end: close_pos + end_tag.len(),
                    end_close_start: close_pos,
                });
            } else {
                // 这个 </tag> 属于嵌套的同名标签，继续往后找
                depth = depth - 1 + open_count;
                search_pos = close_pos + end_tag.len();
            }
        } else {
            break; // 没找到更多闭合标签
        }
    }
    None
}

/// 计算文本片段中某个标签的开标签出现次数（排除自闭合和闭标签）
fn count_tag_open_occurrences(fragment: &str, tag_pattern: &str) -> usize {
    let re = Regex::new(&format!(
        r"{}\b(?![/])(?:\s|>|[a-z_-])",
        regex::escape(tag_pattern)
    ))
    .unwrap_or_else(|_| Regex::new(r"$^").unwrap()); // 永不匹配的空正则作为 fallback
    re.find_iter(fragment).count()
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

/// 生成 Manifest 条目的可读描述，用于日志输出，方便定位问题条目
fn format_entry_description(_entry: &str, identity: &EntryIdentity) -> String {
    match identity {
        EntryIdentity::MetaData(name) => format!("meta-data({})", name),
        EntryIdentity::Component { tag, name } => format!("{}({})", tag, name),
        EntryIdentity::ProviderAuthority(auth) => {
            format!("provider(authorities={})", auth)
        }
        EntryIdentity::Comment(text) => {
            if text.len() > 50 {
                format!("comment({}...)", &text[..50])
            } else {
                format!("comment({})", text)
            }
        }
        EntryIdentity::Raw(raw) => {
            if raw.len() > 50 {
                format!("raw({}...)", &raw[..50])
            } else {
                format!("raw({})", raw)
            }
        }
    }
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

/// 判断一个 Manifest 条目是否为裸的 <data> 自闭合元素。
///
/// <data> 元素在 AndroidManifest 中必须作为 <intent-filter> 的子元素出现，
/// 不能直接放在 <application> 下。如果检测到这种条目，需要特殊路由处理。
fn is_bare_data_element(entry: &str) -> bool {
    let trimmed = entry.trim();
    // 匹配 <data ... /> 形式的自闭合标签（可能是条目的唯一内容）
    let data_re = Regex::new(r#"^\s*<data\b[^>]*/>\s*$"#).unwrap();
    if data_re.is_match(trimmed) {
        return true;
    }
    // 也匹配包含多个标签的片段中，顶层只有 <data ... /> 的情况
    let lines: Vec<&str> = trimmed.lines().collect();
    let non_empty: Vec<&str> = lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("<!--"))
        .collect();
    non_empty.len() == 1 && data_re.is_match(non_empty[0])
}

/// 将裸 <data> 元素路由到目标 Activity 的 intent-filter 内部。
///
/// 策略：
/// 1. 从 <data> 的 android:scheme 属性推断关联的模块（如 WX_APPID → wxapi.WXEntryActivity）
/// 2. 在 manifest 中查找该 Activity（支持后缀匹配）
/// 3. 如果 Activity 已有 <intent-filter>，将 <data> 插入其中
/// 4. 如果没有 intent-filter 但 Activity 存在，创建一个新的并插入 <data>
fn route_data_to_activity_intent_filter(content: &str, data_entry: &str) -> Result<String, String> {
    let scheme = android_attr_value(data_entry, "android:scheme")
        .ok_or_else(|| "<data> 条目缺少 android:scheme 属性".to_string())?;

    // 根据 scheme 前缀推测目标 Activity 名称（可能是短名称如 wxapi.WXEntryActivity）
    let target_hint = infer_target_activity_from_scheme(&scheme);

    // 在 manifest 中查找该 Activity（优先精确匹配，再尝试后缀匹配）
    let activity = find_manifest_component(content, "activity", &target_hint)
        .or_else(|| find_manifest_component_suffix(content, "activity", &target_hint))
        .ok_or_else(|| format!("未找到目标 Activity: {}（已尝试后缀匹配）", target_hint))?;

    // 检查 Activity 内是否已有 intent-filter
    let activity_body = &content[activity.start..activity.end];
    let has_intent_filter = activity_body.contains("<intent-filter");

    let indented_data = indent_xml_fragment(data_entry, 12);

    if has_intent_filter {
        // 找到最后一个 </intent-filter> 的位置（相对于 content），在其前面插入 <data>
        let body_start = activity.start;
        let body_end = activity.end_close_start;

        let body = &content[body_start..body_end];
        let mut last_if_close = None;
        let if_close_pattern = "</intent-filter>";
        let mut search_pos = 0;
        while let Some(rel) = body[search_pos..].find(if_close_pattern) {
            last_if_close = Some(body_start + search_pos + rel);
            search_pos += rel + if_close_pattern.len();
        }

        let insert_pos = last_if_close.ok_or("Activity 中找不到 </intent-filter>".to_string())?;
        Ok(insert_before_index(
            content,
            insert_pos,
            &format!("\n{}", indented_data),
        ))
    } else {
        // Activity 没有 intent-filter，在 </activity> 前创建一个新的
        let insert_pos = activity.end_close_start; // </activity> 的位置
        Ok(insert_before_index(
            content,
            insert_pos,
            &format!(
                "\n        <intent-filter>\n{}\n        </intent-filter>",
                indented_data
            ),
        ))
    }
}

/// 通过 android:name 属性的后缀匹配查找 Manifest 组件。
///
/// 当只知道类名的后半部分（如 wxapi.WXEntryActivity）而不知道完整包名时使用。
fn find_manifest_component_suffix(
    content: &str,
    tag: &str,
    name_suffix: &str,
) -> Option<ManifestComponent> {
    if name_suffix.is_empty() {
        return None;
    }
    let re = Regex::new(&format!(
        r#"(?s)<{}\b[^>]*android:name="[^"]*{}"[^>]*>"#,
        regex::escape(tag),
        regex::escape(name_suffix)
    ))
    .ok()?;

    let mat = re.find(content)?;
    let start_tag = &content[mat.start()..mat.end()];
    let start = mat.start();

    if start_tag.trim_end().ends_with("/>") {
        return Some(ManifestComponent {
            start,
            end: mat.end(),
            end_close_start: mat.end() - 2,
        });
    }

    let end_tag = format!("</{}>", tag);
    let open_pattern = format!("<{}", tag);

    let mut depth = 1;
    let mut search_pos = mat.end();

    while search_pos < content.len() {
        if let Some(rel) = content[search_pos..].find(&end_tag) {
            let close_pos = search_pos + rel;
            let segment = &content[search_pos..close_pos];
            let open_count = count_tag_open_occurrences(segment, &open_pattern);

            if depth - open_count == 1 {
                return Some(ManifestComponent {
                    start,
                    end: close_pos + end_tag.len(),
                    end_close_start: close_pos,
                });
            } else {
                depth = depth - 1 + open_count;
                search_pos = close_pos + end_tag.len();
            }
        } else {
            break;
        }
    }
    None
}

/// 从 <data android:scheme="..."> 的 scheme 值推断对应的目标 Activity 全限定名。
///
/// 常见映射：
/// - ${WX_APPID} / wx...  → PACKAGE.wxapi.WXEntryActivity
/// - tencent{QQ_APPID}    → com.tencent.tauth.AuthActivity
/// - 其他                  → 返回 Err 让调用方回退到普通插入
fn infer_target_activity_from_scheme(scheme: &str) -> String {
    // scheme 通常是模板占位符如 ${WX_APPID} 或实际值
    // 通过 module_application_entries 中已有的 activity 条目来反推
    // 这里做基于常见约定的启发式匹配

    // 微信相关 scheme → WXEntryActivity
    if scheme.contains("WX_APPID") || scheme.starts_with("wx") || scheme.contains("wxa") {
        // 返回通配符模式，由调用方在 manifest 中模糊匹配
        // 实际上我们需要找到已存在的 wxapi.WXEntryActivity
        return "wxapi.WXEntryActivity".to_string();
    }

    // QQ 相关
    if scheme.contains("QQ_APPID") || scheme.starts_with("tencent") {
        return "com.tencent.tauth.AuthActivity".to_string();
    }

    // 新浪微博
    if scheme.contains("SINA") || scheme.contains("sina") {
        return "com.sina.weibo.sdk.share.WbShareTransActivity".to_string();
    }

    // 默认：尝试从所有已知 activity 中找包含 intent-filter 的那个
    // 这里返回一个通用标识，让 route_data_to_activity_intent_filter 做模糊搜索
    String::new()
}

fn find_xml_start_tag_with_attr(
    content: &str,
    tag: &str,
    attr: &str,
    value: &str,
) -> Option<ManifestComponent> {
    let re = Regex::new(&format!(r#"(?s)<{}\b[^>]*>"#, regex::escape(tag))).ok()?;
    for mat in re.find_iter(content) {
        let fragment = mat.as_str();
        if android_attr_value(fragment, attr).as_deref() == Some(value) {
            return Some(ManifestComponent {
                start: mat.start(),
                end: mat.end(),
                end_close_start: mat.end(),
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

/// 确保 settings.gradle 包含 pluginManagement 块（必须在文件最前面）
fn ensure_plugin_management_block(content: &str) -> String {
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

// ============================================================================
// XmlManifestEditor — 基于 quick-xml 的 AndroidManifest.xml 安全编辑器
// ============================================================================

/// 使用 quick-xml Reader 进行结构感知的 AndroidManifest.xml 编辑器。
///
/// 与旧的正则字符串操作不同，本编辑器通过完整解析 XML 事件流来定位标签位置，
/// 确保插入操作不会破坏 XML 结构合法性。
pub struct XmlManifestEditor {
    content: String,
}

impl XmlManifestEditor {
    pub fn from_str(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// 替换内部内容（用于外部修复后更新）
    pub fn replace_content(&mut self, new_content: String) {
        self.content = new_content;
    }

    /// 在 <manifest> 开始标签后添加缺失的 <uses-permission> 声明。
    ///
    /// 使用 quick-xml Reader 定位 <manifest ...> 标签的结束位置（'>' 后），
    /// 在该位置逐个插入缺失的权限元素。
    ///
    /// 如果权限字符串不是以 `<` 开头（即裸权限名如 `"android.permission.CAMERA"`），
    /// 自动包裹为 `<uses-permission android:name="..." />` 格式。
    pub fn add_permissions(&mut self, permissions: &[String]) -> Result<(), String> {
        if permissions.is_empty() {
            return Ok(());
        }

        let manifest_end = Self::find_tag_end(&self.content, "manifest")
            .ok_or_else(|| "AndroidManifest.xml 缺少 <manifest> 标签".to_string())?;

        let mut insertion = String::new();
        for perm in permissions {
            let perm = perm.trim();
            if perm.is_empty() {
                continue;
            }
            // 自动补全 <uses-permission> 标签
            let formatted = if perm.starts_with('<') {
                perm.to_string()
            } else {
                format!(r#"<uses-permission android:name="{}" />"#, perm)
            };
            // 检查是否已存在（用 android:name 值判断）
            if let Some(name) = android_attr_value(&formatted, "android:name") {
                if self
                    .content
                    .contains(&format!(r#"android:name="{}""#, name))
                {
                    continue;
                }
            } else if self.content.contains(&formatted) {
                continue;
            }
            insertion.push_str("\n    ");
            insertion.push_str(&formatted);
        }

        if !insertion.is_empty() {
            self.content.insert_str(manifest_end + 1, &insertion);
        }
        Ok(())
    }

    /// 设置/更新 <application> 标签的属性值。
    ///
    /// 使用正则精确匹配并替换属性值，不重写整个文档，
    /// 从而保留原始 XML 的换行和缩进格式。
    /// 如果同一属性出现多次（如模板残留 + 我们添加的），自动去重保留最后一个值。
    pub fn set_application_attr(
        &mut self,
        attr_name: &str,
        attr_value: &str,
    ) -> Result<(), String> {
        // 查找 <application ...> 开始标签（支持多行，贪婪匹配到 >）
        let re = Regex::new(r#"(?s)(<application\b[^>]*)(>)"#)
            .map_err(|e| format!("编译 application 标签正则失败: {}", e))?;

        let escaped_value = escape_xml_attr(attr_value);
        let attr_re = Regex::new(&format!(r#"\s*{}="[^"]*""#, regex::escape(attr_name))).unwrap();

        if let Some(caps) = re.captures(&self.content) {
            let tag_start = caps.get(1).unwrap().start();
            let tag_end = caps.get(2).unwrap().start(); // > 的位置（不包含 > 本身）
            let tag_content = &self.content[tag_start..tag_end];

            if attr_re.is_match(tag_content) {
                // 属性已存在：在原文档中替换所有出现
                let mut result = self.content.clone();
                let mut matches: Vec<_> = attr_re
                    .find_iter(tag_content)
                    .map(|m| (m.start() + tag_start, m.end() + tag_start))
                    .collect();
                matches.sort_by(|a, b| b.0.cmp(&a.0));
                for (start, end) in matches {
                    result.replace_range(
                        start..end,
                        &format!(r#" {}="{}""#, attr_name, escaped_value),
                    );
                }
                self.content = result;
            } else {
                // 属性不存在：在 > 前追加
                let new_tag = format!(
                    "{}\n        {}=\"{}\">",
                    tag_content.trim_end(),
                    attr_name,
                    escaped_value
                );
                self.content = replace_range(
                    &self.content,
                    tag_start,
                    caps.get(2).unwrap().end(),
                    &new_tag,
                );
            }
            Ok(())
        } else {
            Err("AndroidManifest.xml 缺少 <application> 标签".to_string())
        }
    }

    /// 在 </application> 闭合标签前插入子元素。
    ///
    /// 先通过 identity 检查是否已存在，不存在则使用 quick-xml Reader
    /// 精确定位 </application> 的字节位置并插入。
    ///
    /// 返回 true 表示实际插入了新元素，false 表示已存在（跳过）。
    pub fn add_application_entry(
        &mut self,
        entry: &str,
        identity: &EntryIdentity,
    ) -> Result<bool, String> {
        // 检查是否已存在
        if Self::entry_exists_in_application(&self.content, identity) {
            return Ok(false);
        }

        // 裸 <data> 元素需要路由到对应 Activity 的 intent-filter 内部
        if is_bare_data_element(entry) {
            match route_data_to_activity_intent_filter(&self.content, entry) {
                Ok(new_content) => {
                    self.content = new_content;
                    return Ok(true);
                }
                Err(_) => {} // 路由失败，回退到普通插入路径
            }
        }

        // 定位 </application> 的位置
        let close_pos = Self::find_application_close(&self.content)
            .ok_or_else(|| "AndroidManifest.xml 缺少 </application>".to_string())?;

        let indented = format!("\n{}", indent_xml_fragment(entry, 8));
        self.content.insert_str(close_pos, &indented);
        Ok(true)
    }

    /// 在指定 Activity 的 </activity> 闭合标签前插入子元素（如 intent-filter）。
    ///
    /// 支持精确匹配和后缀匹配 activity 名称。
    /// 返回 true 表示实际插入，false 表示已存在。
    pub fn add_activity_child(
        &mut self,
        activity_name: &str,
        child: &str,
        identity: &ChildIdentity,
    ) -> Result<bool, String> {
        // 检查是否已存在
        if Self::child_exists_in_activity(&self.content, activity_name, identity) {
            return Ok(false);
        }

        // 查找目标 Activity（优先精确匹配，再尝试后缀匹配）
        let activity = Self::find_activity_range(&self.content, activity_name)
            .or_else(|| Self::find_activity_range_suffix(&self.content, activity_name))
            .ok_or_else(|| format!("AndroidManifest.xml 缺少 Activity: {}", activity_name))?;

        let indented = format!("\n{}", indent_xml_fragment(child, 12));
        self.content.insert_str(activity.end_close_start, &indented);
        Ok(true)
    }

    /// 使用 quick-xml 完整解析文档，验证 XML 结构合法性。
    ///
    /// 维护真实的标签栈（基于 Start/End 事件对），检测：
    /// - 标签不匹配
    /// - <data> 不在 <intent-filter> 内部
    /// - 栈未正确清空
    pub fn validate_structure(&self) -> Result<(), String> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_str(&self.content);
        reader.config_mut().trim_text(false);

        let mut tag_stack: Vec<String> = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    // 非法嵌套检查
                    if let Some(parent) = tag_stack.last() {
                        if !can_be_child_of(&name, parent) {
                            return Err(format!(
                                "非法嵌套: <{}> 不能作为 <{}> 的子元素",
                                name, parent
                            ));
                        }
                    }
                    tag_stack.push(name);
                }
                Ok(Event::Empty(_)) => {
                    // 自闭合标签，不处理
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match tag_stack.last() {
                        Some(top) if *top == name => {
                            tag_stack.pop();
                        }
                        Some(top) => {
                            return Err(format!(
                                "标签不匹配: 期望 </{}>，实际遇到 </{}>",
                                top, name
                            ));
                        }
                        None => {
                            return Err(format!("多余的闭合标签 </{}>", name));
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(format!("XML 解析错误: {}", e));
                }
                _ => {}
            }
            buf.clear();
        }

        if !tag_stack.is_empty() {
            return Err(format!("文档结束时仍有未闭合标签: {:?}", tag_stack));
        }

        Ok(())
    }

    // === 内部辅助方法 ===

    /// 使用快速查找定位指定标签名开始标签的 '>' 位置（即开始标签结束的下一个字节）
    fn find_tag_end(content: &str, tag_name: &str) -> Option<usize> {
        let target = format!("<{}", tag_name);
        let start = content.find(&target)?;
        // 从 start 开始找 '>'
        content[start..].find('>').map(|p| start + p)
    }

    /// 从后向前查找最后一个不在注释内的 </application>
    fn find_application_close(content: &str) -> Option<usize> {
        let target = "</application>";
        let comment_start = "<!--";
        let comment_end = "-->";

        let mut search_from = content.len();
        loop {
            let rel = content[..search_from].rfind(target)?;
            let pos = rel;

            let before_pos = &content[..pos];
            let last_comment_open = before_pos.rfind(comment_start);
            let last_comment_close = before_pos.rfind(comment_end);

            match (last_comment_open, last_comment_close) {
                (Some(open_idx), Some(close_idx)) if open_idx > close_idx => {
                    search_from = open_idx;
                }
                _ => return Some(pos),
            }
        }
    }

    /// 检查 application 下是否存在指定 identity 的条目
    fn entry_exists_in_application(content: &str, identity: &EntryIdentity) -> bool {
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
            EntryIdentity::Comment(text) => content.contains(text),
            EntryIdentity::Raw(fragment) => content.contains(fragment.trim()),
        }
    }

    /// 检查 Activity 下是否存在指定 identity 的子元素
    fn child_exists_in_activity(
        content: &str,
        activity_name: &str,
        identity: &ChildIdentity,
    ) -> bool {
        let Some(activity) = Self::find_activity_range(content, activity_name)
            .or_else(|| Self::find_activity_range_suffix(content, activity_name))
        else {
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

    /// 通过 android:name 精确查找 Activity 的范围
    fn find_activity_range(content: &str, name: &str) -> Option<XmlTagRange> {
        Self::find_component_range(content, "activity", name)
    }

    /// 通过 android:name 后缀查找 Activity 的范围
    fn find_activity_range_suffix(content: &str, name_suffix: &str) -> Option<XmlTagRange> {
        if name_suffix.is_empty() {
            return None;
        }
        let re = Regex::new(&format!(
            r#"(?s)<activity\b[^>]*android:name="[^"]*{}"[^>]*>"#,
            regex::escape(name_suffix)
        ))
        .ok()?;
        let mat = re.find(content)?;
        Self::parse_component_from_match(content, "activity", mat.start())
    }

    /// 通用组件范围查找（按 tag + android:name 精确匹配）
    fn find_component_range(content: &str, tag: &str, name: &str) -> Option<XmlTagRange> {
        let start = find_xml_start_tag_with_attr(content, tag, "android:name", name)?;
        Self::parse_component_from_match(content, tag, start.start)
    }

    /// 从已找到的开始标签位置解析完整的组件范围（含深度匹配）
    fn parse_component_from_match(
        content: &str,
        tag: &str,
        start_tag_start: usize,
    ) -> Option<XmlTagRange> {
        // 从指定位置开始查找目标标签的开始标签
        let re = Regex::new(&format!(
            r#"(?s)<{}\b[^>]*android:name="#,
            regex::escape(tag)
        ))
        .ok()?;
        let mat = re.find_at(content, start_tag_start)?;
        let start = mat.start();
        let end_of_open_tag = content[start..].find('>').map(|p| start + p)?;

        if content[start..=end_of_open_tag].trim_end().ends_with("/>") {
            return Some(XmlTagRange {
                start,
                end: end_of_open_tag + 1,
                end_close_start: end_of_open_tag - 1,
            });
        }

        let end_tag = format!("</{}>", tag);
        let open_pattern = format!("<{}", tag);
        let mut depth = 1;
        let mut search_pos = end_of_open_tag + 1;

        while search_pos < content.len() {
            if let Some(rel) = content[search_pos..].find(&end_tag) {
                let close_pos = search_pos + rel;
                let segment = &content[search_pos..close_pos];
                let open_count = count_tag_open_occurrences(segment, &open_pattern);

                // 空片段（如 <activity></activity>）不应有嵌套标签
                // count_tag_open_occurrences 对空字符串可能返回错误值，此处做修正
                let actual_open_count = if segment.is_empty() { 0 } else { open_count };

                if depth - actual_open_count == 1 {
                    return Some(XmlTagRange {
                        start,
                        end: close_pos + end_tag.len(),
                        end_close_start: close_pos,
                    });
                } else {
                    depth = depth - 1 + open_count;
                    search_pos = close_pos + end_tag.len();
                }
            } else {
                break;
            }
        }
        None
    }
}

/// XML 标签的范围信息（复用 ManifestComponent）
type XmlTagRange = ManifestComponent;

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
            module_patch_groups: vec![],
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
        // 验证权限被正确包裹为 <uses-permission> 标签（不是裸字符串）
        assert!(
            manifest.contains(r#"<uses-permission android:name="android.permission.ACCESS_BACKGROUND_LOCATION" />"#),
            "权限应被包裹在 <uses-permission> 标签中"
        );
        // 验证 XML 格式未被压成单行（set_application_attr 不应重写整个文档）
        assert!(
            manifest.contains('\n'),
            "AndroidManifest.xml 应保留换行格式"
        );

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

    #[test]
    fn fix_manifest_xml_structure_passes_through_well_formed_xml() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application android:label="Test">
        <activity android:name=".MainActivity" android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
            </intent-filter>
        </activity>
        <meta-data android:name="key" android:value="val" />
    </application>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        assert_eq!(result, xml, "格式正确的 XML 不应被修改");
    }

    #[test]
    fn fix_manifest_xml_structure_auto_closes_unclosed_activity() {
        let xml = r#"<manifest>
    <application>
        <activity android:name=".Main">
    </application>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        assert!(
            result.contains("</activity>"),
            "修复后应包含自动补全的 </activity>"
        );
        // 验证修复后的 XML 可以再次通过校验（幂等性）
        let re_check = fix_manifest_xml_structure(&result).unwrap();
        assert_eq!(re_check, result, "修复结果应幂等，二次调用不再修改");
    }

    #[test]
    fn fix_manifest_xml_structure_fixes_mismatched_tags() {
        // 交叉嵌套：<manifest><a><b></a></b></manifest>
        let xml = r#"<manifest>
    <a><b></a></b>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        // 应在 </a> 前插入 </b>，在末尾补 </a>
        assert!(result.contains("</b>"), "应补全缺失的 </b>");
        // 验证幂等性
        let re_check = fix_manifest_xml_structure(&result).unwrap();
        assert_eq!(re_check, result, "修复结果应幂等");
    }

    #[test]
    fn fix_manifest_xml_structure_preserves_self_closing_tags() {
        let xml = r#"<manifest>
    <application>
        <meta-data android:name="k" android:value="v" />
        <uses-permission android:name="p" />
    </application>
</manifest>
"#;
        let result = fix_manifest_xml_structure(xml).unwrap();
        assert_eq!(result, xml, "含自闭合标签的正确 XML 不应被修改");
    }
}
