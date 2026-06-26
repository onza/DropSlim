#[cfg(not(target_os = "macos"))]
use std::path::Path;

use super::payloads::ErrorPayload;

#[cfg(target_os = "macos")]
mod platform {
    use std::path::Path;

    use objc2_core_foundation::{CFDictionary, CFNumber, CFString, CFURL};
    use objc2_foundation::{NSString, NSURL};
    use objc2_image_io::{
        kCGImageDestinationLossyCompressionQuality, CGImageDestination, CGImageSource,
    };

    use super::ErrorPayload;

    const HEIC_COMPRESSION_QUALITY: f32 = 0.85;

    pub fn heic_is_animated(input: &Path) -> Result<bool, ErrorPayload> {
        let input_path =
            NSString::from_str(input.to_str().ok_or_else(ErrorPayload::heic_invalid_path)?);
        let input_url = NSURL::fileURLWithPath(&input_path);
        let input_cf_url: &CFURL = input_url.as_ref();

        let source = unsafe { CGImageSource::with_url(input_cf_url, None) }
            .ok_or_else(ErrorPayload::heic_read_failed)?;

        Ok(unsafe { source.count() } > 1)
    }

    fn output_type_identifier(output: &Path) -> objc2_core_foundation::CFRetained<CFString> {
        match output
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("heif") => CFString::from_str("public.heif"),
            _ => CFString::from_str("public.heic"),
        }
    }

    pub fn optimize_heic(input: &Path, output: &Path) -> Result<(), ErrorPayload> {
        let input_path =
            NSString::from_str(input.to_str().ok_or_else(ErrorPayload::heic_invalid_path)?);
        let input_url = NSURL::fileURLWithPath(&input_path);
        let input_cf_url: &CFURL = input_url.as_ref();

        let source = unsafe { CGImageSource::with_url(input_cf_url, None) }
            .ok_or_else(ErrorPayload::heic_read_failed)?;

        let frame_count = unsafe { source.count() };
        if frame_count == 0 {
            return Err(ErrorPayload::heic_no_frames());
        }

        let output_path = NSString::from_str(
            output
                .to_str()
                .ok_or_else(ErrorPayload::heic_invalid_path)?,
        );
        let output_url = NSURL::fileURLWithPath(&output_path);
        let output_cf_url: &CFURL = output_url.as_ref();
        let output_type = output_type_identifier(output);

        let destination =
            unsafe { CGImageDestination::with_url(output_cf_url, &output_type, 1, None) }
                .ok_or_else(ErrorPayload::heic_create_failed)?;

        let quality = CFNumber::new_f32(HEIC_COMPRESSION_QUALITY);

        unsafe {
            let quality_key = kCGImageDestinationLossyCompressionQuality;
            let properties =
                CFDictionary::<CFString, CFNumber>::from_slices(&[quality_key], &[&*quality]);

            destination.add_image_from_source(&source, 0, Some(properties.as_ref()));

            if !destination.finalize() {
                return Err(ErrorPayload::heic_write_failed());
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub use platform::{heic_is_animated, optimize_heic};

#[cfg(not(target_os = "macos"))]
pub fn heic_is_animated(_input: &Path) -> Result<bool, ErrorPayload> {
    Ok(false)
}

#[cfg(not(target_os = "macos"))]
pub fn optimize_heic(_input: &Path, _output: &Path) -> Result<(), ErrorPayload> {
    Err(ErrorPayload::heic_unsupported_platform())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn heic_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test/fixtures/sample.heic")
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn optimizes_heic_fixture() {
        let input = heic_fixture();
        assert!(
            input.is_file(),
            "missing test/fixtures/sample.heic — run: cargo run --example write_heic_fixture"
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let output = dir.path().join("photo.min.heic");

        optimize_heic(&input, &output).expect("optimize heic");

        let output_size = fs::metadata(&output).expect("output metadata").len();
        assert!(output_size > 0);
    }
}
