use serde::Serialize;
use std::{mem::size_of, thread, time::Duration};
use uiautomation::{patterns::UITextPattern, UIAutomation};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
            KEYEVENTF_KEYUP, VK_C, VK_CONTROL, VK_ESCAPE,
        },
        WindowsAndMessaging::{GetForegroundWindow, IsWindow, SetForegroundWindow},
    },
};

const MAX_SELECTION_CHARS: i32 = 20_000;
const MAX_PREVIEW_CHARS: usize = 80;
const HOTKEY_RELEASE_WAIT_MS: u64 = 25;
const HOTKEY_RELEASE_ATTEMPTS: usize = 24;
const HOTKEY_RELEASE_SETTLE_MS: u64 = 90;
const FOREGROUND_RESTORE_SETTLE_MS: u64 = 80;
const CLIPBOARD_COPY_WAIT_MS: u64 = 25;
const CLIPBOARD_COPY_ATTEMPTS: usize = 12;

pub type ForegroundWindow = isize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionSnapshot {
    pub status: SelectionStatus,
    pub text: String,
    pub preview: String,
    pub char_count: usize,
    pub message: String,
    pub source: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectionStatus {
    Captured,
    Empty,
    Unsupported,
    Error,
}

impl Default for SelectionSnapshot {
    fn default() -> Self {
        Self::empty("等待读取选中文本")
    }
}

impl SelectionSnapshot {
    fn captured(text: String) -> Self {
        Self::captured_from(text, "uia", "已读取选中文本")
    }

    fn captured_from(text: String, source: &str, message: &str) -> Self {
        let char_count = text.chars().count();
        Self {
            status: SelectionStatus::Captured,
            preview: make_preview(&text),
            text,
            char_count,
            message: message.to_string(),
            source: source.to_string(),
        }
    }

    fn empty(message: impl Into<String>) -> Self {
        Self {
            status: SelectionStatus::Empty,
            text: String::new(),
            preview: "未读取到选中文本".to_string(),
            char_count: 0,
            message: message.into(),
            source: "uia".to_string(),
        }
    }

    fn unsupported(message: impl Into<String>) -> Self {
        Self {
            status: SelectionStatus::Unsupported,
            text: String::new(),
            preview: "当前应用暂不支持直接取词".to_string(),
            char_count: 0,
            message: message.into(),
            source: "uia".to_string(),
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            status: SelectionStatus::Error,
            text: String::new(),
            preview: "读取失败".to_string(),
            char_count: 0,
            message: message.into(),
            source: "uia".to_string(),
        }
    }
}

pub fn current_foreground_window() -> ForegroundWindow {
    unsafe { GetForegroundWindow() as ForegroundWindow }
}

pub fn capture_selected_text_for_window(
    foreground_window: ForegroundWindow,
    hotkey_keys: Vec<i32>,
) -> SelectionSnapshot {
    match thread::spawn(move || capture_selected_text_with_fallback(foreground_window, hotkey_keys))
        .join()
    {
        Ok(snapshot) => snapshot,
        Err(_) => SelectionSnapshot::error("读取选区时发生内部错误，请重新选择后再试"),
    }
}

fn capture_selected_text_with_fallback(
    foreground_window: ForegroundWindow,
    hotkey_keys: Vec<i32>,
) -> SelectionSnapshot {
    wait_for_selection_hotkey_release(&hotkey_keys);
    restore_foreground_window(foreground_window);
    thread::sleep(Duration::from_millis(FOREGROUND_RESTORE_SETTLE_MS));

    let direct_snapshot = capture_selected_text_inner();
    if matches!(&direct_snapshot.status, SelectionStatus::Captured) {
        return direct_snapshot;
    }

    match capture_selected_text_from_clipboard(foreground_window) {
        Some(text) => {
            SelectionSnapshot::captured_from(text, "clipboard", "已通过剪贴板兜底读取选中文本")
        }
        None => direct_snapshot,
    }
}

fn capture_selected_text_inner() -> SelectionSnapshot {
    let automation = match UIAutomation::new() {
        Ok(automation) => automation,
        Err(_) => return SelectionSnapshot::error("无法初始化 Windows 文本读取能力"),
    };

    let focused = match automation.get_focused_element() {
        Ok(element) => element,
        Err(_) => return SelectionSnapshot::empty("未找到当前焦点控件，请重新选择文本后再试"),
    };

    let mut candidates = vec![focused.clone()];
    if let Ok(walker) = automation.get_control_view_walker() {
        let mut current = focused;
        for _ in 0..4 {
            match walker.get_parent(&current) {
                Ok(parent) => {
                    current = parent.clone();
                    candidates.push(parent);
                }
                Err(_) => break,
            }
        }
    }

    let mut saw_text_pattern = false;
    for element in candidates {
        if let Ok(pattern) = element.get_pattern::<UITextPattern>() {
            saw_text_pattern = true;
            match read_text_pattern_selection(&pattern) {
                Ok(Some(text)) => return SelectionSnapshot::captured(text),
                Ok(None) => {}
                Err(_) => {}
            }
        }
    }

    if saw_text_pattern {
        SelectionSnapshot::empty("未读取到选中文本，请重新选择后再试")
    } else {
        SelectionSnapshot::unsupported("当前应用未暴露可读取的文本选区")
    }
}

struct ClipboardFormatSnapshot {
    format: u32,
    data: Vec<u8>,
}

fn capture_selected_text_from_clipboard(foreground_window: ForegroundWindow) -> Option<String> {
    let before_seq = clipboard_win::seq_num().map(|value| value.get());
    let clipboard_snapshot = snapshot_clipboard_formats();

    restore_foreground_window(foreground_window);
    clear_alt_menu_mode();
    thread::sleep(Duration::from_millis(40));
    if !send_ctrl_c() {
        restore_clipboard_formats(clipboard_snapshot);
        return None;
    }

    let copied = wait_for_copied_text(before_seq);
    restore_clipboard_formats(clipboard_snapshot);
    copied
}

fn snapshot_clipboard_formats() -> Vec<ClipboardFormatSnapshot> {
    let _clipboard = match clipboard_win::Clipboard::new_attempts(10) {
        Ok(clipboard) => clipboard,
        Err(_) => return Vec::new(),
    };

    clipboard_win::EnumFormats::new()
        .filter_map(|format| {
            let mut data = Vec::new();
            clipboard_win::raw::get_vec(format, &mut data)
                .ok()
                .filter(|_| !data.is_empty())
                .map(|_| ClipboardFormatSnapshot { format, data })
        })
        .collect()
}

fn restore_clipboard_formats(snapshot: Vec<ClipboardFormatSnapshot>) {
    if snapshot.is_empty() {
        return;
    }

    let _clipboard = match clipboard_win::Clipboard::new_attempts(10) {
        Ok(clipboard) => clipboard,
        Err(_) => return,
    };

    if clipboard_win::raw::empty().is_err() {
        return;
    }

    for item in snapshot {
        let _ = clipboard_win::raw::set_without_clear(item.format, &item.data);
    }
}

fn wait_for_copied_text(before_seq: Option<u32>) -> Option<String> {
    for _ in 0..CLIPBOARD_COPY_ATTEMPTS {
        thread::sleep(Duration::from_millis(CLIPBOARD_COPY_WAIT_MS));
        let after_seq = clipboard_win::seq_num().map(|value| value.get());
        if before_seq.is_some() && after_seq == before_seq {
            continue;
        }

        if let Ok(value) = clipboard_win::get_clipboard_string() {
            let text = value.trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }

    None
}

fn wait_for_selection_hotkey_release(hotkey_keys: &[i32]) {
    for _ in 0..HOTKEY_RELEASE_ATTEMPTS {
        if hotkey_keys.iter().all(|key| !is_key_down(*key)) {
            thread::sleep(Duration::from_millis(HOTKEY_RELEASE_SETTLE_MS));
            return;
        }
        thread::sleep(Duration::from_millis(HOTKEY_RELEASE_WAIT_MS));
    }

    thread::sleep(Duration::from_millis(HOTKEY_RELEASE_SETTLE_MS));
}

fn is_key_down(key: i32) -> bool {
    (unsafe { GetAsyncKeyState(key) }) < 0
}

fn restore_foreground_window(foreground_window: ForegroundWindow) {
    if foreground_window == 0 {
        return;
    }

    let hwnd = foreground_window as HWND;
    unsafe {
        if IsWindow(hwnd) != 0 {
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

fn clear_alt_menu_mode() {
    let mut inputs = [
        keyboard_input(VK_ESCAPE, 0),
        keyboard_input(VK_ESCAPE, KEYEVENTF_KEYUP),
    ];

    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            size_of::<INPUT>() as i32,
        );
    }
}

fn send_ctrl_c() -> bool {
    let mut inputs = [
        keyboard_input(VK_CONTROL, 0),
        keyboard_input(VK_C, 0),
        keyboard_input(VK_C, KEYEVENTF_KEYUP),
        keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ];

    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    sent == inputs.len() as u32
}

fn keyboard_input(key: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn read_text_pattern_selection(pattern: &UITextPattern) -> Result<Option<String>, ()> {
    let ranges = pattern.get_selection().map_err(|_| ())?;
    if ranges.is_empty() {
        return Ok(None);
    }

    let mut parts = Vec::new();
    for range in ranges {
        let value = range.get_text(MAX_SELECTION_CHARS).map_err(|_| ())?;
        let normalized = value.trim();
        if !normalized.is_empty() {
            parts.push(normalized.to_string());
        }
    }

    if parts.is_empty() {
        Ok(None)
    } else {
        Ok(Some(parts.join("\n")))
    }
}

fn make_preview(text: &str) -> String {
    let mut compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return "未读取到选中文本".to_string();
    }

    if compact.chars().count() > MAX_PREVIEW_CHARS {
        compact = compact.chars().take(MAX_PREVIEW_CHARS).collect::<String>();
        compact.push_str("...");
    }

    compact
}
