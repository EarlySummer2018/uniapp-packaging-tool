use std::fs;
use std::path::PathBuf;

use crate::commands::resource::DetectedModule;
use crate::commands::shared::module::templates::{
    apply_module_name_to_tree, merge_properties_to_tree,
};
use crate::commands::shared::module::types::ModuleConfigTree;

#[tauri::command]
pub async fn parse_project_modules(project_path: String) -> Result<ModuleConfigTree, String> {
    let project_dir = PathBuf::from(&project_path);
    let manifest_path = project_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("manifest.json not found at {}", project_path));
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest.json: {}", e))?;

    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

    let mut tree = ModuleConfigTree::default();

    if let Some(app_plus) = manifest.get("app-plus") {
        if let Some(distribute) = app_plus.get("distribute") {
            if let Some(android) = distribute.get("android") {
                if let Some(modules) = android.get("modules") {
                    if let Some(mods_array) = modules.as_array() {
                        for mod_val in mods_array {
                            if let Some(name) = mod_val.get("name").and_then(|n| n.as_str()) {
                                apply_module_name_to_tree(&mut tree, name);
                            }
                        }
                    }
                }
            }
        }
    }

    let props_path = project_dir
        .join("assets")
        .join("data")
        .join("dcloud_properties.xml");
    if props_path.exists() {
        if let Ok(props_content) = fs::read_to_string(&props_path) {
            merge_properties_to_tree(&mut tree, &props_content)?;
        }
    }

    Ok(tree)
}

pub fn module_config_from_detected_modules(modules: &[DetectedModule]) -> ModuleConfigTree {
    let mut tree = ModuleConfigTree::default();
    for module in modules {
        apply_module_name_to_tree(&mut tree, &module.name);
    }
    tree
}

pub fn normalize_config_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

// ---------------------------------------------------------------------------
// Tauri command wrapper for get_module_template
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_module_template(
    module_name: String,
) -> Result<crate::commands::shared::module::types::ModuleTemplate, String> {
    super::templates::get_module_template_sync(&module_name)
}
