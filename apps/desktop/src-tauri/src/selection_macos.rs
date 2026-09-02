use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use axuielement::{
    ax_attribute::AX_SELECTED_TEXT_ATTRIBUTE, is_process_trusted, is_process_trusted_with_prompt,
    system_wide,
};

const MAX_PREVIEW_CHARS: usize = 80;

pub type ForegroundWindow = ();

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
    pub fn captured(text: String) -> Self {
        Self::captured_from(text, "ax", "已读取选中文本")
    }

    pub fn captured_from(text: String, source: &str, message: &str) -> Self {
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

    pub fn empty(message: impl Into<String>) -> Self {
        Self {
            status: SelectionStatus::Empty,
            text: String::new(),
            preview: "未读取到选中文本".to_string(),
            char_count: 0,
            message: message.into(),
            source: "ax".to_string(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            status: SelectionStatus::Unsupported,
            text: String::new(),
            preview: "当前应用暂不支持直接取词".to_string(),
            char_count: 0,
            message: message.into(),
            source: "ax".to_string(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: SelectionStatus::Error,
            text: String::new(),
            preview: "读取失败".to_string(),
            char_count: 0,
            message: message.into(),
            source: "ax".to_string(),
        }
    }
}

pub fn current_foreground_window() -> ForegroundWindow {}

pub fn capture_selected_text_for_app(
    app: &AppHandle,
    _hotkey_keys: Vec<i32>,
    allow_clipboard_fallback: bool,
) -> SelectionSnapshot {
    if !is_process_trusted() {
        let _ = is_process_trusted_with_prompt();
        return SelectionSnapshot::error("请先在系统设置中允许 QuickPick 使用辅助功能");
    }

    let direct = read_ax_selected_text();
    if direct.status_captured() {
        return direct;
    }

    if !allow_clipboard_fallback {
        return direct;
    }

    capture_selected_text_from_clipboard(app)
        .map(|text| {
            SelectionSnapshot::captured_from(text, "clipboard", "已通过剪贴板兜底读取选中文本")
        })
        .unwrap_or(direct)
}

trait SnapshotStatusExt {
    fn status_captured(&self) -> bool;
}

impl SnapshotStatusExt for SelectionSnapshot {
    fn status_captured(&self) -> bool {
        matches!(self.status, SelectionStatus::Captured)
    }
}

fn read_ax_selected_text() -> SelectionSnapshot {
    let Some(system) = system_wide() else {
        return SelectionSnapshot::error("无法初始化 macOS 辅助功能");
    };

    let mut candidates = Vec::new();
    if let Ok(Some(app)) = system.focused_application() {
        candidates.push(app);
    }
    if let Ok(Some(element)) = system.focused_ui_element() {
        candidates.push(element);
    }

    if candidates.is_empty() {
        return SelectionSnapshot::empty("未找到当前焦点控件，请重新选择文本后再试");
    }

    for element in &candidates {
        match element.string_attribute(AX_SELECTED_TEXT_ATTRIBUTE) {
            Ok(Some(text)) if !text.trim().is_empty() => {
                return SelectionSnapshot::captured(text.trim().to_string());
            }
            _ => {}
        }
    }

    SelectionSnapshot::empty("未读取到选中文本，请重新选择后再试")
}

fn capture_selected_text_from_clipboard(app: &AppHandle) -> Option<String> {
    let previous = app.clipboard().read_text().ok();
    if !send_command_c() {
        return None;
    }

    std::thread::sleep(Duration::from_millis(80));
    let copied = app.clipboard().read_text().ok();
    if let Some(value) = previous {
        let _ = app.clipboard().write_text(value);
    }

    copied
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn send_command_c() -> bool {
    let Ok(mut enigo) = Enigo::new(&Settings::default()) else {
        return false;
    };

    enigo.key(Key::Meta, Direction::Press).is_ok()
        && enigo.key(Key::Unicode('c'), Direction::Click).is_ok()
        && enigo.key(Key::Meta, Direction::Release).is_ok()
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
