mod ai;
mod app_settings;
mod native_popup;
mod native_theme;
mod screenshot;
mod security;
mod selection;

use selection::SelectionSnapshot;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    window::{Effect, EffectsBuilder},
    Emitter, Manager, Theme, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

struct AppState {
    selection_snapshot: Mutex<SelectionSnapshot>,
    result_snapshot: Mutex<ResultSnapshot>,
    hotkey_bindings: Mutex<HotkeyBindings>,
    hotkey_capture_mode: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            selection_snapshot: Mutex::new(SelectionSnapshot::default()),
            result_snapshot: Mutex::new(ResultSnapshot::default()),
            hotkey_bindings: Mutex::new(HotkeyBindings::default()),
            hotkey_capture_mode: Mutex::new(false),
        }
    }
}

#[derive(Clone)]
struct HotkeyBindings {
    selection_hotkey: String,
    screenshot_hotkey: String,
    input_translate_hotkey: String,
    selection_id: u32,
    screenshot_id: u32,
    input_translate_id: u32,
}

impl HotkeyBindings {
    fn from_settings(settings: &app_settings::AppSettings) -> Result<Self, String> {
        let selection = app_settings::parse_hotkey(&settings.selection_hotkey, "划词菜单")?;
        let screenshot = app_settings::parse_hotkey(&settings.screenshot_hotkey, "区域截图")?;
        let input_translate =
            app_settings::parse_hotkey(&settings.input_translate_hotkey, "输入翻译")?;

        Ok(Self {
            selection_hotkey: settings.selection_hotkey.clone(),
            screenshot_hotkey: settings.screenshot_hotkey.clone(),
            input_translate_hotkey: settings.input_translate_hotkey.clone(),
            selection_id: selection.id(),
            screenshot_id: screenshot.id(),
            input_translate_id: input_translate.id(),
        })
    }

