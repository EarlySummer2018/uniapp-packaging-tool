pub(super) fn resolve_ios_manifest_info(
    config: &crate::commands::project::ProjectConfig,
    supplied: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<crate::commands::resource::UniappManifestInfo>, String> {
    if !config.local_path.trim().is_empty() {
        let mut local =
            crate::commands::shared::resource::read_uniapp_manifest_sync(&config.local_path)
                .map_err(|e| {
                    format!(
                        "读取 iOS 本地项目 manifest.json 失败 ({}): {}",
                        config.local_path, e
                    )
                })?;
        if let Some(supplied) = supplied {
            overlay_supplied_ios_manifest_info(&mut local, supplied);
        }
        return Ok(Some(local));
    }
    Ok(supplied.cloned())
}

fn overlay_supplied_ios_manifest_info(
    local: &mut crate::commands::resource::UniappManifestInfo,
    supplied: &crate::commands::resource::UniappManifestInfo,
) {
    local.detected_modules = supplied.detected_modules.clone();
    if !supplied.ios_privacy_descriptions.is_empty() {
        local
            .ios_privacy_descriptions
            .extend(supplied.ios_privacy_descriptions.clone());
    }
    overlay_supplied_sdk_configs(&mut local.manifest_value, &supplied.manifest_value);
    overlay_supplied_ios_distribute(&mut local.manifest_value, &supplied.manifest_value);
}

fn overlay_supplied_sdk_configs(
    local_manifest: &mut Option<serde_json::Value>,
    supplied_manifest: &Option<serde_json::Value>,
) {
    let Some(supplied_sdk_configs) = supplied_manifest
        .as_ref()
        .and_then(|manifest| {
            manifest
                .get("app-plus")?
                .get("distribute")?
                .get("sdkConfigs")
        })
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    if local_manifest.is_none() {
        *local_manifest = supplied_manifest.clone();
    }
    let Some(local_manifest) = local_manifest.as_mut() else {
        return;
    };
    let Some(local_sdk_configs) =
        ensure_json_object_path(local_manifest, &["app-plus", "distribute", "sdkConfigs"])
    else {
        return;
    };
    for (key, value) in supplied_sdk_configs {
        local_sdk_configs.insert(key.clone(), value.clone());
    }
}

fn overlay_supplied_ios_distribute(
    local_manifest: &mut Option<serde_json::Value>,
    supplied_manifest: &Option<serde_json::Value>,
) {
    let Some(supplied_ios) = supplied_manifest
        .as_ref()
        .and_then(|manifest| manifest.get("app-plus")?.get("distribute")?.get("ios"))
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    if local_manifest.is_none() {
        *local_manifest = supplied_manifest.clone();
    }
    let Some(local_manifest) = local_manifest.as_mut() else {
        return;
    };
    let Some(local_ios) =
        ensure_json_object_path(local_manifest, &["app-plus", "distribute", "ios"])
    else {
        return;
    };
    for (key, value) in supplied_ios {
        local_ios.insert(key.clone(), value.clone());
    }
}

fn ensure_json_object_path<'a>(
    root: &'a mut serde_json::Value,
    path: &[&str],
) -> Option<&'a mut serde_json::Map<String, serde_json::Value>> {
    let mut current = root;
    for key in path {
        if !current.is_object() {
            *current = serde_json::Value::Object(serde_json::Map::new());
        }
        let map = current.as_object_mut()?;
        current = map
            .entry((*key).to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    }
    if !current.is_object() {
        *current = serde_json::Value::Object(serde_json::Map::new());
    }
    current.as_object_mut()
}

pub(super) fn effective_app_name(
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> String {
    manifest_info
        .and_then(|info| info.app_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&config.app.name)
        .to_string()
}

pub(super) fn effective_app_version(
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> String {
    manifest_info
        .and_then(|info| info.version_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&config.app.version)
        .to_string()
}

pub(super) fn effective_app_version_code(
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> u32 {
    manifest_info
        .and_then(|info| info.version_code)
        .unwrap_or(config.app.version_code)
}

pub(super) fn validate_ios_app_id(
    resource_app_id: &str,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let Some(manifest_app_id) = manifest_info
        .and_then(|info| info.app_id.as_deref())
        .filter(|app_id| !app_id.trim().is_empty())
    else {
        return Ok(());
    };
    if manifest_app_id == resource_app_id {
        return Ok(());
    }
    Err(format!(
        "iOS 本地 manifest AppId ({}) 与导入资源 AppId ({}) 不一致，无法安全配置 control.xml",
        manifest_app_id, resource_app_id
    ))
}

pub(super) fn validate_ios_config(
    config: &crate::commands::project::ProjectConfig,
    sdk_config: &crate::commands::sdk::GlobalSdkConfig,
) -> Result<(), String> {
    if sdk_config.dcloud_ios_sdk_path.trim().is_empty() {
        return Err("请先在 SDK & 环境管理中配置 DCloud iOS 离线 SDK".to_string());
    }
    if config.ios.dcloud_app_key.trim().is_empty() {
        return Err("请先配置 iOS DCloud AppKey".to_string());
    }
    if config.ios.bundle_id.trim().is_empty() || config.ios.team_id.trim().is_empty() {
        return Err("请先配置 iOS Bundle ID 和 Team ID".to_string());
    }
    if config.ios.provisioning_profile.trim().is_empty() {
        return Err("请先选择 iOS 描述文件 mobileprovision".to_string());
    }
    if !config.ios.certificate.trim().is_empty() && !config.ios.has_certificate_password {
        return Err("导入 iOS P12 证书需要先保存证书密码".to_string());
    }
    Ok(())
}

pub(super) fn ensure_macos(action: &str) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        Ok(())
    } else {
        Err(format!("{} 仅支持 macOS", action))
    }
}
