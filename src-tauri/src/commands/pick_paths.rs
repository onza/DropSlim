use crate::macos_dialog;
use crate::native_ui::load_strings;
use tauri::AppHandle;

#[tauri::command]
pub async fn pick_paths(app: AppHandle) -> Result<Vec<String>, String> {
    let strings = load_strings(&app)?;
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.run_on_main_thread(move || {
        let _ = tx.send(macos_dialog::pick_paths(&strings));
    })
    .map_err(|error| error.to_string())?;

    rx.await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn pick_save_folder(app: AppHandle) -> Result<Vec<String>, String> {
    let strings = load_strings(&app)?;
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.run_on_main_thread(move || {
        let _ = tx.send(macos_dialog::pick_save_folder(&strings));
    })
    .map_err(|error| error.to_string())?;

    rx.await.map_err(|error| error.to_string())?
}
