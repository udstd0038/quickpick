use tauri_plugin_global_shortcut::Code;

/// Physical CGKeyCode values for the keys supported by shortcut capture.
/// An unknown key must block synthetic copy rather than skip the release check.
pub(crate) fn key_code(code: Code) -> Option<i32> {
    Some(match code {
        Code::KeyA => 0,
        Code::KeyS => 1,
        Code::KeyD => 2,
        Code::KeyF => 3,
        Code::KeyH => 4,
        Code::KeyG => 5,
        Code::KeyZ => 6,
        Code::KeyX => 7,
        Code::KeyC => 8,
        Code::KeyV => 9,
        Code::KeyB => 11,
        Code::KeyQ => 12,
        Code::KeyW => 13,
        Code::KeyE => 14,
        Code::KeyR => 15,
        Code::KeyY => 16,
        Code::KeyT => 17,
        Code::Digit1 => 18,
        Code::Digit2 => 19,
        Code::Digit3 => 20,
        Code::Digit4 => 21,
        Code::Digit6 => 22,
        Code::Digit5 => 23,
        Code::Equal => 24,
        Code::Digit9 => 25,
        Code::Digit7 => 26,
        Code::Minus => 27,
        Code::Digit8 => 28,
        Code::Digit0 => 29,
        Code::BracketRight => 30,
        Code::KeyO => 31,
        Code::KeyU => 32,
        Code::BracketLeft => 33,
        Code::KeyI => 34,
        Code::KeyP => 35,
        Code::Enter => 36,
        Code::KeyL => 37,
        Code::KeyJ => 38,
        Code::Quote => 39,
        Code::KeyK => 40,
        Code::Semicolon => 41,
        Code::Backslash => 42,
        Code::Comma => 43,
        Code::Slash => 44,
        Code::KeyN => 45,
        Code::KeyM => 46,
        Code::Period => 47,
        Code::Tab => 48,
        Code::Space => 49,
        Code::Backquote => 50,
        Code::Backspace => 51,
        Code::Escape => 53,
        Code::F17 => 64,
        Code::F18 => 79,
        Code::F19 => 80,
        Code::F20 => 90,
        Code::F5 => 96,
        Code::F6 => 97,
        Code::F7 => 98,
        Code::F3 => 99,
        Code::F8 => 100,
        Code::F9 => 101,
        Code::F11 => 103,
        Code::F13 => 105,
        Code::F16 => 106,
        Code::F14 => 107,
        Code::F10 => 109,
        Code::F12 => 111,
        Code::F15 => 113,
        Code::F4 => 118,
        Code::F2 => 120,
        Code::F1 => 122,
        Code::ArrowLeft => 123,
        Code::ArrowRight => 124,
        Code::ArrowDown => 125,
        Code::ArrowUp => 126,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_default_and_custom_shortcuts_use_physical_codes() {
        assert_eq!(key_code(Code::KeyQ), Some(12));
        assert_eq!(key_code(Code::Digit2), Some(19));
        assert_eq!(key_code(Code::Digit4), Some(21));
        assert_eq!(key_code(Code::F1), Some(122));
        assert_eq!(key_code(Code::ArrowLeft), Some(123));
    }

    #[test]
    fn unsupported_keys_cannot_bypass_release_wait() {
        assert_eq!(key_code(Code::PrintScreen), None);
        assert_eq!(key_code(Code::F24), None);
    }
}
