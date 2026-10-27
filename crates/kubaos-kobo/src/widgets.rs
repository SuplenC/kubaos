//! Styled building blocks. Every function takes the (possibly faded)
//! palette, so a whole screen can fade in by passing `DARK.faded(a)`.

use std::f32::consts::PI;

use iced::widget::canvas;
use iced::widget::canvas::{Frame, Geometry, Path, Stroke, path::Arc};
use iced::widget::{Space, button, column, container, row, scrollable, text};
use std::time::Instant;

use iced::{
    Alignment, Background, Border, Color, Element, Length, Padding, Point, Radians, Rectangle,
    Renderer, Shadow, Size, Theme, Vector, mouse,
};

use crate::app::Message;
use crate::battery::Battery;
use crate::edit::Caret;
use crate::field::{Field, Pointer};
use crate::theme::{self, Icon, Palette, alpha, mix, radius, size, space};

pub fn icon<'a>(i: Icon, px: f32, c: Color) -> iced::widget::Text<'a> {
    text(i.glyph().to_string())
        .font(theme::ICONS)
        .size(px)
        .color(c)
        .line_height(1.0)
}

/// Japanese accent above a title.
pub fn accent<'a>(s: &'a str, p: &Palette) -> iced::widget::Text<'a> {
    text(s).font(theme::JP).size(13).color(p.primary)
}

pub fn title<'a>(s: impl text::IntoFragment<'a>, p: &Palette) -> iced::widget::Text<'a> {
    text(s)
        .font(theme::SANS_SEMIBOLD)
        .size(size::TITLE)
        .color(p.on_surface)
}

pub fn body<'a>(s: impl text::IntoFragment<'a>, p: &Palette) -> iced::widget::Text<'a> {
    text(s)
        .size(size::MD)
        .color(p.on_surface_variant)
        .line_height(1.45)
}

pub fn label<'a>(s: impl text::IntoFragment<'a>, p: &Palette) -> iced::widget::Text<'a> {
    text(s)
        .font(theme::SANS_MEDIUM)
        .size(size::SM)
        .color(p.on_surface_variant)
}

/// Screen header: accent, title, one line of explanation.
pub fn header<'a>(jp: &'a str, t: &'a str, sub: &'a str, p: &Palette) -> Element<'a, Message> {
    column![
        accent(jp, p),
        Space::new().height(2),
        title(t, p),
        Space::new().height(space::SM),
        body(sub, p),
    ]
    .into()
}

fn state_layer(base: Option<Color>, on: Color, status: button::Status) -> Option<Background> {
    let layer = match status {
        button::Status::Hovered => STATE_HOVER,
        button::Status::Pressed => STATE_PRESS,
        _ => 0.0,
    };
    match base {
        Some(b) => Some(Background::Color(mix(b, Color { a: b.a, ..on }, layer))),
        None if layer > 0.0 => Some(Background::Color(alpha(on, layer))),
        None => None,
    }
}

use theme::{STATE_HOVER, STATE_PRESS};

fn button_label<'a>(
    label: &'a str,
    ic: Option<Icon>,
    trailing: bool,
    c: Color,
) -> Element<'a, Message> {
    let t = text(label)
        .font(theme::SANS_SEMIBOLD)
        .size(size::MD)
        .color(c);
    match (ic, trailing) {
        (None, _) => t.into(),
        (Some(i), false) => row![icon(i, 18.0, c), t]
            .spacing(space::SM)
            .align_y(Alignment::Center)
            .into(),
        (Some(i), true) => row![t, icon(i, 18.0, c)]
            .spacing(space::SM)
            .align_y(Alignment::Center)
            .into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Filled primary pill: the one thing to do next.
    Primary,
    /// Secondary container: alternatives.
    Tonal,
    /// Text only: Back, Skip.
    Text,
}

