use std::fs::File;
use std::io::Read;

use super::manifest::{parse_manifest_content, parse_uniapp_manifest};
use super::module_detection::check_module_configured_in_props;
use super::types::{PlatformPackages, ZipAnalysisResult};

fn read_zip_entry_to_string(entry: &mut zip::read::ZipFile) -> Result<String, String> {
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    String::from_utf8(buf).map_err(|e| format!("Invalid UTF-8 in {}: {}", entry.name(), e))
}

pub(super) async fn analyze_uploaded_zip_impl(
    zip_path: String,
) -> Result<ZipAnalysisResult, String> {
    let file = File::open(&zip_path).map_err(|e| format!("Cannot open zip file: {}", e))?;

    let mut reader =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip format: {}", e))?;

    let mut result = ZipAnalysisResult {
        app_name: None,
        app_id: None,
        version_name: None,
        version_code: None,
        package_names: PlatformPackages {
            android_package: None,
            ios_bundle_id: None,
            harmony_bundle: None,
        },
        detected_modules: vec![],
        has_dcloud_properties: false,
        has_resources: false,
        resource_files: vec![],
        error: None,
    };

    let mut manifest_content: Option<String> = None;
    let mut props_content: Option<String> = None;
    let mut resource_entries: Vec<String> = vec![];

    for i in 0..reader.len() {
        let mut entry = reader.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        if name.ends_with("manifest.json")
            && !name.contains("node_modules")
            && !name.contains("unpackage")
        {
            match read_zip_entry_to_string(&mut entry) {
                Ok(content) => {
                    manifest_content = Some(content);
                }
                Err(_) => {}
            }
        }

        if name.ends_with("dcloud_properties.xml") {
            match read_zip_entry_to_string(&mut entry) {
                Ok(content) => {
                    props_content = Some(content);
                    result.has_dcloud_properties = true;
                }
                Err(_) => {}
            }
        }

        if (name.starts_with("www/")
            || name.contains("/assets/")
            || name.starts_with("unpackage/resources/"))
            && !name.ends_with('/')
        {
            resource_entries.push(name);
            result.has_resources = true;
        }
    }

    result.resource_files = resource_entries;

    if let Some(content) = manifest_content {
        let manifest: serde_json::Value = parse_manifest_content(&content)
            .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;
        let manifest_info = parse_uniapp_manifest(
            &manifest,
            std::path::Path::new("manifest.json"),
            std::path::Path::new("."),
            None,
        );
        result.app_name = manifest_info.app_name;
        result.app_id = manifest_info.app_id;
        result.version_name = manifest_info.version_name;
        result.version_code = manifest_info.version_code;
        result.package_names = manifest_info.package_names;
        result.detected_modules = manifest_info.detected_modules;
    }

    if let Some(ref props) = props_content {
        for module in &mut result.detected_modules {
            module.configured = check_module_configured_in_props(&module.name, props);
        }
    }

    Ok(result)
}
