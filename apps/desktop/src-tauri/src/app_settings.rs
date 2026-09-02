use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

use crate::localized_error::error_key;

const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_autostart_enabled")]
    pub autostart_enabled: bool,
    #[serde(default = "default_allow_clipboard_fallback")]
    pub allow_clipboard_fallback: bool,
    #[serde(default = "default_settings_hotkey")]
    pub settings_hotkey: String,
    #[serde(default = "default_selection_hotkey")]
    pub selection_hotkey: String,
    #[serde(default = "default_screenshot_hotkey")]
    pub screenshot_hotkey: String,
    #[serde(default = "default_input_translate_hotkey")]
    pub input_translate_hotkey: String,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default = "default_window_effect")]
    pub window_effect: String,
    #[serde(default = "default_panel_opacity")]
    pub panel_opacity: u8,
    #[serde(default = "default_text_ai_provider")]
    pub text_ai_provider: String,
    #[serde(default)]
    pub text_ai_base_url: String,
    #[serde(default)]
    pub text_ai_model: String,
    #[serde(default = "default_vision_ai_provider")]
    pub vision_ai_provider: String,
    #[serde(default)]
    pub vision_ai_base_url: String,
    #[serde(default)]
    pub vision_ai_model: String,
    #[serde(default = "default_input_ai_provider")]
    pub input_ai_provider: String,
    #[serde(default)]
    pub input_ai_base_url: String,
    #[serde(default)]
    pub input_ai_model: String,
    #[serde(default = "default_input_translate_source_language")]
    pub input_translate_source_language: String,
    #[serde(default = "default_input_translate_target_language")]
    pub input_translate_target_language: String,
    #[serde(default = "default_translation_target_language")]
    pub translation_target_language: String,
    #[serde(default)]
    pub ai_timeout_seconds: u16,
    #[serde(default, skip_serializing)]
    pub ai_provider: String,
    #[serde(default, skip_serializing)]
    pub ai_base_url: String,
    #[serde(default, skip_serializing)]
    pub ai_text_model: String,
    #[serde(default, skip_serializing)]
    pub ai_vision_model: String,
}

fn default_text_ai_provider() -> String {
    "deepseek".to_string()
}

fn default_vision_ai_provider() -> String {
    "xiaomi_mimo".to_string()
}

fn default_input_ai_provider() -> String {
    "deepseek".to_string()
}

fn default_translation_target_language() -> String {
    "zh-Hans".to_string()
}

fn default_input_translate_source_language() -> String {
    "auto".to_string()
}

fn default_input_translate_target_language() -> String {
    "zh-Hans".to_string()
}

fn default_autostart_enabled() -> bool {
    true
}

fn default_allow_clipboard_fallback() -> bool {
    true
}

#[cfg(target_os = "macos")]
fn default_settings_hotkey() -> String {
    "Command+Comma".to_string()
}

#[cfg(not(target_os = "macos"))]
fn default_settings_hotkey() -> String {
    "Alt+0".to_string()
}

#[cfg(target_os = "macos")]
fn default_selection_hotkey() -> String {
    "Command+Option+2".to_string()
}

#[cfg(not(target_os = "macos"))]
fn default_selection_hotkey() -> String {
    "Alt+2".to_string()
}

#[cfg(target_os = "macos")]
fn default_screenshot_hotkey() -> String {
    "Command+Option+3".to_string()
}

#[cfg(not(target_os = "macos"))]
fn default_screenshot_hotkey() -> String {
    "Alt+3".to_string()
}

#[cfg(target_os = "macos")]
fn default_input_translate_hotkey() -> String {
    "Command+Option+4".to_string()
}

#[cfg(not(target_os = "macos"))]
fn default_input_translate_hotkey() -> String {
    "Alt+4".to_string()
}

fn default_ui_language() -> String {
    "system".to_string()
}

