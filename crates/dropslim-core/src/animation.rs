use std::fs;
use std::io::Cursor;
use std::path::Path;

use mp4parse::{read_avif, ParseStrictness};

use super::formats::ImageFormat;
use super::heic::heic_is_animated;
use super::payloads::ErrorPayload;

pub fn ensure_not_animated(input: &Path, format: ImageFormat) -> Result<(), ErrorPayload> {
    if is_animated(input, format)? {
        return Err(ErrorPayload::animated_not_supported());
    }

    Ok(())
}

fn is_animated(input: &Path, format: ImageFormat) -> Result<bool, ErrorPayload> {
    match format {
        ImageFormat::Png => {
            let data = fs::read(input).map_err(|error| ErrorPayload::io(error.to_string()))?;
            Ok(png_is_animated(&data))
        }
        ImageFormat::Webp => {
            let data = fs::read(input).map_err(|error| ErrorPayload::io(error.to_string()))?;
            Ok(webp_is_animated(&data))
        }
        ImageFormat::Avif => {
            let data = fs::read(input).map_err(|error| ErrorPayload::io(error.to_string()))?;
            Ok(avif_is_animated(&data))
        }
        ImageFormat::Heic => heic_is_animated(input),
        _ => Ok(false),
    }
}

fn png_is_animated(data: &[u8]) -> bool {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

    if data.len() < SIGNATURE.len() || &data[..SIGNATURE.len()] != SIGNATURE {
        return false;
    }

    let mut offset = SIGNATURE.len();

    while offset + 12 <= data.len() {
        let chunk_len = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk_type = &data[offset + 4..offset + 8];

        if chunk_type == b"acTL" {
            return true;
        }

        if chunk_type == b"IEND" {
            break;
        }

        let next = offset + 12 + chunk_len;
        if next > data.len() {
            break;
        }

        offset = next;
    }

    false
}

fn webp_is_animated(data: &[u8]) -> bool {
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return false;
    }

    let mut offset = 12;

    while offset + 8 <= data.len() {
        let chunk_type = &data[offset..offset + 4];
        let chunk_size =
            u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as usize;

        if chunk_type == b"ANIM" || chunk_type == b"ANMF" {
            return true;
        }

        if chunk_type == b"VP8X" && chunk_size >= 1 && offset + 8 < data.len() {
            let flags = data[offset + 8];
            if flags & 0x02 != 0 {
                return true;
            }
        }

        offset += 8 + chunk_size + (chunk_size & 1);
    }

    false
}

fn avif_is_animated(data: &[u8]) -> bool {
    let mut cursor = Cursor::new(data);
    read_avif(&mut cursor, ParseStrictness::Normal)
        .ok()
        .is_some_and(|context| context.sequence.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apng_fixture_bytes() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        data.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x0d, b'I', b'H', b'D', b'R', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89,
        ]);
        data.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x08, b'a', b'c', b'T', b'L', 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        data.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x00, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82,
        ]);
        data
    }

    #[test]
    fn detects_apng_chunk() {
        assert!(png_is_animated(&apng_fixture_bytes()));
    }

    #[test]
    fn static_png_is_not_animated() {
        let data = [
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
            0x42, 0x60, 0x82,
        ];

        assert!(!png_is_animated(&data));
    }

    #[test]
    fn detects_vp8x_animation_flag() {
        let data = [
            b'R', b'I', b'F', b'F', 0x1a, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P',
            b'8', b'X', 0x0a, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x00,
        ];

        assert!(webp_is_animated(&data));
    }

    #[test]
    fn static_webp_is_not_animated() {
        let data = [
            b'R', b'I', b'F', b'F', 0x0c, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P',
            b'8', b' ', 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        assert!(!webp_is_animated(&data));
    }
}
