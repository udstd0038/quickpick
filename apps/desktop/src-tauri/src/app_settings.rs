pub use quickpick_core::app_settings::{AppSettings, normalize_settings_for_save};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::Shortcut;
use quickpick_core::repository::{AppPaths, SettingsRepository};
fn repository(app: &AppHandle) -> Result<SettingsRepository, String> {
    let dir = app.path().app_config_dir().map_err(|_| "i18n:settings.locateFailed".to_string())?;
    Ok(SettingsRepository::new(AppPaths { config_dir: dir }))
}
pub fn load_app_settings(app: &AppHandle) -> Result<AppSettings, String> { repository(app)?.load() }
pub fn save_app_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> { repository(app)?.save(settings) }
pub fn parse_hotkey(value: &str, _: &str) -> Result<Shortcut, String> {
    value.parse().map_err(|_| "i18n:hotkey.invalid".to_string())
}
