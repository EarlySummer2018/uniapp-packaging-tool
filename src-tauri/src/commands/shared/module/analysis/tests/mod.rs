use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::analysis::{
    analyze_android_module_config_sync, analyze_ios_module_config_sync,
    android_module_artifact_enabled_for_manifest, android_module_config_report_from_value,
    android_module_gradle_dependency_enabled_for_manifest,
    android_module_gradle_repositories_for_manifest,
};
use crate::commands::shared::module::parsing::module_config_from_detected_modules;
use crate::commands::shared::module::properties::generate_dcloud_properties;
use crate::commands::shared::module::types::{
    LoginModuleConfig, LoginProvider, MapModuleConfig, ModuleConfigTree, PaymentModuleConfig,
    PushModuleConfig, ShareModuleConfig, SimpleModuleConfig, StatisticModuleConfig,
};
use crate::commands::shared::resource::parse_uniapp_manifest;

mod android_report;
mod platform_filters;
mod properties;

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}.xml", name, uuid::Uuid::new_v4()))
}

fn module_config_with_camera_share_oauth_payment() -> ModuleConfigTree {
    let mut config = ModuleConfigTree::default();
    config.camera = Some(SimpleModuleConfig { enabled: true });
    config.share = Some(ShareModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        qq: Some(HashMap::new()),
        sina: Some(HashMap::new()),
    });
    config.login = Some(LoginModuleConfig {
        enabled: true,
        providers: vec![
            LoginProvider {
                name: "weixin".to_string(),
                enabled: true,
                config: HashMap::new(),
            },
            LoginProvider {
                name: "qq".to_string(),
                enabled: true,
                config: HashMap::new(),
            },
        ],
    });
    config.payment = Some(PaymentModuleConfig {
        enabled: true,
        weixin: Some(HashMap::new()),
        alipay: Some(HashMap::new()),
        paypal: Some(HashMap::new()),
        stripe: Some(HashMap::new()),
        google: Some(HashMap::new()),
    });
    config
}