    fn action_for(&self, shortcut: &Shortcut) -> Option<HotkeyAction> {
        let id = shortcut.id();
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
    Selection,
    Screenshot,
    InputTranslate,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SelectionActionResult {
    message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenshotStatusEvent {
    kind: &'static str,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticItem {
    label: &'static str,
    value: String,
    status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticSection {
    title: &'static str,
    detail: &'static str,
    items: Vec<DiagnosticItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MainDiagnostics {
    summary: String,
    sections: Vec<DiagnosticSection>,
}

#[derive(Deserialize)]
struct CapabilityConfig {
    windows: Vec<String>,
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
fn get_main_diagnostics() -> MainDiagnostics {
    let capability_windows = parse_capability_windows();
    let capability_window_count = capability_windows.len();
    let capability_window_list = capability_windows.join(", ");

    MainDiagnostics {
        summary: "主窗口命令链路可用。Alt+2 使用原生 Win32 划词弹窗；Alt+3 使用原生 Win32 框选层，不创建 Tauri/WebView 子窗口。"
            .to_string(),
        sections: vec![
            DiagnosticSection {
                title: "运行入口",
                detail: "确认当前可用入口和已暂停入口。",
                items: vec![
                    DiagnosticItem {
                        label: "主窗口",
                        value: "已渲染并可调用 Rust 命令".to_string(),
                        status: "ok",
                    },
                    DiagnosticItem {
                        label: "全局热键",
                        value: "已注册 Alt+2 和 Alt+3".to_string(),
                        status: "ok",
                    },
                    DiagnosticItem {
                        label: "截图热键",
                        value: "Alt+3 原生框选区域截图".to_string(),
                        status: "ok",
                    },
                    DiagnosticItem {
                        label: "托盘菜单",
                        value: "右键菜单只保留设置、自启动和退出；截图继续通过 Alt+3 触发"
                            .to_string(),
                        status: "ok",
                    },
                ],
            },
            DiagnosticSection {
                title: "资源配置",
                detail: "只读取仓库内构建配置，不访问用户内容。",
                items: vec![
                    DiagnosticItem {
                        label: "Capability 窗口",
                        value: format!("{capability_window_count} 个：{capability_window_list}"),
                        status: "info",
                    },
                    DiagnosticItem {
                        label: "预创建窗口",
                        value: "tauri.conf.json 仅预创建 main".to_string(),
                        status: "ok",
                    },
                    DiagnosticItem {
                        label: "前端资源",
                        value: "frontendDist 指向 ../dist".to_string(),
                        status: "ok",
                    },
                ],
            },
            DiagnosticSection {
                title: "子窗口配置对照",
                detail: "只对照静态标签、URL、授权和创建参数，不实际打开窗口。",
                items: child_window_config_items(&capability_windows),
            },
            DiagnosticSection {
                title: "窗口创建链路风险对照",
                detail: "只比较既有窗口创建特性，不调用任何创建函数。",
                items: window_creation_risk_items(),
            },
            DiagnosticSection {
                title: "前端窗口识别分支对照",
                detail: "只对照 App.tsx 内的 label 识别和 React 视图分支。",
                items: frontend_window_branch_items(),
            },
            DiagnosticSection {
                title: "视图初始化副作用对照",
                detail: "只列出 React 视图初始化时的命令、监听和副作用。",
                items: view_initialization_effect_items(),
            },
            DiagnosticSection {
                title: "视图错误态兜底对照",
                detail: "只列出 invoke 失败时各视图是否能渲染错误提示。",
                items: view_error_fallback_items(),
            },
            DiagnosticSection {
                title: "构建产物加载关系对照",
                detail: "只列出 frontendDist、index.html、资源文件和 WebviewUrl 的静态关系。",
                items: build_artifact_loading_items(),
            },
            DiagnosticSection {
                title: "settings 普通窗口安全验证方案",
                detail: "settings 普通窗口已人工验证失败，保留失败记录和回退边界。",
                items: settings_window_safety_plan_items(),
            },
            DiagnosticSection {
                title: "settings 普通窗口失败与撤回",
                detail: "settings 也出现白窗且无法关闭，入口已撤回。",
                items: settings_window_validation_entry_items(),
            },
            DiagnosticSection {
                title: "子窗口 WebView 创建失败原因对照",
                detail: "汇总已验证现象和下一步静态排查方向，不创建任何窗口。",
                items: child_webview_failure_cause_items(),
            },
            DiagnosticSection {
                title: "配置、权限与插件链路核对",
                detail: "只核对 tauri.conf、capability、插件和窗口创建残留，不打开窗口。",
                items: config_permission_plugin_audit_items(&capability_windows),
            },
            DiagnosticSection {
                title: "capability 授权精简执行状态",
                detail: "capability 已临时精简为仅 main；本诊断不创建窗口。",
                items: capability_trim_plan_items(&capability_windows),
            },
            DiagnosticSection {
                title: "Snow Shot 截图链路审计状态",
                detail: "只展示参考实现审计结果；当前恢复原生框选，不恢复 Tauri/WebView 遮罩或预览。",
                items: snow_shot_screenshot_audit_items(),
            },
            DiagnosticSection {
                title: "Snow Shot 窗口创建与关闭链路审计",
                detail: "只展示 Snow Shot draw 窗口静态审计结论，不调用任何窗口创建函数。",
                items: snow_shot_window_lifecycle_audit_items(),
            },
            DiagnosticSection {
                title: "Snow Shot 非窗口截图核心候选审计",
                detail: "只展示截图捕获、编码和剪贴板候选链路，不执行屏幕读取。",
                items: snow_shot_non_window_capture_audit_items(),
            },
            DiagnosticSection {
                title: "Snow Shot 截图依赖许可证审计",
                detail: "只展示锁定 commit 的顶层许可证核对结果，不引入依赖。",
                items: snow_shot_dependency_license_audit_items(),
            },
            DiagnosticSection {
                title: "只读截图命令草稿实施方案",
                detail: "保留历史草稿边界，并标明当前已升级为原生区域截图命令。",
                items: readonly_screenshot_command_plan_items(),
            },
            DiagnosticSection {
                title: "最小截图依赖引入与扫描方案",
                detail: "只展示后续依赖引入、许可证扫描和回退边界，不修改 Cargo.toml。",
                items: minimal_screenshot_dependency_plan_items(),
            },
            DiagnosticSection {
                title: "传递依赖扫描执行方案",
                detail: "只展示 license scan 执行顺序和阻塞标准，不安装工具、不新增依赖。",
                items: transitive_dependency_scan_plan_items(),
            },
            DiagnosticSection {
                title: "许可证扫描工具选择与离线替代方案",
                detail: "固定自动扫描优先级和离线兜底标准，不安装工具、不新增依赖。",
                items: license_scan_fallback_plan_items(),
            },
            DiagnosticSection {
                title: "最小截图依赖引入状态",
                detail: "展示 xcap 与 image 的依赖引入、审计结果和当前原生区域截图入口状态。",
                items: minimal_screenshot_dependency_introduction_items(),
            },
            DiagnosticSection {
                title: "原生区域截图剪贴板 MVP",
                detail: "Alt+3 使用原生 Win32 框选层；横条菜单支持复制、提取和翻译。",
                items: screenshot_module_scaffold_items(),
            },
            DiagnosticSection {
                title: "风险清单",
                detail: "白窗问题排查完成前，子窗口类入口保持关闭。",
                items: vec![
                    DiagnosticItem {
                        label: "安全预览",
                        value: "入口已撤回".to_string(),
                        status: "blocked",
                    },
                    DiagnosticItem {
                        label: "窗口诊断",
                        value: "入口已撤回".to_string(),
                        status: "blocked",
                    },
                    DiagnosticItem {
                        label: "Tauri 截图遮罩",
                        value: "WebView 入口已撤回；当前改用原生 Win32 框选".to_string(),
                        status: "blocked",
                    },
                    DiagnosticItem {
                        label: "settings",
                        value: "入口已撤回".to_string(),
                        status: "blocked",
                    },
                    DiagnosticItem {
                        label: "下一步",
                        value: "原生划词弹窗和原生结果弹窗已接入；下一步人工验收 Alt+2 文本操作"
                            .to_string(),
                        status: "info",
                    },
                ],
            },
        ],
    }
}

fn child_window_config_items(capability_windows: &[String]) -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "selection_bar",
            value: format!(
                "URL index.html；{}；Rust 创建入口已撤回，Alt+2 改走原生 Win32 划词弹窗",
                capability_status(capability_windows, "selection_bar")
            ),
            status: "blocked",
        },
        DiagnosticItem {
            label: "result",
            value: format!(
                "URL index.html；{}；Rust 创建入口已撤回，文本和截图结果改走原生 Win32 结果弹窗",
                capability_status(capability_windows, "result")
            ),
            status: "blocked",
        },
        DiagnosticItem {
            label: "settings",
            value: format!(
                "URL index.html；{}；创建参数：760x620、可调整、居中",
                capability_status(capability_windows, "settings")
            ),
            status: "info",
        },
        DiagnosticItem {
            label: "screenshot_overlay",
            value: format!(
                "{}；Rust 创建入口已撤回，保持禁用",
                capability_status(capability_windows, "screenshot_overlay")
            ),
            status: "blocked",
        },
        DiagnosticItem {
            label: "对照结论",
            value: "子窗口创建入口均已撤回；本诊断不会调用任何窗口创建函数".to_string(),
            status: "ok",
        },
    ]
}

fn window_creation_risk_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "共同链路",
            value: "settings 创建入口已移除；selection_bar 和 result 的 Rust WebView 创建入口已撤回；主窗口是 tauri.conf.json 预创建".to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "selection_bar",
            value: "入口已撤回：Alt+2 改为原生 Win32 划词弹窗；历史风险为 WebView 无边框、透明、置顶"
                .to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "result",
            value: "入口已撤回：文本和截图 AI 结果改为原生 Win32 结果弹窗；历史风险为 WebView 按需创建且置顶"
                .to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "settings",
            value: "已验证失败：普通标题栏、可调整尺寸、非置顶仍会白屏且关闭失效".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "排查顺序",
            value: "不再继续 WebView 子窗口实测；先验收原生弹窗的关闭、销毁和固定置顶".to_string(),
            status: "info",
        },
    ]
}

fn frontend_window_branch_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "识别方式",
            value: "前端通过 getCurrentWindow().label 识别当前窗口；读取失败时回退 main"
                .to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "已识别标签",
            value: "settings、selection_bar、result、screenshot_overlay；其他标签均按 main 处理"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "渲染顺序",
            value: "selection_bar -> result -> screenshot_overlay -> settings -> main".to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "截图分支",
            value: "ScreenshotOverlay 组件仍存在但入口关闭；Alt+3 调用原生 Win32 框选".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "静态结论",
            value: "未知标签会显示主窗口，不应直接导致空白；下一步可检查各视图初始化副作用"
                .to_string(),
            status: "ok",
        },
    ]
}

