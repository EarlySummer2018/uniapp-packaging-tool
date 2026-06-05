//! AndroidProjectModifier 核心实现：对 Android 工程工作区执行所有构建修改。

use super::gradle::{
    ensure_android_block_content, ensure_apply_plugin_after_android_application,
    ensure_build_type_signing_config, ensure_buildscript_dependency, ensure_buildscript_repository,
    ensure_dependencies_block_content, ensure_gradle_statement, ensure_plugin_management_block,
    ensure_repositories_in_allprojects, ensure_repositories_in_drm, ensure_signing_configs_block,
    ensure_top_level_block_content, escape_gradle_double_quoted, escape_gradle_single_quoted,
    render_aapt_options, render_packaging_options, render_signing_configs, render_source_sets,
    replace_or_insert_android_assignment, replace_or_insert_default_config_assignment,
    set_manifest_placeholders, set_or_insert_root_project_name, validate_base_gradle_dependencies,
};
use super::manifest::{
    child_identity, entry_identity, escape_xml_attr, fix_manifest_xml_structure,
    format_entry_description, set_meta_data_value, set_string_resource,
};
use super::types::{
    BuildModificationContext, EntryIdentity, InsertAndroidPosition, ManifestPatchGroup, MODULE_NAME,
};
use super::xml_editor::XmlManifestEditor;
use regex::Regex;
use std::path::{Path, PathBuf};

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
            let repositories = dependency_repositories_with_defaults(ctx);
            content = ensure_repositories_in_drm(&content, &repositories);
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
            let repositories = dependency_repositories_with_defaults(ctx);
            content = ensure_repositories_in_allprojects(&content, &repositories);
        }
        if uses_huawei_agconnect(ctx) {
            content = ensure_buildscript_repository(
                &content,
                "maven { url 'https://developer.huawei.com/repo/' }",
            );
            content = ensure_buildscript_dependency(
                &content,
                "classpath 'com.huawei.agconnect:agcp:1.9.1.301'",
            );
        }

        self.validate_gradle_syntax(&content, &path)?;
        self.write_file(&path, &content)
    }

    fn modify_app_build_gradle(&self, ctx: &BuildModificationContext) -> Result<(), String> {
        let path = self.workspace_dir.join(MODULE_NAME).join("build.gradle");
        let original_content = self.read_file(&path)?;
        let mut content = original_content.clone();

        if uses_huawei_agconnect(ctx) {
            content =
                ensure_apply_plugin_after_android_application(&content, "com.huawei.agconnect");
        }

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
        content = ensure_android_block_content(&content, &render_aapt_options())?;

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

        // 校验基础依赖库是否存在（文档 4.2 节 ① 要求的关键依赖）
        validate_base_gradle_dependencies(&content, &path)?;

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

            // Fix 1: 去重 <application> 标签中的重复属性（如 allowBackup 出现两次）
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

    pub(crate) fn read_file(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))
    }

    pub(crate) fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
        }
        std::fs::write(path, content).map_err(|e| format!("写入文件失败 {}: {}", path.display(), e))
    }
}

fn uses_huawei_agconnect(ctx: &BuildModificationContext) -> bool {
    ctx.extra_dependencies.iter().any(|dependency| {
        dependency.contains("com.huawei.hms:push") || dependency.contains("com.getui.opt:hwp")
    })
}

fn dependency_repositories_with_defaults(ctx: &BuildModificationContext) -> Vec<String> {
    let mut repositories = vec!["google()".to_string(), "mavenCentral()".to_string()];
    for repository in &ctx.extra_repositories {
        let repository = repository.trim();
        if repository.is_empty() {
            continue;
        }
        if repositories
            .iter()
            .any(|existing| existing.trim() == repository)
        {
            continue;
        }
        repositories.push(repository.to_string());
    }
    repositories
}
