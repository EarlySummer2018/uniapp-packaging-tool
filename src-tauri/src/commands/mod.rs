// 平台目录
pub mod android;
pub mod harmony;
pub mod ios;
pub mod shared;

// 兼容性 shim：保留仍被源码引用的旧路径/便捷路径
pub mod build_android;
pub mod build_history;
pub mod module;
pub mod project;
pub mod resource;
pub mod sdk;
