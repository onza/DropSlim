use crate::native_ui::NativeUiStrings;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn update_native_ui(app: AppHandle, strings: NativeUiStrings) -> Result<(), String> {
    let state = app.state::<crate::native_ui::NativeUiState>();
    *state.strings.lock().map_err(|error| error.to_string())? = strings;
    crate::app_menu::rebuild_app_menu(&app).map_err(|error| error.to_string())?;
    Ok(())
}
