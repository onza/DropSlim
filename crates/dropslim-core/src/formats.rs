use std::path::Path;

use serde::{Deserialize, Serialize};

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "svg", "jpg", "jpeg", "png", "gif", "webp", "avif", "heic", "heif",
];

pub fn is_supported_extension(ext: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
}

pub fn is_supported_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(is_supported_extension)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    Svg,
    Jpeg,
    Png,
    Gif,
    Webp,
    Avif,
    Heic,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "svg" => Some(Self::Svg),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::Webp),
            "avif" => Some(Self::Avif),
            "heic" | "heif" => Some(Self::Heic),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormatSetting {
    #[default]
    Original,
    Jpeg,
    Png,
    Webp,
    Avif,
}

impl OutputFormatSetting {
    pub fn target_format(self) -> Option<ImageFormat> {
        match self {
            Self::Original => None,
            Self::Jpeg => Some(ImageFormat::Jpeg),
            Self::Png => Some(ImageFormat::Png),
            Self::Webp => Some(ImageFormat::Webp),
            Self::Avif => Some(ImageFormat::Avif),
        }
    }

    pub fn extension(self) -> Option<&'static str> {
        match self {
            Self::Original => None,
            Self::Jpeg => Some("jpg"),
            Self::Png => Some("png"),
            Self::Webp => Some("webp"),
            Self::Avif => Some("avif"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn recognizes_supported_extensions() {
        assert!(is_supported_extension("PNG"));
        assert!(is_supported_extension("heic"));
        assert!(is_supported_extension("HEIF"));
        assert!(!is_supported_extension("txt"));
    }

    #[test]
    fn maps_paths_to_formats() {
        assert_eq!(
            ImageFormat::from_path(&PathBuf::from("photo.JPG")),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(ImageFormat::from_path(&PathBuf::from("notes.txt")), None);
    }

    #[test]
    fn output_format_defaults_to_original() {
        assert_eq!(
            OutputFormatSetting::default(),
            OutputFormatSetting::Original
        );
        assert!(OutputFormatSetting::Original.target_format().is_none());
        assert!(OutputFormatSetting::Original.extension().is_none());
    }

    #[test]
    fn output_format_maps_targets() {
        assert_eq!(
            OutputFormatSetting::Jpeg.target_format(),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(OutputFormatSetting::Jpeg.extension(), Some("jpg"));
        assert_eq!(
            OutputFormatSetting::Png.target_format(),
            Some(ImageFormat::Png)
        );
        assert_eq!(OutputFormatSetting::Png.extension(), Some("png"));
        assert_eq!(
            OutputFormatSetting::Webp.target_format(),
            Some(ImageFormat::Webp)
        );
        assert_eq!(OutputFormatSetting::Webp.extension(), Some("webp"));
        assert_eq!(
            OutputFormatSetting::Avif.target_format(),
            Some(ImageFormat::Avif)
        );
        assert_eq!(OutputFormatSetting::Avif.extension(), Some("avif"));
    }

    #[test]
    fn deserializes_output_format_values() {
        assert_eq!(
            serde_json::from_str::<OutputFormatSetting>("\"original\"").unwrap(),
            OutputFormatSetting::Original
        );
        assert_eq!(
            serde_json::from_str::<OutputFormatSetting>("\"jpeg\"").unwrap(),
            OutputFormatSetting::Jpeg
        );
        assert!(serde_json::from_str::<OutputFormatSetting>("\"gif\"").is_err());
        assert!(serde_json::from_str::<OutputFormatSetting>("\"heic\"").is_err());
    }
}
