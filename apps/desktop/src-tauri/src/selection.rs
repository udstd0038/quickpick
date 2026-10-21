use crate::localized_error::error_key;
use serde::Serialize;
use std::{
    collections::VecDeque,
    mem::size_of,
    thread,
    time::{Duration, Instant},
};
use uiautomation::{patterns::UITextPattern, types::ControlType, UIAutomation};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
            KEYEVENTF_KEYUP, VK_C, VK_CONTROL, VK_ESCAPE, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
        },
        WindowsAndMessaging::{
            GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, IsWindow,
            SetForegroundWindow, GUITHREADINFO, GUI_INMENUMODE, GUI_POPUPMENUMODE,
            GUI_SYSTEMMENUMODE,
        },
    },
};

const MAX_SELECTION_CHARS: i32 = 20_000;
const MAX_PREVIEW_CHARS: usize = 80;
const HOTKEY_RELEASE_WAIT_MS: u64 = 25;
const HOTKEY_RELEASE_ATTEMPTS: usize = 120;
const HOTKEY_RELEASE_SETTLE_MS: u64 = 90;
const FOREGROUND_RESTORE_SETTLE_MS: u64 = 80;
const CLIPBOARD_COPY_WAIT_MS: u64 = 25;
const CLIPBOARD_COPY_ATTEMPTS: usize = 48;
const UIA_MAX_NODES: usize = 256;
const UIA_MAX_DEPTH: usize = 16;
const UIA_SEARCH_BUDGET_MS: u64 = 350;

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
    allow_clipboard_fallback: bool,
) -> SelectionSnapshot {
    match thread::spawn(move || {
        capture_selected_text_with_fallback(
            foreground_window,
            hotkey_keys,
            allow_clipboard_fallback,
        )
    })
    .join()
    {
        Ok(snapshot) => snapshot,
        Err(_) => SelectionSnapshot::error("读取选区时发生内部错误，请重新选择后再试"),
    }
}

fn capture_selected_text_with_fallback(
    foreground_window: ForegroundWindow,
    hotkey_keys: Vec<i32>,
    allow_clipboard_fallback: bool,
) -> SelectionSnapshot {
    if !wait_for_capture_hotkey_release(&hotkey_keys) {
        return SelectionSnapshot::empty(error_key("selection.hotkeyHeld"));
    }
    if !restore_foreground_window(foreground_window) {
        return SelectionSnapshot::empty(error_key("selection.sourceUnfocused"));
    }
    thread::sleep(Duration::from_millis(FOREGROUND_RESTORE_SETTLE_MS));

    // Escape can cancel a selection in desktop apps. Only send it for an active menu.
    clear_alt_menu_mode(foreground_window);
    let direct_snapshot = capture_selected_text_inner(foreground_window);
    if matches!(&direct_snapshot.status, SelectionStatus::Captured) {
        return direct_snapshot;
    }
    if !allow_clipboard_fallback {
        return direct_snapshot;
    }

    match capture_selected_text_from_clipboard(foreground_window) {
        Some((text, restored)) => {
            let message = if restored {
                "已通过剪贴板兜底读取选中文本"
            } else {
                "已通过剪贴板兜底读取选中文本，但恢复原剪贴板失败"
            };
            SelectionSnapshot::captured_from(text, "clipboard", message)
        }
        None => direct_snapshot,
    }
}

