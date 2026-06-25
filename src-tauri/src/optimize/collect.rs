use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::formats::is_supported_path;

pub struct CollectResult {
    pub paths: Vec<PathBuf>,
    pub missing: Vec<String>,
    pub unreadable: Vec<String>,
}

fn label_for_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn add_file(
    path: &Path,
    seen: &mut BTreeSet<PathBuf>,
    results: &mut Vec<PathBuf>,
    unreadable: &mut Vec<String>,
) {
    let Ok(resolved) = path.canonicalize() else {
        unreadable.push(label_for_path(path));
        return;
    };

    if !is_supported_path(&resolved) || !seen.insert(resolved.clone()) {
        return;
    }

    results.push(resolved);
}

pub fn collect_image_paths(input_paths: &[String]) -> std::io::Result<CollectResult> {
    let mut seen = BTreeSet::new();
    let mut results = Vec::new();
    let mut missing = Vec::new();
    let mut unreadable = Vec::new();

    for input in input_paths {
        let path = PathBuf::from(input);

        if !path.exists() {
            missing.push(label_for_path(Path::new(input)));
            continue;
        }

        let metadata = std::fs::metadata(&path)?;

        if metadata.is_dir() {
            for entry in WalkDir::new(&path).follow_links(false) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    add_file(entry.path(), &mut seen, &mut results, &mut unreadable);
                }
            }
            continue;
        }

        if metadata.is_file() {
            add_file(&path, &mut seen, &mut results, &mut unreadable);
        }
    }

    Ok(CollectResult {
        paths: results,
        missing,
        unreadable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn collects_single_image() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("photo.png");
        fs::write(&file, b"png").unwrap();

        let collected = collect_image_paths(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(collected.paths.len(), 1);
        assert!(collected.paths[0].ends_with("photo.png"));
        assert!(collected.missing.is_empty());
        assert!(collected.unreadable.is_empty());
    }

    #[test]
    fn reports_missing_paths() {
        let collected = collect_image_paths(&["/no/such/file-or-folder.png".to_string()]).unwrap();
        assert!(collected.paths.is_empty());
        assert_eq!(collected.missing, vec!["file-or-folder.png"]);
        assert!(collected.unreadable.is_empty());
    }

    #[test]
    fn ignores_unsupported_files() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("notes.txt");
        fs::write(&file, b"text").unwrap();

        let collected = collect_image_paths(&[file.to_string_lossy().to_string()]).unwrap();
        assert!(collected.paths.is_empty());
        assert!(collected.missing.is_empty());
        assert!(collected.unreadable.is_empty());
    }

    #[test]
    fn collects_recursively() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join("a.png"), b"png").unwrap();
        fs::write(nested.join("b.jpg"), b"jpg").unwrap();
        fs::write(nested.join("c.webp"), b"webp").unwrap();
        fs::write(nested.join("notes.txt"), b"text").unwrap();

        let collected = collect_image_paths(&[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!(collected.paths.len(), 3);
        assert!(collected.missing.is_empty());
        assert!(collected.unreadable.is_empty());
    }
}