fn view_initialization_effect_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "selection_bar",
            value: "静态 React 分支仍存在；运行入口已撤回，Alt+2 改为 Rust 原生弹窗".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "result",
            value: "静态 React 分支仍存在；运行入口已撤回，AI 结果改为 Rust 原生结果弹窗"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "settings",
            value: "初始化调用 get_app_settings 和 get_api_key_status；不读取 API Key 明文"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "共同点",
            value: "初始化副作用均为 invoke、事件监听或键盘监听；不创建新的 Tauri 窗口".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "下一步",
            value: "settings 已验证失败；下一步继续验收原生弹窗方案，不恢复 WebView 子窗口"
                .to_string(),
            status: "info",
        },
    ]
}

fn view_error_fallback_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "selection_bar",
            value: "get_selection_snapshot 失败时显示“无法读取取词状态”和“无法连接 QuickPick 核心”"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "result",
            value: "AI 结果弹窗不依赖 get_result_snapshot 渲染；复制结果由原生弹窗直接写剪贴板"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "settings",
            value:
                "get_app_settings 失败时使用默认设置；get_api_key_status 失败时显示 Key 状态错误"
                    .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "静态结论",
            value: "三个视图都有错误态兜底；白窗更可能来自 WebView 加载或窗口创建链路".to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "下一步",
            value: "可继续只读排查动态 WebView 创建、窗口事件和 capability 关系，不再打开 settings"
                .to_string(),
            status: "info",
        },
    ]
}

fn build_artifact_loading_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "frontendDist",
            value: "tauri.conf.json 指向 ../dist；release 构建会把该目录作为前端资源根".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "index.html",
            value: "dist/index.html 是 React 入口，包含 #root，并引用 /assets 下的 JS 和 CSS"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "资源文件",
            value: "当前构建产物包含一个 index-*.js 和一个 index-*.css；均由 index.html 引用"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "子窗口 URL",
            value: "历史配置均指向 index.html；当前 Rust 已不保留 selection_bar、result 或 settings 创建入口"
                .to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "静态结论",
            value:
                "主窗口能加载说明基础资源可用；子窗口白屏更可能来自动态窗口 WebView 创建或窗口参数"
                    .to_string(),
            status: "info",
        },
    ]
}

fn settings_window_safety_plan_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "验证对象",
            value:
                "settings 普通窗口已验证失败；暂不验证 selection_bar、result 或 screenshot_overlay"
                    .to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "窗口参数",
            value: "即使保留标题栏、非透明、非置顶、非全屏和可调整尺寸，仍出现白窗".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "关闭路径",
            value: "标题栏关闭和任务栏关闭均失败；已改为撤回入口".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "触发方式",
            value: "主窗口验证按钮已撤回；托盘设置入口也禁用".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "回退规则",
            value: "已按回退规则处理：不再开放任何会创建子窗口的验证入口".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "人工验收",
            value: "人工验收失败：白窗、标题栏无法关闭、任务栏无法关闭".to_string(),
            status: "blocked",
        },
    ]
}

fn settings_window_validation_entry_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "入口位置",
            value: "主窗口“打开 settings 验证”按钮已撤回".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "调用命令",
            value: "open_settings_validation_window 命令已撤回".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "托盘设置",
            value: "托盘“设置”改为安全暂停，避免再次打开白窗".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "刷新诊断",
            value: "刷新仍只读取状态，不会自动打开 settings 或任何子窗口".to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "下一步",
            value: "只在主窗口内排查 Tauri 子窗口创建或 WebView 初始化问题".to_string(),
            status: "blocked",
        },
    ]
}

fn child_webview_failure_cause_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "已确认现象",
            value: "安全预览、静态窗口诊断、截图遮罩和 settings 均出现白窗或关闭失效"
                .to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "已排除方向",
            value: "settings 非透明、非置顶、非全屏仍失败，说明问题不是遮罩样式参数单点导致"
                .to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "资源加载",
            value: "主窗口和 dist/index.html 可加载，基础前端产物缺失的可能性降低".to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "关闭事件",
            value: "Rust 侧没有拦截 settings 关闭事件；标题栏仍失效，需排查子 WebView/事件循环层"
                .to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "高概率链路",
            value: "动态 WebviewWindowBuilder 创建、子窗口 WebView 初始化、capability 或 window-state 交互"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "下一步",
            value: "继续只在主窗口内核对配置、权限、插件和创建链路；不打开任何子窗口".to_string(),
            status: "info",
        },
    ]
}

fn config_permission_plugin_audit_items(capability_windows: &[String]) -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "tauri.conf",
            value: "仅预创建 main；frontendDist 指向 ../dist；没有预创建 settings、result 或 selection_bar"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "capability",
            value: format!(
                "授权窗口标签为 {}；授权不等于创建窗口，但仍需核对多余授权",
                capability_windows.join(", ")
            ),
            status: "paused",
        },
        DiagnosticItem {
            label: "插件链路",
            value: "已启用 opener、single-instance、window-state；window-state 会参与窗口状态管理"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "创建残留",
            value: "selection_bar、result 和 settings 的 WebView 创建入口均已撤回；新增 native_popup 原生弹窗入口"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "入口状态",
            value: "托盘设置安全暂停；Alt+2 用原生划词弹窗，Alt+3 用原生框选，不创建 WebView 子窗口".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "静态结论",
            value: "配置不会自动创建子窗口；下一步可只读核对多余 capability 与 window-state 交互"
                .to_string(),
            status: "info",
        },
    ]
}

fn capability_trim_plan_items(capability_windows: &[String]) -> Vec<DiagnosticItem> {
    let has_only_main = capability_windows.len() == 1 && capability_windows[0] == "main";
    let extra_windows: Vec<&str> = ["settings", "selection_bar", "result", "screenshot_overlay"]
        .into_iter()
        .filter(|label| capability_windows.iter().any(|window| window == label))
        .collect();

    vec![
        DiagnosticItem {
            label: "当前授权",
            value: format!(
                "当前 capability 授权窗口：{}",
                capability_windows.join(", ")
            ),
            status: if has_only_main { "ok" } else { "paused" },
        },
        DiagnosticItem {
            label: "拟保留",
            value: "已保留 main，保证主窗口诊断、核心检查和安全回滚状态可用".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "已暂停",
            value: if extra_windows.is_empty() {
                "settings、selection_bar、result、screenshot_overlay 已不在 capability windows 中"
                    .to_string()
            } else {
                format!(
                    "仍检测到待暂停授权 {}；它们当前没有安全打开入口",
                    extra_windows.join(", ")
                )
            },
            status: if extra_windows.is_empty() {
                "ok"
            } else {
                "paused"
            },
        },
        DiagnosticItem {
            label: "配置状态",
            value: "src-tauri/capabilities/default.json 已临时精简为 windows: [main]".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "风险收益",
            value: "收益是减少子窗口权限面；风险是后续恢复划词或结果窗口时需重新补回授权"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "回退方式",
            value: "若精简后影响主窗口或 Alt+2，可立即恢复原 windows 列表并重新构建".to_string(),
            status: "info",
        },
    ]
}