pub fn btn<'a>(
    label: &'a str,
    ic: Option<Icon>,
    trailing_icon: bool,
    kind: Kind,
    on_press: Option<Message>,
    focused: bool,
    p: &Palette,
) -> Element<'a, Message> {
    let p = *p;
    let enabled = on_press.is_some();
    let (bg, fg) = match kind {
        Kind::Primary => (Some(p.primary), p.on_primary),
        Kind::Tonal => (Some(p.secondary_container), p.on_secondary_container),
        Kind::Text => (None, p.primary),
    };
    let fg_shown = if enabled {
        fg
    } else {
        alpha(p.on_surface, 0.38)
    };
    let b = button(button_label(label, ic, trailing_icon, fg_shown))
        .padding(Padding::from([
            10.0,
            if kind == Kind::Text { 14.0 } else { 22.0 },
        ]))
        .on_press_maybe(on_press)
        .style(move |_t: &Theme, status| {
            let bg = if enabled {
                bg
            } else {
                bg.map(|_| alpha(p.on_surface, 0.10))
            };
            button::Style {
                background: state_layer(bg, fg, status),
                text_color: fg_shown,
                border: Border::default().rounded(radius::FULL),
                shadow: Shadow::default(),
                snap: true,
            }
        });
    ring(b, focused, radius::FULL, &p)
}

