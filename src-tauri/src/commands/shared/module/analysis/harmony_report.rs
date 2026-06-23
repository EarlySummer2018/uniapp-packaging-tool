use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::types::{
    HarmonyModuleConfigField, HarmonyModuleConfigModule, HarmonyModuleConfigReport,
};

pub fn harmony_module_config_report_from_value(
    modules: &[DetectedModule],
    manifest: Option<&serde_json::Value>,
) -> HarmonyModuleConfigReport {
    let mut report = HarmonyModuleConfigReport::default();

    for module in modules {
        if module.category != "map"
            || module.source != "app-harmony"
            || !module
                .platforms
                .iter()
                .any(|platform| platform == "harmony")
        {
            continue;
        }
        let Some(uni_map) = harmony_uni_map_config(manifest) else {
            continue;
        };
        if uni_map.is_empty() {
            continue;
        }

        let key = harmony_uni_map_tencent_key(manifest).map(ToString::to_string);
        report.modules.push(HarmonyModuleConfigModule {
            name: module.name.clone(),
            template_key: "map".to_string(),
            category: module.category.clone(),
            platforms: module.platforms.clone(),
            source: module.source.clone(),
            fields: vec![HarmonyModuleConfigField {
                key: "TENCENT_MAP_KEY".to_string(),
                label: "腾讯地图 Key".to_string(),
                required: true,
                secret: true,
                value_source: key.as_ref().map(|_| "manifest".to_string()),
                value: key,
                placeholder: "app-harmony.distribute.modules.uni-map.tencent.key".to_string(),
                field_type: "text".to_string(),
            }],
        });
    }

    report
}

pub fn harmony_uni_map_tencent_key(manifest: Option<&serde_json::Value>) -> Option<&str> {
    harmony_uni_map_config(manifest)?
        .get("tencent")?
        .as_object()?
        .get("key")?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn harmony_uni_map_config(
    manifest: Option<&serde_json::Value>,
) -> Option<&serde_json::Map<String, serde_json::Value>> {
    manifest?
        .get("app-harmony")?
        .get("distribute")?
        .get("modules")?
        .get("uni-map")?
        .as_object()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harmony_map_module() -> DetectedModule {
        DetectedModule {
            name: "uni-map".to_string(),
            category: "map".to_string(),
            platforms: vec!["harmony".to_string()],
            configured: false,
            required_keys: Vec::new(),
            source: "app-harmony".to_string(),
        }
    }

    #[test]
    fn harmony_report_reads_tencent_map_key_from_manifest() {
        let manifest = serde_json::json!({
            "app-harmony": {
                "distribute": {
                    "modules": {
                        "uni-map": {
                            "tencent": {
                                "key": "312312"
                            }
                        }
                    }
                }
            }
        });

        let report =
            harmony_module_config_report_from_value(&[harmony_map_module()], Some(&manifest));
        let module = report
            .modules
            .first()
            .expect("map module should be reported");
        let field = module.fields.first().expect("map key should be reported");
        assert_eq!(field.key, "TENCENT_MAP_KEY");
        assert_eq!(field.value.as_deref(), Some("312312"));
        assert_eq!(field.value_source.as_deref(), Some("manifest"));
    }

    #[test]
    fn harmony_report_skips_empty_uni_map_object() {
        let manifest = serde_json::json!({
            "app-harmony": {
                "distribute": {
                    "modules": {
                        "uni-map": {}
                    }
                }
            }
        });

        let report =
            harmony_module_config_report_from_value(&[harmony_map_module()], Some(&manifest));
        assert!(report.modules.is_empty());
    }
}
