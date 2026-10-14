use serde::Serialize;
use tauri::AppHandle;

use crate::{clipboard_macos, selection_clipboard::FallbackError};

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
    _app: &AppHandle,
    hotkey_keys: Vec<i32>,
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

    let input_monitoring_allowed = tauri::async_runtime::block_on(
        tauri_plugin_macos_permissions::check_input_monitoring_permission(),
    );
    match clipboard_macos::capture_selection(&hotkey_keys, input_monitoring_allowed) {
        Ok(text) => {
            SelectionSnapshot::captured_from(text, "clipboard", "已通过剪贴板兜底读取选中文本")
        }
        Err(error) => {
            let mut snapshot = SelectionSnapshot::error(clipboard_error_message(error));
            snapshot.source = "clipboard".to_string();
            snapshot
        }
    }
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

fn clipboard_error_message(error: FallbackError) -> &'static str {
    match error {
        FallbackError::ClipboardAccessDenied => "系统拒绝访问剪贴板，本次未模拟复制；请检查 QuickPick 的剪贴板访问设置",
        FallbackError::Busy => "正在读取选中文本，请稍后重试",
        FallbackError::InvalidTrigger => {
            "当前快捷键无法安全用于剪贴板取词，请更换快捷键或使用直接取词"
        }
        FallbackError::SnapshotIncomplete | FallbackError::SnapshotPromise => {
            "当前剪贴板内容无法完整备份，本次未使用剪贴板兜底"
        }
        FallbackError::SnapshotTooLarge => "当前剪贴板内容超过备份上限，本次未使用剪贴板兜底",
        FallbackError::ClipboardChanged => {
            "剪贴板在取词过程中发生变化，已保留较新的内容，请重新选择后再试"
        }
        FallbackError::ClipboardReadFailed => "剪贴板内容读取失败，本次没有返回选中文本",
        FallbackError::CopyTimedOut => "复制未在规定时间内完成，本次没有读取旧剪贴板文本",
        FallbackError::EmptyCopiedText => "复制后未读取到文本，请重新选择文本后再试",
        FallbackError::KeysNotReleased => "快捷键尚未完全松开，本次未模拟复制",
        FallbackError::InputUnavailable => "辅助功能或输入监控权限不可用，请检查权限后重新触发取词",
        FallbackError::KeyboardLayoutUnavailable => {
            "当前键盘布局无法安全模拟复制，本次未使用剪贴板兜底"
        }
        FallbackError::FocusChanged => "当前应用在取词过程中发生变化，请重新选择文本后再试",
        FallbackError::InputFailed => "模拟复制失败，本次没有返回选中文本",
        FallbackError::ReleaseFailed => "模拟复制的按键释放失败，本次没有返回选中文本",
        FallbackError::RestoreFailed => "原剪贴板内容恢复失败，本次没有返回选中文本，请检查剪贴板",
        FallbackError::ReleaseAndRestoreFailed => {
            "模拟复制的按键释放和原剪贴板恢复均失败，本次没有返回选中文本，请检查键盘和剪贴板"
        }
        FallbackError::ReleaseAndClipboardChanged => "模拟复制的按键释放失败，且剪贴板在取词过程中发生变化，已保留较新的内容，请检查键盘后重试",
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
