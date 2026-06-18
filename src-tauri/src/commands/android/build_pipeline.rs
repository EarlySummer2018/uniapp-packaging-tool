//! Android 构建流水线
//!
//! 将 build_android_apk 和 generate_android_project 的共享逻辑提取为可复用的步骤方法

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use tauri::{Manager, Window};

use crate::commands::android::artifacts::{copy_required_aars, inject_huawei_agconnect_json};
use crate::commands::android::environment::{
    android_build_requires_allow_backup_false, android_process_env, expand_home,
    find_apk_in_workspace, resolve_android_build_environment, safe_file_name,
};
use crate::commands::android::icons::{
    apply_android_splashscreen, apply_push_small_icon, generate_icons,
};
use crate::commands::android::manifest_modules::{
    android_module_string_resources, apply_android_manifest_modules, copy_module_activity_sources,
    emit_android_module_config_report, merged_android_module_config, validate_android_config,
};
use crate::commands::android::manifest_patches_render::{
    render_android_module_manifest_patches_for_manifest_impl, render_dependency_excludes_impl,
};
use crate::commands::android::manifest_placeholders::render_android_module_manifest_placeholders;
use crate::commands::android::project_mod;
use crate::commands::android::resources::{
    copy_sdk_assets, import_uniapp_assets, update_dcloud_control,
};
use crate::commands::android::types::{
    emit_log_for_build, render_gradle_dependency_line, timestamp, AndroidBuildEnvironment,
    AndroidManifestPatches, UTS_RUNTIME_DEPS,
};
use crate::commands::module::{
    manifest_push_unipush_v2_enabled, manifest_push_unsupported_version,
    payment_provider_enabled_for_platform, PaymentProvider, PUSH_UNSUPPORTED_VERSION_MESSAGE,
};

const DEFAULT_ANDROIDX_VERSION: &str = "1.0.0";

/// Android 构建上下文，持有构建过程中的所有中间状态
#[allow(dead_code)]
pub struct BuildContext {
    // 输入参数
    pub project_id: String,
    pub resource_path: String,
    pub build_id: String,
    pub manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
    pub module_config: Option<HashMap<String, String>>,

    // 解析后的配置
    pub config: crate::commands::project::ProjectConfig,
    pub sdk_config: crate::commands::sdk::GlobalSdkConfig,
    pub android_env: Option<AndroidBuildEnvironment>,

    // 资源扫描结果
    pub scan: crate::commands::shared::resource_scan::ResourceScanResult,
    pub app_resource_dir: PathBuf,

    // 工作区
    pub workspace: PathBuf,
    pub sdk_layout: crate::commands::android::sdk_layout::AndroidSdkLayout,
    pub sdk_libs: PathBuf,
    pub libs_dst: PathBuf,

    // 模块配置
    pub manifest_modules: Vec<crate::commands::shared::resource::DetectedModule>,
    pub merged_module_config: Option<HashMap<String, String>>,
    pub module_config_report:
        Option<crate::commands::shared::module::types::AndroidModuleConfigReport>,
    pub manifest_value: Option<serde_json::Value>,

    // 依赖收集
    pub extra_deps: BTreeSet<String>,
    pub extra_repos: BTreeSet<String>,
    pub plugin_project_deps: BTreeSet<String>,
    pub plugin_includes: BTreeSet<String>,

    // Manifest 补丁结果
    pub manifest_patches: Option<AndroidManifestPatches>,
    pub manifest_patch_groups: Vec<project_mod::ManifestPatchGroup>,
    pub manifest_placeholders: String,
}

fn push_small_icon_manifest_entry() -> &'static str {
    r#"        <meta-data android:name="com.getui.push.notification.smallIcon" android:resource="@drawable/push_icon" />"#
}

fn push_module_enabled_for(manifest: Option<&serde_json::Value>) -> bool {
    manifest.is_some_and(manifest_push_unipush_v2_enabled)
}

fn push_small_icon_configured_for(
    push_icons: Option<&crate::commands::resource::PushIconsConfig>,
) -> bool {
    push_icons
        .map(|icons| {
            icons
                .small
                .as_deref()
                .map(str::trim)
                .map(|path| !path.is_empty())
                .unwrap_or(false)
                || !icons.small_densities.is_empty()
        })
        .unwrap_or(false)
}

