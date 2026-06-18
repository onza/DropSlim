use std::fs;
use std::path::{Path, PathBuf};

use dropslim_lib::optimize::optimize_image_file;
use tempfile::TempDir;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn fixtures_dir() -> PathBuf {
    project_root().join("test").join("fixtures")
}

fn optimize_to_temp(input: &Path, ext: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join(format!("output.min{ext}"));
    let input_size = fs::metadata(input).expect("input metadata").len();

    optimize_image_file(input, &output, &project_root()).expect("optimize");

    let output_size = fs::metadata(&output).expect("output metadata").len();
    assert!(output_size > 0);
    assert!(output_size < input_size, "expected {output:?} to shrink");

    (dir, output)
}

fn create_raster_fixture(dir: &TempDir, name: &str, create: impl FnOnce(&Path)) -> PathBuf {
    let path = dir.path().join(name);
    create(&path);
    path
}

#[test]
fn optimizes_svg_fixture() {
    let input = fixtures_dir().join("bloat.svg");
    let (_dir, output) = optimize_to_temp(&input, ".svg");
    let contents = fs::read_to_string(output).expect("read svg");
    assert!(contents.contains("<svg"));
}

#[test]
fn optimizes_png_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.png", |path| {
        let img = image::RgbaImage::from_pixel(160, 120, image::Rgba([37, 99, 235, 255]));
        img.save(path).expect("save png");
    });
    optimize_to_temp(&input, ".png");
}

#[test]
fn optimizes_jpeg_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.jpg", |path| {
        let img = image::RgbImage::from_pixel(160, 120, image::Rgb([220, 120, 40]));
        img.save(path).expect("save jpg");
    });
    optimize_to_temp(&input, ".jpg");
}

#[test]
fn optimizes_webp_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.webp", |path| {
        let img = image::RgbaImage::from_pixel(160, 120, image::Rgba([16, 185, 129, 255]));
        img.save(path).expect("save webp");
    });
    optimize_to_temp(&input, ".webp");
}

#[test]
fn optimizes_avif_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.avif", |path| {
        let img = image::RgbImage::from_fn(320, 240, |x, y| {
            image::Rgb([(x % 255) as u8, (y % 255) as u8, ((x + y) % 255) as u8])
        });
        img.save(path).expect("save avif");
    });
    optimize_to_temp(&input, ".avif");
}

#[test]
#[cfg(target_os = "macos")]
fn optimizes_heic_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.heic", |path| {
        use image::{ImageBuffer, Rgba};
        use objc2_core_foundation::{CFDictionary, CFNumber, CFString};
        use objc2_foundation::{NSString, NSURL};
        use objc2_image_io::{
            kCGImageDestinationLossyCompressionQuality, CGImageDestination, CGImageSource,
        };

        let png_path = dir.path().join("sample-source.png");
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(512, 384, |x, y| Rgba([x as u8, y as u8, 200, 255]));
        img.save(&png_path).expect("save png");

        let png_url = NSURL::fileURLWithPath(&NSString::from_str(
            png_path.to_str().expect("png path"),
        ));
        let source = unsafe { CGImageSource::with_url(png_url.as_ref(), None) }
            .expect("png image source");

        let heic_url = NSURL::fileURLWithPath(&NSString::from_str(
            path.to_str().expect("heic path"),
        ));
        let destination = unsafe {
            CGImageDestination::with_url(
                heic_url.as_ref(),
                &CFString::from_str("public.heic"),
                1,
                None,
            )
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

        fs::remove_file(png_path).expect("remove png");
    });

    optimize_to_temp(&input, ".heic");
}

#[test]
fn optimizes_gif_fixture() {
    let gifsicle = project_root()
        .join("vendor")
        .join("gifsicle")
        .join(if cfg!(windows) {
            "gifsicle.exe"
        } else {
            "gifsicle"
        });

    if !gifsicle.exists() {
        panic!(
            "gifsicle not found at {} — run npm ci to install vendor binaries",
            gifsicle.display()
        );
    }

    let input = fixtures_dir().join("bloat.gif");
    optimize_to_temp(&input, ".gif");
}

#[test]
fn optimizes_png_in_place() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "in-place.png", |path| {
        let img = image::RgbaImage::from_pixel(160, 120, image::Rgba([37, 99, 235, 255]));
        img.save(path).expect("save png");
    });
    let input_size = fs::metadata(&input).expect("metadata").len();
    optimize_image_file(&input, &input, &project_root()).expect("in-place");
    let output_size = fs::metadata(&input).expect("metadata").len();
    assert!(output_size < input_size);
}