/// The keyboard focus ring: a 2 px primary outline a few pixels outside the
/// control. Always reserves its space, so focus never moves the layout.
pub fn ring<'a>(
    content: impl Into<Element<'a, Message>>,
    focused: bool,
    r: f32,
    p: &Palette,
) -> Element<'a, Message> {
    let c = if focused {
        p.primary
    } else {
        Color::TRANSPARENT
    };
    container(content)
        .padding(3)
        .style(move |_t| container::Style {
            border: Border {
                color: c,
                width: 2.0,
                radius: (r + 3.0).into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// How far the focus halo sits outside a card, in px.
pub const HALO: f32 = 4.0;

/// Keyboard focus around a card: a tertiary ring [`HALO`] px outside it. Amber means picked on
/// the same cards, so focus never borrows it. The padding is there unfocused too, so moving
/// focus never shifts the layout.
pub fn halo<'a>(
    content: impl Into<Element<'a, Message>>,
    focused: bool,
    r: f32,
    p: &Palette,
) -> Element<'a, Message> {
    let c = if focused { p.tertiary } else { Color::TRANSPARENT };
    container(content)
        .padding(HALO)
        .style(move |_t| container::Style {
            border: Border {
                color: c,
                width: 2.0,
                radius: (r + HALO).into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// How much of something is picked, for its check badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    Empty,
    Partial,
    Full,
}

impl Tick {
    /// `picked` out of `total`. An empty set reads as unpicked, not as fully picked.
    pub fn of(picked: u32, total: u32) -> Self {
        match picked {
            0 => Tick::Empty,
            n if n >= total => Tick::Full,
            _ => Tick::Partial,
        }
    }
}

/// A round check badge `px` across: filled amber with a tick when full, amber-rimmed with a
/// dash when partial, an empty `rim` ring on `fill` when empty.
pub fn check_badge<'a>(tick: Tick, px: f32, rim: Color, fill: Color, p: &Palette) -> Element<'a, Message> {
    let pp = *p;
    let mark: Element<'a, Message> = match tick {
        Tick::Full => icon(Icon::Check, px * 0.72, p.on_primary).into(),
        Tick::Partial => container(Space::new())
            .width(px * 0.46)
            .height(2)
            .style(move |_t| container::Style {
                background: Some(Background::Color(pp.primary)),
                border: Border::default().rounded(1),
                ..container::Style::default()
            })
            .into(),
        Tick::Empty => Space::new().into(),
    };
    let (bg, border) = match tick {
        Tick::Full => (p.primary, p.primary),
        Tick::Partial => (alpha(p.primary, 0.18), p.primary),
        Tick::Empty => (fill, rim),
    };
    container(mark)
        .center(px)
        .style(move |_t| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border,
                width: 2.0,
                radius: radius::FULL.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// A round icon-only button.
#[allow(dead_code)] // for the first-boot setup screens
pub fn icon_btn<'a>(
    i: Icon,
    on_press: Option<Message>,
    focused: bool,
    p: &Palette,
) -> Element<'a, Message> {
    let p = *p;
    let b = button(icon(i, 20.0, p.on_surface_variant))
        .padding(8)
        .on_press_maybe(on_press)
        .style(move |_t: &Theme, status| button::Style {
            background: state_layer(None, p.on_surface, status),
            text_color: p.on_surface_variant,
            border: Border::default().rounded(radius::FULL),
            shadow: Shadow::default(),
            snap: true,
        });
    ring(b, focused, radius::FULL, &p)
}

/// A small icon-and-label button in the muted colour, for the chrome around a screen rather than
/// the screen's own task.
pub fn quiet_btn<'a>(
    label: &'a str,
    ic: Icon,
    on_press: Option<Message>,
    focused: bool,
    p: &Palette,
) -> Element<'a, Message> {
    let p = *p;
    let fg = p.on_surface_variant;
    let content = row![
        icon(ic, 18.0, fg),
        text(label)
            .font(theme::SANS_MEDIUM)
            .size(size::SM)
            .color(fg)
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    let b = button(content)
        .padding(Padding::from([6.0, 12.0]))
        .on_press_maybe(on_press)
        .style(move |_t: &Theme, status| button::Style {
            background: state_layer(None, p.on_surface, status),
            text_color: fg,
            border: Border::default().rounded(radius::FULL),
            shadow: Shadow::default(),
            snap: true,
        });
    ring(b, focused, radius::FULL, &p)
}

/// The battery's outline, in logical pixels: as tall as a line of `size::SM` text.
const BATTERY_BODY: Size = Size::new(22.0, 12.0);

/// The battery in the chrome, beside the power button: drawn filled to its percent, a bolt while
/// the charger is in, and the percent. Muted like `quiet_btn`; low, in the error colour. Drawn
/// rather than a glyph: the bundled icon subset has no battery, and a drawn one shows the level.
pub fn battery<'a>(b: Battery, p: &Palette) -> Element<'a, Message> {
    let fg = if b.is_low() {
        p.error
    } else {
        p.on_surface_variant
    };
    let level = u16::from(b.percent.min(100));
    let mut fill = row![].height(Length::Fill);
    if level > 0 {
        fill = fill.push(
            container(Space::new())
                .width(Length::FillPortion(level))
                .height(Length::Fill)
                .style(move |_t| {
                    container::Style::default()
                        .background(fg)
                        .border(Border::default().rounded(1.5))
                }),
        );
    }
    if level < 100 {
        fill = fill.push(Space::new().width(Length::FillPortion(100 - level)));
    }
    let outline = container(fill)
        .width(BATTERY_BODY.width)
        .height(BATTERY_BODY.height)
        .padding(2)
        .style(move |_t| {
            container::Style::default().border(Border {
                color: fg,
                width: 1.5,
                radius: 3.5.into(),
            })
        });
    let nub = container(Space::new()).width(2).height(5).style(move |_t| {
        container::Style::default()
            .background(fg)
            .border(Border::default().rounded(iced::border::Radius::default().right(1.0)))
    });
    let mut content = row![row![outline, nub].align_y(Alignment::Center)]
        .spacing(6)
        .align_y(Alignment::Center);
    if b.on_charger() {
        content = content.push(icon(Icon::Bolt, 15.0, fg));
    }
    content = content.push(
        text(format!("{}%", b.percent))
            .font(theme::SANS_MEDIUM)
            .size(size::SM)
            .color(fg),
    );
    container(content).padding(Padding::from([6.0, 4.0])).into()
}

/// Label above a control, with an optional note under it.
pub fn field<'a>(
    lbl: &'a str,
    control: impl Into<Element<'a, Message>>,
    note: Option<Element<'a, Message>>,
    p: &Palette,
) -> Element<'a, Message> {
    field_with(lbl, None, control.into(), note, p)
}