fn should_apply_push_small_icon_for(
    manifest: Option<&serde_json::Value>,
    push_icons: Option<&crate::commands::resource::PushIconsConfig>,
) -> bool {
    push_module_enabled_for(manifest) && push_small_icon_configured_for(push_icons)
}

fn manifest_value_for_build_context(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Option<serde_json::Value> {
    manifest_info.and_then(crate::commands::module::manifest_value_from_info)
}

fn render_buildscript_dependency_line(dep: &str) -> Option<String> {
    let dep = dep.trim();
    if dep.is_empty() {
        return None;
    }
    if dep.starts_with("classpath ") {
        Some(dep.to_string())
    } else {
        Some(format!("classpath '{}'", dep))
    }
}

fn google_oauth_enabled(manifest: Option<&serde_json::Value>) -> bool {
    manifest
        .map(|manifest| {
            crate::commands::module::android_module_gradle_dependency_enabled_for_manifest(
                "login",
                "com.google.android.gms:play-services-auth:19.2.0 (Google登录)",
                Some(manifest),
            )
        })
        .unwrap_or(false)
}

fn stripe_payment_enabled(manifest: Option<&serde_json::Value>) -> bool {
    manifest
        .map(|manifest| {
            payment_provider_enabled_for_platform(manifest, PaymentProvider::Stripe, "android")
        })
        .unwrap_or(false)
}

fn payment_androidx_version(
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    extra_deps: &BTreeSet<String>,
) -> Option<String> {
    report_field_value(config_report, "payment", "androidxVersion").or_else(|| {
        extra_deps
            .iter()
            .any(|dependency| dependency.contains("rootProject.ext.androidxVersion"))
            .then(|| DEFAULT_ANDROIDX_VERSION.to_string())
    })
}

fn report_field_value(
    config_report: Option<&crate::commands::module::AndroidModuleConfigReport>,
    template_key: &str,
    field_key: &str,
) -> Option<String> {
    config_report.and_then(|report| {
        report
            .modules
            .iter()
            .find(|module| module.template_key == template_key)
            .and_then(|module| {
                module
                    .fields
                    .iter()
                    .find(|field| field.key == field_key)
                    .and_then(|field| field.value.as_deref())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
            })
    })
}

fn clean_copied_gradle_outputs(workspace: &Path) -> Result<(), String> {
    for path in [
        workspace.join(".gradle"),
        workspace.join("build"),
        workspace.join(project_mod::MODULE_NAME).join(".gradle"),
        workspace.join(project_mod::MODULE_NAME).join("build"),
    ] {
        if path.exists() {
            std::fs::remove_dir_all(&path)
                .map_err(|e| format!("清理 SDK 模板旧构建产物失败 {}: {}", path.display(), e))?;
        }
    }
    Ok(())
}

impl BuildContext {
    /// Step 0: 创建构建上下文（解析参数、加载配置、准备工作区）
    pub fn new(
        project_id: String,
        resource_path: String,
        build_id: Option<String>,
        manifest_info: Option<crate::commands::resource::UniappManifestInfo>,
        module_config: Option<HashMap<String, String>>,
        window: &Window,
        resolve_env: bool,
    ) -> Result<Self, String> {
        let build_id = build_id
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| {
                if resolve_env {
                    format!("android-{}", timestamp())
                } else {
                    format!("android-gen-{}", timestamp())
                }
            });

        if resolve_env {
            emit_log_for_build(
                window,
                &build_id,
                "info",
                "开始 Android APK 构建流程",
                Some(2),
            );
        } else {
            emit_log_for_build(
                window,
                &build_id,
                "info",
                "开始生成 Android 工程（不执行打包）",
                Some(2),
            );
        }

        let config = crate::commands::project::load_project_config_sync(&project_id)?;
        let sdk_config = crate::commands::sdk::load_global_sdk_config_sync()?;
        validate_android_config(&config, &sdk_config)?;

        let android_env = if resolve_env {
            Some(resolve_android_build_environment()?)
        } else {
            None
        };

        let resource_dir = PathBuf::from(&resource_path);
        if !resource_dir.exists() {
            return Err(format!("资源路径不存在: {}", resource_path));
        }
        let scan = crate::commands::shared::resource_scan::scan_imported_resource(
            &resource_dir,
            &resource_dir,
            false,
        )?;
        let app_resource_dir = PathBuf::from(&scan.app_resource_path);
        emit_log_for_build(
            window,
            &build_id,
            "info",
            &format!("检测到 UniApp AppId: {}", scan.app_id),
            Some(5),
        );

        // ===== 1. 工作区准备 =====
        let workspace_base =
            crate::utils::fs::get_project_config_dir(&project_id).join("workspace");
        crate::utils::fs::ensure_directory(&workspace_base)
            .map_err(|e| format!("创建工作区基础目录失败: {}", e))?;
        let workspace = workspace_base.join(safe_file_name(&build_id));
        let sdk_layout = crate::commands::sdk::resolve_android_sdk_layout(&PathBuf::from(
            &sdk_config.dcloud_android_sdk_path,
        ))?;
        crate::utils::fs::copy_recursive(&sdk_layout.integrate_project_dir, &workspace)
            .map_err(|e| format!("复制 HBuilder-Integrate-AS 到工作区失败: {}", e))?;
        // SDK 从 zip 解压后文件可能只读，copy 会保留权限，需确保目录可写
        crate::utils::fs::ensure_writable_tree(&workspace)
            .map_err(|e| format!("设置工作区写权限失败: {}", e))?;
        clean_copied_gradle_outputs(&workspace)?;
        emit_log_for_build(
            window,
            &build_id,
            "success",
            "已从 SDK 复制 HBuilder-Integrate-AS 到工作区",
            Some(10),
        );

        let sdk_libs = sdk_layout.libs_dir.clone();
        let libs_dst = workspace.join(project_mod::MODULE_NAME).join("libs");

        Ok(Self {
            project_id,
            resource_path,
            build_id,
            manifest_info,
            module_config,
            config,
            sdk_config,
            android_env,
            scan,
            app_resource_dir,
            workspace,
            sdk_layout,
            sdk_libs,
            libs_dst,
            manifest_modules: Vec::new(),
            merged_module_config: None,
            module_config_report: None,
            manifest_value: None,
            extra_deps: BTreeSet::new(),
            extra_repos: BTreeSet::new(),
            plugin_project_deps: BTreeSet::new(),
            plugin_includes: BTreeSet::new(),
            manifest_patches: None,
            manifest_patch_groups: Vec::new(),
            manifest_placeholders: String::new(),
        })
    }

    /// Step 1: 注入基础 AAR
    pub fn inject_base_aars(&self, window: &Window) -> Result<(), String> {
        crate::utils::fs::ensure_directory(&self.libs_dst).map_err(|e| e.to_string())?;
        copy_required_aars(&self.sdk_libs, &self.libs_dst, window)?;
        emit_log_for_build(
            window,
            &self.build_id,
            "success",
            "DCloud SDK 基础 AAR 已注入",
            Some(18),
        );
        Ok(())
    }

    /// Step 2: 解析模块配置 + 处理 UTS 插件
    pub fn process_modules_and_uts(&mut self, window: &Window) -> Result<(), String> {
        self.manifest_modules = self
            .manifest_info
            .as_ref()
            .map(|info| info.detected_modules.clone())
            .unwrap_or(self.scan.detected_modules.clone());
        self.merged_module_config =
            merged_android_module_config(&self.config, self.module_config.take());
        self.manifest_value = manifest_value_for_build_context(self.manifest_info.as_ref());
        if self
            .manifest_value
            .as_ref()
            .is_some_and(manifest_push_unsupported_version)
        {
            emit_log_for_build(
                window,
                &self.build_id,
                "warn",
                PUSH_UNSUPPORTED_VERSION_MESSAGE,
                Some(24),
            );
        }
        self.module_config_report = self.manifest_info.as_ref().map(|info| {
            crate::commands::module::analyze_android_module_config_sync(
                info,
                self.merged_module_config.as_ref(),
            )
        });
        if let Some(report) = &self.module_config_report {
            if !report.all_configured {
                let missing = report
                    .missing_required
                    .iter()
                    .map(|item| format!("{}: {}", item.module_name, item.label))
                    .collect::<Vec<_>>()
                    .join("；");
                return Err(format!("Android 模块配置未填写完整: {}", missing));
            }
            emit_android_module_config_report(window, report);
        }

        if self.scan.uts.has_android_uts_plugins {
            self.extra_repos
                .insert("maven { url 'https://jitpack.io' }".to_string());
            for dep in UTS_RUNTIME_DEPS {
                self.extra_deps.insert((*dep).to_string());
            }
            let android_builtin_modules = self
                .scan
                .uts
                .builtin_modules
                .iter()
                .filter(|module| module.android_dir.is_some())
                .cloned()
                .collect::<Vec<_>>();
            super::uts_plugins::process_builtin_uts_modules(
                &android_builtin_modules,
                &self.sdk_libs,
                &self.libs_dst,
                &mut self.extra_deps,
                window,
            )?;

            let android_custom_plugins = self
                .scan
                .uts
                .custom_plugins
                .iter()
                .filter(|plugin| plugin.android_dir.is_some())
                .cloned()
                .collect::<Vec<_>>();
            if !android_custom_plugins.is_empty() {
                super::uts_plugins::process_custom_uts_plugins_uniapp(
                    &android_custom_plugins,
                    &self.workspace,
                    &self.libs_dst,
                    &mut self.extra_repos,
                    &mut self.extra_deps,
                    &mut self.plugin_includes,
                    &mut self.plugin_project_deps,
                    window,
                )?;
                super::uts_plugins::generate_dcloud_uniplugins_json(
                    &android_custom_plugins,
                    &self.workspace,
                )?;
            }
            emit_log_for_build(
                window,
                &self.build_id,
                "success",
                "UTS 插件依赖已扫描并注入",
                Some(26),
            );
        }
        Ok(())
    }

    /// Step 3: 注入 SDK assets + 应用 Manifest 模块 + 华为推送
    pub fn apply_manifest_modules(&mut self, window: &Window) -> Result<(), String> {
        copy_sdk_assets(&self.sdk_layout.assets_dir, &self.workspace, window)?;
        apply_android_manifest_modules(
            self.manifest_modules.as_slice(),
            self.module_config_report.as_ref(),
            self.manifest_value.as_ref(),
            &self.sdk_libs,
            &self.libs_dst,
            &self.workspace,
            &mut self.extra_repos,
            &mut self.extra_deps,
            window,
        )?;

        // 注入华为推送所需的 agconnect-services.json 文件
        if self.push_module_enabled() {
            inject_huawei_agconnect_json(&self.merged_module_config, &self.workspace, window)?;
        }

        Ok(())
    }

    /// Step 4: 渲染 Manifest 补丁 + Placeholder
    pub fn render_patches(&mut self, _window: &Window) -> Result<(), String> {
        let (mut manifest_patches, mut manifest_patch_groups) =
            render_android_module_manifest_patches_for_manifest_impl(
                self.module_config_report.as_ref(),
                self.manifest_value.as_ref(),
                &self.config.android.package_name,
                &self.scan.app_id,
            );
        if self.should_apply_push_small_icon() {
            let entry = push_small_icon_manifest_entry();
            if !manifest_patches.application_entries.trim().is_empty() {
                manifest_patches.application_entries.push('\n');
            }
            manifest_patches.application_entries.push_str(entry);

            if let Some(group) = manifest_patch_groups
                .iter_mut()
                .find(|group| group.module_name == "push")
            {
                group.application_entries.push(entry.to_string());
            } else {
                manifest_patch_groups.push(project_mod::ManifestPatchGroup {
                    module_name: "push".to_string(),
                    permissions: Vec::new(),
                    application_entries: vec![entry.to_string()],
                    intent_filters: Vec::new(),
                });
            }
        }
        let manifest_placeholders = render_android_module_manifest_placeholders(
            self.module_config_report.as_ref(),
            self.manifest_modules.as_slice(),
            &self.config.android.package_name,
        );

        self.manifest_patches = Some(manifest_patches);
        self.manifest_patch_groups = manifest_patch_groups;
        self.manifest_placeholders = manifest_placeholders;
        Ok(())
    }

    /// Step 5: 构建修改上下文并应用工程补丁
    ///
    /// tolerant_passwords: generate_android_project 用 true（密码缺失不报错），
    /// build_apk 用 false
    pub fn apply_modifications(
        &self,
        window: &Window,
        tolerant_passwords: bool,
    ) -> Result<(), String> {
        let store_key = format!("{}-android-store-password", self.config.id);
        let key_key = format!("{}-android-key-password", self.config.id);

        let store_password = if tolerant_passwords {
            match crate::utils::keychain::get_password(&store_key) {
                Ok(Some(pwd)) => pwd,
                Ok(None) => String::new(),
                Err(e) => return Err(format!("读取 Store 密码失败: {}", e)),
            }
        } else {
            crate::utils::keychain::get_password(&store_key)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Keychain 中缺少 Android Store 密码".to_string())?
        };

        let key_password = if tolerant_passwords {
            match crate::utils::keychain::get_password(&key_key) {
                Ok(Some(pwd)) => pwd,
                Ok(None) => String::new(),
                Err(e) => return Err(format!("读取 Key 密码失败: {}", e)),
            }
        } else {
            crate::utils::keychain::get_password(&key_key)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Keychain 中缺少 Android Key 密码".to_string())?
        };

        let manifest_patches = self
            .manifest_patches
            .as_ref()
            .expect("manifest_patches must be set before apply_modifications");
        let mut project_buildscript_dependencies = self
            .scan
            .uts
            .custom_plugins
            .iter()
            .flat_map(|plugin| plugin.project_dependencies.iter())
            .filter_map(|dep| render_buildscript_dependency_line(dep))
            .collect::<BTreeSet<_>>();
        if google_oauth_enabled(self.manifest_value.as_ref()) {
            project_buildscript_dependencies
                .insert("classpath 'com.google.gms:google-services:4.2.0'".to_string());
        }
        let mut min_sdk = self
            .scan
            .uts
            .custom_plugins
            .iter()
            .filter_map(|plugin| plugin.min_sdk_version)
            .max()
            .map(|min_sdk| min_sdk.max(self.config.android.min_sdk_version))
            .unwrap_or(self.config.android.min_sdk_version);
        if stripe_payment_enabled(self.manifest_value.as_ref()) {
            min_sdk = min_sdk.max(21);
        }

        let modification_ctx = project_mod::BuildModificationContext {
            project_name: safe_file_name(&self.config.name),
            package_name: self.config.android.package_name.clone(),
            appid: self.scan.app_id.clone(),
            dcloud_appkey: self.config.android.dcloud_app_key.clone(),
            app_name: self.config.app.name.clone(),
            string_resources: android_module_string_resources(self.module_config_report.as_ref()),
            version_code: self.config.app.version_code,
            version_name: self.config.app.version.clone(),
            compile_sdk: self.config.android.compile_sdk_version,
            target_sdk: self.config.android.target_sdk_version,
            min_sdk,
            keystore_path: self.config.android.keystore.path.clone(),
            key_alias: self.config.android.keystore.alias.clone(),
            key_password,
            store_password,
            android_allow_backup: if android_build_requires_allow_backup_false(&self.extra_deps) {
                "false".to_string()
            } else {
                "true".to_string()
            },
            androidx_version: payment_androidx_version(
                self.module_config_report.as_ref(),
                &self.extra_deps,
            ),
            extra_repositories: self.extra_repos.clone().into_iter().collect(),
            extra_dependencies: self
                .extra_deps
                .iter()
                .map(|dep| render_gradle_dependency_line(dep))
                .collect(),
            project_buildscript_dependencies: project_buildscript_dependencies
                .into_iter()
                .collect(),
            plugin_includes: self.plugin_includes.clone().into_iter().collect(),
            plugin_project_dependencies: self.plugin_project_deps.clone().into_iter().collect(),
            uts_abi_filters: self
                .scan
                .uts
                .custom_plugins
                .iter()
                .filter_map(|plugin| plugin.abis.as_ref())
                .flat_map(|abis| abis.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            android_abi_filters: self
                .manifest_info
                .as_ref()
                .map(|info| info.android.abi_filters.clone())
                .unwrap_or_default(),
            android_permissions: self
                .manifest_info
                .as_ref()
                .map(|info| info.android.permissions.clone())
                .unwrap_or_default(),
            android_exclude_permissions: self
                .manifest_info
                .as_ref()
                .map(|info| info.android.exclude_permissions.clone())
                .unwrap_or_default(),
            android_schemes: self
                .manifest_info
                .as_ref()
                .map(|info| info.android.schemes.clone())
                .unwrap_or_default(),
            uts_hooks_classes: self
                .scan
                .uts
                .custom_plugins
                .iter()
                .filter_map(|plugin| plugin.hooks_class.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            module_permissions: manifest_patches
                .permissions
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.trim().to_string())
                .collect(),
            module_application_entries: manifest_patches
                .application_entries
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect(),
            module_pandora_entry_intent_filters: manifest_patches
                .pandora_entry_intent_filters
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect(),
            module_patch_groups: self.manifest_patch_groups.clone(),
            manifest_placeholders: self.manifest_placeholders.clone(),
            dependency_excludes: render_dependency_excludes_impl(
                &self
                    .extra_deps
                    .iter()
                    .map(|dep| render_gradle_dependency_line(dep))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        };

        let modifier = project_mod::AndroidProjectModifier::new(self.workspace.clone())?;
        modifier.apply_all_modifications(&modification_ctx)?;
        emit_log_for_build(
            window,
            &self.build_id,
            "success",
            "已应用 Android 工程补丁",
            Some(38),
        );
        Ok(())
    }

    /// Step 6: 导入资源（uniapp assets / dcloud_control / icons / splashscreen）
    pub fn import_resources(&self, window: &Window) -> Result<(), String> {
        import_uniapp_assets(&self.app_resource_dir, &self.workspace, &self.scan.app_id)?;
        emit_log_for_build(
            window,
            &self.build_id,
            "success",
            "UniApp 资源已导入 assets/apps",
            Some(48),
        );

        update_dcloud_control(&self.workspace, &self.scan.app_id)?;
        emit_log_for_build(
            window,
            &self.build_id,
            "success",
            "dcloud_control.xml 已更新",
            Some(55),
        );

        let android_icons = self
            .manifest_info
            .as_ref()
            .and_then(|info| info.android_icons.as_ref());
        generate_icons(android_icons, &self.workspace, window)?;
        emit_log_for_build(
            window,
            &self.build_id,
            "success",
            "Android 图标已导入",
            Some(64),
        );

        let push_icons = if self.push_module_enabled() {
            self.manifest_info
                .as_ref()
                .and_then(|info| info.push_icons.as_ref())
        } else {
            None
        };
        apply_push_small_icon(push_icons, &self.workspace, window)?;

        let splashscreen = self
            .manifest_info
            .as_ref()
            .and_then(|info| info.splashscreen.as_ref())
            .or(self.scan.splashscreen.as_ref());
        apply_android_splashscreen(splashscreen, &self.workspace, window)?;
        emit_log_for_build(
            window,
            &self.build_id,
            "success",
            "Android 启动图已处理",
            Some(66),
        );
        Ok(())
    }

    /// Step 7: 复制模块 Activity 源文件 + 最终校验
    pub fn finalize(&self, window: &Window) -> Result<(), String> {
        // 复制模块 Activity Java 源文件（如微信 WXEntryActivity.java）
        copy_module_activity_sources(
            self.manifest_modules.as_slice(),
            &self.sdk_layout.src_dir,
            &self.workspace,
            &self.config.android.package_name,
            window,
        )?;

        // 所有工程补丁完成后，最终校验并修复 AndroidManifest.xml 结构
        project_mod::validate_and_fix_final_manifest(&self.workspace)?;
        Ok(())
    }

    fn push_module_enabled(&self) -> bool {
        push_module_enabled_for(self.manifest_value.as_ref())
    }

    fn should_apply_push_small_icon(&self) -> bool {
        should_apply_push_small_icon_for(
            self.manifest_value.as_ref(),
            self.manifest_info
                .as_ref()
                .and_then(|info| info.push_icons.as_ref()),
        )
    }

    /// Step 8 (仅 build_apk): 执行 Gradle + 收集 APK
    pub async fn execute_gradle_and_collect(
        self,
        window: &Window,
    ) -> Result<super::types::BuildArtifact, String> {
        let android_env = self
            .android_env
            .expect("android_env must be set for build_apk mode");

        emit_log_for_build(
            window,
            &self.build_id,
            "info",
            "执行 Gradle assembleRelease",
            Some(70),
        );
        let app_handle = window.app_handle().clone();
        let output = crate::utils::process::run_command_streaming_with_env_tagged(
            &android_env.gradle_bin.to_string_lossy(),
            &["assembleRelease".to_string()],
            &self.workspace.to_string_lossy(),
            &android_process_env(&android_env),
            app_handle,
            "build-log",
            crate::utils::process::StreamLogMeta {
                build_id: self.build_id.clone(),
                platform: "android".to_string(),
            },
        )
        .await
        .map_err(|e| format!("执行 Gradle 失败: {}", e))?;
        if !output.success {
            return Err(format!("Gradle 构建失败，退出码: {:?}", output.exit_code));
        }

        let apk = find_apk_in_workspace(&self.workspace)
            .into_iter()
            .next()
            .ok_or_else(|| "Gradle 成功结束，但未找到 APK 产物".to_string())?;
        let output_dir = expand_home(&self.config.output_dir);
        crate::utils::fs::ensure_directory(&output_dir)
            .map_err(|e| format!("创建输出目录失败: {}", e))?;
        let ts = timestamp();
        let dest = output_dir.join(format!("{}-v{}.apk", ts, self.config.app.version));
        std::fs::copy(&apk, &dest).map_err(|e| format!("复制 APK 到输出目录失败: {}", e))?;
        let size_bytes = std::fs::metadata(&dest)
            .map(|m| m.len())
            .unwrap_or_default();
        emit_log_for_build(
            window,
            &self.build_id,
            "success",
            &format!("Android 打包完成: {}", dest.display()),
            Some(100),
        );

        Ok(super::types::BuildArtifact {
            platform: "android".to_string(),
            path: dest.to_string_lossy().to_string(),
            file_name: dest
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("app.apk")
                .to_string(),
            size_bytes,
            build_id: self.build_id,
        })
    }

    /// 获取工作区路径（用于 generate_android_project 返回值）
    pub fn workspace_path(&self) -> String {
        self.workspace.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_small_icon_requires_push_module_and_icon_config() {
        let push_icons = crate::commands::resource::PushIconsConfig {
            small: Some("/tmp/push.png".to_string()),
            ..Default::default()
        };
        let valid_manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Push": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "unipush": {
                                "version": "2"
                            }
                        }
                    }
                }
            }
        });
        let invalid_manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Push": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "push": {
                            "unipush": {
                                "version": "1"
                            }
                        }
                    }
                }
            }
        });

        assert!(should_apply_push_small_icon_for(
            Some(&valid_manifest),
            Some(&push_icons),
        ));
        assert!(!should_apply_push_small_icon_for(
            Some(&invalid_manifest),
            Some(&push_icons),
        ));
        assert!(!should_apply_push_small_icon_for(
            Some(&valid_manifest),
            None,
        ));
    }

    #[test]
    fn build_context_manifest_value_uses_cached_manifest_when_path_is_missing() {
        let project_root =
            std::env::temp_dir().join(format!("unipack-build-manifest-{}", uuid::Uuid::new_v4()));
        let manifest = serde_json::json!({
            "appid": "__UNI__BUILD_CACHE",
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "payment": {
                            "weixin": {
                                "appid": "wx-build"
                            }
                        }
                    }
                }
            }
        });
        let mut info = crate::commands::shared::resource::parse_uniapp_manifest(
            &manifest,
            &project_root.join("manifest.json"),
            &project_root,
            None,
        );
        info.manifest_path = project_root
            .join("manifest-was-not-read-again.json")
            .to_string_lossy()
            .to_string();

        let cached = manifest_value_for_build_context(Some(&info))
            .expect("BuildContext should use cached manifestValue");

        assert_eq!(
            cached
                .pointer("/app-plus/distribute/sdkConfigs/payment/weixin/appid")
                .and_then(|value| value.as_str()),
            Some("wx-build")
        );
    }

    #[test]
    fn google_oauth_adds_official_project_buildscript_dependency() {
        let google_manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "oauth": {
                            "google": {}
                        }
                    }
                }
            }
        });
        let facebook_manifest = serde_json::json!({
            "app-plus": {
                "distribute": {
                    "sdkConfigs": {
                        "oauth": {
                            "facebook": {}
                        }
                    }
                }
            }
        });

        assert!(google_oauth_enabled(Some(&google_manifest)));
        assert!(!google_oauth_enabled(Some(&facebook_manifest)));
    }

    #[test]
    fn stripe_payment_requires_android_min_sdk_21() {
        let stripe_manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Payment": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "payment": {
                            "stripe": {
                                "__platform__": ["android"]
                            }
                        }
                    }
                }
            }
        });
        let alipay_manifest = serde_json::json!({
            "app-plus": {
                "modules": {
                    "Payment": {}
                },
                "distribute": {
                    "sdkConfigs": {
                        "payment": {
                            "alipay": {
                                "__platform__": ["android"]
                            }
                        }
                    }
                }
            }
        });

        assert!(stripe_payment_enabled(Some(&stripe_manifest)));
        assert!(!stripe_payment_enabled(Some(&alipay_manifest)));
    }

    #[test]
    fn payment_androidx_version_prefers_config_report_and_falls_back_when_dependency_needs_it() {
        let report = crate::commands::module::AndroidModuleConfigReport {
            modules: vec![crate::commands::module::AndroidModuleConfigModule {
                name: "Payment".to_string(),
                template_key: "payment".to_string(),
                category: "payment".to_string(),
                platforms: vec!["android".to_string()],
                source: "test".to_string(),
                fields: vec![
                    crate::commands::shared::module::types::AndroidModuleConfigField {
                        key: "androidxVersion".to_string(),
                        label: "AndroidX 版本".to_string(),
                        required: false,
                        secret: false,
                        value: Some("1.7.0".to_string()),
                        value_source: Some("user".to_string()),
                        placeholder: "1.0.0".to_string(),
                        field_type: "text".to_string(),
                    },
                ],
            }],
            missing_required: Vec::new(),
            all_configured: true,
        };
        let mut deps = BTreeSet::new();
        deps.insert("androidx.appcompat:appcompat:${rootProject.ext.androidxVersion}".to_string());

        assert_eq!(
            payment_androidx_version(Some(&report), &deps).as_deref(),
            Some("1.7.0")
        );
        assert_eq!(
            payment_androidx_version(None, &deps).as_deref(),
            Some(DEFAULT_ANDROIDX_VERSION)
        );
        assert_eq!(payment_androidx_version(None, &BTreeSet::new()), None);
    }

    #[test]
    fn copied_gradle_outputs_are_removed_without_touching_sources() {
        let workspace =
            std::env::temp_dir().join(format!("unipack-android-clean-{}", uuid::Uuid::new_v4()));
        let source_dir = workspace.join(project_mod::MODULE_NAME).join("src/main");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("AndroidManifest.xml"), "<manifest />").unwrap();

        for stale_dir in [
            workspace.join(".gradle"),
            workspace.join("build"),
            workspace.join(project_mod::MODULE_NAME).join(".gradle"),
            workspace.join(project_mod::MODULE_NAME).join("build"),
        ] {
            std::fs::create_dir_all(&stale_dir).unwrap();
            std::fs::write(stale_dir.join("stale.bin"), "old").unwrap();
        }

        clean_copied_gradle_outputs(&workspace).unwrap();

        assert!(source_dir.join("AndroidManifest.xml").is_file());
        assert!(!workspace.join(".gradle").exists());
        assert!(!workspace.join("build").exists());
        assert!(!workspace
            .join(project_mod::MODULE_NAME)
            .join(".gradle")
            .exists());
        assert!(!workspace
            .join(project_mod::MODULE_NAME)
            .join("build")
            .exists());

        let _ = std::fs::remove_dir_all(workspace);
    }
}
