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
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::key_status(scope).map(|configured| ApiKeyStatus { configured });
    }
    let path = api_key_path(app, scope)?;
    let legacy_path = legacy_api_key_path(app)?;
    Ok(ApiKeyStatus {
        configured: key_file_configured(&path)
            || (normalize_scope(scope) == "text" && key_file_configured(&legacy_path)),
    })
}

#[cfg(target_os = "macos")]
pub fn api_key_status(app: &AppHandle, scope: &str) -> Result<ApiKeyStatus, String> {
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::key_status(scope).map(|configured| ApiKeyStatus { configured });
    }
    let _ = app;
    let entry = mac_keyring_entry(scope)?;
    mac_api_key_status_from_store(&entry)
}

#[cfg(not(target_os = "macos"))]
pub fn save_api_key(app: &AppHandle, scope: &str, api_key: &str) -> Result<(), String> {
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::set_key(scope, !api_key.trim().is_empty());
    }
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
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::set_key(scope, !api_key.trim().is_empty());
    }
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
        .map_err(|error| mac_keyring_error("security.saveFailed", error))
}

#[cfg(not(target_os = "macos"))]
pub fn clear_api_key(app: &AppHandle, scope: &str) -> Result<(), String> {
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::set_key(scope, false);
    }
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
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::set_key(scope, false);
    }
    let _ = app;
    let entry = mac_keyring_entry(scope)?;
    clear_mac_api_key_from_store(&entry)
}

#[cfg(not(target_os = "macos"))]
pub fn load_api_key(app: &AppHandle, scope: &str) -> Result<Option<Zeroizing<String>>, String> {
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::key_status(scope).map(|configured| configured.then(|| Zeroizing::new("synthetic-fixture-key".into())));
    }
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
    #[cfg(feature = "dev-tools")]
    if crate::dev_fixture::active() {
        return crate::dev_fixture::key_status(scope).map(|configured| configured.then(|| Zeroizing::new("synthetic-fixture-key".into())));
    }
    let _ = app;
    let entry = mac_keyring_entry(scope)?;
    load_mac_api_key_from_store(&entry)
}

#[cfg(target_os = "macos")]
trait MacKeyStore {
    fn get_password(&self) -> keyring::Result<String>;
    fn delete_credential(&self) -> keyring::Result<()>;
}

#[cfg(target_os = "macos")]
impl MacKeyStore for keyring::Entry {
    fn get_password(&self) -> keyring::Result<String> {
        keyring::Entry::get_password(self)
    }

    fn delete_credential(&self) -> keyring::Result<()> {
        keyring::Entry::delete_credential(self)
    }
}

#[cfg(target_os = "macos")]
fn mac_api_key_status_from_store(store: &impl MacKeyStore) -> Result<ApiKeyStatus, String> {
    // The temporary password is zeroized when it is dropped after this check.
    Ok(ApiKeyStatus {
        configured: load_mac_api_key_from_store(store)?.is_some(),
    })
}

#[cfg(target_os = "macos")]
fn load_mac_api_key_from_store(
    store: &impl MacKeyStore,
) -> Result<Option<Zeroizing<String>>, String> {
    match store.get_password() {
        Ok(value) => {
            let value = Zeroizing::new(value);
            Ok((!value.is_empty()).then_some(value))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(mac_keyring_error("security.readFailed", error)),
    }
}

#[cfg(target_os = "macos")]
fn clear_mac_api_key_from_store(store: &impl MacKeyStore) -> Result<(), String> {
    // Delete directly: a read can be denied even when deletion is permitted.
    match store.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(mac_keyring_error("security.clearFailed", error)),
    }
}

#[cfg(target_os = "macos")]
fn mac_keyring_error(key: &str, mut error: keyring::Error) -> String {
    error_key_with_detail(key, mac_keyring_error_detail(&mut error))
}

