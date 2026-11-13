use crate::native_probe::{NativeEvent, NativeProbe};
use gpui_kit::{
    component::{
        Disableable,
        button::Button,
        input::{Input, InputState, Textarea, TextareaState},
    },
    prelude::*,
    *,
};
use quickpick_core::region::{MonitorPx, PointDip, RectDip};
use quickpick_platform::probe as platform;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::time::{Duration, Instant};

pub fn run() {
    // 禁用 DirectComposition 的后端忽略 alpha，不能用作实时透明遮罩。
    if std::env::var("GPUI_DISABLE_DIRECT_COMPOSITION")
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    {
        eprintln!("实时遮罩验证需要 DirectComposition；当前后端配置不支持此验证。");
        return;
    }
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(520.0), px(620.0)),
                cx,
            ))),
            window_min_size: Some(size(px(420.0), px(600.0))),
            titlebar: Some(TitlebarOptions {
                title: Some("QuickPick · GPUI 可行性验证".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let opened = gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| Probe::new(window, cx))
        });
        match opened {
            Ok((handle, _)) => {
                let _ = handle.update(cx, |_, window, cx| {
                    gpui_kit::base::Root::update(window, cx, |root, _, cx| {
                        root.style().background = Some(rgba(0xf4f5f7cc).into());
                        cx.notify();
                    });
                });
            }
            Err(_) => {
                eprintln!("GPUI 窗口初始化失败；请检查显卡驱动及 Windows 桌面会话。");
                cx.quit();
            }
        }
    });
}

fn native_handle(window: &Window) -> Option<isize> {
    match HasWindowHandle::window_handle(window).ok()?.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
        _ => None,
    }
}

struct Probe {
    native: Option<isize>,
    input: Entity<InputState>,
    multiline: Entity<TextareaState>,
    status: String,
    overlay_open: bool,
    native_probe: Option<NativeProbe>,
    native_attempted: bool,
}

impl Probe {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            native: native_handle(window),
            input: cx.new(|cx| {
                InputState::new(window, cx).placeholder("中文 IME、选区、粘贴、撤销与 Tab")
            }),
            multiline: cx.new(|cx| {
                TextareaState::new(window, cx).placeholder("多行输入验证；内容只保留在本窗口内存中")
            }),
            status: "此窗口是阶段 0 样机，未替代正式 QuickPick。".into(),
            overlay_open: false,
            native_probe: None,
            native_attempted: false,
        }
    }

    fn enable_native_probe(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.native_attempted {
            return;
        }
        self.native_attempted = true;
        let (sender, receiver) = async_channel::bounded(32);
        match NativeProbe::new(sender) {
            Ok(probe) => {
                self.native_probe = Some(probe);
                self.status = "已启用托盘及 Alt+Shift+F12 验证。退出程序时释放测试热键。".into();
                cx.spawn(async move |view, cx| {
                    while let Ok(event) = receiver.recv().await {
                        if view
                            .update(cx, |view, cx| {
                                if matches!(event, NativeEvent::Quit) {
                                    if view.overlay_open {
                                        view.status = "请先结束当前遮罩/复制验证，再退出。".into();
                                    } else {
                                        cx.quit();
                                    }
                                } else {
                                    view.status = match event {
                                        NativeEvent::Hotkey => {
                                            "GPUI 已收到 Alt+Shift+F12 热键事件。"
                                        }
                                        NativeEvent::Tray => "GPUI 已收到托盘双击事件。",
                                        _ => "GPUI 已收到原生菜单事件。",
                                    }
                                    .into();
                                    if !view.overlay_open
                                        && let Some(native) = view.native
                                    {
                                        platform::show_probe(native);
                                    }
                                }
                                cx.notify();
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                })
                .detach();
            }
            Err(error) => self.status = format!("{error}；解决后请重启验证程序重试。"),
        }
        cx.notify();
    }

    fn open_overlay(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.overlay_open {
            return;
        }
        let monitor = match platform::monitor_at_cursor() {
            Ok(monitor) => monitor,
            Err(error) => {
                self.status = error.into();
                cx.notify();
                return;
            }
        };
        let owner = cx.weak_entity();
        if let Some(native) = self.native {
            platform::hide_overlay(native);
        }
        // 先创建隐藏透明窗口，定位到完整物理显示器后才显示。
        let options = WindowOptions {
            titlebar: None,
            show: false,
            focus: true,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };
        let result = gpui_kit::open_window(options, cx, move |window, cx| {
            cx.new(|cx| Overlay::new(monitor, owner, window, cx))
        });
        match result {
            Ok((handle, overlay)) => {
                let configured = handle.update(cx, |_, window, cx| {
                    // Component 的 Root 默认绘制不透明背景；必须显式覆盖，
                    // 否则即使 swapchain 支持 alpha，选区也看不到实时桌面。
                    gpui_kit::base::Root::update(window, cx, |root, _, cx| {
                        root.style().background = Some(transparent_black().into());
                        cx.notify();
                    });
                    let native = native_handle(window).ok_or("无法取得 GPUI 原生窗口句柄")?;
                    platform::show_overlay(native, monitor)?;
                    overlay.update(cx, |view, cx| {
                        view.scale = window.scale_factor();
                        view.focus.focus(window, cx);
                        cx.notify();
                    });
                    Ok::<_, &'static str>(())
                });
                match configured {
                    Ok(Ok(())) => {
                        self.overlay_open = true;
                        self.status = "验证实时背景和透明区域鼠标输入；Esc 或右键取消。".into();
                    }
                    _ => {
                        let _ = handle.update(cx, |_, window, _| window.remove_window());
                        self.status = "实时遮罩初始化失败，未捕获。".into();
                    }
                }
            }
            Err(_) => self.status = "创建透明窗口失败，未捕获。".into(),
        }
        if !self.overlay_open
            && let Some(native) = self.native
        {
            platform::show_probe(native);
        }
        cx.notify();
    }
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_5()
            .flex()
            .flex_col()
            .gap_4()
            .text_color(rgb(0x20242b))
            .child("QuickPick · GPUI 阶段 0")
            .child("请仅输入测试内容。原生事件验证需单独启用；样机不读取旧配置、不修改开机自启动。")
            .child(Input::new(&self.input))
            .child(Textarea::new(&self.multiline).h(px(150.0)))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("opaque")
                            .label("普通背景")
                            .on_click(|_, window, _| {
                                window.set_background_appearance(WindowBackgroundAppearance::Opaque)
                            }),
                    )
                    .child(Button::new("mica").label("Mica").on_click(|_, window, _| {
                        window.set_background_appearance(WindowBackgroundAppearance::MicaBackdrop)
                    }))
                    .child(
                        Button::new("acrylic")
                            .label("Acrylic")
                            .on_click(|_, window, _| {
                                window
                                    .set_background_appearance(WindowBackgroundAppearance::Blurred)
                            }),
                    ),
            )
            .child(
                Button::new("overlay")
                    .label("验证实时遮罩")
                    .disabled(self.overlay_open)
                    .on_click(cx.listener(Self::open_overlay)),
            )
            .child(
                Button::new("native-probe")
                    .label("启用托盘／测试热键")
                    .disabled(self.native_attempted || self.overlay_open)
                    .on_click(cx.listener(Self::enable_native_probe)),
            )
            .child(self.status.clone())
    }
}