/// Height of a field's label row: fits a small action (the Show toggle,
/// with its focus ring) so fields side by side line up either way.
pub const LABEL_ROW: f32 = 30.0;

/// `field`, with an optional small action on the right of the label.
pub fn field_with<'a>(
    lbl: &'a str,
    action: Option<Element<'a, Message>>,
    control: Element<'a, Message>,
    note: Option<Element<'a, Message>>,
    p: &Palette,
) -> Element<'a, Message> {
    let mut head = row![label(lbl, p), Space::new().width(Length::Fill)]
        .align_y(Alignment::Center)
        .height(LABEL_ROW);
    if let Some(a) = action {
        head = head.push(a);
    }
    let mut c = column![head, control].spacing(4);
    if let Some(n) = note {
        c = c.push(n);
    }
    c.into()
}

/// How a text field shows.
pub struct FieldLook<'a> {
    pub value: &'a str,
    /// Shown dimmed when the value is empty, with the caret in front of it.
    /// A hint, never a value that looks typed ("Your name", not a name).
    pub placeholder: &'a str,
    /// Bullets instead of the value.
    pub secret: bool,
    /// Some when focused: where the caret and selection are.
    pub caret: Option<Caret>,
    /// When the caret last moved (it stays solid for a moment after).
    pub blink_from: Instant,
    /// Read-only: shown dimmed with a lock, never focused.
    pub locked: bool,
    /// Red border: the value is wrong (say why next to it).
    pub error: bool,
    pub icon: Option<Icon>,
}

/// A one-line text field (`field::Field`). Editing is the app's
/// (`edit.rs`); the field draws the caret and selection, blinks, scrolls
/// sideways to keep the caret in view, and turns clicks and drags into
/// `Pointer`s for `on_pointer`.
pub fn text_field<'a>(
    look: FieldLook<'a>,
    on_pointer: Option<impl Fn(Pointer) -> Message + 'a>,
    p: &Palette,
) -> Element<'a, Message> {
    let shown = if look.secret {
        "•".repeat(look.value.chars().count())
    } else {
        look.value.to_string()
    };
    Field {
        shown,
        placeholder: look.placeholder,
        size: size::LG,
        font: theme::SANS,
        icon: look.icon.map(Icon::glyph),
        locked: look.locked,
        error: look.error,
        caret: look.caret.filter(|_| !look.locked),
        blink_from: look.blink_from,
        palette: *p,
        on_pointer: on_pointer
            .filter(|_| !look.locked)
            .map(|f| Box::new(f) as Box<dyn Fn(Pointer) -> Message + 'a>),
    }
    .into()
}

pub const ROW_HEIGHT: f32 = 44.0;
/// Space between list rows (`list_well`'s column spacing).
pub const ROW_GAP: f32 = 2.0;

/// One selectable row in a list.
pub fn list_row<'a>(
    content: impl Into<Element<'a, Message>>,
    selected: bool,
    on_press: Message,
    p: &Palette,
) -> Element<'a, Message> {
    let p = *p;
    button(
        container(content)
            .height(Length::Fill)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(ROW_HEIGHT)
    .padding(Padding::from([0.0, 14.0]))
    .on_press(on_press)
    .style(move |_t: &Theme, status| {
        let (bg, fg) = if selected {
            (Some(p.secondary_container), p.on_secondary_container)
        } else {
            (None, p.on_surface)
        };
        button::Style {
            background: state_layer(bg, p.on_surface, status),
            text_color: fg,
            border: Border::default().rounded(radius::MD),
            shadow: Shadow::default(),
            snap: true,
        }
    })
    .into()
}

pub fn scroll_style(p: Palette) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |_t, status| {
        let hot = matches!(
            status,
            scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
        );
        let rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(alpha(
                    p.on_surface_variant,
                    if hot { 0.45 } else { 0.22 },
                )),
                border: Border::default().rounded(radius::FULL),
            },
        };
        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(p.surface_container_high),
                border: Border::default().rounded(radius::FULL),
                shadow: Shadow::default(),
                icon: p.on_surface,
            },
        }
    }
}

