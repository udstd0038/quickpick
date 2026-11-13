use crate::{app::AppView, ui::*};
use quickpick_core::{
    i18n,
    region::{MonitorPx, PointDip},
    selection_model::{Handle, Hit, SelectionModel},
    session::ScreenshotAction,
};
use quickpick_platform::{probe, system};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
pub struct Overlay {
    owner: WeakEntity<AppView>,
    model: SelectionModel,
    monitor: MonitorPx,
    scale: f32,
    active: bool,
    pub activity: Arc<AtomicBool>,
    topology: Vec<[i64; 6]>,
    language: String,
    cancel: Arc<AtomicBool>,
    last_input: Instant,
    timer: Option<Task<()>>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
    cursor: CursorStyle,
}
fn point_dip(p: Point<Pixels>) -> PointDip {
    PointDip {
        x: f64::from(f32::from(p.x)),
        y: f64::from(f32::from(p.y)),
    }
}
impl Overlay {
    pub fn new(owner: WeakEntity<AppView>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe_window_activation(window, |view, window, cx| {
                if view.active && !window.is_window_active() {
                    view.cancel(window, cx);
                }
            }),
            cx.observe_window_bounds(window, |view, window, cx| {
                if view.active && window.scale_factor() != view.scale {
                    view.cancel(window, cx);
                }
            }),
        ];
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            let _ = weak.update(cx, |view, cx| view.cancel(window, cx));
            false
        });
        Self {
            owner,
            model: SelectionModel::new(1., 1.),
            monitor: MonitorPx {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            scale: 1.,
            active: false,
            activity: Arc::new(AtomicBool::new(false)),
            topology: vec![],
            language: "system".into(),
            cancel: Arc::new(AtomicBool::new(false)),
            last_input: Instant::now(),
            timer: None,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
            cursor: CursorStyle::Crosshair,
        }
    }
    pub fn show(
        &mut self,
        monitor: MonitorPx,
        language: String,
        cancel: Arc<AtomicBool>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.active = false;
        self.activity.store(false, Ordering::Release);
        self.monitor = monitor;
        self.language = language;
        window.set_window_title(&i18n::t(&self.language, "settings.hotkeyScreenshot"));
        self.cancel = cancel;
        if probe::show_overlay(native(window), monitor).is_err() {
            self.invalidate(window, cx);
            return false;
        }
        self.scale = window.scale_factor();
        self.model = SelectionModel::new(
            f64::from(monitor.width) / f64::from(self.scale),
            f64::from(monitor.height) / f64::from(self.scale),
        );
        self.topology = match system::topology() {
            Ok(v) => v,
            Err(_) => {
                self.invalidate(window, cx);
                return false;
            }
        };
        self.active = true;
        self.activity.store(true, Ordering::Release);
        self.last_input = Instant::now();
        self.focus.focus(window, cx);
        self.timer = Some(cx.spawn_in(window, async move |view, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let keep = view
                    .update_in(cx, |view, window, cx| {
                        if !view.active {
                            return false;
                        }
                        if view.last_input.elapsed() >= Duration::from_secs(120)
                            || view.cancel.load(Ordering::Acquire)
                            || !probe::monitor_unchanged(view.monitor)
                            || !probe::is_normal_desktop()
                            || system::topology().map_or(true, |topology| topology != view.topology)
                            || view.scale != window.scale_factor()
                        {
                            view.cancel(window, cx);
                            return false;
                        }
                        if view.model.dragging() && !probe::has_pointer_capture(native(window)) {
                            view.model.lost_capture();
                            cx.notify();
                        }
                        true
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        }));
        cx.notify();
        true
    }
    pub fn invalidate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = false;
        self.activity.store(false, Ordering::Release);
        self.cancel.store(true, Ordering::Release);
        self.model.cancel();
        self.timer = None;
        probe::release_pointer(native(window));
        system::hide(native(window));
        cx.notify();
    }
    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.invalidate(window, cx);
        let _ = self.owner.update(cx, |view, _| view.screenshot_done());
    }
    fn submit(&mut self, action: ScreenshotAction, window: &mut Window, cx: &mut Context<Self>) {
        if !self.active
            || window.scale_factor() != self.scale
            || !probe::monitor_unchanged(self.monitor)
        {
            self.cancel(window, cx);
            return;
        }
        let Some(region) = self.model.submit(self.monitor, f64::from(self.scale)) else {
            return;
        };
        self.active = false;
        self.activity.store(false, Ordering::Release);
        self.model.cancel();
        self.timer = None;
        let handle = native(window);
        probe::release_pointer(handle);
        system::hide(handle);
        let _ = self.owner.update(cx, |owner, cx| {
            owner.capture(
                action,
                self.monitor,
                region,
                handle,
                self.topology.clone(),
                cx,
            )
        });
        cx.notify();
    }
    fn moved(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.active {
            return;
        }
        self.last_input = Instant::now();
        let p = point_dip(event.position);
        if self.model.dragging() {
            self.model.move_to(p);
            cx.notify();
        }
        let old_cursor = self.cursor;
        self.cursor = match self.model.hit(p) {
            Hit::Move => CursorStyle::OpenHand,
            Hit::Empty => CursorStyle::Crosshair,
            Hit::Handle(handle) => match handle {
                Handle::N | Handle::S => CursorStyle::ResizeUpDown,
                Handle::E | Handle::W => CursorStyle::ResizeLeftRight,
                Handle::Nw | Handle::Se => CursorStyle::ResizeUpLeftDownRight,
                Handle::Ne | Handle::Sw => CursorStyle::ResizeUpRightDownLeft,
            },
        };
        if old_cursor != self.cursor {
            cx.notify();
        }
    }
    fn up(&mut self, event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.active {
            self.model.up(
                point_dip(event.position),
                self.monitor,
                f64::from(self.scale),
            );
            self.last_input = Instant::now();
            cx.notify();
        }
    }
}
fn shade(x: f32, y: f32, w: f32, h: f32) -> Div {
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(w.max(0.)))
        .h(px(h.max(0.)))
        .bg(rgba(0x00000059))
}
impl Render for Overlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.weak_entity();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Bubble {
                let _ = view.update(cx, |view, cx| view.moved(event, window, cx));
            }
        });
        let bounds = window.viewport_size();
        let width = f32::from(bounds.width);
        let height = f32::from(bounds.height);
        let language = self.language.clone();
        let mut root = div()
            .id("capture")
            .track_focus(&self.focus)
            .relative()
            .size_full()
            .cursor(self.cursor)
            .capture_key_down(|event, _, cx| {
                if event.keystroke.key == "enter" {
                    cx.stop_propagation();
                }
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, event: &MouseDownEvent, _, cx| {
                    if view.active {
                        view.last_input = Instant::now();
                        view.model.down(point_dip(event.position));
                        cx.notify();
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|view, _, window, cx| view.cancel(window, cx)),
            )
            .on_mouse_up(MouseButton::Left, cx.listener(Self::up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::up))
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, window, cx| {
                view.last_input = Instant::now();
                if event.keystroke.key == "escape" {
                    view.cancel(window, cx);
                }
            }));
        let outline: Hsla = if cx.theme().mode == gpui_kit::component::ThemeMode::Dark {
            rgb(0xb9c5d4).into()
        } else {
            cx.theme().primary
        };
        if let Some(r) = self.model.selection {
            let left = (r.left as f32 * self.scale).floor() / self.scale;
            let top = (r.top as f32 * self.scale).floor() / self.scale;
            let right = (r.right as f32 * self.scale).ceil() / self.scale;
            let bottom = (r.bottom as f32 * self.scale).ceil() / self.scale;
            root = root
                .children([
                    shade(0., 0., width, top),
                    shade(0., bottom, width, height - bottom),
                    shade(0., top, left, bottom - top),
                    shade(right, top, width - right, bottom - top),
                ])
                .child(
                    div()
                        .absolute()
                        .left(px(left))
                        .top(px(top))
                        .w(px(right - left))
                        .h(px(bottom - top))
                        .border(px(1. / self.scale))
                        .border_color(outline),
                );
            for (_, p) in self.model.handles() {
                root = root.child(
                    div()
                        .absolute()
                        .left(px((p.x as f32 - 4.).clamp(0., (width - 8.).max(0.))))
                        .top(px((p.y as f32 - 4.).clamp(0., (height - 8.).max(0.))))
                        .w(px(8.))
                        .h(px(8.))
                        .bg(rgb(
                            if cx.theme().mode == gpui_kit::component::ThemeMode::Dark {
                                0x0b0e12
                            } else {
                                0xffffff
                            },
                        ))
                        .border_1()
                        .border_color(outline),
                );
            }
            if !self.model.dragging() {
                let dimensions = r
                    .to_physical(self.monitor, f64::from(self.scale))
                    .map(|r| format!("{} × {} px", r.width(), r.height()))
                    .unwrap_or_default();
                // 按实际本地化尺寸布局，优先下方，不足时翻转并限制在窗口内。
                let mut bar = overlay_panel(cx)
                    .id("capture-toolbar")
                    .max_w(px(width))
                    .p_2()
                    .text_size(px(13.))
                    .text_color(cx.theme().foreground)
                    .shadow_sm()
                    .cursor(CursorStyle::Arrow)
                    .flex()
                    .gap_1()
                    .items_center()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child(dimensions),
                    );
                for (action, key) in [
                    (ScreenshotAction::Copy, "screenshot.copy"),
                    (ScreenshotAction::Extract, "screenshot.extract"),
                    (ScreenshotAction::Translate, "screenshot.translate"),
                ] {
                    bar = bar.child(
                        action_button(SharedString::from(key), i18n::t(&language, key))
                            .icon(match action {
                                ScreenshotAction::Copy => AppIcon::Copy,
                                ScreenshotAction::Extract => AppIcon::ScanText,
                                ScreenshotAction::Translate => AppIcon::Languages,
                            })
                            .tooltip(i18n::t(&language, key))
                            .on_click(cx.listener(move |view, _, window, cx| {
                                view.submit(action, window, cx)
                            })),
                    );
                }
                bar = bar.child(
                    icon_button("cancel", AppIcon::X, i18n::t(&language, "common.close"))
                        .on_click(cx.listener(|view, _, window, cx| view.cancel(window, cx))),
                );
                root = root.child(
                    anchored()
                        .attach(
                            Bounds {
                                origin: point(px(left), px(top)),
                                size: size(px(right - left), px(bottom - top)),
                            },
                            Anchor::BottomLeft,
                        )
                        .offset(point(px(0.), px(8.)))
                        .snap_to_window_with_margin(px(4.))
                        .child(bar),
                );
            }
        } else {
            root = root.child(shade(0., 0., width, height));
        }
        root
    }
}
