#[cfg(not(target_os = "macos"))]
use std::path::Path;

#[cfg(target_os = "macos")]
mod platform {
    use std::path::Path;

    use objc2_core_foundation::{CFDictionary, CFNumber, CFString, CFURL};
    use objc2_foundation::{NSString, NSURL};
    use objc2_image_io::{
        kCGImageDestinationLossyCompressionQuality, CGImageDestination, CGImageSource,
    };

    const HEIC_COMPRESSION_QUALITY: f32 = 0.85;

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

    pub fn optimize_heic(input: &Path, output: &Path) -> Result<(), String> {
        let input_path = NSString::from_str(
            input
                .to_str()
                .ok_or_else(|| "Invalid file path.".to_string())?,
        );
        let input_url = NSURL::fileURLWithPath(&input_path);
        let input_cf_url: &CFURL = input_url.as_ref();

        let source = unsafe { CGImageSource::with_url(input_cf_url, None) }
            .ok_or_else(|| "Could not read HEIC image.".to_string())?;

        let frame_count = unsafe { source.count() };
        if frame_count == 0 {
            return Err("HEIC image contains no frames.".to_string());
        }

        let output_path = NSString::from_str(
            output
                .to_str()
                .ok_or_else(|| "Invalid file path.".to_string())?,
        );
        let output_url = NSURL::fileURLWithPath(&output_path);
        let output_cf_url: &CFURL = output_url.as_ref();
        let output_type = output_type_identifier(output);

        let destination = unsafe {
            CGImageDestination::with_url(output_cf_url, &output_type, 1, None)
        }
        .ok_or_else(|| "Could not create HEIC output.".to_string())?;

        let quality = CFNumber::new_f32(HEIC_COMPRESSION_QUALITY);

        unsafe {
            let quality_key = &*kCGImageDestinationLossyCompressionQuality;
            let properties = CFDictionary::<CFString, CFNumber>::from_slices(
                &[quality_key],
                &[&*quality],
            );

            destination.add_image_from_source(&source, 0, Some(properties.as_ref()));

            if !destination.finalize() {
                return Err("Could not write optimized HEIC.".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub use platform::optimize_heic;

#[cfg(not(target_os = "macos"))]
pub fn optimize_heic(_input: &Path, _output: &Path) -> Result<(), String> {
    Err("HEIC is only supported on macOS.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[cfg(target_os = "macos")]
    fn write_heic_fixture(path: &Path) {
        use image::{ImageBuffer, Rgba};
        use objc2_core_foundation::{CFDictionary, CFNumber, CFString};
        use objc2_foundation::{NSString, NSURL};
        use objc2_image_io::{
            kCGImageDestinationLossyCompressionQuality, CGImageDestination, CGImageSource,
        };

        let dir = path.parent().expect("fixture parent");
        let png_path = dir.join("heic-fixture-source.png");
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(512, 384, |x, y| Rgba([x as u8, y as u8, 128, 255]));
        img.save(&png_path).expect("write png fixture");

        let png_url = NSURL::fileURLWithPath(&NSString::from_str(
            png_path.to_str().expect("png path"),
        ));
        let source = unsafe { CGImageSource::with_url(png_url.as_ref(), None) }
            .expect("png image source");

        let heic_url = NSURL::fileURLWithPath(&NSString::from_str(
            path.to_str().expect("heic path"),
        ));
        let heic_type = CFString::from_str("public.heic");
        let destination = unsafe {
            CGImageDestination::with_url(heic_url.as_ref(), &heic_type, 1, None)
        }
        .expect("heic destination");

        let quality = CFNumber::new_f32(1.0);

        unsafe {
            let quality_key = &*kCGImageDestinationLossyCompressionQuality;
            let properties = CFDictionary::<CFString, CFNumber>::from_slices(
                &[quality_key],
                &[&*quality],
            );

            destination.add_image_from_source(&source, 0, Some(properties.as_ref()));
            assert!(destination.finalize());
        }

        let _ = fs::remove_file(png_path);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn optimizes_heic_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = dir.path().join("photo.heic");
        write_heic_fixture(&input);

        let output = dir.path().join("photo.min.heic");

        optimize_heic(&input, &output).expect("optimize heic");

        let output_size = fs::metadata(&output).expect("output metadata").len();
        assert!(output_size > 0);
    }
}
