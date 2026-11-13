use crate::ui::*;
use async_channel::Sender;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};
use muda::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem};
use quickpick_core::{app_settings::AppSettings, i18n};
use quickpick_platform::selection;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Settings,
    Selection,
    Screenshot,
    Input,
}
pub enum Event {
    Hotkey(u32, isize),
    Settings,
    Autostart,
    Quit,
    CancelScreenshot,
    RestoreAppearance(isize),
    SyncWindowBounds(isize),
}
impl From<quickpick_platform::window_events::Notice> for Event {
    fn from(notice: quickpick_platform::window_events::Notice) -> Self {
        use quickpick_platform::window_events::Notice;
        match notice {
            Notice::CancelCapture => Self::CancelScreenshot,
            Notice::RestoreAppearance(hwnd) => Self::RestoreAppearance(hwnd),
            Notice::BoundsChanged(hwnd) => Self::SyncWindowBounds(hwnd),
        }
    }
}
pub struct Native {
    manager: GlobalHotKeyManager,
    pub bindings: Vec<(HotKey, Action, Vec<i32>)>,
    tray: TrayIcon,
}
fn bindings(settings: &AppSettings) -> Result<Vec<(HotKey, Action, Vec<i32>)>, String> {
    [
        (&settings.settings_hotkey, Action::Settings),
        (&settings.selection_hotkey, Action::Selection),
        (&settings.screenshot_hotkey, Action::Screenshot),
        (&settings.input_translate_hotkey, Action::Input),
    ]
    .into_iter()
    .map(|(value, action)| {
        let hotkey = value
            .parse::<HotKey>()
            .map_err(|_| "i18n:hotkey.invalid".to_string())?;
        Ok((hotkey, action, release_keys(&hotkey)))
    })
    .collect()
}
impl Native {
    pub fn clear(&mut self) -> Result<(), String> {
        while let Some((key, _, _)) = self.bindings.last() {
            self.manager
                .unregister(*key)
                .map_err(|_| "i18n:hotkey.status.failed".to_string())?;
            self.bindings.pop();
        }
        Ok(())
    }
    pub fn new(settings: &AppSettings, sender: Sender<Event>) -> Result<Self, String> {
        let tx = sender.clone();
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state == HotKeyState::Pressed {
                let _ = tx.try_send(Event::Hotkey(
                    event.id,
                    selection::current_foreground_window(),
                ));
            }
        }));
        let tx = sender.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                let _ = tx.try_send(Event::Settings);
            }
        }));
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == MenuId::new("settings") {
                Event::Settings
            } else if event.id == MenuId::new("autostart") {
                Event::Autostart
            } else if event.id == MenuId::new("exit") {
                Event::Quit
            } else {
                return;
            };
            let _ = sender.try_send(action);
        }));
        let image = image::load_from_memory(include_bytes!("../assets/tray-icon.png"))
            .map_err(|_| "tray icon")?
            .into_rgba8();
        let icon = Icon::from_rgba(image.as_raw().clone(), image.width(), image.height())
            .map_err(|_| "tray icon")?;
        let tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip("QuickPick")
            .with_menu(Box::new(menu(settings)?))
            .with_menu_on_left_click(false)
            .build()
            .map_err(|_| "tray init")?;
        let manager = GlobalHotKeyManager::new().map_err(|_| "i18n:hotkey.invalid")?;
        Ok(Self {
            manager,
            bindings: vec![],
            tray,
        })
    }
    pub fn replace(&mut self, settings: &AppSettings) -> Result<(), String> {
        let next = bindings(settings)?;
        struct Backend<'a>(&'a GlobalHotKeyManager);
        impl quickpick_core::bindings::HotkeyBackend<HotKey> for Backend<'_> {
            fn register(&mut self, key: HotKey) -> Result<(), String> {
                self.0
                    .register(key)
                    .map_err(|_| "i18n:hotkey.status.occupied".into())
            }
            fn unregister(&mut self, key: HotKey) -> Result<(), String> {
                self.0
                    .unregister(key)
                    .map_err(|_| "i18n:hotkey.status.failed".into())
            }
        }
        let old_keys: Vec<_> = self.bindings.iter().map(|(key, _, _)| *key).collect();
        let new_keys: Vec<_> = next.iter().map(|(key, _, _)| *key).collect();
        match quickpick_core::bindings::replace(&old_keys, &new_keys, &mut Backend(&self.manager)) {
            Ok(()) => {
                self.bindings = next;
                Ok(())
            }
            Err(quickpick_core::bindings::SwapError::Apply(error)) => Err(error),
            Err(quickpick_core::bindings::SwapError::Rollback) => {
                for key in old_keys.into_iter().chain(new_keys) {
                    let _ = self.manager.unregister(key);
                }
                self.bindings.clear();
                Err("i18n:hotkey.restoreFailed".into())
            }
        }
    }
    pub fn update_menu(&self, settings: &AppSettings) -> Result<(), String> {
        self.tray.set_menu(Some(Box::new(menu(settings)?)));
        Ok(())
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        for (key, _, _) in &self.bindings {
            let _ = self.manager.unregister(*key);
        }
    }
}
fn menu(settings: &AppSettings) -> Result<Menu, String> {
    let menu = Menu::new();
    let language = &settings.ui_language;
    let settings_item = MenuItem::with_id(
        "settings",
        i18n::t(language, "settings.hotkeySettings"),
        true,
        None,
    );
    let autostart = CheckMenuItem::with_id(
        "autostart",
        i18n::t(language, "settings.autostart"),
        true,
        settings.autostart_enabled,
        None,
    );
    let quit = MenuItem::with_id("exit", i18n::t(language, "tray.exit"), true, None);
    menu.append_items(&[&settings_item, &autostart, &quit])
        .map_err(|_| "tray menu")?;
    Ok(menu)
}

