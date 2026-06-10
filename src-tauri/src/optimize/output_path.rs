use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct UserSettings {
    #[serde(default = "default_true")]
    pub folderswitch: bool,
    #[serde(default = "default_true")]
    pub suffix: bool,
    #[serde(default)]
    pub subfolder: bool,
    pub savepath: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}

pub fn custom_save_folder_missing(settings: &UserSettings) -> bool {
    if settings.folderswitch {
        return false;
    }

    match settings.savepath.as_ref() {
        None => true,
        Some(paths) => paths
            .first()
            .map(|path| path.trim().is_empty())
            .unwrap_or(true),
    }
}

pub fn build_output_path(input: &Path, settings: &UserSettings) -> std::io::Result<PathBuf> {
    let mut dir = input
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    if !settings.folderswitch {
        if let Some(savepath) = settings.savepath.as_ref().and_then(|paths| paths.first()) {
            dir = PathBuf::from(savepath);
        }
    }

    if settings.subfolder {
        dir = dir.join("minified");
    }

    std::fs::create_dir_all(&dir)?;

    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    let extension = input.extension().and_then(|value| value.to_str()).unwrap_or("");

    let file_name = if settings.suffix {
        if extension.is_empty() {
            format!("{stem}.min")
        } else {
            format!("{stem}.min.{extension}")
        }
    } else if extension.is_empty() {
        stem.to_string()
    } else {
        format!("{stem}.{extension}")
    };

    Ok(dir.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn adds_min_suffix_by_default() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("photo.png");
        fs::write(&input, b"x").unwrap();

        let output = build_output_path(
            &input,
            &UserSettings {
                folderswitch: true,
                suffix: true,
                subfolder: false,
                savepath: None,
            },
        )
        .unwrap();

        assert_eq!(output, dir.path().join("photo.min.png"));
    }

    #[test]
    fn overwrites_in_place_without_suffix() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("photo.png");
        fs::write(&input, b"x").unwrap();

        let output = build_output_path(
            &input,
            &UserSettings {
                folderswitch: true,
                suffix: false,
                subfolder: false,
                savepath: None,
            },
        )
        .unwrap();

        assert_eq!(output, dir.path().join("photo.png"));
    }

    #[test]
    fn writes_into_minified_subfolder() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("photo.png");
        fs::write(&input, b"x").unwrap();

        let output = build_output_path(
            &input,
            &UserSettings {
                folderswitch: true,
                suffix: true,
                subfolder: true,
                savepath: None,
            },
        )
        .unwrap();

        assert_eq!(output, dir.path().join("minified").join("photo.min.png"));
        assert!(dir.path().join("minified").is_dir());
    }

    #[test]
    fn uses_custom_save_path() {
        let dir = tempdir().unwrap();
        let custom_dir = dir.path().join("exports");
        fs::create_dir_all(&custom_dir).unwrap();
        let input = dir.path().join("nested").join("photo.png");
        fs::create_dir_all(input.parent().unwrap()).unwrap();
        fs::write(&input, b"x").unwrap();

        let output = build_output_path(
            &input,
            &UserSettings {
                folderswitch: false,
                suffix: true,
                subfolder: false,
                savepath: Some(vec![custom_dir.to_string_lossy().to_string()]),
            },
        )
        .unwrap();

        assert_eq!(output, custom_dir.join("photo.min.png"));
    }

    #[test]
    fn empty_savepath_is_missing() {
        assert!(custom_save_folder_missing(&UserSettings {
            folderswitch: false,
            suffix: true,
            subfolder: false,
            savepath: Some(vec![]),
        }));
    }

    #[test]
    fn blank_savepath_is_missing() {
        assert!(custom_save_folder_missing(&UserSettings {
            folderswitch: false,
            suffix: true,
            subfolder: false,
            savepath: Some(vec!["  ".into()]),
        }));
    }
}
