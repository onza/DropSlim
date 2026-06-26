use std::fs;
use std::path::{Path, PathBuf};

use dropslim_lib::optimize::{optimize_image_file, ErrorPayload};
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
fn optimizes_svg_with_doctype() {
    let input = fixtures_dir().join("dtd.svg");
    let (_dir, output) = optimize_to_temp(&input, ".svg");
    let contents = fs::read_to_string(output).expect("read svg");
    assert!(contents.contains("<svg"));
    assert!(!contents.contains("<!DOCTYPE"));
}

#[test]
fn rejects_animated_png() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("animated.png");
    let output = dir.path().join("output.min.png");
    fs::write(&input, apng_fixture_bytes()).expect("write apng");

    let error = optimize_image_file(&input, &output, &project_root()).expect_err("animated png");
    assert_eq!(error, ErrorPayload::animated_not_supported());
}

fn apng_fixture_bytes() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    data.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x0d, b'I', b'H', b'D', b'R', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89,
    ]);
    data.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x08, b'a', b'c', b'T', b'L', 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    data.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x00, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82,
    ]);
    data
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
    let input = fixtures_dir().join("sample.heic");
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

    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.gif", |path| {
        let img = image::RgbaImage::from_fn(320, 240, |x, y| {
            image::Rgba([(x % 255) as u8, (y % 255) as u8, ((x + y) % 255) as u8, 255])
        });
        img.save(path).expect("save gif");
    });
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
