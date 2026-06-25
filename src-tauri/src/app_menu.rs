#[cfg(target_os = "macos")]
pub fn setup_app_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::Emitter;

    let app_handle = app.clone();
    app.on_menu_event(move |app, event| {
        if event.id().as_ref() == "preferences" {
            let _ = app.emit("open-settings", ());
        }
    });

    rebuild_app_menu(&app_handle)
}

#[cfg(target_os = "macos")]
pub fn rebuild_app_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    let strings = crate::native_ui::load_strings(app)
        .unwrap_or_else(|_| crate::native_ui::NativeUiStrings::english_defaults());

    build_app_menu(app, &strings)
}

#[cfg(target_os = "macos")]
fn build_app_menu(
    app: &tauri::AppHandle,
    strings: &crate::native_ui::NativeUiStrings,
) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

    let about = PredefinedMenuItem::about(app, None, None)?;
    let preferences = MenuItem::with_id(
        app,
        "preferences",
        &strings.preferences,
        true,
        Some("Cmd+,"),
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = PredefinedMenuItem::quit(app, None)?;
    let app_name = app.package_info().name.clone();

    let app_submenu = Submenu::with_items(
        app,
        &app_name,
        true,
        &[
            &about,
            &separator,
            &preferences,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let maximize = PredefinedMenuItem::maximize(app, None)?;
    let front = PredefinedMenuItem::bring_all_to_front(app, None)?;

    let window_submenu = Submenu::with_items(
        app,
        &strings.window,
        true,
        &[
            &minimize,
            &maximize,
            &PredefinedMenuItem::separator(app)?,
            &front,
        ],
    )?;

    let menu = Menu::with_items(app, &[&app_submenu, &window_submenu])?;
    app.set_menu(menu)?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn setup_app_menu(_app: &tauri::AppHandle) -> tauri::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn rebuild_app_menu(_app: &tauri::AppHandle) -> tauri::Result<()> {
    Ok(())
}