pub fn scroll<'a>(
    id: &'static str,
    content: impl Into<Element<'a, Message>>,
    p: &Palette,
) -> scrollable::Scrollable<'a, Message> {
    scrollable(content)
        .id(id)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(6)
                .scroller_width(6)
                .margin(2),
        ))
        .spacing(6)
        .style(scroll_style(*p))
}

/// A surface inside the main panel (lists, the summary blocks).
pub fn well(p: Palette) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(alpha(p.surface_container_low, 0.75))),
        border: Border {
            color: alpha(p.outline_variant, 0.55),
            width: 1.0,
            radius: radius::LG.into(),
        },
        ..container::Style::default()
    }
}

/// A well that shows the focus ring when the list inside it has focus.
pub fn well_ring(p: Palette, focused: bool) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(alpha(p.surface_container_low, 0.75))),
        border: Border {
            color: if focused {
                p.primary
            } else {
                alpha(p.outline_variant, 0.55)
            },
            width: if focused { 2.0 } else { 1.0 },
            radius: radius::LG.into(),
        },
        ..container::Style::default()
    }
}

/// One of several things picked, as a chip. Pressing it sends `on_press` and its close button
/// `on_remove`; `lead` fills it with a check mark, for the one that comes first.
pub fn pick_chip<'a>(
    s: String,
    lead: bool,
    on_press: Option<Message>,
    on_remove: Option<Message>,
    p: &Palette,
) -> Element<'a, Message> {
    let p = *p;
    let (bg, fg) = if lead {
        (p.secondary_container, p.on_secondary_container)
    } else {
        (p.surface_container_high, p.on_surface)
    };
    let style = move |_t: &Theme, status| button::Style {
        background: state_layer(None, fg, status),
        text_color: fg,
        border: Border::default().rounded(radius::FULL),
        shadow: Shadow::default(),
        snap: true,
    };
    let name = text(s).font(theme::SANS_MEDIUM).size(size::SM).color(fg);
    let body: Element<'a, Message> = if lead {
        row![icon(Icon::Check, 15.0, fg), name]
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
    } else {
        name.into()
    };
    let mut parts = row![
        button(body)
            .padding(Padding::from([5.0, 10.0]))
            .on_press_maybe(on_press)
            .style(style)
    ]
    .align_y(Alignment::Center);
    if let Some(remove) = on_remove {
        parts = parts.push(
            button(icon(Icon::Close, 15.0, alpha(fg, 0.8)))
                .padding(Padding::from([5.0, 8.0]))
                .on_press(remove)
                .style(style),
        );
    }
    container(parts)
        .style(move |_t| container::Style {
            background: Some(Background::Color(bg)),
            border: Border::default().rounded(radius::FULL),
            ..container::Style::default()
        })
        .into()
}

pub fn chip<'a>(
    ic: Option<Icon>,
    s: impl text::IntoFragment<'a>,
    fg: Color,
    bg: Color,
) -> Element<'a, Message> {
    let t = text(s).font(theme::SANS_MEDIUM).size(size::SM).color(fg);
    let content: Element<'a, Message> = match ic {
        Some(i) => row![icon(i, 15.0, fg), t]
            .spacing(6)
            .align_y(Alignment::Center)
            .into(),
        None => t.into(),
    };
    container(content)
        .padding(Padding::from([5.0, 10.0]))
        .style(move |_t| container::Style {
            background: Some(Background::Color(bg)),
            border: Border::default().rounded(radius::FULL),
            ..container::Style::default()
        })
        .into()
}

/// Track and knob of a switch, in logical pixels. Fixed: a switch never
/// stretches with its row.
pub const SWITCH: Size = Size::new(44.0, 26.0);

