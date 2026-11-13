/// UI labels use the committed page language, independently of draft values.
pub fn choices(key: &str, display_language: &str) -> Option<Vec<(String, String)>> {
    let values = match key {
        "uiLanguage" => Some(vec![
            "system", "zh-Hans", "zh-Hant", "en", "ko", "ja", "fr", "de", "es",
        ]),
        "themeMode" => Some(vec!["system", "light", "dark"]),
        "windowEffect" => Some(vec!["acrylic", "mica"]),
        "textAiProvider" | "visionAiProvider" | "inputAiProvider" => Some(vec![
            "openai_compatible",
            "deepseek",
            "xiaomi_mimo",
            "kimi",
            "glm",
            "minimax",
            "qwen",
        ]),
        "inputTranslateSourceLanguage" => Some(vec![
            "auto", "zh-Hans", "zh-Hant", "en", "ko", "ja", "fr", "de", "es",
        ]),
        "inputTranslateTargetLanguage" | "translationTargetLanguage" => {
            Some(quickpick_core::i18n::LANGUAGES.to_vec())
        }
        _ => None,
    }?;
    Some(
        values
            .into_iter()
            .map(|id| {
                let label = match id {
                    "system" if key == "themeMode" => "settings.themeSystem",
                    "system" => "settings.languageSystem",
                    "light" => "settings.themeLight",
                    "dark" => "settings.themeDark",
                    "auto" => "language.auto",
                    "zh-Hans" => "language.zhHans",
                    "zh-Hant" => "language.zhHant",
                    "en" => "language.en",
                    "ko" => "language.ko",
                    "ja" => "language.ja",
                    "fr" => "language.fr",
                    "de" => "language.de",
                    "es" => "language.es",
                    "openai_compatible" => "providers.openaiCompatible",
                    "xiaomi_mimo" => "providers.xiaomiMimo",
                    "deepseek" => "DeepSeek",
                    "kimi" => "Kimi",
                    "glm" => "GLM",
                    "minimax" => "MiniMax",
                    "qwen" => "Qwen",
                    "mica" => "Mica",
                    "acrylic" => "Acrylic",
                    _ => id,
                };
                (
                    id.to_owned(),
                    quickpick_core::i18n::t(display_language, label),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickpick_core::app_settings::AppSettings;

    fn selected_label(key: &str, value: &str, language: &str) -> String {
        choices(key, language)
            .unwrap()
            .into_iter()
            .find(|(id, _)| id == value)
            .unwrap()
            .1
    }

    #[test]
    fn english_page_keeps_english_values_after_reset_and_appearance_rollback() {
        let saved = AppSettings {
            ui_language: "en".into(),
            theme_mode: "dark".into(),
            panel_opacity: 70,
            ..AppSettings::default()
        };
        let draft = AppSettings::default();
        assert_eq!(
            selected_label("themeMode", &draft.theme_mode, &saved.ui_language),
            quickpick_core::i18n::t("en", "settings.themeSystem")
        );
        assert_eq!(
            selected_label("themeMode", &saved.theme_mode, &saved.ui_language),
            "Dark"
        );
        assert_eq!(
            selected_label("uiLanguage", &draft.ui_language, &saved.ui_language),
            quickpick_core::i18n::t("en", "settings.languageSystem")
        );
        assert_eq!(
            selected_label("inputTranslateSourceLanguage", "auto", &saved.ui_language),
            quickpick_core::i18n::t("en", "language.auto")
        );
    }

    #[test]
    fn changing_display_language_preserves_all_choice_ids() {
        for key in [
            "uiLanguage",
            "themeMode",
            "windowEffect",
            "textAiProvider",
            "visionAiProvider",
            "inputAiProvider",
            "inputTranslateSourceLanguage",
            "inputTranslateTargetLanguage",
            "translationTargetLanguage",
        ] {
            let english = choices(key, "en").unwrap();
            for language in quickpick_core::i18n::LANGUAGES {
                let localized = choices(key, language).unwrap();
                assert_eq!(
                    english.iter().map(|(id, _)| id).collect::<Vec<_>>(),
                    localized.iter().map(|(id, _)| id).collect::<Vec<_>>()
                );
                assert!(localized.iter().all(|(_, label)| !label.is_empty()));
            }
        }
        assert_eq!(choices("panelOpacity", "en"), None);
        assert_eq!(selected_label("themeMode", "dark", "zh-Hans"), "深色");
    }
}
