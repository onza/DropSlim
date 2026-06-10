use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn project_root(app: &AppHandle) -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
    }

    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?;
    let bundled = resource_dir.join("resources");

    if bundled.join("vendor").exists() {
        Ok(bundled)
    } else {
        Ok(resource_dir)
    }
}

pub const QUICK_ACTION_WORKFLOW: &str = "Optimize with DropSlim.workflow";

pub fn quick_action_source(app: &AppHandle) -> Result<PathBuf, String> {
    let root = project_root(app)?;
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?;

    let candidates = [
        resource_dir
            .join("resources")
            .join("build")
            .join(QUICK_ACTION_WORKFLOW),
        resource_dir.join("build").join(QUICK_ACTION_WORKFLOW),
        root.join("build").join(QUICK_ACTION_WORKFLOW),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("build")
            .join(QUICK_ACTION_WORKFLOW),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("Quick Action workflow is not bundled with this build.".to_string())
}
