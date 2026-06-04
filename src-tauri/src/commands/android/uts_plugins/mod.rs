pub(crate) mod aar_unpack;
pub(crate) mod builtin;
pub(crate) mod custom;
pub(crate) mod gradle;

// Re-export main entry functions for use by commands.rs and other modules
pub use builtin::process_builtin_uts_modules;
pub use custom::generate_dcloud_uniplugins_json;
pub use custom::process_custom_uts_plugins_uniapp;
