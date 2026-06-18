mod android_manifest;
mod android_report;
mod common;
mod ios_report;
mod payment;

use std::collections::HashMap;

use crate::commands::resource::UniappManifestInfo;
use crate::commands::shared::module::types::{AndroidModuleConfigReport, IosModuleConfigReport};

pub use android_manifest::{
    android_amap_geolocation_enabled, android_amap_map_enabled,
    android_module_artifact_enabled_for_manifest,
    android_module_gradle_dependency_enabled_for_manifest,
    android_module_gradle_repositories_for_manifest,
};
pub use android_report::android_module_config_report_from_value;
pub use common::manifest_value_from_info;
pub use ios_report::ios_module_config_report_from_value;
pub use payment::{
    manifest_payment_provider_value, payment_provider_enabled_for_platform, PaymentProvider,
};

// parse_project_modules 与 module_config_from_detected_modules 已移至 parsing.rs

#[tauri::command]
pub async fn analyze_android_module_config(
    manifest_info: UniappManifestInfo,
    user_config: Option<HashMap<String, String>>,
) -> Result<AndroidModuleConfigReport, String> {
    Ok(analyze_android_module_config_sync(
        &manifest_info,
        user_config.as_ref(),
    ))
}

pub fn analyze_android_module_config_sync(
    manifest_info: &UniappManifestInfo,
    user_config: Option<&HashMap<String, String>>,
) -> AndroidModuleConfigReport {
    let manifest_value = manifest_value_from_info(manifest_info);
    android_module_config_report_from_value(
        &manifest_info.detected_modules,
        manifest_value.as_ref(),
        user_config,
    )
}

#[tauri::command]
pub async fn analyze_ios_module_config(
    manifest_info: UniappManifestInfo,
    user_config: Option<HashMap<String, String>>,
) -> Result<IosModuleConfigReport, String> {
    Ok(analyze_ios_module_config_sync(
        &manifest_info,
        user_config.as_ref(),
    ))
}

pub fn analyze_ios_module_config_sync(
    manifest_info: &UniappManifestInfo,
    user_config: Option<&HashMap<String, String>>,
) -> IosModuleConfigReport {
    let manifest_value = manifest_value_from_info(manifest_info);
    ios_module_config_report_from_value(
        &manifest_info.detected_modules,
        manifest_value.as_ref(),
        Some(&manifest_info.ios_privacy_descriptions),
        user_config,
    )
}

#[cfg(test)]
mod tests;
