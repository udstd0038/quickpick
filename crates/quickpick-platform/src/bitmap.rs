/// Encode a clipboard bitmap without capturing the screen or opening the clipboard.
pub(crate) fn encode_clipboard_bmp(image: &image::RgbaImage) -> Result<Vec<u8>, &'static str> {
    if image.width() == 0 || image.height() == 0 {
        return Err("剪贴板图片尺寸无效");
    }
    // clipboard-win 5.4.3 passes a fixed BITMAPINFOHEADER to CreateDIBitmap.
    // RGB24/BI_RGB needs no extended header or channel masks. Desktop RGB
    // channels are retained directly; alpha is not blended into a background.
    use image::buffer::ConvertBuffer;
    let rgb: image::RgbImage = image.convert();
    let mut bmp = Vec::new();
    image::codecs::bmp::BmpEncoder::new(&mut bmp)
        .encode(
            rgb.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|_| "编码剪贴板图片失败")?;
    Ok(bmp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_bitmap_uses_the_supported_rgb24_info_header() {
        let image = image::RgbaImage::from_pixel(32, 24, image::Rgba([255, 0, 0, 255]));
        let bmp = encode_clipboard_bmp(&image).unwrap();
        assert_eq!(u32::from_le_bytes(bmp[14..18].try_into().unwrap()), 40);
        assert_eq!(u32::from_le_bytes(bmp[10..14].try_into().unwrap()), 54);
        assert_eq!(u16::from_le_bytes(bmp[26..28].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(bmp[28..30].try_into().unwrap()), 24);
        assert_eq!(u32::from_le_bytes(bmp[30..34].try_into().unwrap()), 0);
    }

    #[test]
    fn clipboard_bitmap_preserves_rgb_rows_and_padding_without_alpha_blending() {
        for width in [1, 2, 3, 4, 5, 7, 32] {
            let source = image::RgbaImage::from_fn(width, 5, |x, y| {
                image::Rgba([
                    (x * 17 + y * 3) as u8,
                    (x * 5 + y * 43) as u8,
                    (x * 31 + y * 7) as u8,
                    [0, 64, 128, 255][(x + y) as usize % 4],
                ])
            });
            let bmp = encode_clipboard_bmp(&source).unwrap();
            let stride = (width * 3).next_multiple_of(4);
            assert_eq!(bmp.len(), 54 + stride as usize * 5);
            let decoded = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp)
                .unwrap()
                .to_rgb8();
            assert_eq!(decoded.dimensions(), source.dimensions());
            for (expected, actual) in source.pixels().zip(decoded.pixels()) {
                assert_eq!(&expected.0[..3], &actual.0);
            }
        }
    }

    #[test]
    fn empty_clipboard_image_is_rejected_before_opening_the_clipboard() {
        for (width, height) in [(0, 0), (0, 1), (1, 0)] {
            assert!(encode_clipboard_bmp(&image::RgbaImage::new(width, height)).is_err());
        }
    }
}
