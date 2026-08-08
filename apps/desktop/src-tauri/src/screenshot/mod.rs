use crate::native_theme::NativeTheme;
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::ptr::{null, null_mut};
use xcap::Monitor;

pub const MODULE_LABEL: &str = "screenshot";
pub const DRAFT_COMMAND_NAME: &str = "capture_current_monitor_png_draft";
pub const CLIPBOARD_COMMAND_NAME: &str = "capture_current_monitor_to_clipboard";
pub const REGION_COMMAND_NAME: &str = "capture_region_to_clipboard";
pub const CLIPBOARD_FORMAT: &str = "CF_DIB + CF_BITMAP";
const MIN_REGION_SIZE: i32 = 8;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureTarget {
    CurrentMonitor,
    Region(CaptureRegion),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDraftRequest {
    pub target: CaptureTarget,
    pub include_cursor: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDraftResult {
    pub status: CaptureDraftStatus,
    pub message: String,
    pub png_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureDraftStatus {
    Disabled,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureClipboardResult {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
    pub clipboard_format: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorScreenshotPayload {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
    pub png_data_url: String,
}

pub struct CapturedRegionImage {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
    pub png_bytes: Vec<u8>,
    bmp_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotScaffoldStatus {
    pub module_label: &'static str,
    pub draft_command_name: &'static str,
    pub clipboard_command_name: &'static str,
    pub region_command_name: &'static str,
    pub dependencies_ready: bool,
    pub invoke_registered: bool,
    pub screen_reading_enabled: bool,
    pub clipboard_writing_enabled: bool,
    pub window_entry_enabled: bool,
    pub next_step: &'static str,
}

pub fn scaffold_status() -> ScreenshotScaffoldStatus {
    ScreenshotScaffoldStatus {
        module_label: MODULE_LABEL,
        draft_command_name: DRAFT_COMMAND_NAME,
        clipboard_command_name: CLIPBOARD_COMMAND_NAME,
        region_command_name: REGION_COMMAND_NAME,
        dependencies_ready: true,
        invoke_registered: true,
        screen_reading_enabled: true,
        clipboard_writing_enabled: true,
        window_entry_enabled: true,
        next_step:
            "截图横条菜单已接入复制、提取和翻译；继续保持 Tauri/WebView 遮罩和预览子窗口关闭",
    }
}

pub fn disabled_capture_result(_request: CaptureDraftRequest) -> CaptureDraftResult {
    CaptureDraftResult {
        status: CaptureDraftStatus::Disabled,
        message: "历史草稿接口保持禁用；真实区域截图请使用 capture_region_to_clipboard".to_string(),
        png_bytes: Vec::new(),
    }
}

pub fn capture_current_monitor_to_clipboard() -> Result<CaptureClipboardResult, String> {
    let monitors = Monitor::all().map_err(|error| format!("读取显示器列表失败：{error}"))?;
    if monitors.is_empty() {
        return Err("未发现可截图的显示器".to_string());
    }

    let monitor_index = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);
    let monitor = monitors
        .into_iter()
        .nth(monitor_index)
        .ok_or_else(|| "选择显示器失败，请稍后重试".to_string())?;
    let monitor_name = monitor
        .friendly_name()
        .or_else(|_| monitor.name())
        .unwrap_or_else(|_| "当前显示器".to_string());

    let image = monitor
        .capture_image()
        .map_err(|error| format!("读取屏幕失败：{error}"))?;
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return Err("截图结果为空，请稍后重试".to_string());
    }

    let bmp_bytes = rgba_image_to_bmp_bytes(&image)?;
    copy_bmp_bytes_to_clipboard(&bmp_bytes)?;

    Ok(CaptureClipboardResult {
        width,
        height,
        monitor_name: monitor_name.clone(),
        clipboard_format: CLIPBOARD_FORMAT,
        message: format!(
            "已复制当前显示器截图：{}x{}（{}）",
            width, height, monitor_name
        ),
    })
}

pub fn capture_current_monitor_screenshot() -> Result<MonitorScreenshotPayload, String> {
    let monitors = Monitor::all().map_err(|error| format!("读取显示器列表失败：{error}"))?;
    if monitors.is_empty() {
        return Err("未发现可截图的显示器".to_string());
    }

    let monitor_index = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);
    let monitor = monitors
        .into_iter()
        .nth(monitor_index)
        .ok_or_else(|| "选择显示器失败，请稍后重试".to_string())?;
    let monitor_name = monitor
        .friendly_name()
        .or_else(|_| monitor.name())
        .unwrap_or_else(|_| "当前显示器".to_string());

    let image = monitor
        .capture_image()
        .map_err(|error| format!("读取屏幕失败：{error}"))?;
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return Err("截图结果为空，请稍后重试".to_string());
    }

    let png_bytes = rgba_image_to_png_bytes(&image)?;
    let encoded_image = general_purpose::STANDARD.encode(png_bytes);
    let png_data_url = format!("data:image/png;base64,{encoded_image}");

    Ok(MonitorScreenshotPayload {
        width,
        height,
        monitor_name,
        png_data_url,
    })
}

fn copy_bmp_bytes_to_clipboard(bmp_bytes: &[u8]) -> Result<(), String> {
    const BMP_FILE_HEADER_SIZE: usize = 14;

    let _clipboard = clipboard_win::Clipboard::new_attempts(10)
        .map_err(|_| "打开系统剪贴板失败，请稍后再试".to_string())?;
    let dib_bytes = bmp_bytes
        .get(BMP_FILE_HEADER_SIZE..)
        .ok_or_else(|| "截图图片格式无效，无法复制".to_string())?;

    clipboard_win::raw::empty().map_err(|_| "清空剪贴板失败，请稍后再试".to_string())?;
    clipboard_win::raw::set_without_clear(clipboard_win::formats::CF_DIB, dib_bytes)
        .map_err(|_| "写入图片到剪贴板失败，请稍后再试".to_string())?;
    clipboard_win::raw::set_bitmap_with(bmp_bytes, clipboard_win::options::NoClear)
        .map_err(|_| "写入图片到剪贴板失败，请稍后再试".to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PointI {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RectI {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl RectI {
    fn normalized(start: PointI, end: PointI) -> Self {
        Self {
            left: start.x.min(end.x),
            top: start.y.min(end.y),
            right: start.x.max(end.x),
            bottom: start.y.max(end.y),
        }
    }

    fn width(self) -> i32 {
        (self.right - self.left).max(0)
    }

    fn height(self) -> i32 {
        (self.bottom - self.top).max(0)
    }

    fn contains(self, point: PointI) -> bool {
        point.x >= self.left
            && point.x <= self.right
            && point.y >= self.top
            && point.y <= self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionMenuAction {
    Copy,
    Extract,
    Translate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionMenuSelection {
    pub action: RegionMenuAction,
    pub screen_x: i32,
    pub screen_y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SelectedRegion {
    screen_x: i32,
    screen_y: i32,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverlayResult {
    None,
    Action(RegionMenuSelection),
    Cancel,
}

#[derive(Clone, Copy, Debug)]
struct MenuButton {
    action: RegionMenuAction,
    rect: RectI,
}

struct NativeOverlayState {
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: i32,
    monitor_height: i32,
    dragging: bool,
    drag_start: Option<PointI>,
    drag_current: Option<PointI>,
    selection: Option<RectI>,
    menu_buttons: Vec<MenuButton>,
    hovered_menu_action: Option<RegionMenuAction>,
    pressed_menu_action: Option<RegionMenuAction>,
    result: OverlayResult,
    theme: NativeTheme,
}

impl NativeOverlayState {
    fn new(
        monitor_x: i32,
        monitor_y: i32,
        monitor_width: i32,
        monitor_height: i32,
        theme: NativeTheme,
    ) -> Self {
        Self {
            monitor_x,
            monitor_y,
            monitor_width,
            monitor_height,
            dragging: false,
            drag_start: None,
            drag_current: None,
            selection: None,
            menu_buttons: Vec::new(),
            hovered_menu_action: None,
            pressed_menu_action: None,
            result: OverlayResult::None,
            theme,
        }
    }

    fn active_rect(&self) -> Option<RectI> {
        if self.dragging {
            match (self.drag_start, self.drag_current) {
                (Some(start), Some(current)) => Some(RectI::normalized(start, current)),
                _ => None,
            }
        } else {
            self.selection
        }
    }

    fn set_selection(&mut self, rect: RectI) {
        self.selection = Some(rect);
        self.menu_buttons = menu_buttons_for(rect, self.monitor_width, self.monitor_height);
        self.hovered_menu_action = None;
        self.pressed_menu_action = None;
    }

    fn selected_region(&self) -> Option<SelectedRegion> {
        let rect = self.selection?;
        if rect.width() < MIN_REGION_SIZE || rect.height() < MIN_REGION_SIZE {
            return None;
        }
        Some(SelectedRegion {
            screen_x: self.monitor_x + rect.left,
            screen_y: self.monitor_y + rect.top,
            width: rect.width() as u32,
            height: rect.height() as u32,
        })
    }

    fn selected_action(&self, action: RegionMenuAction) -> Option<RegionMenuSelection> {
        let region = self.selected_region()?;
        Some(RegionMenuSelection {
            action,
            screen_x: region.screen_x,
            screen_y: region.screen_y,
            width: region.width,
            height: region.height,
        })
    }
}

#[allow(dead_code)]
pub fn select_region_action(theme: NativeTheme) -> Result<Option<RegionMenuSelection>, String> {
    let monitor = current_cursor_monitor()?;
    let monitor_x = monitor
        .x()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;
    let monitor_y = monitor
        .y()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;
    let monitor_width = i32::try_from(
        monitor
            .width()
            .map_err(|error| format!("读取显示器尺寸失败：{error}"))?,
    )
    .map_err(|_| "显示器宽度过大，无法框选".to_string())?;
    let monitor_height = i32::try_from(
        monitor
            .height()
            .map_err(|error| format!("读取显示器尺寸失败：{error}"))?,
    )
    .map_err(|_| "显示器高度过大，无法框选".to_string())?;

    native_overlay::run(monitor_x, monitor_y, monitor_width, monitor_height, theme)
}

pub fn capture_selected_region(
    selection: RegionMenuSelection,
) -> Result<CapturedRegionImage, String> {
    let monitor = Monitor::from_point(selection.screen_x, selection.screen_y)
        .map_err(|error| format!("定位截图显示器失败：{error}"))?;
    let monitor_x = monitor
        .x()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;
    let monitor_y = monitor
        .y()
        .map_err(|error| format!("读取显示器位置失败：{error}"))?;
    let monitor_width = monitor
        .width()
        .map_err(|error| format!("读取显示器尺寸失败：{error}"))?;
    let monitor_height = monitor
        .height()
        .map_err(|error| format!("读取显示器尺寸失败：{error}"))?;
    let monitor_name = monitor
        .friendly_name()
        .or_else(|_| monitor.name())
        .unwrap_or_else(|_| "当前显示器".to_string());

    let relative_x = u32::try_from(selection.screen_x - monitor_x)
        .map_err(|_| "截图区域超出显示器范围".to_string())?;
    let relative_y = u32::try_from(selection.screen_y - monitor_y)
        .map_err(|_| "截图区域超出显示器范围".to_string())?;
    let capture_width = selection
        .width
        .min(monitor_width.saturating_sub(relative_x));
    let capture_height = selection
        .height
        .min(monitor_height.saturating_sub(relative_y));
    if capture_width < MIN_REGION_SIZE as u32 || capture_height < MIN_REGION_SIZE as u32 {
        return Err("截图区域过小，请重新框选".to_string());
    }

    let image = monitor
        .capture_region(relative_x, relative_y, capture_width, capture_height)
        .map_err(|error| format!("读取截图区域失败：{error}"))?;
    let bmp_bytes = rgba_image_to_bmp_bytes(&image)?;
    let png_bytes = rgba_image_to_png_bytes(&image)?;

    Ok(CapturedRegionImage {
        width: capture_width,
        height: capture_height,
        monitor_name,
        png_bytes,
        bmp_bytes,
    })
}

pub fn copy_captured_region_to_clipboard(
    image: &CapturedRegionImage,
) -> Result<CaptureClipboardResult, String> {
    copy_bmp_bytes_to_clipboard(&image.bmp_bytes)?;

    Ok(CaptureClipboardResult {
        width: image.width,
        height: image.height,
        monitor_name: image.monitor_name.clone(),
        clipboard_format: CLIPBOARD_FORMAT,
        message: format!(
            "已复制区域截图：{}x{}（{}）",
            image.width, image.height, image.monitor_name
        ),
    })
}

fn current_cursor_monitor() -> Result<Monitor, String> {
    let mut point = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
    if unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) != 0 } {
        return Monitor::from_point(point.x, point.y)
            .map_err(|error| format!("定位当前显示器失败：{error}"));
    }

    let monitors = Monitor::all().map_err(|error| format!("读取显示器列表失败：{error}"))?;
    monitors
        .into_iter()
        .find(|monitor| monitor.is_primary().unwrap_or(false))
        .ok_or_else(|| "未发现可截图的显示器".to_string())
}

fn menu_buttons_for(selection: RectI, monitor_width: i32, monitor_height: i32) -> Vec<MenuButton> {
    const BUTTON_WIDTH: i32 = 68;
    const BUTTON_HEIGHT: i32 = 34;
    const GAP: i32 = 8;
    const MENU_PADDING: i32 = 8;

    let total_width = BUTTON_WIDTH * 3 + GAP * 2;
    let mut left = selection.right - total_width;
    let mut top = selection.bottom + MENU_PADDING;

    if total_width + MENU_PADDING * 2 > monitor_width {
        left = ((monitor_width - total_width) / 2).max(0);
    } else {
        if left < MENU_PADDING {
            left = MENU_PADDING;
        }
        if left + total_width > monitor_width - MENU_PADDING {
            left = (monitor_width - total_width - MENU_PADDING).max(MENU_PADDING);
        }
    }
    if top + BUTTON_HEIGHT > monitor_height - MENU_PADDING {
        top = selection.top - BUTTON_HEIGHT - MENU_PADDING;
    }
    if top < MENU_PADDING {
        top = MENU_PADDING;
    }

    [
        RegionMenuAction::Copy,
        RegionMenuAction::Extract,
        RegionMenuAction::Translate,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, action)| {
        let button_left = left + index as i32 * (BUTTON_WIDTH + GAP);
        MenuButton {
            action,
            rect: RectI {
                left: button_left,
                top,
                right: button_left + BUTTON_WIDTH,
                bottom: top + BUTTON_HEIGHT,
            },
        }
    })
    .collect()
}

#[cfg(windows)]
#[allow(dead_code)]
mod native_overlay {
    use super::*;
    use std::mem::zeroed;
    use windows_sys::Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen,
            CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, FrameRect,
            GetStockObject, InvalidateRect, Rectangle, RoundRect, SelectObject, SetBkMode,
            SetTextColor, UpdateWindow, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HDC, HOLLOW_BRUSH,
            PAINTSTRUCT, PS_SOLID, SRCCOPY, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
                GetWindowLongPtrW, KillTimer, LoadCursorW, PostQuitMessage, RegisterClassW,
                SetCursor, SetForegroundWindow, SetLayeredWindowAttributes, SetTimer,
                SetWindowLongPtrW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
                GWLP_USERDATA, IDC_ARROW, IDC_CROSS, LWA_COLORKEY, MSG, SW_SHOW, WM_DESTROY,
                WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
                WM_NCDESTROY, WM_PAINT, WM_RBUTTONDOWN, WM_SETCURSOR, WM_TIMER, WNDCLASSW,
                WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
            },
        },
    };

    const CLASS_NAME: &str = "QuickPickNativeRegionOverlay";
    const WINDOW_TITLE: &str = "QuickPick 区域截图";
    const TRANSPARENT_KEY: COLORREF = 0x00030201;
    const TIMER_ID: usize = 1;
    const TIMEOUT_MS: u32 = 60_000;
    const ESC_KEY: WPARAM = 0x1b;
    const UI_FONT_FACE: &str = "Segoe UI";
    const UI_FONT_SIZE: i32 = 15;
    const UI_FONT_WEIGHT_STRONG: i32 = 600;

    #[derive(Clone, Copy)]
    enum ButtonVisualState {
        Default,
        Hovered,
        Pressed,
    }

    #[derive(Clone, Copy)]
    struct OverlayPalette {
        selection: COLORREF,
        selection_inner: COLORREF,
        panel_bg: COLORREF,
        panel_border: COLORREF,
        button_bg: COLORREF,
        button_hover: COLORREF,
        button_pressed: COLORREF,
        button_border: COLORREF,
        text: COLORREF,
    }

    pub fn run(
        monitor_x: i32,
        monitor_y: i32,
        monitor_width: i32,
        monitor_height: i32,
        theme: NativeTheme,
    ) -> Result<Option<RegionMenuSelection>, String> {
        if monitor_width < MIN_REGION_SIZE || monitor_height < MIN_REGION_SIZE {
            return Err("显示器尺寸过小，无法框选".to_string());
        }

        let class_name = to_wide(CLASS_NAME);
        let title = to_wide(WINDOW_TITLE);
        let instance = unsafe { GetModuleHandleW(null()) };
        let cursor = unsafe { LoadCursorW(null_mut(), IDC_CROSS) };
        let window_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            hCursor: cursor,
            lpszClassName: class_name.as_ptr(),
            ..unsafe { zeroed() }
        };

        unsafe {
            RegisterClassW(&window_class);
        }

        let mut state =
            NativeOverlayState::new(monitor_x, monitor_y, monitor_width, monitor_height, theme);
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_POPUP,
                monitor_x,
                monitor_y,
                monitor_width,
                monitor_height,
                null_mut(),
                null_mut(),
                instance,
                null(),
            )
        };

        if hwnd.is_null() {
            return Err("创建原生区域截图层失败".to_string());
        }

        unsafe {
            SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                &mut state as *mut NativeOverlayState as isize,
            );
            SetLayeredWindowAttributes(hwnd, TRANSPARENT_KEY, 255, LWA_COLORKEY);
            SetTimer(hwnd, TIMER_ID, TIMEOUT_MS, None);
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
            SetForegroundWindow(hwnd);
        }

        let mut message: MSG = unsafe { zeroed() };
        loop {
            let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
            if result <= 0 {
                break;
            }
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }

        let region = match state.result {
            OverlayResult::Action(selection) => Some(selection),
            OverlayResult::None | OverlayResult::Cancel => None,
        };
        Ok(region)
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NativeOverlayState;
        if state_ptr.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let state = &mut *state_ptr;

        match message {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_overlay(hwnd, state);
                0
            }
            WM_SETCURSOR => {
                set_overlay_cursor(!(state.selection.is_some() && !state.dragging));
                1
            }
            WM_LBUTTONDOWN => {
                let point = clamp_point(
                    lparam_point(lparam),
                    state.monitor_width,
                    state.monitor_height,
                );
                if let Some(button) = state
                    .menu_buttons
                    .iter()
                    .find(|button| button.rect.contains(point))
                    .copied()
                {
                    state.hovered_menu_action = Some(button.action);
                    state.pressed_menu_action = Some(button.action);
                    SetCapture(hwnd);
                    invalidate(hwnd);
                    return 0;
                }

                state.dragging = true;
                state.drag_start = Some(point);
                state.drag_current = Some(point);
                state.selection = None;
                state.menu_buttons.clear();
                state.hovered_menu_action = None;
                state.pressed_menu_action = None;
                SetCapture(hwnd);
                set_overlay_cursor(true);
                invalidate(hwnd);
                0
            }
            WM_MOUSEMOVE => {
                if state.dragging {
                    set_overlay_cursor(true);
                    state.drag_current = Some(clamp_point(
                        lparam_point(lparam),
                        state.monitor_width,
                        state.monitor_height,
                    ));
                    invalidate(hwnd);
                } else if state.selection.is_some() {
                    set_overlay_cursor(false);
                    let point = clamp_point(
                        lparam_point(lparam),
                        state.monitor_width,
                        state.monitor_height,
                    );
                    let next_hover = state
                        .menu_buttons
                        .iter()
                        .find(|button| button.rect.contains(point))
                        .map(|button| button.action);
                    if next_hover != state.hovered_menu_action {
                        state.hovered_menu_action = next_hover;
                        invalidate(hwnd);
                    }
                }
                0
            }
            WM_LBUTTONUP => {
                if let Some(pressed) = state.pressed_menu_action.take() {
                    ReleaseCapture();
                    let point = clamp_point(
                        lparam_point(lparam),
                        state.monitor_width,
                        state.monitor_height,
                    );
                    let released = state
                        .menu_buttons
                        .iter()
                        .find(|button| button.rect.contains(point))
                        .map(|button| button.action);
                    state.hovered_menu_action = released;
                    if released == Some(pressed) {
                        if let Some(selection) = state.selected_action(pressed) {
                            state.result = OverlayResult::Action(selection);
                            let _ = KillTimer(hwnd, TIMER_ID);
                            let _ = DestroyWindow(hwnd);
                        }
                    } else {
                        invalidate(hwnd);
                    }
                    return 0;
                }

                if state.dragging {
                    state.dragging = false;
                    ReleaseCapture();
                    let current = clamp_point(
                        lparam_point(lparam),
                        state.monitor_width,
                        state.monitor_height,
                    );
                    if let Some(start) = state.drag_start {
                        let rect = RectI::normalized(start, current);
                        if rect.width() >= MIN_REGION_SIZE && rect.height() >= MIN_REGION_SIZE {
                            state.set_selection(rect);
                        } else {
                            state.selection = None;
                            state.menu_buttons.clear();
                            state.hovered_menu_action = None;
                            state.pressed_menu_action = None;
                        }
                    }
                    state.drag_start = None;
                    state.drag_current = None;
                    set_overlay_cursor(state.selection.is_none());
                    invalidate(hwnd);
                }
                0
            }
            WM_RBUTTONDOWN => {
                state.result = OverlayResult::Cancel;
                let _ = KillTimer(hwnd, TIMER_ID);
                let _ = DestroyWindow(hwnd);
                0
            }
            WM_KEYDOWN => {
                if wparam == ESC_KEY {
                    state.result = OverlayResult::Cancel;
                    let _ = KillTimer(hwnd, TIMER_ID);
                    let _ = DestroyWindow(hwnd);
                    return 0;
                }
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_TIMER => {
                if wparam == TIMER_ID {
                    state.result = OverlayResult::Cancel;
                    let _ = DestroyWindow(hwnd);
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }

    unsafe fn paint_overlay(hwnd: HWND, state: &NativeOverlayState) {
        let mut paint: PAINTSTRUCT = zeroed();
        let window_dc = BeginPaint(hwnd, &mut paint);
        let full_rect = RECT {
            left: 0,
            top: 0,
            right: state.monitor_width,
            bottom: state.monitor_height,
        };
        if state.monitor_width <= 0 || state.monitor_height <= 0 {
            EndPaint(hwnd, &paint);
            return;
        }

        let buffer_dc = CreateCompatibleDC(window_dc);
        let buffer_bitmap =
            CreateCompatibleBitmap(window_dc, state.monitor_width, state.monitor_height);
        if buffer_dc.is_null() || buffer_bitmap.is_null() {
            if !buffer_bitmap.is_null() {
                DeleteObject(buffer_bitmap as _);
            }
            if !buffer_dc.is_null() {
                DeleteDC(buffer_dc);
            }
            paint_overlay_body(window_dc, state, full_rect);
            EndPaint(hwnd, &paint);
            return;
        }

        let old_bitmap = SelectObject(buffer_dc, buffer_bitmap as _);
        paint_overlay_body(buffer_dc, state, full_rect);
        BitBlt(
            window_dc,
            0,
            0,
            state.monitor_width,
            state.monitor_height,
            buffer_dc,
            0,
            0,
            SRCCOPY,
        );
        if !old_bitmap.is_null() {
            SelectObject(buffer_dc, old_bitmap);
        }
        DeleteObject(buffer_bitmap as _);
        DeleteDC(buffer_dc);
        EndPaint(hwnd, &paint);
    }

    unsafe fn paint_overlay_body(hdc: HDC, state: &NativeOverlayState, full_rect: RECT) {
        fill_rect(hdc, full_rect, TRANSPARENT_KEY);
        let palette = overlay_palette(state.theme);

        if let Some(rect) = state.active_rect() {
            draw_selection(hdc, rect, palette);
        }

        if state.selection.is_some() {
            draw_menu_shell(hdc, &state.menu_buttons, palette);
            for button in &state.menu_buttons {
                let visual_state = if state.pressed_menu_action == Some(button.action) {
                    ButtonVisualState::Pressed
                } else if state.hovered_menu_action == Some(button.action) {
                    ButtonVisualState::Hovered
                } else {
                    ButtonVisualState::Default
                };
                draw_button(hdc, *button, visual_state, palette);
            }
        } else if !state.dragging {
            draw_tip(hdc, state.monitor_width, palette);
        }
    }

    fn overlay_palette(theme: NativeTheme) -> OverlayPalette {
        if theme.is_dark() {
            OverlayPalette {
                selection: rgb(82, 154, 184),
                selection_inner: rgb(8, 10, 13),
                panel_bg: rgb(18, 22, 28),
                panel_border: rgb(54, 63, 76),
                button_bg: rgb(30, 36, 45),
                button_hover: rgb(38, 47, 58),
                button_pressed: rgb(24, 29, 37),
                button_border: rgb(65, 77, 92),
                text: rgb(232, 237, 244),
            }
        } else {
            OverlayPalette {
                selection: rgb(255, 255, 255),
                selection_inner: rgb(255, 255, 255),
                panel_bg: rgb(248, 251, 255),
                panel_border: rgb(193, 214, 249),
                button_bg: rgb(238, 244, 255),
                button_hover: rgb(225, 236, 255),
                button_pressed: rgb(213, 230, 255),
                button_border: rgb(188, 211, 249),
                text: rgb(23, 34, 53),
            }
        }
    }

    unsafe fn draw_selection(hdc: HDC, rect: RectI, palette: OverlayPalette) {
        let selection_rect = to_rect(rect);
        let brush = CreateSolidBrush(palette.selection);
        FrameRect(hdc, &selection_rect, brush);
        let inner = RECT {
            left: rect.left + 1,
            top: rect.top + 1,
            right: rect.right - 1,
            bottom: rect.bottom - 1,
        };
        FrameRect(hdc, &inner, brush);
        DeleteObject(brush as _);

        let pen = CreatePen(PS_SOLID, 1, palette.selection_inner);
        let old_pen = SelectObject(hdc, pen as _);
        let old_brush = SelectObject(hdc, GetStockObject(HOLLOW_BRUSH));
        Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        DeleteObject(pen as _);
    }

    unsafe fn draw_tip(hdc: HDC, monitor_width: i32, palette: OverlayPalette) {
        let width = 286;
        let left = ((monitor_width - width) / 2).max(12);
        let rect = RECT {
            left,
            top: 18,
            right: left + width,
            bottom: 52,
        };
        paint_round_rect(hdc, rect, 14, palette.panel_bg, palette.panel_border);
        draw_text(hdc, "拖拽框选，Esc/右键取消", rect, palette.text);
    }

    unsafe fn draw_menu_shell(hdc: HDC, buttons: &[MenuButton], palette: OverlayPalette) {
        if buttons.is_empty() {
            return;
        }

        let shell = RectI {
            left: buttons[0].rect.left - 6,
            top: buttons[0].rect.top - 6,
            right: buttons[buttons.len() - 1].rect.right + 6,
            bottom: buttons[0].rect.bottom + 6,
        };
        let shell_rect = to_rect(shell);
        paint_round_rect(hdc, shell_rect, 18, palette.panel_bg, palette.panel_border);
    }

    unsafe fn draw_button(
        hdc: HDC,
        button: MenuButton,
        visual_state: ButtonVisualState,
        palette: OverlayPalette,
    ) {
        let background = match visual_state {
            ButtonVisualState::Pressed => palette.button_pressed,
            ButtonVisualState::Hovered => palette.button_hover,
            ButtonVisualState::Default => palette.button_bg,
        };
        let rect = match visual_state {
            ButtonVisualState::Pressed => inset_rect(to_rect(button.rect), 1, 1),
            ButtonVisualState::Default | ButtonVisualState::Hovered => to_rect(button.rect),
        };
        paint_round_rect(hdc, rect, 12, background, palette.button_border);
        let label = match button.action {
            RegionMenuAction::Copy => "复制",
            RegionMenuAction::Extract => "提取",
            RegionMenuAction::Translate => "翻译",
        };
        draw_text(hdc, label, rect, palette.text);
    }

    unsafe fn draw_text(hdc: HDC, text: &str, mut rect: RECT, color: COLORREF) {
        let wide = to_wide(text);
        let face = to_wide(UI_FONT_FACE);
        let font = CreateFontW(
            -UI_FONT_SIZE,
            0,
            0,
            0,
            UI_FONT_WEIGHT_STRONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            face.as_ptr(),
        );
        let old_font = if font.is_null() {
            null_mut()
        } else {
            SelectObject(hdc, font as _)
        };
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, color);
        DrawTextW(
            hdc,
            wide.as_ptr(),
            -1,
            &mut rect,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER,
        );
        if !old_font.is_null() {
            SelectObject(hdc, old_font);
        }
        if !font.is_null() {
            DeleteObject(font as _);
        }
    }

    unsafe fn fill_rect(hdc: HDC, rect: RECT, color: COLORREF) {
        let brush = CreateSolidBrush(color);
        FillRect(hdc, &rect, brush);
        DeleteObject(brush as _);
    }

    unsafe fn paint_round_rect(
        hdc: HDC,
        rect: RECT,
        radius: i32,
        fill: COLORREF,
        border: COLORREF,
    ) {
        let brush = CreateSolidBrush(fill);
        let pen = CreatePen(PS_SOLID, 1, border);
        let old_brush = SelectObject(hdc, brush as _);
        let old_pen = SelectObject(hdc, pen as _);
        RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
        );
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
        DeleteObject(pen as _);
        DeleteObject(brush as _);
    }

    unsafe fn set_overlay_cursor(cross: bool) {
        let cursor_id = if cross { IDC_CROSS } else { IDC_ARROW };
        SetCursor(LoadCursorW(null_mut(), cursor_id));
    }

    fn inset_rect(rect: RECT, x: i32, y: i32) -> RECT {
        RECT {
            left: rect.left + x,
            top: rect.top + y,
            right: rect.right - x,
            bottom: rect.bottom - y,
        }
    }

    fn to_rect(rect: RectI) -> RECT {
        RECT {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }

    fn lparam_point(lparam: LPARAM) -> PointI {
        PointI {
            x: (lparam & 0xffff) as i16 as i32,
            y: ((lparam >> 16) & 0xffff) as i16 as i32,
        }
    }

    fn clamp_point(point: PointI, width: i32, height: i32) -> PointI {
        PointI {
            x: point.x.clamp(0, width.saturating_sub(1)),
            y: point.y.clamp(0, height.saturating_sub(1)),
        }
    }

    unsafe fn invalidate(hwnd: HWND) {
        InvalidateRect(hwnd, null(), 0);
    }

    fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
        red as COLORREF | ((green as COLORREF) << 8) | ((blue as COLORREF) << 16)
    }

    fn to_wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
mod native_overlay {
    use super::*;

    pub fn run(
        _monitor_x: i32,
        _monitor_y: i32,
        _monitor_width: i32,
        _monitor_height: i32,
        _theme: NativeTheme,
    ) -> Result<Option<RegionMenuSelection>, String> {
        Err("区域截图框选当前只支持 Windows".to_string())
    }
}

fn rgba_image_to_bmp_bytes(image: &image::RgbaImage) -> Result<Vec<u8>, String> {
    const BMP_FILE_HEADER_SIZE: usize = 14;
    const BMP_INFO_HEADER_SIZE: usize = 40;
    const BYTES_PER_PIXEL: usize = 4;

    let width = image.width() as usize;
    let height = image.height() as usize;
    if width == 0 || height == 0 {
        return Err("截图结果为空，请稍后重试".to_string());
    }

    let row_size = width
        .checked_mul(BYTES_PER_PIXEL)
        .ok_or_else(|| "截图尺寸过大，无法复制".to_string())?;
    let pixel_size = row_size
        .checked_mul(height)
        .ok_or_else(|| "截图尺寸过大，无法复制".to_string())?;
    let header_size = BMP_FILE_HEADER_SIZE + BMP_INFO_HEADER_SIZE;
    let file_size = header_size
        .checked_add(pixel_size)
        .ok_or_else(|| "截图尺寸过大，无法复制".to_string())?;

    let width_i32 = i32::try_from(width).map_err(|_| "截图宽度过大，无法复制".to_string())?;
    let height_i32 = i32::try_from(height).map_err(|_| "截图高度过大，无法复制".to_string())?;
    let file_size_u32 =
        u32::try_from(file_size).map_err(|_| "截图尺寸过大，无法复制".to_string())?;
    let pixel_size_u32 =
        u32::try_from(pixel_size).map_err(|_| "截图尺寸过大，无法复制".to_string())?;

    let mut bytes = Vec::with_capacity(file_size);
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&file_size_u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(header_size as u32).to_le_bytes());

    bytes.extend_from_slice(&(BMP_INFO_HEADER_SIZE as u32).to_le_bytes());
    bytes.extend_from_slice(&width_i32.to_le_bytes());
    bytes.extend_from_slice(&height_i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&32u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&pixel_size_u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());

    let raw = image.as_raw();
    for y in (0..height).rev() {
        let row_start = y * row_size;
        let row_end = row_start + row_size;
        for rgba in raw[row_start..row_end].chunks_exact(BYTES_PER_PIXEL) {
            bytes.push(rgba[2]);
            bytes.push(rgba[1]);
            bytes.push(rgba[0]);
            bytes.push(rgba[3]);
        }
    }

    Ok(bytes)
}

fn rgba_image_to_png_bytes(image: &image::RgbaImage) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;

    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ColorType::Rgba8.into(),
        )
        .map_err(|_| "截图图片编码失败，请稍后重试".to_string())?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_exposes_native_region_capture() {
        let status = scaffold_status();

        assert!(status.dependencies_ready);
        assert!(status.invoke_registered);
        assert!(status.screen_reading_enabled);
        assert!(status.clipboard_writing_enabled);
        assert!(status.window_entry_enabled);
        assert_eq!(status.region_command_name, REGION_COMMAND_NAME);
    }

    #[test]
    fn menu_buttons_stay_inside_monitor_bounds() {
        let buttons = menu_buttons_for(
            RectI {
                left: 20,
                top: 20,
                right: 180,
                bottom: 100,
            },
            220,
            140,
        );

        assert_eq!(buttons.len(), 3);
        assert_eq!(buttons[0].action, RegionMenuAction::Copy);
        assert_eq!(buttons[1].action, RegionMenuAction::Extract);
        assert_eq!(buttons[2].action, RegionMenuAction::Translate);

        for button in buttons {
            assert!(button.rect.left >= 0);
            assert!(button.rect.top >= 0);
            assert!(button.rect.right <= 220);
            assert!(button.rect.bottom <= 140);
        }
    }

    #[test]
    fn disabled_capture_returns_no_image_bytes() {
        let result = disabled_capture_result(CaptureDraftRequest {
            target: CaptureTarget::CurrentMonitor,
            include_cursor: false,
        });

        assert!(matches!(result.status, CaptureDraftStatus::Disabled));
        assert!(result.png_bytes.is_empty());
    }

    #[test]
    fn rgba_image_is_encoded_as_bottom_up_bmp() {
        let image = image::RgbaImage::from_raw(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        )
        .expect("valid test image");

        let bytes = rgba_image_to_bmp_bytes(&image).expect("bmp bytes");

        assert_eq!(&bytes[0..2], b"BM");
        assert_eq!(bytes.len(), 14 + 40 + 16);
        assert_eq!(
            u32::from_le_bytes(bytes[2..6].try_into().unwrap()),
            bytes.len() as u32
        );
        assert_eq!(u32::from_le_bytes(bytes[10..14].try_into().unwrap()), 54);

        let pixels = &bytes[54..];
        assert_eq!(&pixels[0..4], &[255, 0, 0, 255]);
        assert_eq!(&pixels[4..8], &[255, 255, 255, 255]);
        assert_eq!(&pixels[8..12], &[0, 0, 255, 255]);
        assert_eq!(&pixels[12..16], &[0, 255, 0, 255]);
    }

    #[test]
    fn rgba_image_can_be_encoded_as_png() {
        let image =
            image::RgbaImage::from_raw(1, 1, vec![12, 34, 56, 255]).expect("valid test image");

        let bytes = rgba_image_to_png_bytes(&image).expect("png bytes");

        assert_eq!(&bytes[0..8], b"\x89PNG\r\n\x1a\n");
    }
}
