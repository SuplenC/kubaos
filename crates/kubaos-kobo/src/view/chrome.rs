//! What frames the panel screens: the glass panel, the step rail beside it, the footer under each
//! screen and the key hints in it.

use iced::Background;
use iced::Border;
use iced::ContentFit;
use iced::Length;
use iced::Rectangle;
use iced::Shadow;
use iced::Size as WindowSize;
use iced::Vector;
use iced::widget;
use iced::widget::container;
use iced::widget::image;
use kubaos_ui::Look;
use kubaos_ui::Scale;
use kubaos_ui::color;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::step::StepState as RailState;
use kubaos_ui::component::step::step;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::motion::ease;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Shadow as ShadowToken;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::content;
use super::lockup;
use super::power::power_button;
use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::app::ProbeState;
use crate::app::Screen;
use crate::ui;
use crate::ui::El;

/// The room between a screen's body and the footer under it.
pub(super) const FOOTER_GAP: Space = Space::S16;

/// The glass panel's rectangle in a window `win` logical pixels big, at `scale`: inset from the
/// window's edges, and never wider than the rail beside two columns of text with the page's
/// insets, so lines don't get silly on a big output. Its edges are on device pixels.
pub fn panel_rect(win: WindowSize, scale: Scale) -> Rectangle {
    let column = f32::from(Size::Column.px());
    let margin = f32::from(if wide(win) { Space::S32 } else { Space::S16 }.px());
    let widest = f32::from(Size::Rail.px() + 2 * Space::S40.px()) + 2.0 * column;
    let width = (win.width - 2.0 * margin).min(widest);
    let height = win.height - 2.0 * margin;
    Rectangle {
        x: scale.snap((win.width - width) / 2.0),
        y: margin,
        width: scale.snap(width),
        height,
    }
}

/// Whether the window has room for two columns of text side by side.
pub(super) fn wide(win: WindowSize) -> bool {
    win.width >= 2.0 * f32::from(Size::Column.px())
}

pub(super) fn panel<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let r = panel_rect(app.size, look.scale);
    let corner = radius(Radius::Surface);
    let shadow = widget::container(widget::Space::new())
        .width(r.width)
        .height(r.height)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(c.surface)),
            border: Border::default().rounded(corner),
            shadow: Shadow {
                color: color::alpha(c.shadow, ShadowToken::Panel.opacity()),
                offset: Vector::new(0.0, f32::from(ShadowToken::Panel.y())),
                blur_radius: f32::from(ShadowToken::Panel.blur()),
            },
            ..container::Style::default()
        });
    let mut card = widget::stack![shadow];
    if let Some((_, glass)) = &app.glass {
        card = card.push(
            image(glass.clone())
                .width(r.width)
                .height(r.height)
                .content_fit(ContentFit::Fill)
                .border_radius(corner),
        );
    }
    let glass = color::alpha(c.surface, Opacity::Glass.value());
    let edge = look.outline_soft();
    let hairline = look.hairline();
    let surface = widget::container(row(look, Gap::NONE).push(rail(app, look)).push(content(app, look)))
        .width(r.width)
        .height(r.height)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(glass)),
            border: Border {
                color: edge,
                width: hairline,
                radius: corner.into(),
            },
            ..container::Style::default()
        });
    card = card.push(surface);
    widget::container(card)
        .padding(iced::Padding::ZERO.top(r.y).left(r.x))
        .into()
}

/// The symbol of a step in the rail.
fn step_icon(screen: Screen) -> Icon {
    match screen {
        Screen::Welcome => Icon::Bolt,
        Screen::Language => Icon::Language,
        Screen::Timezone => Icon::Schedule,
        Screen::Wifi => Icon::Wifi,
        Screen::Account => Icon::Person,
        Screen::Disk => Icon::HardDrive,
        Screen::Git => Icon::Code,
        Screen::Wallpapers => Icon::Wallpaper,
        Screen::Summary => Icon::ListAlt,
        Screen::Installing | Screen::Done | Screen::Failed => Icon::Download,
    }
}