/// An on/off switch. Space or Enter (or a click) on it sends `on_toggle`.
#[allow(dead_code)] // for the first-boot setup screens
pub fn switch<'a>(
    on: bool,
    on_toggle: Option<Message>,
    focused: bool,
    p: &Palette,
) -> Element<'a, Message> {
    switch_anim(on, 1.0, on_toggle, focused, p)
}

/// `switch`, with the knob `progress` (0..=1) of the way to its new side.
pub fn switch_anim<'a>(
    on: bool,
    progress: f32,
    on_toggle: Option<Message>,
    focused: bool,
    p: &Palette,
) -> Element<'a, Message> {
    let p = *p;
    let enabled = on_toggle.is_some();
    let t = if on { progress } else { 1.0 - progress };
    let knob = canvas(SwitchKnob {
        t: t.clamp(0.0, 1.0),
        enabled,
        p,
    })
    .width(SWITCH.width)
    .height(SWITCH.height);
    let b = button(knob)
        .padding(0)
        .on_press_maybe(on_toggle)
        .style(|_t: &Theme, _s| button::Style {
            background: None,
            ..button::Style::default()
        });
    ring(b, focused, radius::FULL, &p)
}

struct SwitchKnob {
    /// 0 off, 1 on, anything between while it moves.
    t: f32,
    enabled: bool,
    p: Palette,
}

impl<Message> canvas::Program<Message> for SwitchKnob {
    type State = ();

    fn draw(
        &self,
        _s: &(),
        renderer: &Renderer,
        _th: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let p = self.p;
        let t = self.t;
        let a = if self.enabled { 1.0 } else { 0.38 };
        let mut frame = Frame::new(renderer, bounds.size());
        let (w, h) = (bounds.width, bounds.height);
        let track = Path::rounded_rectangle(Point::ORIGIN, bounds.size(), (h / 2.0).into());
        let off_bg = p.surface_container_highest;
        let mut bg = mix(off_bg, p.primary, t);
        if self.enabled && cursor.is_over(bounds) {
            bg = mix(bg, p.on_surface, STATE_HOVER);
        }
        frame.fill(&track, alpha(bg, a));
        if t < 1.0 {
            let inset = Path::rounded_rectangle(
                Point::new(0.75, 0.75),
                Size::new(w - 1.5, h - 1.5),
                ((h - 1.5) / 2.0).into(),
            );
            frame.stroke(
                &inset,
                Stroke::default()
                    .with_width(1.5)
                    .with_color(alpha(p.outline, a * (1.0 - t))),
            );
        }
        // The knob grows a little when on, like M3's handle.
        let r = 7.0 + 3.0 * t;
        let pad = h / 2.0;
        let x = pad + (w - 2.0 * pad) * t;
        let knob = Path::circle(Point::new(x, h / 2.0), r);
        frame.fill(&knob, alpha(mix(p.outline, p.on_primary, t), a));
        vec![frame.into_geometry()]
    }
}

/// A horizontal bar filled to `fraction`, with rounded ends.
pub fn bar<'a>(fraction: f32, height: f32, fill: Color, track: Color) -> Element<'a, Message> {
    let parts = 10_000u16;
    let filled = (fraction.clamp(0.0, 1.0) * parts as f32).round() as u16;
    let pill = move |c: Color| {
        move |_t: &Theme| container::Style {
            background: Some(Background::Color(c)),
            border: Border::default().rounded(height / 2.0),
            ..container::Style::default()
        }
    };
    let mut r = row![].height(height);
    if filled > 0 {
        r = r.push(
            container(Space::new())
                .width(Length::FillPortion(filled))
                .height(height)
                .style(move |_t: &Theme| container::Style {
                    background: Some(Background::Color(fill)),
                    border: Border::default().rounded(height / 2.0),
                    shadow: Shadow {
                        color: alpha(fill, 0.55),
                        offset: Vector::ZERO,
                        blur_radius: 12.0,
                    },
                    ..container::Style::default()
                }),
        );
    }
    if filled < parts {
        r = r.push(Space::new().width(Length::FillPortion(parts - filled)));
    }
    container(r).width(Length::Fill).style(pill(track)).into()
}

