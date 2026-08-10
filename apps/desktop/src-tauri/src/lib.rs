mod ai;
mod app_settings;
mod localized_error;
mod screenshot;
mod security;
mod selection;

use selection::SelectionSnapshot;
use localized_error::error_key;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    window::{Color, Effect, EffectsBuilder},
    Emitter, Manager, Theme, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

struct AppState {
    selection_snapshot: Mutex<SelectionSnapshot>,
    result_snapshot: Mutex<ResultSnapshot>,
    last_text_translation: Mutex<Option<LastTextTranslation>>,
    last_image_translation: Mutex<Option<LastImageTranslation>>,
    last_input_result: Mutex<Option<String>>,
    hotkey_bindings: Mutex<HotkeyBindings>,
    hotkey_capture_mode: Mutex<bool>,
}

#[derive(Clone)]
struct LastTextTranslation {
    action: String,
    text: String,
    source_preview: String,
    source_char_count: usize,
}

#[derive(Clone)]
struct LastImageTranslation {
    action: String,
    png_bytes: Vec<u8>,
    source_preview: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            selection_snapshot: Mutex::new(SelectionSnapshot::default()),
            result_snapshot: Mutex::new(ResultSnapshot::default()),
            last_text_translation: Mutex::new(None),
            last_image_translation: Mutex::new(None),
            last_input_result: Mutex::new(None),
            hotkey_bindings: Mutex::new(HotkeyBindings::default()),
            hotkey_capture_mode: Mutex::new(false),
        }
    }
}

#[derive(Clone)]
struct HotkeyBindings {
    settings_hotkey: String,
    selection_hotkey: String,
    screenshot_hotkey: String,
    input_translate_hotkey: String,
    settings_id: Option<u32>,
    selection_id: u32,
    screenshot_id: u32,
    input_translate_id: u32,
}

impl HotkeyBindings {
    fn from_settings(settings: &app_settings::AppSettings) -> Result<Self, String> {
        let settings_hotkey = app_settings::parse_hotkey(&settings.settings_hotkey, "设置")?;
        let selection = app_settings::parse_hotkey(&settings.selection_hotkey, "划词菜单")?;
        let screenshot = app_settings::parse_hotkey(&settings.screenshot_hotkey, "区域截图")?;
        let input_translate =
            app_settings::parse_hotkey(&settings.input_translate_hotkey, "输入翻译")?;

        Ok(Self {
            settings_hotkey: settings.settings_hotkey.clone(),
            selection_hotkey: settings.selection_hotkey.clone(),
            screenshot_hotkey: settings.screenshot_hotkey.clone(),
            input_translate_hotkey: settings.input_translate_hotkey.clone(),
            settings_id: Some(settings_hotkey.id()),
            selection_id: selection.id(),
            screenshot_id: screenshot.id(),
            input_translate_id: input_translate.id(),
        })
    }

    fn action_for(&self, shortcut: &Shortcut) -> Option<HotkeyAction> {
        let id = shortcut.id();
        if self.settings_id == Some(id) {
            return Some(HotkeyAction::Settings);
        }
        if id == self.selection_id {
            return Some(HotkeyAction::Selection);
        }
        if id == self.screenshot_id {
            return Some(HotkeyAction::Screenshot);
        }
        if id == self.input_translate_id {
            return Some(HotkeyAction::InputTranslate);
        }
        None
    }
}

impl Default for HotkeyBindings {
    fn default() -> Self {
        Self::from_settings(&app_settings::AppSettings::default())
            .expect("default QuickPick hotkeys must be valid")
    }
}

