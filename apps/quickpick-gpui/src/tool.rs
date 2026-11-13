use crate::{app::AppView, ui::*};
use quickpick_core::{
    ai, i18n,
    session::{AiScope, Session},
};
use quickpick_platform::{security, system};
use std::sync::Arc;
use zeroize::Zeroizing;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Selection,
    Result,
    Input,
}
#[derive(Clone)]
pub enum Source {
    Text {
        scope: AiScope,
        action: String,
        text: Zeroizing<String>,
    },
    Image {
        action: String,
        png: Arc<Zeroizing<Vec<u8>>>,
    },
}
#[derive(Clone)]
pub struct ToolServices {
    pub foreground: isize,
    pub settings: quickpick_core::app_settings::AppSettings,
    pub paths: quickpick_core::repository::AppPaths,
    pub runtime: tokio::runtime::Handle,
}
pub struct ToolView {
    pub services: ToolServices,
    pub role: Role,
    pub owner: WeakEntity<AppView>,
    pub native: isize,
    pub pinned: bool,
    pub session: Session,
    pub source: Option<Source>,
    pub content: String,
    pub error: Option<String>,
    pub loading: bool,
    pub visible: bool,
    pub input: Entity<TextareaState>,
    pub output: Entity<TextareaState>,
    pub source_language: Entity<ChoiceState>,
    pub target_language: Entity<ChoiceState>,
    pending: Option<tokio::task::AbortHandle>,
    lifetime: u64,
    result_epoch: usize,
    _subscriptions: Vec<Subscription>,
    focus: FocusHandle,
}
impl ToolView {
    pub fn new(
        role: Role,
        owner: WeakEntity<AppView>,
        services: ToolServices,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let settings = services.settings.clone();
        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder(i18n::t(&settings.ui_language, "input.placeholder"))
        });
        let output = cx.new(|cx| {
            let mut state = TextareaState::new(window, cx);
            state.set_readonly(true, cx);
            state
        });
        let source_language = language_select(&settings.ui_language, "auto", true, window, cx);
        let target_language = language_select(
            &settings.ui_language,
            if role == Role::Input {
                &settings.input_translate_target_language
            } else {
                &settings.translation_target_language
            },
            false,
            window,
            cx,
        );
        let mut subscriptions = vec![cx.observe_window_activation(window, |view, window, cx| {
            if view.visible && !view.pinned && !window.is_window_active() {
                view.close(window, cx);
            }
        })];
        subscriptions.push(
            cx.subscribe_in(&input, window, |view, _, event, window, cx| {
                if let InputEvent::PressEnter {
                    secondary: true, ..
                } = event
                {
                    view.submit_input(window, cx);
                }
            }),
        );
        for select in [&source_language, &target_language] {
            subscriptions.push(cx.subscribe_in(
                select,
                window,
                |view, _, event: &SelectEvent<Choices>, window, cx| {
                    if matches!(event, SelectEvent::Confirm(_)) && view.source.is_some() {
                        view.start(window, cx);
                    }
                },
            ));
        }
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            let _ = weak.update(cx, |view, cx| view.close(window, cx));
            false
        });
        Self {
            services,
            role,
            owner,
            native: native(window),
            pinned: false,
            session: Session::default(),
            source: None,
            content: String::new(),
            error: None,
            loading: false,
            visible: false,
            input,
            output,
            source_language,
            target_language,
            pending: None,
            lifetime: 0,
            result_epoch: 0,
            _subscriptions: subscriptions,
            focus: cx.focus_handle(),
        }
    }
    pub fn sync_labels(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let language = &self.services.settings.ui_language;
        relabel_languages(&self.source_language, language, true, window, cx);
        relabel_languages(&self.target_language, language, false, window, cx);
        self.input.update(cx, |state, cx| {
            state.set_placeholder(i18n::t(language, "input.placeholder"), window, cx)
        });
        cx.notify();
    }
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.lifetime = self.lifetime.wrapping_add(1);
        self.session.cancel();
        if let Some(task) = self.pending.take() {
            task.abort();
        }
        self.source = None;
        {
            use zeroize::Zeroize;
            self.content.zeroize();
        }
        self.error = None;
        self.loading = false;
        self.input.update(cx, |state, cx| state.clean(window, cx));
        self.output.update(cx, |state, cx| state.clean(window, cx));
    }
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.visible = false;
        self.clear(window, cx);
        let restore = quickpick_platform::selection::current_foreground_window() == self.native;
        system::hide(self.native);
        if restore {
            system::restore_foreground(self.services.foreground);
        }
        cx.notify();
    }
    pub fn show(
        &mut self,
        source: Option<Source>,
        services: ToolServices,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Native geometry restoration may run while another GPUI window is
        // updating. Read the final backend bounds before the first layout.
        window.bounds_changed(cx);
        self.clear(window, cx);
        self.source = source;
        self.visible = true;
        self.services = services;
        self.sync_labels(window, cx);
        window.set_window_title(&i18n::t(
            &self.services.settings.ui_language,
            match self.role {
                Role::Selection => "selection.title",
                Role::Input => "input.title",
                Role::Result => "result.success",
            },
        ));
        let settings = self.services.settings.clone();
        appearance(window, &settings, cx);
        if self.role == Role::Selection
            && let Some(Source::Text { text, .. }) = &self.source
        {
            self.content = text.to_string();
        }
        system::show(self.native, self.role != Role::Result);
        window.bounds_changed(cx);
        self.focus.focus(window, cx);
        if self.role == Role::Input {
            self.input.update(cx, |state, cx| state.focus(window, cx));
        } else if self.role == Role::Result && self.source.is_some() {
            self.start(window, cx);
        }
        cx.notify();
    }
    fn submit_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.role != Role::Input {
            return;
        }
        if self.input.update(cx, |state, cx| {
            state.marked_text_range(window, cx).is_some()
        }) {
            return;
        }
        let text = self.input.read(cx).value().to_string();
        if text.trim().is_empty() {
            return;
        }
        self.source = Some(Source::Text {
            scope: AiScope::Input,
            action: "translate".into(),
            text: Zeroizing::new(text),
        });
        self.start(window, cx);
    }
    pub fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.role == Role::Input {
            let text = self.input.read(cx).value().to_string();
            if text.trim().is_empty() {
                return;
            }
            self.source = Some(Source::Text {
                scope: AiScope::Input,
                action: "translate".into(),
                text: Zeroizing::new(text),
            });
        }
        let Some(source) = self.source.clone() else {
            return;
        };
        let settings = self.services.settings.clone();
        let paths = self.services.paths.clone();
        let runtime = self.services.runtime.clone();
        let from = self
            .source_language
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or_else(|| "auto".into());
        let to = self
            .target_language
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or_else(|| settings.translation_target_language.clone());
        if let Some(task) = self.pending.take() {
            task.abort();
        }
        let id = self.session.begin();
        self.loading = true;
        self.error = None;
        self.content.clear();
        self.output.update(cx, |state, cx| state.clean(window, cx));
        let task = runtime.spawn(async move {
            let scope = match &source {
                Source::Text { scope, .. } => *scope,
                Source::Image { .. } => AiScope::Vision,
            };
            let key =
                tokio::task::spawn_blocking(move || security::load_api_key(&paths, scope.as_str()))
                    .await
                    .map_err(|_| "i18n:security.readFailed".to_string())??
                    .ok_or_else(|| {
                        format!(
                            "i18n:{}",
                            match scope {
                                AiScope::Text => "ai.apiKeyMissing",
                                AiScope::Vision => "screenshot.apiKeyMissing",
                                AiScope::Input => "input.apiKeyMissing",
                            }
                        )
                    })?;
            ai::validate_configuration(&settings, &key, scope)?;
            match source {
                Source::Text {
                    scope: AiScope::Input,
                    action,
                    text,
                } => ai::run_input_text_action(&settings, &key, &action, &text, &from, &to).await,
                Source::Text { action, text, .. } => {
                    ai::run_text_action(&settings, &key, &action, &text, &from, &to).await
                }
                Source::Image { action, png } => {
                    ai::run_image_action(&settings, &key, &action, &png, &from, &to).await
                }
            }
        });
        self.pending = Some(task.abort_handle());
        cx.spawn_in(window, async move |view, cx| {
            let result = task.await;
            let _ = view.update_in(cx, |view, window, cx| {
                if !view.visible || !view.session.finish(id) {
                    return;
                }
                view.pending = None;
                view.loading = false;
                match result {
                    Ok(Ok(text)) => {
                        view.result_epoch = view.result_epoch.wrapping_add(1);
                        view.content = text;
                        view.output.update(cx, |state, cx| {
                            state.set_value(view.content.clone(), window, cx)
                        });
                    }
                    Ok(Err(error)) => view.error = Some(error),
                    Err(_) => view.error = Some("i18n:ai.requestFailed".into()),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    fn language(&self, _cx: &App) -> String {
        self.services.settings.ui_language.clone()
    }
}
impl Render for ToolView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let language = self.language(cx);
        let t = |key: &str| i18n::t(&language, key);
        let selection = self.role == Role::Selection;
        let translation = self.role == Role::Input
            || match &self.source {
                Some(Source::Text { action, .. } | Source::Image { action, .. }) => {
                    action == "translate"
                }
                None => false,
            };
        let mut root = div()
            .id("tool")
            .track_focus(&self.focus)
            .key_context("QuickPickTool")
            .size_full()
            .overflow_hidden()
            .p_2()
            .text_size(px(13.))
            .text_color(cx.theme().foreground)
            .flex()
            .flex_col()
            .gap_2()
            .when(selection, |root| root.flex_row().items_center())
            .capture_key_down(cx.listener(|view, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    view.close(window, cx);
                }
            }));
        let controls = |cx: &mut Context<Self>| {
            div()
                .flex()
                .gap_1()
                .flex_shrink_0()
                .child(
                    icon_button(
                        "pin",
                        if self.pinned {
                            AppIcon::PinOff
                        } else {
                            AppIcon::Pin
                        },
                        t(if self.pinned {
                            "common.unpin"
                        } else {
                            "common.pin"
                        }),
                    )
                    .selected(self.pinned)
                    .toggled(self.pinned)
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.pinned = !view.pinned;
                        system::pin(view.native, view.pinned);
                        cx.notify();
                    })),
                )
                .child(
                    icon_button("close", AppIcon::X, t("common.close"))
                        .on_click(cx.listener(|view, _, window, cx| view.close(window, cx))),
                )
        };
        let mut header = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_between()
            .gap_2()
            .window_control_area(WindowControlArea::Drag);
        if self.role == Role::Input || !translation {
            header = header.child(div().flex_1().child(t(if self.role == Role::Input {
                "input.title"
            } else {
                match &self.source {
                    Some(Source::Image { .. }) => "screenshot.extract",
                    Some(Source::Text { .. }) => "common.summarize",
                    None => "result.error",
                }
            })));
        }
        let mut language_bar = div()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .min_w(px(0.));
        if !selection && translation {
            language_bar = language_bar.child(
                div()
                    .flex()
                    .items_center()
                    .flex_1()
                    .min_w(px(0.))
                    .gap_2()
                    .child(Select::new(&self.source_language).w_full().min_w(px(0.)))
                    .child(
                        icon_button("swap", AppIcon::ArrowLeftRight, t("result.switchDirection"))
                            .flex_shrink_0()
                            .on_click(cx.listener(|view, _, window, cx| {
                                let from = view
                                    .source_language
                                    .read(cx)
                                    .selected_value()
                                    .cloned()
                                    .unwrap_or_default();
                                let to = view
                                    .target_language
                                    .read(cx)
                                    .selected_value()
                                    .cloned()
                                    .unwrap_or_default();
                                view.source_language.update(cx, |state, cx| {
                                    state.set_selected_value(&to, window, cx)
                                });
                                if from != "auto" {
                                    view.target_language.update(cx, |state, cx| {
                                        state.set_selected_value(&from, window, cx)
                                    });
                                }
                                if view.source.is_some() {
                                    view.start(window, cx);
                                }
                            })),
                    )
                    .child(Select::new(&self.target_language).w_full().min_w(px(0.))),
            );
        }
        if !selection {
            if self.role == Role::Result && translation {
                // Symmetric space keeps the language direction centered independently of controls.
                header = header
                    .child(div().w(px(60.)).flex_shrink_0())
                    .child(language_bar.flex_1())
                    .child(controls(cx));
                root = root.child(header);
            } else {
                root = root.child(header.child(controls(cx)));
                if translation {
                    root = root.child(language_bar);
                }
            }
        }
        if self.role == Role::Selection {
            let required = [
                "common.copy",
                "common.translate",
                "common.summarize",
                "common.search",
            ]
            .iter()
            .map(|key| label_width(&t(key), window, cx) + 48.)
            .sum::<f32>()
                + 12.;
            let labels = required
                <= f32::from(window.viewport_size().width)
                    - 84.
                    - if self.error.is_some() { 36. } else { 0. };
            let mut actions = div()
                .flex()
                .flex_1()
                .items_center()
                .gap_1()
                .window_control_area(WindowControlArea::Drag);
            for (action, label) in [
                ("copy", "common.copy"),
                ("translate", "common.translate"),
                ("summarize", "common.summarize"),
                ("search", "common.search"),
            ] {
                actions = actions.child(
                    action_button(action, if labels { t(label) } else { String::new() })
                        .icon(match action {
                            "copy" => AppIcon::Copy,
                            "translate" => AppIcon::Languages,
                            "summarize" => AppIcon::FileText,
                            _ => AppIcon::Search,
                        })
                        .tooltip(t(label))
                        .accessibility_label(t(label))
                        .when(!labels, |button| button.w(px(32.)).px_0())
                        .disabled(self.content.is_empty())
                        .on_click(cx.listener(move |view, _, window, cx| {
                            let text = view.content.clone();
                            let owner = view.owner.clone();
                            if action == "copy" || action == "search" {
                                let lifetime = view.lifetime;
                                let task = cx.background_executor().spawn(async move {
                                    if action == "copy" {
                                        system::copy_text(&text)
                                    } else {
                                        system::search(&text)
                                    }
                                });
                                cx.spawn_in(window, async move |view, cx| {
                                    let result = task.await;
                                    let _ = view.update_in(cx, |view, window, cx| {
                                        if !view.visible || view.lifetime != lifetime {
                                            return;
                                        }
                                        if let Err(error) = result {
                                            view.error = Some(error);
                                            cx.notify();
                                        } else {
                                            view.close(window, cx);
                                        }
                                    });
                                })
                                .detach();
                            } else {
                                view.close(window, cx);
                                let _ = owner.update(cx, |owner, cx| {
                                    owner.show_tool(
                                        Role::Result,
                                        Some(Source::Text {
                                            scope: AiScope::Text,
                                            action: action.into(),
                                            text: Zeroizing::new(text),
                                        }),
                                        cx,
                                    )
                                });
                            }
                        })),
                );
            }
            if let Some(error) = &self.error {
                root = root.child(
                    icon_button("selection-error", AppIcon::X, i18n::error(&language, error))
                        .text_color(cx.theme().danger),
                );
            }
            root = root.child(actions).child(controls(cx));
        } else {
            if self.role == Role::Input {
                root = root.child(
                    Textarea::new(&self.input)
                        .h(px(96.))
                        .flex_shrink_0()
                        .text_size(px(14.)),
                );
            }
            let content = div().flex_1().min_h(px(0.)).overflow_hidden();
            if self.loading {
                root = root.child(
                    panel(cx)
                        .p_3()
                        .flex_1()
                        .min_h(px(0.))
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(Skeleton::new().w_full().h(px(16.)))
                        .child(Skeleton::new().w(px(240.)).h(px(16.)))
                        .child(Skeleton::new().w(px(180.)).h(px(16.))),
                );
            } else {
                root = root.child(
                    content
                        .child(Textarea::new(&self.output).h_full().text_size(px(14.)))
                        .with_animation(
                            ("result-appearance", self.result_epoch),
                            Animation::new(std::time::Duration::from_millis(140)),
                            |this, progress| this.opacity(progress),
                        ),
                );
            }
            if !selection && let Some(error) = &self.error {
                root = root.child(
                    div()
                        .flex_shrink_0()
                        .max_h(px(64.))
                        .id("tool-error")
                        .overflow_y_scroll()
                        .text_color(cx.theme().danger)
                        .child(i18n::error(&language, error)),
                );
            }
            root =
                root.child(
                    panel(cx)
                        .p_2()
                        .flex_shrink_0()
                        .flex()
                        .justify_end()
                        .gap_2()
                        .when(self.role == Role::Input, |footer| {
                            footer.child(
                                action_button("translate", t("common.translate"))
                                    .icon(AppIcon::Languages)
                                    .disabled(self.loading)
                                    .on_click(cx.listener(|view, _, window, cx| {
                                        view.submit_input(window, cx)
                                    })),
                            )
                        })
                        .when(self.error.is_some(), |footer| {
                            footer.child(
                                action_button("retry", t("common.retry"))
                                    .icon(AppIcon::RotateCcw)
                                    .disabled(self.source.is_none())
                                    .on_click(
                                        cx.listener(|view, _, window, cx| view.start(window, cx)),
                                    ),
                            )
                        })
                        .child(
                            action_button("copy", t("result.copy"))
                                .icon(AppIcon::Copy)
                                .disabled(self.content.is_empty())
                                .on_click(cx.listener(|view, _, window, cx| {
                                    let text = view.content.clone();
                                    let lifetime = view.lifetime;
                                    let task = cx
                                        .background_executor()
                                        .spawn(async move { system::copy_text(&text) });
                                    cx.spawn_in(window, async move |view, cx| {
                                        let result = task.await;
                                        let _ = view.update_in(cx, |view, _, cx| {
                                            if !view.visible || view.lifetime != lifetime {
                                                return;
                                            }
                                            if let Err(error) = result {
                                                view.error = Some(error);
                                                cx.notify();
                                            }
                                        });
                                    })
                                    .detach();
                                })),
                        ),
                );
        }
        root
    }
}

impl Drop for ToolView {
    fn drop(&mut self) {
        if let Some(task) = self.pending.take() {
            task.abort();
        }
        self.source = None;
        use zeroize::Zeroize;
        self.content.zeroize();
    }
}
