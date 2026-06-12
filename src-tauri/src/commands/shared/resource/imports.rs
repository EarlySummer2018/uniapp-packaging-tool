use super::types::{ImportedResource, ResourceImportInput, ResourceType};

pub(super) async fn import_resource_impl(
    _project_path: String,
    resource_type: String,
    source_path: String,
) -> Result<ImportedResource, String> {
    let path = std::path::Path::new(&source_path);
    if !path.exists() {
        return Err(format!("Resource file not found: {}", source_path));
    }

    let metadata = tokio::fs::metadata(&source_path)
        .await
        .map_err(|e| e.to_string())?;

    let res_type = match resource_type.as_str() {
        "image" => ResourceType::Image,
        "font" => ResourceType::Font,
        "audio" => ResourceType::Audio,
        "video" => ResourceType::Video,
        "json" => ResourceType::Json,
        "raw" => ResourceType::Raw,
        _ => ResourceType::Other,
    };

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ImportedResource {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        r#type: res_type,
        source_path,
        size_bytes: metadata.len(),
        imported_at: chrono::Utc::now().to_rfc3339(),
        metadata: serde_json::json!({}),
    })
}

pub(super) async fn import_resources_batch_impl(
    project_path: String,
    resources: Vec<ResourceImportInput>,
) -> Result<Vec<ImportedResource>, String> {
    let mut results = Vec::with_capacity(resources.len());
    for input in resources {
        let result = import_resource_impl(project_path.clone(), input.r#type, input.path).await?;
        results.push(result);
    }
    Ok(results)
}

pub(super) async fn get_resource_list_impl(
    project_path: String,
) -> Result<Vec<ImportedResource>, String> {
    let resources_dir = std::path::Path::new(&project_path).join("resources");
    if !resources_dir.exists() {
        return Ok(Vec::new());
    }
    let mut resources = Vec::new();
    let mut entries = tokio::fs::read_dir(&resources_dir)
        .await
        .map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let meta = entry.metadata().await.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        resources.push(ImportedResource {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            r#type: ResourceType::Raw,
            source_path: entry.path().to_string_lossy().to_string(),
            size_bytes: meta.len(),
            imported_at: chrono::Utc::now().to_rfc3339(),
            metadata: serde_json::json!({}),
        });
    }
    Ok(resources)
}

pub(super) async fn remove_resource_impl(
    _project_path: String,
    _resource_id: String,
) -> Result<(), String> {
    Ok(())
}