#[derive(Clone, Copy)]
enum HotkeyAction {
    Settings,
    Selection,
    Screenshot,
    InputTranslate,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SelectionActionResult {
    message: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
enum HotkeyRegistrationStatus {
    Registered,
    Occupied,
    Failed,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyStatus {
    name: String,
    status: HotkeyRegistrationStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsActionResult {
    message: String,
    hotkey_statuses: Vec<HotkeyStatus>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenshotStatusEvent {
    kind: &'static str,
    message: String,
}

fn provider_id(settings: &app_settings::AppSettings) -> &str {
    match settings.text_ai_provider.trim() {
        "deepseek" => "deepseek",
        "xiaomi_mimo" => "xiaomi_mimo",
        "kimi" => "kimi",
        "glm" => "glm",
        "minimax" => "minimax",
        "qwen" => "qwen",
        _ => "openai_compatible",
    }
}

fn vision_provider_id(settings: &app_settings::AppSettings) -> &str {
    match settings.vision_ai_provider.trim() {
        "xiaomi_mimo" => "xiaomi_mimo",
        "kimi" => "kimi",
        "glm" => "glm",
        "minimax" => "minimax",
        "qwen" => "qwen",
        _ => "openai_compatible",
    }
}

fn text_provider_has_default_base_url(settings: &app_settings::AppSettings) -> bool {
    !matches!(provider_id(settings), "openai_compatible")
}

fn text_provider_has_default_model(settings: &app_settings::AppSettings) -> bool {
    !matches!(provider_id(settings), "openai_compatible")
}

fn vision_provider_has_default_base_url(settings: &app_settings::AppSettings) -> bool {
    !matches!(vision_provider_id(settings), "openai_compatible")
}

fn vision_provider_has_default_model(settings: &app_settings::AppSettings) -> bool {
    !matches!(vision_provider_id(settings), "openai_compatible")
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultSnapshot {
    status: String,
    kind: String,
    title: String,
    content: String,
    detail: String,
    source_preview: String,
    source_char_count: usize,
    can_copy: bool,
    can_retry: bool,
    source_language: String,
    target_language: String,
    translation_direction: String,
    can_switch_language: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InputReadyPayload {
    status: String,
    content: String,
    detail: String,
    source_language: String,
    target_language: String,
    direction: String,
}

impl Default for ResultSnapshot {
    fn default() -> Self {
        Self {
            status: "empty".to_string(),
            kind: "empty".to_string(),
            title: "QuickPick 结果".to_string(),
            content: "暂无结果".to_string(),
            detail: "从划词弹窗或截图菜单点击 AI 功能后，会打开独立原生结果弹窗。".to_string(),
            source_preview: String::new(),
            source_char_count: 0,
            can_copy: false,
            can_retry: false,
            source_language: default_translation_source_language(),
            target_language: "zh-Hans".to_string(),
            translation_direction: "right".to_string(),
            can_switch_language: false,
        }
    }
}

fn default_translation_source_language() -> String {
    "auto".to_string()
}

fn default_translation_direction() -> String {
    "right".to_string()
}

fn normalize_translation_direction(direction: &str) -> String {
    if direction == "left" {
        "left".to_string()
    } else {
        "right".to_string()
    }
}

fn initial_source_language() -> String {
    default_translation_source_language()
}

fn translation_request_languages(
    left_language: &str,
    right_language: &str,
    direction: &str,
) -> (String, String) {
    if direction == "left" {
        (right_language.to_string(), left_language.to_string())
    } else {
        (left_language.to_string(), right_language.to_string())
    }
}

impl ResultSnapshot {
    fn text_meta(action: &str) -> Result<(&'static str, &'static str, &'static str), String> {
        match action {
            "translate" => Ok(("text_translate", "文本翻译", "文本翻译")),
            "summarize" => Ok(("text_summarize", "文本总结", "文本总结")),
            _ => Err("未知的结果类型".to_string()),
        }
    }

    fn image_meta(action: &str) -> Result<(&'static str, &'static str, &'static str), String> {
        match action {
            "extract" => Ok(("image_extract", "截图文字提取", "图片文字提取")),
            "translate" => Ok(("image_translate", "截图文字翻译", "图片文字翻译")),
            _ => Err("未知的图片结果类型".to_string()),
        }
    }

    fn text_ai_preflight(
        action: &str,
        source_preview: String,
        source_char_count: usize,
        settings: &app_settings::AppSettings,
        api_key_configured: bool,
    ) -> Result<Self, String> {
        let (kind, title, action_label) = Self::text_meta(action)?;

        let base_url = settings.text_ai_base_url.trim();
        let text_model = settings.text_ai_model.trim();
        let mut setup_issues = Vec::new();

        if base_url.is_empty() && !text_provider_has_default_base_url(settings) {
            setup_issues.push("文本模型 Base URL");
        } else if !base_url.is_empty()
            && !(base_url.starts_with("https://") || base_url.starts_with("http://"))
        {
            setup_issues.push("文本模型 Base URL（需以 http:// 或 https:// 开头）");
        }

        if text_model.is_empty() && !text_provider_has_default_model(settings) {
            setup_issues.push("文本模型");
        }

        if !api_key_configured {
            setup_issues.push("文本模型 API Key");
        }

        if !setup_issues.is_empty() {
            let issue_list = setup_issues
                .iter()
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
                .join("\n");

            return Ok(Self {
                status: "error".to_string(),
                kind: kind.to_string(),
                title: title.to_string(),
                content: format!("还不能开始{action_label}。\n请先到设置里补齐：\n{issue_list}"),
                detail: "本次没有发起 AI 请求，也没有上传选中文本。".to_string(),
                source_preview,
                source_char_count,
                can_copy: false,
                can_retry: true,
                source_language: initial_source_language(),
                target_language: settings.translation_target_language.clone(),
                translation_direction: default_translation_direction(),
                can_switch_language: action == "translate",
            });
        }

        Ok(Self {
            status: "placeholder".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content: format!("{action_label}配置检查通过。下一小步会接入真实 OpenAI 兼容请求。"),
            detail: format!(
                "将使用当前文本模型配置和 {} 秒超时；本次仍未发起网络请求，也没有上传选中文本。",
                settings.ai_timeout_seconds
            ),
            source_preview,
            source_char_count,
            can_copy: false,
            can_retry: false,
            source_language: initial_source_language(),
            target_language: settings.translation_target_language.clone(),
            translation_direction: default_translation_direction(),
            can_switch_language: action == "translate",
        })
    }

    fn text_loading(
        action: &str,
        source_preview: String,
        source_char_count: usize,
        source_language: String,
        target_language: String,
        translation_direction: String,
    ) -> Result<Self, String> {
        let (kind, title, action_label) = Self::text_meta(action)?;

        Ok(Self {
            status: "loading".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content: format!("正在进行{action_label}..."),
            detail: String::new(),
            source_preview,
            source_char_count,
            can_copy: false,
            can_retry: false,
            source_language,
            target_language,
            translation_direction: normalize_translation_direction(&translation_direction),
            can_switch_language: action == "translate",
        })
    }

    fn text_success(
        action: &str,
        source_preview: String,
        source_char_count: usize,
        content: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
    ) -> Result<Self, String> {
        let (kind, title, _) = Self::text_meta(action)?;

        Ok(Self {
            status: "success".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content,
            detail: String::new(),
            source_preview,
            source_char_count,
            can_copy: true,
            can_retry: true,
            source_language,
            target_language,
            translation_direction: normalize_translation_direction(&translation_direction),
            can_switch_language: action == "translate",
        })
    }

    fn text_error(
        action: &str,
        source_preview: String,
        source_char_count: usize,
        message: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
    ) -> Result<Self, String> {
        let (kind, title, _) = Self::text_meta(action)?;

        Ok(Self {
            status: "error".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content: message,
            detail: "未覆盖剪贴板。请检查设置后重新从划词弹窗触发。".to_string(),
            source_preview,
            source_char_count,
            can_copy: false,
            can_retry: true,
            source_language,
            target_language,
            translation_direction: normalize_translation_direction(&translation_direction),
            can_switch_language: action == "translate",
        })
    }

    fn image_ai_preflight(
        action: &str,
        source_preview: String,
        settings: &app_settings::AppSettings,
        api_key_configured: bool,
    ) -> Result<Self, String> {
        let (kind, title, action_label) = Self::image_meta(action)?;

        let base_url = settings.vision_ai_base_url.trim();
        let vision_model = settings.vision_ai_model.trim();
        let mut setup_issues = Vec::new();

        if base_url.is_empty() && !vision_provider_has_default_base_url(settings) {
            setup_issues.push("视觉模型 Base URL");
        } else if !base_url.is_empty()
            && !(base_url.starts_with("https://") || base_url.starts_with("http://"))
        {
            setup_issues.push("视觉模型 Base URL（需以 http:// 或 https:// 开头）");
        }

        if vision_model.is_empty() && !vision_provider_has_default_model(settings) {
            setup_issues.push("视觉模型");
        }

        if !api_key_configured {
            setup_issues.push("视觉模型 API Key");
        }

        if !setup_issues.is_empty() {
            let issue_list = setup_issues
                .iter()
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
                .join("\n");

            return Ok(Self {
                status: "error".to_string(),
                kind: kind.to_string(),
                title: title.to_string(),
                content: format!("还不能开始{action_label}。\n请先补齐：\n{issue_list}"),
                detail: "本次没有读取截图区域，也没有发起 AI 请求。".to_string(),
                source_preview,
                source_char_count: 0,
                can_copy: false,
                can_retry: false,
                source_language: initial_source_language(),
                target_language: settings.translation_target_language.clone(),
                translation_direction: default_translation_direction(),
                can_switch_language: action == "translate",
            });
        }

        Ok(Self {
            status: "placeholder".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content: format!("{action_label}配置检查通过。"),
            detail: format!(
                "将使用当前视觉模型配置和 {} 秒超时；截图只会在本次请求中临时上传。",
                settings.ai_timeout_seconds
            ),
            source_preview,
            source_char_count: 0,
            can_copy: false,
            can_retry: false,
            source_language: initial_source_language(),
            target_language: settings.translation_target_language.clone(),
            translation_direction: default_translation_direction(),
            can_switch_language: action == "translate",
        })
    }

    fn image_loading(
        action: &str,
        source_preview: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
    ) -> Result<Self, String> {
        let (kind, title, action_label) = Self::image_meta(action)?;

        Ok(Self {
            status: "loading".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content: format!("正在进行{action_label}..."),
            detail: String::new(),
            source_preview,
            source_char_count: 0,
            can_copy: false,
            can_retry: false,
            source_language,
            target_language,
            translation_direction: normalize_translation_direction(&translation_direction),
            can_switch_language: action == "translate",
        })
    }

    fn image_success(
        action: &str,
        source_preview: String,
        content: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
    ) -> Result<Self, String> {
        let (kind, title, _) = Self::image_meta(action)?;

        Ok(Self {
            status: "success".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content,
            detail: String::new(),
            source_preview,
            source_char_count: 0,
            can_copy: true,
            can_retry: false,
            source_language,
            target_language,
            translation_direction: normalize_translation_direction(&translation_direction),
            can_switch_language: action == "translate",
        })
    }

    fn image_error(
        action: &str,
        source_preview: String,
        message: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
    ) -> Result<Self, String> {
        let (kind, title, _) = Self::image_meta(action)?;

        Ok(Self {
            status: "error".to_string(),
            kind: kind.to_string(),
            title: title.to_string(),
            content: message,
            detail: "未覆盖剪贴板，也未保存截图。请检查 AI 配置后重新截图。".to_string(),
            source_preview,
            source_char_count: 0,
            can_copy: false,
            can_retry: false,
            source_language,
            target_language,
            translation_direction: normalize_translation_direction(&translation_direction),
            can_switch_language: action == "translate",
        })
    }
}

#[tauri::command]
fn ping() -> &'static str {
    "QuickPick 核心已连接"
}

#[tauri::command]
fn get_selection_snapshot(state: tauri::State<'_, AppState>) -> SelectionSnapshot {
    state
        .selection_snapshot
        .lock()
        .map(|snapshot| snapshot.clone())
        .unwrap_or_else(|_| SelectionSnapshot::default())
}

#[tauri::command]
fn get_result_snapshot(state: tauri::State<'_, AppState>) -> ResultSnapshot {
    state
        .result_snapshot
        .lock()
        .map(|snapshot| snapshot.clone())
        .unwrap_or_else(|_| ResultSnapshot::default())
}

#[tauri::command]
fn get_app_settings(app: tauri::AppHandle) -> Result<app_settings::AppSettings, String> {
    app_settings::load_app_settings(&app)
}

#[tauri::command]
fn save_app_settings(
    app: tauri::AppHandle,
    settings: app_settings::AppSettings,
) -> Result<SettingsActionResult, String> {
    let previous_settings = app_settings::load_app_settings(&app).unwrap_or_default();
    let settings = app_settings::normalize_settings_for_save(settings)?;
    let hotkey_statuses = replace_global_shortcuts(&app, &settings)?;
    if let Err(error) = app_settings::save_app_settings(&app, &settings) {
        let _ = replace_global_shortcuts(&app, &previous_settings);
        return Err(error);
    }
    apply_autostart_setting(settings.autostart_enabled)?;
    sync_window_appearance(&app, &settings);
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = build_tray_menu(&app, settings.autostart_enabled, &settings.ui_language) {
            let _ = tray.set_menu(Some(menu));
        }
    }
    let _ = app.emit("app-settings-changed", settings.clone());

    Ok(SettingsActionResult {
        message: "设置已保存".to_string(),
        hotkey_statuses,
    })
}

#[tauri::command]
fn get_api_key_status(
    app: tauri::AppHandle,
    scope: Option<String>,
) -> Result<security::ApiKeyStatus, String> {
    security::api_key_status(&app, scope.as_deref().unwrap_or("text"))
}

#[tauri::command]
fn save_api_key(
    app: tauri::AppHandle,
    api_key: String,
    scope: Option<String>,
) -> Result<SelectionActionResult, String> {
    let scope = scope.unwrap_or_else(|| "text".to_string());
    security::save_api_key(&app, &scope, &api_key)?;

    Ok(SelectionActionResult {
        message: if scope == "vision" {
            "视觉模型 API Key 已加密保存".to_string()
        } else {
            "文本模型 API Key 已加密保存".to_string()
        },
    })
}

#[tauri::command]
fn clear_api_key(
    app: tauri::AppHandle,
    scope: Option<String>,
) -> Result<SelectionActionResult, String> {
    let scope = scope.unwrap_or_else(|| "text".to_string());
    security::clear_api_key(&app, &scope)?;

    Ok(SelectionActionResult {
        message: if scope == "vision" {
            "视觉模型 API Key 已清除".to_string()
        } else {
            "文本模型 API Key 已清除".to_string()
        },
    })
}

#[tauri::command]
fn copy_selection_text(state: tauri::State<'_, AppState>) -> Result<SelectionActionResult, String> {
    copy_selection_text_from_state(&state)
}

fn copy_selection_text_from_state(state: &AppState) -> Result<SelectionActionResult, String> {
    let text = current_selection_text(state)?;

    clipboard_win::set_clipboard_string(&text)
        .map_err(|_| error_key("clipboard.writeFailed"))?;

    Ok(SelectionActionResult {
        message: "已复制".to_string(),
    })
}

#[tauri::command]
fn search_selection_text(
    state: tauri::State<'_, AppState>,
) -> Result<SelectionActionResult, String> {
    search_selection_text_from_state(&state)
}

fn search_selection_text_from_state(state: &AppState) -> Result<SelectionActionResult, String> {
    const MAX_SEARCH_CHARS: usize = 500;

    let text = current_selection_text(state)?;
    if text.chars().count() > MAX_SEARCH_CHARS {
        return Err(error_key("search.tooLong"));
    }

    let url = format!(
        "https://www.bing.com/search?q={}",
        encode_query_component(&text)
    );

    if tauri_plugin_opener::open_url(url.clone(), None::<&str>).is_err() {
        let fallback = std::process::Command::new("cmd")
            .args(["/C", "start", "", url.as_str()])
            .spawn()
            .and_then(|mut child| child.wait());
        if fallback.is_err() {
            return Err(error_key("search.browserFailed"));
        }
    }

    Ok(SelectionActionResult {
        message: "已打开搜索".to_string(),
    })
}

#[tauri::command]
fn copy_result_content(state: tauri::State<'_, AppState>) -> Result<SelectionActionResult, String> {
    let snapshot = state
        .result_snapshot
        .lock()
        .map_err(|_| error_key("result.readFailed"))?;

    if snapshot.status != "success" || !snapshot.can_copy {
        return Err(error_key("result.noCopyable"));
    }

    let content = snapshot.content.trim();
    if content.is_empty() {
        return Err(error_key("result.emptyCopy"));
    }

    clipboard_win::set_clipboard_string(content)
        .map_err(|_| error_key("clipboard.writeFailed"))?;

    Ok(SelectionActionResult {
        message: "结果已复制".to_string(),
    })
}

#[tauri::command]
fn copy_input_result(state: tauri::State<'_, AppState>) -> Result<SelectionActionResult, String> {
    let result = state
        .last_input_result
        .lock()
        .map_err(|_| error_key("input.readResultFailed"))?
        .clone();

    let content = result.as_deref().unwrap_or_default().trim();
    if content.is_empty() {
        return Err(error_key("input.noCopyableResult"));
    }

    clipboard_win::set_clipboard_string(content)
        .map_err(|_| error_key("clipboard.writeFailed"))?;

    Ok(SelectionActionResult {
        message: "输入翻译结果已复制".to_string(),
    })
}

#[tauri::command]
async fn run_text_ai_action(
    app: tauri::AppHandle,
    action: String,
) -> Result<SelectionActionResult, String> {
    run_text_ai_action_inner(app, action).await
}

async fn run_text_ai_action_inner(
    app: tauri::AppHandle,
    action: String,
) -> Result<SelectionActionResult, String> {
    let state = app.state::<AppState>();
    let (text, source_preview, source_char_count) = match current_selection_payload(&state) {
        Ok(payload) => payload,
        Err(error) => {
            show_native_message_popup(
                &app,
                "划词 AI 失败".to_string(),
                error.clone(),
                "未读取到可处理文本，本次没有发起 AI 请求。".to_string(),
            );
            return Err(error);
        }
    };
    let settings = match app_settings::load_app_settings(&app) {
        Ok(settings) => settings,
        Err(error) => {
            show_native_message_popup(
                &app,
                "AI 配置读取失败".to_string(),
                error.clone(),
                "请在主窗口检查 AI 配置后重试。".to_string(),
            );
            return Err(error);
        }
    };
    let api_key_configured = match security::api_key_status(&app, "text") {
        Ok(status) => status.configured,
        Err(error) => {
            show_native_message_popup(
                &app,
                "API Key 状态读取失败".to_string(),
                error.clone(),
                "未读取 API Key 明文，本次没有发起 AI 请求。".to_string(),
            );
            return Err(error);
        }
    };
    let preflight_snapshot = ResultSnapshot::text_ai_preflight(
        &action,
        source_preview.clone(),
        source_char_count,
        &settings,
        api_key_configured,
    )?;

    if preflight_snapshot.status == "error" {
        show_native_result_popup(&app, &preflight_snapshot);
        set_result_snapshot(&state, preflight_snapshot)?;

        return Ok(SelectionActionResult {
            message: "请先补齐 AI 配置".to_string(),
        });
    }

    let source_language = initial_source_language();
    let target_language = settings.translation_target_language.clone();
    let translation_direction = default_translation_direction();
    let loading_snapshot = ResultSnapshot::text_loading(
        &action,
        source_preview.clone(),
        source_char_count,
        source_language.clone(),
        target_language.clone(),
        translation_direction.clone(),
    )?;
    if let Ok(mut last_text) = state.last_text_translation.lock() {
        *last_text = Some(LastTextTranslation {
            action: action.clone(),
            text: text.clone(),
            source_preview: source_preview.clone(),
            source_char_count,
        });
    }
    show_webview_result_snapshot(&app, &loading_snapshot);
    set_result_snapshot(&state, loading_snapshot)?;

    let ai_result = match security::load_api_key(&app, "text") {
        Ok(Some(mut api_key)) => {
            let result = ai::run_text_action(
                &settings,
                &api_key,
                &action,
                &text,
                &source_language,
                &target_language,
            )
            .await;
            api_key.clear();
            result
        }
        Ok(None) => Err(error_key("ai.textApiKeyMissing")),
        Err(error) => Err(error),
    };

    let final_snapshot = match ai_result {
        Ok(content) => ResultSnapshot::text_success(
            &action,
            source_preview.clone(),
            source_char_count,
            content,
            source_language.clone(),
            target_language.clone(),
            translation_direction.clone(),
        )?,
        Err(error) => ResultSnapshot::text_error(
            &action,
            source_preview.clone(),
            source_char_count,
            error,
            source_language.clone(),
            target_language.clone(),
            translation_direction.clone(),
        )?,
    };
    show_webview_result_snapshot(&app, &final_snapshot);
    set_result_snapshot(&state, final_snapshot)?;

    Ok(SelectionActionResult {
        message: "AI 请求已完成".to_string(),
    })
}

#[tauri::command]
fn clear_result_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<SelectionActionResult, String> {
    let mut current = state
        .result_snapshot
        .lock()
        .map_err(|_| error_key("result.clearFailed"))?;
    *current = ResultSnapshot::default();

    Ok(SelectionActionResult {
        message: "已关闭".to_string(),
    })
}

#[tauri::command]
fn capture_current_monitor_to_clipboard(
    app: tauri::AppHandle,
) -> Result<SelectionActionResult, String> {
    let result = screenshot::capture_current_monitor_to_clipboard()?;
    emit_screenshot_status(&app, "success", result.message.clone());

    Ok(SelectionActionResult {
        message: result.message,
    })
}

#[tauri::command]
fn capture_monitor_screenshot() -> Result<screenshot::MonitorScreenshotPayload, String> {
    screenshot::capture_current_monitor_screenshot()
}

#[tauri::command]
async fn capture_region_to_clipboard(
    app: tauri::AppHandle,
) -> Result<SelectionActionResult, String> {
    if !show_screenshot_overlay(&app) {
        return Err(error_key("screenshot.windowUnavailable"));
    }

    Ok(SelectionActionResult {
        message: "截图窗口已打开".to_string(),
    })
}

#[tauri::command]
fn set_hotkey_capture_mode(
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<SelectionActionResult, String> {
    state
        .hotkey_capture_mode
        .lock()
        .map(|mut current| {
            *current = enabled;
        })
        .map_err(|_| error_key("hotkey.captureToggleFailed"))?;

    Ok(SelectionActionResult {
        message: if enabled {
            "快捷键录制已开始".to_string()
        } else {
            "快捷键录制已结束".to_string()
        },
    })
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_window_controls::init())
        .plugin(tauri_plugin_frameless_window::init())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_selection_snapshot,
            get_result_snapshot,
            get_app_settings,
            save_app_settings,
            get_api_key_status,
            save_api_key,
            clear_api_key,
            copy_selection_text,
            search_selection_text,
            copy_result_content,
            copy_input_result,
            run_text_ai_action,
            clear_result_snapshot,
            capture_current_monitor_to_clipboard,
            capture_monitor_screenshot,
            capture_region_to_clipboard,
            set_hotkey_capture_mode,
            request_input_translation,
            request_result_translation,
            capture_region_rect
        ])
        .setup(|app| {
            let settings = app_settings::load_app_settings(app.handle()).unwrap_or_default();
            for label in [
                "selection",
                "result",
                "input",
                "screenshot_overlay",
            ] {
                if let Some(window) = app.get_webview_window(label) {
                    let _ = window.hide();
                }
            }
            if let Err(error) = apply_autostart_setting(settings.autostart_enabled) {
                eprintln!("QuickPick autostart sync skipped: {error}");
            }
            sync_window_appearance(app.handle(), &settings);
            setup_global_shortcuts(app.handle(), &settings)?;
            setup_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::CloseRequested { api, .. } if window.label() == "selection_bar" => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::CloseRequested { api, .. } if window.label() == "screenshot_overlay" => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::CloseRequested { api, .. } if window.label() == "result" => {
                api.prevent_close();
                if let Some(app) = window.app_handle().try_state::<AppState>() {
                    if let Ok(mut current) = app.result_snapshot.lock() {
                        *current = ResultSnapshot::default();
                    }
                }
                let _ = window.hide();
            }
            WindowEvent::CloseRequested { api, .. }
                if matches!(
                    window.label(),
                    "input" | "selection"
                ) =>
            {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::Focused(false) if window.label() == "selection_bar" => {
                let _ = window.hide();
            }
            WindowEvent::Focused(false) if window.label() == "selection" => {
                let _ = window.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running QuickPick");
}

fn current_selection_text(state: &AppState) -> Result<String, String> {
    let snapshot = state
        .selection_snapshot
        .lock()
        .map_err(|_| error_key("selection.readFailed"))?;
    let text = snapshot.text.trim();

    if text.is_empty() {
        Err(error_key("selection.noText"))
    } else {
        Ok(text.to_string())
    }
}

fn current_selection_payload(state: &AppState) -> Result<(String, String, usize), String> {
    let snapshot = state
        .selection_snapshot
        .lock()
        .map_err(|_| error_key("selection.readFailed"))?;
    let text = snapshot.text.trim();

    if text.is_empty() {
        Err(error_key("selection.noText"))
    } else {
        Ok((
            text.to_string(),
            snapshot.preview.clone(),
            snapshot.char_count,
        ))
    }
}

fn set_result_snapshot(state: &AppState, snapshot: ResultSnapshot) -> Result<(), String> {
    let mut current = state
        .result_snapshot
        .lock()
        .map_err(|_| error_key("result.writeFailed"))?;
    *current = snapshot;
    Ok(())
}

fn set_app_result_snapshot(app: &tauri::AppHandle, snapshot: ResultSnapshot) -> Result<(), String> {
    let state = app.state::<AppState>();
    set_result_snapshot(&state, snapshot)
}

fn encode_query_component(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char);
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}

fn setup_global_shortcuts(
    app: &tauri::AppHandle,
    settings: &app_settings::AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, event| {
                if event.state != ShortcutState::Released {
                    return;
                }
                let capture_mode = app
                    .state::<AppState>()
                    .hotkey_capture_mode
                    .lock()
                    .map(|enabled| *enabled)
                    .unwrap_or(false);
                if capture_mode {
                    return;
                }

                let action = app
                    .state::<AppState>()
                    .hotkey_bindings
                    .lock()
                    .ok()
                    .and_then(|bindings| bindings.action_for(shortcut));

                match action {
                    Some(HotkeyAction::Settings) => show_settings_window(app),
                    Some(HotkeyAction::Selection) => {
                        activate_selection_bar(app, shortcut_release_keys(shortcut))
                    }
                    Some(HotkeyAction::Screenshot) => capture_region_from_entry(app),
                    Some(HotkeyAction::InputTranslate) => {
                        activate_input_translate(app);
                    }
                    None => {}
                }
            })
            .build(),
    )?;

    if let Err(error) = replace_global_shortcuts(app, settings) {
        eprintln!("QuickPick global shortcut registration skipped: {error}");
    }

    Ok(())
}

fn replace_global_shortcuts(
    app: &tauri::AppHandle,
    settings: &app_settings::AppSettings,
) -> Result<Vec<HotkeyStatus>, String> {
    let next = HotkeyBindings::from_settings(settings)?;
    let previous = app
        .state::<AppState>()
        .hotkey_bindings
        .lock()
        .map(|bindings| bindings.clone())
        .map_err(|_| localized_error::error_key("hotkey.readStateFailed"))?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|_| localized_error::error_key("hotkey.unregisterFailed"))?;

    let statuses = match register_hotkey_bindings(app, &next) {
        Ok(statuses) => statuses,
        Err(error) => {
            let _ = register_hotkey_bindings(app, &previous);
            if let Ok(mut bindings) = app.state::<AppState>().hotkey_bindings.lock() {
                *bindings = previous;
            }
            return Err(error);
        }
    };

    app.state::<AppState>()
        .hotkey_bindings
        .lock()
        .map(|mut bindings| {
            let mut registered = next;
            if statuses.iter().any(|status| {
                status.name == "设置"
                    && !matches!(status.status, HotkeyRegistrationStatus::Registered)
            }) {
                registered.settings_id = None;
            }
            *bindings = registered;
            statuses
        })
        .map_err(|_| localized_error::error_key("hotkey.updateStateFailed"))
}

fn register_hotkey_bindings(
    app: &tauri::AppHandle,
    bindings: &HotkeyBindings,
) -> Result<Vec<HotkeyStatus>, String> {
    let mut statuses = Vec::new();
    for (name, shortcut) in [
        ("设置", bindings.settings_hotkey.as_str()),
        ("划词菜单", bindings.selection_hotkey.as_str()),
        ("区域截图", bindings.screenshot_hotkey.as_str()),
        ("输入翻译", bindings.input_translate_hotkey.as_str()),
    ] {
        match app.global_shortcut().register(shortcut) {
            Ok(()) => statuses.push(HotkeyStatus {
                name: name.to_string(),
                status: HotkeyRegistrationStatus::Registered,
            }),
            Err(error) if name == "设置" => {
                eprintln!("QuickPick settings hotkey registration skipped: {error}");
                statuses.push(HotkeyStatus {
                    name: name.to_string(),
                    status: HotkeyRegistrationStatus::Occupied,
                });
            }
            Err(error) => {
                return Err(localized_error::error_key_with_detail(
                    "hotkey.registerFailed",
                    &format!("{name}:{error}"),
                ));
            }
        }
    }

    Ok(statuses)
}

fn sync_window_appearance(app: &tauri::AppHandle, settings: &app_settings::AppSettings) {
    if let Err(error) = apply_window_appearance(app, settings) {
        eprintln!("QuickPick window appearance sync skipped: {error}");
    }
}

fn apply_window_appearance(
    app: &tauri::AppHandle,
    settings: &app_settings::AppSettings,
) -> Result<(), String> {
    let theme = match settings.theme_mode.as_str() {
        "light" => Some(Theme::Light),
        "dark" | "workbench" => Some(Theme::Dark),
        _ => None,
    };
    app.set_theme(theme);

    for label in ["main", "selection", "result", "input"] {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        let _ = window.set_decorations(false);
        let _ = window.set_title(&localized_window_title(label, &settings.ui_language));
        if let Err(error) = window.set_theme(theme) {
            eprintln!("QuickPick {label} window theme sync skipped: {error}");
        }
        let _ = window.set_shadow(true);
        if let Err(error) = apply_window_vibrancy(&window, settings) {
            eprintln!("QuickPick {label} window vibrancy sync skipped: {error}");
        }
    }

    Ok(())
}

fn apply_window_vibrancy(
    window: &tauri::WebviewWindow,
    settings: &app_settings::AppSettings,
) -> Result<(), String> {
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));

    if settings.window_effect.as_str() == "mica" {
        let effect = match settings.theme_mode.as_str() {
            "light" => Effect::MicaLight,
            "dark" | "workbench" => Effect::MicaDark,
            _ => Effect::Mica,
        };
        window
            .set_effects(EffectsBuilder::new().effect(effect).build())
            .map_err(|error| {
                localized_error::error_key_with_detail("windowEffect.micaFailed", &error.to_string())
            })?;
        return Ok(());
    }

    window_vibrancy::apply_blur(window, None)
        .map_err(|error| {
            localized_error::error_key_with_detail(
                "windowEffect.acrylicFailed",
                &error.to_string(),
            )
        })
}

fn refresh_current_window_glass(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let settings = app_settings::load_app_settings(app).unwrap_or_default();
    if let Err(error) = apply_window_vibrancy(window, &settings) {
        eprintln!("QuickPick window vibrancy refresh skipped: {error}");
    }
}

fn shortcut_release_keys(shortcut: &Shortcut) -> Vec<i32> {
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
    if shortcut.mods.contains(Modifiers::SUPER) {
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

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
    let settings = app_settings::load_app_settings(app).unwrap_or_default();
    let tray_menu = build_tray_menu(app, settings.autostart_enabled, &settings.ui_language)?;

    let tray = TrayIconBuilder::new()
        .tooltip("QuickPick")
        .icon(icon)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                show_settings_window(tray.app_handle());
            }
        })
        .build(app)?;
    tray.set_menu(Some(tray_menu))?;
    tray.on_menu_event(|app, event| match event.id().as_ref() {
        "settings" => show_settings_window(app),
        "selection" => activate_selection_bar(app, Vec::new()),
        "screenshot" => capture_region_from_entry(app),
        "input" => activate_input_translate(app),
        "reset-hotkeys" => reset_global_hotkeys_from_tray(app),
        "autostart" => toggle_autostart_from_tray(app),
        "quit" => app.exit(0),
        _ => {}
    });

    Ok(())
}

