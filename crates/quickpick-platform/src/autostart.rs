use quickpick_core::localized_error::{self, error_key};

pub fn snapshot() -> Result<Option<(u32, Vec<u8>)>, String> {
    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{HKEY_CURRENT_USER, RRF_NOEXPAND, RRF_RT_ANY, RegGetValueW},
    };
    let path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let name = wide_null("QuickPick");
    let mut kind = 0;
    let mut len = 0;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            name.as_ptr(),
            RRF_RT_ANY | RRF_NOEXPAND,
            &mut kind,
            std::ptr::null_mut(),
            &mut len,
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if status != ERROR_SUCCESS {
        return Err(error_key("autostart.openFailed"));
    }
    let mut data = vec![0; len as usize];
    if unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            name.as_ptr(),
            RRF_RT_ANY | RRF_NOEXPAND,
            &mut kind,
            data.as_mut_ptr().cast(),
            &mut len,
        )
    } != ERROR_SUCCESS
    {
        return Err(error_key("autostart.openFailed"));
    }
    data.truncate(len as usize);
    Ok(Some((kind, data)))
}
pub fn restore(snapshot: Option<(u32, Vec<u8>)>) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, RegCloseKey,
            RegCreateKeyExW, RegSetValueExW,
        },
    };
    let Some((kind, data)) = snapshot else {
        return disable_autostart();
    };
    let path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let name = wide_null("QuickPick");
    let mut key = std::ptr::null_mut();
    if unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            std::ptr::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    } != ERROR_SUCCESS
    {
        return Err(error_key("autostart.openFailed"));
    }
    let status = unsafe {
        RegSetValueExW(
            key,
            name.as_ptr(),
            0,
            kind,
            data.as_ptr(),
            data.len() as u32,
        )
    };
    unsafe {
        RegCloseKey(key);
    }
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(error_key("autostart.writeFailed"))
    }
}
pub fn apply_autostart_setting(enabled: bool) -> Result<(), String> {
    if enabled {
        enable_autostart()
    } else {
        disable_autostart()
    }
}

#[cfg(windows)]
fn enable_autostart() -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
            RegCreateKeyExW, RegSetValueExW,
        },
    };

    let current_exe = std::env::current_exe().map_err(|_| error_key("autostart.locateFailed"))?;
    let command = format!("\"{}\"", current_exe.to_string_lossy());
    let key_path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = wide_null("QuickPick");
    let value_data = wide_null(&command);
    let mut key = std::ptr::null_mut();

    let create_status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            key_path.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    if create_status != ERROR_SUCCESS {
        return Err(localized_error::error_key_with_detail(
            "autostart.openFailed",
            &create_status.to_string(),
        ));
    }

    let set_status = unsafe {
        RegSetValueExW(
            key,
            value_name.as_ptr(),
            0,
            REG_SZ,
            value_data.as_ptr() as *const u8,
            (value_data.len() * std::mem::size_of::<u16>()) as u32,
        )
    };
    unsafe {
        RegCloseKey(key);
    }

    if set_status != ERROR_SUCCESS {
        return Err(localized_error::error_key_with_detail(
            "autostart.writeFailed",
            &set_status.to_string(),
        ));
    }

    Ok(())
}

#[cfg(not(windows))]
fn enable_autostart() -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn disable_autostart() -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, RegCloseKey,
            RegCreateKeyExW, RegDeleteValueW,
        },
    };

    let key_path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = wide_null("QuickPick");
    let mut key = std::ptr::null_mut();

    let create_status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            key_path.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    if create_status != ERROR_SUCCESS {
        return Err(localized_error::error_key_with_detail(
            "autostart.openFailed",
            &create_status.to_string(),
        ));
    }

    let delete_status = unsafe { RegDeleteValueW(key, value_name.as_ptr()) };
    unsafe {
        RegCloseKey(key);
    }

    if delete_status != ERROR_SUCCESS && delete_status != ERROR_FILE_NOT_FOUND {
        return Err(localized_error::error_key_with_detail(
            "autostart.deleteFailed",
            &delete_status.to_string(),
        ));
    }

    Ok(())
}

#[cfg(not(windows))]
fn disable_autostart() -> Result<(), String> {
    Ok(())
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
