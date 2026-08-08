#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeTheme {
    Light,
    Dark,
}

impl NativeTheme {
    pub fn from_theme_mode(theme_mode: &str) -> Self {
        match theme_mode.trim() {
            "light" => Self::Light,
            "dark" => Self::Dark,
            "workbench" => Self::Dark,
            _ => system_theme(),
        }
    }

    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}

impl Default for NativeTheme {
    fn default() -> Self {
        system_theme()
    }
}

#[cfg(windows)]
fn system_theme() -> NativeTheme {
    use std::{mem::size_of, ptr::null_mut};
    use windows_sys::Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ,
        },
    };

    let path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let name = wide_null("AppsUseLightTheme");
    let mut key = null_mut();

    let open_status =
        unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, KEY_READ, &mut key) };
    if open_status != ERROR_SUCCESS || key.is_null() {
        return NativeTheme::Light;
    }

    let mut value: u32 = 1;
    let mut value_size = size_of::<u32>() as u32;
    let query_status = unsafe {
        RegQueryValueExW(
            key,
            name.as_ptr(),
            null_mut(),
            null_mut(),
            &mut value as *mut u32 as *mut u8,
            &mut value_size,
        )
    };

    unsafe {
        RegCloseKey(key);
    }

    if query_status == ERROR_SUCCESS && value == 0 {
        NativeTheme::Dark
    } else {
        NativeTheme::Light
    }
}

#[cfg(not(windows))]
fn system_theme() -> NativeTheme {
    NativeTheme::Light
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