fn build_tray_menu(
    app: &tauri::AppHandle,
    autostart_enabled: bool,
    ui_language: &str,
) -> tauri::Result<Menu<tauri::Wry>> {
    use tauri::menu::PredefinedMenuItem;

    let settings = MenuItem::with_id(
        app,
        "settings",
        localized_label(ui_language, "settings"),
        true,
        None::<&str>,
    )?;
    let selection = MenuItem::with_id(
        app,
        "selection",
        localized_label(ui_language, "selection_menu"),
        true,
        None::<&str>,
    )?;
    let screenshot = MenuItem::with_id(
        app,
        "screenshot",
        localized_label(ui_language, "region_screenshot"),
        true,
        None::<&str>,
    )?;
    let input = MenuItem::with_id(
        app,
        "input",
        localized_label(ui_language, "input_translation"),
        true,
        None::<&str>,
    )?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        localized_label(ui_language, "start_on_boot"),
        true,
        autostart_enabled,
        None::<&str>,
    )?;
    let reset_hotkeys = MenuItem::with_id(
        app,
        "reset-hotkeys",
        localized_label(ui_language, "reset_hotkeys"),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        localized_label(ui_language, "quit"),
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &settings,
            &separator,
            &selection,
            &screenshot,
            &input,
            &separator,
            &autostart,
            &reset_hotkeys,
            &separator,
            &quit,
        ],
    )?;
    Ok(menu)
}