fn snow_shot_screenshot_audit_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "参考来源",
            value: "Snow Shot main @ c7f2d9fe3114ad0dba6e5efdfe4bd8ecbc1f1de3；本项仅为静态审计"
                .to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "候选链路",
            value: "Rust screenshot.rs、tauri-commands/screenshot、前端 functions/pages draw 可作为最小截图链路候选"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "依赖审计",
            value: "xcap 为 Apache-2.0，scap/device_query 为 MIT；传递依赖未审计前不直接引入"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "MVP 排除",
            value: "滚动截图、标注、贴图、历史记录、录屏、插件和 Excalidraw 暂不进入首版"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "安全闸门",
            value: "Alt+3 已使用原生框选；Tauri 全屏遮罩和截图预览子窗口继续关闭"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "下一步",
            value: "先稳定原生框选复制，再接入 OCR 提取和翻译按钮".to_string(),
            status: "info",
        },
    ]
}

fn snow_shot_window_lifecycle_audit_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "创建方式",
            value: "运行时创建 draw-* WebView，URL 为 /draw；不是 tauri.conf 预创建窗口"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "窗口参数",
            value: "无边框、透明、跳过任务栏、1x1、不可调整、默认不可见且不聚焦".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "Windows 样式",
            value: "set_draw_window_style 在 Windows 当前为占位；主要样式仍来自创建参数"
                .to_string(),
            status: "info",
        },
        DiagnosticItem {
            label: "初始动作",
            value: "创建后禁用 DWM 转场并 hide；内容显示依赖 /draw 前端初始化和事件流".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "关闭链路",
            value: "release-draw-page 事件最终调用当前窗口 close；WebView 未加载时该兜底不可用"
                .to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "QuickPick 结论",
            value: "暂不复用 draw 窗口循环；先做非窗口截图核心或普通可关闭窗口验证".to_string(),
            status: "ok",
        },
    ]
}

fn snow_shot_non_window_capture_audit_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "当前显示器",
            value: "capture_current_monitor 可返回 PNG/WebP 字节；依赖鼠标所在显示器和 xcap 捕获"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "全显示器",
            value: "capture_all_monitors 可返回 PNG；Windows 分支含 Rgba8 和 SharedBuffer 优化，可先禁用优化"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "全屏流程",
            value: "capture_full_screen 会保存图片和截图历史；QuickPick 首版不直接复用这段落盘逻辑"
                .to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "图片编码",
            value: "encode_image 支持 PNG/WebP/JPEG/AVIF；MVP 先保留 PNG，减少兼容风险".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "剪贴板",
            value: "Snow Shot 使用 CF_DIB 写图；QuickPick 优先用 Tauri 图片剪贴板或等价重写"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "迁移边界",
            value: "先做只读截图命令草稿，不创建窗口、不保存历史、不自动上传".to_string(),
            status: "ok",
        },
    ]
}

fn snow_shot_dependency_license_audit_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "xcap",
            value: "锁定 d353731a；Cargo.toml 与 LICENSE 均标识 Apache-2.0；未发现 NOTICE 文件"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "scap",
            value: "锁定 88914f1；Cargo.toml 与 LICENSE 均标识 MIT；未发现 NOTICE 文件".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "device_query",
            value: "锁定 7b9a56c；Cargo.toml 与 LICENSE 均标识 MIT；未发现 NOTICE 文件".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "自定义分支",
            value: "仍需按 commit 固定来源和修改边界，不能假定与 crates.io 发布包完全一致"
                .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "传递依赖",
            value: "尚未跑 cargo-deny 或完整 license scan；真正引入前必须补做".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "QuickPick 结论",
            value: "顶层许可证未阻塞静态参考；当前使用 crates.io xcap 等价实现区域捕获".to_string(),
            status: "ok",
        },
    ]
}

fn readonly_screenshot_command_plan_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "命令草稿",
            value: "已新增 capture_region_to_clipboard，并注册为原生区域截图 MVP".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "依赖策略",
            value: "优先最小化评估 xcap + image；新增前先做传递依赖 license scan".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "权限边界",
            value: "已接 Alt+3 快捷键；仅创建原生 Win32 临时框选层".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "返回数据",
            value: "计划返回 PNG 字节、宽高和错误类型；日志只记尺寸与状态，不记图片内容"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "隐私边界",
            value: "全程内存处理；只在用户触发截图时写剪贴板，不保存文件、不上传 AI、不保存历史"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "验证顺序",
            value: "先完成编译与构建验证；区域框选后续仍需单独验证关闭路径".to_string(),
            status: "paused",
        },
    ]
}

fn minimal_screenshot_dependency_plan_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "候选组合",
            value: "第一候选仅评估 xcap + image；暂不引入 scap、device_query 或 SharedBuffer"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "引入方式",
            value: "先建独立 screenshot_core 模块并保持未暴露；不接 invoke_handler".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "扫描要求",
            value: "新增依赖后必须跑 cargo tree 和 license scan，记录传递依赖许可证".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "版本策略",
            value: "优先固定明确版本；若使用 git commit，必须记录来源、commit、许可证和回退方案"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "回退线",
            value: "若依赖体积、许可证或编译失败不可接受，立即撤回依赖并保留静态方案".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "安全线",
            value: "最小依赖扫描已完成；当前只允许用户触发的原生区域截图".to_string(),
            status: "ok",
        },
    ]
}

fn transitive_dependency_scan_plan_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "工具现状",
            value: "cargo tree 可用；cargo-deny 和 cargo-about 当前未安装".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "依赖树",
            value: "新增依赖后先跑 cargo tree --locked --prefix depth，记录新增包和版本"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "许可证扫描",
            value: "优先使用 cargo-deny 或 cargo-about；工具缺失时暂停，不暴露截图命令".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "记录格式",
            value: "记录包名、版本、来源、许可证、是否新增、处理结论和阻塞原因".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "阻塞标准",
            value: "GPL、AGPL、未知、自定义、无 LICENSE 或无法确认来源时暂停".to_string(),
            status: "blocked",
        },
        DiagnosticItem {
            label: "安全线",
            value: "扫描通过前不注册 invoke、不读取屏幕、不保存截图、不打开窗口".to_string(),
            status: "blocked",
        },
    ]
}

