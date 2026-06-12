pub mod batch;
pub mod commands;
pub mod converter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::scan_files,
            commands::convert_files,
            commands::open_folder,
            commands::get_app_info,
            commands::open_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
