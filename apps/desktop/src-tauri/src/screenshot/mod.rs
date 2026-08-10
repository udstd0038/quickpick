use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use xcap::Monitor;

pub const CLIPBOARD_FORMAT: &str = "CF_DIB + CF_BITMAP";
const MIN_REGION_SIZE: i32 = 8;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureClipboardResult {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
    pub clipboard_format: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorScreenshotPayload {
    pub width: u32,
    pub height: u32,
    pub monitor_x: i32,
    pub monitor_y: i32,
    pub monitor_name: String,
    pub png_data_url: String,
}

pub struct CapturedRegionImage {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
    pub png_bytes: Vec<u8>,
    bmp_bytes: Vec<u8>,
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
    pub screen_x: i32,
    pub screen_y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn capture_current_monitor_to_clipboard() -> Result<CaptureClipboardResult, String> {
    let monitors = Monitor::all().map_err(|error| format!("读取显示器列表失败：{error}"))?;
    if monitors.is_empty() {
        return Err("未发现可截图的显示器".to_string());
    }

    let monitor_index = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);
    let monitor = monitors
        .into_iter()
        .nth(monitor_index)
        .ok_or_else(|| "选择显示器失败，请稍后重试".to_string())?;
    let monitor_name = monitor
        .friendly_name()
        .or_else(|_| monitor.name())
        .unwrap_or_else(|_| "当前显示器".to_string());

    let image = monitor
        .capture_image()
        .map_err(|error| format!("读取屏幕失败：{error}"))?;
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return Err("截图结果为空，请稍后重试".to_string());
    }

    let bmp_bytes = rgba_image_to_bmp_bytes(&image)?;
    copy_bmp_bytes_to_clipboard(&bmp_bytes)?;

    Ok(CaptureClipboardResult {
        width,
        height,
        monitor_name: monitor_name.clone(),
        clipboard_format: CLIPBOARD_FORMAT,
        message: format!(
            "已复制当前显示器截图：{}x{}（{}）",
            width, height, monitor_name
        ),
    })
}

pub fn capture_current_monitor_screenshot() -> Result<MonitorScreenshotPayload, String> {
    let monitors = Monitor::all().map_err(|error| format!("读取显示器列表失败：{error}"))?;
    if monitors.is_empty() {
        return Err("未发现可截图的显示器".to_string());
    }

    let monitor_index = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);
    let monitor = monitors
        .into_iter()
        .nth(monitor_index)
        .ok_or_else(|| "选择显示器失败，请稍后重试".to_string())?;
    let monitor_name = monitor
        .friendly_name()
        .or_else(|_| monitor.name())
        .unwrap_or_else(|_| "当前显示器".to_string());
    let monitor_x = monitor
        .x()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;
    let monitor_y = monitor
        .y()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;

    let image = monitor
        .capture_image()
        .map_err(|error| format!("读取屏幕失败：{error}"))?;
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return Err("截图结果为空，请稍后重试".to_string());
    }

    let png_bytes = rgba_image_to_png_bytes(&image)?;
    let encoded_image = general_purpose::STANDARD.encode(png_bytes);
    let png_data_url = format!("data:image/png;base64,{encoded_image}");

    Ok(MonitorScreenshotPayload {
        width,
        height,
        monitor_x,
        monitor_y,
        monitor_name,
        png_data_url,
    })
}

pub fn capture_selected_region(
    selection: RegionMenuSelection,
) -> Result<CapturedRegionImage, String> {
    let monitor = Monitor::from_point(selection.screen_x, selection.screen_y)
        .map_err(|error| format!("定位截图显示器失败：{error}"))?;
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
    let monitor_name = monitor
        .friendly_name()
        .or_else(|_| monitor.name())
        .unwrap_or_else(|_| "当前显示器".to_string());

    let (relative_x, relative_y, capture_width, capture_height) = region_to_monitor_relative(
        selection,
        monitor_x,
        monitor_y,
        monitor_width,
        monitor_height,
    )?;

    let image = monitor
        .capture_region(relative_x, relative_y, capture_width, capture_height)
        .map_err(|error| format!("读取截图区域失败：{error}"))?;
    let bmp_bytes = rgba_image_to_bmp_bytes(&image)?;
    let png_bytes = rgba_image_to_png_bytes(&image)?;

    Ok(CapturedRegionImage {
        width: capture_width,
        height: capture_height,
        monitor_name,
        png_bytes,
        bmp_bytes,
    })
}

fn region_to_monitor_relative(
    selection: RegionMenuSelection,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
) -> Result<(u32, u32, u32, u32), String> {
    let relative_x = u32::try_from(selection.screen_x - monitor_x)
        .map_err(|_| "截图区域超出显示器范围".to_string())?;
    let relative_y = u32::try_from(selection.screen_y - monitor_y)
        .map_err(|_| "截图区域超出显示器范围".to_string())?;
    let capture_width = selection
        .width
        .min(monitor_width.saturating_sub(relative_x));
    let capture_height = selection
        .height
        .min(monitor_height.saturating_sub(relative_y));
    if capture_width < MIN_REGION_SIZE as u32 || capture_height < MIN_REGION_SIZE as u32 {
        return Err("截图区域过小，请重新框选".to_string());
    }

    Ok((relative_x, relative_y, capture_width, capture_height))
}

pub fn copy_captured_region_to_clipboard(
    image: &CapturedRegionImage,
) -> Result<CaptureClipboardResult, String> {
    copy_bmp_bytes_to_clipboard(&image.bmp_bytes)?;

    Ok(CaptureClipboardResult {
        width: image.width,
        height: image.height,
        monitor_name: image.monitor_name.clone(),
        clipboard_format: CLIPBOARD_FORMAT,
        message: format!(
            "已复制区域截图：{}x{}（{}）",
            image.width, image.height, image.monitor_name
        ),
    })
}

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
    if width == 0 || height == 0 {
        return Err("截图结果为空，请稍后重试".to_string());
    }

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

    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8.into(),
        )
        .map_err(|_| "截图图片编码失败，请稍后重试".to_string())?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_coordinates_are_converted_to_monitor_relative() {
        let selection = RegionMenuSelection {
            action: RegionMenuAction::Copy,
            screen_x: 1920,
            screen_y: 200,
            width: 400,
            height: 100,
        };

        let converted =
            region_to_monitor_relative(selection, 1920, 0, 1920, 1080).expect("valid region");

        assert_eq!(converted, (0, 200, 400, 100));

        let clamped =
            region_to_monitor_relative(selection, 1920, 0, 1920, 260).expect("valid region");

        assert_eq!(clamped, (0, 200, 400, 60));
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
