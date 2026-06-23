mod commands;
mod utils;

use commands::{
    android, harmony, ios,
    shared::{build_history, files, project, sdk, settings},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::compat::apply_runtime_profile();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(android::AppState {})
        .invoke_handler(tauri::generate_handler![
            project::create_project,
            project::get_project,
            project::update_project,
            project::delete_project,
            project::list_projects,
            project::save_project_config,
            project::save_signing_secret,
            project::delete_signing_secret,
            project::get_signing_secret_status,
            crate::commands::shared::resource::import_resource,
            crate::commands::shared::resource::import_resources_batch,
            crate::commands::shared::resource::get_resource_list,
            crate::commands::shared::resource::remove_resource,
            crate::commands::shared::resource::analyze_uploaded_zip,
            crate::commands::shared::resource_scan::import_uniapp_resource,
            crate::commands::shared::resource::read_uniapp_manifest,
            sdk::list_sdks,
            sdk::install_sdk,
            sdk::uninstall_sdk,
            sdk::get_sdk_info,
            sdk::get_global_sdk_config,
            sdk::add_sdk_path,
            sdk::remove_sdk_path,
            android::commands::prepare_android_build,
            android::commands::run_android_build,
            android::commands::build_android_apk,
            android::commands::generate_android_project,
            ios::build::generate_ios_project,
            ios::build::build_ios_ipa,
            harmony::build::generate_harmony_project,
            harmony::build::build_harmony_hap,
            crate::commands::shared::env::check_env,
            crate::commands::shared::env::get_full_env_report,
            android::env_check::check_android_env,
            ios::env_check::check_ios_env,
            harmony::env_check::check_harmony_env,
            crate::commands::shared::env_validate::validate_tool_path,
            crate::commands::shared::env::save_env_override,
            crate::commands::shared::env::get_env_overrides,
            files::read_text_file,
            files::append_build_log,
            files::cleanup_build_temporary_files,
            settings::get_app_settings,
            settings::migrate_cache_dir,
            crate::commands::shared::module::parsing::parse_project_modules,
            crate::commands::shared::module::properties::save_module_config,
            crate::commands::shared::module::parsing::get_module_template,
            crate::commands::shared::module::analysis::analyze_android_module_config,
            crate::commands::shared::module::analysis::analyze_ios_module_config,
            crate::commands::shared::module::analysis::analyze_harmony_module_config,
            android::certificate::analyze_android_keystore,
            android::certificate::generate_android_keystore,
            build_history::get_build_history,
            build_history::add_build_record,
            build_history::update_build_record,
            build_history::clear_build_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
