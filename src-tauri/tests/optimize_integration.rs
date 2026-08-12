use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use dropslim_lib::optimize::{optimize_image_file, ErrorPayload, OutputFormatSetting};
use image::codecs::gif::{GifDecoder, GifEncoder};
use image::{AnimationDecoder, Delay, Frame, GenericImageView, ImageBuffer, Rgba};
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

    optimize_image_file(
        input,
        &output,
        &project_root(),
        None,
        OutputFormatSetting::Original,
    )
    .expect("optimize");

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

fn require_gifsicle() -> Option<PathBuf> {
    let gifsicle = project_root()
        .join("vendor")
        .join("gifsicle")
        .join(if cfg!(windows) {
            "gifsicle.exe"
        } else {
            "gifsicle"
        });

    if !gifsicle.exists() {
        eprintln!(
            "skip: gifsicle not found at {} — run npm ci to install vendor binaries",
            gifsicle.display()
        );
        return None;
    }

    Some(gifsicle)
}

fn write_animated_gif(path: &Path) {
    let frame_a: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(48, 48, |x, y| Rgba([x as u8, y as u8, 40, 255]));
    let frame_b: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(48, 48, |x, y| Rgba([y as u8, x as u8, 200, 255]));

    let frames = vec![
        Frame::from_parts(frame_a, 0, 0, Delay::from_numer_denom_ms(100, 1)),
        Frame::from_parts(frame_b, 0, 0, Delay::from_numer_denom_ms(100, 1)),
    ];

    let file = fs::File::create(path).expect("create gif");
    let mut encoder = GifEncoder::new(file);
    encoder.encode_frames(frames).expect("encode animated gif");
}

fn gif_frame_count(path: &Path) -> usize {
    let data = fs::read(path).expect("read gif");
    let decoder = GifDecoder::new(Cursor::new(data)).expect("decode gif");
    decoder
        .into_frames()
        .collect_frames()
        .expect("collect gif frames")
        .len()
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

    let error = optimize_image_file(
        &input,
        &output,
        &project_root(),
        None,
        OutputFormatSetting::Original,
    )
    .expect_err("animated png");
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
    if require_gifsicle().is_none() {
        return;
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
fn optimizes_animated_gif_preserving_frames() {
    if require_gifsicle().is_none() {
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("animated.gif");
    let output = dir.path().join("animated.min.gif");
    write_animated_gif(&input);

    assert!(
        gif_frame_count(&input) >= 2,
        "fixture should be an animated gif"
    );

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        None,
        OutputFormatSetting::Original,
    )
    .expect("optimize animated gif");

    assert!(
        gif_frame_count(&output) >= 2,
        "optimized gif should still contain multiple frames"
    );
}

#[test]
fn resizes_jpeg_to_max_dimensions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "large.jpg", |path| {
        let img = image::RgbImage::from_fn(800, 600, |x, y| {
            image::Rgb([(x % 255) as u8, (y % 255) as u8, 40])
        });
        img.save(path).expect("save jpg");
    });
    let output = dir.path().join("large.min.jpg");

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        Some((Some(200), Some(200))),
        OutputFormatSetting::Original,
    )
    .expect("optimize");

    let optimized = image::open(&output).expect("open output");
    assert_eq!(optimized.dimensions(), (200, 150));
}

#[test]
fn ignores_dimension_limits_for_gif() {
    if require_gifsicle().is_none() {
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.gif", |path| {
        let img = image::RgbaImage::from_fn(120, 80, |x, y| {
            image::Rgba([(x % 255) as u8, (y % 255) as u8, 90, 255])
        });
        img.save(path).expect("save gif");
    });
    let output = dir.path().join("sample.min.gif");

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        Some((Some(40), Some(40))),
        OutputFormatSetting::Original,
    )
    .expect("optimize gif");

    let optimized = image::open(&output).expect("open output");
    assert_eq!(optimized.dimensions(), (120, 80));
}

#[test]
fn optimizes_png_in_place() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "in-place.png", |path| {
        let img = image::RgbaImage::from_pixel(160, 120, image::Rgba([37, 99, 235, 255]));
        img.save(path).expect("save png");
    });
    let input_size = fs::metadata(&input).expect("metadata").len();
    optimize_image_file(
        &input,
        &input,
        &project_root(),
        None,
        OutputFormatSetting::Original,
    )
    .expect("in-place");
    let output_size = fs::metadata(&input).expect("metadata").len();
    assert!(output_size < input_size);
}

#[test]
fn converts_png_to_jpeg() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.png", |path| {
        let img = image::RgbaImage::from_pixel(160, 120, image::Rgba([37, 99, 235, 255]));
        img.save(path).expect("save png");
    });
    let output = dir.path().join("sample.min.jpg");

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        None,
        OutputFormatSetting::Jpeg,
    )
    .expect("convert");

    let data = fs::read(&output).expect("read output");
    assert_eq!(&data[..2], b"\xff\xd8");
    let optimized = image::open(&output).expect("open output");
    assert_eq!(optimized.dimensions(), (160, 120));
}

#[test]
fn converts_jpeg_to_webp() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.jpg", |path| {
        let img = image::RgbImage::from_pixel(120, 80, image::Rgb([220, 120, 40]));
        img.save(path).expect("save jpg");
    });
    let output = dir.path().join("sample.min.webp");

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        None,
        OutputFormatSetting::Webp,
    )
    .expect("convert");

    let data = fs::read(&output).expect("read output");
    assert_eq!(&data[..4], b"RIFF");
    let optimized = image::open(&output).expect("open output");
    assert_eq!(optimized.dimensions(), (120, 80));
}

#[test]
fn converts_png_to_avif() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = create_raster_fixture(&dir, "sample.png", |path| {
        let img = image::RgbaImage::from_pixel(160, 120, image::Rgba([37, 99, 235, 255]));
        img.save(path).expect("save png");
    });
    let output = dir.path().join("sample.min.avif");

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        None,
        OutputFormatSetting::Avif,
    )
    .expect("convert");

    let data = fs::read(&output).expect("read output");
    assert!(data.len() > 12);
    assert_eq!(&data[4..8], b"ftyp");
    let optimized = image::open(&output).expect("open output");
    assert_eq!(optimized.dimensions(), (160, 120));
}

#[cfg(target_os = "macos")]
#[test]
fn converts_heic_to_jpeg() {
    let input = fixtures_dir().join("sample.heic");
    assert!(
        input.is_file(),
        "missing test/fixtures/sample.heic — run: cargo run --example write_heic_fixture"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("sample.min.jpg");

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        None,
        OutputFormatSetting::Jpeg,
    )
    .expect("convert heic");

    let data = fs::read(&output).expect("read output");
    assert_eq!(&data[..2], b"\xff\xd8");
    let optimized = image::open(&output).expect("open output");
    assert_eq!(optimized.dimensions(), (512, 384));
}

#[cfg(target_os = "macos")]
#[test]
fn converts_heic_to_jpeg_with_dimension_limits() {
    let input = fixtures_dir().join("sample.heic");
    assert!(
        input.is_file(),
        "missing test/fixtures/sample.heic — run: cargo run --example write_heic_fixture"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("sample.min.jpg");

    optimize_image_file(
        &input,
        &output,
        &project_root(),
        Some((Some(256), None)),
        OutputFormatSetting::Jpeg,
    )
    .expect("convert heic with limits");

    let optimized = image::open(&output).expect("open output");
    assert_eq!(optimized.dimensions(), (256, 192));
}
