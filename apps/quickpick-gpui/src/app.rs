use crate::{
    native::{Action, Event, Native},
    overlay::Overlay,
    settings::{Control, SettingsForm},
    tool::{Role, Source, ToolServices, ToolView},
    ui::*,
};
use quickpick_core::{
    app_settings::{AppSettings, normalize_settings_for_save},
    appearance::AppearancePreview,
    i18n,
    repository::SettingsRepository,
    session::{AiScope, ScreenshotAction},
};
use quickpick_platform::{autostart, probe, security, selection, system};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use zeroize::Zeroizing;

gpui_kit::assets::icon_assets!(
    IconAssets,
    [
        X,
        Check,
        ChevronDown,
        ChevronRight,
        Eye,
        EyeOff,
        Loader,
        Pin,
        Copy,
        ArrowLeftRight,
        Settings,
        Minus,
        Maximize,
        PinOff,
        Search,
        Languages,
        FileText,
        ScanText,
        RotateCcw
    ]
);
struct AppAssets;
impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path == "brand.png" {
            return Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../assets/brand.png"
            ))));
        }
        IconAssets.load(path)
    }
    fn list(&self, path: &str) -> gpui_kit::Result<Vec<SharedString>> {
        let mut items = IconAssets.list(path)?;
        if "brand.png".starts_with(path) {
            items.push("brand.png".into());
        }
        Ok(items)
    }
}
pub fn run() {
    let candidate = std::env::args().any(|arg| arg == "--candidate");
    if std::env::var("GPUI_DISABLE_DIRECT_COMPOSITION")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        system::message(&i18n::t("system", "startup.graphics"));
        return;
    }
    let Some(_instance) = system::InstanceGuard::acquire().ok().flatten() else {
        return;
    };
    if system::old_instance_running() {
        system::message(&i18n::t("system", "startup.running"));
        return;
    }
    let paths = match system::paths() {
        Ok(paths) => paths,
        Err(_) => {
            system::message(&i18n::t("system", "startup.path"));
            return;
        }
    };
    let repository = SettingsRepository::new(paths);
    let settings = match repository.load() {
        Ok(settings) => settings,
        Err(_) => {
            system::message(&i18n::t("system", "settings.readFailed"));
            return;
        }
    };
    let configured = std::array::from_fn(|i| {
        security::api_key_status(&repository.paths, ["text", "vision", "input"][i])
            .is_ok_and(|s| s.configured)
    });
    let runtime_owner = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(2)
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => return,
    };
    let runtime = runtime_owner.handle().clone();
    if !candidate && autostart::apply_autostart_setting(settings.autostart_enabled).is_err() {
        system::message(&i18n::error(
            &settings.ui_language,
            "i18n:autostart.writeFailed",
        ));
    }
    let platform = match gpui_windows_backend::WindowsPlatform::new(false) {
        Ok(platform) => platform,
        Err(_) => {
            system::message(&i18n::t(&settings.ui_language, "startup.graphics"));
            return;
        }
    };
    Application::with_platform(std::rc::Rc::new(platform))
        .with_assets(AppAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            let options = WindowOptions {
                titlebar: None,
                show: false,
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(420.), px(600.)),
                    cx,
                ))),
                window_min_size: Some(size(px(420.), px(600.))),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            };
            let result = gpui_kit::open_window(options, cx, move |window, cx| {
                cx.new(|cx| {
                    AppView::new(
                        repository, settings, configured, runtime, candidate, window, cx,
                    )
                })
            });
            match result {
                Ok((handle, view)) => {
                    let _ = handle.update(cx, |_, window, cx| {
                        glass_root(
                            window,
                            cx.global::<VisualState>().0.surface_alpha(),
                            false,
                            cx,
                        )
                    });
                    let attention =
                        view.read(cx).native.is_none() || !view.read(cx).form.status.is_empty();
                    if attention {
                        view.update(cx, |view, cx| view.show_settings(cx));
                    }
                }
                Err(_) => {
                    system::message(&i18n::t("system", "input.windowUnavailable"));
                    cx.quit();
                }
            }
        });
    runtime_owner.shutdown_background();
}

