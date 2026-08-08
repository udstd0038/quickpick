use crate::{native_theme::NativeTheme, selection::SelectionSnapshot};
use std::{
    ptr::null_mut,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread,
    time::Duration,
};
use xcap::Monitor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionPopupAction {
    Copy,
    Translate,
    Summarize,
    Search,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayMenuAction {
    Settings,
    ToggleAutostart,
    Quit,
}

struct ResultPopupUpdate {
    title: String,
    content: String,
    detail: String,
    status: String,
    source_language: String,
    target_language: String,
    translation_direction: String,
    can_switch_language: bool,
}

pub enum ResultPopupEvent {
    TranslationChanged {
        source_language: String,
        target_language: String,
        direction: String,
    },
}

pub enum InputPopupEvent {
    TranslateRequested {
        input_text: String,
        source_language: String,
        target_language: String,
        direction: String,
    },
}

pub struct ResultPopupHandle {
    sender: Sender<ResultPopupUpdate>,
    event_receiver: Mutex<Receiver<ResultPopupEvent>>,
}

struct InputPopupUpdate {
    content: String,
    detail: String,
    status: String,
    source_language: String,
    target_language: String,
    translation_direction: String,
}

pub struct InputPopupHandle {
    sender: Sender<InputPopupUpdate>,
    event_receiver: Mutex<Receiver<InputPopupEvent>>,
}

impl InputPopupHandle {
    pub fn update(
        &self,
        content: String,
        detail: String,
        status: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
    ) -> bool {
        self.sender
            .send(InputPopupUpdate {
                content,
                detail,
                status,
                source_language,
                target_language,
                translation_direction,
            })
            .is_ok()
    }

    pub fn recv_event(&self) -> Option<InputPopupEvent> {
        self.event_receiver.lock().ok()?.recv().ok()
    }
}

impl ResultPopupHandle {
    pub fn update(
        &self,
        title: String,
        content: String,
        detail: String,
        status: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
        can_switch_language: bool,
    ) -> bool {
        self.sender
            .send(ResultPopupUpdate {
                title,
                content,
                detail,
                status,
                source_language,
                target_language,
                translation_direction,
                can_switch_language,
            })
            .is_ok()
    }

    pub fn recv_event(&self) -> Option<ResultPopupEvent> {
        self.event_receiver.lock().ok()?.recv().ok()
    }
}

pub fn select_text_action(
    snapshot: SelectionSnapshot,
    theme: NativeTheme,
) -> Result<Option<SelectionPopupAction>, String> {
    native_window::run_selection(snapshot, theme)
}

pub fn select_tray_menu_action(
    theme: NativeTheme,
    autostart_enabled: bool,
) -> Result<Option<TrayMenuAction>, String> {
    native_window::run_tray_menu(theme, autostart_enabled)
}

pub fn open_result_popup(
    title: String,
    content: String,
    detail: String,
    status: String,
    source_language: String,
    target_language: String,
    translation_direction: String,
    can_switch_language: bool,
    theme: NativeTheme,
) -> Result<ResultPopupHandle, String> {
    let (sender, receiver) = mpsc::channel();
    let (event_sender, event_receiver) = mpsc::channel();
    thread::Builder::new()
        .name("quickpick-result-popup".to_string())
        .spawn(move || {
            let _ = native_window::run_result_updatable(
                title,
                content,
                detail,
                status,
                source_language,
                target_language,
                translation_direction,
                can_switch_language,
                theme,
                receiver,
                event_sender,
            );
        })
        .map_err(|error| format!("创建原生结果弹窗线程失败：{error}"))?;

    Ok(ResultPopupHandle {
        sender,
        event_receiver: Mutex::new(event_receiver),
    })
}

pub fn show_result_popup(
    title: String,
    content: String,
    detail: String,
    source_language: String,
    target_language: String,
    translation_direction: String,
    can_switch_language: bool,
    theme: NativeTheme,
) {
    let _ = thread::Builder::new()
        .name("quickpick-result-popup".to_string())
        .spawn(move || {
            let _ = native_window::run_result(
                title,
                content,
                detail,
                "static".to_string(),
                source_language,
                target_language,
                translation_direction,
                can_switch_language,
                theme,
            );
        });
}

pub fn open_input_popup(
    input_text: String,
    source_language: String,
    target_language: String,
    translation_direction: String,
    content: String,
    detail: String,
    status: String,
    theme: NativeTheme,
) -> Result<InputPopupHandle, String> {
    let (sender, receiver) = mpsc::channel();
        let (event_sender, event_receiver) = mpsc::channel();
        let spawn_input = move || {
            match native_window::run_input_updatable(
                input_text,
                source_language,
                target_language,
                translation_direction,
                content,
                detail,
                status,
                theme,
                receiver,
                event_sender,
            ) {
                Ok(()) => {}
                Err(error) => eprintln!("QuickPick input popup thread error: {error}"),
            }
        };
        thread::Builder::new()
            .name("quickpick-input-popup".to_string())
            .spawn(spawn_input)
        .map_err(|error| format!("创建输入翻译弹窗线程失败：{error}"))?;

    Ok(InputPopupHandle {
        sender,
        event_receiver: Mutex::new(event_receiver),
    })
}

fn clamp_window_position(width: i32, height: i32) -> (i32, i32) {
    let mut point = windows_sys::Win32::Foundation::POINT { x: 160, y: 160 };
    let _ = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) };
    let fallback = (point.x + 18, point.y + 18);

    let monitor = match Monitor::from_point(point.x, point.y) {
        Ok(monitor) => monitor,
        Err(_) => return fallback,
    };
    let monitor_x = monitor.x().unwrap_or(0);
    let monitor_y = monitor.y().unwrap_or(0);
    let monitor_width = monitor.width().unwrap_or(1280) as i32;
    let monitor_height = monitor.height().unwrap_or(720) as i32;
    let margin = 14;

    let max_x = monitor_x + monitor_width - width - margin;
    let max_y = monitor_y + monitor_height - height - margin;
    let x = (point.x + 18).clamp(monitor_x + margin, max_x.max(monitor_x + margin));
    let y = (point.y + 18).clamp(monitor_y + margin, max_y.max(monitor_y + margin));
    (x, y)
}

fn clamp_tray_menu_position(width: i32, height: i32) -> (i32, i32) {
    let mut point = windows_sys::Win32::Foundation::POINT { x: 160, y: 160 };
    let _ = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) };
    let fallback = (point.x - width + 8, point.y - height - 8);

    let monitor = match Monitor::from_point(point.x, point.y) {
        Ok(monitor) => monitor,
        Err(_) => return fallback,
    };
    let monitor_x = monitor.x().unwrap_or(0);
    let monitor_y = monitor.y().unwrap_or(0);
    let monitor_width = monitor.width().unwrap_or(1280) as i32;
    let monitor_height = monitor.height().unwrap_or(720) as i32;
    let margin = 8;

    let max_x = monitor_x + monitor_width - width - margin;
    let max_y = monitor_y + monitor_height - height - margin;
    let x = (point.x - width + 8).clamp(monitor_x + margin, max_x.max(monitor_x + margin));
    let y = (point.y - height - 8).clamp(monitor_y + margin, max_y.max(monitor_y + margin));
    (x, y)
}