fn system_translation_language() -> String {
    let Some(locale) = sys_locale::get_locale() else {
        return "zh-Hans".to_string();
    };
    let locale = locale.to_ascii_lowercase();
    if locale.starts_with("zh") {
        if locale.starts_with("zh-tw") || locale.starts_with("zh-hk") || locale.starts_with("zh-mo")
        {
            "zh-Hant".to_string()
        } else {
            "zh-Hans".to_string()
        }
    } else if locale.starts_with("en") {
        "en".to_string()
    } else if locale.starts_with("ko") {
        "ko".to_string()
    } else if locale.starts_with("ja") {
        "ja".to_string()
    } else if locale.starts_with("fr") {
        "fr".to_string()
    } else if locale.starts_with("de") {
        "de".to_string()
    } else if locale.starts_with("es") {
        "es".to_string()
    } else {
        "zh-Hans".to_string()
    }
}

fn default_theme_mode() -> String {
    "system".to_string()
}

fn default_window_effect() -> String {
    "acrylic".to_string()
}

fn default_panel_opacity() -> u8 {
    100
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart_enabled: default_autostart_enabled(),
            allow_clipboard_fallback: default_allow_clipboard_fallback(),
            settings_hotkey: default_settings_hotkey(),
            selection_hotkey: default_selection_hotkey(),
            screenshot_hotkey: default_screenshot_hotkey(),
            input_translate_hotkey: default_input_translate_hotkey(),
            ui_language: default_ui_language(),
            theme_mode: default_theme_mode(),
            window_effect: default_window_effect(),
            panel_opacity: default_panel_opacity(),
            text_ai_provider: default_text_ai_provider(),
            text_ai_base_url: String::new(),
            text_ai_model: String::new(),
            vision_ai_provider: default_vision_ai_provider(),
            vision_ai_base_url: String::new(),
            vision_ai_model: String::new(),
            input_ai_provider: default_input_ai_provider(),
            input_ai_base_url: String::new(),
            input_ai_model: String::new(),
            input_translate_source_language: default_input_translate_source_language(),
            input_translate_target_language: default_input_translate_target_language(),
            translation_target_language: default_translation_target_language(),
            ai_timeout_seconds: 30,
            ai_provider: String::new(),
            ai_base_url: String::new(),
            ai_text_model: String::new(),
            ai_vision_model: String::new(),
        }
    }
}

pub fn load_app_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(&path).map_err(|_| error_key("settings.readFailed"))?;
    serde_json::from_str::<AppSettings>(&content)
        .map(normalize_settings_for_load)
        .map_err(|_| error_key("settings.invalidFormat"))
}

pub fn save_app_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let settings = normalize_settings_for_save(settings.clone())?;

    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| error_key("settings.createDirFailed"))?;
    }

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|_| error_key("settings.serializeFailed"))?;
    fs::write(path, content).map_err(|_| error_key("settings.writeFailed"))
}

pub fn normalize_settings_for_save(mut settings: AppSettings) -> Result<AppSettings, String> {
    normalize_common_fields(&mut settings);
    settings.settings_hotkey = normalize_hotkey_for_save(&settings.settings_hotkey, "设置")?;
    settings.selection_hotkey = normalize_hotkey_for_save(&settings.selection_hotkey, "划词菜单")?;
    settings.screenshot_hotkey =
        normalize_hotkey_for_save(&settings.screenshot_hotkey, "区域截图")?;
    settings.input_translate_hotkey =
        normalize_hotkey_for_save(&settings.input_translate_hotkey, "输入翻译")?;
    validate_settings(&settings)?;

    Ok(settings)
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| error_key("settings.locateFailed"))?;
    Ok(dir.join(SETTINGS_FILE_NAME))
}

fn normalize_settings_for_load(mut settings: AppSettings) -> AppSettings {
    normalize_common_fields(&mut settings);
    settings.settings_hotkey =
        normalize_hotkey_for_load(&settings.settings_hotkey, &default_settings_hotkey());
    settings.selection_hotkey =
        normalize_hotkey_for_load(&settings.selection_hotkey, &default_selection_hotkey());
    settings.screenshot_hotkey =
        normalize_hotkey_for_load(&settings.screenshot_hotkey, &default_screenshot_hotkey());
    settings.input_translate_hotkey = normalize_hotkey_for_load(
        &settings.input_translate_hotkey,
        &default_input_translate_hotkey(),
    );

    if settings.settings_hotkey == settings.selection_hotkey
        || settings.settings_hotkey == settings.screenshot_hotkey
        || settings.settings_hotkey == settings.input_translate_hotkey
        || settings.selection_hotkey == settings.screenshot_hotkey
        || settings.selection_hotkey == settings.input_translate_hotkey
        || settings.screenshot_hotkey == settings.input_translate_hotkey
    {
        settings.settings_hotkey = default_settings_hotkey();
        settings.selection_hotkey = default_selection_hotkey();
        settings.screenshot_hotkey = default_screenshot_hotkey();
        settings.input_translate_hotkey = default_input_translate_hotkey();
    }

    settings
}

