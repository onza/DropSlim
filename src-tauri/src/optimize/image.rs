use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

use oxipng::{optimize_from_memory, Options, StripChunks};
use oxvg_ast::parse::roxmltree::{parse_with_options, ParsingOptions};
use oxvg_ast::serialize::Node;
use oxvg_ast::visitor::Info;
use oxvg_optimiser::Jobs;
use ravif::{Encoder, Img, RGBA8};
use zenjpeg::encoder::{ChromaSubsampling, EncoderConfig, PixelLayout, Unstoppable};

use super::animation::ensure_not_animated;
use super::formats::ImageFormat;
use super::heic::optimize_heic;
use super::payloads::ErrorPayload;
use super::temp_paths::TempFile;
use super::tools::gifsicle_path;

const JPEG_QUALITY: u8 = 85;
const PNG_QUANTIZE_MIN_QUALITY: u8 = 70;
const PNG_QUANTIZE_MAX_QUALITY: u8 = 100;
const PNG_DITHERING_LEVEL: f32 = 1.0;
const WEBP_QUALITY: f32 = 80.0;
const AVIF_QUALITY: f32 = 50.0;
const AVIF_SPEED: u8 = 4;

fn with_safe_source<F>(input: &Path, output: &Path, optimize: F) -> Result<(), ErrorPayload>
where
    F: FnOnce(&Path, &Path) -> Result<(), ErrorPayload>,
{
    if input == output {
        let tmp = TempFile::at(output);
        fs::copy(input, tmp.path()).map_err(|error| ErrorPayload::io(error.to_string()))?;
        optimize(tmp.path(), output)
    } else {
        optimize(input, output)
    }
}

fn io_error(error: impl ToString) -> ErrorPayload {
    ErrorPayload::io(error.to_string())
}

fn optimize_svg(input: &Path, output: &Path) -> Result<(), String> {
    let data = fs::read_to_string(input).map_err(|error| error.to_string())?;
    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let optimized = parse_with_options(&data, options, |dom, allocator| {
        let jobs = Jobs::default();
        jobs.run(dom, &Info::new(allocator))
            .map_err(|error| error.to_string())?;
        dom.serialize().map_err(|error| error.to_string())
    })
    .map_err(|error| error.to_string())??;

    fs::write(output, optimized).map_err(|error| error.to_string())
}

fn optimize_jpeg(input: &Path, output: &Path) -> Result<(), String> {
    let img = image::open(input).map_err(|error| error.to_string())?;
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let config = EncoderConfig::ycbcr(JPEG_QUALITY, ChromaSubsampling::Quarter).progressive(true);

    let mut enc = config
        .encode_from_bytes(width, height, PixelLayout::Rgb8Srgb)
        .map_err(|error| error.to_string())?;
    enc.push_packed(rgb.as_raw(), Unstoppable)
        .map_err(|error| error.to_string())?;
    let jpeg = enc.finish().map_err(|error| error.to_string())?;

    fs::write(output, jpeg).map_err(|error| error.to_string())
}

fn png_recompress_options() -> Options {
    Options {
        bit_depth_reduction: false,
        color_type_reduction: false,
        palette_reduction: false,
        grayscale_reduction: false,
        scale_16: false,
        strip: StripChunks::Safe,
        fast_evaluation: true,
        ..Default::default()
    }
}

fn png_uses_palette(input: &Path) -> Result<bool, String> {
    let decoder = png::Decoder::new(BufReader::new(
        fs::File::open(input).map_err(|error| error.to_string())?,
    ));
    let reader = decoder.read_info().map_err(|error| error.to_string())?;

    Ok(reader.info().color_type == png::ColorType::Indexed)
}

fn optimize_png_oxipng_only(input: &Path, output: &Path) -> Result<(), String> {
    let data = fs::read(input).map_err(|error| error.to_string())?;
    let optimized = optimize_from_memory(&data, &png_recompress_options())
        .map_err(|error| error.to_string())?;

    fs::write(output, optimized).map_err(|error| error.to_string())
}

fn optimize_png_quantized(input: &Path, output: &Path) -> Result<(), String> {
    let img = image::open(input).map_err(|error| error.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let mut liq = imagequant::new();
    liq.set_quality(PNG_QUANTIZE_MIN_QUALITY, PNG_QUANTIZE_MAX_QUALITY)
        .map_err(|error| error.to_string())?;

    let pixels: Vec<imagequant::RGBA> = rgba
        .pixels()
        .map(|pixel| imagequant::RGBA::new(pixel[0], pixel[1], pixel[2], pixel[3]))
        .collect();

    let mut liq_image = liq
        .new_image(pixels, width as usize, height as usize, 0.0)
        .map_err(|error| error.to_string())?;
    let mut quantization = liq
        .quantize(&mut liq_image)
        .map_err(|error| error.to_string())?;
    quantization
        .set_dithering_level(PNG_DITHERING_LEVEL)
        .map_err(|error| error.to_string())?;

    let (palette, pixels) = quantization
        .remapped(&mut liq_image)
        .map_err(|error| error.to_string())?;

    let mut buffer = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buffer, width, height);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        let palette_bytes: Vec<u8> = palette
            .iter()
            .flat_map(|color| [color.r, color.g, color.b])
            .collect();
        let transparency: Vec<u8> = palette.iter().map(|color| color.a).collect();
        encoder.set_palette(palette_bytes.as_slice());
        encoder.set_trns(transparency.as_slice());

        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        writer
            .write_image_data(&pixels)
            .map_err(|error| error.to_string())?;
    }

    let optimized = optimize_from_memory(&buffer, &png_recompress_options())
        .map_err(|error| error.to_string())?;

    fs::write(output, optimized).map_err(|error| error.to_string())
}

