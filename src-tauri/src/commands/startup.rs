use crate::startup_paths;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct StartupState {
    pub paths: Mutex<Vec<String>>,
}

pub fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn emit_startup_paths(app: &AppHandle, paths: Vec<String>) {
    let paths = startup_paths::filter_paths(paths);

    if paths.is_empty() {
        return;
    }

    let _ = app.emit("startup-paths", paths);
    focus_main_window(app);
}

#[tauri::command]
pub fn consume_startup_paths(state: State<StartupState>) -> Vec<String> {
    std::mem::take(&mut *state.paths.lock().expect("startup paths lock poisoned"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_startup_paths_drains_state() {
        let state = StartupState {
            paths: Mutex::new(vec!["/tmp/a.png".to_string(), "/tmp/b.jpg".to_string()]),
        };

        let first = consume_startup_paths_inner(&state);
        assert_eq!(first, vec!["/tmp/a.png".to_string(), "/tmp/b.jpg".to_string()]);

        let second = consume_startup_paths_inner(&state);
        assert!(second.is_empty());
    }

    fn consume_startup_paths_inner(state: &StartupState) -> Vec<String> {
        std::mem::take(&mut *state.paths.lock().expect("startup paths lock poisoned"))
    }
}