fn normalize_common_fields(settings: &mut AppSettings) {
    let legacy_provider = settings.ai_provider.trim().to_string();
    if settings.text_ai_provider.trim().is_empty() && !settings.ai_provider.trim().is_empty() {
        settings.text_ai_provider = settings.ai_provider.trim().to_string();
    }
    if settings.vision_ai_provider.trim().is_empty() && !settings.ai_provider.trim().is_empty() {
        settings.vision_ai_provider = match legacy_provider.as_str() {
            "deepseek" => "deepseek".to_string(),
            value => value.to_string(),
        };
    }
    if settings.text_ai_base_url.trim().is_empty() && !settings.ai_base_url.trim().is_empty() {
        settings.text_ai_base_url = settings.ai_base_url.trim().to_string();
    }
    if legacy_provider != "deepseek"
        && settings.vision_ai_base_url.trim().is_empty()
        && !settings.ai_base_url.trim().is_empty()
    {
        settings.vision_ai_base_url = settings.ai_base_url.trim().to_string();
    }
    if settings.text_ai_model.trim().is_empty() && !settings.ai_text_model.trim().is_empty() {
        settings.text_ai_model = settings.ai_text_model.trim().to_string();
    }
    if legacy_provider == "deepseek" && settings.vision_ai_model.trim().is_empty() {
        settings.vision_ai_model = "deepseek-v4-flash-vision-exp".to_string();
    } else if legacy_provider != "deepseek"
        && settings.vision_ai_model.trim().is_empty()
        && !settings.ai_vision_model.trim().is_empty()
    {
        settings.vision_ai_model = settings.ai_vision_model.trim().to_string();
    }

    settings.text_ai_provider = normalize_text_provider(&settings.text_ai_provider);
    settings.vision_ai_provider = normalize_vision_provider(&settings.vision_ai_provider);
    settings.input_ai_provider = normalize_text_provider(&settings.input_ai_provider);
    settings.text_ai_base_url = settings.text_ai_base_url.trim().to_string();
    settings.text_ai_model = settings.text_ai_model.trim().to_string();
    settings.vision_ai_base_url = settings.vision_ai_base_url.trim().to_string();
    settings.vision_ai_model = settings.vision_ai_model.trim().to_string();
    settings.input_ai_base_url = settings.input_ai_base_url.trim().to_string();
    settings.input_ai_model = settings.input_ai_model.trim().to_string();
    settings.translation_target_language =
        normalize_translation_language(&settings.translation_target_language);
    settings.input_translate_source_language =
        normalize_translation_source_language(&settings.input_translate_source_language);
    settings.input_translate_target_language =
        normalize_translation_language(&settings.input_translate_target_language);
    settings.ui_language = match settings.ui_language.trim() {
        "zh-Hans" | "zh-Hant" | "en" | "ko" | "ja" | "fr" | "de" | "es" => {
            settings.ui_language.trim().to_string()
        }
        _ => default_ui_language(),
    };
    if settings.ui_language == "system" {
        let target = system_translation_language();
        settings.translation_target_language = target.clone();
        settings.input_translate_target_language = target;
    }
    settings.ai_provider = match settings.ai_provider.trim() {
        "deepseek" => "deepseek".to_string(),
        "xiaomi_mimo" => "xiaomi_mimo".to_string(),
        "kimi" => "kimi".to_string(),
        "glm" => "glm".to_string(),
        "minimax" => "minimax".to_string(),
        "qwen" => "qwen".to_string(),
        _ => "openai_compatible".to_string(),
    };
    settings.ai_base_url.clear();
    settings.ai_text_model.clear();
    settings.ai_vision_model.clear();
    settings.theme_mode = match settings.theme_mode.trim() {
        "light" => "light".to_string(),
        "dark" => "dark".to_string(),
        "workbench" => "dark".to_string(),
        _ => "system".to_string(),
    };
    settings.window_effect = match settings.window_effect.trim() {
        "mica" => "mica".to_string(),
        _ => "acrylic".to_string(),
    };
    settings.panel_opacity = settings.panel_opacity.clamp(30, 100);

    if settings.ai_timeout_seconds < 5 || settings.ai_timeout_seconds > 120 {
        settings.ai_timeout_seconds = AppSettings::default().ai_timeout_seconds;
    }
}