fn localized_label<'a>(language: &'a str, key: &'a str) -> &'a str {
    let resolved = resolved_ui_language(language);
    match resolved.as_str() {
        "zh-Hant" => match key {
            "settings" => "設定",
            "selection_menu" => "劃詞選單",
            "region_screenshot" => "區域截圖",
            "input_translation" => "輸入翻譯",
            "start_on_boot" => "開機自啟",
            "reset_hotkeys" => "重設快捷鍵",
            "quit" => "結束",
            "selection" => "劃詞",
            "result" => "結果",
            "input" => "輸入翻譯",
            _ => key,
        },
        "en" => match key {
            "settings" => "Settings",
            "selection_menu" => "Selection Menu",
            "region_screenshot" => "Region Screenshot",
            "input_translation" => "Input Translation",
            "start_on_boot" => "Start on Boot",
            "reset_hotkeys" => "Reset Hotkeys",
            "quit" => "Quit",
            "selection" => "Selection",
            "result" => "Result",
            "input" => "Input Translation",
            _ => key,
        },
        "ko" => match key {
            "settings" => "설정",
            "selection_menu" => "선택 메뉴",
            "region_screenshot" => "영역 캡처",
            "input_translation" => "입력 번역",
            "start_on_boot" => "부팅 시 시작",
            "reset_hotkeys" => "단축키 초기화",
            "quit" => "종료",
            "selection" => "선택",
            "result" => "결과",
            "input" => "입력 번역",
            _ => key,
        },
        "ja" => match key {
            "settings" => "設定",
            "selection_menu" => "選択メニュー",
            "region_screenshot" => "領域スクリーンショット",
            "input_translation" => "入力翻訳",
            "start_on_boot" => "起動時に開始",
            "reset_hotkeys" => "ショートカットをリセット",
            "quit" => "終了",
            "selection" => "選択",
            "result" => "結果",
            "input" => "入力翻訳",
            _ => key,
        },
        "fr" => match key {
            "settings" => "Paramètres",
            "selection_menu" => "Menu de sélection",
            "region_screenshot" => "Capture de région",
            "input_translation" => "Traduction de saisie",
            "start_on_boot" => "Démarrage automatique",
            "reset_hotkeys" => "Réinitialiser les raccourcis",
            "quit" => "Quitter",
            "selection" => "Sélection",
            "result" => "Résultat",
            "input" => "Traduction de saisie",
            _ => key,
        },
        "de" => match key {
            "settings" => "Einstellungen",
            "selection_menu" => "Auswahlmenü",
            "region_screenshot" => "Bereichs-Screenshot",
            "input_translation" => "Eingabeübersetzung",
            "start_on_boot" => "Beim Start ausführen",
            "reset_hotkeys" => "Tastenkürzel zurücksetzen",
            "quit" => "Beenden",
            "selection" => "Auswahl",
            "result" => "Ergebnis",
            "input" => "Eingabeübersetzung",
            _ => key,
        },
        "es" => match key {
            "settings" => "Configuración",
            "selection_menu" => "Menú de selección",
            "region_screenshot" => "Captura de región",
            "input_translation" => "Traducción de entrada",
            "start_on_boot" => "Iniciar con el sistema",
            "reset_hotkeys" => "Restablecer atajos",
            "quit" => "Salir",
            "selection" => "Selección",
            "result" => "Resultado",
            "input" => "Traducción de entrada",
            _ => key,
        },
        _ => match key {
            "settings" => "设置",
            "selection_menu" => "划词菜单",
            "region_screenshot" => "区域截图",
            "input_translation" => "输入翻译",
            "start_on_boot" => "自启动",
            "reset_hotkeys" => "重置快捷键",
            "quit" => "退出",
            "selection" => "划词",
            "result" => "结果",
            "input" => "输入翻译",
            _ => key,
        },
    }
}

