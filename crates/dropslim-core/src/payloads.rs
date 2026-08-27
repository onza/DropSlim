use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SummaryPayload {
    AlreadyOptimized {
        size: String,
    },
    Saved {
        percent: u64,
        from: String,
        to: String,
    },
    SavedMore {
        percent: u64,
        from: String,
        to: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummaryPayload {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub bytes_before: u64,
    pub bytes_after: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ErrorPayload {
    pub fn file_not_found() -> Self {
        Self::code("fileNotFound")
    }

    pub fn no_supported_images() -> Self {
        Self::code("noSupportedImages")
    }

    pub fn too_many_images(count: u32, max: u32) -> Self {
        Self {
            code: "tooManyImages".into(),
            count: Some(count),
            max: Some(max),
            detail: None,
        }
    }

    pub fn save_folder_required() -> Self {
        Self::code("saveFolderRequired")
    }

    pub fn optimization_in_progress() -> Self {
        Self::code("optimizationInProgress")
    }

    pub fn unsupported_format() -> Self {
        Self::code("unsupportedFormat")
    }

    pub fn gif_optimizer_unavailable() -> Self {
        Self::code("gifOptimizerUnavailable")
    }

    pub fn gifsicle_failed(code: impl Into<String>) -> Self {
        Self {
            code: "gifsicleFailed".into(),
            count: None,
            max: None,
            detail: Some(code.into()),
        }
    }

    pub fn heic_invalid_path() -> Self {
        Self::code("heicInvalidPath")
    }

    pub fn heic_read_failed() -> Self {
        Self::code("heicReadFailed")
    }

    pub fn heic_no_frames() -> Self {
        Self::code("heicNoFrames")
    }

    pub fn heic_create_failed() -> Self {
        Self::code("heicCreateFailed")
    }

    pub fn heic_write_failed() -> Self {
        Self::code("heicWriteFailed")
    }

    pub fn heic_unsupported_platform() -> Self {
        Self::code("heicUnsupportedPlatform")
    }

    pub fn animated_not_supported() -> Self {
        Self::code("animatedNotSupported")
    }

    pub fn io(detail: impl Into<String>) -> Self {
        Self {
            code: "io".into(),
            count: None,
            max: None,
            detail: Some(detail.into()),
        }
    }

    fn code(code: &str) -> Self {
        Self {
            code: code.into(),
            count: None,
            max: None,
            detail: None,
        }
    }

    pub fn from_message(message: &str) -> Self {
        if message.starts_with("gifsicle exited with ") {
            Self::gifsicle_failed(message.trim_start_matches("gifsicle exited with "))
        } else {
            Self::io(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_gifsicle_exit_code() {
        assert_eq!(
            ErrorPayload::from_message("gifsicle exited with 1"),
            ErrorPayload::gifsicle_failed("1")
        );
    }

    #[test]
    fn falls_back_to_io_for_unknown_messages() {
        assert_eq!(
            ErrorPayload::from_message("disk full"),
            ErrorPayload::io("disk full")
        );
    }
}
