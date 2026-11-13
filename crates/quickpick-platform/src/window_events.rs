use std::mem::size_of;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Dwm::{DWMWA_SYSTEMBACKDROP_TYPE, DwmSetWindowAttribute},
    UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            SIZE_MINIMIZED, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_ENTERSIZEMOVE, WM_EXITSIZEMOVE,
            WM_NCDESTROY, WM_POWERBROADCAST, WM_SIZE,
        },
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    CancelCapture,
    RestoreAppearance(isize),
    BoundsChanged(isize),
}
struct Hook {
    resize_edges: bool,
    callback: Box<dyn Fn(Notice)>,
}
const ID: usize = 0x51504943;
/// 同一 HWND 的子类回调只持有消息发送闭包，销毁时释放，不替换 GPUI 的窗口过程。
pub fn install(native: isize, callback: impl Fn(Notice) + 'static) -> Result<(), String> {
    install_with_edges(native, true, callback)
}
pub fn install_overlay(native: isize, callback: impl Fn(Notice) + 'static) -> Result<(), String> {
    install_with_edges(native, false, callback)
}
fn install_with_edges(
    native: isize,
    resize_edges: bool,
    callback: impl Fn(Notice) + 'static,
) -> Result<(), String> {
    let state = Box::into_raw(Box::new(Hook {
        resize_edges,
        callback: Box::new(callback),
    }));
    if unsafe { SetWindowSubclass(native as HWND, Some(callback_proc), ID, state as usize) } == 0 {
        unsafe {
            drop(Box::from_raw(state));
        }
        return Err("窗口事件初始化失败".into());
    }
    unsafe {
        windows_sys::Win32::System::RemoteDesktop::WTSRegisterSessionNotification(
            native as HWND,
            0,
        );
    }
    Ok(())
}
unsafe extern "system" fn callback_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    reference: usize,
) -> LRESULT {
    let state = unsafe { &*(reference as *const Hook) };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, WM_NCHITTEST, WM_NCLBUTTONDBLCLK, WM_NCLBUTTONDOWN,
    };
    if state.resize_edges && native_resize_enabled(hwnd) {
        if msg == WM_NCHITTEST {
            if let Some(edge) = native_resize_hit(hwnd, lparam) {
                return edge as LRESULT;
            }
        } else if matches!(msg, WM_NCLBUTTONDOWN | WM_NCLBUTTONDBLCLK)
            && (10..=17).contains(&wparam)
        {
            // GPUI's Windows backend does not implement start_window_resize.
            // Let Windows handle resize presses instead of forwarding them to
            // client controls whose mouse handlers could consume the press.
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
    }
    match msg {
        // Only enqueue a notice. GPUI reads the backend's final size after this
        // native message returns; never re-enter its window from the subclass.
        WM_SIZE if wparam != SIZE_MINIMIZED as usize => {
            (state.callback)(Notice::BoundsChanged(hwnd as isize))
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED => (state.callback)(Notice::CancelCapture),
        WM_POWERBROADCAST if wparam == 4 => (state.callback)(Notice::CancelCapture),
        0x02b1 if wparam == 7 => (state.callback)(Notice::CancelCapture),
        WM_ENTERSIZEMOVE => clear_backdrops(hwnd as isize),
        WM_EXITSIZEMOVE => (state.callback)(Notice::RestoreAppearance(hwnd as isize)),
        WM_NCDESTROY => unsafe {
            windows_sys::Win32::System::RemoteDesktop::WTSUnRegisterSessionNotification(hwnd);
            RemoveWindowSubclass(hwnd, Some(callback_proc), ID);
            drop(Box::from_raw(reference as *mut Hook));
        },
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
fn native_resize_enabled(hwnd: HWND) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetWindowLongPtrW, IsIconic, IsZoomed, WS_THICKFRAME,
    };
    unsafe {
        IsZoomed(hwnd) == 0
            && IsIconic(hwnd) == 0
            && (GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_THICKFRAME) != 0
    }
}
fn native_resize_hit(hwnd: HWND, packed_point: LPARAM) -> Option<i32> {
    use quickpick_core::{
        window_frame::{FrameEdge, resize_edge},
        window_state::WindowGeometry,
    };
    use windows_sys::Win32::{
        Foundation::RECT,
        UI::{HiDpi::GetDpiForWindow, WindowsAndMessaging::*},
    };
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return None;
    }
    let frame = WindowGeometry {
        x: rect.left,
        y: rect.top,
        width: u32::try_from(i64::from(rect.right) - i64::from(rect.left)).ok()?,
        height: u32::try_from(i64::from(rect.bottom) - i64::from(rect.top)).ok()?,
    };
    let x = (packed_point as u32 as u16 as i16) as i32;
    let y = ((packed_point as u32 >> 16) as u16 as i16) as i32;
    let scale = unsafe { GetDpiForWindow(hwnd) }.max(96) as f32 / 96.;
    Some(match resize_edge(frame, x, y, scale)? {
        FrameEdge::Left => HTLEFT,
        FrameEdge::Right => HTRIGHT,
        FrameEdge::Top => HTTOP,
        FrameEdge::Bottom => HTBOTTOM,
        FrameEdge::TopLeft => HTTOPLEFT,
        FrameEdge::TopRight => HTTOPRIGHT,
        FrameEdge::BottomLeft => HTBOTTOMLEFT,
        FrameEdge::BottomRight => HTBOTTOMRIGHT,
    } as i32)
}
/// Mica (DWM) and Acrylic (accent policy) must not remain enabled together.
/// Used before a native effect transition and for the temporary drag fallback.
pub fn clear_backdrops(native: isize) {
    let none = 1i32;
    unsafe {
        DwmSetWindowAttribute(
            native as HWND,
            DWMWA_SYSTEMBACKDROP_TYPE as u32,
            (&none as *const i32).cast(),
            size_of::<i32>() as u32,
        );
    }
    disable_accent(native as HWND);
}
fn disable_accent(hwnd: HWND) {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    #[repr(C)]
    struct Accent {
        state: i32,
        flags: i32,
        color: u32,
        animation: i32,
    }
    #[repr(C)]
    struct Attribute {
        attribute: i32,
        data: *mut std::ffi::c_void,
        size: usize,
    }
    let module: Vec<u16> = "user32.dll".encode_utf16().chain(Some(0)).collect();
    unsafe {
        if let Some(address) = GetProcAddress(
            GetModuleHandleW(module.as_ptr()),
            c"SetWindowCompositionAttribute".as_ptr().cast(),
        ) {
            let function: unsafe extern "system" fn(HWND, *mut Attribute) -> i32 =
                std::mem::transmute(address);
            let mut accent = Accent {
                state: 0,
                flags: 0,
                color: 0,
                animation: 0,
            };
            let mut data = Attribute {
                attribute: 19,
                data: (&mut accent as *mut Accent).cast(),
                size: size_of::<Accent>(),
            };
            function(hwnd, &mut data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use windows_sys::Win32::{
        Foundation::HMODULE,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, SIZE_MAXIMIZED,
            SIZE_RESTORED, SendMessageW, UnregisterClassW, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        },
    };
    struct HiddenWindow {
        hwnd: HWND,
        module: HMODULE,
        class: Vec<u16>,
    }
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            unsafe {
                if !self.hwnd.is_null() {
                    DestroyWindow(self.hwnd);
                }
                UnregisterClassW(self.class.as_ptr(), self.module);
            }
        }
    }
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    unsafe extern "system" fn fixture_proc(
        hwnd: HWND,
        msg: u32,
        wp: WPARAM,
        lp: LPARAM,
    ) -> LRESULT {
        if msg == windows_sys::Win32::UI::WindowsAndMessaging::WM_NCHITTEST {
            return 1;
        }
        unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
    }
    fn hidden_window() -> HiddenWindow {
        // A hidden, process-owned fixture only: no desktop capture or clipboard.
        let class: Vec<u16> = format!(
            "QuickPickWindowTest{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
        .encode_utf16()
        .chain(Some(0))
        .collect();
        let module = unsafe { GetModuleHandleW(std::ptr::null()) };
        let definition = WNDCLASSW {
            lpfnWndProc: Some(fixture_proc),
            hInstance: module,
            lpszClassName: class.as_ptr(),
            ..Default::default()
        };
        assert_ne!(unsafe { RegisterClassW(&definition) }, 0);
        let mut fixture = HiddenWindow {
            hwnd: std::ptr::null_mut(),
            module,
            class,
        };
        fixture.hwnd = unsafe {
            CreateWindowExW(
                0,
                fixture.class.as_ptr(),
                fixture.class.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                160,
                120,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                module,
                std::ptr::null(),
            )
        };
        assert!(!fixture.hwnd.is_null());
        fixture
    }
    #[test]
    fn restored_and_maximized_size_messages_notify_without_minimized_sizes() {
        let fixture = hidden_window();
        let notices = Rc::new(RefCell::new(Vec::new()));
        let received = notices.clone();
        install(fixture.hwnd as isize, move |notice| {
            received.borrow_mut().push(notice)
        })
        .unwrap();
        let dimensions = (120isize << 16) | 160;
        unsafe {
            SendMessageW(fixture.hwnd, WM_SIZE, SIZE_RESTORED as usize, dimensions);
        }
        assert_eq!(
            &*notices.borrow(),
            &[Notice::BoundsChanged(fixture.hwnd as isize)]
        );
        unsafe {
            SendMessageW(fixture.hwnd, WM_SIZE, SIZE_MINIMIZED as usize, 0);
        }
        assert_eq!(notices.borrow().len(), 1);
        unsafe {
            SendMessageW(fixture.hwnd, WM_SIZE, SIZE_MAXIMIZED as usize, dimensions);
        }
        assert_eq!(
            &*notices.borrow(),
            &[Notice::BoundsChanged(fixture.hwnd as isize); 2]
        );
    }
    fn hit(hwnd: HWND, x: i32, y: i32) -> LRESULT {
        let packed = ((y as u16 as u32) << 16) | x as u16 as u32;
        unsafe {
            SendMessageW(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::WM_NCHITTEST,
                0,
                packed as LPARAM,
            )
        }
    }
    #[test]
    fn native_hook_exposes_all_eight_resize_directions() {
        use windows_sys::Win32::{Foundation::RECT, UI::WindowsAndMessaging::*};
        let fixture = hidden_window();
        install(fixture.hwnd as isize, |_| {}).unwrap();
        let mut r = RECT::default();
        assert_ne!(unsafe { GetWindowRect(fixture.hwnd, &mut r) }, 0);
        let mx = (r.left + r.right) / 2;
        let my = (r.top + r.bottom) / 2;
        for (x, y, expected) in [
            (r.left + 1, r.top + 1, HTTOPLEFT),
            (r.right - 2, r.top + 1, HTTOPRIGHT),
            (r.left + 1, r.bottom - 2, HTBOTTOMLEFT),
            (r.right - 2, r.bottom - 2, HTBOTTOMRIGHT),
            (r.left + 1, my, HTLEFT),
            (r.right - 2, my, HTRIGHT),
            (mx, r.top + 1, HTTOP),
            (mx, r.bottom - 2, HTBOTTOM),
        ] {
            assert_eq!(hit(fixture.hwnd, x, y), expected as LRESULT);
        }
        assert_eq!(hit(fixture.hwnd, mx, my), HTCLIENT as LRESULT);
    }
    #[test]
    fn overlay_hook_does_not_claim_resize_edges() {
        let fixture = hidden_window();
        install_overlay(fixture.hwnd as isize, |_| {}).unwrap();
        // Fixture procedure returns client even with WS_THICKFRAME: the overlay
        // hook must leave hit testing entirely to its existing capture backend.
        let mut rect = windows_sys::Win32::Foundation::RECT::default();
        assert_ne!(
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(fixture.hwnd, &mut rect)
            },
            0
        );
        assert_eq!(hit(fixture.hwnd, rect.left + 1, rect.top + 1), 1);
    }
}
