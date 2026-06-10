use super::shared::{quick_action_source, QUICK_ACTION_WORKFLOW};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[derive(serde::Serialize)]
pub struct InstallQuickActionResult {
    pub ok: bool,
    pub message: String,
}

pub fn show_install_quick_action_dialog(app: &AppHandle, result: &InstallQuickActionResult) {
    app.dialog()
        .message(result.message.clone())
        .title(if result.ok {
            "Finder Quick Action installed"
        } else {
            "Installation failed"
        })
        .kind(if result.ok {
            MessageDialogKind::Info
        } else {
            MessageDialogKind::Error
        })
        .buttons(MessageDialogButtons::Ok)
        .show(|_| {});
}

pub fn install_quick_action_internal(app: &AppHandle) -> InstallQuickActionResult {
    let source = match quick_action_source(app) {
        Ok(source) => source,
        Err(message) => {
            return InstallQuickActionResult { ok: false, message };
        }
    };

    let home = match std::env::var("HOME") {
        Ok(value) => value,
        Err(error) => {
            return InstallQuickActionResult {
                ok: false,
                message: error.to_string(),
            };
        }
    };

    let dest = PathBuf::from(home)
        .join("Library")
        .join("Services")
        .join(QUICK_ACTION_WORKFLOW);

    let Some(parent) = dest.parent() else {
        return InstallQuickActionResult {
            ok: false,
            message: "Invalid Quick Action install path.".to_string(),
        };
    };

    match fs::remove_dir_all(&dest)
        .or_else(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(error)
            }
        })
        .and_then(|_| fs::create_dir_all(parent))
        .and_then(|_| copy_dir_recursive(&source, &dest))
    {
        Ok(()) => InstallQuickActionResult {
            ok: true,
            message: "Finder Quick Action installed.\n\nSelect images or a folder in Finder, then use Quick Actions → Optimize with DropSlim.\n\nIf it does not appear right away, check System Settings → Privacy & Security → Extensions → Finder.".to_string(),
        },
        Err(error) => InstallQuickActionResult {
            ok: false,
            message: error.to_string(),
        },
    }
}

fn copy_dir_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(source_path, target_path)?;
        }
    }

    Ok(())
}

pub fn install_quick_action_with_dialog(app: &AppHandle) -> InstallQuickActionResult {
    let result = install_quick_action_internal(app);
    show_install_quick_action_dialog(app, &result);
    result
}

#[tauri::command]
pub fn install_quick_action(app: AppHandle) -> InstallQuickActionResult {
    install_quick_action_with_dialog(&app)
}
