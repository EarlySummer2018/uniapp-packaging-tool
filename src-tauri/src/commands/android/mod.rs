pub mod artifacts;
pub mod build_pipeline;
pub(crate) mod certificate;
pub mod commands;
pub(crate) mod env_check;
pub mod environment;
pub mod icons;
pub mod manifest_modules;
pub mod manifest_patches_render;
pub mod manifest_placeholders;
pub mod modules;
pub mod resources;
pub(crate) mod sdk_layout;
pub mod types;
pub mod uts_plugins;

// Re-export types used externally (e.g., lib.rs uses android::AppState)
pub use types::{AppState, BuildArtifact, BuildLogEvent};
