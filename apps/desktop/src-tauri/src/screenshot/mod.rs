mod session;

pub use session::{ScreenshotSession, ScreenshotStore};

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::io::{self, Write};
use xcap::Monitor;
use zeroize::{Zeroize, Zeroizing};

pub const CLIPBOARD_FORMAT: &str = "CF_DIB + CF_BITMAP";
pub const CLIPBOARD_FORMAT_MACOS: &str = "PNG";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureClipboardResult {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
    pub clipboard_format: &'static str,
    pub message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorScreenshotPayload {
    pub session_id: u64,
    pub monitor_id: u32,
    /// PNG pixel dimensions, independent of the monitor's coordinate system.
    pub width: u32,
    pub height: u32,
    /// macOS: CoreGraphics points. Windows: desktop physical pixels.
    pub monitor_x: i32,
    pub monitor_y: i32,
    pub monitor_width: u32,
    pub monitor_height: u32,
    pub monitor_name: String,
    pub png_data_url: String,
}

pub struct CapturedRegionImage {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
    pub png_bytes: Vec<u8>,
    #[cfg_attr(windows, allow(dead_code))]
    rgba_bytes: Vec<u8>,
    bmp_bytes: Vec<u8>,
}

impl Drop for CapturedRegionImage {
    fn drop(&mut self) {
        self.png_bytes.zeroize();
        self.rgba_bytes.zeroize();
        self.bmp_bytes.zeroize();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionMenuAction {
    Copy,
    Extract,
    Translate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionMenuSelection {
    pub action: RegionMenuAction,
    pub session_id: Option<u64>,
    /// Pixel coordinates relative to the captured image, never desktop coordinates.
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Uses CoreGraphics point coordinates on macOS and desktop physical pixels on Windows.
pub fn monitor_at_point(x: i32, y: i32) -> Result<Monitor, String> {
    Monitor::from_point(x, y).map_err(|error| format!("定位鼠标所在显示器失败：{error}"))
}

pub fn capture_monitor_screenshot(
    session_id: u64,
    monitor: &Monitor,
) -> Result<(MonitorScreenshotPayload, ScreenshotSession), String> {
    session::validate_session_id(session_id)?;
    let monitor_id = monitor
        .id()
        .map_err(|error| format!("读取显示器标识失败：{error}"))?;
    // friendly_name() calls NSScreen on macOS; capture workers must avoid AppKit.
    let monitor_name = monitor
        .name()
        .unwrap_or_else(|_| format!("Display #{monitor_id}"));
    let monitor_x = monitor
        .x()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;
    let monitor_y = monitor
        .y()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;

    let monitor_width = monitor
        .width()
        .map_err(|error| format!("读取显示器尺寸失败：{error}"))?;
    let monitor_height = monitor
        .height()
        .map_err(|error| format!("读取显示器尺寸失败：{error}"))?;
    #[cfg(target_os = "macos")]
    let scale = monitor
        .scale_factor()
        .map_err(|error| format!("读取显示器缩放失败：{error}"))?;
    #[cfg(not(target_os = "macos"))]
    let scale = 1.0;
    projected_capture_size(monitor_width, monitor_height, scale)?;

    let image = ErasedRgbaImage(
        monitor
            .capture_image()
            .map_err(|error| format!("读取屏幕失败：{error}"))?,
    );
    session::rgba_size(image.0.width(), image.0.height())?;
    let png_bytes = Zeroizing::new(rgba_image_to_png_bytes(&image.0)?);
    let mut png_data_url = String::from("data:image/png;base64,");
    png_data_url
        .try_reserve_exact(((png_bytes.len() + 2) / 3) * 4)
        .map_err(|_| "截图预览内存分配失败，请选择较小区域".to_string())?;
    general_purpose::STANDARD.encode_string(&*png_bytes, &mut png_data_url);
    let payload = MonitorScreenshotPayload {
        session_id,
        monitor_id,
        width: image.0.width(),
        height: image.0.height(),
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
        monitor_name,
        png_data_url,
    };
    let session = ScreenshotSession::new(payload, image.into_inner())?;
    Ok((session.payload().clone(), session))
}

fn projected_capture_size(width: u32, height: u32, scale: f32) -> Result<(u32, u32), String> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err("显示器缩放无效，请重新截图".to_string());
    }
    let pixel_width = (width as f64 * scale as f64).ceil();
    let pixel_height = (height as f64 * scale as f64).ceil();
    if pixel_width > u32::MAX as f64 || pixel_height > u32::MAX as f64 {
        return Err("截图尺寸超过内存限制".to_string());
    }
    let dimensions = (pixel_width as u32, pixel_height as u32);
    session::rgba_size(dimensions.0, dimensions.1)?;
    Ok(dimensions)
}

pub fn crop_selected_region(
    session: ScreenshotSession,
    selection: RegionMenuSelection,
) -> Result<CapturedRegionImage, String> {
    let monitor_name = session.payload().monitor_name.clone();
    // into_crop consumes/erases the full frame and preview before ROI encoding.
    let image = ErasedRgbaImage(session.into_crop(selection)?);
    let png_bytes = Zeroizing::new(rgba_image_to_png_bytes(&image.0)?);
    let bmp_bytes = if cfg!(windows) {
        rgba_image_to_bmp_bytes(&image.0)?
    } else {
        Vec::new()
    };
    Ok(CapturedRegionImage {
        width: image.0.width(),
        height: image.0.height(),
        monitor_name,
        png_bytes: take_zeroizing_bytes(png_bytes),
        rgba_bytes: image.into_inner().into_raw(),
        bmp_bytes,
    })
}

struct ErasedRgbaImage(image::RgbaImage);

impl ErasedRgbaImage {
    fn into_inner(mut self) -> image::RgbaImage {
        std::mem::replace(&mut self.0, image::RgbaImage::new(0, 0))
    }
}

impl Drop for ErasedRgbaImage {
    fn drop(&mut self) {
        let bytes: &mut [u8] = self.0.as_mut();
        bytes.zeroize();
    }
}

fn take_zeroizing_bytes(mut bytes: Zeroizing<Vec<u8>>) -> Vec<u8> {
    std::mem::take(&mut *bytes)
}

pub fn copy_captured_region_to_clipboard(
    _app: &tauri::AppHandle,
    image: &CapturedRegionImage,
) -> Result<CaptureClipboardResult, String> {
    #[cfg(windows)]
    copy_bmp_bytes_to_clipboard(&image.bmp_bytes)?;

    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_clipboard_manager::ClipboardExt;

        let tauri_image = tauri::image::Image::new(&image.rgba_bytes, image.width, image.height);
        _app.clipboard()
            .write_image(&tauri_image)
            .map_err(|error| format!("写入截图到剪贴板失败：{error}"))?;
    }

    let clipboard_format = if cfg!(target_os = "macos") {
        CLIPBOARD_FORMAT_MACOS
    } else {
        CLIPBOARD_FORMAT
    };

    Ok(CaptureClipboardResult {
        width: image.width,
        height: image.height,
        monitor_name: image.monitor_name.clone(),
        clipboard_format,
        message: format!(
            "已复制区域截图：{}x{}（{}）",
            image.width, image.height, image.monitor_name
        ),
    })
}

#[cfg(windows)]
fn copy_bmp_bytes_to_clipboard(bmp_bytes: &[u8]) -> Result<(), String> {
    const BMP_FILE_HEADER_SIZE: usize = 14;

    let _clipboard = clipboard_win::Clipboard::new_attempts(10)
        .map_err(|_| "打开系统剪贴板失败，请稍后再试".to_string())?;
    let dib_bytes = bmp_bytes
        .get(BMP_FILE_HEADER_SIZE..)
        .ok_or_else(|| "截图图片格式无效，无法复制".to_string())?;

    clipboard_win::raw::empty().map_err(|_| "清空剪贴板失败，请稍后再试".to_string())?;
    clipboard_win::raw::set_without_clear(clipboard_win::formats::CF_DIB, dib_bytes)
        .map_err(|_| "写入图片到剪贴板失败，请稍后再试".to_string())?;
    clipboard_win::raw::set_bitmap_with(bmp_bytes, clipboard_win::options::NoClear)
        .map_err(|_| "写入图片到剪贴板失败，请稍后再试".to_string())
}

fn rgba_image_to_bmp_bytes(image: &image::RgbaImage) -> Result<Vec<u8>, String> {
    const BMP_FILE_HEADER_SIZE: usize = 14;
    const BMP_INFO_HEADER_SIZE: usize = 40;
    const BYTES_PER_PIXEL: usize = 4;

    let width = image.width() as usize;
    let height = image.height() as usize;
    session::rgba_size(image.width(), image.height())?;

    let row_size = width
        .checked_mul(BYTES_PER_PIXEL)
        .ok_or_else(|| "截图尺寸过大，无法复制".to_string())?;
    let pixel_size = row_size
        .checked_mul(height)
        .ok_or_else(|| "截图尺寸过大，无法复制".to_string())?;
    let header_size = BMP_FILE_HEADER_SIZE + BMP_INFO_HEADER_SIZE;
    let file_size = header_size
        .checked_add(pixel_size)
        .ok_or_else(|| "截图尺寸过大，无法复制".to_string())?;

    let width_i32 = i32::try_from(width).map_err(|_| "截图宽度过大，无法复制".to_string())?;
    let height_i32 = i32::try_from(height).map_err(|_| "截图高度过大，无法复制".to_string())?;
    let file_size_u32 =
        u32::try_from(file_size).map_err(|_| "截图尺寸过大，无法复制".to_string())?;
    let pixel_size_u32 =
        u32::try_from(pixel_size).map_err(|_| "截图尺寸过大，无法复制".to_string())?;

    let mut bytes = Vec::with_capacity(file_size);
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&file_size_u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(header_size as u32).to_le_bytes());

    bytes.extend_from_slice(&(BMP_INFO_HEADER_SIZE as u32).to_le_bytes());
    bytes.extend_from_slice(&width_i32.to_le_bytes());
    bytes.extend_from_slice(&height_i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&32u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&pixel_size_u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());

    let raw = image.as_raw();
    for y in (0..height).rev() {
        let row_start = y * row_size;
        let row_end = row_start + row_size;
        for rgba in raw[row_start..row_end].chunks_exact(BYTES_PER_PIXEL) {
            bytes.push(rgba[2]);
            bytes.push(rgba[1]);
            bytes.push(rgba[0]);
            bytes.push(rgba[3]);
        }
    }

    Ok(bytes)
}

fn rgba_image_to_png_bytes(image: &image::RgbaImage) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;

    session::rgba_size(image.width(), image.height())?;
    let mut bytes = LimitedPngBuffer {
        bytes: Zeroizing::new(Vec::new()),
        limit: session::MAX_PNG_BYTES,
    };
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8.into(),
        )
        .map_err(|_| "截图图片编码失败或超过内存限制，请选择较小区域".to_string())?;
    Ok(take_zeroizing_bytes(bytes.bytes))
}

