// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod keystore;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            keystore::store_key,
            keystore::get_key,
            keystore::delete_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