fn localized_window_title(label: &str, language: &str) -> String {
    let resolved = resolved_ui_language(language);
    match label {
        "main" => format!("QuickPick {}", localized_label(&resolved, "settings")),
        "selection" => format!("QuickPick {}", localized_label(&resolved, "selection")),
        "result" => format!("QuickPick {}", localized_label(&resolved, "result")),
        "input" => format!("QuickPick {}", localized_label(&resolved, "input")),
        _ => format!("QuickPick {label}"),
    }
}

fn resolved_ui_language(language: &str) -> String {
    if language != "system" {
        return language.to_string();
    }

    let Some(locale) = sys_locale::get_locale() else {
        return "zh-Hans".to_string();
    };
    let locale = locale.to_ascii_lowercase();
    if locale.starts_with("zh") {
        if locale.starts_with("zh-tw")
            || locale.starts_with("zh-hk")
            || locale.starts_with("zh-mo")
        {
            "zh-Hant".to_string()
        } else {
            "zh-Hans".to_string()
        }
    } else if locale.starts_with("en") {
        "en".to_string()
    } else if locale.starts_with("ko") {
        "ko".to_string()
    } else if locale.starts_with("ja") {
        "ja".to_string()
    } else if locale.starts_with("fr") {
        "fr".to_string()
    } else if locale.starts_with("de") {
        "de".to_string()
    } else if locale.starts_with("es") {
        "es".to_string()
    } else {
        "zh-Hans".to_string()
    }
}

