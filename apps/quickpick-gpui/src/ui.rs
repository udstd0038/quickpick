pub use gpui_kit::{
    component::{
        ActiveTheme, Disableable, Selectable,
        button::Button,
        button::ButtonVariants,
        checkbox::Checkbox,
        input::{Input, InputEvent, InputState, Textarea, TextareaState},
        scroll::ScrollableElement,
        select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
        skeleton::Skeleton,
        slider::{Slider, SliderEvent, SliderState, SliderValue},
    },
    prelude::*,
    *,
};
use quickpick_core::i18n;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[derive(Clone)]
pub struct Choice {
    pub id: String,
    pub label: SharedString,
}
impl SelectItem for Choice {
    type Value = String;
    fn title(&self) -> SharedString {
        self.label.clone()
    }
    fn value(&self) -> &String {
        &self.id
    }
}
pub type Choices = SearchableVec<Choice>;
pub type ChoiceState = SelectState<Choices>;
pub fn select(
    items: Vec<(String, String)>,
    value: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ChoiceState> {
    let selected = items
        .iter()
        .position(|(id, _)| id == value)
        .map(gpui_kit::component::IndexPath::new);
    let values = SearchableVec::new(
        items
            .into_iter()
            .map(|(id, label)| Choice {
                id,
                label: label.into(),
            })
            .collect::<Vec<_>>(),
    );
    cx.new(|cx| SelectState::new(values, selected, window, cx))
}
pub fn language_select(
    language: &str,
    value: &str,
    auto: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ChoiceState> {
    select(language_items(language, auto), value, window, cx)
}
fn language_items(language: &str, auto: bool) -> Vec<(String, String)> {
    let mut items = vec![];
    if auto {
        items.push(("auto".into(), i18n::t(language, "language.auto")));
    }
    for code in i18n::LANGUAGES {
        let key = match code {
            "zh-Hans" => "language.zhHans",
            "zh-Hant" => "language.zhHant",
            "en" => "language.en",
            "ko" => "language.ko",
            "ja" => "language.ja",
            "fr" => "language.fr",
            "de" => "language.de",
            "es" => "language.es",
            _ => "",
        };
        items.push((
            code.into(),
            i18n::t(language, if key.is_empty() { code } else { key }),
        ));
    }
    items
}
pub fn relabel_languages(
    state: &Entity<ChoiceState>,
    language: &str,
    auto: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let choices = SearchableVec::new(
        language_items(language, auto)
            .into_iter()
            .map(|(id, label)| Choice {
                id,
                label: label.into(),
            })
            .collect::<Vec<_>>(),
    );
    state.update(cx, |state, cx| state.set_items(choices, window, cx));
}
pub fn native(window: &Window) -> isize {
    match HasWindowHandle::window_handle(window).map(|h| h.as_raw()) {
        Ok(RawWindowHandle::Win32(h)) => h.hwnd.get(),
        _ => 0,
    }
}
pub fn glass_root(window: &mut Window, opacity: f32, transparent: bool, cx: &mut App) {
    let mut bg = cx.theme().background;
    bg.a = if transparent { 0. } else { opacity };
    gpui_kit::base::Root::update(window, cx, |root, _, cx| {
        root.style().background = Some(bg.into());
        cx.notify();
    });
}
pub use gpui_kit::assets::IconName as AppIcon;
use quickpick_core::appearance::AppearanceDraft;
#[derive(Clone)]
pub struct VisualState(pub AppearanceDraft);
impl Global for VisualState {}

/// One application appearance; tool services retain committed business settings.
pub fn install_visual(draft: AppearanceDraft, window: &mut Window, cx: &mut App) {
    let mode = match draft.theme_mode.as_str() {
        "dark" => gpui_kit::component::ThemeMode::Dark,
        "light" => gpui_kit::component::ThemeMode::Light,
        _ => window.appearance().into(),
    };
    cx.set_global(VisualState(draft));
    if cx.theme().mode != mode {
        gpui_kit::component::Theme::change(mode, Some(window), cx);
    }
    let dark = mode == gpui_kit::component::ThemeMode::Dark;
    gpui_kit::component::Theme::update(cx, |theme| {
        theme.font_family = "Segoe UI".into();
        theme.font_size = px(13.);
        theme.radius = px(8.);
        theme.radius_lg = px(10.);
        theme.window_radius = px(10.);
        let c = &mut theme.colors;
        c.background = rgb(if dark { 0x0a0e13 } else { 0xeef5ff }).into();
        c.foreground = rgb(if dark { 0xeaf3ff } else { 0x13213c }).into();
        c.border = rgba(if dark { 0xa4b4ca33 } else { 0x2f6fec33 }).into();
        c.input = rgb(if dark { 0x0b0e12 } else { 0xffffff }).into();
        c.input.a = 0.;
        c.popover = rgb(if dark { 0x181d24 } else { 0xf8fbff }).into();
        c.popover_foreground = c.foreground;
        c.button = rgb(if dark { 0x1e242d } else { 0xffffff }).into();
        c.button.a = 0.;
        c.button_foreground = c.foreground;
        c.button_hover = rgb(if dark { 0x293442 } else { 0xe2eeff }).into();
        c.button_active = rgb(if dark { 0x324254 } else { 0xcbdfff }).into();
        c.button_hover.a = 0.16;
        c.button_active.a = 0.24;
        c.primary = rgb(0x2f6fec).into();
        c.primary_hover = rgb(0x3e7cf4).into();
        c.primary_active = rgb(0x255acb).into();
        c.primary_foreground = rgb(0xffffff).into();
        c.button_primary = c.primary;
        c.button_primary_hover = c.primary_hover;
        c.button_primary_active = c.primary_active;
        c.button_primary_foreground = c.primary_foreground;
        c.slider_bar = c.primary;
        c.slider_thumb = rgb(if dark { 0x9cbcff } else { 0x5c89d9 }).into();
        c.ring = c.primary;
        c.accent = c.button_hover;
        c.accent_foreground = c.foreground;
        c.muted = rgb(if dark { 0x1b2129 } else { 0xe2eeff }).into();
        c.muted.a = 0.12;
        c.muted_foreground = rgb(if dark { 0xcbd5e1 } else { 0x51637e }).into();
        c.skeleton = c.muted;
    });
}
pub fn appearance(
    window: &mut Window,
    settings: &quickpick_core::app_settings::AppSettings,
    cx: &mut App,
) {
    apply_appearance(window, settings, cx);
}
pub fn restore_appearance(
    window: &mut Window,
    settings: &quickpick_core::app_settings::AppSettings,
    cx: &mut App,
) {
    apply_appearance(window, settings, cx);
}
fn apply_appearance(
    window: &mut Window,
    settings: &quickpick_core::app_settings::AppSettings,
    cx: &mut App,
) {
    let draft = cx
        .try_global::<VisualState>()
        .map(|v| v.0.clone())
        .unwrap_or_else(|| settings.into());
    quickpick_platform::window_events::clear_backdrops(native(window));
    window.set_background_appearance(if draft.window_effect == "mica" {
        WindowBackgroundAppearance::MicaBackdrop
    } else {
        WindowBackgroundAppearance::Blurred
    });
    glass_root(window, draft.surface_alpha(), false, cx);
}
fn panel_with_alpha(cx: &App, alpha: f32) -> Div {
    let mut background: Hsla = rgb(if cx.theme().mode == gpui_kit::component::ThemeMode::Dark {
        0x13171d
    } else {
        0xffffff
    })
    .into();
    background.a = alpha;
    div()
        .border_1()
        .border_color(cx.theme().border)
        .rounded(px(8.))
        .bg(background)
}
pub fn panel(cx: &App) -> Div {
    panel_with_alpha(cx, 0.)
}
pub fn overlay_panel(cx: &App) -> Div {
    // Capture controls remain readable; ordinary opacity never fills the overlay root.
    panel_with_alpha(cx, 0.94)
}
pub fn action_button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    let label = label.into();
    Button::new(id)
        .accessibility_label(label.clone())
        .child(
            div()
                .text_size(px(13.))
                .min_w(px(0.))
                .whitespace_nowrap()
                .text_ellipsis()
                .child(label),
        )
        .h(px(32.))
        .px_3()
        .rounded(px(7.))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}
pub fn icon_button(
    id: impl Into<ElementId>,
    icon: AppIcon,
    tooltip: impl Into<SharedString>,
) -> Button {
    let tooltip = tooltip.into();
    Button::new(id)
        .icon(icon)
        .accessibility_label(tooltip.clone())
        .tooltip(tooltip)
        .w(px(28.))
        .h(px(28.))
        .compact()
        .rounded(px(7.))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}

pub fn label_width(label: &str, window: &Window, cx: &App) -> f32 {
    let run = TextRun {
        len: label.len(),
        font: font("Segoe UI"),
        color: cx.theme().foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    f32::from(
        window
            .text_system()
            .shape_line(label.to_owned().into(), px(13.), &[run], None)
            .width,
    )
}
