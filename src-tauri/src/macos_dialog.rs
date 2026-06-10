#[cfg(target_os = "macos")]
pub fn pick_paths() -> Result<Vec<String>, String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};
    use objc2_foundation::NSString;

    let mtm = MainThreadMarker::new().ok_or_else(|| "must run on main thread".to_string())?;
    let panel = NSOpenPanel::openPanel(mtm);

    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(true);
    panel.setCanCreateDirectories(false);
    panel.setTitle(Some(&NSString::from_str("Choose images or folders")));

    let response = panel.runModal();

    if response != NSModalResponseOK {
        return Ok(vec![]);
    }

    let urls = panel.URLs();
    let mut paths = Vec::new();

    for index in 0..urls.len() {
        let url = urls.objectAtIndex(index);

        if let Some(path) = url.path() {
            paths.push(path.to_string());
        }
    }

    Ok(paths)
}

#[cfg(target_os = "macos")]
pub fn pick_save_folder() -> Result<Vec<String>, String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};
    use objc2_foundation::NSString;

    let mtm = MainThreadMarker::new().ok_or_else(|| "must run on main thread".to_string())?;
    let panel = NSOpenPanel::openPanel(mtm);

    panel.setCanChooseFiles(false);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(false);
    panel.setCanCreateDirectories(true);
    panel.setTitle(Some(&NSString::from_str("Choose a save folder")));

    let response = panel.runModal();

    if response != NSModalResponseOK {
        return Ok(vec![]);
    }

    let urls = panel.URLs();

    if urls.len() == 0 {
        return Ok(vec![]);
    }

    let url = urls.objectAtIndex(0);

    if let Some(path) = url.path() {
        Ok(vec![path.to_string()])
    } else {
        Ok(vec![])
    }
}

#[cfg(not(target_os = "macos"))]
pub fn pick_save_folder() -> Result<Vec<String>, String> {
    Err("pick_save_folder is only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn pick_paths() -> Result<Vec<String>, String> {
    Err("pick_paths is only supported on macOS".to_string())
}