fn reset_global_hotkeys_from_tray(app: &tauri::AppHandle) {
    if let Ok(mut capture_mode) = app.state::<AppState>().hotkey_capture_mode.lock() {
        *capture_mode = false;
    }

    let settings = app_settings::load_app_settings(app).unwrap_or_default();
    if let Err(error) = replace_global_shortcuts(app, &settings) {
        eprintln!("QuickPick hotkey reset failed: {error}");
    }
}

fn toggle_autostart_from_tray(app: &tauri::AppHandle) {
    let mut settings = match app_settings::load_app_settings(app) {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("QuickPick autostart toggle skipped: {error}");
            return;
        }
    };
    settings.autostart_enabled = !settings.autostart_enabled;

    match app_settings::save_app_settings(app, &settings)
        .and_then(|_| apply_autostart_setting(settings.autostart_enabled))
    {
        Ok(()) => {
            if let Some(tray) = app.tray_by_id("main") {
                if let Ok(menu) =
                    build_tray_menu(app, settings.autostart_enabled, &settings.ui_language)
                {
                    let _ = tray.set_menu(Some(menu));
                }
            }
        }
        Err(error) => {
            eprintln!("QuickPick autostart toggle failed: {error}");
        }
    }
}

fn capture_region_from_entry(app: &tauri::AppHandle) {
    if show_screenshot_overlay(app) {
        return;
    }

    eprintln!("QuickPick screenshot WebView window is unavailable");
}

fn show_screenshot_overlay(app: &tauri::AppHandle) -> bool {
    let Some(window) = app.get_webview_window("screenshot_overlay") else {
        return false;
    };
    match screenshot::capture_current_monitor_screenshot() {
        Ok(payload) => {
            let _ = app.emit_to("screenshot_overlay", "screenshot-ready", payload);
        }
        Err(error) => {
            let _ = app.emit_to("screenshot_overlay", "screenshot-error", error);
        }
    }
    let _ = window.set_always_on_top(true);
    let _ = window.show();
    let _ = window.set_focus();
    true
}

#[tauri::command(rename_all = "camelCase")]
async fn capture_region_rect(
    app: tauri::AppHandle,
    screen_x: i32,
    screen_y: i32,
    width: u32,
    height: u32,
    action: String,
) -> Result<SelectionActionResult, String> {
    let action = match action.as_str() {
        "extract" => screenshot::RegionMenuAction::Extract,
        "translate" => screenshot::RegionMenuAction::Translate,
        _ => screenshot::RegionMenuAction::Copy,
    };
    let selection = screenshot::RegionMenuSelection {
        action,
        screen_x,
        screen_y,
        width,
        height,
    };
    handle_region_menu_selection(app, selection).await
}

async fn handle_region_menu_selection(
    app: tauri::AppHandle,
    selection: screenshot::RegionMenuSelection,
) -> Result<SelectionActionResult, String> {
    match selection.action {
        screenshot::RegionMenuAction::Copy => {
            let image = screenshot::capture_selected_region(selection)?;
            let result = screenshot::copy_captured_region_to_clipboard(&image)?;
            emit_screenshot_status(&app, "success", result.message.clone());

            Ok(SelectionActionResult {
                message: result.message,
            })
        }
        screenshot::RegionMenuAction::Extract | screenshot::RegionMenuAction::Translate => {
            let action = match selection.action {
                screenshot::RegionMenuAction::Extract => "extract",
                screenshot::RegionMenuAction::Translate => "translate",
                screenshot::RegionMenuAction::Copy => unreachable!(),
            };
            run_screenshot_ai_action(app, selection, action).await
        }
    }
}

async fn run_screenshot_ai_action(
    app: tauri::AppHandle,
    selection: screenshot::RegionMenuSelection,
    action: &'static str,
) -> Result<SelectionActionResult, String> {
    let source_preview = format!("区域截图 {}x{}", selection.width, selection.height);
    let settings = match app_settings::load_app_settings(&app) {
        Ok(settings) => settings,
        Err(error) => {
            show_native_message_popup(
                &app,
                "AI 配置读取失败".to_string(),
                error.clone(),
                "请在主窗口检查 AI 配置后重试。".to_string(),
            );
            return Err(error);
        }
    };
    let api_key_configured = match security::api_key_status(&app, "vision") {
        Ok(status) => status.configured,
        Err(error) => {
            show_native_message_popup(
                &app,
                "API Key 状态读取失败".to_string(),
                error.clone(),
                "未读取 API Key 明文，本次没有发起 AI 请求。".to_string(),
            );
            return Err(error);
        }
    };
    let preflight = ResultSnapshot::image_ai_preflight(
        action,
        source_preview.clone(),
        &settings,
        api_key_configured,
    )?;

    if preflight.status == "error" {
        show_native_result_popup(&app, &preflight);
        set_app_result_snapshot(&app, preflight)?;
        emit_screenshot_status(&app, "error", "请先补齐 AI 配置".to_string());

        return Ok(SelectionActionResult {
            message: "请先补齐 AI 配置".to_string(),
        });
    }

    let source_language = initial_source_language();
    let target_language = settings.translation_target_language.clone();
    let translation_direction = default_translation_direction();
    let loading = ResultSnapshot::image_loading(
        action,
        source_preview.clone(),
        source_language.clone(),
        target_language.clone(),
        translation_direction.clone(),
    )?;
    show_webview_result_snapshot(&app, &loading);
    set_app_result_snapshot(&app, loading)?;
    emit_screenshot_status(&app, "success", "正在读取截图区域".to_string());

    let image = match screenshot::capture_selected_region(selection) {
        Ok(image) => image,
        Err(error) => {
            let snapshot = ResultSnapshot::image_error(
                action,
                source_preview,
                error.clone(),
                source_language.clone(),
                target_language.clone(),
                translation_direction.clone(),
            )?;
            show_webview_result_snapshot(&app, &snapshot);
            set_app_result_snapshot(&app, snapshot)?;
            emit_screenshot_status(&app, "error", error.clone());
            return Err(error);
        }
    };

    let source_preview = format!(
        "区域截图 {}x{}（{}）",
        image.width, image.height, image.monitor_name
    );
    if let Ok(mut last_image) = app.state::<AppState>().last_image_translation.lock() {
        *last_image = Some(LastImageTranslation {
            action: action.to_string(),
            png_bytes: image.png_bytes.clone(),
            source_preview: source_preview.clone(),
        });
    }
    let loading = ResultSnapshot::image_loading(
        action,
        source_preview.clone(),
        source_language.clone(),
        target_language.clone(),
        translation_direction.clone(),
    )?;
    show_webview_result_snapshot(&app, &loading);
    set_app_result_snapshot(&app, loading)?;
    emit_screenshot_status(&app, "success", "正在处理截图文字".to_string());

    let ai_result = match security::load_api_key(&app, "vision") {
        Ok(Some(mut api_key)) => {
            let result = ai::run_image_action(
                &settings,
                &api_key,
                action,
                &image.png_bytes,
                &source_language,
                &target_language,
            )
            .await;
            api_key.clear();
            result
        }
        Ok(None) => Err("视觉模型 API Key 未配置，请先保存".to_string()),
        Err(error) => Err(error),
    };

    let (kind, message, final_snapshot) = match ai_result {
        Ok(content) => (
            "success",
            "图片 AI 请求已完成".to_string(),
            ResultSnapshot::image_success(
                action,
                source_preview.clone(),
                content,
                source_language.clone(),
                target_language.clone(),
                translation_direction.clone(),
            )?,
        ),
        Err(error) => (
            "error",
            "图片 AI 请求失败".to_string(),
            ResultSnapshot::image_error(
                action,
                source_preview.clone(),
                error,
                source_language.clone(),
                target_language.clone(),
                translation_direction.clone(),
            )?,
        ),
    };
    show_webview_result_snapshot(&app, &final_snapshot);
    set_app_result_snapshot(&app, final_snapshot)?;
    emit_screenshot_status(&app, kind, message.clone());

    Ok(SelectionActionResult { message })
}

