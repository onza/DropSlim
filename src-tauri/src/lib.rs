mod app_menu;
mod commands;
mod native_ui;
pub mod optimize;
mod platform;
mod startup_paths;

use commands::startup::{emit_startup_paths, focus_main_window, StartupState};
use std::sync::Mutex;
use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_process::init())
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
                let paths: Vec<String> = argv
                    .iter()
                    .skip(1)
                    .filter(|arg| startup_paths::is_startup_path(arg))
                    .cloned()
                    .collect();

                if paths.is_empty() {
                    focus_main_window(app);
                } else {
                    emit_startup_paths(app, paths);
                }
            }));
    }

    let app = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::optimize::optimize_paths,
            commands::optimize::cancel_optimization,
            commands::pick_paths::pick_paths,
            commands::pick_paths::pick_save_folder,
            commands::startup::consume_startup_paths,
            commands::native_ui::update_native_ui,
        ])
        .setup(|app| {
            let initial_paths = startup_paths::parse_startup_args(std::env::args().skip(1));

            app.manage(StartupState {
                paths: Mutex::new(initial_paths),
            });
            app.manage(commands::optimize::OptimizationState::default());
            app.manage(native_ui::NativeUiState::default());

            app_menu::setup_app_menu(app.handle())?;

            if let Some(window) = app.get_webview_window("main") {
                #[cfg(debug_assertions)]
                {
                    let _ = window.set_title("DropSlim (Dev)");
                }

                #[cfg(target_os = "macos")]
                {
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

                    let _ = apply_vibrancy(
                        &window,
                        NSVisualEffectMaterial::UnderWindowBackground,
                        None,
                        None,
                    );
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let RunEvent::Opened { urls } = event {
            let paths: Vec<String> = urls
                .iter()
                .filter_map(|url| url.to_file_path().ok())
                .map(|path| path.to_string_lossy().to_string())
                .filter(|path| startup_paths::is_startup_path(path))
                .collect();

            if !paths.is_empty() {
                emit_startup_paths(app_handle, paths);
            }
        }
    });
}
