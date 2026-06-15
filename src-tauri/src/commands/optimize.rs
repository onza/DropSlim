use super::shared::project_root;
use crate::optimize::{process_paths, UserSettings};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

pub struct OptimizationState {
    pub guard: Arc<Mutex<()>>,
    pub cancel: Arc<AtomicBool>,
}

impl Default for OptimizationState {
    fn default() -> Self {
        Self {
            guard: Arc::new(Mutex::new(())),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[tauri::command]
pub async fn optimize_paths(
    app: AppHandle,
    paths: Vec<String>,
    settings: UserSettings,
) -> Result<(), String> {
    let state = app.state::<OptimizationState>();
    let _lock = state
        .guard
        .try_lock()
        .map_err(|_| "An optimization is already in progress.".to_string())?;

    state.cancel.store(false, Ordering::SeqCst);
    let project_root = project_root(&app)?;
    process_paths(
        app.clone(),
        paths,
        settings,
        project_root,
        Arc::clone(&state.cancel),
    )
    .await
}

#[tauri::command]
pub fn cancel_optimization(app: AppHandle) -> Result<(), String> {
    let state = app.state::<OptimizationState>();
    state.cancel.store(true, Ordering::SeqCst);
    Ok(())
}