use global_hotkey::hotkey::{Code, Modifiers};
pub fn release_keys(shortcut: &HotKey) -> Vec<i32> {
    let mut keys = Vec::new();

    if shortcut.mods.contains(Modifiers::CONTROL) {
        keys.push(0x11);
    }
    if shortcut.mods.contains(Modifiers::SHIFT) {
        keys.push(0x10);
    }
    if shortcut.mods.contains(Modifiers::ALT) {
        keys.push(0x12);
    }
    if shortcut.mods.contains(Modifiers::META) {
        keys.push(0x5b);
        keys.push(0x5c);
    }
    if let Some(key) = shortcut_key_virtual_code(shortcut.key) {
        keys.push(key);
    }

    keys
}

fn shortcut_key_virtual_code(code: Code) -> Option<i32> {
    match code {
        Code::Digit0 => Some(0x30),
        Code::Digit1 => Some(0x31),
        Code::Digit2 => Some(0x32),
        Code::Digit3 => Some(0x33),
        Code::Digit4 => Some(0x34),
        Code::Digit5 => Some(0x35),
        Code::Digit6 => Some(0x36),
        Code::Digit7 => Some(0x37),
        Code::Digit8 => Some(0x38),
        Code::Digit9 => Some(0x39),
        Code::KeyA => Some(0x41),
        Code::KeyB => Some(0x42),
        Code::KeyC => Some(0x43),
        Code::KeyD => Some(0x44),
        Code::KeyE => Some(0x45),
        Code::KeyF => Some(0x46),
        Code::KeyG => Some(0x47),
        Code::KeyH => Some(0x48),
        Code::KeyI => Some(0x49),
        Code::KeyJ => Some(0x4a),
        Code::KeyK => Some(0x4b),
        Code::KeyL => Some(0x4c),
        Code::KeyM => Some(0x4d),
        Code::KeyN => Some(0x4e),
        Code::KeyO => Some(0x4f),
        Code::KeyP => Some(0x50),
        Code::KeyQ => Some(0x51),
        Code::KeyR => Some(0x52),
        Code::KeyS => Some(0x53),
        Code::KeyT => Some(0x54),
        Code::KeyU => Some(0x55),
        Code::KeyV => Some(0x56),
        Code::KeyW => Some(0x57),
        Code::KeyX => Some(0x58),
        Code::KeyY => Some(0x59),
        Code::KeyZ => Some(0x5a),
        Code::F1 => Some(0x70),
        Code::F2 => Some(0x71),
        Code::F3 => Some(0x72),
        Code::F4 => Some(0x73),
        Code::F5 => Some(0x74),
        Code::F6 => Some(0x75),
        Code::F7 => Some(0x76),
        Code::F8 => Some(0x77),
        Code::F9 => Some(0x78),
        Code::F10 => Some(0x79),
        Code::F11 => Some(0x7a),
        Code::F12 => Some(0x7b),
        Code::Space => Some(0x20),
        Code::Tab => Some(0x09),
        Code::Enter => Some(0x0d),
        Code::Escape => Some(0x1b),
        Code::PrintScreen => Some(0x2c),
        Code::ArrowLeft => Some(0x25),
        Code::ArrowUp => Some(0x26),
        Code::ArrowRight => Some(0x27),
        Code::ArrowDown => Some(0x28),
        _ => None,
    }
}