fn license_scan_fallback_plan_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "首选工具",
            value: "优先 cargo-deny check licenses；可复现、可阻塞 CI，适合后续长期维护"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "报告工具",
            value: "cargo-about 可作为许可证报告补充；缺失时不影响先执行阻塞判断".to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "离线兜底",
            value: "工具未安装时，用带 Windows 目标的 cargo metadata、cargo tree 和本地 LICENSE 文件人工核对"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "执行边界",
            value: "metadata 固定 --filter-platform x86_64-pc-windows-msvc，避免离线解析其他平台缺包"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "通过条件",
            value: "每个新增传递包都有明确许可证、来源和处理结论，且不触发阻塞清单".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "阻塞条件",
            value: "缺少本地源码、许可证字段不一致、LICENSE 文件缺失或无法确认时暂停新增依赖"
                .to_string(),
            status: "blocked",
        },
    ]
}

fn minimal_screenshot_dependency_introduction_items() -> Vec<DiagnosticItem> {
    vec![
        DiagnosticItem {
            label: "依赖清单",
            value: "Cargo.toml 已显式加入 xcap 0.9.7 和 image 0.25 PNG 最小组合".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "xcap 许可证",
            value: "crates.io 与本地包均声明 Apache-2.0，并包含 LICENSE 文件".to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "Windows 依赖树",
            value: "xcap Windows 目标传递树为 39 个包，metadata 未发现 GPL、AGPL 或未知许可证"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "锁文件变化",
            value:
                "Cargo.lock 已锁定 xcap 及跨平台解析项；Windows 审计以 x86_64-pc-windows-msvc 为准"
                    .to_string(),
            status: "paused",
        },
        DiagnosticItem {
            label: "当前边界",
            value: "已注册原生区域截图命令；复制写剪贴板，提取/翻译走用户触发的视觉模型"
                .to_string(),
            status: "ok",
        },
        DiagnosticItem {
            label: "入口状态",
            value: "Alt+3 区域截图已启用；托盘截图入口已撤回；Tauri/WebView 截图遮罩和预览继续关闭"
                .to_string(),
            status: "paused",
        },
    ]
}

fn screenshot_module_scaffold_items() -> Vec<DiagnosticItem> {
    let status = screenshot::scaffold_status();
    let disabled_result = screenshot::disabled_capture_result(screenshot::CaptureDraftRequest {
        target: screenshot::CaptureTarget::CurrentMonitor,
        include_cursor: false,
    });
    let disabled_status = match disabled_result.status {
        screenshot::CaptureDraftStatus::Disabled => "Disabled",
    };
    let region_request = screenshot::CaptureDraftRequest {
        target: screenshot::CaptureTarget::Region(screenshot::CaptureRegion {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        }),
        include_cursor: false,
    };
    let region_boundary = match region_request.target {
        screenshot::CaptureTarget::Region(region) => {
            format!(
                "Region 类型已定义：{}x{}，原生框选会生成真实区域",
                region.width, region.height
            )
        }
        screenshot::CaptureTarget::CurrentMonitor => "CurrentMonitor 类型已定义".to_string(),
    };

    vec![
        DiagnosticItem {
            label: "模块路径",
            value: format!(
                "src-tauri/src/{}/mod.rs 已编译进 Rust crate",
                status.module_label
            ),
            status: "ok",
        },
        DiagnosticItem {
            label: "命令命名",
            value: format!(
                "历史草稿 {} 已保留；整屏命令 {} 和区域命令 {} 已加入 invoke_handler",
                status.draft_command_name,
                status.clipboard_command_name,
                status.region_command_name
            ),
            status: "ok",
        },
        DiagnosticItem {
            label: "类型边界",
            value: region_boundary,
            status: "ok",
        },
        DiagnosticItem {
            label: "禁用结果",
            value: format!(
                "历史草稿仍为 {}；{}；返回 PNG 字节数 {}",
                disabled_status,
                disabled_result.message,
                disabled_result.png_bytes.len()
            ),
            status: "paused",
        },
        DiagnosticItem {
            label: "入口状态",
            value: format!(
                "invoke={}，读屏={}，剪贴板={}，原生框选={}",
                status.invoke_registered,
                status.screen_reading_enabled,
                status.clipboard_writing_enabled,
                status.window_entry_enabled
            ),
            status: "ok",
        },
        DiagnosticItem {
            label: "下一步",
            value: status.next_step.to_string(),
            status: "paused",
        },
    ]
}

fn capability_status(capability_windows: &[String], label: &str) -> &'static str {
    if capability_windows.iter().any(|window| window == label) {
        "capability 已授权"
    } else {
        "capability 未授权"
    }
}

