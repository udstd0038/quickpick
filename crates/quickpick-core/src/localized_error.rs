pub fn error_key(key: &str) -> String {
    format!("i18n:{key}")
}

pub fn error_key_with_detail(key: &str, detail: &str) -> String {
    format!("i18n:{key}:{detail}")
}
