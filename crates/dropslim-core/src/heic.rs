#[cfg(not(target_os = "macos"))]
use std::path::Path;

use super::payloads::ErrorPayload;

#[cfg(target_os = "macos")]
mod platform {
    use std::path::Path;
    use std::slice;

    use image::DynamicImage;
    use objc2_core_foundation::{CFData, CFDictionary, CFMutableData, CFNumber, CFString, CFURL};
    use objc2_foundation::{NSString, NSURL};
    use objc2_image_io::{
        kCGImageDestinationLossyCompressionQuality, CGImageDestination, CGImageSource,
    };

    use super::ErrorPayload;

    const HEIC_COMPRESSION_QUALITY: f32 = 0.85;

    fn open_heic_source(
        input: &Path,
    ) -> Result<objc2_core_foundation::CFRetained<CGImageSource>, ErrorPayload> {
        let input_path =
            NSString::from_str(input.to_str().ok_or_else(ErrorPayload::heic_invalid_path)?);
        let input_url = NSURL::fileURLWithPath(&input_path);
        let input_cf_url: &CFURL = input_url.as_ref();

        unsafe { CGImageSource::with_url(input_cf_url, None) }
            .ok_or_else(ErrorPayload::heic_read_failed)
    }

    pub fn heic_is_animated(input: &Path) -> Result<bool, ErrorPayload> {
        let source = open_heic_source(input)?;
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
        let source = open_heic_source(input)?;

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

    /// Decode HEIC/HEIF via ImageIO into a `DynamicImage` (macOS only).
    /// Uses an in-memory PNG round-trip so we can reuse the existing raster encoders.
    pub fn decode_heic(input: &Path) -> Result<DynamicImage, ErrorPayload> {
        let source = open_heic_source(input)?;

        let frame_count = unsafe { source.count() };
        if frame_count == 0 {
            return Err(ErrorPayload::heic_no_frames());
        }

        let png_data = CFMutableData::new(None, 0).ok_or_else(ErrorPayload::heic_create_failed)?;
        let png_type = CFString::from_str("public.png");
        let destination = unsafe { CGImageDestination::with_data(&png_data, &png_type, 1, None) }
            .ok_or_else(ErrorPayload::heic_create_failed)?;

        unsafe {
            destination.add_image_from_source(&source, 0, None);
            if !destination.finalize() {
                return Err(ErrorPayload::heic_write_failed());
            }
        }

        let cf_data: &CFData = png_data.as_ref();
        let len = cf_data.length() as usize;
        if len == 0 {
            return Err(ErrorPayload::heic_read_failed());
        }

        let bytes = unsafe { slice::from_raw_parts(cf_data.byte_ptr(), len) };
        image::load_from_memory(bytes).map_err(|_| ErrorPayload::heic_read_failed())
    }
}

#[cfg(target_os = "macos")]
pub use platform::{decode_heic, heic_is_animated, optimize_heic};

#[cfg(not(target_os = "macos"))]
pub fn heic_is_animated(_input: &Path) -> Result<bool, ErrorPayload> {
    Ok(false)
}

#[cfg(not(target_os = "macos"))]
pub fn optimize_heic(_input: &Path, _output: &Path) -> Result<(), ErrorPayload> {
    Err(ErrorPayload::heic_unsupported_platform())
}

#[cfg(not(target_os = "macos"))]
pub fn decode_heic(_input: &Path) -> Result<image::DynamicImage, ErrorPayload> {
    Err(ErrorPayload::heic_unsupported_platform())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    use image::GenericImageView;

    fn heic_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test/fixtures/sample.heic")
    }

    #[test]
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

    #[test]
    fn decodes_heic_fixture() {
        let input = heic_fixture();
        assert!(
            input.is_file(),
            "missing test/fixtures/sample.heic — run: cargo run --example write_heic_fixture"
        );

        let img = decode_heic(&input).expect("decode heic");
        assert_eq!(img.dimensions(), (512, 384));
    }
}