fn normalize_text_provider(value: &str) -> String {
    match value.trim() {
        "openai_compatible" => "openai_compatible".to_string(),
        "deepseek" => "deepseek".to_string(),
        "xiaomi_mimo" => "xiaomi_mimo".to_string(),
        "kimi" => "kimi".to_string(),
        "glm" => "glm".to_string(),
        "minimax" => "minimax".to_string(),
        "qwen" => "qwen".to_string(),
        _ => default_text_ai_provider(),
    }
}

fn normalize_vision_provider(value: &str) -> String {
    match value.trim() {
        "openai_compatible" => "openai_compatible".to_string(),
        "deepseek" => "deepseek".to_string(),
        "xiaomi_mimo" => "xiaomi_mimo".to_string(),
        "kimi" => "kimi".to_string(),
        "glm" => "glm".to_string(),
        "minimax" => "minimax".to_string(),
        "qwen" => "qwen".to_string(),
        _ => default_vision_ai_provider(),
    }
}

fn normalize_translation_language(value: &str) -> String {
    match value.trim() {
        "zh-Hans" => "zh-Hans".to_string(),
        "zh-Hant" => "zh-Hant".to_string(),
        "en" => "en".to_string(),
        "ja" => "ja".to_string(),
        "ko" => "ko".to_string(),
        "fr" => "fr".to_string(),
        "de" => "de".to_string(),
        "es" => "es".to_string(),
        _ => default_translation_target_language(),
    }
}

fn normalize_translation_source_language(value: &str) -> String {
    match value.trim() {
        "auto" => "auto".to_string(),
        value if normalize_translation_language(value) == value => value.to_string(),
        _ => default_input_translate_source_language(),
    }
}

fn normalize_hotkey_for_load(value: &str, fallback: &str) -> String {
    normalize_hotkey_for_save(value, "快捷键").unwrap_or_else(|_| fallback.to_string())
}

fn normalize_hotkey_for_save(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(error_key("hotkey.empty"));
    }

    let shortcut = parse_hotkey(trimmed, label)?;
    let has_required_modifier = if cfg!(target_os = "macos") {
        shortcut.mods.contains(Modifiers::ALT) || shortcut.mods.contains(Modifiers::SUPER)
    } else {
        shortcut.mods.contains(Modifiers::ALT)
    };
    if !has_required_modifier {
        return Err(error_key("hotkey.altRequired"));
    }

    Ok(format_shortcut(shortcut))
}

pub fn parse_hotkey(value: &str, _label: &str) -> Result<Shortcut, String> {
    value
        .parse::<Shortcut>()
        .map_err(|_| error_key("hotkey.invalid"))
}

