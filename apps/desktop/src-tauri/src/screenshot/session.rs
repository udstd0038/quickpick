use image::RgbaImage;
use zeroize::Zeroize;

use super::{MonitorScreenshotPayload, RegionMenuSelection};

pub(super) const MIN_REGION_SIZE: u32 = 8;
pub(super) const MAX_RGBA_BYTES: usize = 128 * 1024 * 1024;
pub(super) const MAX_PNG_BYTES: usize = 64 * 1024 * 1024;
const DATA_URL_PREFIX_BYTES: usize = "data:image/png;base64,".len();
const MAX_PREVIEW_BYTES: usize = ((MAX_PNG_BYTES + 2) / 3) * 4 + DATA_URL_PREFIX_BYTES;
// Session IDs cross the JavaScript IPC boundary as numbers.
const MAX_SESSION_ID: u64 = (1_u64 << 53) - 1;

pub(super) fn validate_session_id(id: u64) -> Result<(), String> {
    if id == 0 || id > MAX_SESSION_ID {
        return Err("截图会话标识无效，请重新截图".to_string());
    }
    Ok(())
}

pub(super) fn rgba_size(width: u32, height: u32) -> Result<usize, String> {
    if width == 0 || height == 0 {
        return Err("截图结果为空，请稍后重试".to_string());
    }
    let bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .filter(|bytes| *bytes <= MAX_RGBA_BYTES)
        .ok_or_else(|| "截图尺寸超过内存限制，请选择较小的显示器或分辨率".to_string())?;
    Ok(bytes)
}

/// Owns one captured frame and its preview. Never clone or serialize the full frame.
pub struct ScreenshotSession {
    payload: MonitorScreenshotPayload,
    pixels: RgbaImage,
}

impl ScreenshotSession {
    pub(crate) fn new(
        payload: MonitorScreenshotPayload,
        pixels: RgbaImage,
    ) -> Result<Self, String> {
        let session = Self { payload, pixels };
        validate_session_id(session.id())?;
        if session.payload.monitor_width == 0 || session.payload.monitor_height == 0 {
            return Err("显示器尺寸无效，请重新截图".to_string());
        }
        let bytes = rgba_size(session.pixels.width(), session.pixels.height())?;
        if session.payload.width != session.pixels.width()
            || session.payload.height != session.pixels.height()
            || session.pixels.as_raw().len() != bytes
        {
            return Err("截图像素尺寸不一致，请重新截图".to_string());
        }
        if session.payload.png_data_url.len() > MAX_PREVIEW_BYTES
            || !session
                .payload
                .png_data_url
                .starts_with("data:image/png;base64,")
        {
            return Err("截图预览超过内存限制或格式无效".to_string());
        }
        Ok(session)
    }

    pub fn id(&self) -> u64 {
        self.payload.session_id
    }

    pub fn payload(&self) -> &MonitorScreenshotPayload {
        &self.payload
    }

    pub(super) fn into_crop(mut self, selection: RegionMenuSelection) -> Result<RgbaImage, String> {
        if selection.session_id != Some(self.id()) {
            return Err("截图会话已失效，请重新截图".to_string());
        }
        if selection.width < MIN_REGION_SIZE || selection.height < MIN_REGION_SIZE {
            return Err("截图区域过小，请重新框选".to_string());
        }
        let right = selection
            .x
            .checked_add(selection.width)
            .ok_or_else(|| "截图区域超出快照范围".to_string())?;
        let bottom = selection
            .y
            .checked_add(selection.height)
            .ok_or_else(|| "截图区域超出快照范围".to_string())?;
        if right > self.pixels.width() || bottom > self.pixels.height() {
            return Err("截图区域超出快照范围，不支持跨显示器框选".to_string());
        }

        if selection.x == 0
            && selection.y == 0
            && selection.width == self.pixels.width()
            && selection.height == self.pixels.height()
        {
            // Move a full-frame selection without duplicating the largest buffer.
            return Ok(std::mem::replace(&mut self.pixels, RgbaImage::new(0, 0)));
        }
        Ok(image::imageops::crop_imm(
            &self.pixels,
            selection.x,
            selection.y,
            selection.width,
            selection.height,
        )
        .to_image())
        // The full frame and preview are erased on return, before ROI encoding.
    }
}

impl Drop for ScreenshotSession {
    fn drop(&mut self) {
        let bytes: &mut [u8] = self.pixels.as_mut();
        bytes.zeroize();
        self.payload.png_data_url.zeroize();
    }
}