pub struct AppView {
    pub settings: AppSettings,
    pub repository: SettingsRepository,
    pub runtime: tokio::runtime::Handle,
    pub form: SettingsForm,
    visual: AppearancePreview,
    nav_scroll: ScrollHandle,
    body_scroll: ScrollHandle,
    native: Option<Native>,
    main: AnyWindowHandle,
    main_native: isize,
    focus: FocusHandle,
    tools: HashMap<Role, (AnyWindowHandle, Entity<ToolView>)>,
    window_states: quickpick_core::window_state::WindowStates,
    exiting: bool,
    overlay: Option<(AnyWindowHandle, Entity<Overlay>)>,
    pub capture_busy: bool,
    capture_cancel: Arc<AtomicBool>,
    pub saving: bool,
    candidate: bool,
    recording_epoch: u64,
    last_foreground: isize,
    events: async_channel::Sender<Event>,
    configured: [bool; 3],
    _subscriptions: Vec<Subscription>,
}
impl AppView {
    fn new(
        repository: SettingsRepository,
        settings: AppSettings,
        configured: [bool; 3],
        runtime: tokio::runtime::Handle,
        candidate: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        window.set_window_title(&i18n::t(&settings.ui_language, "settings.title"));
        let visual = AppearancePreview::new(&settings);
        install_visual(visual.effective().clone(), window, cx);
        let window_states = quickpick_core::window_state::load(&repository.paths);
        let mut form = SettingsForm::new(&settings, &settings.ui_language, window, cx);
        Self::observe_form(&mut form, cx);
        let (sender, receiver) = async_channel::bounded(64);
        let main_sender = sender.clone();
        let _ =
            quickpick_platform::window_events::install(crate::ui::native(window), move |notice| {
                let _ = main_sender.try_send(notice.into());
            });
        let mut native = match Native::new(&settings, sender.clone()) {
            Ok(native) => Some(native),
            Err(error) => {
                form.status = error;
                None
            }
        };
        if let Some(native) = &mut native
            && let Err(error) = native.replace(&settings)
        {
            form.status = error;
        }
        cx.spawn(async move |view, cx| {
            while let Ok(event) = receiver.recv().await {
                if view
                    .update(cx, |view, cx| view.on_event(event, cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            let _ = weak.update(cx, |view, cx| view.close_settings(window, cx));
            false
        });
        let subscriptions = vec![cx.observe_window_appearance(window, |view, window, cx| {
            if view.visual.effective().theme_mode == "system" {
                view.refresh_visual_in(window, cx);
            }
        })];
        Self {
            settings,
            repository,
            runtime,
            form,
            visual,
            nav_scroll: ScrollHandle::new(),
            body_scroll: ScrollHandle::new(),
            native,
            main: window.window_handle(),
            main_native: crate::ui::native(window),
            focus: cx.focus_handle(),
            tools: HashMap::new(),
            window_states,
            exiting: false,
            overlay: None,
            capture_busy: false,
            capture_cancel: Arc::new(AtomicBool::new(false)),
            saving: false,
            candidate,
            recording_epoch: 0,
            last_foreground: 0,
            events: sender,
            configured,
            _subscriptions: subscriptions,
        }
    }
    fn observe_form(form: &mut SettingsForm, cx: &mut Context<Self>) {
        for key in ["themeMode", "windowEffect"] {
            if let Some(Control::Select(state)) = form.controls.get(key) {
                form._subscriptions.push(cx.subscribe(
                    state,
                    |view, _, event: &SelectEvent<Choices>, cx| {
                        if matches!(event, SelectEvent::Confirm(_)) {
                            view.preview_appearance(cx);
                        }
                    },
                ));
            }
        }
        if let Some(Control::Slider(state)) = form.controls.get("panelOpacity") {
            form._subscriptions
                .push(cx.subscribe(state, |view, _, _: &SliderEvent, cx| {
                    view.preview_appearance(cx)
                }));
        }
        for (prefix, scope, status) in [
            ("text", AiScope::Text, "settings.textProviderFilled"),
            ("vision", AiScope::Vision, "settings.visionProviderFilled"),
            ("input", AiScope::Input, "settings.inputProviderFilled"),
        ] {
            if let Some(Control::Select(select)) = form.controls.get(&format!("{prefix}AiProvider"))
            {
                form._subscriptions.push(cx.subscribe(
                    select,
                    move |view, _, event: &SelectEvent<Choices>, cx| {
                        if let SelectEvent::Confirm(Some(provider)) = event {
                            let (base, model) =
                                quickpick_core::ai::provider_defaults(provider, scope);
                            let fields = [
                                (format!("{prefix}AiBaseUrl"), base),
                                (format!("{prefix}AiModel"), model),
                            ];
                            let values: Vec<_> = fields
                                .into_iter()
                                .filter_map(|(key, value)| {
                                    if let Some(Control::Text(input)) = view.form.controls.get(&key)
                                    {
                                        Some((input.clone(), value))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let _ = view.main.update(cx, |_, window, cx| {
                                for (input, value) in values {
                                    input
                                        .update(cx, |input, cx| input.set_value(value, window, cx));
                                }
                            });
                            view.form.status = status.into();
                            cx.notify();
                        }
                    },
                ));
            }
        }
    }
    fn refresh_visual_in(&mut self, window: &mut Window, cx: &mut App) {
        install_visual(self.visual.effective().clone(), window, cx);
        appearance(window, &self.settings, cx);
        for (handle, _) in self.tools.values() {
            let _ = handle.update(cx, |_, window, cx| appearance(window, &self.settings, cx));
        }
    }
    fn preview_appearance(&mut self, cx: &mut Context<Self>) {
        let draft = self.form.appearance(cx);
        if &draft == self.visual.effective() {
            return;
        }
        self.visual.preview(draft);
        let _ = self
            .main
            .update(cx, |_, window, cx| self.refresh_visual_in(window, cx));
        cx.notify();
    }
    fn rollback_visual_in(&mut self, window: &mut Window, cx: &mut App) {
        self.visual.hide();
        self.form.restore_appearance(&self.settings, window, cx);
        self.refresh_visual_in(window, cx);
    }
    fn close_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rollback_visual_in(window, cx);
        self.form.clear_secrets(window, cx);
        self.recording_epoch = self.recording_epoch.wrapping_add(1);
        self.form.recording = None;
        if let Some(native) = &mut self.native {
            let _ = native.replace(&self.settings);
        }
        if self.native.is_none() {
            cx.quit();
        } else {
            system::hide(self.main_native);
        }
    }
    pub fn show_settings(&mut self, cx: &mut Context<Self>) {
        let reset = system::needs_reset(self.main_native);
        if reset {
            system::show(self.main_native, true);
        }
        let _ = self.main.update(cx, |_, window, cx| {
            if reset {
                if window.is_maximized() {
                    system::show(self.main_native, true);
                }
                window.resize(size(px(420.), px(600.)));
            } else {
                window.activate_window();
            }
            window.bounds_changed(cx);
            self.refresh_visual_in(window, cx);
        });
    }
    fn on_event(&mut self, event: Event, cx: &mut Context<Self>) {
        if self.exiting {
            return;
        }
        match event {
            Event::Settings => {
                if self.capture_busy {
                    self.cancel_overlay(cx);
                }
                self.show_settings(cx)
            }
            Event::Quit => {
                self.exiting = true;
                self.capture_cancel.store(true, Ordering::Release);
                for (role, (_, view)) in &self.tools {
                    if let Some(rect) = system::geometry(view.read(cx).native) {
                        self.window_states.insert(
                            match role {
                                Role::Selection => "selection",
                                Role::Result => "result",
                                Role::Input => "input",
                            }
                            .into(),
                            rect,
                        );
                    }
                }
                self.hide_tools(cx);
                let paths = self.repository.paths.clone();
                let states = self.window_states.clone();
                let task = self
                    .runtime
                    .spawn_blocking(move || quickpick_core::window_state::save(&paths, &states));
                cx.spawn(async move |_, cx| {
                    let _ = task.await;
                    cx.update(|cx| cx.quit());
                })
                .detach();
            }
            Event::Autostart => {
                if !self.saving {
                    let mut settings = self.settings.clone();
                    settings.autostart_enabled = !settings.autostart_enabled;
                    self.save(settings, cx);
                }
            }
            Event::CancelScreenshot => self.cancel_overlay(cx),
            Event::SyncWindowBounds(hwnd) => {
                if hwnd == self.main_native {
                    let _ = self
                        .main
                        .update(cx, |_, window, cx| window.bounds_changed(cx));
                } else {
                    for (handle, view) in self.tools.values() {
                        if view.read(cx).native == hwnd {
                            let _ = handle.update(cx, |_, window, cx| window.bounds_changed(cx));
                            break;
                        }
                    }
                }
            }
            Event::RestoreAppearance(hwnd) => {
                let settings = self.settings.clone();
                if hwnd == self.main_native {
                    let _ = self.main.update(cx, |_, window, cx| {
                        restore_appearance(window, &settings, cx)
                    });
                } else {
                    for (handle, view) in self.tools.values() {
                        if view.read(cx).native == hwnd {
                            let _ = handle.update(cx, |_, window, cx| {
                                restore_appearance(window, &settings, cx)
                            });
                        }
                    }
                }
            }

            Event::Hotkey(id, source) => {
                let own = source == self.main_native
                    || self
                        .tools
                        .values()
                        .any(|(_, view)| view.read(cx).native == source);
                let source = if own {
                    self.last_foreground
                } else {
                    self.last_foreground = source;
                    source
                };
                if self.saving || self.form.recording.is_some() {
                    return;
                }
                let Some((_, action, keys)) = self
                    .native
                    .as_ref()
                    .and_then(|native| native.bindings.iter().find(|(key, _, _)| key.id() == id))
                    .cloned()
                else {
                    return;
                };
                match action {
                    Action::Settings => self.show_settings(cx),
                    Action::Input => self.show_tool(Role::Input, None, cx),
                    Action::Selection => self.read_selection(source, keys, cx),
                    Action::Screenshot => self.prepare_overlay(source, keys, cx),
                }
            }
        }
    }
    pub fn show_tool(&mut self, role: Role, source: Option<Source>, cx: &mut Context<Self>) {
        let services = ToolServices {
            settings: self.settings.clone(),
            paths: self.repository.paths.clone(),
            runtime: self.runtime.clone(),
            foreground: self.last_foreground,
        };
        if !self.tools.contains_key(&role) {
            let owner = cx.weak_entity();
            let options = WindowOptions {
                titlebar: None,
                show: false,
                focus: role != Role::Result,
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(
                        px(480.),
                        px(if role == Role::Selection { 58. } else { 460. }),
                    ),
                    cx,
                ))),
                window_min_size: Some(size(
                    px(360.),
                    px(if role == Role::Selection { 58. } else { 300. }),
                )),
                is_resizable: role != Role::Selection,
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            };
            match gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| ToolView::new(role, owner, services.clone(), window, cx))
            }) {
                Ok(tool) => {
                    let sender = self.events.clone();
                    let _ = tool.0.update(cx, |_, window, _| {
                        quickpick_platform::window_events::install_overlay(
                            crate::ui::native(window),
                            move |notice| {
                                let _ = sender.try_send(notice.into());
                            },
                        )
                    });
                    let key = match role {
                        Role::Selection => "selection",
                        Role::Result => "result",
                        Role::Input => "input",
                    };
                    if let Some(geometry) = self.window_states.get(key) {
                        system::restore_geometry(
                            tool.1.read(cx).native,
                            *geometry,
                            role == Role::Selection,
                        );
                    }
                    self.tools.insert(role, tool);
                }
                Err(_) => {
                    self.form.status = "i18n:input.windowUnavailable".into();
                    self.show_settings(cx);
                    return;
                }
            }
        }
        let (handle, view) = self.tools.get(&role).unwrap().clone();
        let _ = handle.update(cx, |_, window, cx| {
            view.update(cx, |view, cx| view.show(source, services, window, cx))
        });
    }
    fn hide_tools(&mut self, cx: &mut Context<Self>) {
        let _ = self
            .main
            .update(cx, |_, window, cx| self.rollback_visual_in(window, cx));
        for (handle, view) in self.tools.values() {
            let _ = handle.update(cx, |_, window, cx| {
                view.update(cx, |view, cx| view.close(window, cx))
            });
        }
        system::hide(self.main_native);
    }
    fn show_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.show_tool(Role::Result, None, cx);
        if let Some((_, view)) = self.tools.get(&Role::Result) {
            view.update(cx, |view, cx| {
                view.error = Some(error);
                cx.notify();
            });
        }
    }
    fn read_selection(&mut self, source: isize, keys: Vec<i32>, cx: &mut Context<Self>) {
        if self.capture_busy {
            return;
        }
        self.capture_busy = true;
        self.capture_cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.capture_cancel.clone();
        self.hide_tools(cx);
        let fallback = self.settings.allow_clipboard_fallback;
        let task = self.runtime.spawn_blocking(move || {
            let Ok(_guard) = quickpick_platform::CAPTURE_ENTRY.try_lock() else {
                return None;
            };
            Some(selection::capture_selected_text_for_window(
                source, keys, fallback,
            ))
        });
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |view, cx| {
                if !Arc::ptr_eq(&view.capture_cancel, &cancel) {
                    return;
                }
                view.capture_busy = false;
                if cancel.load(Ordering::Acquire) {
                    return;
                }
                match result {
                    Ok(Some(snapshot)) => {
                        if matches!(snapshot.status, selection::SelectionStatus::Captured) {
                            view.show_tool(
                                Role::Selection,
                                Some(Source::Text {
                                    scope: AiScope::Text,
                                    action: "translate".into(),
                                    text: Zeroizing::new(snapshot.text),
                                }),
                                cx,
                            );
                        } else {
                            view.show_error(snapshot.message, cx);
                        }
                    }
                    Ok(None) => {}
                    Err(_) => view.show_error("i18n:selection.noText".into(), cx),
                }
            });
        })
        .detach();
    }
    fn prepare_overlay(&mut self, source: isize, keys: Vec<i32>, cx: &mut Context<Self>) {
        if self.capture_busy {
            return;
        }
        self.capture_busy = true;
        self.capture_cancel = Arc::new(AtomicBool::new(false));
        self.hide_tools(cx);
        let task = self.runtime.spawn_blocking(move || {
            if !selection::wait_for_capture_hotkey_release(&keys) {
                return Err("i18n:selection.hotkeyHeld".to_string());
            }
            if selection::current_foreground_window() != source {
                return Err("i18n:screenshot.sourceUnfocused".into());
            }
            selection::clear_alt_menu_mode(source);
            probe::monitor_at_cursor().map_err(str::to_owned)
        });
        let cancel = self.capture_cancel.clone();
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |view, cx| {
                if !Arc::ptr_eq(&view.capture_cancel, &cancel) {
                    return;
                }
                if cancel.load(Ordering::Acquire) {
                    view.capture_busy = false;
                    return;
                }
                match result {
                    Ok(Ok(monitor)) => view.open_overlay(monitor, cx),
                    Ok(Err(error)) => {
                        view.capture_busy = false;
                        view.show_error(error, cx);
                    }
                    Err(_) => {
                        view.capture_busy = false;
                        view.show_error("i18n:capture.failed".into(), cx);
                    }
                }
            });
        })
        .detach();
    }
    fn open_overlay(&mut self, monitor: quickpick_core::region::MonitorPx, cx: &mut Context<Self>) {
        if self.overlay.is_none() {
            let owner = cx.weak_entity();
            let options = WindowOptions {
                titlebar: None,
                show: false,
                kind: WindowKind::PopUp,
                is_movable: false,
                is_resizable: false,
                is_minimizable: false,
                window_background: WindowBackgroundAppearance::Transparent,
                ..Default::default()
            };
            match gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| Overlay::new(owner, window, cx))
            }) {
                Ok(view) => {
                    let sender = self.events.clone();
                    let activity = view.1.read(cx).activity.clone();
                    let _ = view.0.update(cx, |_, window, _| {
                        quickpick_platform::window_events::install(
                            crate::ui::native(window),
                            move |notice| {
                                if activity.load(Ordering::Acquire)
                                    && matches!(
                                        notice,
                                        quickpick_platform::window_events::Notice::CancelCapture
                                    )
                                {
                                    let _ = sender.try_send(Event::CancelScreenshot);
                                }
                            },
                        )
                    });
                    self.overlay = Some(view)
                }
                Err(_) => {
                    self.capture_busy = false;
                    self.show_error("i18n:screenshot.windowUnavailable".into(), cx);
                    return;
                }
            }
        }
        let (handle, overlay) = self.overlay.as_ref().unwrap().clone();
        let cancel = self.capture_cancel.clone();
        let language = self.settings.ui_language.clone();
        let shown = handle
            .update(cx, |_, window, cx| {
                glass_root(window, 0., true, cx);
                overlay.update(cx, |overlay, cx| {
                    overlay.show(monitor, language, cancel, window, cx)
                })
            })
            .unwrap_or(false);
        if !shown {
            self.capture_busy = false;
            self.show_error("i18n:capture.failed".into(), cx);
        }
    }
    fn cancel_overlay(&mut self, cx: &mut Context<Self>) {
        self.capture_cancel.store(true, Ordering::Release);
        if let Some((handle, overlay)) = &self.overlay {
            let _ = handle.update(cx, |_, window, cx| {
                overlay.update(cx, |overlay, cx| overlay.invalidate(window, cx))
            });
        }
        self.capture_busy = false;
    }
    pub fn screenshot_done(&mut self) {
        self.capture_busy = false;
    }
    pub fn capture(
        &mut self,
        action: ScreenshotAction,
        monitor: quickpick_core::region::MonitorPx,
        region: quickpick_core::region::RegionRectPx,
        native: isize,
        topology: Vec<[i64; 6]>,
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        let paths = self.repository.paths.clone();
        let cancel = self.capture_cancel.clone();
        let task_cancel = cancel.clone();
        let task = self.runtime.spawn_blocking(move || {
            let _guard = quickpick_platform::CAPTURE_ENTRY
                .try_lock()
                .map_err(|_| "i18n:capture.busy".to_string())?;
            quickpick_core::capture::execute_capture(
                action,
                region,
                || {
                    let key = security::load_api_key(&paths, "vision")?
                        .ok_or("i18n:screenshot.apiKeyMissing")?;
                    quickpick_core::ai::validate_configuration(&settings, &key, AiScope::Vision)
                },
                || task_cancel.load(Ordering::Acquire),
                &mut quickpick_platform::capture::WindowsCapture {
                    native,
                    monitor,
                    topology,
                },
            )
        });
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |view, cx| {
                if !Arc::ptr_eq(&view.capture_cancel, &cancel) {
                    return;
                }
                view.capture_busy = false;
                if cancel.load(Ordering::Acquire) {
                    return;
                }
                match result {
                    Ok(Ok(Some(png))) => view.show_tool(
                        Role::Result,
                        Some(Source::Image {
                            action: if action == ScreenshotAction::Extract {
                                "extract"
                            } else {
                                "translate"
                            }
                            .into(),
                            png: Arc::new(Zeroizing::new(png)),
                        }),
                        cx,
                    ),
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => view.show_error(error, cx),
                    Err(_) => view.show_error("i18n:capture.failed".into(), cx),
                }
            });
        })
        .detach();
    }
    fn save(&mut self, draft: AppSettings, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        if self.candidate && draft.autostart_enabled != self.settings.autostart_enabled {
            self.form.status = "i18n:settings.autostartCandidate".into();
            cx.notify();
            return;
        }
        let draft = match normalize_settings_for_save(draft) {
            Ok(v) => v,
            Err(error) => {
                self.form.status = error;
                cx.notify();
                return;
            }
        };
        if let Some(native) = &mut self.native {
            if let Err(error) = native.replace(&draft) {
                self.form.status = error;
                cx.notify();
                return;
            }
        } else {
            self.form.status = "i18n:hotkey.restoreFailed".into();
            cx.notify();
            return;
        }
        self.saving = true;
        let repository = self.repository.clone();
        let old = self.settings.clone();
        let next = draft.clone();
        let task = self.runtime.spawn_blocking(move || {
            let changed = old.autostart_enabled != next.autostart_enabled;
            let previous = if changed {
                Some(autostart::snapshot()?)
            } else {
                None
            };
            if changed {
                autostart::apply_autostart_setting(next.autostart_enabled)?;
            }
            if let Err(error) = repository.save(&next) {
                if let Some(snapshot) = previous {
                    autostart::restore(snapshot)?;
                }
                return Err(error);
            }
            Ok::<_, String>(())
        });
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |view, cx| {
                view.saving = false;
                match result {
                    Ok(Ok(())) => {
                        view.settings = draft;
                        view.visual.save_finished(Some(&view.settings));
                        let _ = view
                            .main
                            .update(cx, |_, window, cx| view.refresh_visual_in(window, cx));
                        if let Some(native) = &view.native {
                            let _ = native.update_menu(&view.settings);
                        }
                        view.form.status = "settings.saved".into();
                        let settings = view.settings.clone();
                        for (handle, tool) in view.tools.values() {
                            let _ = handle.update(cx, |_, window, cx| {
                                tool.update(cx, |tool, cx| {
                                    tool.services.settings = settings.clone();
                                    tool.sync_labels(window, cx);
                                    appearance(window, &settings, cx);
                                    cx.notify();
                                });
                            });
                        }
                        let _ = view.main.update(cx, |_, window, cx| {
                            view.form.clear_secrets(window, cx);
                            let category = view.form.category;
                            view.form =
                                SettingsForm::new(&settings, &settings.ui_language, window, cx);
                            view.form.category = category;
                            view.nav_scroll.scroll_to_item(category);
                            view.form.status = "settings.saved".into();
                            window.set_window_title(&i18n::t(
                                &settings.ui_language,
                                "settings.title",
                            ));
                            appearance(window, &settings, cx)
                        });
                        Self::observe_form(&mut view.form, cx);
                    }
                    failure => {
                        view.visual.save_finished(None);
                        if let Some(native) = &mut view.native {
                            let _ = native.replace(&view.settings);
                        }
                        view.form.status = match failure {
                            Ok(Err(e)) => e,
                            _ => "i18n:settings.writeFailed".into(),
                        };
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    fn key_action(
        &mut self,
        index: usize,
        clear: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.saving {
            return;
        }
        let paths = self.repository.paths.clone();
        let scope = ["text", "vision", "input"][index];
        let key = Zeroizing::new(self.form.key_inputs[index].read(cx).value().to_string());
        self.form.key_inputs[index].update(cx, |input, cx| input.clean(window, cx));
        self.saving = true;
        let task = self.runtime.spawn_blocking(move || {
            if clear {
                security::clear_api_key(&paths, scope)
            } else {
                security::save_api_key(&paths, scope, &key)
            }
        });
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |view, cx| {
                view.saving = false;
                match result {
                    Ok(Ok(())) => {
                        view.configured[index] = !clear;
                        view.form.status = "settings.saved".into();
                    }
                    Ok(Err(error)) => view.form.status = error,
                    Err(_) => view.form.status = "i18n:security.saveFailed".into(),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let language = self.settings.ui_language.clone();
        let t = |key: &str| i18n::t(&language, key);
        let mut root = div()
            .id("settings")
            .track_focus(&self.focus)
            .size_full()
            .text_size(px(13.))
            .text_color(cx.theme().foreground)
            .flex()
            .flex_col()
            .overflow_hidden();
        let titlebar = div()
            .h(px(36.))
            .flex_shrink_0()
            .px_2()
            .flex()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap_2()
                    .window_control_area(WindowControlArea::Drag)
                    .child(img("brand.png").w(px(20.)).h(px(20.)).flex_shrink_0())
                    .child(t("settings.title")),
            )
            .child(
                icon_button("min", AppIcon::Minus, t("settings.minimize"))
                    .w(px(42.))
                    .h(px(32.))
                    .on_click(|_, window, _| window.minimize_window()),
            )
            .child(
                icon_button(
                    "maximize",
                    if window.is_maximized() {
                        AppIcon::Copy
                    } else {
                        AppIcon::Maximize
                    },
                    t("settings.maximize"),
                )
                .w(px(42.))
                .h(px(32.))
                .on_click(|_, window, cx| {
                    if window.is_maximized() {
                        system::show(native(window), true);
                        window.bounds_changed(cx);
                    } else {
                        window.zoom_window();
                    }
                }),
            )
            .child(
                icon_button("close", AppIcon::X, t("settings.close"))
                    .w(px(42.))
                    .h(px(32.))
                    .on_click(cx.listener(|view, _, window, cx| view.close_settings(window, cx))),
            );
        root = root.child(titlebar);
        let mut tabs = panel(cx)
            .id("settings-navigation")
            .m_2()
            .p_1()
            .flex_shrink_0()
            .flex()
            .flex_nowrap()
            .gap_1()
            .overflow_x_scroll()
            .track_scroll(&self.nav_scroll);
        for (index, label) in SettingsForm::categories().into_iter().enumerate() {
            tabs = tabs.child(
                action_button(("category", index), t(label))
                    .px_2()
                    .flex_shrink_0()
                    .selected(self.form.category == index)
                    .on_click(cx.listener(move |view, _, _, cx| {
                        view.form.category = index;
                        view.nav_scroll.scroll_to_item(index);
                        view.body_scroll.set_offset(point(px(0.), px(0.)));
                        cx.notify();
                    })),
            );
        }
        root = root.child(tabs.horizontal_scrollbar(&self.nav_scroll));
        let mut body = panel(cx)
            .id("settings-scroll")
            .track_scroll(&self.body_scroll)
            .mx_2()
            .mb_2()
            .p_3()
            .min_h(px(0.))
            .overflow_y_scroll()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_size(px(16.))
                    .pb_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(t(SettingsForm::categories()[self.form.category])),
            );
        for (key, label) in self.form.fields() {
            let control = self.form.controls.get(key).unwrap().clone();
            let mut row = div()
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap_2()
                .pb_3()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(t(label)),
                );
            row = match control {
                Control::Toggle(value) => row.child(
                    Checkbox::new(SharedString::from(key))
                        .checked(value)
                        .disabled(self.saving)
                        .label(t(if value {
                            "settings.enabled"
                        } else {
                            "settings.disabled"
                        }))
                        .on_click(cx.listener(move |view, value, _, cx| {
                            view.form
                                .controls
                                .insert(key.into(), Control::Toggle(*value));
                            cx.notify();
                        })),
                ),
                Control::Text(input) if key.ends_with("Hotkey") => row.child(
                    action_button(
                        SharedString::from(key),
                        if self.form.recording.as_deref() == Some(key) {
                            t("settings.hotkeyCapture")
                        } else {
                            input.read(cx).value().to_string()
                        },
                    )
                    .disabled(self.saving)
                    .on_click(cx.listener(move |view, _, window, cx| {
                        if let Some(native) = &mut view.native
                            && let Err(error) = native.clear()
                        {
                            let _ = native.replace(&view.settings);
                            view.form.status = error;
                            cx.notify();
                            return;
                        }
                        view.recording_epoch = view.recording_epoch.wrapping_add(1);
                        view.form.recording = Some(key.into());
                        view.focus.focus(window, cx);
                        cx.notify();
                    })),
                ),
                Control::Text(input) => row.child(
                    Input::new(&input)
                        .text_size(px(14.))
                        .h(px(32.))
                        .disabled(self.saving),
                ),
                Control::Select(select) => {
                    row.child(Select::new(&select).h(px(32.)).disabled(self.saving))
                }
                Control::Slider(slider) => row.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(Slider::new(&slider).flex_1().disabled(self.saving))
                        .child(format!("{}%", self.form.appearance(cx).panel_opacity)),
                ),
            };
            if key.ends_with("Hotkey")
                && let Some(Control::Text(input)) = self.form.controls.get(key)
            {
                let registered = input
                    .read(cx)
                    .value()
                    .parse::<global_hotkey::hotkey::HotKey>()
                    .ok()
                    .is_some_and(|key| {
                        self.native.as_ref().is_some_and(|native| {
                            native
                                .bindings
                                .iter()
                                .any(|(active, _, _)| active.id() == key.id())
                        })
                    });
                row = row.child(t(if registered {
                    "hotkey.status.registered"
                } else {
                    "hotkey.status.failed"
                }));
            }
            body = body.child(row);
        }
        if (2..=4).contains(&self.form.category) {
            let index = self.form.category - 2;
            body = body
                .child(t(if self.configured[index] {
                    "settings.encrypted"
                } else {
                    "settings.notConfigured"
                }))
                .child(Input::new(&self.form.key_inputs[index]))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            action_button(
                                "save-key",
                                t([
                                    "settings.saveTextKey",
                                    "settings.saveVisionKey",
                                    "settings.saveInputKey",
                                ][index]),
                            )
                            .disabled(self.saving)
                            .on_click(cx.listener(
                                move |view, _, window, cx| {
                                    view.key_action(index, false, window, cx)
                                },
                            )),
                        )
                        .child(
                            action_button("clear-key", t("settings.clear"))
                                .disabled(self.saving)
                                .on_click(cx.listener(move |view, _, window, cx| {
                                    view.key_action(index, true, window, cx)
                                })),
                        ),
                );
        }
        if self.form.category == 6 {
            for key in [
                "settings.contentHistoryValue",
                "settings.allowClipboardFallbackDescription",
            ] {
                body = body.child(t(key));
            }
        }
        root.child(body.vertical_scrollbar(&self.body_scroll))
            .child(
                panel(cx)
                    .mx_2()
                    .mb_2()
                    .p_3()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .id("settings-status")
                            .max_h(px(48.))
                            .overflow_y_scroll()
                            .text_size(px(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child(i18n::error(&language, &self.form.status)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                action_button("save", t("settings.save"))
                                    .primary()
                                    .flex_1()
                                    .disabled(self.saving || self.form.recording.is_some())
                                    .on_click(cx.listener(|view, _, _, cx| {
                                        match view.form.collect(cx) {
                                            Ok(settings) => view.save(settings, cx),
                                            Err(error) => {
                                                view.form.status = error;
                                                cx.notify();
                                            }
                                        }
                                    })),
                            )
                            .child(
                                icon_button("reset", AppIcon::RotateCcw, t("settings.reset"))
                                    .h(px(32.))
                                    .disabled(self.saving)
                                    .on_click(cx.listener(|view, _, window, cx| {
                                        view.form.clear_secrets(window, cx);
                                        view.form = SettingsForm::new(
                                            &AppSettings::default(),
                                            &view.settings.ui_language,
                                            window,
                                            cx,
                                        );
                                        Self::observe_form(&mut view.form, cx);
                                        view.preview_appearance(cx);
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, window, cx| {
                let Some(key) = view.form.recording.clone() else {
                    return;
                };
                if event.keystroke.key == "escape" {
                    view.recording_epoch = view.recording_epoch.wrapping_add(1);
                    view.form.recording = None;
                    if let Some(native) = &mut view.native {
                        let _ = native.replace(&view.settings);
                    }
                    cx.notify();
                    return;
                }
                let mods = &event.keystroke.modifiers;
                if !mods.alt {
                    return;
                }
                let mut parts = vec![];
                if mods.control {
                    parts.push("Ctrl".to_string());
                }
                if mods.shift {
                    parts.push("Shift".to_string());
                }
                parts.push("Alt".into());
                if mods.platform {
                    parts.push("Win".into());
                }
                parts.push(event.keystroke.key.to_ascii_uppercase());
                let text = parts.join("+");
                if quickpick_core::app_settings::parse_hotkey(&text, "").is_ok() {
                    let release_keys = text
                        .parse::<global_hotkey::hotkey::HotKey>()
                        .map(|key| crate::native::release_keys(&key))
                        .unwrap_or_default();
                    if let Some(Control::Text(input)) = view.form.controls.get(&key) {
                        input.update(cx, |input, cx| input.set_value(text, window, cx));
                    }
                    let epoch = view.recording_epoch;
                    let task = view.runtime.spawn_blocking(move || {
                        selection::wait_for_capture_hotkey_release(&release_keys)
                    });
                    cx.spawn(async move |view, cx| {
                        let _ = task.await;
                        let _ = view.update(cx, |view, cx| {
                            if epoch != view.recording_epoch {
                                return;
                            }
                            view.form.recording = None;
                            if let Some(native) = &mut view.native {
                                let _ = native.replace(&view.settings);
                            }
                            cx.notify();
                        });
                    })
                    .detach();
                }
            }))
    }
}

pub fn print_bindings() {
    let Ok(paths) = system::paths() else {
        return;
    };
    let Ok(settings) = SettingsRepository::new(paths).load() else {
        return;
    };
    let rows: Vec<_> = [
        ("settings", &settings.settings_hotkey),
        ("selection", &settings.selection_hotkey),
        ("screenshot", &settings.screenshot_hotkey),
        ("input", &settings.input_translate_hotkey),
    ]
    .into_iter()
    .filter_map(|(role, key)| {
        key.parse::<global_hotkey::hotkey::HotKey>()
            .ok()
            .map(|hotkey| {
                let mut press = crate::native::release_keys(&hotkey);
                press.retain(|key| *key != 0x5c);
                serde_json::json!({"role":role,"key":key,"id":hotkey.id(),"press":press})
            })
    })
    .collect();
    let language = &settings.ui_language;
    println!(
        "{}",
        serde_json::json!({"bindings":rows,"titles":{"settings":i18n::t(language,"settings.title"),"selection":i18n::t(language,"selection.title"),"input":i18n::t(language,"input.title"),"screenshot":i18n::t(language,"settings.hotkeyScreenshot")}})
    );
}

impl Drop for AppView {
    fn drop(&mut self) {
        self.capture_cancel.store(true, Ordering::Release);
    }
}