fn emit_screenshot_status(app: &tauri::AppHandle, kind: &'static str, message: String) {
    let _ = app.emit_to(
        "main",
        "screenshot-status-updated",
        ScreenshotStatusEvent { kind, message },
    );
}

fn activate_selection_bar(app: &tauri::AppHandle, hotkey_keys: Vec<i32>) {
    let foreground_window = selection::current_foreground_window();
    let app = app.clone();
    std::thread::spawn(move || {
        refresh_selection_snapshot(&app, foreground_window, hotkey_keys);
        let _ = show_selection_webview(&app);
    });
}

fn show_selection_webview(app: &tauri::AppHandle) -> bool {
    let Some(window) = app.get_webview_window("selection") else {
        return false;
    };
    #[cfg(windows)]
    {
        use windows_sys::Win32::{
            Foundation::POINT,
            UI::WindowsAndMessaging::{
                GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
            },
        };

        let mut cursor = POINT { x: 0, y: 0 };
        unsafe {
            GetCursorPos(&mut cursor);
        }
        let window_width = 480;
        let window_height = 58;
        let max_x =
            (unsafe { GetSystemMetrics(SM_CXSCREEN) - window_width - 8 }).max(8);
        let max_y =
            (unsafe { GetSystemMetrics(SM_CYSCREEN) - window_height - 8 }).max(8);
        let below_y = cursor.y + 18;
        let y = if below_y > max_y {
            (cursor.y - window_height - 18).clamp(8, max_y)
        } else {
            below_y.clamp(8, max_y)
        };
        let _ = window.set_position(tauri::PhysicalPosition::new(
            cursor.x.clamp(8, max_x),
            y,
        ));
    }
    let _ = window.set_always_on_top(true);
    let _ = window.show();
    refresh_current_window_glass(app, &window);
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                SetForegroundWindow(hwnd.0);
            }
        }
    }
    let _ = window.set_focus();
    true
}

fn activate_input_translate(app: &tauri::AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let _ = tauri::async_runtime::block_on(run_input_translate_flow(app.clone()));
    });
}

fn show_input_webview(
    app: &tauri::AppHandle,
    source_language: String,
    target_language: String,
    direction: String,
) -> bool {
    let Some(window) = app.get_webview_window("input") else {
        return false;
    };

    let _ = window.set_always_on_top(true);
    let _ = window.show();
    refresh_current_window_glass(app, &window);
    let _ = window.set_focus();
    let _ = app.emit_to(
        "input",
        "input-ready",
        InputReadyPayload {
            status: "waiting".to_string(),
            content: String::new(),
            detail: String::new(),
            source_language,
            target_language,
            direction,
        },
    );
    true
}

async fn run_input_translate_flow(app: tauri::AppHandle) -> Result<SelectionActionResult, String> {
    let settings = match app_settings::load_app_settings(&app) {
        Ok(settings) => settings,
        Err(error) => {
            show_native_message_popup(
                &app,
                "输入翻译配置读取失败".to_string(),
                error.clone(),
                "请到设置页检查输入模型配置。".to_string(),
            );
            return Err(error);
        }
    };
    let api_key_configured = match security::api_key_status(&app, "input") {
        Ok(status) => status.configured,
        Err(error) => {
            show_native_message_popup(
                &app,
                "输入模型 Key 状态读取失败".to_string(),
                error.clone(),
                "未读取 API Key 明文，本次没有发起 AI 请求。".to_string(),
            );
            return Err(error);
        }
    };
    let input_provider = settings.input_ai_provider.trim();
    let needs_explicit_base_url =
        input_provider == "openai_compatible" && settings.input_ai_base_url.trim().is_empty();
    let needs_explicit_model =
        input_provider == "openai_compatible" && settings.input_ai_model.trim().is_empty();
    if !api_key_configured || needs_explicit_base_url || needs_explicit_model {
        show_native_message_popup(
            &app,
            "输入模型配置不完整".to_string(),
            "请先到设置页完成输入模型供应商、Base URL、模型和 API Key 配置。".to_string(),
            "本次没有发起 AI 请求。".to_string(),
        );
        return Ok(SelectionActionResult {
            message: "请先补齐输入模型配置".to_string(),
        });
    }

    let source_language = settings.input_translate_source_language.clone();
    let target_language = settings.input_translate_target_language.clone();
    let translation_direction = default_translation_direction();
    if show_input_webview(
        &app,
        source_language.clone(),
        target_language.clone(),
        translation_direction.clone(),
    ) {
        return Ok(SelectionActionResult {
            message: "输入翻译窗口已打开".to_string(),
        });
    }

    Err(error_key("input.windowUnavailable"))
}

#[tauri::command(rename_all = "camelCase")]
async fn request_input_translation(
    app: tauri::AppHandle,
    input_text: String,
    source_language: String,
    target_language: String,
    direction: String,
) -> Result<SelectionActionResult, String> {
    let settings = app_settings::load_app_settings(&app)?;
    let api_key_configured = security::api_key_status(&app, "input")?.configured;
    let input_provider = settings.input_ai_provider.trim();
    let needs_explicit_base_url =
        input_provider == "openai_compatible" && settings.input_ai_base_url.trim().is_empty();
    let needs_explicit_model =
        input_provider == "openai_compatible" && settings.input_ai_model.trim().is_empty();
    let trimmed = input_text.trim().to_string();

    if let Ok(mut current) = app.state::<AppState>().last_input_result.lock() {
        *current = None;
    }

    let (status, content, detail) = if !api_key_configured || needs_explicit_base_url || needs_explicit_model {
        (
            "error".to_string(),
            String::new(),
            "请先到设置页完成输入模型配置。".to_string(),
        )
    } else if trimmed.is_empty() {
        (
            "error".to_string(),
            String::new(),
            "请输入要翻译的文本。".to_string(),
        )
    } else {
        let (request_source_language, request_target_language) =
            translation_request_languages(&source_language, &target_language, &direction);
        let ai_result = match security::load_api_key(&app, "input") {
            Ok(Some(mut api_key)) => {
                let result = ai::run_input_text_action(
                    &settings,
                    &api_key,
                    "translate",
                    &trimmed,
                    &request_source_language,
                    &request_target_language,
                )
                .await;
                api_key.clear();
                result
            }
            Ok(None) => Err(error_key("input.apiKeyMissing")),
            Err(error) => Err(error),
        };

        match ai_result {
            Ok(content) => {
                if let Ok(mut current) = app.state::<AppState>().last_input_result.lock() {
                    *current = Some(content.clone());
                }
                ("success".to_string(), content, String::new())
            }
            Err(error) => ("error".to_string(), String::new(), error),
        }
    };

    let _ = app.emit_to(
        "input",
        "input-ready",
        InputReadyPayload {
            status,
            content,
            detail,
            source_language,
            target_language,
            direction,
        },
    );

    Ok(SelectionActionResult {
        message: "输入翻译已请求".to_string(),
    })
}