fn capture_selected_text_inner(foreground_window: ForegroundWindow) -> SelectionSnapshot {
    let automation = match UIAutomation::new() {
        Ok(automation) => automation,
        Err(_) => return SelectionSnapshot::error("无法初始化 Windows 文本读取能力"),
    };

    let root = match automation.element_from_handle(foreground_window.into()) {
        Ok(element) => element,
        Err(_) => return SelectionSnapshot::unsupported(error_key("selection.controlUnavailable")),
    };
    let walker = match automation.get_raw_view_walker() {
        Ok(walker) => walker,
        Err(_) => return SelectionSnapshot::error(error_key("selection.controlUnavailable")),
    };
    let started = Instant::now();
    let budget = Duration::from_millis(UIA_SEARCH_BUDGET_MS);

    let mut candidates = Vec::new();
    if let Ok(mut current) = automation.get_focused_element() {
        for _ in 0..32 {
            candidates.push(current.clone());
            if automation
                .compare_elements(&current, &root)
                .unwrap_or(false)
            {
                break;
            }
            if started.elapsed() >= budget {
                candidates.clear();
                break;
            }
            current = match walker.get_parent(&current) {
                Ok(parent) => parent,
                Err(_) => break,
            };
        }
        // Only use the focus path when it belongs to the original foreground window.
        if !candidates
            .last()
            .is_some_and(|element| automation.compare_elements(element, &root).unwrap_or(false))
        {
            candidates.clear();
        }
    }

    let mut saw_text_pattern = false;
    for element in candidates {
        if current_foreground_window() != foreground_window {
            return SelectionSnapshot::empty(error_key("selection.sourceUnfocused"));
        }
        if let Ok(pattern) = element.get_pattern::<UITextPattern>() {
            saw_text_pattern = true;
            match read_text_pattern_selection(&pattern) {
                Ok(Some(text)) => return SelectionSnapshot::captured(text),
                Ok(None) => {}
                Err(_) => {}
            }
        }
    }

    // WebViews and custom desktop controls often expose selection on a descendant,
    // rather than the focused control or one of its ancestors.
    let mut pending = VecDeque::from([(root, 0usize)]);
    let mut visited = 0;
    while let Some((element, depth)) = pending.pop_front() {
        if visited >= UIA_MAX_NODES || started.elapsed() >= budget {
            break;
        }
        if current_foreground_window() != foreground_window {
            break;
        }
        visited += 1;
        if let Ok(pattern) = element.get_pattern::<UITextPattern>() {
            saw_text_pattern = true;
            if let Ok(Some(text)) = read_text_pattern_selection(&pattern) {
                return SelectionSnapshot::captured(text);
            }
        }
        if depth >= UIA_MAX_DEPTH {
            continue;
        }
        let mut child = walker.get_first_child(&element).ok();
        while let Some(element) = child {
            if visited + pending.len() >= UIA_MAX_NODES || started.elapsed() >= budget {
                break;
            }
            child = walker.get_next_sibling(&element).ok();
            pending.push_back((element, depth + 1));
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

fn capture_selected_text_from_clipboard(
    foreground_window: ForegroundWindow,
) -> Option<(String, bool)> {
    // Never accept an old clipboard value as newly copied selection text.
    let (clipboard_snapshot, before_seq) = snapshot_clipboard_formats()?;

    if current_foreground_window() != foreground_window
        || !selection_keys_released(&[])
        || clipboard_win::seq_num().map(|value| value.get()) != Some(before_seq)
        || !send_ctrl_c()
    {
        return None;
    }

    let (text, sequence) = wait_for_copied_text(before_seq, foreground_window)?;
    let restored = restore_clipboard_formats(clipboard_snapshot, sequence);
    Some((text, restored))
}

fn snapshot_clipboard_formats() -> Option<(Vec<ClipboardFormatSnapshot>, u32)> {
    let _clipboard = clipboard_win::Clipboard::new_attempts(10).ok()?;

    let mut had_formats = false;
    let formats: Vec<_> = clipboard_win::EnumFormats::new()
        .filter_map(|format| {
            had_formats = true;
            let mut data = Vec::new();
            clipboard_win::raw::get_vec(format, &mut data)
                .ok()
                .filter(|_| !data.is_empty())
                .map(|_| ClipboardFormatSnapshot { format, data })
        })
        .collect();
    // A genuinely empty clipboard must be cleared again after the temporary copy.
    // If nothing could be backed up, leave the original clipboard untouched.
    if had_formats && formats.is_empty() {
        None
    } else {
        // Reading delayed formats can itself change the sequence number. Sample
        // only after the backup is complete and while the clipboard is still locked.
        Some((formats, clipboard_win::seq_num()?.get()))
    }
}

fn restore_clipboard_formats(
    snapshot: Vec<ClipboardFormatSnapshot>,
    expected_sequence: u32,
) -> bool {
    let _clipboard = match clipboard_win::Clipboard::new_attempts(10) {
        Ok(clipboard) => clipboard,
        Err(_) => return false,
    };

    // Do not overwrite a clipboard update that arrived after the temporary copy.
    if clipboard_win::seq_num().map(|value| value.get()) != Some(expected_sequence) {
        return false;
    }
    if clipboard_win::raw::empty().is_err() {
        return false;
    }

    let mut restored = true;
    for item in snapshot {
        if clipboard_win::raw::set_without_clear(item.format, &item.data).is_err() {
            restored = false;
        }
    }
    restored
}

fn wait_for_copied_text(
    before_seq: u32,
    foreground_window: ForegroundWindow,
) -> Option<(String, u32)> {
    for _ in 0..CLIPBOARD_COPY_ATTEMPTS {
        thread::sleep(Duration::from_millis(CLIPBOARD_COPY_WAIT_MS));
        if current_foreground_window() != foreground_window {
            return None;
        }
        let Some(after_seq) = clipboard_win::seq_num().map(|value| value.get()) else {
            continue;
        };
        if after_seq == before_seq {
            continue;
        }

        if let Ok(value) = clipboard_win::get_clipboard_string() {
            let text = value.trim();
            if !text.is_empty()
                && current_foreground_window() == foreground_window
                && clipboard_win::seq_num().map(|value| value.get()) == Some(after_seq)
            {
                return Some((
                    text.chars().take(MAX_SELECTION_CHARS as usize).collect(),
                    after_seq,
                ));
            }
        }
    }

    None
}

pub(crate) fn wait_for_capture_hotkey_release(hotkey_keys: &[i32]) -> bool {
    for _ in 0..HOTKEY_RELEASE_ATTEMPTS {
        if selection_keys_released(hotkey_keys) {
            thread::sleep(Duration::from_millis(HOTKEY_RELEASE_SETTLE_MS));
            return selection_keys_released(hotkey_keys);
        }
        thread::sleep(Duration::from_millis(HOTKEY_RELEASE_WAIT_MS));
    }

    false
}

fn selection_keys_released(hotkey_keys: &[i32]) -> bool {
    hotkey_keys.iter().all(|key| !is_key_down(*key))
        && [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN]
            .iter()
            .all(|key| !is_key_down(*key as i32))
}

fn is_key_down(key: i32) -> bool {
    (unsafe { GetAsyncKeyState(key) }) < 0
}

fn restore_foreground_window(foreground_window: ForegroundWindow) -> bool {
    if foreground_window == 0 {
        return false;
    }

    let hwnd = foreground_window as HWND;
    unsafe {
        if IsWindow(hwnd) != 0 {
            let _ = SetForegroundWindow(hwnd);
        }
    }
    current_foreground_window() == foreground_window
}

pub(crate) fn clear_alt_menu_mode(foreground_window: ForegroundWindow) {
    if current_foreground_window() != foreground_window {
        return;
    }
    let mut info = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    let native_menu_active = unsafe {
        let thread_id = GetWindowThreadProcessId(foreground_window as HWND, std::ptr::null_mut());
        thread_id != 0
            && GetGUIThreadInfo(thread_id, &mut info) != 0
            && info.flags & (GUI_INMENUMODE | GUI_POPUPMENUMODE | GUI_SYSTEMMENUMODE) != 0
    };
    // Firefox and Electron can expose a custom menu only through accessibility.
    let accessible_menu_active = UIAutomation::new()
        .and_then(|automation| automation.get_focused_element())
        .and_then(|element| element.get_control_type())
        .is_ok_and(|control| {
            matches!(
                control,
                ControlType::Menu | ControlType::MenuBar | ControlType::MenuItem
            )
        });
    if !native_menu_active && !accessible_menu_active {
        return;
    }
    if current_foreground_window() != foreground_window || !selection_keys_released(&[]) {
        return;
    }
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
    thread::sleep(Duration::from_millis(FOREGROUND_RESTORE_SETTLE_MS));
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
