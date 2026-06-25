use std::path::{Path, PathBuf};

pub fn dropslim_temp_path(output: &Path) -> PathBuf {
    let extension = output
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();
    let stem = output
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");

    output.with_file_name(format!("{stem}.dropslim{extension}"))
}

pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn at(output: &Path) -> Self {
        Self {
            path: dropslim_temp_path(output),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_dropslim_temp_path() {
        let output = PathBuf::from("/tmp/photo.min.png");
        assert_eq!(
            dropslim_temp_path(&output),
            PathBuf::from("/tmp/photo.min.dropslim.png")
        );
    }
}
