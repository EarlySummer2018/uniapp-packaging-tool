// 平台目录
pub mod android;
pub mod harmony;
pub mod ios;
pub mod shared;

// 兼容性 shim：保持 crate::commands::xxx 旧路径有效（lib.rs 零改动）
pub mod build_android;
pub mod build_harmony;
pub mod build_history;
pub mod build_ios;
pub mod certificate;
pub mod env;
pub mod files;
pub mod module;
pub mod project;
pub mod resource;
pub mod sdk;
pub mod settings;
