use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxvg_ast::parse::roxmltree::parse;
use oxvg_ast::serialize::Node;
use oxvg_ast::visitor::Info;
use oxvg_optimiser::Jobs;
use ravif::{Encoder, Img, RGBA8};

use super::formats::{ImageFormat, SUPPORTED_FORMATS_LABEL};
use super::tools::gifsicle_path;

fn temp_source_path(output: &Path) -> PathBuf {
    let extension = output
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();
    let stem = output
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("tmp");

    output.with_file_name(format!("{stem}.dropslim{extension}"))
}

fn with_safe_source<F>(input: &Path, output: &Path, optimize: F) -> Result<(), String>
where
    F: FnOnce(&Path, &Path) -> Result<(), String>,
{
    if input == output {
        let tmp = temp_source_path(output);
        fs::copy(input, &tmp).map_err(|error| error.to_string())?;

        let result = optimize(&tmp, output);
        let _ = fs::remove_file(&tmp);
        result
    } else {
        optimize(input, output)
    }
}

fn optimize_svg(input: &Path, output: &Path) -> Result<(), String> {
    let data = fs::read_to_string(input).map_err(|error| error.to_string())?;
    let optimized = parse(&data, |dom, allocator| {
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

    let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    comp.set_size(width as usize, height as usize);
    comp.set_quality(85.0);
    comp.set_progressive_mode();
    comp.set_optimize_coding(true);

    let mut comp = comp
        .start_compress(Vec::new())
        .map_err(|error| error.to_string())?;
    comp.write_scanlines(rgb.as_raw())
        .map_err(|error| error.to_string())?;
    let jpeg = comp.finish().map_err(|error| error.to_string())?;

    fs::write(output, jpeg).map_err(|error| error.to_string())
}

fn optimize_png(input: &Path, output: &Path) -> Result<(), String> {
    let img = image::open(input).map_err(|error| error.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let mut liq = imagequant::new();
    liq.set_quality(70, 100)
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
        .set_dithering_level(1.0)
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

        let mut writer = encoder
            .write_header()
            .map_err(|error| error.to_string())?;
        writer
            .write_image_data(&pixels)
            .map_err(|error| error.to_string())?;
    }

    fs::write(output, buffer).map_err(|error| error.to_string())
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
    let webp = encoder.encode(80.0);

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

    let encoder = Encoder::new().with_quality(50.0).with_speed(4);
    let encoded = encoder
        .encode_rgba(Img::new(
            pixels.as_slice(),
            width as usize,
            height as usize,
        ))
        .map_err(|error| error.to_string())?;

    fs::write(output, encoded.avif_file).map_err(|error| error.to_string())
}

pub fn optimize_image_file(
    input: &Path,
    output: &Path,
    project_root: &Path,
) -> Result<(), String> {
    let format = ImageFormat::from_path(input)
        .ok_or_else(|| format!("Only {SUPPORTED_FORMATS_LABEL} are supported."))?;

    let gifsicle = gifsicle_path(project_root);

    with_safe_source(input, output, |source, destination| match format {
        ImageFormat::Svg => optimize_svg(source, destination),
        ImageFormat::Jpeg => optimize_jpeg(source, destination),
        ImageFormat::Png => optimize_png(source, destination),
        ImageFormat::Gif => {
            let gifsicle = gifsicle
                .as_deref()
                .ok_or_else(|| "GIF optimizer is not available.".to_string())?;
            optimize_gif(source, destination, gifsicle)
        }
        ImageFormat::Webp => optimize_webp(source, destination),
        ImageFormat::Avif => optimize_avif(source, destination),
    })
}