fn parse_capability_windows() -> Vec<String> {
    const CAPABILITY_JSON: &str = include_str!("../capabilities/default.json");

    serde_json::from_str::<CapabilityConfig>(CAPABILITY_JSON)
        .map(|config| config.windows)
        .unwrap_or_else(|_| vec!["读取 capability 失败".to_string()])
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
) -> Result<SelectionActionResult, String> {
    let previous_settings = app_settings::load_app_settings(&app).unwrap_or_default();
    let settings = app_settings::normalize_settings_for_save(settings)?;
    replace_global_shortcuts(&app, &settings)?;
    if let Err(error) = app_settings::save_app_settings(&app, &settings) {
        let _ = replace_global_shortcuts(&app, &previous_settings);
        return Err(error);
    }
    apply_autostart_setting(settings.autostart_enabled)?;
    sync_window_appearance(&app, &settings);

    Ok(SelectionActionResult {
        message: "设置已保存".to_string(),
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
        .map_err(|_| "写入剪贴板失败，请稍后再试".to_string())?;

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
        return Err("搜索内容过长，请缩短选择范围后再试".to_string());
    }

    let url = format!(
        "https://www.bing.com/search?q={}",
        encode_query_component(&text)
    );

    tauri_plugin_opener::open_url(url, None::<&str>)
        .map_err(|_| "打开默认浏览器失败，请检查系统默认浏览器设置".to_string())?;

    Ok(SelectionActionResult {
        message: "已打开搜索".to_string(),
    })
}

#[tauri::command]
fn copy_result_content(state: tauri::State<'_, AppState>) -> Result<SelectionActionResult, String> {
    let snapshot = state
        .result_snapshot
        .lock()
        .map_err(|_| "读取结果状态失败，请重试".to_string())?;

    if snapshot.status != "success" || !snapshot.can_copy {
        return Err("当前没有可复制的结果".to_string());
    }

    let content = snapshot.content.trim();
    if content.is_empty() {
        return Err("结果为空，无法复制".to_string());
    }

    clipboard_win::set_clipboard_string(content)
        .map_err(|_| "写入剪贴板失败，请稍后再试".to_string())?;

    Ok(SelectionActionResult {
        message: "结果已复制".to_string(),
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
    let result_popup = open_native_result_popup(&app, &loading_snapshot);
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
        Ok(None) => Err("文本模型 API Key 未配置，请先到设置中保存".to_string()),
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
    update_native_result_popup(&app, &result_popup, &final_snapshot);
    set_result_snapshot(&state, final_snapshot)?;

    if action == "translate" {
        process_text_language_switches(
            &app,
            &result_popup,
            &settings,
            &text,
            source_preview,
            source_char_count,
        )
        .await?;
    }

    Ok(SelectionActionResult {
        message: "AI 请求已完成".to_string(),
    })
}

async fn process_text_language_switches(
    app: &tauri::AppHandle,
    result_popup: &Option<native_popup::ResultPopupHandle>,
    settings: &app_settings::AppSettings,
    text: &str,
    source_preview: String,
    source_char_count: usize,
) -> Result<(), String> {
    let Some(handle) = result_popup else {
        return Ok(());
    };

    while let Some(event) = handle.recv_event() {
        let native_popup::ResultPopupEvent::TranslationChanged {
            source_language,
            target_language,
            direction,
        } = event;
        let (request_source_language, request_target_language) =
            translation_request_languages(&source_language, &target_language, &direction);
        let loading = ResultSnapshot::text_loading(
            "translate",
            source_preview.clone(),
            source_char_count,
            source_language.clone(),
            target_language.clone(),
            direction.clone(),
        )?;
        update_native_result_popup(app, result_popup, &loading);
        set_app_result_snapshot(app, loading)?;

        let ai_result = match security::load_api_key(app, "text") {
            Ok(Some(mut api_key)) => {
                let result = ai::run_text_action(
                    settings,
                    &api_key,
                    "translate",
                    text,
                    &request_source_language,
                    &request_target_language,
                )
                .await;
                api_key.clear();
                result
            }
            Ok(None) => Err("文本模型 API Key 未配置，请先到设置中保存".to_string()),
            Err(error) => Err(error),
        };

        let snapshot = match ai_result {
            Ok(content) => ResultSnapshot::text_success(
                "translate",
                source_preview.clone(),
                source_char_count,
                content,
                source_language.clone(),
                target_language,
                direction.clone(),
            )?,
            Err(error) => ResultSnapshot::text_error(
                "translate",
                source_preview.clone(),
                source_char_count,
                error,
                source_language.clone(),
                target_language,
                direction.clone(),
            )?,
        };
        update_native_result_popup(app, result_popup, &snapshot);
        set_app_result_snapshot(app, snapshot)?;
    }

    Ok(())
}

#[tauri::command]
fn clear_result_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<SelectionActionResult, String> {
    let mut current = state
        .result_snapshot
        .lock()
        .map_err(|_| "清理结果状态失败".to_string())?;
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
async fn capture_region_to_clipboard(
    app: tauri::AppHandle,
) -> Result<SelectionActionResult, String> {
    let selection = screenshot::select_region_action(native_theme_for_app(&app))?
        .ok_or_else(|| "已取消截图".to_string())?;
    handle_region_menu_selection(app, selection).await
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
        .map_err(|_| "切换快捷键录制状态失败".to_string())?;

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
        .invoke_handler(tauri::generate_handler![
            ping,
            get_main_diagnostics,
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
            run_text_ai_action,
            clear_result_snapshot,
            capture_current_monitor_to_clipboard,
            capture_region_to_clipboard,
            set_hotkey_capture_mode
        ])
        .setup(|app| {
            let settings = app_settings::load_app_settings(app.handle()).unwrap_or_default();
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
            WindowEvent::Focused(false) if window.label() == "selection_bar" => {
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
        .map_err(|_| "读取选区状态失败，请重新选择后再试".to_string())?;
    let text = snapshot.text.trim();

    if text.is_empty() {
        Err("未读取到可操作的选中文本，请重新选择后再试".to_string())
    } else {
        Ok(text.to_string())
    }
}

fn current_selection_payload(state: &AppState) -> Result<(String, String, usize), String> {
    let snapshot = state
        .selection_snapshot
        .lock()
        .map_err(|_| "读取选区状态失败，请重新选择后再试".to_string())?;
    let text = snapshot.text.trim();

    if text.is_empty() {
        Err("未读取到可操作的选中文本，请重新选择后再试".to_string())
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
        .map_err(|_| "写入结果状态失败，请重试".to_string())?;
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
                    Some(HotkeyAction::Selection) => {
                        activate_selection_bar(app, shortcut_release_keys(shortcut))
                    }
                    Some(HotkeyAction::Screenshot) => capture_region_from_entry(app),
                    Some(HotkeyAction::InputTranslate) => activate_input_translate(app),
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
) -> Result<(), String> {
    let next = HotkeyBindings::from_settings(settings)?;
    let previous = app
        .state::<AppState>()
        .hotkey_bindings
        .lock()
        .map(|bindings| bindings.clone())
        .map_err(|_| "读取当前快捷键状态失败".to_string())?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|_| "注销旧快捷键失败".to_string())?;

    if let Err(error) = register_hotkey_bindings(app, &next) {
        let _ = register_hotkey_bindings(app, &previous);
        if let Ok(mut bindings) = app.state::<AppState>().hotkey_bindings.lock() {
            *bindings = previous;
        }
        return Err(error);
    }

    app.state::<AppState>()
        .hotkey_bindings
        .lock()
        .map(|mut bindings| {
            *bindings = next;
        })
        .map_err(|_| "更新快捷键状态失败".to_string())
}

fn register_hotkey_bindings(
    app: &tauri::AppHandle,
    bindings: &HotkeyBindings,
) -> Result<(), String> {
    app.global_shortcut()
        .register_multiple([
            bindings.selection_hotkey.as_str(),
            bindings.screenshot_hotkey.as_str(),
            bindings.input_translate_hotkey.as_str(),
        ])
        .map_err(|error| format!("快捷键注册失败，可能已被其他软件占用：{error}"))
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

    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    let _ = window.set_decorations(false);
    window
        .set_theme(theme)
        .map_err(|error| format!("应用窗口主题失败：{error}"))?;

    let _ = window.set_shadow(true);
    let mica_effect = mica_effect_for_theme(settings);
    let effects = Some(EffectsBuilder::new().effect(mica_effect).build());

    window
        .set_effects(effects)
        .map_err(|error| format!("应用窗口效果失败：{error}"))
}

fn mica_effect_for_theme(settings: &app_settings::AppSettings) -> Effect {
    match settings.theme_mode.as_str() {
        "light" => Effect::MicaLight,
        "dark" | "workbench" => Effect::MicaDark,
        _ => Effect::Mica,
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

    TrayIconBuilder::new()
        .tooltip("QuickPick")
        .icon(icon)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Right,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_tray_menu(tray.app_handle().clone());
            }
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

    Ok(())
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
        Ok(()) => {}
        Err(error) => {
            eprintln!("QuickPick autostart toggle failed: {error}");
        }
    }
}

fn show_tray_menu(app: tauri::AppHandle) {
    let _ = std::thread::spawn(move || {
        let settings = app_settings::load_app_settings(&app).unwrap_or_default();
        let theme = native_theme::NativeTheme::from_theme_mode(&settings.theme_mode);
        let action = match native_popup::select_tray_menu_action(theme, settings.autostart_enabled)
        {
            Ok(action) => action,
            Err(error) => {
                eprintln!("QuickPick tray menu skipped: {error}");
                None
            }
        };

        match action {
            Some(native_popup::TrayMenuAction::Settings) => show_settings_window(&app),
            Some(native_popup::TrayMenuAction::ToggleAutostart) => toggle_autostart_from_tray(&app),
            Some(native_popup::TrayMenuAction::Quit) => app.exit(0),
            None => {}
        }
    });
}

fn capture_region_from_entry(app: &tauri::AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let (kind, message) = match screenshot::select_region_action(native_theme_for_app(&app)) {
            Ok(Some(selection)) => {
                match tauri::async_runtime::block_on(handle_region_menu_selection(
                    app.clone(),
                    selection,
                )) {
                    Ok(result) => ("success", result.message),
                    Err(error) => ("error", error),
                }
            }
            Ok(None) => ("error", "已取消截图".to_string()),
            Err(error) => ("error", error),
        };
        emit_screenshot_status(&app, kind, message);
    });
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
    let result_popup = open_native_result_popup(&app, &loading);
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
            update_native_result_popup(&app, &result_popup, &snapshot);
            set_app_result_snapshot(&app, snapshot)?;
            emit_screenshot_status(&app, "error", error.clone());
            return Err(error);
        }
    };

    let source_preview = format!(
        "区域截图 {}x{}（{}）",
        image.width, image.height, image.monitor_name
    );
    let loading = ResultSnapshot::image_loading(
        action,
        source_preview.clone(),
        source_language.clone(),
        target_language.clone(),
        translation_direction.clone(),
    )?;
    update_native_result_popup(&app, &result_popup, &loading);
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
    update_native_result_popup(&app, &result_popup, &final_snapshot);
    set_app_result_snapshot(&app, final_snapshot)?;
    emit_screenshot_status(&app, kind, message.clone());

    if action == "translate" {
        process_image_language_switches(
            &app,
            &result_popup,
            &settings,
            &image.png_bytes,
            source_preview,
        )
        .await?;
    }

    Ok(SelectionActionResult { message })
}

async fn process_image_language_switches(
    app: &tauri::AppHandle,
    result_popup: &Option<native_popup::ResultPopupHandle>,
    settings: &app_settings::AppSettings,
    png_bytes: &[u8],
    source_preview: String,
) -> Result<(), String> {
    let Some(handle) = result_popup else {
        return Ok(());
    };

    while let Some(event) = handle.recv_event() {
        let native_popup::ResultPopupEvent::TranslationChanged {
            source_language,
            target_language,
            direction,
        } = event;
        let (request_source_language, request_target_language) =
            translation_request_languages(&source_language, &target_language, &direction);
        let loading = ResultSnapshot::image_loading(
            "translate",
            source_preview.clone(),
            source_language.clone(),
            target_language.clone(),
            direction.clone(),
        )?;
        update_native_result_popup(app, result_popup, &loading);
        set_app_result_snapshot(app, loading)?;
        emit_screenshot_status(app, "success", "正在重新翻译截图文字".to_string());

        let ai_result = match security::load_api_key(app, "vision") {
            Ok(Some(mut api_key)) => {
                let result = ai::run_image_action(
                    settings,
                    &api_key,
                    "translate",
                    png_bytes,
                    &request_source_language,
                    &request_target_language,
                )
                .await;
                api_key.clear();
                result
            }
            Ok(None) => Err("视觉模型 API Key 未配置，请先保存".to_string()),
            Err(error) => Err(error),
        };

        let (kind, snapshot) = match ai_result {
            Ok(content) => (
                "success",
                ResultSnapshot::image_success(
                    "translate",
                    source_preview.clone(),
                    content,
                    source_language.clone(),
                    target_language,
                    direction.clone(),
                )?,
            ),
            Err(error) => (
                "error",
                ResultSnapshot::image_error(
                    "translate",
                    source_preview.clone(),
                    error,
                    source_language.clone(),
                    target_language,
                    direction.clone(),
                )?,
            ),
        };
        update_native_result_popup(app, result_popup, &snapshot);
        set_app_result_snapshot(app, snapshot)?;
        emit_screenshot_status(app, kind, "图片翻译已更新".to_string());
    }

    Ok(())
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
        let snapshot = current_selection_snapshot(&app);

        match native_popup::select_text_action(snapshot, native_theme_for_app(&app)) {
            Ok(Some(native_popup::SelectionPopupAction::Copy)) => {
                let state = app.state::<AppState>();
                if let Err(error) = copy_selection_text_from_state(&state) {
                    show_native_message_popup(
                        &app,
                        "划词复制失败".to_string(),
                        error,
                        "未覆盖剪贴板。".to_string(),
                    );
                }
            }
            Ok(Some(native_popup::SelectionPopupAction::Search)) => {
                let state = app.state::<AppState>();
                if let Err(error) = search_selection_text_from_state(&state) {
                    show_native_message_popup(
                        &app,
                        "划词搜索失败".to_string(),
                        error,
                        "未打开浏览器。".to_string(),
                    );
                }
            }
            Ok(Some(native_popup::SelectionPopupAction::Translate)) => {
                let _ = tauri::async_runtime::block_on(run_text_ai_action_inner(
                    app.clone(),
                    "translate".to_string(),
                ));
            }
            Ok(Some(native_popup::SelectionPopupAction::Summarize)) => {
                let _ = tauri::async_runtime::block_on(run_text_ai_action_inner(
                    app.clone(),
                    "summarize".to_string(),
                ));
            }
            Ok(None) => {}
            Err(error) => {
                show_native_message_popup(
                    &app,
                    "划词弹窗失败".to_string(),
                    error,
                    "本次没有读取或上传选中文本。".to_string(),
                );
            }
        }
    });
}

