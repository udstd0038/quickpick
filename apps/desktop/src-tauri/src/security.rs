use quickpick_core::repository::AppPaths;
use tauri::{AppHandle,Manager};
pub use quickpick_platform::security::ApiKeyStatus;
fn paths(app:&AppHandle)->Result<AppPaths,String>{Ok(AppPaths{config_dir:app.path().app_config_dir().map_err(|_|"i18n:security.locateFailed".to_string())?})}
pub fn api_key_status(app:&AppHandle,scope:&str)->Result<ApiKeyStatus,String>{quickpick_platform::security::api_key_status(&paths(app)?,scope)}
pub fn save_api_key(app:&AppHandle,scope:&str,value:&str)->Result<(),String>{quickpick_platform::security::save_api_key(&paths(app)?,scope,value)}
pub fn clear_api_key(app:&AppHandle,scope:&str)->Result<(),String>{quickpick_platform::security::clear_api_key(&paths(app)?,scope)}
pub fn load_api_key(app:&AppHandle,scope:&str)->Result<Option<zeroize::Zeroizing<String>>,String>{quickpick_platform::security::load_api_key(&paths(app)?,scope)}
