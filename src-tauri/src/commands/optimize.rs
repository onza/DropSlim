use super::shared::project_root;
use crate::optimize::{process_paths, UserSettings};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

pub struct OptimizationGuard(Arc<Mutex<()>>);

impl Default for OptimizationGuard {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(())))
    }
}

#[tauri::command]
pub async fn optimize_paths(
    app: AppHandle,
    paths: Vec<String>,
    settings: UserSettings,
) -> Result<(), String> {
    let guard = app.state::<OptimizationGuard>();
    let _lock = guard
        .0
        .try_lock()
        .map_err(|_| "An optimization is already in progress.".to_string())?;

    let project_root = project_root(&app)?;
    process_paths(app.clone(), paths, settings, project_root).await
}