/// An indeterminate arc spinner. `t` is seconds since some fixed start.
pub struct Spinner {
    pub t: f32,
    pub color: Color,
    pub width: f32,
}

impl<Message> canvas::Program<Message> for Spinner {
    type State = ();

    fn draw(
        &self,
        _s: &(),
        renderer: &Renderer,
        _th: &Theme,
        bounds: Rectangle,
        _c: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let r = bounds.width.min(bounds.height) / 2.0 - self.width;
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        // Material's spinner: a rotating arc whose length breathes.
        // A faint full ring underneath, so the shortest arc still reads as
        // a spinner and not a stray comma.
        frame.stroke(
            &Path::circle(center, r),
            Stroke::default()
                .with_width(self.width)
                .with_color(alpha(self.color, 0.18)),
        );
        let rot = self.t * 2.0 * PI * 0.8;
        let sweep = PI * (0.45 + 0.6 * (0.5 + 0.5 * (self.t * PI * 1.2).sin()));
        let path = Path::new(|b| {
            b.arc(Arc {
                center,
                radius: r,
                start_angle: Radians(rot),
                end_angle: Radians(rot + sweep),
            });
        });
        frame.stroke(
            &path,
            Stroke::default()
                .with_width(self.width)
                .with_color(self.color)
                .with_line_cap(canvas::LineCap::Round),
        );
        vec![frame.into_geometry()]
    }
}

pub fn spinner<'a>(t: f32, px: f32, color: Color) -> Element<'a, Message> {
    canvas(Spinner {
        t,
        color,
        width: (px / 9.0).max(2.0),
    })
    .width(px)
    .height(px)
    .into()
}

/// A play triangle on a dark disc: the badge on a video wallpaper's thumbnail. Drawn, since the
/// bundled icon font has no play icon.
struct PlayBadge {
    fg: Color,
    bg: Color,
}

impl<Message> canvas::Program<Message> for PlayBadge {
    type State = ();

    fn draw(
        &self,
        _s: &(),
        renderer: &Renderer,
        _th: &Theme,
        bounds: Rectangle,
        _c: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let r = bounds.width.min(bounds.height) / 2.0;
        let c = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        frame.fill(&Path::circle(c, r), self.bg);
        // The triangle's centroid sits a little right of the disc's centre, so it looks centred.
        let s = r * 0.42;
        let tri = Path::new(|b| {
            // design-allow(size): corners in units of the triangle's `s`.
            b.move_to(Point::new(c.x - s * 0.7, c.y - s));
            // design-allow(size): corners in units of the triangle's `s`.
            b.line_to(Point::new(c.x + s * 1.1, c.y));
            // design-allow(size): corners in units of the triangle's `s`.
            b.line_to(Point::new(c.x - s * 0.7, c.y + s));
            b.close();
        });
        frame.fill(&tri, self.fg);
        vec![frame.into_geometry()]
    }
}

pub fn play_badge<'a>(px: f32, fg: Color, bg: Color) -> Element<'a, Message> {
    canvas(PlayBadge { fg, bg }).width(px).height(px).into()
}

/// The Show/Hide toggle for secrets: small, for the right of a label.
pub fn show_toggle<'a>(
    shown: bool,
    on_press: Message,
    focused: bool,
    p: &Palette,
) -> Element<'a, Message> {
    let pp = *p;
    let fg = p.primary;
    let b = button(
        row![
            icon(
                if shown {
                    Icon::VisibilityOff
                } else {
                    Icon::Visibility
                },
                16.0,
                fg
            ),
            text(if shown { "Hide" } else { "Show" })
                .font(theme::SANS_MEDIUM)
                .size(size::SM)
                .color(fg),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([1.0, 8.0]))
    .on_press(on_press)
    .style(move |_t: &Theme, status| button::Style {
        background: match status {
            button::Status::Hovered => Some(Background::Color(alpha(pp.primary, 0.10))),
            button::Status::Pressed => Some(Background::Color(alpha(pp.primary, 0.16))),
            _ => None,
        },
        text_color: fg,
        border: Border::default().rounded(radius::FULL),
        shadow: Shadow::default(),
        snap: true,
    });
    ring(b, focused, radius::FULL, &pp)
}

