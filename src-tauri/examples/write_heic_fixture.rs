//! One-off helper: `cargo run --example write_heic_fixture`
//! Writes `test/fixtures/sample.heic` for CI-stable HEIC tests.

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("write_heic_fixture requires macOS");
    std::process::exit(1);
}

#[cfg(target_os = "macos")]
fn main() {
    use std::path::PathBuf;

    use image::{ImageBuffer, Rgba};
    use objc2_core_foundation::{CFDictionary, CFNumber, CFString};
    use objc2_foundation::{NSString, NSURL};
    use objc2_image_io::{
        kCGImageDestinationLossyCompressionQuality, CGImageDestination, CGImageSource,
    };

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixture = root.join("test").join("fixtures").join("sample.heic");
    let png_path = root
        .join("test")
        .join("fixtures")
        .join(".sample-source.png");

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(512, 384, |x, y| Rgba([x as u8, y as u8, 200, 255]));
    img.save(&png_path).expect("write png");

    let png_url = NSURL::fileURLWithPath(&NSString::from_str(png_path.to_str().expect("png path")));
    let source =
        unsafe { CGImageSource::with_url(png_url.as_ref(), None) }.expect("png image source");

    let heic_url =
        NSURL::fileURLWithPath(&NSString::from_str(fixture.to_str().expect("heic path")));
    let heic_type = CFString::from_str("public.heic");
    let destination =
        unsafe { CGImageDestination::with_url(heic_url.as_ref(), &heic_type, 1, None) }
            .expect("heic destination");

    let quality = CFNumber::new_f32(1.0);

    unsafe {
        let quality_key = &*kCGImageDestinationLossyCompressionQuality;
        let properties =
            CFDictionary::<CFString, CFNumber>::from_slices(&[quality_key], &[&*quality]);

        destination.add_image_from_source(&source, 0, Some(properties.as_ref()));
        assert!(destination.finalize(), "HEIC encode failed");
    }

    std::fs::remove_file(png_path).ok();
    println!("write_heic_fixture: ok ({})", fixture.display());
}
