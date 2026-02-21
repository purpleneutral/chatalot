// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod keystore;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    // Accept self-signed TLS certificates (self-hosted servers)
                    #[cfg(target_os = "linux")]
                    {
                        let _ = window.with_webview(|webview| {
                            use webkit2gtk::{WebViewExt, WebsiteDataManagerExt, TLSErrorsPolicy};
                            if let Some(dm) = webview.inner().website_data_manager() {
                                dm.set_tls_errors_policy(TLSErrorsPolicy::Ignore);
                            }
                        });
                    }
                    // Open devtools in debug builds
                    #[cfg(debug_assertions)]
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            keystore::store_key,
            keystore::get_key,
            keystore::delete_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