/// Under a confirmation field: nothing while it's still a prefix of the
/// original (you're typing it), "Doesn't match." once it can't be, a check
/// when it matches.
pub fn confirm_note<'a>(orig: &str, confirm: &str, p: &Palette) -> Option<Element<'a, Message>> {
    if confirm.is_empty() {
        None
    } else if confirm == orig {
        Some(
            row![
                icon(Icon::Check, 14.0, p.tertiary),
                text("Matches.").size(size::SM).color(p.tertiary)
            ]
            .spacing(4)
            .align_y(Alignment::Center)
            .into(),
        )
    } else if orig.starts_with(confirm) {
        None
    } else {
        Some(text("Doesn't match.").size(size::SM).color(p.error).into())
    }
}

/// The confirmation is wrong, not just unfinished.
pub fn mismatched(orig: &str, confirm: &str) -> bool {
    !confirm.is_empty() && confirm != orig && !orig.starts_with(confirm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::DARK;
    use iced::widget::Id;
    use iced_test::selector;
    use iced_test::simulator::Simulator;

    /// Lays `el` out in a window of `size` and returns the bounds of `id`.
    fn bounds(el: Element<'_, Message>, size: Size) -> Rectangle {
        let mut ui = Simulator::with_size(Default::default(), size, el);
        ui.find(selector::id("probe"))
            .expect("probe found")
            .bounds()
    }

    /// A row as tall as the window, with a switch in it: the switch keeps
    /// its own size (plus the focus ring's 3 px on each side).
    #[test]
    fn switch_never_stretches() {
        for size in [Size::new(500.0, 120.0), Size::new(1920.0, 1080.0)] {
            for on in [false, true] {
                let el = row![
                    Space::new().width(Length::Fill).height(Length::Fill),
                    container(switch(on, Some(Message::HoldEnd), true, &DARK)).id(Id::new("probe")),
                ]
                .height(Length::Fill)
                .align_y(Alignment::Center)
                .into();
                let b = bounds(el, size);
                assert_eq!(
                    (b.width, b.height),
                    (SWITCH.width + 6.0, SWITCH.height + 6.0),
                    "switch at {size:?}"
                );
            }
        }
    }

    #[test]
    fn a_set_reads_full_partial_or_empty_from_its_picks() {
        assert_eq!(Tick::of(0, 5), Tick::Empty);
        assert_eq!(Tick::of(3, 5), Tick::Partial);
        assert_eq!(Tick::of(5, 5), Tick::Full);
        assert_eq!(Tick::of(0, 0), Tick::Empty);
    }

    /// Text fields are one line tall however tall their row is, and as wide
    /// as they're given.
    #[test]
    fn text_field_height_is_fixed() {
        let look = || FieldLook {
            value: "Kuba Example",
            placeholder: "Your name",
            secret: false,
            caret: Some(Caret::all("Kuba Example")),
            blink_from: Instant::now(),
            locked: false,
            error: false,
            icon: Some(Icon::Search),
        };
        let mut heights = vec![];
        for size in [Size::new(400.0, 100.0), Size::new(1600.0, 900.0)] {
            let el = iced::widget::column![
                container(text_field(look(), None::<fn(Pointer) -> Message>, &DARK))
                    .id(Id::new("probe"))
            ]
            .height(Length::Fill)
            .into();
            let b = bounds(el, size);
            assert_eq!(b.width, size.width);
            heights.push(b.height);
        }
        assert_eq!(heights[0], heights[1]);
        assert!(heights[0] < 50.0, "{heights:?}");
    }
}