#[cfg(target_os = "macos")]
fn mac_keyring_error_detail(error: &mut keyring::Error) -> &'static str {
    // Backend Display/Debug implementations can include credential data.
    // Return fixed categories and scrub malformed plaintext before dropping it.
    match error {
        keyring::Error::PlatformFailure(_) => "keychain backend failure",
        keyring::Error::NoStorageAccess(_) => "keychain access denied or locked",
        keyring::Error::NoEntry => "keychain entry missing",
        keyring::Error::BadEncoding(bytes) => {
            bytes.zeroize();
            "keychain data is not valid UTF-8"
        }
        keyring::Error::BadDataFormat(bytes, _) => {
            bytes.zeroize();
            "keychain data format invalid"
        }
        keyring::Error::BadStoreFormat(reason) => {
            reason.zeroize();
            "keychain store format invalid"
        }
        keyring::Error::TooLong(name, _) => {
            name.zeroize();
            "keychain value too long"
        }
        keyring::Error::Invalid(attribute, reason) => {
            attribute.zeroize();
            reason.zeroize();
            "keychain parameter invalid"
        }
        keyring::Error::Ambiguous(_) => "multiple keychain entries matched",
        keyring::Error::NoDefaultStore => "keychain store unavailable",
        keyring::Error::NotSupportedByStore(reason) => {
            reason.zeroize();
            "keychain operation unsupported"
        }
        _ => "keychain operation failed",
    }
}

