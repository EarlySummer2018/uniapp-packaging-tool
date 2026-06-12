mod assets;
mod imports;
mod manifest;
mod module_detection;
mod types;
mod zip;

pub use manifest::{parse_uniapp_manifest, read_manifest_file, read_uniapp_manifest_sync};
#[allow(unused_imports)]
pub use types::{
    AndroidIconsConfig, AndroidManifestConfig, DetectedModule, ImportedResource, IosIconsConfig,
    PlatformPackages, PushIconsConfig, ResourceImportInput, ResourceType, SplashscreenConfig,
    UniappManifestInfo, ZipAnalysisResult,
};

#[tauri::command]
pub async fn read_uniapp_manifest(project_path: String) -> Result<UniappManifestInfo, String> {
    read_uniapp_manifest_sync(&project_path)
}

#[tauri::command]
pub async fn import_resource(
    project_path: String,
    resource_type: String,
    source_path: String,
) -> Result<ImportedResource, String> {
    imports::import_resource_impl(project_path, resource_type, source_path).await
}

#[tauri::command]
pub async fn import_resources_batch(
    project_path: String,
    resources: Vec<ResourceImportInput>,
) -> Result<Vec<ImportedResource>, String> {
    imports::import_resources_batch_impl(project_path, resources).await
}

#[tauri::command]
pub async fn get_resource_list(project_path: String) -> Result<Vec<ImportedResource>, String> {
    imports::get_resource_list_impl(project_path).await
}

#[tauri::command]
pub async fn remove_resource(project_path: String, resource_id: String) -> Result<(), String> {
    imports::remove_resource_impl(project_path, resource_id).await
}

#[tauri::command]
pub async fn analyze_uploaded_zip(zip_path: String) -> Result<ZipAnalysisResult, String> {
    zip::analyze_uploaded_zip_impl(zip_path).await
}

#[cfg(test)]
mod tests;
