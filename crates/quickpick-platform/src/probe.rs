use quickpick_core::region::{MonitorPx, RegionRectPx};
use std::{
    mem::size_of,
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{HWND, POINT},
    Graphics::{
        Dwm::DwmFlush,
        Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint},
    },
    System::StationsAndDesktops::{
        CloseDesktop, DESKTOP_READOBJECTS, GetUserObjectInformationW, OpenInputDesktop, UOI_NAME,
    },
    UI::Input::KeyboardAndMouse::{GetCapture, ReleaseCapture},
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetPhysicalCursorPos, GetWindowLongPtrW, HWND_TOPMOST, IsWindowVisible,
        SW_HIDE, SW_SHOW, SW_SHOWNOACTIVATE, SWP_FRAMECHANGED, SetWindowLongPtrW, SetWindowPos,
        ShowWindow, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    },
};

pub fn monitor_at_cursor() -> Result<MonitorPx, &'static str> {
    let mut point = POINT { x: 0, y: 0 };
    // 光标不可用时落在主显示器；不读取屏幕像素。
    unsafe {
        GetPhysicalCursorPos(&mut point);
    }
    monitor_at(point)
}

fn monitor_at(point: POINT) -> Result<MonitorPx, &'static str> {
    let handle = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTOPRIMARY) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetMonitorInfoW(handle, &mut info) } == 0 {
        return Err("读取显示器边界失败");
    }
    let rect = info.rcMonitor;
    Ok(MonitorPx {
        x: rect.left,
        y: rect.top,
        width: u32::try_from(i64::from(rect.right) - i64::from(rect.left))
            .map_err(|_| "显示器宽度无效")?,
        height: u32::try_from(i64::from(rect.bottom) - i64::from(rect.top))
            .map_err(|_| "显示器高度无效")?,
    })
}

pub fn monitor_unchanged(monitor: MonitorPx) -> bool {
    monitor_at(POINT {
        x: monitor.x,
        y: monitor.y,
    })
    .is_ok_and(|current| current == monitor)
}

/// 只接受普通交互桌面，无法确认时拒绝捕获；不切换桌面或提升权限。
pub fn is_normal_desktop() -> bool {
    unsafe {
        let desktop = OpenInputDesktop(0, 0, DESKTOP_READOBJECTS);
        if desktop.is_null() {
            return false;
        }
        let mut name = [0u16; 256];
        let mut needed = 0;
        let ok = GetUserObjectInformationW(
            desktop,
            UOI_NAME,
            name.as_mut_ptr().cast(),
            size_of_val(&name) as u32,
            &mut needed,
        );
        CloseDesktop(desktop);
        if ok == 0 {
            return false;
        }
        let len = name
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(name.len());
        String::from_utf16_lossy(&name[..len]).eq_ignore_ascii_case("default")
    }
}

pub fn has_pointer_capture(native: isize) -> bool {
    unsafe { GetCapture() == native as HWND }
}

pub fn release_pointer(native: isize) {
    if has_pointer_capture(native) {
        unsafe {
            ReleaseCapture();
        }
    }
}

/// 必须在 GPUI UI 线程上调用；不更改 renderer 的合成/alpha 模式。
/// `native` 来自同一窗口的 raw-window-handle。
pub fn show_overlay(native: isize, monitor: MonitorPx) -> Result<(), &'static str> {
    let hwnd = native as HWND;
    let width = i32::try_from(monitor.width).map_err(|_| "显示器宽度无效")?;
    let height = i32::try_from(monitor.height).map_err(|_| "显示器高度无效")?;
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        // 透明中心仍需接收鼠标；不得添加 WS_EX_TRANSPARENT。
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            ((ex | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW & !WS_EX_TRANSPARENT) as isize,
        );
        if SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            monitor.x,
            monitor.y,
            width,
            height,
            SWP_FRAMECHANGED,
        ) == 0
        {
            return Err("定位遮罩窗口失败");
        }
        ShowWindow(hwnd, SW_SHOW);
    }
    Ok(())
}

pub fn hide_overlay(native: isize) {
    unsafe {
        ShowWindow(native as HWND, SW_HIDE);
    }
}

pub fn show_probe(native: isize) {
    unsafe {
        ShowWindow(native as HWND, SW_SHOWNOACTIVATE);
    }
}

/// 只在用户点击样机的“复制选区”后，由单个后台任务调用。
/// 仅返回非敏感状态；不创建 PNG/Base64、文件或网络请求。
pub fn copy_after_hide(
    native: isize,
    monitor: MonitorPx,
    region: RegionRectPx,
) -> Result<(), &'static str> {
    let image = capture_after_hide(native, monitor, region)?;
    copy_image(&image)
}

pub fn capture_after_hide(
    native: isize,
    monitor: MonitorPx,
    region: RegionRectPx,
) -> Result<image::RgbaImage, &'static str> {
    let deadline = Instant::now() + Duration::from_millis(500);
    while unsafe { IsWindowVisible(native as HWND) } != 0 {
        if Instant::now() >= deadline {
            return Err("遮罩隐藏超时，未捕获");
        }
        thread::sleep(Duration::from_millis(10));
    }
    if unsafe { DwmFlush() } < 0 {
        return Err("桌面合成同步失败，未捕获");
    }
    thread::sleep(Duration::from_millis(80));
    if Instant::now() >= deadline
        || unsafe { IsWindowVisible(native as HWND) } != 0
        || !monitor_unchanged(monitor)
        || !is_normal_desktop()
    {
        return Err("窗口或显示器状态变化，未捕获");
    }
    let x = u32::try_from(i64::from(region.x()) - i64::from(monitor.x)).map_err(|_| "选区越界")?;
    let y = u32::try_from(i64::from(region.y()) - i64::from(monitor.y)).map_err(|_| "选区越界")?;
    if u64::from(x) + u64::from(region.width()) > u64::from(monitor.width)
        || u64::from(y) + u64::from(region.height()) > u64::from(monitor.height)
    {
        return Err("选区越界");
    }
    let source =
        xcap::Monitor::from_point(region.x(), region.y()).map_err(|_| "定位捕获显示器失败")?;
    if source.x().ok() != Some(monitor.x)
        || source.y().ok() != Some(monitor.y)
        || source.width().ok() != Some(monitor.width)
        || source.height().ok() != Some(monitor.height)
    {
        return Err("捕获显示器已变化");
    }
    let image = source
        .capture_region(x, y, region.width(), region.height())
        .map_err(|_| "捕获区域失败")?;
    Ok(image)
}
pub fn copy_image(image: &image::RgbaImage) -> Result<(), &'static str> {
    let bmp = crate::bitmap::encode_clipboard_bmp(image)?;
    let dib = bmp.get(14..).ok_or("剪贴板图片格式无效")?;
    let _clipboard = clipboard_win::Clipboard::new_attempts(10).map_err(|_| "打开剪贴板失败")?;
    clipboard_win::raw::empty().map_err(|_| "清空剪贴板失败")?;
    clipboard_win::raw::set_without_clear(clipboard_win::formats::CF_DIB, dib)
        .map_err(|_| "写入图片失败")?;
    clipboard_win::raw::set_bitmap_with(&bmp, clipboard_win::options::NoClear)
        .map_err(|_| "写入位图失败")
}
