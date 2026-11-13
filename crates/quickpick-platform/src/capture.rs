use quickpick_core::{
    capture::CaptureService,
    region::{MonitorPx, RegionRectPx},
};
pub struct WindowsCapture {
    pub native: isize,
    pub monitor: MonitorPx,
    pub topology: Vec<[i64; 6]>,
}
impl CaptureService for WindowsCapture {
    type Image = image::RgbaImage;
    fn capture_hidden(&mut self, region: RegionRectPx) -> Result<Self::Image, String> {
        if crate::system::topology()? != self.topology {
            return Err("显示器布局发生变化".into());
        }
        crate::probe::capture_after_hide(self.native, self.monitor, region).map_err(str::to_owned)
    }
    fn copy(&mut self, image: &Self::Image) -> Result<(), String> {
        crate::probe::copy_image(image).map_err(str::to_owned)
    }
    fn png(&mut self, image: &Self::Image) -> Result<Vec<u8>, String> {
        use image::ImageEncoder;
        let mut png = vec![];
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|_| "i18n:ai.imageEmpty".to_string())?;
        Ok(png)
    }
}
