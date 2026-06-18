use std::path::Path;

use crate::commands::ios::build::pbxproj::{register_pbx_linked_files, IosPbxLinkedFile};
use crate::commands::ios::modules::common::ios_manifest_info_module_enabled;

#[derive(Debug, Clone)]
pub(crate) struct IosUiWebviewIntegration {
    pub(crate) linked_count: usize,
}

pub(crate) fn ios_ui_webview_enabled(
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> bool {
    ios_manifest_info_module_enabled(manifest_info, "UIWebview")
}

pub(crate) fn apply_ios_ui_webview_module(
    project_root: &Path,
    project_file: &Path,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<Option<IosUiWebviewIntegration>, String> {
    if !ios_ui_webview_enabled(manifest_info) {
        return Ok(None);
    }

    let linked_files = ios_ui_webview_linked_files();
    validate_ios_ui_webview_local_linked_files(project_root, &linked_files)?;
    let linked_count = register_pbx_linked_files(project_file, &linked_files)?;

    Ok(Some(IosUiWebviewIntegration { linked_count }))
}

fn ios_ui_webview_linked_files() -> Vec<IosPbxLinkedFile> {
    vec![
        IosPbxLinkedFile::local_static("libH5WEUIWebview.a"),
        IosPbxLinkedFile::system_framework("JavaScriptCore.framework"),
        IosPbxLinkedFile::system_framework("Foundation.framework"),
        IosPbxLinkedFile::system_framework("UIKit.framework"),
    ]
}

fn validate_ios_ui_webview_local_linked_files(
    project_root: &Path,
    files: &[IosPbxLinkedFile],
) -> Result<(), String> {
    let libs_dir = ios_sdk_support_dir(project_root)?.join("Libs");
    for file in files.iter().copied().filter(|file| file.is_local()) {
        let candidate = libs_dir.join(file.name);
        if !candidate.exists() {
            return Err(format!(
                "iOS UIWebview 模块缺少 SDK 依赖文件: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn ios_sdk_support_dir(project_root: &Path) -> Result<std::path::PathBuf, String> {
    project_root
        .parent()
        .map(|workspace| workspace.join("SDK"))
        .ok_or_else(|| format!("iOS 工程路径异常: {}", project_root.display()))
}