fn validate_settings(settings: &AppSettings) -> Result<(), String> {
    let settings_shortcut = parse_hotkey(&settings.settings_hotkey, "设置")?;
    let selection_shortcut = parse_hotkey(&settings.selection_hotkey, "划词菜单")?;
    let screenshot_shortcut = parse_hotkey(&settings.screenshot_hotkey, "区域截图")?;
    if settings_shortcut.id() == selection_shortcut.id()
        || settings_shortcut.id() == screenshot_shortcut.id()
    {
        return Err(error_key("hotkey.settingsDuplicate"));
    }
    if selection_shortcut.id() == screenshot_shortcut.id() {
        return Err(error_key("hotkey.selectionScreenshotDuplicate"));
    }
    let input_shortcut = parse_hotkey(&settings.input_translate_hotkey, "输入翻译")?;
    if input_shortcut.id() == settings_shortcut.id()
        || input_shortcut.id() == selection_shortcut.id()
        || input_shortcut.id() == screenshot_shortcut.id()
    {
        return Err(error_key("hotkey.inputDuplicate"));
    }

    if !matches!(settings.theme_mode.as_str(), "system" | "light" | "dark") {
        return Err(error_key("settings.themeInvalid"));
    }

    if !matches!(
        settings.ui_language.as_str(),
        "system" | "zh-Hans" | "zh-Hant" | "en" | "ko" | "ja" | "fr" | "de" | "es"
    ) {
        return Err(error_key("settings.uiLanguageInvalid"));
    }

    if !matches!(settings.window_effect.as_str(), "acrylic" | "mica") {
        return Err(error_key("settings.windowEffectInvalid"));
    }

    if !(30..=100).contains(&settings.panel_opacity) {
        return Err(error_key("settings.opacityInvalid"));
    }

    validate_base_url(&settings.text_ai_base_url, "文本模型 Base URL")?;
    validate_base_url(&settings.vision_ai_base_url, "视觉模型 Base URL")?;
    validate_base_url(&settings.input_ai_base_url, "输入模型 Base URL")?;

    if settings.ai_timeout_seconds < 5 || settings.ai_timeout_seconds > 120 {
        return Err(error_key("settings.timeoutInvalid"));
    }

    if settings.text_ai_model.chars().count() > 120
        || settings.vision_ai_model.chars().count() > 120
        || settings.input_ai_model.chars().count() > 120
    {
        return Err(error_key("settings.modelTooLong"));
    }

    Ok(())
}

fn validate_base_url(value: &str, _label: &str) -> Result<(), String> {
    validate_ai_base_url(value)
}

pub fn validate_ai_base_url(value: &str) -> Result<(), String> {
    let base_url = value.trim();
    if base_url.is_empty() {
        return Ok(());
    }

    if base_url.starts_with("https://") {
        return Ok(());
    }
    if !base_url.starts_with("http://") {
        return Err(error_key("settings.baseUrlInvalid"));
    }
    if is_http_host_allowed(base_url) {
        return Ok(());
    }

    Err(error_key("settings.baseUrlInsecure"))
}

fn is_http_host_allowed(base_url: &str) -> bool {
    let rest = &base_url["http://".len()..];
    let rest = rest.split('/').next().unwrap_or("");
    let rest = rest.split('?').next().unwrap_or("");
    let rest = rest.split('#').next().unwrap_or("");
    let host_with_port = rest.rsplit('@').next().unwrap_or("");

    let host = if host_with_port.starts_with('[') {
        host_with_port
            .split_once(']')
            .map(|(host, _)| &host[1..])
            .unwrap_or("")
    } else {
        host_with_port.split(':').next().unwrap_or("")
    };

    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V4(v4) => {
                let octets = v4.octets();
                v4.is_loopback()
                    || octets[0] == 10
                    || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                    || (octets[0] == 192 && octets[1] == 168)
            }
            std::net::IpAddr::V6(v6) => v6.is_loopback(),
        };
    }

    false
}

