use std::path::PathBuf;

/// Locate vendor/gifsicle for the running binary.
///
/// Checks the executable directory, then walks up a few levels (cargo target/),
/// then falls back to the current working directory.
pub fn resolve_project_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        for dir in exe.ancestors().take(5) {
            if dir.join("vendor").join("gifsicle").is_dir() {
                return dir.to_path_buf();
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        for dir in cwd.ancestors().take(5) {
            if dir.join("vendor").join("gifsicle").is_dir() {
                return dir.to_path_buf();
            }
        }
        return cwd;
    }

    PathBuf::from(".")
}