#[cfg(windows)]
mod native_window {
    use super::*;
    use std::mem::zeroed;
    use windows_sys::Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen,
            CreateRoundRectRgn, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint,
            FillRect, FrameRect, IntersectClipRect, InvalidateRect, LineTo, MoveToEx, RestoreDC,
            RoundRect, SaveDC, SelectObject, SetBkMode, SetTextColor, SetWindowRgn,
            UpdateWindow, DT_CALCRECT, DT_CENTER, DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE,
            DT_VCENTER,
            DT_WORDBREAK, HDC, PAINTSTRUCT, PS_SOLID, SRCCOPY, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, SetFocus},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
                GetCursorPos, GetForegroundWindow, GetMessageW, GetWindowLongPtrW, GetWindowRect,
                IsWindow, LoadCursorW, PeekMessageW, PostQuitMessage, RegisterClassW,
                SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow,
                TranslateMessage, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HWND_NOTOPMOST,
                HWND_TOPMOST, IDC_ARROW, MSG, PM_REMOVE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
                SWP_NOZORDER,
                SWP_SHOWWINDOW, SW_SHOW, SW_SHOWNOACTIVATE, WM_CHAR, WM_DESTROY, WM_ERASEBKGND,
                WM_KEYDOWN, WM_LBUTTONDOWN,
                WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCDESTROY, WM_PAINT,
                WM_QUIT, WM_SIZE, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
            },
        },
    };

    const RESULT_CLASS_NAME: &str = "QuickPickNativeResultPopup";
    const SELECTION_CLASS_NAME: &str = "QuickPickNativeSelectionPopup";
    const TRAY_MENU_CLASS_NAME: &str = "QuickPickNativeTrayMenu";
    const INPUT_CLASS_NAME: &str = "QuickPickNativeInputPopup";
    const RESULT_WINDOW_TITLE: &str = "QuickPick 结果";
    const SELECTION_WINDOW_TITLE: &str = "QuickPick 划词";
    const TRAY_MENU_WINDOW_TITLE: &str = "QuickPick 托盘菜单";
    const INPUT_WINDOW_TITLE: &str = "QuickPick 输入翻译";
    const ESC_KEY: WPARAM = 0x1b;
    const REVEAL_FRAMES: u32 = 8;
    const NATIVE_POPUP_WIDTH: i32 = 480;
    const UI_FONT_FACE: &str = "Segoe UI";
    const UI_FONT_SIZE: i32 = 15;
    const UI_FONT_WEIGHT_NORMAL: i32 = 400;
    const UI_FONT_WEIGHT_STRONG: i32 = 600;

    fn popup_margin(width: i32) -> i32 {
        if width <= 160 { 6 } else { 14 }
    }

    fn compact_popup(width: i32) -> bool {
        width <= 500
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ResultButtonKind {
        SourceLanguage,
        Direction,
        TargetLanguage,
        Copy,
        Pin,
        Close,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum InputButtonKind {
        SourceLanguage,
        Direction,
        TargetLanguage,
        Translate,
        Copy,
        Pin,
        Close,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum LanguageMenuKind {
        Source,
        Target,
    }

    trait LanguageMenuState {
        fn open_language_menu(&self) -> Option<LanguageMenuKind>;
        fn set_open_language_menu(&mut self, kind: Option<LanguageMenuKind>);
        fn hovered_language_option(&self) -> Option<usize>;
        fn set_hovered_language_option(&mut self, option: Option<usize>);
        fn source_language(&self) -> &str;
        fn set_source_language(&mut self, value: String);
        fn target_language(&self) -> &str;
        fn set_target_language(&mut self, value: String);
        fn translation_direction(&self) -> &str;
        fn set_translation_direction(&mut self, value: String);
        fn language_menu_allows_auto(&self, kind: LanguageMenuKind) -> bool {
            match kind {
                LanguageMenuKind::Source => self.translation_direction() != "left",
                LanguageMenuKind::Target => self.translation_direction() == "left",
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SelectionButtonKind {
        Copy,
        Translate,
        Summarize,
        Search,
        Close,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TrayMenuButtonKind {
        Settings,
        ToggleAutostart,
        Quit,
    }

    #[derive(Clone, Copy)]
    struct Button<T> {
        kind: T,
        rect: RECT,
    }

    #[derive(Clone, Copy)]
    enum ButtonVisualState {
        Default,
        Hovered,
        Pressed,
    }

    #[derive(Clone, Copy)]
    struct PopupPalette {
        window_bg: COLORREF,
        window_border: COLORREF,
        panel_bg: COLORREF,
        panel_border: COLORREF,
        text: COLORREF,
        text_secondary: COLORREF,
        button_bg: COLORREF,
        button_hover: COLORREF,
        button_pressed: COLORREF,
        button_border: COLORREF,
        primary_bg: COLORREF,
        primary_hover: COLORREF,
        primary_pressed: COLORREF,
        primary_border: COLORREF,
        primary_text: COLORREF,
        skeleton_base: COLORREF,
        skeleton_shine: COLORREF,
    }

    struct ResultPopupState {
        title: String,
        content: String,
        detail: String,
        status: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
        can_switch_language: bool,
        theme: NativeTheme,
        event_sender: Option<Sender<ResultPopupEvent>>,
        pinned: bool,
        scroll: i32,
        animation_tick: u32,
        reveal_frame: Option<u32>,
        restore_focus: bool,
        hovered_button: Option<ResultButtonKind>,
        pressed_button: Option<ResultButtonKind>,
        buttons: Vec<Button<ResultButtonKind>>,
        dragging: bool,
        drag_origin: (i32, i32),
        drag_window_rect: RECT,
        open_language_menu: Option<LanguageMenuKind>,
        hovered_language_option: Option<usize>,
        language_menu_options: Vec<Button<usize>>,
        content_rect: RECT,
    }

    impl LanguageMenuState for ResultPopupState {
        fn open_language_menu(&self) -> Option<LanguageMenuKind> {
            self.open_language_menu
        }
        fn set_open_language_menu(&mut self, kind: Option<LanguageMenuKind>) {
            self.open_language_menu = kind;
        }
        fn hovered_language_option(&self) -> Option<usize> {
            self.hovered_language_option
        }
        fn set_hovered_language_option(&mut self, option: Option<usize>) {
            self.hovered_language_option = option;
        }
        fn source_language(&self) -> &str {
            &self.source_language
        }
        fn set_source_language(&mut self, value: String) {
            self.source_language = value;
        }
        fn target_language(&self) -> &str {
            &self.target_language
        }
        fn set_target_language(&mut self, value: String) {
            self.target_language = value;
        }
        fn translation_direction(&self) -> &str {
            &self.translation_direction
        }
        fn set_translation_direction(&mut self, value: String) {
            self.translation_direction = value;
        }
    }

    impl DragState for ResultPopupState {
        fn drag_origin(&self) -> (i32, i32) {
            self.drag_origin
        }
        fn drag_window_rect(&self) -> RECT {
            self.drag_window_rect
        }
    }

    struct InputPopupState {
        input_text: String,
        content: String,
        detail: String,
        status: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
        theme: NativeTheme,
        event_sender: Option<Sender<InputPopupEvent>>,
        pinned: bool,
        scroll: i32,
        animation_tick: u32,
        reveal_frame: Option<u32>,
        hovered_button: Option<InputButtonKind>,
        pressed_button: Option<InputButtonKind>,
        buttons: Vec<Button<InputButtonKind>>,
        dragging: bool,
        drag_origin: (i32, i32),
        drag_window_rect: RECT,
        open_language_menu: Option<LanguageMenuKind>,
        hovered_language_option: Option<usize>,
        language_menu_options: Vec<Button<usize>>,
        input_focused: bool,
        edit_rect: RECT,
        content_rect: RECT,
        result_rect: RECT,
    }

    impl LanguageMenuState for InputPopupState {
        fn open_language_menu(&self) -> Option<LanguageMenuKind> {
            self.open_language_menu
        }
        fn set_open_language_menu(&mut self, kind: Option<LanguageMenuKind>) {
            self.open_language_menu = kind;
        }
        fn hovered_language_option(&self) -> Option<usize> {
            self.hovered_language_option
        }
        fn set_hovered_language_option(&mut self, option: Option<usize>) {
            self.hovered_language_option = option;
        }
        fn source_language(&self) -> &str {
            &self.source_language
        }
        fn set_source_language(&mut self, value: String) {
            self.source_language = value;
        }
        fn target_language(&self) -> &str {
            &self.target_language
        }
        fn set_target_language(&mut self, value: String) {
            self.target_language = value;
        }
        fn translation_direction(&self) -> &str {
            &self.translation_direction
        }
        fn set_translation_direction(&mut self, value: String) {
            self.translation_direction = value;
        }
    }

    impl DragState for InputPopupState {
        fn drag_origin(&self) -> (i32, i32) {
            self.drag_origin
        }
        fn drag_window_rect(&self) -> RECT {
            self.drag_window_rect
        }
    }

    struct SelectionPopupState {
        snapshot: SelectionSnapshot,
        theme: NativeTheme,
        buttons: Vec<Button<SelectionButtonKind>>,
        result: Option<SelectionPopupAction>,
        restore_focus: bool,
        hovered_button: Option<SelectionButtonKind>,
        pressed_button: Option<SelectionButtonKind>,
    }

    struct TrayMenuState {
        theme: NativeTheme,
        autostart_enabled: bool,
        buttons: Vec<Button<TrayMenuButtonKind>>,
        result: Option<TrayMenuAction>,
        hovered_button: Option<TrayMenuButtonKind>,
        pressed_button: Option<TrayMenuButtonKind>,
    }

    impl ResultPopupState {
        fn new(
            title: String,
            content: String,
            detail: String,
            status: String,
            source_language: String,
            target_language: String,
            translation_direction: String,
            can_switch_language: bool,
            theme: NativeTheme,
            event_sender: Option<Sender<ResultPopupEvent>>,
        ) -> Self {
            Self {
                title,
                content,
                detail,
                status,
                source_language,
                target_language,
                translation_direction,
                can_switch_language,
                theme,
                event_sender,
                pinned: false,
                scroll: 0,
                animation_tick: 0,
                reveal_frame: None,
                restore_focus: true,
                hovered_button: None,
                pressed_button: None,
                buttons: Vec::new(),
                dragging: false,
                drag_origin: (0, 0),
                drag_window_rect: empty_rect(),
                open_language_menu: None,
                hovered_language_option: None,
                language_menu_options: Vec::new(),
                content_rect: empty_rect(),
            }
        }

        fn loading(&self) -> bool {
            self.status == "loading"
        }
    }

    impl SelectionPopupState {
        fn new(snapshot: SelectionSnapshot, theme: NativeTheme) -> Self {
            Self {
                snapshot,
                theme,
                buttons: Vec::new(),
                result: None,
                restore_focus: true,
                hovered_button: None,
                pressed_button: None,
            }
        }
    }

    impl TrayMenuState {
        fn new(theme: NativeTheme, autostart_enabled: bool) -> Self {
            Self {
                theme,
                autostart_enabled,
                buttons: Vec::new(),
                result: None,
                hovered_button: None,
                pressed_button: None,
            }
        }
    }

    pub fn run_result(
        title: String,
        content: String,
        detail: String,
        status: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
        can_switch_language: bool,
        theme: NativeTheme,
    ) -> Result<(), String> {
        let width = NATIVE_POPUP_WIDTH;
        let height = 460;
        let (x, y) = clamp_window_position(width, height);
        let previous_foreground = unsafe { GetForegroundWindow() };
        let mut state = ResultPopupState::new(
            title,
            content,
            detail,
            status,
            source_language,
            target_language,
            translation_direction,
            can_switch_language,
            theme,
            None,
        );
        let hwnd = create_popup_window(
            RESULT_CLASS_NAME,
            RESULT_WINDOW_TITLE,
            result_window_proc,
            x,
            y,
            width,
            height,
            &mut state as *mut ResultPopupState as isize,
            false,
        )?;
        message_loop(hwnd);
        if state.restore_focus {
            restore_previous_foreground(previous_foreground);
        }
        Ok(())
    }

    pub fn run_result_updatable(
        title: String,
        content: String,
        detail: String,
        status: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
        can_switch_language: bool,
        theme: NativeTheme,
        receiver: Receiver<ResultPopupUpdate>,
        event_sender: Sender<ResultPopupEvent>,
    ) -> Result<(), String> {
        let width = NATIVE_POPUP_WIDTH;
        let height = 420;
        let (x, y) = clamp_window_position(width, height);
        let previous_foreground = unsafe { GetForegroundWindow() };
        let mut state = ResultPopupState::new(
            title,
            content,
            detail,
            status,
            source_language,
            target_language,
            translation_direction,
            can_switch_language,
            theme,
            Some(event_sender),
        );
        let hwnd = create_popup_window(
            RESULT_CLASS_NAME,
            RESULT_WINDOW_TITLE,
            result_window_proc,
            x,
            y,
            width,
            height,
            &mut state as *mut ResultPopupState as isize,
            false,
        )?;

        result_message_loop(hwnd, &mut state, receiver);
        if state.restore_focus {
            restore_previous_foreground(previous_foreground);
        }
        Ok(())
    }

    pub fn run_selection(
        snapshot: SelectionSnapshot,
        theme: NativeTheme,
    ) -> Result<Option<SelectionPopupAction>, String> {
        let width = 404;
        let height = 58;
        let (x, y) = clamp_window_position(width, height);
        let previous_foreground = unsafe { GetForegroundWindow() };
        let mut state = SelectionPopupState::new(snapshot, theme);
        let hwnd = create_popup_window(
            SELECTION_CLASS_NAME,
            SELECTION_WINDOW_TITLE,
            selection_window_proc,
            x,
            y,
            width,
            height,
            &mut state as *mut SelectionPopupState as isize,
            true,
        )?;

        message_loop(hwnd);
        if state.restore_focus {
            restore_previous_foreground(previous_foreground);
        }
        Ok(state.result)
    }

    pub fn run_tray_menu(
        theme: NativeTheme,
        autostart_enabled: bool,
    ) -> Result<Option<TrayMenuAction>, String> {
        let width = 142;
        let height = 126;
        let (x, y) = clamp_tray_menu_position(width, height);
        let mut state = TrayMenuState::new(theme, autostart_enabled);
        let hwnd = create_popup_window(
            TRAY_MENU_CLASS_NAME,
            TRAY_MENU_WINDOW_TITLE,
            tray_menu_window_proc,
            x,
            y,
            width,
            height,
            &mut state as *mut TrayMenuState as isize,
            true,
        )?;
        message_loop(hwnd);
        Ok(state.result)
    }

    unsafe extern "system" fn result_window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ResultPopupState;
        if state_ptr.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let state = &mut *state_ptr;

        match message {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_result(hwnd, state);
                0
            }
            WM_MOUSEMOVE => {
                let point = point_from_lparam(lparam);
                if state.dragging {
                    drag_popup_window(hwnd, state, point);
                }
                let previous_language_hover = state.hovered_language_option;
                if state.open_language_menu.is_some() {
                    state.hovered_language_option = hit_button(&state.language_menu_options, point);
                    if state.hovered_language_option != previous_language_hover {
                        unsafe {
                            InvalidateRect(hwnd, null_mut(), 0);
                        }
                    }
                }
                let previous_hover = state.hovered_button;
                let next_hover = hit_button(&state.buttons, point);
                if next_hover != previous_hover {
                    state.hovered_button = next_hover;
                    invalidate_result_buttons(hwnd, state, previous_hover, next_hover);
                }
                0
            }
            WM_LBUTTONDOWN => {
                let point = point_from_lparam(lparam);
                if point.1 <= 44 && hit_button(&state.buttons, point).is_none() {
                    state.dragging = true;
                    state.drag_origin = cursor_screen_pos();
                    unsafe {
                        GetWindowRect(hwnd, &mut state.drag_window_rect);
                    }
                    SetCapture(hwnd);
                    return 0;
                }
                if state.open_language_menu.is_some() {
                    if let Some(index) = hit_button(&state.language_menu_options, point) {
                        let changed = select_language_option(state, index);
                        if changed {
                            trigger_translation_refresh(hwnd, state);
                        } else {
                            state.open_language_menu = None;
                            state.hovered_language_option = None;
                            unsafe {
                                InvalidateRect(hwnd, null_mut(), 0);
                            }
                        }
                        return 0;
                    }

                    if hit_button(&state.buttons, point).is_none() {
                        state.open_language_menu = None;
                        state.hovered_language_option = None;
                        unsafe {
                            InvalidateRect(hwnd, null_mut(), 0);
                        }
                        return 0;
                    }
                }

                if let Some(kind) = hit_button(&state.buttons, point) {
                    state.hovered_button = Some(kind);
                    state.pressed_button = Some(kind);
                    SetCapture(hwnd);
                    invalidate_result_buttons(hwnd, state, Some(kind), Some(kind));
                    return 0;
                }

                0
            }
            WM_LBUTTONUP => {
                if state.dragging {
                    state.dragging = false;
                    ReleaseCapture();
                }
                if let Some(pressed) = state.pressed_button.take() {
                    ReleaseCapture();
                    let point = point_from_lparam(lparam);
                    let released = hit_button(&state.buttons, point);
                    state.hovered_button = released;
                    if released == Some(pressed) {
                        match pressed {
                            ResultButtonKind::SourceLanguage => {
                                toggle_language_menu(hwnd, state, LanguageMenuKind::Source);
                            }
                            ResultButtonKind::TargetLanguage => {
                                toggle_language_menu(hwnd, state, LanguageMenuKind::Target);
                            }
                            ResultButtonKind::Direction => {
                                state.open_language_menu = None;
                                state.hovered_language_option = None;
                                state.translation_direction =
                                    if state.translation_direction == "left" {
                                        "right".to_string()
                                    } else {
                                        if state.source_language == "auto" {
                                            state.source_language =
                                                opposite_language(&state.target_language);
                                        }
                                        "left".to_string()
                                    };
                                if actual_target_language(state) == "auto" {
                                    state.translation_direction = "right".to_string();
                                }
                                trigger_translation_refresh(hwnd, state);
                            }
                            ResultButtonKind::Copy => {
                                let _ = clipboard_win::set_clipboard_string(&state.content);
                                invalidate_result_buttons(hwnd, state, Some(pressed), released);
                            }
                            ResultButtonKind::Pin => {
                                state.pinned = !state.pinned;
                                SetWindowPos(
                                    hwnd,
                                    if state.pinned {
                                        HWND_TOPMOST
                                    } else {
                                        HWND_NOTOPMOST
                                    },
                                    0,
                                    0,
                                    0,
                                    0,
                                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                                );
                                invalidate_result_buttons(hwnd, state, Some(pressed), released);
                            }
                            ResultButtonKind::Close => {
                                DestroyWindow(hwnd);
                            }
                        }
                    } else {
                        invalidate_result_buttons(hwnd, state, Some(pressed), released);
                    }
                    return 0;
                }
                0
            }
            WM_MOUSEWHEEL => {
                let delta = wheel_delta(wparam);
                state.scroll = (state.scroll - delta / 4).clamp(0, 1200);
                InvalidateRect(hwnd, null_mut(), 0);
                0
            }
            WM_KEYDOWN => {
                if wparam == ESC_KEY {
                    DestroyWindow(hwnd);
                    0
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            WM_KILLFOCUS => {
                state.pressed_button = None;
                ReleaseCapture();
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_DESTROY => {
                state.pressed_button = None;
                ReleaseCapture();
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }

    unsafe extern "system" fn selection_window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SelectionPopupState;
        if state_ptr.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let state = &mut *state_ptr;

        match message {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_selection(hwnd, state);
                0
            }
            WM_MOUSEMOVE => {
                let point = point_from_lparam(lparam);
                let next_hover = hit_button(&state.buttons, point);
                if next_hover != state.hovered_button {
                    let previous = state.hovered_button;
                    state.hovered_button = next_hover;
                    invalidate_selection_buttons(hwnd, state, previous, next_hover);
                }
                0
            }
            WM_LBUTTONDOWN => {
                let point = point_from_lparam(lparam);
                if let Some(kind) = hit_button(&state.buttons, point) {
                    state.hovered_button = Some(kind);
                    state.pressed_button = Some(kind);
                    SetCapture(hwnd);
                    InvalidateRect(hwnd, null_mut(), 0);
                    return 0;
                }

                DestroyWindow(hwnd);
                0
            }
            WM_LBUTTONUP => {
                if let Some(pressed) = state.pressed_button.take() {
                    ReleaseCapture();
                    let point = point_from_lparam(lparam);
                    let released = hit_button(&state.buttons, point);
                    state.hovered_button = released;
                    if released == Some(pressed) {
                        state.result = match pressed {
                            SelectionButtonKind::Copy => Some(SelectionPopupAction::Copy),
                            SelectionButtonKind::Translate => Some(SelectionPopupAction::Translate),
                            SelectionButtonKind::Summarize => Some(SelectionPopupAction::Summarize),
                            SelectionButtonKind::Search => Some(SelectionPopupAction::Search),
                            SelectionButtonKind::Close => None,
                        };
                        DestroyWindow(hwnd);
                    } else {
                        InvalidateRect(hwnd, null_mut(), 0);
                    }
                    return 0;
                }
                0
            }
            WM_KEYDOWN => {
                if wparam == ESC_KEY {
                    DestroyWindow(hwnd);
                    0
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            WM_KILLFOCUS => {
                state.restore_focus = false;
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }

    unsafe extern "system" fn tray_menu_window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayMenuState;
        if state_ptr.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let state = &mut *state_ptr;

        match message {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_tray_menu(hwnd, state);
                0
            }
            WM_MOUSEMOVE => {
                let point = point_from_lparam(lparam);
                let next_hover = hit_button(&state.buttons, point);
                if next_hover != state.hovered_button {
                    state.hovered_button = next_hover;
                    InvalidateRect(hwnd, null_mut(), 0);
                }
                0
            }
            WM_LBUTTONDOWN => {
                let point = point_from_lparam(lparam);
                if let Some(kind) = hit_button(&state.buttons, point) {
                    state.hovered_button = Some(kind);
                    state.pressed_button = Some(kind);
                    SetCapture(hwnd);
                    InvalidateRect(hwnd, null_mut(), 0);
                } else {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_LBUTTONUP => {
                if let Some(pressed) = state.pressed_button.take() {
                    ReleaseCapture();
                    let point = point_from_lparam(lparam);
                    let released = hit_button(&state.buttons, point);
                    state.hovered_button = released;
                    if released == Some(pressed) {
                        state.result = Some(match pressed {
                            TrayMenuButtonKind::Settings => TrayMenuAction::Settings,
                            TrayMenuButtonKind::ToggleAutostart => TrayMenuAction::ToggleAutostart,
                            TrayMenuButtonKind::Quit => TrayMenuAction::Quit,
                        });
                        DestroyWindow(hwnd);
                    } else {
                        InvalidateRect(hwnd, null_mut(), 0);
                    }
                    return 0;
                }
                0
            }
            WM_KEYDOWN => {
                if wparam == ESC_KEY {
                    DestroyWindow(hwnd);
                    0
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            WM_KILLFOCUS => {
                state.pressed_button = None;
                ReleaseCapture();
                0
            }
            WM_DESTROY => {
                state.pressed_button = None;
                ReleaseCapture();
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }

    fn create_popup_window(
        class_name: &str,
        title: &str,
        proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        state_ptr: isize,
        activate: bool,
    ) -> Result<HWND, String> {
        let class_name = to_wide(class_name);
        let title = to_wide(title);
        let instance = unsafe { GetModuleHandleW(null_mut()) };
        let cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
        let window_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(proc),
            hInstance: instance,
            hCursor: cursor,
            lpszClassName: class_name.as_ptr(),
            ..unsafe { zeroed() }
        };

        unsafe {
            RegisterClassW(&window_class);
        }

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_POPUP,
                x,
                y,
                width,
                height,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            )
        };

        if hwnd.is_null() {
            return Err("创建原生弹窗失败".to_string());
        }

        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr);
            apply_round_window_region(hwnd, width, height, 22);
            show_popup_window(hwnd, activate);
        }

        Ok(hwnd)
    }

    unsafe fn show_popup_window(hwnd: HWND, activate: bool) {
        if activate {
            ShowWindow(hwnd, SW_SHOW);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
            UpdateWindow(hwnd);
            let _ = SetForegroundWindow(hwnd);
            SetFocus(hwnd);
        } else {
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            SetWindowPos(
                hwnd,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            UpdateWindow(hwnd);
        }
    }

    fn message_loop(_hwnd: HWND) {
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
    }

    fn result_message_loop(
        hwnd: HWND,
        state: &mut ResultPopupState,
        receiver: Receiver<ResultPopupUpdate>,
    ) {
        let mut message: MSG = unsafe { zeroed() };
        loop {
            while unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if message.message == WM_QUIT {
                    return;
                }
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            let mut changed = false;
            while let Ok(update) = receiver.try_recv() {
                let was_loading = state.loading();
                state.title = update.title;
                state.content = update.content;
                state.detail = update.detail;
                state.status = update.status;
                state.source_language = update.source_language;
                state.target_language = update.target_language;
                state.translation_direction = update.translation_direction;
                state.can_switch_language = update.can_switch_language;
                state.open_language_menu = None;
                state.hovered_language_option = None;
                state.language_menu_options.clear();
                state.scroll = 0;
                if was_loading && !state.loading() {
                    state.reveal_frame = Some(0);
                }
                changed = true;
            }

            let mut animated = false;
            if state.loading() {
                state.animation_tick = state.animation_tick.wrapping_add(1);
                animated = true;
            }
            if let Some(frame) = state.reveal_frame {
                if frame < REVEAL_FRAMES {
                    state.reveal_frame = Some(frame + 1);
                    animated = true;
                } else {
                    state.reveal_frame = None;
                }
            }

            if changed {
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                    UpdateWindow(hwnd);
                }
            } else if animated {
                invalidate_result_content(hwnd, state);
                unsafe {
                    UpdateWindow(hwnd);
                }
            }

            if unsafe { IsWindow(hwnd) } == 0 {
                return;
            }

            thread::sleep(Duration::from_millis(30));
        }
    }

    fn restore_previous_foreground(hwnd: HWND) {
        if hwnd.is_null() {
            return;
        }

        unsafe {
            if IsWindow(hwnd) != 0 {
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }

    fn invalidate_result_buttons(
        hwnd: HWND,
        state: &ResultPopupState,
        first: Option<ResultButtonKind>,
        second: Option<ResultButtonKind>,
    ) {
        let mut invalidated = false;
        for kind in [first, second].into_iter().flatten() {
            if let Some(button) = state.buttons.iter().find(|button| button.kind == kind) {
                let rect = button.rect;
                unsafe {
                    InvalidateRect(hwnd, &rect, 0);
                }
                invalidated = true;
            }
        }

        if !invalidated {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
    }

    fn invalidate_result_content(hwnd: HWND, state: &ResultPopupState) {
        if rect_has_area(state.content_rect) {
            let rect = state.content_rect;
            unsafe {
                InvalidateRect(hwnd, &rect, 0);
            }
        } else {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
    }

    fn paint_result(hwnd: HWND, state: &mut ResultPopupState) {
        let mut paint: PAINTSTRUCT = unsafe { zeroed() };
        let window_dc = unsafe { BeginPaint(hwnd, &mut paint) };
        let mut client = empty_rect();
        unsafe {
            GetClientRect(hwnd, &mut client);
        }
        let width = client.right - client.left;
        let height = client.bottom - client.top;
        if width <= 0 || height <= 0 {
            unsafe {
                EndPaint(hwnd, &paint);
            }
            return;
        }

        let buffer_dc = unsafe { CreateCompatibleDC(window_dc) };
        let buffer_bitmap = unsafe { CreateCompatibleBitmap(window_dc, width, height) };
        if buffer_dc.is_null() || buffer_bitmap.is_null() {
            if !buffer_bitmap.is_null() {
                unsafe {
                    DeleteObject(buffer_bitmap as _);
                }
            }
            if !buffer_dc.is_null() {
                unsafe {
                    DeleteDC(buffer_dc);
                }
            }
            paint_result_body(window_dc, state, client);
            unsafe {
                EndPaint(hwnd, &paint);
            }
            return;
        }

        let old_bitmap = unsafe { SelectObject(buffer_dc, buffer_bitmap as _) };
        paint_result_body(buffer_dc, state, client);
        unsafe {
            BitBlt(window_dc, 0, 0, width, height, buffer_dc, 0, 0, SRCCOPY);
            if !old_bitmap.is_null() {
                SelectObject(buffer_dc, old_bitmap);
            }
            DeleteObject(buffer_bitmap as _);
            DeleteDC(buffer_dc);
            EndPaint(hwnd, &paint);
        }
    }

    fn paint_result_body(hdc: HDC, state: &mut ResultPopupState, client: RECT) {
        let palette = popup_palette(state.theme);
        let margin = popup_margin(client.right - client.left);
        fill_rect(hdc, client, palette.window_bg);
        frame_rect(hdc, client, palette.window_border);

        if !state.can_switch_language {
            let header = RECT {
                left: margin,
                top: 10,
                right: client.right - margin,
                bottom: 48,
            };
            draw_text_strong(
                hdc,
                &state.title,
                header,
                palette.text,
                DT_SINGLELINE | DT_VCENTER,
            );
        }

        let buttons_top = client.bottom - 46;
        state.buttons = result_buttons(client, buttons_top, state.can_switch_language);
        state.content_rect = RECT {
            left: margin,
            top: 58,
            right: client.right - margin,
            bottom: buttons_top - 12,
        };

        paint_soft_rect(
            hdc,
            state.content_rect,
            14,
            palette.panel_bg,
            palette.panel_border,
        );
        if state.loading() {
            paint_skeleton(hdc, state.content_rect, state.animation_tick, palette);
        } else {
            paint_result_content(hdc, state, palette);
        }

        if !state.detail.trim().is_empty() {
            let detail_rect = RECT {
                left: margin,
                top: buttons_top - 26,
                right: client.right - margin,
                bottom: buttons_top - 6,
            };
            draw_text(
                hdc,
                &state.detail,
                detail_rect,
                palette.text_secondary,
                DT_SINGLELINE | DT_END_ELLIPSIS | DT_VCENTER,
            );
        }

        for button in &state.buttons {
            let visual_state = if state.pressed_button == Some(button.kind) {
                ButtonVisualState::Pressed
            } else if state.hovered_button == Some(button.kind) {
                ButtonVisualState::Hovered
            } else {
                ButtonVisualState::Default
            };

            match button.kind {
                ResultButtonKind::Pin | ResultButtonKind::Close => paint_result_icon_button(
                    hdc,
                    button.rect,
                    button.kind,
                    button.kind == ResultButtonKind::Pin && state.pinned,
                    visual_state,
                    palette,
                ),
                ResultButtonKind::SourceLanguage
                | ResultButtonKind::Direction
                | ResultButtonKind::TargetLanguage
                | ResultButtonKind::Copy => {
                    let label = match button.kind {
                        ResultButtonKind::SourceLanguage => {
                            language_selector_label(&state.source_language)
                        }
                        ResultButtonKind::Direction => {
                            direction_button_label(&state.translation_direction)
                        }
                        ResultButtonKind::TargetLanguage => {
                            language_selector_label(&state.target_language)
                        }
                        ResultButtonKind::Copy => "复制".to_string(),
                        ResultButtonKind::Pin | ResultButtonKind::Close => unreachable!(),
                    };
                    paint_button(hdc, button.rect, &label, false, visual_state, palette);
                }
            }
        }

        let buttons_snapshot = state.buttons.clone();
        let mut menu_options = Vec::new();
        paint_language_menu::<ResultButtonKind>(
            hdc,
            state,
            &buttons_snapshot,
            &mut menu_options,
            client,
            palette,
        );
        state.language_menu_options = menu_options;
    }

    fn paint_selection(hwnd: HWND, state: &mut SelectionPopupState) {
        let mut paint: PAINTSTRUCT = unsafe { zeroed() };
        let window_dc = unsafe { BeginPaint(hwnd, &mut paint) };
        let mut client = empty_rect();
        unsafe {
            GetClientRect(hwnd, &mut client);
        }
        let width = client.right - client.left;
        let height = client.bottom - client.top;
        let buffer_dc = unsafe { CreateCompatibleDC(window_dc) };
        let buffer_bitmap = unsafe { CreateCompatibleBitmap(window_dc, width, height) };
        let buffered = !buffer_dc.is_null() && !buffer_bitmap.is_null();
        if !buffered {
            if !buffer_bitmap.is_null() {
                unsafe {
                    DeleteObject(buffer_bitmap as _);
                }
            }
            if !buffer_dc.is_null() {
                unsafe {
                    DeleteDC(buffer_dc);
                }
            }
        }
        let old_bitmap = if buffered {
            unsafe { SelectObject(buffer_dc, buffer_bitmap as _) }
        } else {
            null_mut()
        };
        let hdc = if buffered { buffer_dc } else { window_dc };

        let palette = popup_palette(state.theme);
        fill_rect(hdc, client, palette.window_bg);
        paint_soft_rect(hdc, client, 22, palette.panel_bg, palette.panel_border);

        state.buttons = selection_buttons(client, state.snapshot.char_count > 0);
        for button in &state.buttons {
            let label = match button.kind {
                SelectionButtonKind::Copy => "复制",
                SelectionButtonKind::Translate => "翻译",
                SelectionButtonKind::Summarize => "总结",
                SelectionButtonKind::Search => "搜索",
                SelectionButtonKind::Close => "关闭",
            };
            let visual_state = if state.pressed_button == Some(button.kind) {
                ButtonVisualState::Pressed
            } else if state.hovered_button == Some(button.kind) {
                ButtonVisualState::Hovered
            } else {
                ButtonVisualState::Default
            };
            paint_button(hdc, button.rect, label, false, visual_state, palette);
        }

        unsafe {
            if buffered {
                BitBlt(window_dc, 0, 0, width, height, buffer_dc, 0, 0, SRCCOPY);
                if !old_bitmap.is_null() {
                    SelectObject(buffer_dc, old_bitmap);
                }
                DeleteObject(buffer_bitmap as _);
                DeleteDC(buffer_dc);
            }
            EndPaint(hwnd, &paint);
        }
    }

    fn invalidate_selection_buttons(
        hwnd: HWND,
        state: &SelectionPopupState,
        first: Option<SelectionButtonKind>,
        second: Option<SelectionButtonKind>,
    ) {
        for kind in [first, second].into_iter().flatten() {
            if let Some(button) = state.buttons.iter().find(|button| button.kind == kind) {
                let rect = button.rect;
                unsafe {
                    InvalidateRect(hwnd, &rect, 0);
                }
            }
        }
    }

    fn paint_tray_menu(hwnd: HWND, state: &mut TrayMenuState) {
        let mut paint: PAINTSTRUCT = unsafe { zeroed() };
        let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
        let mut client = empty_rect();
        unsafe {
            GetClientRect(hwnd, &mut client);
        }

        let palette = popup_palette(state.theme);
        fill_rect(hdc, client, palette.window_bg);
        paint_soft_rect(hdc, client, 16, palette.panel_bg, palette.panel_border);

        state.buttons = tray_menu_buttons(client);
        for button in &state.buttons {
            let visual_state = if state.pressed_button == Some(button.kind) {
                ButtonVisualState::Pressed
            } else if state.hovered_button == Some(button.kind) {
                ButtonVisualState::Hovered
            } else {
                ButtonVisualState::Default
            };
            let label = match button.kind {
                TrayMenuButtonKind::Settings => "设置".to_string(),
                TrayMenuButtonKind::ToggleAutostart if state.autostart_enabled => {
                    "✓ 自启动".to_string()
                }
                TrayMenuButtonKind::ToggleAutostart => "自启动".to_string(),
                TrayMenuButtonKind::Quit => "退出".to_string(),
            };
            paint_tray_menu_item(hdc, button.rect, &label, visual_state, palette);
        }

        unsafe {
            EndPaint(hwnd, &paint);
        }
    }

    fn result_buttons(
        client: RECT,
        top: i32,
        can_switch_language: bool,
    ) -> Vec<Button<ResultButtonKind>> {
        let width = client.right - client.left;
        let margin = popup_margin(width);
        let copy_width = 64;
        let copy_height = 38;
        let gap = 4;
        let mut buttons = Vec::new();

        buttons.push(Button {
            kind: ResultButtonKind::Copy,
            rect: RECT {
                left: client.right - margin - copy_width,
                top,
                right: client.right - margin,
                bottom: top + copy_height,
            },
        });

        let icon_size = 30;
        let icon_top = 8;
        let mut icon_right = client.right - margin;
        for kind in [ResultButtonKind::Close, ResultButtonKind::Pin] {
            buttons.push(Button {
                kind,
                rect: RECT {
                    left: icon_right - icon_size,
                    top: icon_top,
                    right: icon_right,
                    bottom: icon_top + icon_size,
                },
            });
            icon_right -= icon_size + gap;
        }

        if can_switch_language {
            let source_width = 64;
            let direction_width = 48;
            let target_width = 64;
            let top = 8;
            let height = 34;
            let total_width = source_width + direction_width + target_width + gap * 2;
            let left = ((client.right - client.left - total_width) / 2).max(margin);
            buttons.push(Button {
                kind: ResultButtonKind::SourceLanguage,
                rect: RECT {
                    left,
                    top,
                    right: left + source_width,
                    bottom: top + height,
                },
            });
            buttons.push(Button {
                kind: ResultButtonKind::Direction,
                rect: RECT {
                    left: left + source_width + gap,
                    top,
                    right: left + source_width + gap + direction_width,
                    bottom: top + height,
                },
            });
            buttons.push(Button {
                kind: ResultButtonKind::TargetLanguage,
                rect: RECT {
                    left: left + source_width + gap + direction_width + gap,
                    top,
                    right: left + source_width + gap + direction_width + gap + target_width,
                    bottom: top + height,
                },
            });
        }

        buttons
    }

    fn selection_buttons(client: RECT, _can_use: bool) -> Vec<Button<SelectionButtonKind>> {
        let width = 68;
        let height = 34;
        let gap = 8;
        let top = (client.bottom - height) / 2;
        let total_width = width * 5 + gap * 4;
        let mut left = client.right - total_width - 12;
        let mut buttons = Vec::new();
        for kind in [
            SelectionButtonKind::Copy,
            SelectionButtonKind::Translate,
            SelectionButtonKind::Summarize,
            SelectionButtonKind::Search,
            SelectionButtonKind::Close,
        ] {
            buttons.push(Button {
                kind,
                rect: RECT {
                    left,
                    top,
                    right: left + width,
                    bottom: top + height,
                },
            });
            left += width + gap;
        }

        buttons
    }

    fn tray_menu_buttons(client: RECT) -> Vec<Button<TrayMenuButtonKind>> {
        let item_height = 32;
        let gap = 4;
        let mut top = client.top + 10;
        let left = client.left + 8;
        let right = client.right - 8;
        let mut buttons = Vec::new();
        for kind in [
            TrayMenuButtonKind::Settings,
            TrayMenuButtonKind::ToggleAutostart,
            TrayMenuButtonKind::Quit,
        ] {
            buttons.push(Button {
                kind,
                rect: RECT {
                    left,
                    top,
                    right,
                    bottom: top + item_height,
                },
            });
            top += item_height + gap;
        }
        buttons
    }

    fn language_selector_label(code: &str) -> String {
        format!("{} ▾", compact_language_label(code))
    }

    fn direction_button_label(direction: &str) -> String {
        if direction == "left" {
            "←".to_string()
        } else {
            "→".to_string()
        }
    }

    trait LanguageMenuAnchor: Copy + PartialEq {
        fn from_language_menu(kind: LanguageMenuKind) -> Self;
    }

    impl LanguageMenuAnchor for ResultButtonKind {
        fn from_language_menu(kind: LanguageMenuKind) -> Self {
            match kind {
                LanguageMenuKind::Source => ResultButtonKind::SourceLanguage,
                LanguageMenuKind::Target => ResultButtonKind::TargetLanguage,
            }
        }
    }

    impl LanguageMenuAnchor for InputButtonKind {
        fn from_language_menu(kind: LanguageMenuKind) -> Self {
            match kind {
                LanguageMenuKind::Source => InputButtonKind::SourceLanguage,
                LanguageMenuKind::Target => InputButtonKind::TargetLanguage,
            }
        }
    }

    fn toggle_language_menu(
        hwnd: HWND,
        state: &mut impl LanguageMenuState,
        kind: LanguageMenuKind,
    ) {
        state.set_hovered_language_option(None);
        state.set_open_language_menu(if state.open_language_menu() == Some(kind) {
            None
        } else {
            Some(kind)
        });
        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }
    }

    fn select_language_option(state: &mut impl LanguageMenuState, index: usize) -> bool {
        let Some(kind) = state.open_language_menu() else {
            return false;
        };
        let options = language_menu_options(state.language_menu_allows_auto(kind));
        let Some((code, _)) = options.get(index) else {
            return false;
        };

        let changed = match kind {
            LanguageMenuKind::Source if state.source_language() != *code => {
                state.set_source_language((*code).to_string());
                true
            }
            LanguageMenuKind::Target if state.target_language() != *code => {
                state.set_target_language((*code).to_string());
                true
            }
            _ => false,
        };

        if actual_target_language(state) == "auto" {
            state.set_translation_direction(match kind {
                LanguageMenuKind::Source => "right".to_string(),
                LanguageMenuKind::Target => "left".to_string(),
            });
        }

        state.set_open_language_menu(None);
        state.set_hovered_language_option(None);
        changed
    }

    fn actual_target_language(state: &impl LanguageMenuState) -> String {
        if state.translation_direction() == "left" {
            state.source_language().to_string()
        } else {
            state.target_language().to_string()
        }
    }

    fn paint_language_menu<K: LanguageMenuAnchor>(
        hdc: HDC,
        state: &mut impl LanguageMenuState,
        buttons: &[Button<K>],
        options_out: &mut Vec<Button<usize>>,
        client: RECT,
        palette: PopupPalette,
    ) {
        options_out.clear();
        let Some(kind) = state.open_language_menu() else {
            return;
        };

        let anchor_kind = K::from_language_menu(kind);
        let Some(anchor) = buttons
            .iter()
            .find(|button| button.kind == anchor_kind)
            .map(|button| button.rect)
        else {
            return;
        };

        let options = language_menu_options(state.language_menu_allows_auto(kind));
        let row_height = 30;
        let width = if compact_popup(client.right - client.left) {
            96
        } else {
            132
        };
        let height = row_height * options.len() as i32 + 8;
        let margin = popup_margin(client.right - client.left);
        let mut left = ((anchor.left + anchor.right - width) / 2).max(margin);
        if left + width > client.right - margin {
            left = client.right - margin - width;
        }
        let top = anchor.bottom + 7;
        let menu_rect = RECT {
            left,
            top,
            right: left + width,
            bottom: top + height,
        };

        paint_soft_rect(hdc, menu_rect, 12, palette.panel_bg, palette.panel_border);

        let current = match kind {
            LanguageMenuKind::Source => state.source_language(),
            LanguageMenuKind::Target => state.target_language(),
        };
        for (index, (code, label)) in options.iter().enumerate() {
            let option_rect = RECT {
                left: menu_rect.left + 5,
                top: menu_rect.top + 4 + index as i32 * row_height,
                right: menu_rect.right - 5,
                bottom: menu_rect.top + 4 + (index as i32 + 1) * row_height,
            };
            options_out.push(Button {
                kind: index,
                rect: option_rect,
            });

            let selected = *code == current;
            let hovered = state.hovered_language_option() == Some(index);
            if selected || hovered {
                paint_soft_rect(
                    hdc,
                    option_rect,
                    9,
                    if selected {
                        palette.button_pressed
                    } else {
                        palette.button_hover
                    },
                    if selected {
                        palette.button_border
                    } else {
                        palette.panel_border
                    },
                );
            }

            let text_rect = RECT {
                left: option_rect.left + 12,
                top: option_rect.top,
                right: option_rect.right - 10,
                bottom: option_rect.bottom,
            };
            let label = if selected {
                format!("✓ {label}")
            } else {
                (*label).to_string()
            };
            draw_text(
                hdc,
                &label,
                text_rect,
                palette.text,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
        }
    }

    fn language_menu_options(include_auto: bool) -> Vec<(&'static str, &'static str)> {
        let mut options = Vec::new();
        if include_auto {
            options.push(("auto", "自动检测"));
        }
        options.extend([
            ("zh-Hans", "简体中文"),
            ("en", "英文"),
            ("ja", "日文"),
            ("ko", "韩文"),
            ("fr", "法文"),
            ("de", "德文"),
            ("es", "西班牙文"),
        ]);
        options
    }

    fn compact_language_label(code: &str) -> &'static str {
        match code {
            "auto" => "自动",
            "en" => "英文",
            "ja" => "日文",
            "ko" => "韩文",
            "fr" => "法文",
            "de" => "德文",
            "es" => "西语",
            _ => "中文",
        }
    }

    fn opposite_language(code: &str) -> String {
        match code {
            "zh-Hans" => "en",
            "auto" => "zh-Hans",
            _ => "zh-Hans",
        }
        .to_string()
    }

    fn trigger_translation_refresh(hwnd: HWND, state: &mut ResultPopupState) {
        state.status = "loading".to_string();
        state.content.clear();
        state.detail.clear();
        state.scroll = 0;
        state.reveal_frame = None;
        if let Some(sender) = &state.event_sender {
            let _ = sender.send(ResultPopupEvent::TranslationChanged {
                source_language: state.source_language.clone(),
                target_language: state.target_language.clone(),
                direction: state.translation_direction.clone(),
            });
        }
        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }
    }

    fn popup_palette(theme: NativeTheme) -> PopupPalette {
        if theme.is_dark() {
            PopupPalette {
                window_bg: rgb(8, 10, 13),
                window_border: rgb(45, 53, 64),
                panel_bg: rgb(18, 22, 28),
                panel_border: rgb(54, 63, 76),
                text: rgb(232, 237, 244),
                text_secondary: rgb(148, 158, 172),
                button_bg: rgb(30, 36, 45),
                button_hover: rgb(38, 47, 58),
                button_pressed: rgb(24, 29, 37),
                button_border: rgb(65, 77, 92),
                primary_bg: rgb(38, 91, 116),
                primary_hover: rgb(45, 108, 136),
                primary_pressed: rgb(30, 73, 96),
                primary_border: rgb(77, 135, 160),
                primary_text: rgb(236, 249, 255),
                skeleton_base: rgb(30, 37, 46),
                skeleton_shine: rgb(48, 60, 73),
            }
        } else {
            PopupPalette {
                window_bg: rgb(246, 250, 255),
                window_border: rgb(199, 214, 236),
                panel_bg: rgb(252, 254, 255),
                panel_border: rgb(214, 226, 244),
                text: rgb(23, 34, 53),
                text_secondary: rgb(88, 102, 123),
                button_bg: rgb(238, 244, 255),
                button_hover: rgb(225, 236, 255),
                button_pressed: rgb(213, 230, 255),
                button_border: rgb(188, 211, 249),
                primary_bg: rgb(47, 111, 236),
                primary_hover: rgb(38, 97, 220),
                primary_pressed: rgb(36, 94, 203),
                primary_border: rgb(118, 164, 247),
                primary_text: rgb(255, 255, 255),
                skeleton_base: rgb(229, 238, 252),
                skeleton_shine: rgb(247, 251, 255),
            }
        }
    }

    fn paint_button(
        hdc: windows_sys::Win32::Graphics::Gdi::HDC,
        rect: RECT,
        label: &str,
        primary: bool,
        visual_state: ButtonVisualState,
        palette: PopupPalette,
    ) {
        let rect = match visual_state {
            ButtonVisualState::Pressed => inset_rect(rect, 1, 1),
            ButtonVisualState::Default | ButtonVisualState::Hovered => rect,
        };
        let background = match (primary, visual_state) {
            (true, ButtonVisualState::Pressed) => palette.primary_pressed,
            (true, ButtonVisualState::Hovered) => palette.primary_hover,
            (true, ButtonVisualState::Default) => palette.primary_bg,
            (false, ButtonVisualState::Pressed) => palette.button_pressed,
            (false, ButtonVisualState::Hovered) => palette.button_hover,
            (false, ButtonVisualState::Default) => palette.button_bg,
        };
        let text = if primary {
            palette.primary_text
        } else {
            palette.text
        };
        let border = if primary {
            palette.primary_border
        } else {
            palette.button_border
        };
        paint_soft_rect(hdc, rect, 12, background, border);
        draw_text_strong(
            hdc,
            label,
            rect,
            text,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER,
        );
    }

    fn paint_tray_menu_item(
        hdc: windows_sys::Win32::Graphics::Gdi::HDC,
        rect: RECT,
        label: &str,
        visual_state: ButtonVisualState,
        palette: PopupPalette,
    ) {
        let rect = match visual_state {
            ButtonVisualState::Pressed => inset_rect(rect, 1, 1),
            ButtonVisualState::Default | ButtonVisualState::Hovered => rect,
        };
        let background = match visual_state {
            ButtonVisualState::Pressed => palette.button_pressed,
            ButtonVisualState::Hovered => palette.button_hover,
            ButtonVisualState::Default => palette.button_bg,
        };
        let border = match visual_state {
            ButtonVisualState::Default => palette.panel_border,
            ButtonVisualState::Hovered | ButtonVisualState::Pressed => palette.button_border,
        };
        paint_soft_rect(hdc, rect, 10, background, border);
        let text_rect = RECT {
            left: rect.left + 14,
            top: rect.top,
            right: rect.right - 10,
            bottom: rect.bottom,
        };
        draw_text_strong(
            hdc,
            label,
            text_rect,
            palette.text,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
    }

    fn paint_result_icon_button(
        hdc: windows_sys::Win32::Graphics::Gdi::HDC,
        rect: RECT,
        kind: ResultButtonKind,
        active: bool,
        visual_state: ButtonVisualState,
        palette: PopupPalette,
    ) {
        let rect = match visual_state {
            ButtonVisualState::Pressed => inset_rect(rect, 1, 1),
            ButtonVisualState::Default | ButtonVisualState::Hovered => rect,
        };
        let background = match (active, visual_state) {
            (true, ButtonVisualState::Pressed) => palette.primary_pressed,
            (true, ButtonVisualState::Hovered) => palette.primary_hover,
            (true, ButtonVisualState::Default) => palette.primary_bg,
            (false, ButtonVisualState::Pressed) => palette.button_pressed,
            (false, ButtonVisualState::Hovered) => palette.button_hover,
            (false, ButtonVisualState::Default) => palette.button_bg,
        };
        let border = if active {
            palette.primary_border
        } else {
            palette.button_border
        };
        let icon = if active {
            palette.primary_text
        } else {
            palette.text
        };
        paint_soft_rect(hdc, rect, 12, background, border);
        paint_result_icon(hdc, rect, kind, icon);
    }

    fn paint_result_icon(
        hdc: windows_sys::Win32::Graphics::Gdi::HDC,
        rect: RECT,
        kind: ResultButtonKind,
        color: COLORREF,
    ) {
        let center_x = (rect.left + rect.right) / 2;
        let center_y = (rect.top + rect.bottom) / 2;
        unsafe {
            let pen = CreatePen(PS_SOLID, 2, color);
            let old_pen = if pen.is_null() {
                null_mut()
            } else {
                SelectObject(hdc, pen as _)
            };

            match kind {
                ResultButtonKind::Close => {
                    MoveToEx(hdc, center_x - 5, center_y - 5, null_mut());
                    LineTo(hdc, center_x + 6, center_y + 6);
                    MoveToEx(hdc, center_x + 5, center_y - 5, null_mut());
                    LineTo(hdc, center_x - 6, center_y + 6);
                }
                ResultButtonKind::Pin => {
                    MoveToEx(hdc, center_x - 5, center_y - 8, null_mut());
                    LineTo(hdc, center_x + 5, center_y - 8);
                    LineTo(hdc, center_x + 4, center_y - 2);
                    LineTo(hdc, center_x + 7, center_y + 3);
                    LineTo(hdc, center_x - 7, center_y + 3);
                    LineTo(hdc, center_x - 4, center_y - 2);
                    LineTo(hdc, center_x - 5, center_y - 8);
                    MoveToEx(hdc, center_x, center_y + 3, null_mut());
                    LineTo(hdc, center_x, center_y + 10);
                }
                ResultButtonKind::SourceLanguage
                | ResultButtonKind::Direction
                | ResultButtonKind::TargetLanguage
                | ResultButtonKind::Copy => {}
            }

            if !old_pen.is_null() {
                SelectObject(hdc, old_pen);
            }
            if !pen.is_null() {
                DeleteObject(pen as _);
            }
        }
    }

    fn paint_result_content(hdc: HDC, state: &ResultPopupState, palette: PopupPalette) {
        let clip_rect = inset_rect(state.content_rect, 12, 10);
        let saved = unsafe { SaveDC(hdc) };
        unsafe {
            IntersectClipRect(
                hdc,
                clip_rect.left,
                clip_rect.top,
                clip_rect.right,
                clip_rect.bottom,
            );
        }

        let reveal_offset = state
            .reveal_frame
            .map(|frame| (REVEAL_FRAMES.saturating_sub(frame).min(REVEAL_FRAMES) as i32) * 2)
            .unwrap_or(0);
        let reveal_color = state
            .reveal_frame
            .map(|frame| {
                let progress = frame.min(REVEAL_FRAMES);
                blend_rgb(
                    palette.text_secondary,
                    palette.text,
                    progress,
                    REVEAL_FRAMES,
                )
            })
            .unwrap_or(palette.text);

        let mut content_rect = clip_rect;
        content_rect.top = clip_rect.top - state.scroll + reveal_offset;
        content_rect.bottom = content_rect.top + 2400;
        draw_text(
            hdc,
            &state.content,
            content_rect,
            reveal_color,
            DT_WORDBREAK | DT_NOPREFIX,
        );

        if saved != 0 {
            unsafe {
                RestoreDC(hdc, saved);
            }
        }
    }

    fn paint_skeleton(hdc: HDC, content_rect: RECT, tick: u32, palette: PopupPalette) {
        let area = inset_rect(content_rect, 16, 16);
        let available_width = (area.right - area.left).max(120);
        let rows = [
            (0, available_width - 54),
            (28, available_width - 18),
            (56, available_width - 82),
            (98, available_width - 40),
            (126, available_width - 112),
            (168, available_width - 68),
        ];

        for (index, (top_offset, width)) in rows.iter().enumerate() {
            let rect = RECT {
                left: area.left,
                top: area.top + *top_offset,
                right: area.left + (*width).max(96),
                bottom: area.top + *top_offset + 14,
            };
            paint_soft_rect(hdc, rect, 10, palette.skeleton_base, palette.skeleton_base);

            let saved = unsafe { SaveDC(hdc) };
            unsafe {
                IntersectClipRect(hdc, rect.left, rect.top, rect.right, rect.bottom);
            }
            let sweep_width = 72;
            let span = (rect.right - rect.left + sweep_width).max(sweep_width);
            let sweep_left =
                rect.left - sweep_width + ((tick as i32 * 13 + index as i32 * 27) % span);
            let shine = RECT {
                left: sweep_left,
                top: rect.top,
                right: sweep_left + sweep_width,
                bottom: rect.bottom,
            };
            paint_soft_rect(
                hdc,
                shine,
                10,
                palette.skeleton_shine,
                palette.skeleton_shine,
            );
            if saved != 0 {
                unsafe {
                    RestoreDC(hdc, saved);
                }
            }
        }
    }

    fn draw_text(hdc: HDC, text: &str, rect: RECT, color: COLORREF, format: u32) {
        draw_text_with_font(hdc, text, rect, color, format, UI_FONT_WEIGHT_NORMAL);
    }

    fn draw_text_strong(hdc: HDC, text: &str, rect: RECT, color: COLORREF, format: u32) {
        draw_text_with_font(hdc, text, rect, color, format, UI_FONT_WEIGHT_STRONG);
    }

    fn draw_text_with_font(
        hdc: HDC,
        text: &str,
        mut rect: RECT,
        color: COLORREF,
        format: u32,
        weight: i32,
    ) {
        let wide = to_wide(text);
        let face = to_wide(UI_FONT_FACE);
        unsafe {
            let font = CreateFontW(
                -UI_FONT_SIZE,
                0,
                0,
                0,
                weight,
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
            DrawTextW(hdc, wide.as_ptr(), -1, &mut rect, format);
            if !old_font.is_null() {
                SelectObject(hdc, old_font);
            }
            if !font.is_null() {
                DeleteObject(font as _);
            }
        }
    }

    fn measure_text_extent(hdc: HDC, text: &str, rect: RECT, format: u32) -> RECT {
        let mut measured = rect;
        let wide = to_wide(text);
        let face = to_wide(UI_FONT_FACE);
        unsafe {
            let font = CreateFontW(
                -UI_FONT_SIZE,
                0,
                0,
                0,
                UI_FONT_WEIGHT_NORMAL,
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
            DrawTextW(hdc, wide.as_ptr(), -1, &mut measured, DT_CALCRECT | format);
            if !old_font.is_null() {
                SelectObject(hdc, old_font);
            }
            if !font.is_null() {
                DeleteObject(font as _);
            }
        }
        measured
    }

    fn fill_rect(hdc: windows_sys::Win32::Graphics::Gdi::HDC, rect: RECT, color: COLORREF) {
        unsafe {
            let brush = CreateSolidBrush(color);
            FillRect(hdc, &rect, brush);
            DeleteObject(brush);
        }
    }

    fn frame_rect(hdc: windows_sys::Win32::Graphics::Gdi::HDC, rect: RECT, color: COLORREF) {
        unsafe {
            let brush = CreateSolidBrush(color);
            FrameRect(hdc, &rect, brush);
            DeleteObject(brush);
        }
    }

    fn paint_soft_rect(
        hdc: windows_sys::Win32::Graphics::Gdi::HDC,
        rect: RECT,
        radius: i32,
        fill: COLORREF,
        border: COLORREF,
    ) {
        unsafe {
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
    }

    fn apply_round_window_region(hwnd: HWND, width: i32, height: i32, radius: i32) {
        unsafe {
            let region = CreateRoundRectRgn(0, 0, width + 1, height + 1, radius, radius);
            if !region.is_null() && SetWindowRgn(hwnd, region, 1) == 0 {
                DeleteObject(region as _);
            }
        }
    }

    fn inset_rect(rect: RECT, x: i32, y: i32) -> RECT {
        RECT {
            left: rect.left + x,
            top: rect.top + y,
            right: rect.right - x,
            bottom: rect.bottom - y,
        }
    }

    fn empty_rect() -> RECT {
        RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        }
    }

    fn rect_has_area(rect: RECT) -> bool {
        rect.right > rect.left && rect.bottom > rect.top
    }

    fn point_in_rect(rect: RECT, point: (i32, i32)) -> bool {
        point.0 >= rect.left
            && point.0 <= rect.right
            && point.1 >= rect.top
            && point.1 <= rect.bottom
    }

    fn hit_button<T: Copy>(buttons: &[Button<T>], point: (i32, i32)) -> Option<T> {
        buttons
            .iter()
            .find(|button| point_in_rect(button.rect, point))
            .map(|button| button.kind)
    }

    fn drag_popup_window(
        hwnd: HWND,
        state: &mut impl DragState,
        _point: (i32, i32),
    ) {
        let origin = state.drag_origin();
        let cursor = cursor_screen_pos();
        let dx = cursor.0 - origin.0;
        let dy = cursor.1 - origin.1;
        if dx != 0 || dy != 0 {
            let rect = state.drag_window_rect();
            let x = rect.left + dx;
            let y = rect.top + dy;
            unsafe {
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    x,
                    y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }

    fn cursor_screen_pos() -> (i32, i32) {
        let mut cursor = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        unsafe {
            GetCursorPos(&mut cursor);
        }
        (cursor.x, cursor.y)
    }

    trait DragState {
        fn drag_origin(&self) -> (i32, i32);
        fn drag_window_rect(&self) -> RECT;
    }

    fn point_from_lparam(lparam: LPARAM) -> (i32, i32) {
        let x = (lparam & 0xffff) as i16 as i32;
        let y = ((lparam >> 16) & 0xffff) as i16 as i32;
        (x, y)
    }

    fn wheel_delta(wparam: WPARAM) -> i32 {
        ((wparam >> 16) & 0xffff) as i16 as i32
    }

    fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
        red as COLORREF | ((green as COLORREF) << 8) | ((blue as COLORREF) << 16)
    }

    fn blend_rgb(start: COLORREF, end: COLORREF, step: u32, total: u32) -> COLORREF {
        let total = total.max(1);
        let step = step.min(total);
        let channel = |shift: u32| {
            let start = ((start >> shift) & 0xff) as i32;
            let end = ((end >> shift) & 0xff) as i32;
            (start + (end - start) * step as i32 / total as i32) as u8
        };
        rgb(channel(0), channel(8), channel(16))
    }

    fn to_wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    impl InputPopupState {
        fn new(
            input_text: String,
            source_language: String,
            target_language: String,
            translation_direction: String,
            content: String,
            detail: String,
            status: String,
            theme: NativeTheme,
            event_sender: Option<Sender<InputPopupEvent>>,
        ) -> Self {
            Self {
                input_text,
                content,
                detail,
                status,
                source_language,
                target_language,
                translation_direction,
                theme,
                event_sender,
                pinned: false,
                scroll: 0,
                animation_tick: 0,
                reveal_frame: None,
                hovered_button: None,
                pressed_button: None,
                buttons: Vec::new(),
                dragging: false,
                drag_origin: (0, 0),
                drag_window_rect: empty_rect(),
                open_language_menu: None,
                hovered_language_option: None,
                language_menu_options: Vec::new(),
                input_focused: true,
                edit_rect: empty_rect(),
                content_rect: empty_rect(),
                result_rect: empty_rect(),
            }
        }

        fn loading(&self) -> bool {
            self.status == "loading"
        }
    }

    pub fn run_input_updatable(
        input_text: String,
        source_language: String,
        target_language: String,
        translation_direction: String,
        content: String,
        detail: String,
        status: String,
        theme: NativeTheme,
        receiver: Receiver<InputPopupUpdate>,
        event_sender: Sender<InputPopupEvent>,
    ) -> Result<(), String> {
        let width = NATIVE_POPUP_WIDTH;
        let height = 460;
        let (x, y) = clamp_window_position(width, height);
        let mut state = InputPopupState::new(
            input_text,
            source_language,
            target_language,
            translation_direction,
            content,
            detail,
            status,
            theme,
            Some(event_sender),
        );
        preset_input_layout(&mut state, width, height);
        let hwnd = match create_popup_window(
            INPUT_CLASS_NAME,
            INPUT_WINDOW_TITLE,
            input_window_proc,
            x,
            y,
            width,
            height,
            &mut state as *mut InputPopupState as isize,
            true,
        ) {
            Ok(hwnd) => hwnd,
            Err(error) => {
                eprintln!("QuickPick input popup create failed: {error}");
                return Err(error);
            }
        };
        unsafe {
            SetForegroundWindow(hwnd);
            SetFocus(hwnd);
        }
        input_message_loop(hwnd, &mut state, receiver);
        Ok(())
    }

    unsafe extern "system" fn input_window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCDESTROY {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut InputPopupState;
        if state_ptr.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let state = &mut *state_ptr;

        match message {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_input(hwnd, state);
                0
            }
            WM_SIZE => {
                layout_input_state(hwnd, state);
                0
            }
            WM_MOUSEMOVE => {
                let point = point_from_lparam(lparam);
                if state.dragging {
                    drag_popup_window(hwnd, state, point);
                }
                let previous_language_hover = state.hovered_language_option;
                if state.open_language_menu.is_some() {
                    state.hovered_language_option = hit_button(&state.language_menu_options, point);
                    if state.hovered_language_option != previous_language_hover {
                        invalidate_input_language_options(
                            hwnd,
                            state,
                            previous_language_hover,
                            state.hovered_language_option,
                        );
                    }
                }
                let previous_hover = state.hovered_button;
                let next_hover = hit_button(&state.buttons, point);
                if next_hover != previous_hover {
                    state.hovered_button = next_hover;
                    invalidate_input_buttons(hwnd, state, previous_hover, next_hover);
                }
                0
            }
            WM_LBUTTONDOWN => {
                let point = point_from_lparam(lparam);
                if point.1 <= 46 && hit_button(&state.buttons, point).is_none() {
                    state.dragging = true;
                    state.drag_origin = cursor_screen_pos();
                    unsafe {
                        GetWindowRect(hwnd, &mut state.drag_window_rect);
                    }
                    SetCapture(hwnd);
                    return 0;
                }
                if state.open_language_menu.is_some() {
                    if let Some(index) = hit_button(&state.language_menu_options, point) {
                        let changed = select_language_option(state, index);
                        if changed {
                            trigger_input_translation(hwnd, state);
                        } else {
                            state.open_language_menu = None;
                            state.hovered_language_option = None;
                            unsafe {
                                InvalidateRect(hwnd, null_mut(), 0);
                            }
                        }
                        return 0;
                    }

                    if hit_button(&state.buttons, point).is_none() {
                        state.open_language_menu = None;
                        state.hovered_language_option = None;
                        unsafe {
                            InvalidateRect(hwnd, null_mut(), 0);
                        }
                        return 0;
                    }
                }

                if let Some(kind) = hit_button(&state.buttons, point) {
                    state.hovered_button = Some(kind);
                    state.pressed_button = Some(kind);
                    SetCapture(hwnd);
                    invalidate_input_buttons(hwnd, state, Some(kind), Some(kind));
                    return 0;
                }

                if point_in_rect(state.edit_rect, point) {
                    state.input_focused = true;
                    unsafe {
                        SetFocus(hwnd);
                    }
                    invalidate_input_edit(hwnd, state);
                    return 0;
                }

                0
            }
            WM_LBUTTONUP => {
                if state.dragging {
                    state.dragging = false;
                    ReleaseCapture();
                }
                if let Some(pressed) = state.pressed_button.take() {
                    ReleaseCapture();
                    let point = point_from_lparam(lparam);
                    let released = hit_button(&state.buttons, point);
                    state.hovered_button = released;
                    if released == Some(pressed) {
                        match pressed {
                            InputButtonKind::SourceLanguage => {
                                toggle_language_menu(hwnd, state, LanguageMenuKind::Source);
                            }
                            InputButtonKind::TargetLanguage => {
                                toggle_language_menu(hwnd, state, LanguageMenuKind::Target);
                            }
                            InputButtonKind::Direction => {
                                state.open_language_menu = None;
                                state.hovered_language_option = None;
                                state.translation_direction =
                                    if state.translation_direction == "left" {
                                        "right".to_string()
                                    } else {
                                        if state.source_language == "auto" {
                                            state.source_language =
                                                opposite_language(&state.target_language);
                                        }
                                        "left".to_string()
                                    };
                                if actual_target_language(state) == "auto" {
                                    state.translation_direction = "right".to_string();
                                }
                                trigger_input_translation(hwnd, state);
                            }
                            InputButtonKind::Translate => {
                                trigger_input_translation(hwnd, state);
                            }
                            InputButtonKind::Copy => {
                                let _ = clipboard_win::set_clipboard_string(&state.content);
                                invalidate_input_buttons(hwnd, state, Some(pressed), released);
                            }
                            InputButtonKind::Pin => {
                                state.pinned = !state.pinned;
                                if state.pinned {
                                    unsafe {
                                        SetWindowPos(
                                            hwnd,
                                            HWND_TOPMOST,
                                            0,
                                            0,
                                            0,
                                            0,
                                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                                        );
                                    }
                                } else {
                                    unsafe {
                                        SetWindowPos(
                                            hwnd,
                                            HWND_NOTOPMOST,
                                            0,
                                            0,
                                            0,
                                            0,
                                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                                        );
                                    }
                                }
                                invalidate_input_buttons(hwnd, state, Some(pressed), released);
                            }
                            InputButtonKind::Close => {
                                DestroyWindow(hwnd);
                            }
                        }
                    } else {
                        invalidate_input_buttons(hwnd, state, Some(pressed), released);
                    }
                    return 0;
                }
                0
            }
            WM_MOUSEWHEEL => {
                let delta = wheel_delta(wparam);
                if rect_has_area(state.result_rect)
                    && point_in_rect(state.result_rect, point_from_lparam(lparam))
                {
                    state.scroll = (state.scroll - delta / 8).clamp(0, 1200);
                    unsafe {
                        InvalidateRect(hwnd, &state.result_rect, 0);
                    }
                }
                0
            }
            WM_KEYDOWN => {
                if wparam == ESC_KEY {
                    DestroyWindow(hwnd);
                    0
                } else if state.input_focused {
                    handle_input_keydown(hwnd, state, wparam, lparam);
                    0
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            WM_CHAR => {
                if state.input_focused {
                    let code_unit = wparam as u32;
                    if let Some(ch) = char::from_u32(code_unit) {
                        if ch as u32 >= 32 || ch == '\r' || ch == '\n' {
                            state.input_text.push(ch);
                            invalidate_input_edit(hwnd, state);
                        }
                    } else if code_unit == 0 {
                        state.input_text.push('\0');
                        invalidate_input_edit(hwnd, state);
                    }
                    0
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            WM_DESTROY => {
                state.pressed_button = None;
                ReleaseCapture();
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }

    fn input_message_loop(
        hwnd: HWND,
        state: &mut InputPopupState,
        receiver: Receiver<InputPopupUpdate>,
    ) {
        let mut message: MSG = unsafe { zeroed() };
        loop {
            while unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if message.message == WM_QUIT {
                    return;
                }
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            let mut changed = false;
            while let Ok(update) = receiver.try_recv() {
                let was_loading = state.loading();
                state.content = update.content;
                state.detail = update.detail;
                state.status = update.status;
                state.source_language = update.source_language;
                state.target_language = update.target_language;
                state.translation_direction = update.translation_direction;
                state.open_language_menu = None;
                state.hovered_language_option = None;
                state.language_menu_options.clear();
                state.scroll = 0;
                if was_loading && !state.loading() {
                    state.reveal_frame = Some(0);
                }
                changed = true;
            }

            let mut animated = false;
            if state.loading() {
                state.animation_tick = state.animation_tick.wrapping_add(1);
                animated = true;
            }
            if state.input_focused && !state.loading() {
                state.animation_tick = state.animation_tick.wrapping_add(1);
                animated = true;
            }
            if let Some(frame) = state.reveal_frame {
                if frame < REVEAL_FRAMES {
                    state.reveal_frame = Some(frame + 1);
                    animated = true;
                } else {
                    state.reveal_frame = None;
                }
            }

            if changed {
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                    UpdateWindow(hwnd);
                }
            } else if animated {
                let rect = if state.input_focused && !state.loading() {
                    state.edit_rect
                } else {
                    state.result_rect
                };
                if rect_has_area(rect) {
                    unsafe {
                        InvalidateRect(hwnd, &rect, 0);
                        UpdateWindow(hwnd);
                    }
                }
            }

            if unsafe { IsWindow(hwnd) } == 0 {
                return;
            }

            thread::sleep(Duration::from_millis(30));
        }
    }

    fn layout_input_state(hwnd: HWND, state: &mut InputPopupState) {
        let mut client = empty_rect();
        unsafe {
            GetClientRect(hwnd, &mut client);
        }
        let right = client.right.max(client.left + 1);
        let bottom = client.bottom.max(client.top + 1);
        let margin = popup_margin(right - client.left);

        state.edit_rect = RECT {
            left: margin,
            top: 56,
            right: right - margin,
            bottom: 128,
        };
        state.result_rect = RECT {
            left: margin,
            top: 148,
            right: right - margin,
            bottom: bottom - 50,
        };
        state.buttons = input_buttons(client);
    }

    fn preset_input_layout(state: &mut InputPopupState, width: i32, height: i32) {
        let margin = popup_margin(width);
        state.edit_rect = RECT {
            left: margin,
            top: 56,
            right: width - margin,
            bottom: 128,
        };
        state.result_rect = RECT {
            left: margin,
            top: 148,
            right: width - margin,
            bottom: height - 50,
        };
    }

    fn input_buttons(client: RECT) -> Vec<Button<InputButtonKind>> {
        let mut buttons = Vec::new();
        let right = client.right.max(client.left + 1);
        let bottom = client.bottom.max(client.top + 1);
        let width = right - client.left;
        let margin = popup_margin(width);
        let action_width = 64;
        let action_height = 38;
        let action_gap = 8;

        let icon_size = 30;
        let icon_top = 8;
        let mut icon_right = right - margin;
        for kind in [InputButtonKind::Close, InputButtonKind::Pin] {
            buttons.push(Button {
                kind,
                rect: RECT {
                    left: icon_right - icon_size,
                    top: icon_top,
                    right: icon_right,
                    bottom: icon_top + icon_size,
                },
            });
            icon_right -= icon_size + 8;
        }

        let source_width = 64;
        let direction_width = 48;
        let target_width = 64;
        let top = 8;
        let height = 34;
        let gap = 4;
        let total_width = source_width + direction_width + target_width + gap * 2;
        let left = ((right - total_width) / 2).max(margin);
        buttons.push(Button {
            kind: InputButtonKind::SourceLanguage,
            rect: RECT {
                left,
                top,
                right: left + source_width,
                bottom: top + height,
            },
        });
        buttons.push(Button {
            kind: InputButtonKind::Direction,
            rect: RECT {
                left: left + source_width + gap,
                top,
                right: left + source_width + gap + direction_width,
                bottom: top + height,
            },
        });
        buttons.push(Button {
            kind: InputButtonKind::TargetLanguage,
            rect: RECT {
                left: left + source_width + gap + direction_width + gap,
                top,
                right: left + source_width + gap + direction_width + gap + target_width,
                bottom: top + height,
            },
        });

        buttons.push(Button {
            kind: InputButtonKind::Copy,
            rect: RECT {
                left: right - margin - action_width,
                top: bottom - 8 - action_height,
                right: right - margin,
                bottom: bottom - 8,
            },
        });
        buttons.push(Button {
            kind: InputButtonKind::Translate,
            rect: RECT {
                left: right - margin - action_width - action_gap - action_width,
                top: bottom - 8 - action_height,
                right: right - margin - action_width - action_gap,
                bottom: bottom - 8,
            },
        });

        buttons
    }

    fn paint_input(hwnd: HWND, state: &mut InputPopupState) {
        let mut paint: PAINTSTRUCT = unsafe { zeroed() };
        let window_dc = unsafe { BeginPaint(hwnd, &mut paint) };
        let mut client = empty_rect();
        unsafe {
            GetClientRect(hwnd, &mut client);
        }
        let width = client.right - client.left;
        let height = client.bottom - client.top;
        let buffer_dc = unsafe { CreateCompatibleDC(window_dc) };
        let buffer_bitmap = unsafe { CreateCompatibleBitmap(window_dc, width, height) };
        let buffered = !buffer_dc.is_null() && !buffer_bitmap.is_null();
        if !buffered {
            if !buffer_bitmap.is_null() {
                unsafe {
                    DeleteObject(buffer_bitmap as _);
                }
            }
            if !buffer_dc.is_null() {
                unsafe {
                    DeleteDC(buffer_dc);
                }
            }
        }
        let old_bitmap = if buffered {
            unsafe { SelectObject(buffer_dc, buffer_bitmap as _) }
        } else {
            null_mut()
        };
        let hdc = if buffered { buffer_dc } else { window_dc };
        let palette = popup_palette(state.theme);
        fill_rect(hdc, client, palette.window_bg);
        frame_rect(hdc, client, palette.window_border);

        layout_input_state(hwnd, state);
        let caret_visible = state.input_focused && (state.animation_tick / 20) % 2 == 0;
        paint_input_edit_panel(
            hdc,
            state.edit_rect,
            &state.input_text,
            state.input_focused,
            caret_visible,
            palette,
        );
        paint_soft_rect(hdc, state.result_rect, 14, palette.panel_bg, palette.panel_border);
        if state.loading() {
            paint_skeleton(hdc, state.result_rect, state.animation_tick, palette);
        } else {
            let saved = unsafe { SaveDC(hdc) };
            let clip = inset_rect(state.result_rect, 12, 10);
            unsafe {
                IntersectClipRect(hdc, clip.left, clip.top, clip.right, clip.bottom);
            }
            let mut content_rect = clip;
            content_rect.top = clip.top - state.scroll
                + state
                    .reveal_frame
                    .map(|frame| (REVEAL_FRAMES.saturating_sub(frame).min(REVEAL_FRAMES) as i32) * 2)
                    .unwrap_or(0);
            content_rect.bottom = content_rect.top + 2400;
            draw_text(
                hdc,
                &state.content,
                content_rect,
                state
                    .reveal_frame
                    .map(|frame| {
                        let progress = frame.min(REVEAL_FRAMES);
                        blend_rgb(palette.text_secondary, palette.text, progress, REVEAL_FRAMES)
                    })
                    .unwrap_or(palette.text),
                DT_WORDBREAK | DT_NOPREFIX,
            );
            if saved != 0 {
                unsafe {
                    RestoreDC(hdc, saved);
                }
            }
        }

        for button in &state.buttons {
            let visual_state = if state.pressed_button == Some(button.kind) {
                ButtonVisualState::Pressed
            } else if state.hovered_button == Some(button.kind) {
                ButtonVisualState::Hovered
            } else {
                ButtonVisualState::Default
            };
            match button.kind {
                InputButtonKind::Pin | InputButtonKind::Close => {
                    paint_result_icon_button(
                        hdc,
                        button.rect,
                        if button.kind == InputButtonKind::Pin {
                            ResultButtonKind::Pin
                        } else {
                            ResultButtonKind::Close
                        },
                        button.kind == InputButtonKind::Pin && state.pinned,
                        visual_state,
                        palette,
                    );
                }
                InputButtonKind::SourceLanguage
                | InputButtonKind::Direction
                | InputButtonKind::TargetLanguage
                | InputButtonKind::Translate
                | InputButtonKind::Copy => {
                    let label: String = match button.kind {
                        InputButtonKind::SourceLanguage => {
                            language_selector_label(&state.source_language)
                        }
                        InputButtonKind::Direction => {
                            direction_button_label(&state.translation_direction)
                        }
                        InputButtonKind::TargetLanguage => {
                            language_selector_label(&state.target_language)
                        }
                        InputButtonKind::Translate => "翻译".to_string(),
                        InputButtonKind::Copy => "复制".to_string(),
                        _ => String::new(),
                    };
                    paint_button(
                        hdc,
                        button.rect,
                        &label,
                        button.kind == InputButtonKind::Translate
                            || button.kind == InputButtonKind::Copy,
                        visual_state,
                        palette,
                    );
                }
            }
        }

        let buttons_snapshot = state.buttons.clone();
        let mut menu_options = Vec::new();
        paint_language_menu::<InputButtonKind>(
            hdc,
            state,
            &buttons_snapshot,
            &mut menu_options,
            client,
            palette,
        );
        state.language_menu_options = menu_options;
        unsafe {
            if buffered {
                BitBlt(window_dc, 0, 0, width, height, buffer_dc, 0, 0, SRCCOPY);
                if !old_bitmap.is_null() {
                    SelectObject(buffer_dc, old_bitmap);
                }
                DeleteObject(buffer_bitmap as _);
                DeleteDC(buffer_dc);
            }
            EndPaint(hwnd, &paint);
        }
    }

    fn paint_input_edit_panel(
        hdc: HDC,
        rect: RECT,
        text: &str,
        focused: bool,
        caret_visible: bool,
        palette: PopupPalette,
    ) {
        let panel = RECT {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        };
        paint_soft_rect(hdc, panel, 14, palette.panel_bg, palette.panel_border);
        let text_rect = inset_rect(panel, 12, 10);
        let display_text = if text.is_empty() && !focused {
            "请输入要翻译的文本"
        } else {
            text
        };
        draw_text(
            hdc,
            display_text,
            text_rect,
            if text.is_empty() {
                palette.text_secondary
            } else {
                palette.text
            },
            DT_WORDBREAK | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        if focused && caret_visible {
            let caret = if text.is_empty() {
                RECT {
                    left: text_rect.left,
                    top: text_rect.top + 2,
                    right: text_rect.left + 2,
                    bottom: text_rect.top + UI_FONT_SIZE + 5,
                }
            } else {
                let measured = measure_text_extent(
                    hdc,
                    text,
                    text_rect,
                    DT_WORDBREAK | DT_NOPREFIX | DT_END_ELLIPSIS,
                );
                let x = measured.right.clamp(text_rect.left, text_rect.right - 2);
                let top = (measured.bottom - UI_FONT_SIZE - 2).max(text_rect.top);
                RECT {
                    left: x,
                    top,
                    right: x + 2,
                    bottom: top + UI_FONT_SIZE + 5,
                }
            };
            fill_rect(hdc, caret, palette.text);
        }
    }

    fn invalidate_input_buttons(
        hwnd: HWND,
        state: &InputPopupState,
        first: Option<InputButtonKind>,
        second: Option<InputButtonKind>,
    ) {
        for kind in [first, second].into_iter().flatten() {
            if let Some(button) = state.buttons.iter().find(|button| button.kind == kind) {
                let rect = button.rect;
                unsafe {
                    InvalidateRect(hwnd, &rect, 0);
                }
            }
        }
    }

    fn invalidate_input_language_options(
        hwnd: HWND,
        state: &InputPopupState,
        first: Option<usize>,
        second: Option<usize>,
    ) {
        for index in [first, second].into_iter().flatten() {
            if let Some(option) = state.language_menu_options.get(index) {
                let rect = option.rect;
                unsafe {
                    InvalidateRect(hwnd, &rect, 0);
                }
            }
        }
    }

    fn trigger_input_translation(hwnd: HWND, state: &mut InputPopupState) {
        let input_text = state.input_text.clone();
        if let Some(sender) = &state.event_sender {
            let _ = sender.send(InputPopupEvent::TranslateRequested {
                input_text,
                source_language: state.source_language.clone(),
                target_language: state.target_language.clone(),
                direction: state.translation_direction.clone(),
            });
        }
        state.status = "loading".to_string();
        state.content.clear();
        state.detail.clear();
        state.scroll = 0;
        state.reveal_frame = None;
        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }
    }

    fn handle_input_keydown(
        hwnd: HWND,
        state: &mut InputPopupState,
        wparam: WPARAM,
        lparam: LPARAM,
    ) {
        const VK_BACK: usize = 0x08;
        const VK_RETURN: usize = 0x0d;
        const VK_DELETE: usize = 0x2e;
        const VK_V: usize = 0x56;
        let ctrl_pressed = (lparam & (1 << 29)) != 0;

        if wparam == VK_BACK {
            state.input_text.pop();
        } else if wparam == VK_DELETE {
            state.input_text.pop();
        } else if wparam == VK_RETURN {
            state.input_text.push('\n');
        } else if wparam == VK_V && ctrl_pressed {
            if let Some(text) = read_clipboard_text() {
                state.input_text.push_str(&text);
            }
        }
        invalidate_input_edit(hwnd, state);
    }

    fn invalidate_input_edit(hwnd: HWND, state: &InputPopupState) {
        if rect_has_area(state.edit_rect) {
            let rect = state.edit_rect;
            unsafe {
                InvalidateRect(hwnd, &rect, 0);
            }
        } else {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
    }

    fn read_clipboard_text() -> Option<String> {
        clipboard_win::get_clipboard_string().ok()
    }

}

#[cfg(not(windows))]
mod native_window {
    use super::*;

    pub fn run_selection(
        _snapshot: SelectionSnapshot,
        _theme: NativeTheme,
    ) -> Result<Option<SelectionPopupAction>, String> {
        Ok(None)
    }

    pub fn run_result(
        _title: String,
        _content: String,
        _detail: String,
        _status: String,
        _theme: NativeTheme,
    ) -> Result<(), String> {
        Ok(())
    }

    pub fn run_result_updatable(
        _title: String,
        _content: String,
        _detail: String,
        _status: String,
        _theme: NativeTheme,
        _receiver: Receiver<ResultPopupUpdate>,
    ) -> Result<(), String> {
        Ok(())
    }

    pub fn run_input_updatable(
        _input_text: String,
        _source_language: String,
        _target_language: String,
        _translation_direction: String,
        _content: String,
        _detail: String,
        _status: String,
        _theme: NativeTheme,
        _receiver: Receiver<InputPopupUpdate>,
        _event_sender: Sender<InputPopupEvent>,
    ) -> Result<(), String> {
        Ok(())
    }
}
