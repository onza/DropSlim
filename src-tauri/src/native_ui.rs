use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeUiStrings {
    #[serde(default)]
    pub preferences: String,
    #[serde(default)]
    pub window: String,
    #[serde(default)]
    pub pick_images: String,
    #[serde(default)]
    pub pick_save_folder: String,
}

impl NativeUiStrings {
    pub fn english_defaults() -> Self {
        Self {
            preferences: "Preferences…".into(),
            window: "Window".into(),
            pick_images: "Choose images or folders".into(),
            pick_save_folder: "Choose a save folder".into(),
        }
    }
}

pub struct NativeUiState {
    pub strings: Mutex<NativeUiStrings>,
}

impl Default for NativeUiState {
    fn default() -> Self {
        Self {
            strings: Mutex::new(NativeUiStrings::english_defaults()),
        }
    }
}

pub fn load_strings(app: &AppHandle) -> Result<NativeUiStrings, String> {
    app.try_state::<NativeUiState>()
        .map(|state| {
            state
                .strings
                .lock()
                .map_err(|error| error.to_string())
                .map(|guard| guard.clone())
        })
        .unwrap_or_else(|| Ok(NativeUiStrings::english_defaults()))
}
