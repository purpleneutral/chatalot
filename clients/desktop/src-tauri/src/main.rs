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
                    #[cfg(target_os = "linux")]
                    {
                        let _ = window.with_webview(|webview| {
                            use webkit2gtk::{
                                WebViewExt, WebsiteDataManagerExt, SettingsExt,
                                TLSErrorsPolicy, PermissionRequestExt,
                                UserMediaPermissionRequest, DeviceInfoPermissionRequest,
                            };
                            use webkit2gtk::glib::Cast;

                            let wv = webview.inner();

                            // Accept self-signed TLS certificates (self-hosted servers)
                            if let Some(dm) = wv.website_data_manager() {
                                dm.set_tls_errors_policy(TLSErrorsPolicy::Ignore);
                            }

                            // Enable WebRTC and media stream (not enabled by wry by default)
                            if let Some(settings) = wv.settings() {
                                settings.set_enable_media_stream(true);
                                settings.set_enable_media(true);
                                settings.set_enable_media_capabilities(true);
                                settings.set_enable_mediasource(true);
                                settings.set_media_playback_requires_user_gesture(false);
                            }

                            // Auto-grant microphone/camera/device-enumeration permissions
                            // (WebKitGTK has no permission prompt UI, so these are denied by default)
                            wv.connect_permission_request(|_wv, request| {
                                if request.downcast_ref::<UserMediaPermissionRequest>().is_some()
                                    || request.downcast_ref::<DeviceInfoPermissionRequest>().is_some()
                                {
                                    request.allow();
                                    return true;
                                }
                                false
                            });
                        });
                    }
                    // TODO: remove after debugging voice calls
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
