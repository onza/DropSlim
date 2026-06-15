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
