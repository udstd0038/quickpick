use crate::app_settings::AppSettings;

/// Transient appearance only; never serialized into the settings schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppearanceDraft {
    pub theme_mode: String,
    pub window_effect: String,
    pub panel_opacity: u8,
}
impl AppearanceDraft {
    /// Only the ordinary window shell uses the configured opacity.
    /// Foreground text, accent buttons and transient menus remain independently legible.
    pub fn surface_alpha(&self) -> f32 {
        f32::from(self.panel_opacity.clamp(30, 100)) / 100. * 0.92
    }
}

impl From<&AppSettings> for AppearanceDraft {
    fn from(settings: &AppSettings) -> Self {
        Self {
            theme_mode: settings.theme_mode.clone(),
            window_effect: settings.window_effect.clone(),
            panel_opacity: settings.panel_opacity,
        }
    }
}
#[derive(Clone)]
pub struct AppearancePreview {
    committed: AppearanceDraft,
    draft: Option<AppearanceDraft>,
}
impl AppearancePreview {
    pub fn new(settings: &AppSettings) -> Self {
        Self {
            committed: settings.into(),
            draft: None,
        }
    }
    pub fn effective(&self) -> &AppearanceDraft {
        self.draft.as_ref().unwrap_or(&self.committed)
    }
    pub fn preview(&mut self, draft: AppearanceDraft) {
        self.draft = Some(draft);
    }
    pub fn save_finished(&mut self, saved: Option<&AppSettings>) {
        if let Some(settings) = saved {
            self.committed = settings.into();
            self.draft = None;
        }
    }
    pub fn hide(&mut self) {
        self.draft = None;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_never_changes_saved_business_settings() {
        let saved = AppSettings {
            text_ai_model: "committed-model".into(),
            ..Default::default()
        };
        let before = serde_json::to_value(&saved).unwrap();
        let mut state = AppearancePreview::new(&saved);
        state.preview(AppearanceDraft {
            theme_mode: "dark".into(),
            window_effect: "mica".into(),
            panel_opacity: 55,
        });
        assert_eq!(state.effective().panel_opacity, 55);
        assert_eq!(serde_json::to_value(&saved).unwrap(), before);
    }
    #[test]
    fn hide_rolls_back_unsaved_appearance() {
        let saved = AppSettings::default();
        let mut state = AppearancePreview::new(&saved);
        let mut draft = AppearanceDraft::from(&saved);
        draft.panel_opacity = 42;
        state.preview(draft);
        state.hide();
        assert_eq!(state.effective(), &AppearanceDraft::from(&saved));
    }
    #[test]
    fn successful_save_becomes_the_next_rollback_target() {
        let mut saved = AppSettings::default();
        let mut state = AppearancePreview::new(&saved);
        saved.theme_mode = "dark".into();
        state.save_finished(Some(&saved));
        let mut next = AppearanceDraft::from(&saved);
        next.panel_opacity = 30;
        state.preview(next);
        state.hide();
        assert_eq!(state.effective(), &AppearanceDraft::from(&saved));
    }
    #[test]
    fn failed_save_keeps_preview_until_hidden() {
        let saved = AppSettings::default();
        let mut state = AppearancePreview::new(&saved);
        let mut draft = AppearanceDraft::from(&saved);
        draft.theme_mode = "light".into();
        state.preview(draft.clone());
        state.save_finished(None);
        assert_eq!(state.effective(), &draft);
        state.hide();
        assert_eq!(state.effective(), &AppearanceDraft::from(&saved));
    }
    #[test]
    fn opacity_is_clamped_for_the_single_window_surface() {
        for (opacity, expected) in [
            (0, 0.276),
            (30, 0.276),
            (70, 0.644),
            (100, 0.92),
            (255, 0.92),
        ] {
            let draft = AppearanceDraft {
                panel_opacity: opacity,
                ..AppearanceDraft::from(&AppSettings::default())
            };
            assert!((draft.surface_alpha() - expected).abs() < 0.00001);
        }
    }
    #[test]
    fn opacity_changes_background_surfaces_monotonically() {
        let mut draft = AppearanceDraft::from(&AppSettings::default());
        draft.panel_opacity = 30;
        let low = draft.surface_alpha();
        draft.panel_opacity = 70;
        let medium = draft.surface_alpha();
        draft.panel_opacity = 100;
        let high = draft.surface_alpha();
        assert!(0. < low && low < medium && medium < high && high < 1.);
    }
}
