use crate::ui::*;
use quickpick_core::app_settings::AppSettings;
use std::collections::BTreeMap;
#[derive(Clone)]
pub enum Control {
    Text(Entity<InputState>),
    Select(Entity<ChoiceState>),
    Toggle(bool),
    Slider(Entity<SliderState>),
}
pub struct SettingsForm {
    pub controls: BTreeMap<String, Control>,
    pub category: usize,
    pub status: String,
    pub key_inputs: [Entity<InputState>; 3],
    pub recording: Option<String>,
    pub _subscriptions: Vec<Subscription>,
}
const CATEGORIES: [&str; 7] = [
    "settings.general",
    "settings.hotkeys",
    "settings.aiSelection",
    "settings.aiScreenshot",
    "settings.aiInput",
    "settings.appearance",
    "settings.privacy",
];
impl SettingsForm {
    pub fn new(
        settings: &AppSettings,
        display_language: &str,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let data = serde_json::to_value(settings).expect("settings");
        let mut controls = BTreeMap::new();
        for (key, value) in data.as_object().unwrap() {
            let control =
                if let Some(choices) = crate::settings_choices::choices(key, display_language) {
                    Control::Select(select(
                        choices,
                        value.as_str().unwrap_or_default(),
                        window,
                        cx,
                    ))
                } else if let Some(value) = value.as_bool() {
                    Control::Toggle(value)
                } else if key == "panelOpacity" {
                    Control::Slider(cx.new(|_| {
                        SliderState::new()
                            .min(30.)
                            .max(100.)
                            .default_value(value.as_f64().unwrap_or(100.) as f32)
                    }))
                } else {
                    Control::Text(cx.new(|cx| {
                        let mut state = InputState::new(window, cx);
                        state.set_value(
                            value
                                .as_str()
                                .map(str::to_owned)
                                .unwrap_or_else(|| value.to_string()),
                            window,
                            cx,
                        );
                        state
                    }))
                };
            controls.insert(key.clone(), control);
        }
        Self {
            controls,
            category: 0,
            status: String::new(),
            recording: None,
            _subscriptions: vec![],
            key_inputs: std::array::from_fn(|_| {
                cx.new(|cx| InputState::new(window, cx).masked(true))
            }),
        }
    }
    pub fn appearance(&self, cx: &App) -> quickpick_core::appearance::AppearanceDraft {
        let selected = |key: &str| match &self.controls[key] {
            Control::Select(state) => state.read(cx).selected_value().cloned().unwrap_or_default(),
            _ => unreachable!(),
        };
        let opacity = match &self.controls["panelOpacity"] {
            Control::Slider(state) => match state.read(cx).value() {
                SliderValue::Single(v) | SliderValue::Range(v, _) => v.round() as u8,
            },
            _ => unreachable!(),
        };
        quickpick_core::appearance::AppearanceDraft {
            theme_mode: selected("themeMode"),
            window_effect: selected("windowEffect"),
            panel_opacity: opacity,
        }
    }
    pub fn restore_appearance(&self, settings: &AppSettings, window: &mut Window, cx: &mut App) {
        for (key, value) in [
            ("themeMode", &settings.theme_mode),
            ("windowEffect", &settings.window_effect),
        ] {
            if let Control::Select(state) = &self.controls[key] {
                state.update(cx, |state, cx| state.set_selected_value(value, window, cx));
            }
        }
        if let Control::Slider(state) = &self.controls["panelOpacity"] {
            state.update(cx, |state, cx| {
                state.set_value(f32::from(settings.panel_opacity), window, cx)
            });
        }
    }
    pub fn collect(&self, cx: &App) -> Result<AppSettings, String> {
        let mut data = serde_json::Map::new();
        for (key, control) in &self.controls {
            let value = match control {
                Control::Text(input) => {
                    let text = input.read(cx).value().to_string();
                    if key == "aiTimeoutSeconds" {
                        serde_json::json!(
                            text.parse::<u16>()
                                .map_err(|_| "i18n:settings.timeoutInvalid")?
                        )
                    } else {
                        serde_json::json!(text)
                    }
                }
                Control::Select(select) => serde_json::json!(
                    select
                        .read(cx)
                        .selected_value()
                        .cloned()
                        .unwrap_or_default()
                ),
                Control::Toggle(value) => serde_json::json!(value),
                Control::Slider(slider) => {
                    let value = match slider.read(cx).value() {
                        SliderValue::Single(v) => v,
                        SliderValue::Range(v, _) => v,
                    };
                    serde_json::json!(value.round() as u8)
                }
            };
            data.insert(key.clone(), value);
        }
        serde_json::from_value(data.into()).map_err(|_| "i18n:settings.invalidFormat".into())
    }
    pub fn clear_secrets(&self, window: &mut Window, cx: &mut App) {
        for input in &self.key_inputs {
            input.update(cx, |input, cx| input.clean(window, cx));
        }
    }
    pub fn categories() -> [&'static str; 7] {
        CATEGORIES
    }
    pub fn fields(&self) -> Vec<(&'static str, &'static str)> {
        match self.category {
            0 => vec![
                ("autostartEnabled", "settings.autostart"),
                ("allowClipboardFallback", "settings.allowClipboardFallback"),
                ("uiLanguage", "settings.language"),
                ("translationTargetLanguage", "result.targetLanguage"),
                ("inputTranslateSourceLanguage", "input.sourceLanguage"),
                ("inputTranslateTargetLanguage", "input.targetLanguage"),
            ],
            1 => vec![
                ("settingsHotkey", "settings.title"),
                ("selectionHotkey", "settings.aiSelection"),
                ("screenshotHotkey", "settings.aiScreenshot"),
                ("inputTranslateHotkey", "input.title"),
            ],
            2 => vec![
                ("textAiProvider", "settings.textProvider"),
                ("textAiBaseUrl", "Base URL"),
                ("textAiModel", "settings.textModel"),
                ("aiTimeoutSeconds", "settings.aiTimeout"),
            ],
            3 => vec![
                ("visionAiProvider", "settings.visionProvider"),
                ("visionAiBaseUrl", "Base URL"),
                ("visionAiModel", "settings.visionModel"),
            ],
            4 => vec![
                ("inputAiProvider", "settings.inputProvider"),
                ("inputAiBaseUrl", "Base URL"),
                ("inputAiModel", "settings.inputModel"),
            ],
            5 => vec![
                ("themeMode", "settings.theme"),
                ("windowEffect", "settings.windowEffect"),
                ("panelOpacity", "settings.opacity"),
            ],
            _ => vec![],
        }
    }
}