struct LimitedPngBuffer {
    bytes: Zeroizing<Vec<u8>>,
    limit: usize,
}

impl Write for LimitedPngBuffer {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes
            .len()
            .checked_add(buffer.len())
            .filter(|len| *len <= self.limit)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "PNG exceeds memory limit")
            })?;
        self.bytes
            .try_reserve_exact(buffer.len())
            .map_err(|_| io::Error::new(io::ErrorKind::OutOfMemory, "PNG allocation failed"))?;
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retina_crop_uses_preview_pixels_and_preserves_encoded_content() {
        let pixels =
            image::RgbaImage::from_fn(32, 24, |x, y| image::Rgba([x as u8, y as u8, 73, 255]));
        let session = ScreenshotSession::new(
            MonitorScreenshotPayload {
                session_id: 1,
                monitor_id: 5,
                width: 32,
                height: 24,
                monitor_x: -16,
                monitor_y: 0,
                monitor_width: 16,
                monitor_height: 12,
                monitor_name: "Retina".to_string(),
                png_data_url: "data:image/png;base64,AAAA".to_string(),
            },
            pixels,
        )
        .unwrap();
        let image = crop_selected_region(
            session,
            RegionMenuSelection {
                action: RegionMenuAction::Copy,
                session_id: Some(1),
                x: 16,
                y: 8,
                width: 16,
                height: 16,
            },
        )
        .unwrap();
        assert_eq!((image.width, image.height), (16, 16));
        assert_eq!(image.rgba_bytes.len(), 16 * 16 * 4);
        let decoded =
            image::load_from_memory_with_format(&image.png_bytes, image::ImageFormat::Png)
                .unwrap()
                .into_rgba8();
        assert_eq!(decoded.get_pixel(0, 0).0, [16, 8, 73, 255]);
        assert_eq!(decoded.get_pixel(15, 15).0, [31, 23, 73, 255]);
        #[cfg(windows)]
        assert_eq!(image.bmp_bytes.len(), 14 + 40 + 16 * 16 * 4);
    }

    #[test]
    fn projected_capture_budget_rejects_invalid_or_excessive_scale() {
        assert_eq!(projected_capture_size(16, 12, 2.0).unwrap(), (32, 24));
        for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(projected_capture_size(16, 12, scale).is_err());
        }
        assert!(projected_capture_size(u32::MAX, 1, 2.0).is_err());
        assert!(projected_capture_size(16_384, 16_384, 1.0).is_err());
    }

    #[test]
    fn png_writer_refuses_over_budget_writes_without_appending_data() {
        let mut writer = LimitedPngBuffer {
            bytes: Zeroizing::new(Vec::new()),
            limit: 4,
        };
        writer.write_all(&[1, 2, 3]).unwrap();
        assert!(writer.write_all(&[4, 5]).is_err());
        assert_eq!(&*writer.bytes, &[1, 2, 3]);
    }

    #[test]
    fn rgba_image_is_encoded_as_bottom_up_bmp() {
        let image = image::RgbaImage::from_raw(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        )
        .expect("valid test image");

        let bytes = rgba_image_to_bmp_bytes(&image).expect("bmp bytes");

        assert_eq!(&bytes[0..2], b"BM");
        assert_eq!(bytes.len(), 14 + 40 + 16);
        assert_eq!(
            u32::from_le_bytes(bytes[2..6].try_into().unwrap()),
            bytes.len() as u32
        );
        assert_eq!(u32::from_le_bytes(bytes[10..14].try_into().unwrap()), 54);

        let pixels = &bytes[54..];
        assert_eq!(&pixels[0..4], &[255, 0, 0, 255]);
        assert_eq!(&pixels[4..8], &[255, 255, 255, 255]);
        assert_eq!(&pixels[8..12], &[0, 0, 255, 255]);
        assert_eq!(&pixels[12..16], &[0, 255, 0, 255]);
    }

    #[test]
    fn rgba_image_can_be_encoded_as_png() {
        let image =
            image::RgbaImage::from_raw(1, 1, vec![12, 34, 56, 255]).expect("valid test image");

        let bytes = rgba_image_to_png_bytes(&image).expect("png bytes");

        assert_eq!(&bytes[0..8], b"\x89PNG\r\n\x1a\n");
    }
}
