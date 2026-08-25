use std::path::{Path, PathBuf};

pub fn gifsicle_path(project_root: &Path) -> Option<PathBuf> {
    let binary_name = if cfg!(windows) {
        "gifsicle.exe"
    } else {
        "gifsicle"
    };
    let candidate = project_root
        .join("vendor")
        .join("gifsicle")
        .join(binary_name);

    candidate.exists().then_some(candidate)
}