fn rail<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let flow_ = app.flow();
    let cur = app.flow_index(app.screen);
    let locked = app.install.as_ref().is_some_and(|i| i.outcome.is_none());
    // Back to back, like a menu's items: the rail holds every step, its hints and the power
    // button down a 768 px window.
    let mut steps = stack(look, Gap::NONE);
    for (i, s) in flow_.iter().enumerate().skip(1) {
        let done = i < cur || app.screen == Screen::Done;
        let current = i == cur;
        let failed = current && app.screen == Screen::Failed;
        let clickable = !locked && i <= app.reached && !current && *s != Screen::Installing;
        let target = *s;
        let state = if current {
            RailState::Current
        } else if done {
            RailState::Done
        } else {
            RailState::Todo
        };
        steps = steps.push(
            step(s.rail_label())
                .icon(if failed { Icon::Error } else { step_icon(*s) })
                .state(state)
                .mark()
                .on_press_maybe(clickable.then_some(Message::Click(F::Step(target))))
                .view(look),
        );
    }

    let version = app
        .ready_probe()
        .and_then(|p| p.payload.as_ref())
        .map(|m| m.version.clone())
        .unwrap_or_default();
    // The brand, the hints and the footer line up with the steps' symbols, inside the rows' own
    // padding.
    let inside = Inset::x(Space::S12);
    let brand = stack(look, Space::S4).padding(inside).push(lockup(look)).push(text(
        format!("Kōbō · {} {version}", app.codename()),
        Type::Caption,
        c.on_surface_variant,
    ));
    let hints = ui::hint_column(look, &hints(app));
    let footer = row(look, Space::S8)
        .center()
        .padding(inside)
        .push(power_button(app, look))
        .push(fill());
    let footer = match app.battery {
        Some(b) => footer.push(ui::battery(look, b)),
        None => footer,
    };
    let rail_fill = color::alpha(c.surface_container_low, Opacity::Well.value());
    widget::container(
        stack(look, Gap::NONE)
            .height(Extent::Fill)
            .push(brand)
            .push(gap(Space::S20))
            .push(steps)
            .push(widget::Space::new().height(Length::Fill))
            .push(widget::container(hints).padding(inside.padding()))
            .push(gap(Space::S16))
            .push(footer),
    )
    // 12 at the sides, the rows' corners concentric with the rail's; 16 above and below, the
    // container padding at its width, so its ends hold no more air than a panel's.
    .padding(Inset::xy(Space::S12, Space::S16).padding())
    .width(f32::from(Size::Rail.px()))
    .height(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(rail_fill)),
        border: Border {
            radius: iced::border::Radius::default().left(radius(Radius::Surface)),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

/// The keys that do something on this screen, as (key, what it does). Every one of them works
/// (`tour.rs` tests check the important ones).
pub fn hints(app: &App) -> Vec<(&'static str, &'static str)> {
    let mut v = vec![];
    match app.screen {
        Screen::Welcome => v.push((
            "Enter",
            if matches!(app.probe, ProbeState::Failed(_)) {
                "try again"
            } else {
                "start"
            },
        )),
        Screen::Language if app.is_focused(F::KeymapQ) => v.extend([
            ("A–Z", "search"),
            ("↑ ↓", "move"),
            ("Space", "pick a layout"),
            ("Shift+Space", "make it the default"),
            ("Tab", "next field"),
        ]),
        Screen::Language | Screen::Timezone => {
            v.extend([("A–Z", "search"), ("↑ ↓", "pick"), ("Tab", "next field")]);
        }
        Screen::Wifi => v.extend([("↑ ↓", "pick"), ("Tab", "next field")]),
        Screen::Account | Screen::Git => v.push(("Tab", "next field")),
        Screen::Wallpapers => v.extend([("Space", "pick or drop"), ("← ↑ ↓ →", "move"), ("Tab", "next set")]),
        Screen::Disk => v.extend([
            ("↑ ↓", "pick a disk"),
            ("← →", "keep or erase"),
            ("Space", "switch"),
            ("Tab", "next field"),
        ]),
        Screen::Summary => v.push(("Enter", "hold to install")),
        Screen::Installing => v.push(("Enter", log_verb(app))),
        Screen::Failed => v.extend([
            (
                "Enter",
                match app.focus {
                    Some(F::SaveLog) => "save the log",
                    Some(F::ToggleLog) => log_verb(app),
                    Some(F::Back) => "back to the review",
                    _ => "try again",
                },
            ),
            ("Tab", "move"),
            ("Esc", "back to the review"),
        ]),
        Screen::Done => v.extend([
            ("Enter", if app.is_focused(F::Stay) { "stay here" } else { "reboot" }),
            ("Tab", "move"),
        ]),
    }
    if matches!(
        app.screen,
        Screen::Language
            | Screen::Timezone
            | Screen::Wifi
            | Screen::Account
            | Screen::Disk
            | Screen::Git
            | Screen::Wallpapers
    ) {
        v.extend([("Enter", "continue"), ("Esc", "back")]);
    } else if app.screen == Screen::Summary {
        v.push(("Esc", "back"));
    }
    v
}

fn log_verb(app: &App) -> &'static str {
    if app.log_open.0 { "hide the log" } else { "show the log" }
}

pub(super) fn footer<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let next_label = match app.screen {
        Screen::Wifi if app.form.wifi_ssid.is_none() => "Skip",
        Screen::Git if app.form.git_email.trim().is_empty() => "Skip",
        _ => "Continue",
    };
    let blocker = app.blocker();
    let mut r = row(look, Space::S16).center().push(
        button("Back")
            .kind(Kind::Text)
            .icon(Icon::ArrowBack)
            .on_press(Message::Click(F::Back))
            .focused(app.is_focused(F::Back))
            .view(look),
    );
    r = r.push(fill());
    if let Some(b) = blocker {
        // Enter while blocked: the reason flashes in the error colour, then settles back.
        let flash = app
            .nudged
            .map_or(0.0, |at| {
                1.0 - (app.now - at).as_secs_f32() / crate::app::NUDGE.as_secs_f32()
            })
            .clamp(0.0, 1.0);
        let colour = color::mix(c.on_surface_variant, c.error, ease(Easing::Standard, flash));
        r = r.push(
            row(look, Space::S8)
                .center()
                .push(icon(look, Icon::Error, IconSize::S16, colour))
                .push(text(b, Type::Small, colour)),
        );
    } else if let Some(note) = footer_note(app) {
        r = r.push(text(note, Type::Small, c.on_surface_variant));
    }
    if app.screen != Screen::Summary {
        r = r.push(
            button(next_label)
                .trailing_icon(Icon::ArrowForward)
                .on_press_maybe(blocker.is_none().then_some(Message::Click(F::Next)))
                .focused(app.is_focused(F::Next))
                .view(look),
        );
    }
    r.into()
}

/// What the footer says beside Continue when nothing blocks it: on Wallpapers, how many are
/// picked. It sits here rather than under the page's scroll, so the scroll runs down to the
/// footer with nothing loose under its cut. With none picked, the blocker says so instead.
fn footer_note(app: &App) -> Option<String> {
    if app.screen != Screen::Wallpapers {
        return None;
    }
    let sets = app.wallpaper_sets();
    let total: u32 = sets.iter().map(|s| s.images).sum();
    match app.form.wallpapers_picked(sets) {
        0 => None,
        n if n == total => Some(format!("All {total} picked.")),
        n => Some(format!("{n} of {total} picked.")),
    }
}
