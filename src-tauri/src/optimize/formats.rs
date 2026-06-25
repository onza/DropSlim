use std::path::Path;

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
}