struct Overlay {
    monitor: MonitorPx,
    scale: f32,
    owner: WeakEntity<Probe>,
    start: Option<PointDip>,
    selection: Option<RectDip>,
    previous: Option<RectDip>,
    last_input: Instant,
    focus: FocusHandle,
    submitting: bool,
    _subscriptions: Vec<Subscription>,
}

impl Overlay {
    fn new(
        monitor: MonitorPx,
        owner: WeakEntity<Probe>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = vec![cx.observe_window_activation(window, |view, window, cx| {
            if !window.is_window_active() && !view.submitting {
                view.cancel(window, cx);
            }
        })];
        // 定时任务仅在此验证遮罩存在时运行；不做截图或持续重绘。
        cx.spawn_in(window, async move |view, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let keep = view
                    .update_in(cx, |view, window, cx| {
                        if view.submitting {
                            return false;
                        }
                        if view.last_input.elapsed() >= Duration::from_secs(120)
                            || !platform::monitor_unchanged(view.monitor)
                            || !platform::is_normal_desktop()
                            || window.scale_factor() != view.scale
                        {
                            view.cancel(window, cx);
                            return false;
                        }
                        if view.start.is_some()
                            && native_handle(window)
                                .is_none_or(|native| !platform::has_pointer_capture(native))
                        {
                            view.start = None;
                            view.selection = view.previous.take();
                            cx.notify();
                        }
                        true
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        })
        .detach();
        Self {
            monitor,
            scale: window.scale_factor(),
            owner,
            start: None,
            selection: None,
            previous: None,
            last_input: Instant::now(),
            focus: cx.focus_handle(),
            submitting: false,
            _subscriptions: subscriptions,
        }
    }

    fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.submitting {
            return;
        }
        self.start = None;
        self.selection = None;
        self.previous = None;
        if let Some(native) = native_handle(window) {
            platform::release_pointer(native);
        }
        let _ = self.owner.update(cx, |owner, cx| {
            owner.overlay_open = false;
            owner.status = "遮罩已取消，未捕获、未写剪贴板。".into();
            if let Some(native) = owner.native {
                platform::show_probe(native);
            }
            cx.notify();
        });
        window.remove_window();
    }

