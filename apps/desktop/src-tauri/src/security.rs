use serde::Serialize;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};
use zeroize::{Zeroize, Zeroizing};

use crate::localized_error::error_key;
#[cfg(target_os = "macos")]
use crate::localized_error::error_key_with_detail;

#[cfg(windows)]
use windows::{
    core::w,
    Win32::{
        Foundation::{LocalFree, HLOCAL},
        Security::Cryptography::{
            CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        },
    },
};

const API_KEY_FILE_NAME: &str = "api_key.dpapi";
const TEXT_API_KEY_FILE_NAME: &str = "text_api_key.dpapi";
const VISION_API_KEY_FILE_NAME: &str = "vision_api_key.dpapi";
const INPUT_API_KEY_FILE_NAME: &str = "input_api_key.dpapi";
const MAX_API_KEY_BYTES: usize = 8 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyStatus {
    pub configured: bool,
}

#[cfg(not(target_os = "macos"))]
pub fn api_key_status(app: &AppHandle, scope: &str) -> Result<ApiKeyStatus, String> {
    let path = api_key_path(app, scope)?;
    let legacy_path = legacy_api_key_path(app)?;
    Ok(ApiKeyStatus {
        configured: key_file_configured(&path)
            || (normalize_scope(scope) == "text" && key_file_configured(&legacy_path)),
    })
}

#[cfg(target_os = "macos")]
pub fn api_key_status(app: &AppHandle, scope: &str) -> Result<ApiKeyStatus, String> {
    let _ = app;
    let entry = mac_keyring_entry(scope)?;
    Ok(ApiKeyStatus {
        configured: match entry.get_password() {
            Ok(value) => !value.is_empty(),
            Err(_) => false,
        },
    })
}

#[cfg(not(target_os = "macos"))]
pub fn save_api_key(app: &AppHandle, scope: &str, api_key: &str) -> Result<(), String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(error_key("security.apiKeyEmpty"));
    }

    if trimmed.len() > MAX_API_KEY_BYTES {
        return Err(error_key("security.apiKeyTooLong"));
    }

    let mut plaintext = Zeroizing::new(trimmed.as_bytes().to_vec());
    let encrypted = protect_data(&plaintext)?;
    plaintext.zeroize();

    let path = api_key_path(app, scope)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| error_key("security.createDirFailed"))?;
    }

    fs::write(path, encrypted).map_err(|_| error_key("security.saveFailed"))
}

#[cfg(target_os = "macos")]
pub fn save_api_key(app: &AppHandle, scope: &str, api_key: &str) -> Result<(), String> {
    let _ = app;
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(error_key("security.apiKeyEmpty"));
    }
    if trimmed.len() > MAX_API_KEY_BYTES {
        return Err(error_key("security.apiKeyTooLong"));
    }

    let entry = mac_keyring_entry(scope)?;
    entry
        .set_password(trimmed)
        .map_err(|error| error_key_with_detail("security.saveFailed", &error.to_string()))
}

