mod commands;
mod utils;

use commands::{
    build_android, build_harmony, build_history, build_ios, certificate, env, files, module,
    project, resource, sdk, settings,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(build_android::AppState {})
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
            resource::import_resource,
            resource::import_resources_batch,
            resource::get_resource_list,
            resource::remove_resource,
            resource::analyze_uploaded_zip,
            resource::import_uniapp_resource,
            resource::read_uniapp_manifest,
            sdk::list_sdks,
            sdk::install_sdk,
            sdk::uninstall_sdk,
            sdk::get_sdk_info,
            sdk::get_global_sdk_config,
            sdk::add_sdk_path,
            sdk::remove_sdk_path,
            build_android::prepare_android_build,
            build_android::run_android_build,
            build_android::build_android_apk,
            build_ios::prepare_ios_build,
            build_ios::run_ios_build,
            build_ios::build_ios_ipa,
            build_harmony::prepare_harmony_build,
            build_harmony::run_harmony_build,
            build_harmony::build_harmony_hap,
            env::check_env,
            env::get_full_env_report,
            env::check_android_env,
            env::check_ios_env,
            env::check_harmony_env,
            env::validate_tool_path,
            env::save_env_override,
            env::get_env_overrides,
            files::read_text_file,
            files::append_build_log,
            files::cleanup_build_temporary_files,
            settings::get_app_settings,
            settings::migrate_cache_dir,
            module::parse_project_modules,
            module::save_module_config,
            module::get_module_template,
            module::analyze_android_module_config,
            certificate::analyze_android_keystore,
            certificate::generate_android_keystore,
            certificate::list_ios_certificates,
            certificate::list_ios_provisioning_profiles,
            build_history::get_build_history,
            build_history::add_build_record,
            build_history::update_build_record,
            build_history::clear_build_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
