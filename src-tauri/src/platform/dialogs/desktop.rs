use crate::native_ui::NativeUiStrings;
use crate::optimize::formats::SUPPORTED_EXTENSIONS;
use rfd::FileDialog;

pub fn pick_paths(strings: &NativeUiStrings) -> Result<Vec<String>, String> {
    Ok(FileDialog::new()
        .set_title(&strings.pick_images)
        .add_filter("Images", SUPPORTED_EXTENSIONS)
        .pick_files()
        .unwrap_or_default()
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect())
}

pub fn pick_save_folder(strings: &NativeUiStrings) -> Result<Vec<String>, String> {
    Ok(FileDialog::new()
        .set_title(&strings.pick_save_folder)
        .pick_folder()
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect())
}
