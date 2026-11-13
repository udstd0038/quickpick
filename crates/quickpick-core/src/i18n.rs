use std::{collections::HashMap, sync::LazyLock};
static STRINGS: LazyLock<HashMap<String, HashMap<String, String>>> =
    LazyLock::new(|| serde_json::from_str(include_str!("locales.json")).expect("embedded locales"));
pub const LANGUAGES: [&str; 8] = ["zh-Hans", "zh-Hant", "en", "ko", "ja", "fr", "de", "es"];
pub fn resolve(language: &str) -> String {
    let language = if language == "system" {
        sys_locale::get_locale().unwrap_or_default()
    } else {
        language.into()
    };
    let lower = language.to_ascii_lowercase();
    if lower.starts_with("zh") {
        if lower.contains("hant")
            || lower.contains("tw")
            || lower.contains("hk")
            || lower.contains("mo")
        {
            "zh-Hant".into()
        } else {
            "zh-Hans".into()
        }
    } else {
        LANGUAGES
            .into_iter()
            .find(|code| lower.starts_with(code))
            .unwrap_or("zh-Hans")
            .into()
    }
}
pub fn t(language: &str, key: &str) -> String {
    STRINGS
        .get(&resolve(language))
        .and_then(|values| values.get(key))
        .cloned()
        .unwrap_or_else(|| key.into())
}
pub fn error(language: &str, key: &str) -> String {
    match key.strip_prefix("i18n:") {
        Some(rest) => {
            let (key, detail) = rest.split_once(':').unwrap_or((rest, ""));
            format!(
                "{}{}{}",
                t(language, key),
                if detail.is_empty() { "" } else { " " },
                detail
            )
        }
        None => t(language, key),
    }
}
pub fn interpolate(language: &str, key: &str, params: &[(&str, &str)]) -> String {
    let mut value = t(language, key);
    for (key, param) in params {
        value = value.replace(&format!("{{{key}}}"), param);
    }
    value
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dictionaries_and_success_payloads_are_separate() {
        for language in LANGUAGES {
            assert!(!t(language, "settings.save").contains("settings.save"));
        }
        assert_eq!(resolve("zh-HK"), "zh-Hant");
        assert!(
            interpolate(
                "en",
                "result.sourceTarget",
                &[("source", "A"), ("target", "B")]
            )
            .contains("A")
        );
    }
    #[test]
    fn english_dictionary_contains_no_untranslated_chinese() {
        for (key, value) in &STRINGS["en"] {
            assert!(
                !value
                    .chars()
                    .any(|ch| ('\u{3400}'..='\u{9fff}').contains(&ch)),
                "Untranslated Chinese in English key: {key}"
            );
        }
    }
    #[test]
    fn english_format_strings_keep_their_named_parameters() {
        for key in [
            "result.sourceTarget",
            "input.sourceTarget",
            "settings.recordHotkey",
            "settings.hotkeyConflict",
        ] {
            let parameters = |value: &str| -> Vec<String> {
                let mut result = value
                    .split('{')
                    .skip(1)
                    .filter_map(|part| part.split_once('}').map(|(key, _)| key.to_owned()))
                    .collect::<Vec<_>>();
                result.sort();
                result
            };
            assert_eq!(
                parameters(&STRINGS["en"][key]),
                parameters(&STRINGS["zh-Hans"][key]),
                "Parameter mismatch: {key}"
            );
        }
    }
}