fn optimize_png(input: &Path, output: &Path) -> Result<(), String> {
    if png_uses_palette(input)? {
        optimize_png_quantized(input, output)
    } else {
        optimize_png_oxipng_only(input, output)
    }
}

fn optimize_gif(input: &Path, output: &Path, gifsicle: &Path) -> Result<(), String> {
    let status = Command::new(gifsicle)
        .args([
            "-o",
            &output.to_string_lossy(),
            &input.to_string_lossy(),
            "-O3",
            "-i",
        ])
        .status()
        .map_err(|error| error.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("gifsicle exited with {status}"))
    }
}

fn optimize_webp(input: &Path, output: &Path) -> Result<(), String> {
    let img = image::open(input).map_err(|error| error.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let encoder = webp::Encoder::from_rgba(rgba.as_raw(), width, height);
    let webp = encoder.encode(WEBP_QUALITY);

    fs::write(output, &*webp).map_err(|error| error.to_string())
}

fn optimize_avif(input: &Path, output: &Path) -> Result<(), String> {
    let img = image::open(input).map_err(|error| error.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let pixels: Vec<RGBA8> = rgba
        .pixels()
        .map(|pixel| RGBA8::new(pixel[0], pixel[1], pixel[2], pixel[3]))
        .collect();

    let encoder = Encoder::new()
        .with_quality(AVIF_QUALITY)
        .with_speed(AVIF_SPEED);
    let encoded = encoder
        .encode_rgba(Img::new(pixels.as_slice(), width as usize, height as usize))
        .map_err(|error| error.to_string())?;

    fs::write(output, encoded.avif_file).map_err(|error| error.to_string())
}

pub fn optimize_image_file(
    input: &Path,
    output: &Path,
    project_root: &Path,
) -> Result<(), ErrorPayload> {
    let format = ImageFormat::from_path(input).ok_or_else(ErrorPayload::unsupported_format)?;

    if matches!(
        format,
        ImageFormat::Png | ImageFormat::Webp | ImageFormat::Avif | ImageFormat::Heic
    ) {
        ensure_not_animated(input, format)?;
    }

    let gifsicle = gifsicle_path(project_root);

    with_safe_source(input, output, |source, destination| match format {
        ImageFormat::Svg => optimize_svg(source, destination).map_err(io_error),
        ImageFormat::Jpeg => optimize_jpeg(source, destination).map_err(io_error),
        ImageFormat::Png => optimize_png(source, destination).map_err(io_error),
        ImageFormat::Gif => {
            let gifsicle = gifsicle
                .as_deref()
                .ok_or_else(ErrorPayload::gif_optimizer_unavailable)?;
            optimize_gif(source, destination, gifsicle)
                .map_err(|message| ErrorPayload::from_message(&message))
        }
        ImageFormat::Webp => optimize_webp(source, destination).map_err(io_error),
        ImageFormat::Avif => optimize_avif(source, destination).map_err(io_error),
        ImageFormat::Heic => optimize_heic(source, destination),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};
    use std::io::Cursor;
    use tempfile::tempdir;

    fn write_indexed_png(path: &Path) {
        let mut buffer = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buffer, 2, 2);
            encoder.set_color(png::ColorType::Indexed);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_palette(&[255, 0, 0, 0, 255, 0, 0, 0, 255]);
            let mut writer = encoder.write_header().expect("png header");
            writer.write_image_data(&[0, 1, 0, 1]).expect("png pixels");
        }

        fs::write(path, buffer).expect("write indexed png");
    }

    fn write_rgba_png(path: &Path) {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(8, 8, Rgba([12, 34, 56, 255]));
        img.save(path).expect("write rgba png");
    }

    #[test]
    fn detects_indexed_png_palette() {
        let dir = tempdir().expect("tempdir");
        let indexed = dir.path().join("indexed.png");
        let rgba = dir.path().join("rgba.png");
        write_indexed_png(&indexed);
        write_rgba_png(&rgba);

        assert!(png_uses_palette(&indexed).expect("indexed png"));
        assert!(!png_uses_palette(&rgba).expect("rgba png"));
    }

    #[test]
    fn optimizes_true_color_png_with_oxipng_only() {
        let dir = tempdir().expect("tempdir");
        let input = dir.path().join("photo.png");
        write_rgba_png(&input);
        let output = dir.path().join("photo.min.png");

        optimize_png(&input, &output).expect("optimize png");

        let optimized = fs::read(&output).expect("read output");
        let info = png::Decoder::new(Cursor::new(&optimized))
            .read_info()
            .expect("decode optimized png")
            .info()
            .color_type;

        assert_ne!(info, png::ColorType::Indexed);
    }

    #[test]
    fn keeps_quantize_path_for_indexed_png() {
        let dir = tempdir().expect("tempdir");
        let input = dir.path().join("graphic.png");
        write_indexed_png(&input);
        let output = dir.path().join("graphic.min.png");

        optimize_png(&input, &output).expect("optimize indexed png");

        let optimized = fs::read(&output).expect("read output");
        let info = png::Decoder::new(Cursor::new(&optimized))
            .read_info()
            .expect("decode optimized png")
            .info()
            .color_type;

        assert_eq!(info, png::ColorType::Indexed);
    }
}