#[tauri::command(rename_all = "camelCase")]
async fn request_result_translation(
    app: tauri::AppHandle,
    source_language: String,
    target_language: String,
    direction: String,
) -> Result<SelectionActionResult, String> {
    let settings = app_settings::load_app_settings(&app)?;
    let (request_source_language, request_target_language) =
        translation_request_languages(&source_language, &target_language, &direction);
    let state = app.state::<AppState>();
    let text_payload = state
        .last_text_translation
        .lock()
        .map_err(|_| error_key("result.readContextFailed"))?
        .clone();

    if let Some(payload) = text_payload {
        let ai_result = match security::load_api_key(&app, "text") {
            Ok(Some(mut api_key)) => {
                let result = ai::run_text_action(
                    &settings,
                    &api_key,
                    &payload.action,
                    &payload.text,
                    &request_source_language,
                    &request_target_language,
                )
                .await;
                api_key.clear();
                result
            }
            Ok(None) => Err(error_key("ai.textApiKeyMissing")),
            Err(error) => Err(error),
        };
        let snapshot = match ai_result {
            Ok(content) => ResultSnapshot::text_success(
                &payload.action,
                payload.source_preview,
                payload.source_char_count,
                content,
                source_language.clone(),
                target_language.clone(),
                direction.clone(),
            )?,
            Err(error) => ResultSnapshot::text_error(
                &payload.action,
                payload.source_preview,
                payload.source_char_count,
                error,
                source_language.clone(),
                target_language.clone(),
                direction.clone(),
            )?,
        };
        set_app_result_snapshot(&app, snapshot.clone())?;
        let _ = app.emit_to("result", "result-ready", snapshot);
        return Ok(SelectionActionResult {
            message: "结果翻译已更新".to_string(),
        });
    }

    let image_payload = state
        .last_image_translation
        .lock()
        .map_err(|_| error_key("screenshot.readContextFailed"))?
        .clone();
    if let Some(payload) = image_payload {
        let ai_result = match security::load_api_key(&app, "vision") {
            Ok(Some(mut api_key)) => {
                let result = ai::run_image_action(
                    &settings,
                    &api_key,
                    &payload.action,
                    &payload.png_bytes,
                    &request_source_language,
                    &request_target_language,
                )
                .await;
                api_key.clear();
                result
            }
            Ok(None) => Err(error_key("screenshot.apiKeyMissing")),
            Err(error) => Err(error),
        };
        let snapshot = match ai_result {
            Ok(content) => ResultSnapshot::image_success(
                &payload.action,
                payload.source_preview,
                content,
                source_language.clone(),
                target_language.clone(),
                direction.clone(),
            )?,
            Err(error) => ResultSnapshot::image_error(
                &payload.action,
                payload.source_preview,
                error,
                source_language.clone(),
                target_language.clone(),
                direction.clone(),
            )?,
        };
        set_app_result_snapshot(&app, snapshot.clone())?;
        let _ = app.emit_to("result", "result-ready", snapshot);
        return Ok(SelectionActionResult {
            message: "截图翻译已更新".to_string(),
        });
    }

    Err(error_key("result.languageSwitchUnsupported"))
}

fn refresh_selection_snapshot(
    app: &tauri::AppHandle,
    foreground_window: selection::ForegroundWindow,
    hotkey_keys: Vec<i32>,
) {
    let snapshot = selection::capture_selected_text_for_window(foreground_window, hotkey_keys);
    let state = app.state::<AppState>();

    if let Ok(mut current) = state.selection_snapshot.lock() {
        *current = snapshot;
    };
}

fn show_native_message_popup(
    app: &tauri::AppHandle,
    title: String,
    content: String,
    detail: String,
) {
    let snapshot = ResultSnapshot {
        status: "error".to_string(),
        kind: "message".to_string(),
        title,
        content,
        detail,
        source_preview: String::new(),
        source_char_count: 0,
        can_copy: false,
        can_retry: false,
        source_language: default_translation_source_language(),
        target_language: "zh-Hans".to_string(),
        translation_direction: default_translation_direction(),
        can_switch_language: false,
    };
    show_webview_result_snapshot(app, &snapshot);
}

fn show_native_result_popup(app: &tauri::AppHandle, snapshot: &ResultSnapshot) {
    show_webview_result_snapshot(app, snapshot);
}

fn show_webview_result_snapshot(app: &tauri::AppHandle, snapshot: &ResultSnapshot) {
    let Some(window) = app.get_webview_window("result") else {
        return;
    };

    let _ = window.set_title(&snapshot.title);
    let _ = window.set_always_on_top(true);
    let _ = window.show();
    refresh_current_window_glass(app, &window);
    let _ = window.set_focus();
    let _ = app.emit_to("result", "result-ready", snapshot.clone());
}

fn apply_autostart_setting(enabled: bool) -> Result<(), String> {
    if enabled {
        enable_autostart()
    } else {
        disable_autostart()
    }
}

#[cfg(windows)]
fn enable_autostart() -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_SET_VALUE,
            REG_OPTION_NON_VOLATILE, REG_SZ,
        },
    };

    let current_exe =
        std::env::current_exe().map_err(|_| error_key("autostart.locateFailed"))?;
    let command = format!("\"{}\"", current_exe.to_string_lossy());
    let key_path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = wide_null("QuickPick");
    let value_data = wide_null(&command);
    let mut key = std::ptr::null_mut();

    let create_status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            key_path.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    if create_status != ERROR_SUCCESS {
        return Err(localized_error::error_key_with_detail(
            "autostart.openFailed",
            &create_status.to_string(),
        ));
    }

    let set_status = unsafe {
        RegSetValueExW(
            key,
            value_name.as_ptr(),
            0,
            REG_SZ,
            value_data.as_ptr() as *const u8,
            (value_data.len() * std::mem::size_of::<u16>()) as u32,
        )
    };
    unsafe {
        RegCloseKey(key);
    }

    if set_status != ERROR_SUCCESS {
        return Err(localized_error::error_key_with_detail(
            "autostart.writeFailed",
            &set_status.to_string(),
        ));
    }

    Ok(())
}

#[cfg(not(windows))]
fn enable_autostart() -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn disable_autostart() -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            RegCloseKey, RegCreateKeyExW, RegDeleteValueW, HKEY_CURRENT_USER, KEY_SET_VALUE,
            REG_OPTION_NON_VOLATILE,
        },
    };

    let key_path = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = wide_null("QuickPick");
    let mut key = std::ptr::null_mut();

    let create_status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            key_path.as_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    if create_status != ERROR_SUCCESS {
        return Err(localized_error::error_key_with_detail(
            "autostart.openFailed",
            &create_status.to_string(),
        ));
    }

    let delete_status = unsafe { RegDeleteValueW(key, value_name.as_ptr()) };
    unsafe {
        RegCloseKey(key);
    }

    if delete_status != ERROR_SUCCESS && delete_status != ERROR_FILE_NOT_FOUND {
        return Err(localized_error::error_key_with_detail(
            "autostart.deleteFailed",
            &delete_status.to_string(),
        ));
    }

    Ok(())
}

#[cfg(not(windows))]
fn disable_autostart() -> Result<(), String> {
    Ok(())
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn show_settings_window(app: &tauri::AppHandle) {
    if let Ok(settings) = app_settings::load_app_settings(app) {
        sync_window_appearance(app, &settings);
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(&localized_window_title(
            "main",
            &app_settings::load_app_settings(app)
                .map(|settings| settings.ui_language)
                .unwrap_or_default(),
        ));
        let _ = window.set_decorations(false);
        let _ = window.unminimize();
        let _ = window.show();
        refresh_current_window_glass(app, &window);
        let _ = window.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_hotkey_alt_4_waits_for_alt_and_digit_4() {
        let shortcut = app_settings::parse_hotkey("Alt+4", "划词菜单").unwrap();
        let keys = shortcut_release_keys(&shortcut);

        assert!(keys.contains(&0x12));
        assert!(keys.contains(&0x34));
    }

    #[test]
    fn settings_hotkey_maps_to_settings_action() {
        let bindings = HotkeyBindings::default();
        let shortcut = app_settings::parse_hotkey("Alt+0", "设置").unwrap();

        assert!(matches!(
            bindings.action_for(&shortcut),
            Some(HotkeyAction::Settings)
        ));
    }

    #[test]
    fn selection_hotkey_with_ctrl_waits_for_all_modifiers_and_main_key() {
        let shortcut = app_settings::parse_hotkey("Ctrl+Alt+Q", "划词菜单").unwrap();
        let keys = shortcut_release_keys(&shortcut);

        assert!(keys.contains(&0x11));
        assert!(keys.contains(&0x12));
        assert!(keys.contains(&0x51));
    }
}
