#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{pick_paths, pick_save_folder};

#[cfg(not(target_os = "macos"))]
use crate::native_ui::NativeUiStrings;

#[cfg(not(target_os = "macos"))]
pub fn pick_save_folder(_strings: &NativeUiStrings) -> Result<Vec<String>, String> {
    Err("pick_save_folder is only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn pick_paths(_strings: &NativeUiStrings) -> Result<Vec<String>, String> {
    Err("pick_paths is only supported on macOS".to_string())
}
