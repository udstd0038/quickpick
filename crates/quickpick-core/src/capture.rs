use crate::{region::RegionRectPx, session::ScreenshotAction};
pub trait CaptureService {
    type Image;
    fn capture_hidden(&mut self, region: RegionRectPx) -> Result<Self::Image, String>;
    fn copy(&mut self, image: &Self::Image) -> Result<(), String>;
    fn png(&mut self, image: &Self::Image) -> Result<Vec<u8>, String>;
}
/// 配置和取消检查属于执行边界，进入/拖动遮罩不调用此函数。
pub fn execute_capture<C: CaptureService>(
    action: ScreenshotAction,
    region: RegionRectPx,
    preflight: impl FnOnce() -> Result<(), String>,
    cancelled: impl Fn() -> bool,
    service: &mut C,
) -> Result<Option<Vec<u8>>, String> {
    if cancelled() {
        return Ok(None);
    }
    if action != ScreenshotAction::Copy {
        preflight()?;
    }
    if cancelled() {
        return Ok(None);
    }
    let image = service.capture_hidden(region)?;
    if cancelled() {
        return Ok(None);
    }
    match action {
        ScreenshotAction::Copy => {
            service.copy(&image)?;
            Ok(None)
        }
        _ => service.png(&image).map(Some),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::{MonitorPx, RectDip};
    #[derive(Default)]
    struct Fake {
        capture: usize,
        copy: usize,
        png: usize,
    }
    impl CaptureService for Fake {
        type Image = ();
        fn capture_hidden(&mut self, _: RegionRectPx) -> Result<(), String> {
            self.capture += 1;
            Ok(())
        }
        fn copy(&mut self, _: &()) -> Result<(), String> {
            self.copy += 1;
            Ok(())
        }
        fn png(&mut self, _: &()) -> Result<Vec<u8>, String> {
            self.png += 1;
            Ok(vec![1])
        }
    }
    fn rect() -> RegionRectPx {
        RectDip {
            left: 0.,
            top: 0.,
            right: 10.,
            bottom: 10.,
        }
        .to_physical(
            MonitorPx {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            1.,
        )
        .unwrap()
    }
    #[test]
    fn invalid_vision_configuration_never_captures() {
        let mut f = Fake::default();
        assert!(
            execute_capture(
                ScreenshotAction::Translate,
                rect(),
                || Err("missing".into()),
                || false,
                &mut f
            )
            .is_err()
        );
        assert_eq!(f.capture, 0);
    }
    #[test]
    fn cancelled_session_never_captures() {
        let mut f = Fake::default();
        execute_capture(ScreenshotAction::Copy, rect(), || Ok(()), || true, &mut f).unwrap();
        assert_eq!(f.capture, 0);
    }
    #[test]
    fn copy_never_checks_ai_or_encodes_png() {
        let mut f = Fake::default();
        execute_capture(
            ScreenshotAction::Copy,
            rect(),
            || panic!("AI invoked"),
            || false,
            &mut f,
        )
        .unwrap();
        assert_eq!((f.capture, f.copy, f.png), (1, 1, 0));
    }
    #[test]
    fn ai_never_writes_clipboard() {
        let mut f = Fake::default();
        assert_eq!(
            execute_capture(
                ScreenshotAction::Extract,
                rect(),
                || Ok(()),
                || false,
                &mut f
            )
            .unwrap(),
            Some(vec![1])
        );
        assert_eq!((f.capture, f.copy, f.png), (1, 0, 1));
    }
}
