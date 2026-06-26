use std::path::Path;

#[cfg(target_os = "macos")]
const APP_EXECUTABLE_PATTERN: &str = ".app/Contents/MacOS/";

pub fn is_startup_path(arg: &str) -> bool {
    if arg.is_empty() || arg.starts_with('-') {
        return false;
    }

    let path = Path::new(arg);

    if !path.is_absolute() {
        return false;
    }

    let resolved = match path.canonicalize() {
        Ok(value) => value,
        Err(_) => return false,
    };

    if is_app_executable_path(&resolved) {
        return false;
    }

    resolved.exists()
}

fn is_app_executable_path(resolved: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        if resolved.to_string_lossy().contains(APP_EXECUTABLE_PATTERN) {
            return true;
        }
    }

    if let Ok(exec_path) = std::env::current_exe() {
        if let Ok(exec_resolved) = exec_path.canonicalize() {
            if exec_resolved == resolved {
                return true;
            }
        }
    }

    false
}

pub fn filter_paths(paths: Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .filter(|path| is_startup_path(path))
        .collect()
}

pub fn parse_startup_args<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    filter_paths(
        args.into_iter()
            .map(|arg| arg.as_ref().to_string())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn accepts_absolute_existing_paths() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("dropslim-startup-{stamp}.png"));
        fs::write(&tmp, b"x").expect("write temp file");

        assert!(is_startup_path(tmp.to_str().expect("utf8")));

        let _ = fs::remove_file(tmp);
    }

    #[test]
    fn rejects_flags_and_relative_paths() {
        assert!(!is_startup_path("--foo"));
        assert!(!is_startup_path("."));
        assert!(!is_startup_path("photo.png"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn rejects_app_executable_paths() {
        assert!(!is_startup_path(
            "/Applications/DropSlim.app/Contents/MacOS/dropslim"
        ));
    }

    #[test]
    fn rejects_current_executable_path() {
        let exec = std::env::current_exe().expect("current exe");
        let exec = exec.to_string_lossy().to_string();

        assert!(!is_startup_path(&exec));
    }

    #[test]
    fn parses_file_paths_from_args() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("dropslim-startup-{stamp}.png"));
        fs::write(&tmp, b"x").expect("write temp file");
        let tmp = tmp.to_string_lossy().to_string();

        let exec = std::env::current_exe()
            .expect("current exe")
            .to_string_lossy()
            .to_string();
        let parsed = parse_startup_args([exec.as_str(), tmp.as_str()]);

        assert_eq!(parsed, vec![tmp]);

        let _ = fs::remove_file(&parsed[0]);
    }
}