#[cfg(not(target_os = "macos"))]
pub fn clear_api_key(app: &AppHandle, scope: &str) -> Result<(), String> {
    let path = api_key_path(app, scope)?;
    if path.exists() {
        fs::remove_file(path).map_err(|_| error_key("security.clearFailed"))?;
    }
    if normalize_scope(scope) == "text" {
        let legacy_path = legacy_api_key_path(app)?;
        if legacy_path.exists() {
            fs::remove_file(legacy_path).map_err(|_| error_key("security.clearLegacyFailed"))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn clear_api_key(app: &AppHandle, scope: &str) -> Result<(), String> {
    let _ = app;
    let entry = mac_keyring_entry(scope)?;
    let configured = matches!(
        entry.get_password(),
        Ok(value) if !value.is_empty()
    );
    if !configured {
        return Ok(());
    }
    entry
        .delete_credential()
        .map_err(|error| error_key_with_detail("security.clearFailed", &error.to_string()))
}

#[cfg(not(target_os = "macos"))]
pub fn load_api_key(app: &AppHandle, scope: &str) -> Result<Option<Zeroizing<String>>, String> {
    let path = api_key_path(app, scope)?;
    let legacy_path = legacy_api_key_path(app)?;
    let path = if path.exists() {
        path
    } else if normalize_scope(scope) == "text" && legacy_path.exists() {
        legacy_path
    } else {
        path
    };
    if !path.exists() {
        return Ok(None);
    }

    let encrypted = fs::read(path).map_err(|_| error_key("security.readFailed"))?;
    let plaintext = unprotect_data(&encrypted)?;
    let value = match String::from_utf8(plaintext) {
        Ok(value) => Zeroizing::new(value),
        Err(error) => {
            let mut bytes = error.into_bytes();
            bytes.zeroize();
            return Err(error_key("security.decryptInvalid"));
        }
    };

    Ok(Some(value))
}

#[cfg(target_os = "macos")]
pub fn load_api_key(app: &AppHandle, scope: &str) -> Result<Option<Zeroizing<String>>, String> {
    let _ = app;
    let entry = mac_keyring_entry(scope)?;
    match entry.get_password() {
        Ok(value) if !value.is_empty() => Ok(Some(Zeroizing::new(value))),
        Ok(_) => Ok(None),
        Err(error) => Err(error_key_with_detail(
            "security.readFailed",
            &error.to_string(),
        )),
    }
}

#[cfg(target_os = "macos")]
fn mac_keyring_entry(scope: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new("com.quickpick.desktop", &normalize_scope(scope))
        .map_err(|error| error_key_with_detail("security.readFailed", &error.to_string()))
}

fn api_key_path(app: &AppHandle, scope: &str) -> Result<PathBuf, String> {
    let file_name = api_key_file_name(scope);
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| error_key("security.locateFailed"))?;
    Ok(dir.join(file_name))
}

fn api_key_file_name(scope: &str) -> &'static str {
    match normalize_scope(scope).as_str() {
        "vision" => VISION_API_KEY_FILE_NAME,
        "input" => INPUT_API_KEY_FILE_NAME,
        _ => TEXT_API_KEY_FILE_NAME,
    }
}

fn legacy_api_key_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| error_key("security.locateFailed"))?;
    Ok(dir.join(API_KEY_FILE_NAME))
}

fn normalize_scope(scope: &str) -> String {
    match scope.trim() {
        "vision" => "vision".to_string(),
        "input" => "input".to_string(),
        _ => "text".to_string(),
    }
}

fn key_file_configured(path: &PathBuf) -> bool {
    path.exists() && path.metadata().map(|meta| meta.len()).unwrap_or(0) > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_scope_maps_to_isolated_dpapi_files() {
        assert_eq!(api_key_file_name("text"), "text_api_key.dpapi");
        assert_eq!(api_key_file_name("vision"), "vision_api_key.dpapi");
        assert_eq!(api_key_file_name("input"), "input_api_key.dpapi");
        assert_eq!(api_key_file_name("unknown"), "text_api_key.dpapi");
    }
}

#[cfg(windows)]
fn protect_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: data
            .len()
            .try_into()
            .map_err(|_| error_key("security.apiKeyDataTooLong"))?,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();

    // DPAPI returns a LocalAlloc buffer that must be copied and released with LocalFree.
    unsafe {
        CryptProtectData(
            &input,
            w!("QuickPick API Key"),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|_| error_key("security.encryptFailed"))?;

        copy_blob_and_free(output)
    }
}

#[cfg(windows)]
fn unprotect_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: data
            .len()
            .try_into()
            .map_err(|_| error_key("security.apiKeyDataTooLong"))?,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();

    // DPAPI returns a LocalAlloc buffer that must be copied and released with LocalFree.
    unsafe {
        CryptUnprotectData(
            &input,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
        .map_err(|_| error_key("security.decryptFailed"))?;

        copy_blob_and_free(output)
    }
}

#[cfg(windows)]
unsafe fn copy_blob_and_free(blob: CRYPT_INTEGER_BLOB) -> Result<Vec<u8>, String> {
    if blob.pbData.is_null() || blob.cbData == 0 {
        return Err(error_key("security.dpapiEmpty"));
    }

    let bytes = unsafe { std::slice::from_raw_parts(blob.pbData, blob.cbData as usize) }.to_vec();
    let _ = unsafe { LocalFree(Some(HLOCAL(blob.pbData.cast()))) };
    Ok(bytes)
}

#[cfg(not(windows))]
fn protect_data(_data: &[u8]) -> Result<Vec<u8>, String> {
    Err(error_key("security.unsupportedPlatform"))
}

#[cfg(not(windows))]
fn unprotect_data(_data: &[u8]) -> Result<Vec<u8>, String> {
    Err(error_key("security.unsupportedPlatform"))
}
