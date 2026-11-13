use crate::{
    app_settings::{AppSettings, normalize_settings_for_load, normalize_settings_for_save},
    localized_error::error_key,
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
}
#[derive(Clone)]
pub struct SettingsRepository {
    pub paths: AppPaths,
}
impl SettingsRepository {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
    pub fn load(&self) -> Result<AppSettings, String> {
        let path = self.paths.config_dir.join("settings.json");
        match fs::read(path) {
            Ok(bytes) => serde_json::from_slice::<AppSettings>(&bytes)
                .map(normalize_settings_for_load)
                .map_err(|_| error_key("settings.invalidFormat")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(AppSettings::default())
            }
            Err(_) => Err(error_key("settings.readFailed")),
        }
    }
    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        let settings = normalize_settings_for_save(settings.clone())?;
        let bytes = serde_json::to_vec_pretty(&settings)
            .map_err(|_| error_key("settings.serializeFailed"))?;
        atomic_write(&self.paths.config_dir.join("settings.json"), &bytes)
            .map_err(|_| error_key("settings.writeFailed"))
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().ok_or(std::io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".quickpick-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrupt_configuration_is_not_overwritten() {
        let dir =
            std::env::temp_dir().join(format!("quickpick-config-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        fs::write(&path, b"broken").unwrap();
        let repository = SettingsRepository::new(AppPaths {
            config_dir: dir.clone(),
        });
        assert!(repository.load().is_err());
        assert_eq!(fs::read(&path).unwrap(), b"broken");
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn settings_roundtrip_preserves_domains_and_replaces_atomically() {
        let dir =
            std::env::temp_dir().join(format!("quickpick-roundtrip-test-{}", std::process::id()));
        let repository = SettingsRepository::new(AppPaths {
            config_dir: dir.clone(),
        });
        let mut settings = AppSettings {
            text_ai_model: "text-model".into(),
            input_ai_model: "input-model".into(),
            ..AppSettings::default()
        };
        repository.save(&settings).unwrap();
        settings.vision_ai_model = "vision-model".into();
        repository.save(&settings).unwrap();
        let loaded = repository.load().unwrap();
        assert_eq!(loaded.text_ai_model, "text-model");
        assert_eq!(loaded.vision_ai_model, "vision-model");
        assert_eq!(loaded.input_ai_model, "input-model");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
}
