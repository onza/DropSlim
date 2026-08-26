use std::path::Path;

use dropslim_core::{OutputFormatSetting, UserSettings};

pub fn settings_from_flags(
    no_suffix: bool,
    subfolder: bool,
    out: Option<&Path>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    format: OutputFormatSetting,
) -> UserSettings {
    let limit_dimensions = max_width.is_some() || max_height.is_some();
    let (folderswitch, savepath) = match out {
        Some(dir) => (false, Some(vec![dir.to_string_lossy().into_owned()])),
        None => (true, None),
    };

    UserSettings {
        folderswitch,
        suffix: !no_suffix,
        subfolder,
        savepath,
        limit_dimensions,
        max_width,
        max_height,
        output_format: format,
    }
}

pub fn validate_out_dir(out: Option<&Path>) -> Result<(), String> {
    let Some(dir) = out else {
        return Ok(());
    };
    if dir.as_os_str().is_empty() {
        return Err("output directory must not be empty".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_writes_beside_source_with_suffix() {
        let settings = settings_from_flags(
            false,
            false,
            None,
            None,
            None,
            OutputFormatSetting::Original,
        );
        assert!(settings.folderswitch);
        assert!(settings.suffix);
        assert!(!settings.subfolder);
        assert!(settings.savepath.is_none());
        assert!(!settings.limit_dimensions);
    }

    #[test]
    fn out_disables_folderswitch() {
        let out = PathBuf::from("/tmp/out");
        let settings = settings_from_flags(
            true,
            true,
            Some(&out),
            Some(800),
            None,
            OutputFormatSetting::Webp,
        );
        assert!(!settings.folderswitch);
        assert!(!settings.suffix);
        assert!(settings.subfolder);
        assert_eq!(settings.savepath.as_deref(), Some(&["/tmp/out".to_string()][..]));
        assert!(settings.limit_dimensions);
        assert_eq!(settings.max_width, Some(800));
        assert_eq!(settings.output_format, OutputFormatSetting::Webp);
    }
}
