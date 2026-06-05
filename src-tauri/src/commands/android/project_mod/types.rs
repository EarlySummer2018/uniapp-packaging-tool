//! 公共类型定义。

/// Android 工程中的模块名（对应 HBuilder-Integrate-AS 中的 simpleDemo 目录）。
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

/// Android 工程构建修改上下文，包含所有构建配置参数。
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
    pub project_buildscript_dependencies: Vec<String>,
    pub plugin_includes: Vec<String>,
    pub plugin_project_dependencies: Vec<String>,
    pub uts_abi_filters: Vec<String>,
    pub uts_hooks_classes: Vec<String>,
    /// 旧字段（扁平化），保留向后兼容
    pub module_permissions: Vec<String>,
    pub module_application_entries: Vec<String>,
    pub module_pandora_entry_intent_filters: Vec<String>,
    /// 新字段：按模块分组的补丁，用于逐模块安全插入
    pub module_patch_groups: Vec<ManifestPatchGroup>,
    pub manifest_placeholders: String,
    pub dependency_excludes: String,
}

/// Gradle 文件中 android 块内的插入位置。
#[derive(Clone, Copy)]
pub(crate) enum InsertAndroidPosition {
    Top,
}

/// Manifest application 条目的身份标识，用于去重判断。
#[derive(Clone)]
pub enum EntryIdentity {
    MetaData(String),
    Component { tag: String, name: String },
    ProviderAuthority(String),
    Comment(String),
    Raw(String),
}

/// Manifest activity 子元素的身份标识，用于去重判断。
#[derive(Clone)]
pub enum ChildIdentity {
    IntentFilterDataScheme(String),
    IntentFilterAction(String),
    Raw(String),
}