fn activate_input_translate(app: &tauri::AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let _ = tauri::async_runtime::block_on(run_input_translate_flow(app.clone()));
    });
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
    let result_popup = match native_popup::open_input_popup(
        String::new(),
        source_language.clone(),
        target_language.clone(),
        translation_direction.clone(),
        String::new(),
        String::new(),
        "waiting".to_string(),
        native_theme_for_app(&app),
    ) {
        Ok(popup) => popup,
        Err(error) => {
            show_native_message_popup(
                &app,
                "输入翻译弹窗创建失败".to_string(),
                error.clone(),
                "请稍后重试。".to_string(),
            );
            return Err(error);
        }
    };

    process_input_translation_events(
        &app,
        &result_popup,
        &settings,
        source_language,
        target_language,
        translation_direction,
    )
    .await?;

    Ok(SelectionActionResult {
        message: "输入翻译已关闭".to_string(),
    })
}

async fn process_input_translation_events(
    app: &tauri::AppHandle,
    result_popup: &native_popup::InputPopupHandle,
    settings: &app_settings::AppSettings,
    source_language: String,
    target_language: String,
    translation_direction: String,
) -> Result<(), String> {
    loop {
        let (input_text, source_language, target_language, translation_direction);
        if let Some(update) = result_popup.recv_event() {
            let native_popup::InputPopupEvent::TranslateRequested {
                input_text: event_input,
                source_language: event_source,
                target_language: event_target,
                direction: event_direction,
            } = update;
            input_text = event_input;
            source_language = event_source;
            target_language = event_target;
            translation_direction = event_direction;
        } else {
            return Ok(());
        }

        let trimmed = input_text.trim().to_string();
        if trimmed.is_empty() {
            result_popup.update(String::new(), "请输入要翻译的文本。".to_string(), "waiting".to_string(), source_language, target_language, translation_direction);
            continue;
        }

        let (request_source_language, request_target_language) = translation_request_languages(
            &source_language,
            &target_language,
            &translation_direction,
        );
        result_popup.update(
            String::new(),
            String::new(),
            "loading".to_string(),
            source_language.clone(),
            target_language.clone(),
            translation_direction.clone(),
        );

        let ai_result = match security::load_api_key(app, "input") {
            Ok(Some(mut api_key)) => {
                let result = ai::run_input_text_action(
                    settings,
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
            Ok(None) => Err("输入模型 API Key 未配置，请先到设置中保存".to_string()),
            Err(error) => Err(error),
        };

        match ai_result {
            Ok(content) => {
                result_popup.update(
                    content,
                    String::new(),
                    "success".to_string(),
                    source_language,
                    target_language,
                    translation_direction,
                );
            }
            Err(error) => {
                result_popup.update(
                    String::new(),
                    error,
                    "error".to_string(),
                    source_language,
                    target_language,
                    translation_direction,
                );
            }
        }
    }
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

fn current_selection_snapshot(app: &tauri::AppHandle) -> SelectionSnapshot {
    let state = app.state::<AppState>();

    state
        .selection_snapshot
        .lock()
        .map(|snapshot| snapshot.clone())
        .unwrap_or_else(|_| SelectionSnapshot::default())
}

fn native_theme_for_app(app: &tauri::AppHandle) -> native_theme::NativeTheme {
    app_settings::load_app_settings(app)
        .map(|settings| native_theme::NativeTheme::from_theme_mode(&settings.theme_mode))
        .unwrap_or_default()
}

fn show_native_message_popup(
    app: &tauri::AppHandle,
    title: String,
    content: String,
    detail: String,
) {
    native_popup::show_result_popup(
        title,
        content,
        detail,
        default_translation_source_language(),
        "zh-Hans".to_string(),
        default_translation_direction(),
        false,
        native_theme_for_app(app),
    );
}

fn show_native_result_popup(app: &tauri::AppHandle, snapshot: &ResultSnapshot) {
    native_popup::show_result_popup(
        snapshot.title.clone(),
        snapshot.content.clone(),
        snapshot.detail.clone(),
        snapshot.source_language.clone(),
        snapshot.target_language.clone(),
        snapshot.translation_direction.clone(),
        snapshot.can_switch_language,
        native_theme_for_app(app),
    );
}

fn open_native_result_popup(
    app: &tauri::AppHandle,
    snapshot: &ResultSnapshot,
) -> Option<native_popup::ResultPopupHandle> {
    match native_popup::open_result_popup(
        snapshot.title.clone(),
        snapshot.content.clone(),
        snapshot.detail.clone(),
        snapshot.status.clone(),
        snapshot.source_language.clone(),
        snapshot.target_language.clone(),
        snapshot.translation_direction.clone(),
        snapshot.can_switch_language,
        native_theme_for_app(app),
    ) {
        Ok(handle) => Some(handle),
        Err(_) => {
            show_native_result_popup(app, snapshot);
            None
        }
    }
}

fn update_native_result_popup(
    app: &tauri::AppHandle,
    handle: &Option<native_popup::ResultPopupHandle>,
    snapshot: &ResultSnapshot,
) {
    if let Some(handle) = handle {
        if handle.update(
            snapshot.title.clone(),
            snapshot.content.clone(),
            snapshot.detail.clone(),
            snapshot.status.clone(),
            snapshot.source_language.clone(),
            snapshot.target_language.clone(),
            snapshot.translation_direction.clone(),
            snapshot.can_switch_language,
        ) {
            return;
        }
    }

    show_native_result_popup(app, snapshot);
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

    let current_exe = std::env::current_exe().map_err(|_| "无法定位当前程序路径".to_string())?;
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
        return Err(format!("打开启动项注册表失败：{create_status}"));
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
        return Err(format!("写入启动项注册表失败：{set_status}"));
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
        return Err(format!("打开启动项注册表失败：{create_status}"));
    }

    let delete_status = unsafe { RegDeleteValueW(key, value_name.as_ptr()) };
    unsafe {
        RegCloseKey(key);
    }

    if delete_status != ERROR_SUCCESS && delete_status != ERROR_FILE_NOT_FOUND {
        return Err(format!("删除启动项注册表失败：{delete_status}"));
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
        let _ = window.set_title("QuickPick 设置");
        let _ = window.set_decorations(false);
        let _ = window.unminimize();
        let _ = window.show();
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
    fn selection_hotkey_with_ctrl_waits_for_all_modifiers_and_main_key() {
        let shortcut = app_settings::parse_hotkey("Ctrl+Alt+Q", "划词菜单").unwrap();
        let keys = shortcut_release_keys(&shortcut);

        assert!(keys.contains(&0x11));
        assert!(keys.contains(&0x12));
        assert!(keys.contains(&0x51));
    }
}
