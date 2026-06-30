use crate::native_ui::NativeUiStrings;

struct OpenPanelOptions {
    title: String,
    choose_files: bool,
    choose_directories: bool,
    multiple: bool,
    create_directories: bool,
}

fn run_open_panel(options: OpenPanelOptions) -> Result<Vec<String>, String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};
    use objc2_foundation::NSString;

    let mtm = MainThreadMarker::new().ok_or_else(|| "must run on main thread".to_string())?;
    let panel = NSOpenPanel::openPanel(mtm);

    panel.setCanChooseFiles(options.choose_files);
    panel.setCanChooseDirectories(options.choose_directories);
    panel.setAllowsMultipleSelection(options.multiple);
    panel.setCanCreateDirectories(options.create_directories);
    panel.setTitle(Some(&NSString::from_str(&options.title)));

    if panel.runModal() != NSModalResponseOK {
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

pub fn pick_paths(strings: &NativeUiStrings) -> Result<Vec<String>, String> {
    run_open_panel(OpenPanelOptions {
        title: strings.pick_images.clone(),
        choose_files: true,
        choose_directories: true,
        multiple: true,
        create_directories: false,
    })
}

pub fn pick_save_folder(strings: &NativeUiStrings) -> Result<Vec<String>, String> {
    let mut paths = run_open_panel(OpenPanelOptions {
        title: strings.pick_save_folder.clone(),
        choose_files: false,
        choose_directories: true,
        multiple: false,
        create_directories: true,
    })?;

    paths.truncate(1);
    Ok(paths)
}
