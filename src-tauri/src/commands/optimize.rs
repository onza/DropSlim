use super::shared::project_root;
use crate::optimize::{app_event_sink, process_paths_with_sink, ErrorPayload, UserSettings};
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
) -> Result<(), ErrorPayload> {
    let state = app.state::<OptimizationState>();
    let _lock = state
        .guard
        .try_lock()
        .map_err(|_| ErrorPayload::optimization_in_progress())?;

    state.cancel.store(false, Ordering::SeqCst);
    let project_root = project_root(&app).map_err(ErrorPayload::io)?;
    process_paths_with_sink(
        app_event_sink(app.clone()),
        paths,
        settings,
        project_root,
        Arc::clone(&state.cancel),
    )
    .await
    .map_err(|error| ErrorPayload::from_message(&error))
}

#[tauri::command]
pub fn cancel_optimization(app: AppHandle) -> Result<(), String> {
    let state = app.state::<OptimizationState>();
    state.cancel.store(true, Ordering::SeqCst);
    Ok(())
}
