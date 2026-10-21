use image::{ImageFormat, ImageReader, RgbaImage};
use std::{fs, path::Path};

const MAX_DIMENSION: u32 = 4096;
const MAX_RGBA_BYTES: u64 = 64 * 1024 * 1024;

pub fn png_to_bmp(input: &Path, output: &Path) -> Result<(), String> {
    let image = read_png(input)?;
    let bytes = encode_bmp(&image, image.width(), image.height())?;
    fs::write(output, bytes).map_err(|error| format!("Cannot write BMP: {error}"))
}

pub fn png_to_bmp_cropped(
    input: &Path,
    output: &Path,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let image = read_png(input)?;
    let bytes = encode_bmp(&image, width, height)?;
    fs::write(output, bytes).map_err(|error| format!("Cannot write BMP: {error}"))
}

fn read_png(input: &Path) -> Result<RgbaImage, String> {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_RGBA_BYTES);
    let mut reader =
        ImageReader::open(input).map_err(|error| format!("Cannot read PNG: {error}"))?;
    reader.set_format(ImageFormat::Png);
    reader.limits(limits);
    reader
        .decode()
        .map(|image| image.into_rgba8())
        .map_err(|error| format!("Cannot decode PNG: {error}"))
}

// Windows BITMAPINFOHEADER, 24-bit BGR, bottom-up rows and zeroed 4-byte padding.
// Installer artwork must be fully opaque; silently discarding alpha changes it.
fn encode_bmp(image: &RgbaImage, width: u32, height: u32) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 || width > image.width() || height > image.height() {
        return Err("BMP crop is empty or outside the input PNG.".into());
    }
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err("BMP dimensions exceed the resource limit.".into());
    }
    let stride = (width as usize * 3 + 3) & !3;
    let pixels_length = stride * height as usize;
    let mut bytes = vec![0; 54 + pixels_length];
    let file_length = bytes.len() as u32;
    bytes[0..2].copy_from_slice(b"BM");
    bytes[2..6].copy_from_slice(&file_length.to_le_bytes());
    bytes[10..14].copy_from_slice(&54u32.to_le_bytes());
    bytes[14..18].copy_from_slice(&40u32.to_le_bytes());
    bytes[18..22].copy_from_slice(&(width as i32).to_le_bytes());
    bytes[22..26].copy_from_slice(&(height as i32).to_le_bytes());
    bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
    bytes[28..30].copy_from_slice(&24u16.to_le_bytes());
    bytes[34..38].copy_from_slice(&(pixels_length as u32).to_le_bytes());
    for y in 0..height {
        let row = 54 + (height - y - 1) as usize * stride;
        for x in 0..width {
            let rgba = image.get_pixel(x, y).0;
            if rgba[3] != 255 {
                return Err("Installer BMP source contains transparent pixels.".into());
            }
            let offset = row + x as usize * 3;
            bytes[offset..offset + 3].copy_from_slice(&[rgba[2], rgba[1], rgba[0]]);
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn bmp_is_bottom_up_bgr_with_padded_rows() {
        let mut image = RgbaImage::new(1, 2);
        image.put_pixel(0, 0, Rgba([1, 2, 3, 255]));
        image.put_pixel(0, 1, Rgba([4, 5, 6, 255]));
        let bytes = encode_bmp(&image, 1, 2).unwrap();
        assert_eq!(&bytes[54..], &[6, 5, 4, 0, 3, 2, 1, 0]);
        assert_eq!(u32::from_le_bytes(bytes[2..6].try_into().unwrap()), 62);
        assert_eq!(u16::from_le_bytes(bytes[28..30].try_into().unwrap()), 24);
        assert_eq!(u32::from_le_bytes(bytes[30..34].try_into().unwrap()), 0);
    }

    #[test]
    fn square_padding_is_cropped_without_rescaling() {
        let mut image = RgbaImage::from_pixel(3, 3, Rgba([10, 20, 30, 255]));
        image.put_pixel(1, 0, Rgba([21, 22, 23, 255]));
        image.put_pixel(0, 2, Rgba([0, 0, 0, 0]));
        let bytes = encode_bmp(&image, 2, 1).unwrap();
        assert_eq!(&bytes[54..], &[30, 20, 10, 23, 22, 21, 0, 0]);
        assert_eq!(i32::from_le_bytes(bytes[18..22].try_into().unwrap()), 2);
        assert_eq!(i32::from_le_bytes(bytes[22..26].try_into().unwrap()), 1);
    }

    #[test]
    fn transparent_source_and_invalid_crops_fail() {
        let image = RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 0]));
        assert!(encode_bmp(&image, 2, 2)
            .unwrap_err()
            .contains("transparent"));
        for (width, height) in [(0, 2), (2, 0), (3, 2), (2, 3)] {
            assert!(encode_bmp(&image, width, height).is_err());
        }
    }
}
