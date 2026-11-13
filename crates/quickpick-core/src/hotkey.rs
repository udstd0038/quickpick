pub use keyboard_types::{Code, Modifiers};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub mods: Modifiers,
    pub key: Code,
}
impl Shortcut {
    pub fn id(self) -> u32 {
        (self.mods.bits() << 16) | self.key as u32
    }
}
impl FromStr for Shortcut {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut mods = Modifiers::empty();
        let mut key = None;
        for token in value.split('+').map(str::trim) {
            if key.is_some() {
                return Err(());
            }
            let modifier = match token.to_ascii_lowercase().as_str() {
                "alt" => Some(Modifiers::ALT),
                "shift" => Some(Modifiers::SHIFT),
                "ctrl" | "control" => Some(Modifiers::CONTROL),
                "win" | "super" | "meta" => Some(Modifiers::META),
                _ => None,
            };
            if let Some(modifier) = modifier {
                mods.insert(modifier);
                continue;
            }
            let code = if token.len() == 1 && token.as_bytes()[0].is_ascii_digit() {
                format!("Digit{token}")
            } else if token.len() == 1 && token.as_bytes()[0].is_ascii_alphabetic() {
                format!("Key{}", token.to_ascii_uppercase())
            } else {
                match token.to_ascii_lowercase().as_str() {
                    "space" => "Space".into(),
                    "tab" => "Tab".into(),
                    "enter" => "Enter".into(),
                    "escape" | "esc" => "Escape".into(),
                    "printscreen" => "PrintScreen".into(),
                    "left" | "arrowleft" => "ArrowLeft".into(),
                    "right" | "arrowright" => "ArrowRight".into(),
                    "up" | "arrowup" => "ArrowUp".into(),
                    "down" | "arrowdown" => "ArrowDown".into(),
                    _ => token.into(),
                }
            };
            key = Some(code.parse::<Code>().map_err(|_| ())?);
        }
        Ok(Self {
            mods,
            key: key.ok_or(())?,
        })
    }
}
