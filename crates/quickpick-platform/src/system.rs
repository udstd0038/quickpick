use quickpick_core::{localized_error::error_key, repository::AppPaths};
use std::{mem::size_of, path::PathBuf};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        Threading::{CreateMutexW, GetCurrentProcessId},
    },
    UI::{
        Shell::ShellExecuteW,
        WindowsAndMessaging::{
            HWND_NOTOPMOST, HWND_TOPMOST, IsIconic, SW_HIDE, SW_RESTORE, SW_SHOWNOACTIVATE,
            SW_SHOWNORMAL, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow,
            SetWindowPos, ShowWindow,
        },
    },
};
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub struct InstanceGuard(HANDLE);
impl InstanceGuard {
    pub fn acquire() -> Result<Option<Self>, String> {
        let name = wide("Local\\QuickPick.Instance.com.quickpick.desktop");
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err("无法建立单实例锁".into());
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe {
                CloseHandle(handle);
            }
            return Ok(None);
        }
        Ok(Some(Self(handle)))
    }
}
impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
pub fn old_instance_running() -> bool {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return true;
        }
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut next = Process32FirstW(snapshot, &mut entry);
        let mut found = false;
        while next != 0 {
            let n = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            if entry.th32ProcessID != GetCurrentProcessId()
                && String::from_utf16_lossy(&entry.szExeFile[..n])
                    .eq_ignore_ascii_case("quickpick.exe")
            {
                found = true;
                break;
            }
            next = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        found
    }
}
pub fn paths() -> Result<AppPaths, String> {
    use windows_sys::Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{FOLDERID_RoamingAppData, SHGetKnownFolderPath},
    };
    let mut p = std::ptr::null_mut();
    if unsafe { SHGetKnownFolderPath(&FOLDERID_RoamingAppData, 0, std::ptr::null_mut(), &mut p) }
        < 0
    {
        return Err(error_key("settings.locateFailed"));
    }
    let mut len = 0;
    unsafe {
        while *p.add(len) != 0 {
            len += 1;
        }
    }
    use std::os::windows::ffi::OsStringExt;
    let directory = unsafe { std::ffi::OsString::from_wide(std::slice::from_raw_parts(p, len)) };
    unsafe {
        CoTaskMemFree(p.cast());
    }
    Ok(AppPaths {
        config_dir: PathBuf::from(directory).join("com.quickpick.desktop"),
    })
}
pub fn hide(native: isize) {
    unsafe {
        ShowWindow(native as HWND, SW_HIDE);
    }
}
pub fn show(native: isize, focus: bool) {
    unsafe {
        ShowWindow(
            native as HWND,
            if IsIconic(native as HWND) != 0 {
                SW_RESTORE
            } else if focus {
                SW_SHOWNORMAL
            } else {
                SW_SHOWNOACTIVATE
            },
        );
        if focus {
            SetForegroundWindow(native as HWND);
        }
    }
}
pub fn pin(native: isize, pinned: bool) {
    unsafe {
        SetWindowPos(
            native as HWND,
            if pinned { HWND_TOPMOST } else { HWND_NOTOPMOST },
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}
pub fn copy_text(text: &str) -> Result<(), String> {
    clipboard_win::set_clipboard_string(text).map_err(|_| error_key("clipboard.writeFailed"))
}
pub fn search(text: &str) -> Result<(), String> {
    if text.chars().count() > 500 {
        return Err(error_key("search.tooLong"));
    }
    let mut query = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            query.push(byte as char);
        } else {
            query.push_str(&format!("%{byte:02X}"));
        }
    }
    let url = wide(&format!("https://www.bing.com/search?q={query}"));
    let verb = wide("open");
    if unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            url.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize
        <= 32
    {
        Err(error_key("search.browserFailed"))
    } else {
        Ok(())
    }
}

/// 显示器全局物理边界和有效 DPI；不读取屏幕像素。
pub fn topology() -> Result<Vec<[i64; 6]>, String> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
    };
    struct State {
        rows: Vec<[i64; 6]>,
        failed: bool,
    }
    unsafe extern "system" fn visit(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> i32 {
        let state = unsafe { &mut *(data as *mut State) };
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let mut dx = 0;
        let mut dy = 0;
        if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0
            || unsafe {
                windows_sys::Win32::UI::HiDpi::GetDpiForMonitor(monitor, 0, &mut dx, &mut dy)
            } < 0
        {
            state.failed = true;
            return 0;
        }
        state.rows.push([
            info.rcMonitor.left.into(),
            info.rcMonitor.top.into(),
            info.rcMonitor.right.into(),
            info.rcMonitor.bottom.into(),
            dx.into(),
            dy.into(),
        ]);
        1
    }
    let mut state = State {
        rows: vec![],
        failed: false,
    };
    if unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(visit),
            (&mut state as *mut State) as LPARAM,
        )
    } == 0
        || state.failed
        || state.rows.is_empty()
    {
        return Err("无法读取显示器布局".into());
    }
    state.rows.sort();
    Ok(state.rows)
}

pub fn message(text: &str) {
    let caption = wide("QuickPick");
    let text = wide(text);
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            0x10,
        );
    }
}

pub fn needs_reset(native: isize) -> bool {
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(native as HWND) == 0
            || IsIconic(native as HWND) != 0
    }
}
pub fn restore_foreground(native: isize) {
    unsafe {
        if windows_sys::Win32::UI::WindowsAndMessaging::IsWindow(native as HWND) != 0 {
            SetForegroundWindow(native as HWND);
        }
    }
}

pub fn geometry(native: isize) -> Option<quickpick_core::window_state::WindowGeometry> {
    use windows_sys::Win32::{Foundation::RECT, UI::WindowsAndMessaging::GetWindowRect};
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(native as HWND, &mut rect) } == 0 {
        return None;
    }
    Some(quickpick_core::window_state::WindowGeometry {
        x: rect.left,
        y: rect.top,
        width: u32::try_from(rect.right - rect.left).ok()?,
        height: u32::try_from(rect.bottom - rect.top).ok()?,
    })
}
pub fn restore_geometry(
    native: isize,
    geometry: quickpick_core::window_state::WindowGeometry,
    selection: bool,
) {
    use windows_sys::Win32::{
        Foundation::POINT,
        Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint},
        UI::{HiDpi::GetDpiForWindow, WindowsAndMessaging::SWP_NOZORDER},
    };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let monitor = unsafe {
        MonitorFromPoint(
            POINT {
                x: geometry.x,
                y: geometry.y,
            },
            MONITOR_DEFAULTTONEAREST,
        )
    };
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return;
    }
    unsafe {
        SetWindowPos(
            native as HWND,
            std::ptr::null_mut(),
            geometry.x,
            geometry.y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    let scale = unsafe { GetDpiForWindow(native as HWND) }.max(96) as f32 / 96.;
    let width = if selection {
        480. * scale
    } else {
        (geometry.width as f32).max(360. * scale)
    }
    .min((info.rcWork.right - info.rcWork.left) as f32) as i32;
    let height = if selection {
        58. * scale
    } else {
        (geometry.height as f32).max(300. * scale)
    }
    .min((info.rcWork.bottom - info.rcWork.top) as f32) as i32;
    let x = geometry
        .x
        .clamp(info.rcWork.left, info.rcWork.right - width);
    let y = geometry
        .y
        .clamp(info.rcWork.top, info.rcWork.bottom - height);
    unsafe {
        SetWindowPos(
            native as HWND,
            std::ptr::null_mut(),
            x,
            y,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}
