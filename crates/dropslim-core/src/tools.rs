use std::path::{Path, PathBuf};

pub fn gifsicle_binary_name() -> &'static str {
    if cfg!(windows) {
        "gifsicle.exe"
    } else {
        "gifsicle"
    }
}

/// Resolve gifsicle for encode paths.
///
/// Order: `DROPSLIM_GIFSICLE` → `{project_root}/vendor/gifsicle/<bin>` → `PATH`.
pub fn gifsicle_path(project_root: &Path) -> Option<PathBuf> {
    let binary_name = gifsicle_binary_name();

    if let Ok(from_env) = std::env::var("DROPSLIM_GIFSICLE") {
        let path = PathBuf::from(from_env.trim());
        if !path.as_os_str().is_empty() && path.is_file() {
            return Some(path);
        }
    }

    let bundled = project_root
        .join("vendor")
        .join("gifsicle")
        .join(binary_name);
    if bundled.is_file() {
        return Some(bundled);
    }

    find_on_path(binary_name)
}

fn find_on_path(binary_name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn falls_back_to_bundled_vendor() {
        let dir = TempDir::new().expect("tempdir");
        let bundled = dir
            .path()
            .join("vendor")
            .join("gifsicle")
            .join(gifsicle_binary_name());
        fs::create_dir_all(bundled.parent().unwrap()).expect("mkdir");
        fs::write(&bundled, b"bundled").expect("write bundled");

        assert_eq!(
            gifsicle_path(dir.path()).as_deref(),
            Some(bundled.as_path())
        );
    }
}
