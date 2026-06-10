use std::path::{Path, PathBuf};

pub fn gifsicle_path(project_root: &Path) -> Option<PathBuf> {
    let candidate = project_root
        .join("vendor")
        .join("gifsicle")
        .join("gifsicle");

    candidate.exists().then_some(candidate)
}
