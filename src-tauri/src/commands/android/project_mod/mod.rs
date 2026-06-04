//! Android 工程补丁器模块。
//!
//! 构建流程会先从用户配置的 DCloud Android 离线 SDK 复制
//! `HBuilder-Integrate-AS` 到临时工作区，本模块只修改这份工作区副本。
//!
//! 本模块从原 `utils::android_project_mod` 迁移而来，按职责拆分为子模块：
//! - [`types`] — 公共类型定义（常量、结构体、枚举）
//! - [`modifier`] — AndroidProjectModifier 核心实现
//! - [`gradle`] — Gradle 文件操作与块解析
//! - [`manifest`] — AndroidManifest.xml 处理
//! - [`xml_editor`] — 基于 quick-xml 的结构化编辑器

pub mod gradle;
pub mod manifest;
pub mod modifier;
pub mod types;
pub mod xml_editor;

#[cfg(test)]
mod tests;

// Re-export 关键公共 API，保持外部调用路径兼容
pub use types::{BuildModificationContext, ManifestPatchGroup, MODULE_NAME};
pub use modifier::AndroidProjectModifier;
pub use manifest::validate_and_fix_final_manifest;
