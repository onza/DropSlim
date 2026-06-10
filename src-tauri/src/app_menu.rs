#[cfg(target_os = "macos")]
pub fn setup_app_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    use crate::commands::install_quick_action::install_quick_action_with_dialog;
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
    use tauri::Emitter;

    let about = PredefinedMenuItem::about(app, None, None)?;
    let preferences =
        MenuItem::with_id(app, "preferences", "Preferences…", true, Some("Cmd+,"))?;
    let install_quick_action = MenuItem::with_id(
        app,
        "install_quick_action",
        "Install Finder Quick Action",
        true,
        None::<&str>,
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
            &install_quick_action,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let maximize = PredefinedMenuItem::maximize(app, None)?;
    let front = PredefinedMenuItem::bring_all_to_front(app, None)?;

    let window_submenu = Submenu::with_items(
        app,
        "Window",
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

    app.on_menu_event(move |app, event| {
        match event.id().as_ref() {
            "preferences" => {
                let _ = app.emit("open-settings", ());
            }
            "install_quick_action" => {
                install_quick_action_with_dialog(app);
            }
            _ => {}
        }
    });

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn setup_app_menu(_app: &tauri::AppHandle) -> tauri::Result<()> {
    Ok(())
}