/// Pure state machine; the caller supplies synchronization and performs OS work.
#[derive(Default)]
pub struct ScreenshotStore {
    generation: u64,
    active_id: Option<u64>,
    capturing: bool,
    session: Option<ScreenshotSession>,
    error: Option<String>,
}

impl ScreenshotStore {
    pub fn begin(&mut self) -> Result<u64, String> {
        let next = self
            .generation
            .checked_add(1)
            .filter(|id| *id <= MAX_SESSION_ID)
            .ok_or_else(|| "截图会话计数已耗尽，请重新启动 QuickPick".to_string())?;
        self.cancel_all();
        self.generation = next;
        self.active_id = Some(next);
        self.capturing = true;
        Ok(next)
    }

    /// Returns true only when this completion belongs to the pending request.
    pub fn complete(&mut self, id: u64, result: Result<ScreenshotSession, String>) -> bool {
        if !self.capturing || self.active_id != Some(id) {
            // Any supplied stale session is dropped and erased here.
            return false;
        }
        self.capturing = false;
        match result {
            Ok(session) if session.id() == id => self.session = Some(session),
            Ok(_) => self.error = Some("截图会话标识不一致，请重新截图".to_string()),
            Err(error) => self.error = Some(error),
        }
        true
    }

    pub fn peek_payload(&self) -> Result<Option<MonitorScreenshotPayload>, String> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(self
            .session
            .as_ref()
            .map(|session| session.payload().clone()))
    }

    pub fn take_matching(&mut self, id: Option<u64>) -> Result<ScreenshotSession, String> {
        if id.is_none() || id != self.active_id {
            return Err("截图会话已失效，请重新截图".to_string());
        }
        if self.capturing {
            return Err("截图尚未完成，请稍后再试".to_string());
        }
        let session = self.session.take().ok_or_else(|| {
            self.error
                .clone()
                .unwrap_or_else(|| "没有可用的截图快照".to_string())
        })?;
        self.active_id = None;
        self.error = None;
        Ok(session)
    }

    pub fn cancel_matching(&mut self, id: u64) -> bool {
        if self.active_id != Some(id) {
            return false;
        }
        self.cancel_all();
        true
    }

    pub fn cancel_all(&mut self) {
        self.session = None;
        self.error = None;
        self.active_id = None;
        self.capturing = false;
    }

    pub fn active_id(&self) -> Option<u64> {
        self.active_id
    }

    pub fn is_capturing(&self) -> bool {
        self.capturing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screenshot::RegionMenuAction;

    fn session(id: u64, scale: u32) -> ScreenshotSession {
        let width = 16 * scale;
        let height = 12 * scale;
        let pixels = RgbaImage::from_fn(width, height, |x, y| {
            image::Rgba([x as u8, y as u8, 47, 255])
        });
        ScreenshotSession::new(
            MonitorScreenshotPayload {
                session_id: id,
                monitor_id: 7,
                width,
                height,
                monitor_x: -16,
                monitor_y: -12,
                monitor_width: 16,
                monitor_height: 12,
                monitor_name: "Synthetic monitor".to_string(),
                png_data_url: "data:image/png;base64,AAAA".to_string(),
            },
            pixels,
        )
        .unwrap()
    }

    fn selection(id: Option<u64>, x: u32, y: u32, width: u32, height: u32) -> RegionMenuSelection {
        RegionMenuSelection {
            action: RegionMenuAction::Copy,
            session_id: id,
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn crops_snapshot_pixels_at_one_and_two_times_scale() {
        for scale in [1, 2] {
            let cropped = session(1, scale)
                .into_crop(selection(
                    Some(1),
                    8 * scale,
                    4 * scale,
                    8 * scale,
                    8 * scale,
                ))
                .unwrap();
            assert_eq!(cropped.dimensions(), (8 * scale, 8 * scale));
            assert_eq!(
                cropped.get_pixel(0, 0).0,
                [(8 * scale) as u8, (4 * scale) as u8, 47, 255]
            );
            assert_eq!(
                cropped.get_pixel(8 * scale - 1, 8 * scale - 1).0,
                [(16 * scale - 1) as u8, (12 * scale - 1) as u8, 47, 255]
            );
        }
    }

    #[test]
    fn rejects_small_out_of_bounds_overflow_and_missing_or_wrong_session() {
        for request in [
            selection(Some(1), 0, 0, 7, 8),
            selection(Some(1), 9, 0, 8, 8),
            selection(Some(1), 0, 5, 8, 8),
            selection(Some(1), u32::MAX, 0, 8, 8),
            selection(Some(1), 0, u32::MAX, 8, 8),
            selection(None, 0, 0, 8, 8),
            selection(Some(2), 0, 0, 8, 8),
        ] {
            assert!(session(1, 1).into_crop(request).is_err());
        }
    }

    #[test]
    fn whole_snapshot_selection_preserves_the_owned_buffer() {
        let original = session(1, 1);
        let original_ptr = original.pixels.as_raw().as_ptr();
        let cropped = original
            .into_crop(selection(Some(1), 0, 0, 16, 12))
            .unwrap();
        assert_eq!(cropped.as_raw().as_ptr(), original_ptr);
        assert_eq!(cropped.get_pixel(15, 11).0, [15, 11, 47, 255]);
    }

    #[test]
    fn dimension_budget_and_metadata_are_validated_without_large_allocations() {
        assert!(rgba_size(0, 1).is_err());
        assert!(rgba_size(u32::MAX, u32::MAX).is_err());
        assert!(rgba_size(16_384, 16_384).is_err());
        assert_eq!(rgba_size(7_680, 4_320).unwrap(), 132_710_400);
        let pixels = RgbaImage::new(8, 8);
        let mut payload = session(1, 1).payload().clone();
        payload.width = 8;
        assert!(ScreenshotSession::new(payload, pixels).is_err());
    }

    #[test]
    fn newer_capture_rejects_old_success_failure_and_cancel() {
        let mut store = ScreenshotStore::default();
        let old = store.begin().unwrap();
        let current = store.begin().unwrap();
        assert!(current > old);
        assert!(!store.complete(old, Ok(session(old, 1))));
        assert!(!store.complete(old, Err("old error".to_string())));
        assert!(!store.cancel_matching(old));
        assert!(store.is_capturing());
        assert_eq!(store.active_id(), Some(current));
        assert!(store.complete(current, Ok(session(current, 2))));
        assert!(!store.is_capturing());
        assert_eq!(store.peek_payload().unwrap().unwrap().session_id, current);
        assert!(store.take_matching(Some(old)).is_err());
        assert!(store.take_matching(None).is_err());
        assert_eq!(store.take_matching(Some(current)).unwrap().id(), current);
        assert!(store.peek_payload().unwrap().is_none());
        assert_eq!(store.active_id(), None);
        assert!(!store.complete(current, Ok(session(current, 1))));
    }

    #[test]
    fn cancel_pending_capture_rejects_late_completion() {
        let mut store = ScreenshotStore::default();
        let id = store.begin().unwrap();
        assert!(store.cancel_matching(id));
        assert!(!store.is_capturing());
        assert!(!store.complete(id, Ok(session(id, 1))));
        assert!(!store.complete(id, Err("late error".to_string())));
        assert!(store.peek_payload().unwrap().is_none());
        let next = store.begin().unwrap();
        assert!(next > id);
        store.cancel_all();
        assert!(!store.complete(next, Ok(session(next, 1))));
    }

    #[test]
    fn failure_is_visible_and_next_capture_clears_it() {
        let mut store = ScreenshotStore::default();
        let id = store.begin().unwrap();
        assert!(store.take_matching(Some(id)).is_err());
        assert!(store.complete(id, Err("capture failed".to_string())));
        assert_eq!(store.peek_payload().err().unwrap(), "capture failed");
        assert!(store.take_matching(Some(id)).is_err());
        assert!(!store.complete(id, Ok(session(id, 1))));
        let next = store.begin().unwrap();
        assert!(store.peek_payload().unwrap().is_none());
        assert!(store.complete(next, Ok(session(next, 1))));
        assert!(store.cancel_matching(next));
        assert!(store.peek_payload().unwrap().is_none());
    }

    #[test]
    fn mismatched_result_is_rejected_and_generation_never_wraps() {
        let mut store = ScreenshotStore::default();
        let id = store.begin().unwrap();
        assert!(store.complete(id, Ok(session(id + 1, 1))));
        assert!(store.peek_payload().is_err());
        store.generation = MAX_SESSION_ID;
        assert!(store.begin().is_err());
        assert_eq!(store.generation, MAX_SESSION_ID);
    }
}
