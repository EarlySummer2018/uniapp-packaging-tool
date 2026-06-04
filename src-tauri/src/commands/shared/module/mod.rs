pub mod analysis;
pub mod field_specs;
pub mod parsing;
pub mod properties;
pub mod templates;
pub mod types;

// Re-export common types so that external `crate::commands::module::` paths keep working
pub use types::{
    AndroidModuleConfigModule, AndroidModuleConfigReport, LoginProvider, ModuleConfigTree,
};

// Re-export functions used by other modules (e.g. build_android.rs)
pub use analysis::{
    analyze_android_module_config_sync, android_module_artifact_enabled_for_manifest,
    android_module_gradle_dependency_enabled_for_manifest,
    android_module_gradle_repositories_for_manifest,
};
pub use parsing::module_config_from_detected_modules;
pub use properties::generate_dcloud_properties;
pub use templates::{android_module_template_key, get_module_template_sync};