fn format_shortcut(shortcut: Shortcut) -> String {
    let mut parts = Vec::new();

    if shortcut.mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if shortcut.mods.contains(Modifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    if shortcut.mods.contains(Modifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if shortcut.mods.contains(Modifiers::SUPER) {
        #[cfg(target_os = "macos")]
        parts.push("Command".to_string());
        #[cfg(not(target_os = "macos"))]
        parts.push("Win".to_string());
    }

    parts.push(format_code(shortcut.key));
    parts.join("+")
}

fn format_code(code: Code) -> String {
    match code {
        Code::Digit0 => "0".to_string(),
        Code::Digit1 => "1".to_string(),
        Code::Digit2 => "2".to_string(),
        Code::Digit3 => "3".to_string(),
        Code::Digit4 => "4".to_string(),
        Code::Digit5 => "5".to_string(),
        Code::Digit6 => "6".to_string(),
        Code::Digit7 => "7".to_string(),
        Code::Digit8 => "8".to_string(),
        Code::Digit9 => "9".to_string(),
        Code::Comma => "Comma".to_string(),
        Code::KeyA => "A".to_string(),
        Code::KeyB => "B".to_string(),
        Code::KeyC => "C".to_string(),
        Code::KeyD => "D".to_string(),
        Code::KeyE => "E".to_string(),
        Code::KeyF => "F".to_string(),
        Code::KeyG => "G".to_string(),
        Code::KeyH => "H".to_string(),
        Code::KeyI => "I".to_string(),
        Code::KeyJ => "J".to_string(),
        Code::KeyK => "K".to_string(),
        Code::KeyL => "L".to_string(),
        Code::KeyM => "M".to_string(),
        Code::KeyN => "N".to_string(),
        Code::KeyO => "O".to_string(),
        Code::KeyP => "P".to_string(),
        Code::KeyQ => "Q".to_string(),
        Code::KeyR => "R".to_string(),
        Code::KeyS => "S".to_string(),
        Code::KeyT => "T".to_string(),
        Code::KeyU => "U".to_string(),
        Code::KeyV => "V".to_string(),
        Code::KeyW => "W".to_string(),
        Code::KeyX => "X".to_string(),
        Code::KeyY => "Y".to_string(),
        Code::KeyZ => "Z".to_string(),
        Code::F1 => "F1".to_string(),
        Code::F2 => "F2".to_string(),
        Code::F3 => "F3".to_string(),
        Code::F4 => "F4".to_string(),
        Code::F5 => "F5".to_string(),
        Code::F6 => "F6".to_string(),
        Code::F7 => "F7".to_string(),
        Code::F8 => "F8".to_string(),
        Code::F9 => "F9".to_string(),
        Code::F10 => "F10".to_string(),
        Code::F11 => "F11".to_string(),
        Code::F12 => "F12".to_string(),
        Code::Space => "Space".to_string(),
        Code::Tab => "Tab".to_string(),
        Code::Enter => "Enter".to_string(),
        Code::Escape => "Esc".to_string(),
        Code::PrintScreen => "PrintScreen".to_string(),
        _ => code.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with_hotkeys(
        selection_hotkey: &str,
        screenshot_hotkey: &str,
        input_translate_hotkey: &str,
    ) -> AppSettings {
        AppSettings {
            selection_hotkey: selection_hotkey.to_string(),
            screenshot_hotkey: screenshot_hotkey.to_string(),
            input_translate_hotkey: input_translate_hotkey.to_string(),
            ..AppSettings::default()
        }
    }

    #[test]
    fn hotkeys_are_normalized_for_save() {
        let settings = settings_with_hotkeys(" alt + q ", "CTRL+ALT+3", "Alt+4");
        let settings = normalize_settings_for_save(settings).unwrap();

        assert_eq!(settings.selection_hotkey, "Alt+Q");
        assert_eq!(settings.screenshot_hotkey, "Ctrl+Alt+3");
        assert_eq!(settings.input_translate_hotkey, "Alt+4");
    }

    #[test]
    fn settings_hotkey_is_normalized_and_duplicates_are_rejected() {
        let settings = AppSettings {
            settings_hotkey: " alt + 0 ".to_string(),
            ..AppSettings::default()
        };
        let settings = normalize_settings_for_save(settings).unwrap();

        assert_eq!(settings.settings_hotkey, "Alt+0");

        let duplicate_hotkey = AppSettings::default().selection_hotkey;
        let settings = AppSettings {
            settings_hotkey: duplicate_hotkey,
            ..AppSettings::default()
        };

        assert!(normalize_settings_for_save(settings).is_err());
    }

    #[test]
    fn hotkeys_must_include_alt() {
        let settings = settings_with_hotkeys("Ctrl+Q", "Alt+3", "Alt+4");

        assert!(normalize_settings_for_save(settings).is_err());
    }

    #[test]
    fn hotkeys_must_not_duplicate() {
        let settings = settings_with_hotkeys("Alt+2", "alt+Digit2", "Alt+4");

        assert!(normalize_settings_for_save(settings).is_err());
    }

    #[test]
    fn input_hotkey_must_not_duplicate_other_actions() {
        let settings = settings_with_hotkeys("Alt+2", "Alt+3", "alt+Digit2");

        assert!(normalize_settings_for_save(settings).is_err());
    }

    #[test]
    fn appearance_settings_are_normalized() {
        let settings = AppSettings {
            theme_mode: "neon".to_string(),
            window_effect: "legacy_unsupported".to_string(),
            ..AppSettings::default()
        };
        let settings = normalize_settings_for_save(settings).unwrap();

        assert_eq!(settings.theme_mode, "system");
        assert_eq!(settings.window_effect, "acrylic");
        assert_eq!(settings.panel_opacity, 100);
        assert!(settings.allow_clipboard_fallback);

        let settings = AppSettings {
            theme_mode: "workbench".to_string(),
            window_effect: "mica".to_string(),
            ..AppSettings::default()
        };
        let settings = normalize_settings_for_save(settings).unwrap();

        assert_eq!(settings.theme_mode, "dark");
        assert_eq!(settings.window_effect, "mica");
    }

    #[test]
    fn ui_language_and_traditional_chinese_target_are_normalized() {
        let settings = AppSettings {
            ui_language: "en".to_string(),
            translation_target_language: "zh-Hant".to_string(),
            input_translate_target_language: "zh-Hant".to_string(),
            ..AppSettings::default()
        };
        let settings = normalize_settings_for_save(settings).unwrap();

        assert_eq!(settings.ui_language, "en");
        assert_eq!(settings.translation_target_language, "zh-Hant");
        assert_eq!(settings.input_translate_target_language, "zh-Hant");
    }

    #[test]
    fn supported_ai_providers_are_preserved() {
        for provider in [
            "openai_compatible",
            "deepseek",
            "xiaomi_mimo",
            "kimi",
            "glm",
            "minimax",
            "qwen",
        ] {
            let settings = AppSettings {
                text_ai_provider: provider.to_string(),
                ..AppSettings::default()
            };
            let settings = normalize_settings_for_save(settings).unwrap();

            assert_eq!(settings.text_ai_provider, provider);
        }
    }

    #[test]
    fn vision_provider_keeps_deepseek() {
        let settings = AppSettings {
            vision_ai_provider: "deepseek".to_string(),
            ..AppSettings::default()
        };
        let settings = normalize_settings_for_save(settings).unwrap();

        assert_eq!(settings.vision_ai_provider, "deepseek");
    }

    #[test]
    fn legacy_single_provider_is_migrated_to_split_settings() {
        let settings = AppSettings {
            ai_provider: "deepseek".to_string(),
            ai_base_url: "https://api.deepseek.com".to_string(),
            ai_text_model: "deepseek-v4-flash".to_string(),
            ai_vision_model: "deepseek-v4-flash".to_string(),
            text_ai_provider: String::new(),
            vision_ai_provider: String::new(),
            ..AppSettings::default()
        };
        let settings = normalize_settings_for_save(settings).unwrap();

        assert_eq!(settings.text_ai_provider, "deepseek");
        assert_eq!(settings.vision_ai_provider, "deepseek");
        assert_eq!(settings.text_ai_model, "deepseek-v4-flash");
        assert_eq!(settings.vision_ai_model, "deepseek-v4-flash-vision-exp");
    }

    #[test]
    fn external_http_base_url_is_rejected() {
        assert!(validate_ai_base_url("http://example.com/v1").is_err());
        assert!(normalize_settings_for_save(AppSettings {
            text_ai_base_url: "http://example.com/v1".to_string(),
            ..AppSettings::default()
        })
        .is_err());
    }

    #[test]
    fn loopback_and_private_http_base_urls_are_allowed() {
        for url in [
            "http://localhost:11434/v1",
            "http://127.0.0.1:11434/v1",
            "http://[::1]:11434/v1",
            "http://10.0.0.5:8080/v1",
            "http://172.16.1.5:8080/v1",
            "http://192.168.1.10:8080/v1",
        ] {
            assert!(
                validate_ai_base_url(url).is_ok(),
                "expected {url} to be allowed"
            );
        }
    }
}