#[cfg(target_os = "macos")]
fn mac_keyring_entry(scope: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new("com.quickpick.desktop", &normalize_scope(scope))
        .map_err(|error| mac_keyring_error("security.readFailed", error))
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

#[cfg(all(test, target_os = "macos"))]
mod mac_tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    struct FakeKeyStore {
        password: RefCell<Option<keyring::Result<String>>>,
        deletion: RefCell<Option<keyring::Result<()>>>,
        reads: Cell<usize>,
        deletes: Cell<usize>,
    }

    impl FakeKeyStore {
        fn new(password: keyring::Result<String>, deletion: keyring::Result<()>) -> Self {
            Self {
                password: RefCell::new(Some(password)),
                deletion: RefCell::new(Some(deletion)),
                reads: Cell::new(0),
                deletes: Cell::new(0),
            }
        }
    }

    impl MacKeyStore for FakeKeyStore {
        fn get_password(&self) -> keyring::Result<String> {
            self.reads.set(self.reads.get() + 1);
            self.password
                .borrow_mut()
                .take()
                .expect("one read expected")
        }

        fn delete_credential(&self) -> keyring::Result<()> {
            self.deletes.set(self.deletes.get() + 1);
            self.deletion
                .borrow_mut()
                .take()
                .expect("one delete expected")
        }
    }

    fn access_denied() -> keyring::Error {
        keyring::Error::NoStorageAccess(Box::new(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "keychain access denied",
        )))
    }

    fn assert_read_failure(make_error: impl Fn() -> keyring::Error) {
        let store = FakeKeyStore::new(Err(make_error()), Ok(()));
        let error = mac_api_key_status_from_store(&store)
            .err()
            .expect("status must report the read failure");
        assert!(error.starts_with("i18n:security.readFailed:"));

        let store = FakeKeyStore::new(Err(make_error()), Ok(()));
        let error = load_mac_api_key_from_store(&store)
            .err()
            .expect("load must report the read failure");
        assert!(error.starts_with("i18n:security.readFailed:"));
    }

    #[test]
    fn missing_entry_is_unconfigured_and_can_be_loaded_as_none() {
        let store = FakeKeyStore::new(Err(keyring::Error::NoEntry), Ok(()));
        assert!(!mac_api_key_status_from_store(&store).unwrap().configured);

        let store = FakeKeyStore::new(Err(keyring::Error::NoEntry), Ok(()));
        assert!(load_mac_api_key_from_store(&store).unwrap().is_none());
    }

    #[test]
    fn nonempty_entry_is_configured_and_loadable() {
        let store = FakeKeyStore::new(Ok("test-keychain-key".to_string()), Ok(()));
        assert!(mac_api_key_status_from_store(&store).unwrap().configured);

        let store = FakeKeyStore::new(Ok("test-keychain-key".to_string()), Ok(()));
        let value = load_mac_api_key_from_store(&store).unwrap().unwrap();
        assert_eq!(value.as_str(), "test-keychain-key");
    }

    #[test]
    fn empty_entry_is_unconfigured_and_loads_none() {
        let store = FakeKeyStore::new(Ok(String::new()), Ok(()));
        assert!(!mac_api_key_status_from_store(&store).unwrap().configured);

        let store = FakeKeyStore::new(Ok(String::new()), Ok(()));
        assert!(load_mac_api_key_from_store(&store).unwrap().is_none());
    }

    #[test]
    fn access_denied_is_not_reported_as_a_missing_entry() {
        assert_read_failure(access_denied);
    }

    #[test]
    fn backend_failure_is_not_reported_as_a_missing_entry() {
        assert_read_failure(|| {
            keyring::Error::PlatformFailure(Box::new(std::io::Error::other(
                "keychain backend unavailable",
            )))
        });
    }

    #[test]
    fn clear_attempts_deletion_even_when_reading_would_be_denied() {
        let store = FakeKeyStore::new(Err(access_denied()), Ok(()));
        assert!(clear_mac_api_key_from_store(&store).is_ok());
        assert_eq!(store.reads.get(), 0);
        assert_eq!(store.deletes.get(), 1);
    }

    #[test]
    fn clear_deletes_an_empty_entry_without_reading_it() {
        let store = FakeKeyStore::new(Ok(String::new()), Ok(()));
        assert!(clear_mac_api_key_from_store(&store).is_ok());
        assert_eq!(store.reads.get(), 0);
        assert_eq!(store.deletes.get(), 1);
    }

    #[test]
    fn clearing_a_missing_entry_is_successful() {
        let store = FakeKeyStore::new(Err(access_denied()), Err(keyring::Error::NoEntry));
        assert!(clear_mac_api_key_from_store(&store).is_ok());
        assert_eq!(store.reads.get(), 0);
        assert_eq!(store.deletes.get(), 1);
    }

    #[test]
    fn denied_deletion_does_not_report_success() {
        let store = FakeKeyStore::new(Err(access_denied()), Err(access_denied()));
        let error = clear_mac_api_key_from_store(&store)
            .err()
            .expect("denied deletion must fail");
        assert!(error.starts_with("i18n:security.clearFailed:"));
        assert_eq!(store.reads.get(), 0);
        assert_eq!(store.deletes.get(), 1);
    }

    #[test]
    fn errors_do_not_expose_backend_or_invalid_payload_details() {
        const TEST_KEY: &str = "test-keychain-sensitive-key";
        for operation in [
            "security.readFailed",
            "security.saveFailed",
            "security.clearFailed",
        ] {
            let errors = [
                keyring::Error::PlatformFailure(Box::new(std::io::Error::other(TEST_KEY))),
                keyring::Error::NoStorageAccess(Box::new(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    TEST_KEY,
                ))),
                keyring::Error::BadEncoding(TEST_KEY.as_bytes().to_vec()),
                keyring::Error::BadDataFormat(
                    TEST_KEY.as_bytes().to_vec(),
                    Box::new(std::io::Error::other(TEST_KEY)),
                ),
                keyring::Error::BadStoreFormat(TEST_KEY.to_string()),
                keyring::Error::TooLong(TEST_KEY.to_string(), 0),
                keyring::Error::Invalid(TEST_KEY.to_string(), TEST_KEY.to_string()),
                keyring::Error::NotSupportedByStore(TEST_KEY.to_string()),
            ];
            for error in errors {
                let output = mac_keyring_error(operation, error);
                assert!(output.starts_with(&format!("i18n:{operation}:")));
                assert!(!output.contains(TEST_KEY));
            }
        }
    }

    #[test]
    fn malformed_password_bytes_are_zeroized_before_error_is_discarded() {
        let mut errors = [
            keyring::Error::BadEncoding(b"test-keychain-key".to_vec()),
            keyring::Error::BadDataFormat(
                b"test-keychain-key".to_vec(),
                Box::new(std::io::Error::other("invalid data")),
            ),
        ];
        for error in &mut errors {
            let _ = mac_keyring_error_detail(error);
            match error {
                keyring::Error::BadEncoding(bytes) | keyring::Error::BadDataFormat(bytes, _) => {
                    assert!(bytes.is_empty());
                }
                _ => unreachable!(),
            }
        }
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