    fn copy(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.submitting || self.start.is_some() {
            return;
        }
        if window.scale_factor() != self.scale || !platform::monitor_unchanged(self.monitor) {
            self.cancel(window, cx);
            return;
        }
        let Some(region) = self
            .selection
            .and_then(|rect| rect.to_physical(self.monitor, f64::from(window.scale_factor())))
        else {
            return;
        };
        let Some(native) = native_handle(window) else {
            self.cancel(window, cx);
            return;
        };
        self.submitting = true;
        self.selection = None;
        self.previous = None;
        platform::release_pointer(native);
        platform::hide_overlay(native);
        let monitor = self.monitor;
        let task = cx
            .background_executor()
            .spawn(async move { platform::copy_after_hide(native, monitor, region) });
        cx.spawn_in(window, async move |view, cx| {
            let result = task.await;
            let _ = view.update_in(cx, |view, window, cx| {
                let _ = view.owner.update(cx, |owner, cx| {
                    owner.overlay_open = false;
                    owner.status = match result {
                        Ok(()) => {
                            "已复制选区。请粘贴到外部应用，检查像素边界和遮罩是否残留。".into()
                        }
                        Err(error) => error.into(),
                    };
                    if let Some(native) = owner.native {
                        platform::show_probe(native);
                    }
                    cx.notify();
                });
                window.remove_window();
            });
        })
        .detach();
    }

    fn move_pointer(&mut self, position: Point<Pixels>, window: &Window, cx: &mut Context<Self>) {
        self.last_input = Instant::now();
        if let Some(start) = self.start {
            let viewport = window.viewport_size();
            let end = PointDip {
                x: f64::from(f32::from(position.x).clamp(0.0, f32::from(viewport.width))),
                y: f64::from(f32::from(position.y).clamp(0.0, f32::from(viewport.height))),
            };
            self.selection = if (end.x - start.x).hypot(end.y - start.y) > 3.0 {
                RectDip::between(start, end)
            } else {
                None
            };
            cx.notify();
        }
    }

    fn end_drag(&mut self, event: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.move_pointer(event.position, window, cx);
        if self.start.take().is_some() {
            if self
                .selection
                .and_then(|rect| rect.to_physical(self.monitor, f64::from(self.scale)))
                .is_none()
            {
                self.selection = self.previous;
            }
            self.previous = None;
            cx.notify();
        }
    }
}

fn block(x: f32, y: f32, width: f32, height: f32) -> Div {
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(width.max(0.0)))
        .h(px(height.max(0.0)))
        .bg(rgba(0x00000059))
}

impl Render for Overlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size();
        let width = f32::from(viewport.width);
        let height = f32::from(viewport.height);
        let view = cx.weak_entity();
        // GPUI Windows 后端在按下时 SetCapture；不以根 hitbox 限制拖出屏幕后的移动事件。
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Bubble {
                let _ = view.update(cx, |view, cx| view.move_pointer(event.position, window, cx));
            }
        });
        let mut canvas = div()
            .id("overlay")
            .track_focus(&self.focus)
            .size_full()
            .relative()
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, window, cx| {
                view.last_input = Instant::now();
                if event.keystroke.key == "escape" {
                    view.cancel(window, cx);
                }
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|view, _, window, cx| view.cancel(window, cx)),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, event: &MouseDownEvent, _, cx| {
                    if view.submitting {
                        return;
                    }
                    view.last_input = Instant::now();
                    view.previous = view.selection;
                    view.start = Some(PointDip {
                        x: f64::from(f32::from(event.position.x)),
                        y: f64::from(f32::from(event.position.y)),
                    });
                    view.selection = None;
                    cx.notify();
                }),
            )
            .on_mouse_up(MouseButton::Left, cx.listener(Self::end_drag))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::end_drag));
        if let Some(rect) = self.selection {
            let left = rect.left as f32;
            let top = rect.top as f32;
            let right = rect.right as f32;
            let bottom = rect.bottom as f32;
            canvas = canvas
                .children([
                    block(0.0, 0.0, width, top),
                    block(0.0, bottom, width, height - bottom),
                    block(0.0, top, left, bottom - top),
                    block(right, top, width - right, bottom - top),
                ])
                .child(
                    div()
                        .absolute()
                        .left(px(left))
                        .top(px(top))
                        .w(px(right - left))
                        .h(px(bottom - top))
                        .border(px(1.0 / window.scale_factor()))
                        .border_color(rgb(0x3b82f6)),
                );
            if self.start.is_none() {
                let dimensions = rect
                    .to_physical(self.monitor, f64::from(window.scale_factor()))
                    .map(|region| format!("{} × {} px", region.width(), region.height()))
                    .unwrap_or_default();
                canvas =
                    canvas.child(
                        div()
                            .id("toolbar")
                            .absolute()
                            .left(px(left.clamp(0.0, (width - 320.0).max(0.0))))
                            .top(px(if bottom + 52.0 < height {
                                bottom + 8.0
                            } else {
                                (top - 48.0).max(0.0)
                            }))
                            .p_2()
                            .bg(rgb(0xf4f5f7))
                            .text_color(rgb(0x20242b))
                            .flex()
                            .gap_2()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(dimensions)
                            .child(
                                Button::new("copy")
                                    .label("复制选区")
                                    .on_click(cx.listener(Self::copy)),
                            )
                            .child(Button::new("cancel").label("取消").on_click(
                                cx.listener(|view, _, window, cx| view.cancel(window, cx)),
                            )),
                    );
            }
        } else {
            canvas = canvas.child(block(0.0, 0.0, width, height));
        }
        canvas
    }
}
