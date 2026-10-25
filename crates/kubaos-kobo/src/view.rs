//! Everything on screen. Panel screens share the glass panel with the step
//! rail; Welcome and Done are full-bleed over the wallpaper. The power menu
//! opens over either. The battery sits beside the power button on both. First-boot setup is its own app (setup/).
//!
//! Copy follows docs/BRAND.md: plain facts first, attitude around them, at
//! most one swear per screen, none near the disk that gets erased or the
//! passphrase prompts.

use std::f32::consts::PI;
use std::sync::LazyLock;
use std::time::Duration;

use iced::border::Radius;
use iced::gradient::Linear;
use iced::widget::{
    Space, button, center, column, container, image, mouse_area, opaque, row, stack, svg, text,
};
use iced::{
    Alignment, Background, Border, Color, ContentFit, Element, Gradient, Length, Padding, Radians,
    Rectangle, Shadow, Size, Theme, Vector,
};

use kubaos_core::Level;
use kubaos_core::probe::DiskInfo;

use crate::app::{App, F, HOLD, List, Message, PowerAction, ProbeState, Screen, WALLPAPER_COLUMNS, ids, wallpaper_item};
use crate::data;
use crate::elide::elide;
use crate::grid::grid;
use crate::model::{self, DiskExt, DiskMode, human_bytes};
use crate::progress::{LOG_VIEW_LINES, Outcome, StepState, fmt_duration};
use crate::theme::{self, DARK, Icon, Palette, alpha, jp, mix, motion, radius, size, space};
use crate::widgets::{
    FieldLook, Kind, ROW_GAP, accent, bar, battery, body, btn, chip, field, field_with, header,
    icon, label, list_row, pick_chip, play_badge, quiet_btn, ring, scroll, spinner, switch_anim, text_field,
    title, well, well_ring,
};
use crate::widgets::HALO;
use crate::widgets::Tick;
use crate::widgets::check_badge;
use crate::widgets::halo;

type El<'a> = Element<'a, Message>;

/// The glass panel's rectangle for a window size, in logical pixels.
pub fn panel_rect(win: Size) -> Rectangle {
    let margin = if win.width >= 1100.0 { 36.0 } else { 16.0 };
    // Grows with big outputs (1920x1080 at scale 1 under cage), but never
    // so wide that lines get silly.
    let w = (win.width - 2.0 * margin).clamp(320.0, 1440.0);
    let h = (win.height - 2.0 * margin).clamp(320.0, 940.0);
    Rectangle {
        x: ((win.width - w) / 2.0).round(),
        y: ((win.height - h) / 2.0).round(),
        width: w,
        height: h,
    }
}

pub fn view(app: &App) -> El<'_> {
    let fg: El = if app.screen.in_panel() {
        panel(app)
    } else {
        hero(app)
    };
    let mut layers = stack![background(app), fg]
        .width(Length::Fill)
        .height(Length::Fill);
    // The panel screens carry the power button in the rail; the full-bleed ones in the corner.
    let mut corner = row![].spacing(space::MD).align_y(Alignment::Center);
    if app.backend.is_demo() {
        corner = corner.push(chip(
            Some(Icon::Bolt),
            "Demo",
            DARK.tertiary,
            alpha(DARK.tertiary_container, 0.45),
        ));
    }
    if !app.screen.in_panel() {
        if let Some(b) = app.battery {
            corner = corner.push(battery(b, &DARK));
        }
        corner = corner.push(power_button(app));
    }
    layers = layers.push(
        container(corner)
            .width(Length::Fill)
            .align_right(Length::Fill)
            .padding(14),
    );
    if app.power.is_some() {
        layers = layers.push(power_menu(app));
    }
    if let Some((msg, at)) = &app.toast {
        let t = (app.now - *at).as_secs_f32();
        let a = (t / 0.2).min(1.0) * ((4.0 - t) / 0.3).clamp(0.0, 1.0);
        let toast = container(
            text(msg.as_str())
                .size(size::MD)
                .color(alpha(DARK.surface, a)),
        )
        .padding(Padding::from([12.0, 18.0]))
        .style(move |_t| container::Style {
            background: Some(Background::Color(alpha(DARK.on_surface, a))),
            border: Border::default().rounded(radius::MD),
            ..container::Style::default()
        });
        layers = layers.push(
            container(toast)
                .center_x(Length::Fill)
                .align_bottom(Length::Fill)
                .padding(28),
        );
    }
    crate::pointer::idle(layers, app.pointer_idle).into()
}

fn background(app: &App) -> El<'_> {
    let base = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_t| container::Style::default().background(DARK.surface_container_lowest));
    let mut s = stack![base];
    if let Some((h, at)) = &app.wallpaper {
        let t = ((app.now - *at).as_secs_f32() / motion::SLOW.as_secs_f32()).min(1.0);
        let a = motion::bezier(motion::STANDARD, t);
        s = s.push(
            image(h.clone())
                .content_fit(ContentFit::Cover)
                .width(Length::Fill)
                .height(Length::Fill)
                .opacity(a),
        );
    }
    let hero = !app.screen.in_panel();
    let scrim = move |_t: &Theme| {
        let g = if hero {
            // Dark on the left where the text sits, the city glowing on the right.
            Linear::new(Radians(PI / 2.0))
                .add_stop(0.0, alpha(DARK.surface_container_lowest, 0.94))
                .add_stop(0.45, alpha(DARK.surface_container_lowest, 0.78))
                .add_stop(0.75, alpha(DARK.surface_container_lowest, 0.30))
                .add_stop(1.0, alpha(DARK.surface_container_lowest, 0.12))
        } else {
            Linear::new(Radians(PI))
                .add_stop(0.0, alpha(DARK.scrim, 0.30))
                .add_stop(1.0, alpha(DARK.scrim, 0.55))
        };
        container::Style::default().background(Background::Gradient(Gradient::Linear(g)))
    };
    s.push(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(scrim),
    )
    .into()
}

/// クバ + KubaOS, `h` logical pixels tall.
fn lockup<'a>(h: f32) -> El<'a> {
    // The mark's SVG has room for its glow; the glyphs fill ~72% of it.
    let mark = svg(svg::Handle::from_memory(theme::LOGO_MARK))
        .width(h * 1.5)
        .height(h * 1.5);
    let word = svg(svg::Handle::from_memory(theme::LOGO_WORDMARK))
        .height(h)
        .width(Length::Shrink);
    row![mark, word]
        .spacing(h * 0.05)
        .align_y(Alignment::Center)
        .into()
}

/// The neon sign's handle, made once: a handle made from bytes gets a new id every time, so one
/// made in `view` would be decoded and uploaded again on every frame.
static SIGN: LazyLock<image::Handle> = LazyLock::new(|| image::Handle::from_bytes(theme::LOGO_SIGN));

/// The neon sign, `w` logical pixels wide. Nearly half its height is the glow on the wall around
/// the glass.
fn sign<'a>(w: f32) -> El<'a> {
    image(SIGN.clone()).width(w).height(w / theme::LOGO_SIGN_ASPECT).into()
}

// ---------------------------------------------------------------- panel --

fn panel(app: &App) -> El<'_> {
    let r = panel_rect(app.size);
    let shadow = container(Space::new())
        .width(r.width)
        .height(r.height)
        .style(|_t| container::Style {
            background: Some(Background::Color(DARK.surface)),
            border: Border::default().rounded(radius::PANEL),
            shadow: Shadow {
                color: alpha(DARK.scrim, 0.55),
                offset: Vector::new(0.0, 18.0),
                blur_radius: 48.0,
            },
            ..container::Style::default()
        });
    let mut card = stack![shadow];
    if let Some((_, h)) = &app.glass {
        card = card.push(
            image(h.clone())
                .width(r.width)
                .height(r.height)
                .content_fit(ContentFit::Fill)
                .border_radius(radius::PANEL),
        );
    }
    let surface = container(row![rail(app), content(app)])
        .width(r.width)
        .height(r.height)
        .style(|_t| container::Style {
            background: Some(Background::Color(alpha(DARK.surface, 0.80))),
            border: Border {
                color: alpha(DARK.outline_variant, 0.7),
                width: 1.0,
                radius: radius::PANEL.into(),
            },
            ..container::Style::default()
        });
    card = card.push(surface);
    center(card).into()
}

fn rail(app: &App) -> El<'_> {
    let p = DARK;
    let flow = app.flow();
    let cur = app.flow_index(app.screen);
    let locked = app.install.as_ref().is_some_and(|i| i.outcome.is_none());
    let mut items = column![].spacing(2);
    for (i, s) in flow.iter().enumerate().skip(1) {
        let done = i < cur || app.screen == Screen::Done;
        let current = i == cur;
        let failed = current && app.screen == Screen::Failed;
        let clickable = !locked && i <= app.reached && !current && *s != Screen::Installing;
        let (lead_bg, lead): (Color, El) = if failed {
            (p.error, icon(Icon::Close, 15.0, p.on_error).into())
        } else if done {
            (
                alpha(p.primary, 0.16),
                icon(Icon::Check, 15.0, p.primary).into(),
            )
        } else if current {
            (
                p.primary,
                text(i.to_string())
                    .font(theme::SANS_SEMIBOLD)
                    .size(12)
                    .color(p.on_primary)
                    .into(),
            )
        } else {
            (
                Color::TRANSPARENT,
                text(i.to_string())
                    .font(theme::SANS_MEDIUM)
                    .size(12)
                    .color(p.outline)
                    .into(),
            )
        };
        let dot = container(lead)
            .center(24)
            .style(move |_t| container::Style {
                background: Some(Background::Color(lead_bg)),
                border: Border {
                    color: if lead_bg == Color::TRANSPARENT {
                        alpha(p.outline, 0.6)
                    } else {
                        Color::TRANSPARENT
                    },
                    width: 1.0,
                    radius: radius::FULL.into(),
                },
                ..container::Style::default()
            });
        let color = if current {
            p.on_secondary_container
        } else if done {
            p.on_surface
        } else {
            p.on_surface_variant
        };
        let lbl = text(s.rail_label())
            .font(if current {
                theme::SANS_SEMIBOLD
            } else {
                theme::SANS_MEDIUM
            })
            .size(size::MD)
            .color(if current || done {
                color
            } else {
                alpha(color, 0.7)
            });
        let target = *s;
        let item = button(row![dot, lbl].spacing(space::MD).align_y(Alignment::Center))
            .width(Length::Fill)
            .padding(Padding::from([8.0, 10.0]))
            .on_press_maybe(clickable.then_some(Message::Click(F::Step(target))))
            .style(move |_t: &Theme, status| {
                let bg = if current {
                    Some(alpha(p.secondary_container, 0.85))
                } else if matches!(status, button::Status::Hovered) {
                    Some(alpha(p.on_surface, theme::STATE_HOVER))
                } else {
                    None
                };
                button::Style {
                    background: bg.map(Background::Color),
                    text_color: color,
                    border: Border::default().rounded(radius::MD),
                    shadow: Shadow::default(),
                    snap: true,
                }
            });
        items = items.push(item);
    }

    let version = app
        .ready_probe()
        .and_then(|p| p.payload.as_ref())
        .map(|m| m.version.clone())
        .unwrap_or_default();
    let brand = column![
        lockup(24.0),
        text(format!("Kōbō · {} {version}", app.codename()))
            .font(theme::MONO)
            .size(size::XS)
            .color(p.outline),
    ]
    .spacing(2);

    let keys = column(hints(app).into_iter().map(|(k, what)| keycap_row(k, what))).spacing(6);

    container(
        column![
            brand,
            Space::new().height(space::XL),
            items,
            Space::new().height(Length::Fill),
            keys,
            Space::new().height(space::LG),
            rail_footer(app),
        ]
        .padding(Padding {
            top: 22.0,
            right: 14.0,
            bottom: 24.0,
            left: 10.0,
        }),
    )
    .width(if app.size.width >= 1100.0 { 236 } else { 200 })
    .height(Length::Fill)
    .style(|_t| container::Style {
        background: Some(Background::Color(alpha(
            DARK.surface_container_lowest,
            0.42,
        ))),
        border: Border {
            radius: Radius::default().left(radius::PANEL),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

/// The keys that do something on this screen, as (key, what it does).
/// Every one of them works (`tour.rs` tests check the important ones).
/// The power button, and the battery when the machine has one.
fn rail_footer(app: &App) -> El<'_> {
    let mut r =
        row![power_button(app), Space::new().width(Length::Fill)].align_y(Alignment::Center);
    if let Some(b) = app.battery {
        r = r.push(battery(b, &DARK));
    }
    r.into()
}

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
            ("Shift Space", "make it the default"),
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
            (
                "Enter",
                if app.is_focused(F::Stay) {
                    "stay here"
                } else {
                    "reboot"
                },
            ),
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
    if app.log_open.0 {
        "hide the log"
    } else {
        "show the log"
    }
}

/// Key hints in a row, for the full-bleed screens.
fn hint_row<'a>(app: &App, p: &Palette) -> El<'a> {
    let pp = *p;
    row(hints(app).into_iter().map(|(k, what)| {
        row![keycap(k, &pp), text(what).size(size::XS).color(pp.outline)]
            .spacing(space::SM)
            .align_y(Alignment::Center)
            .into()
    }))
    .spacing(space::XL)
    .into()
}

fn keycap<'a>(k: &'a str, p: &Palette) -> El<'a> {
    let p = *p;
    container(
        text(k)
            .font(theme::MONO)
            .size(10.5)
            .color(p.on_surface_variant),
    )
    .padding(Padding::from([2.0, 6.0]))
    .style(move |_t| container::Style {
        background: Some(Background::Color(alpha(p.surface_container_highest, 0.7))),
        border: Border {
            color: alpha(p.outline_variant, 0.9),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn keycap_row<'a>(k: &'a str, what: &'a str) -> El<'a> {
    row![
        keycap(k, &DARK),
        text(what).size(size::XS).color(DARK.outline)
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center)
    .padding(Padding::from([0.0, 14.0]))
    .into()
}

fn content(app: &App) -> El<'_> {
    let f = app.fade();
    let p = DARK.faded(f);
    let dir = if app.transition.1 { 1.0 } else { -1.0 };
    let dx = (1.0 - f) * 36.0 * dir;
    let body: El = match app.screen {
        Screen::Language => language(app, &p),
        Screen::Timezone => timezone(app, &p),
        Screen::Wifi => wifi(app, &p),
        Screen::Account => account(app, &p),
        Screen::Disk => disk(app, &p),
        Screen::Git => git(app, &p),
        Screen::Wallpapers => wallpapers(app, &p),
        Screen::Summary => summary(app, &p),
        Screen::Installing => installing(app, &p),
        Screen::Failed => failed(app, &p),
        _ => Space::new().into(),
    };
    let mut col = column![container(body).height(Length::Fill)].height(Length::Fill);
    if !matches!(app.screen, Screen::Installing | Screen::Failed) {
        col = col
            .push(Space::new().height(space::LG))
            .push(footer(app, &p));
    }
    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 34.0,
            right: (40.0 - dx).max(0.0),
            bottom: 26.0,
            left: (40.0 + dx).max(0.0),
        })
        .clip(true)
        .into()
}

fn footer<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let next_label = match app.screen {
        Screen::Wifi if app.form.wifi_ssid.is_none() => "Skip",
        Screen::Git if app.form.git_email.trim().is_empty() => "Skip",
        _ => "Continue",
    };
    let blocker = app.blocker();
    let mut r = row![btn(
        "Back",
        Some(Icon::ArrowBack),
        false,
        Kind::Text,
        Some(Message::Click(F::Back)),
        app.is_focused(F::Back),
        p
    )]
    .align_y(Alignment::Center)
    .spacing(space::LG);
    r = r.push(Space::new().width(Length::Fill));
    if let Some(b) = blocker {
        // Enter while blocked: the reason flashes in the error colour.
        let k = app
            .nudged
            .map(|at| 1.0 - (app.now - at).as_secs_f32() / crate::app::NUDGE.as_secs_f32())
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let c = mix(
            p.on_surface_variant,
            p.error,
            motion::bezier(motion::STANDARD, k),
        );
        r = r.push(
            row![
                icon(Icon::Error, 16.0, alpha(c, 0.9)),
                text(b).size(size::SM).color(c)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }
    if app.screen != Screen::Summary {
        r = r.push(btn(
            next_label,
            Some(Icon::ArrowForward),
            true,
            Kind::Primary,
            blocker.is_none().then_some(Message::Click(F::Next)),
            app.is_focused(F::Next),
            p,
        ));
    }
    r.into()
}

// -------------------------------------------------------------- screens --

fn input<'a>(app: &'a App, f: F, placeholder: &'a str, ic: Option<Icon>, p: &Palette) -> El<'a> {
    input_err(app, f, placeholder, ic, false, p)
}

/// `input` with a red border when `error` (the reason goes under it).
fn input_err<'a>(
    app: &'a App,
    f: F,
    placeholder: &'a str,
    ic: Option<Icon>,
    error: bool,
    p: &Palette,
) -> El<'a> {
    text_field(
        FieldLook {
            value: app.field_value(f),
            placeholder,
            secret: f.is_secret() && !app.show_secrets,
            caret: app.is_focused(f).then_some(app.caret),
            blink_from: app.focus_at,
            locked: false,
            error,
            icon: ic,
        },
        Some(move |ptr| Message::FieldPointer(f, ptr)),
        p,
    )
}

fn list_well<'a>(id: &'static str, rows: Vec<El<'a>>, focused: bool, p: &Palette) -> El<'a> {
    let inner: El = if rows.is_empty() {
        center(
            text("Nothing matches. Typo?")
                .size(size::MD)
                .color(p.on_surface_variant),
        )
        .into()
    } else {
        scroll(
            id,
            column(rows).spacing(ROW_GAP).padding(Padding {
                top: 0.0,
                right: 12.0,
                bottom: 0.0,
                left: 0.0,
            }),
            p,
        )
        .on_scroll(move |v| Message::Scrolled(id, v.absolute_offset().y, v.bounds().height))
        .height(Length::Fill)
        .into()
    };
    container(inner)
        .padding(6)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(well_ring(*p, focused))
        .into()
}

fn check_if<'a>(on: bool, p: &Palette) -> El<'a> {
    if on {
        icon(Icon::Check, 18.0, p.on_secondary_container).into()
    } else {
        Space::new().width(18).into()
    }
}

fn two_line<'a>(main: String, sub: String, selected: bool, p: &Palette) -> El<'a> {
    picked_line(main, sub, selected, selected, None, p)
}

/// A list row's text: `main`, then `tag` if any, then `sub` in mono. `selected` colours it for
/// the selected row's background, and `checked` shows the check mark.
fn picked_line<'a>(
    main: String,
    sub: String,
    selected: bool,
    checked: bool,
    tag: Option<&'a str>,
    p: &Palette,
) -> El<'a> {
    let (c1, c2) = if selected {
        (
            p.on_secondary_container,
            alpha(p.on_secondary_container, 0.75),
        )
    } else {
        (p.on_surface, p.on_surface_variant)
    };
    let mut line = row![elide(main, theme::SANS_MEDIUM, size::MD, c1).width(Length::Fill)]
        .align_y(Alignment::Center);
    if let Some(tag) = tag {
        line = line.push(Space::new().width(space::SM)).push(chip(
            None,
            tag,
            p.primary,
            alpha(p.primary, 0.16),
        ));
    }
    line.push(Space::new().width(space::MD))
        .push(text(sub).font(theme::MONO).size(size::XS).color(c2))
        .push(Space::new().width(space::MD))
        .push(check_if(checked, p))
        .into()
}

/// The keymap list: every layout, a check mark on the picked ones, the highlight on the row
/// Space acts on.
fn keymap_rows<'a>(app: &'a App, p: &Palette) -> Vec<El<'a>> {
    let at = app.keymap_row();
    let default = app.form.keymaps.first();
    app.list_items(List::Keymap)
        .into_iter()
        .map(|v| {
            let here = v == at;
            let picked = app.form.keymap_picked(&v);
            let tag = (default == Some(&v) && app.form.keymaps.len() > 1).then_some("Default");
            let name = data::keymap_name(&v).into_owned();
            list_row(
                picked_line(name, v.clone(), here, picked, tag, p),
                here,
                Message::Pick(List::Keymap, v),
                p,
            )
        })
        .collect()
}

/// The picked layouts, the default first. A click makes one the default; its close button
/// unpicks it, all but the last one.
fn keymap_chips<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let keymaps = &app.form.keymaps;
    let chips = keymaps.iter().enumerate().map(|(i, k)| {
        let lead = i == 0;
        pick_chip(
            data::keymap_name(k).into_owned(),
            lead,
            (!lead).then(|| Message::KeymapDefault(k.clone())),
            (keymaps.len() > 1).then(|| Message::KeymapDrop(k.clone())),
            p,
        )
    });
    let hint = if keymaps.len() > 1 {
        "The first is the default. Click one to make it the default."
    } else {
        "Space picks more. Shift+Space makes one the default."
    };
    column![
        row(chips)
            .spacing(space::SM)
            .wrap()
            .vertical_spacing(space::SM),
        text(hint).size(size::XS).color(p.on_surface_variant),
    ]
    .spacing(6)
    .into()
}

fn language<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let rows = |list: List, cur: &str| -> Vec<El<'a>> {
        app.list_items(list)
            .into_iter()
            .map(|v| {
                let sel = v == cur;
                let name = data::locale_name(&v).into_owned();
                list_row(
                    two_line(name, v.clone(), sel, p),
                    sel,
                    Message::Pick(list, v),
                    p,
                )
            })
            .collect()
    };
    let lang_focus = app.is_focused(F::LocaleQ);
    let key_focus = app.is_focused(F::KeymapQ);
    let col = |lbl, search: El<'a>, list: El<'a>| -> El<'a> {
        column![label(lbl, p), search, list]
            .spacing(10)
            .width(Length::FillPortion(1))
            .into()
    };
    column![
        header(
            jp::LANGUAGE,
            "Language and keyboard",
            "The language you want to read, and the layouts your fingers actually know. Type to search, arrows to move, Space to pick a layout.",
            p
        ),
        Space::new().height(space::XL),
        row![
            col(
                "Language",
                input(app, F::LocaleQ, "Search languages", Some(Icon::Search), p),
                list_well(ids::LOCALE_LIST, rows(List::Locale, &app.form.locale), lang_focus, p),
            ),
            column![
                col(
                    "Keyboard",
                    input(app, F::KeymapQ, "Search layouts", Some(Icon::Search), p),
                    list_well(ids::KEYMAP_LIST, keymap_rows(app, p), key_focus, p),
                ),
                keymap_chips(app, p),
                input(app, F::KeyboardTest, "Mash some keys to check the layout", Some(Icon::Keyboard), p),
            ]
            .spacing(10)
            .width(Length::FillPortion(1)),
        ]
        .spacing(space::XL)
        .height(Length::Fill),
    ]
    .into()
}

fn timezone<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let rows: Vec<El> = app
        .list_items(List::Timezone)
        .into_iter()
        .map(|tz| {
            let sel = tz == app.form.timezone;
            let (city, region) = data::split_tz(&tz);
            list_row(
                two_line(city, region, sel, p),
                sel,
                Message::Pick(List::Timezone, tz),
                p,
            )
        })
        .collect();
    let (city, region) = data::split_tz(&app.form.timezone);
    let pp = *p;
    let side = container(
        column![
            icon(Icon::Public, 40.0, p.primary),
            Space::new().height(space::LG),
            text(city)
                .font(theme::SANS_SEMIBOLD)
                .size(34)
                .color(p.on_surface),
            text(region).size(size::LG).color(p.on_surface_variant),
            Space::new().height(Length::Fill),
            text("Your clock, calendar and every timestamp in your logs follow this.")
                .size(size::SM)
                .color(p.on_surface_variant),
            Space::new().height(space::SM),
            text(app.form.timezone.clone())
                .font(theme::MONO)
                .size(size::SM)
                .color(p.outline),
        ]
        .spacing(2),
    )
    .padding(space::XXL)
    .width(300)
    .height(Length::Fill)
    .style(move |_t| container::Style {
        background: Some(Background::Gradient(Gradient::Linear(
            Linear::new(Radians(PI * 0.75))
                .add_stop(0.0, alpha(pp.primary_container, 0.35))
                .add_stop(1.0, alpha(pp.surface_container_low, 0.6)),
        ))),
        border: Border {
            color: alpha(pp.outline_variant, 0.55),
            width: 1.0,
            radius: radius::LG.into(),
        },
        ..container::Style::default()
    });
    column![
        header(
            jp::TIMEZONE,
            "Time zone",
            "Where your clock lives. Start typing a city.",
            p
        ),
        Space::new().height(space::XL),
        row![
            column![
                input(
                    app,
                    F::TzQ,
                    "Search cities or regions",
                    Some(Icon::Search),
                    p
                ),
                list_well(ids::TZ_LIST, rows, app.is_focused(F::TzQ), p)
            ]
            .spacing(10)
            .width(Length::Fill),
            side,
        ]
        .spacing(space::XL)
        .height(Length::Fill),
    ]
    .into()
}

fn wifi<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let Some(probe) = app.ready_probe() else {
        return Space::new().into();
    };
    let none = app.form.wifi_ssid.is_none();
    let fg_none = if none {
        p.on_secondary_container
    } else {
        p.on_surface
    };
    let mut rows: Vec<El> = vec![list_row(
        row![
            icon(Icon::WifiOff, 20.0, fg_none),
            elide(
                "Fuck Wi-Fi, I've got a cable",
                theme::SANS_MEDIUM,
                size::MD,
                fg_none
            )
            .width(Length::Fill),
            Space::new().width(WIFI_TRAIL),
            check_if(none, p),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
        none,
        Message::WifiNone,
        p,
    )];
    for w in &probe.wifi {
        let sel = app.form.wifi_ssid.as_deref() == Some(&w.ssid);
        let strength = match w.signal {
            75.. => Icon::Wifi4,
            50.. => Icon::Wifi3,
            25.. => Icon::Wifi2,
            _ => Icon::Wifi1,
        };
        let fg = if sel {
            p.on_secondary_container
        } else {
            p.on_surface
        };
        rows.push(list_row(
            row![
                icon(strength, 20.0, if sel { fg } else { p.on_surface_variant }),
                elide(w.ssid.clone(), theme::SANS_MEDIUM, size::MD, fg).width(Length::Fill),
                // Lock or "open" in one fixed column, so they line up.
                container(if w.secured {
                    El::from(icon(Icon::Lock, 16.0, if sel { fg } else { p.outline }))
                } else {
                    text("Open")
                        .size(size::XS)
                        .color(if sel { fg } else { p.outline })
                        .into()
                })
                .width(WIFI_TRAIL)
                .align_x(Alignment::Center),
                check_if(sel, p),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            sel,
            Message::Pick(List::Wifi, w.ssid.clone()),
            p,
        ));
    }
    let mut col = column![
        header(
            jp::NETWORK,
            "Wi-Fi",
            "Optional. I don't need the internet to install; this only saves a network for your first boot.",
            p
        ),
        Space::new().height(space::XL),
        container(list_well(
            ids::WIFI_LIST,
            rows,
            app.is_focused(F::WifiList),
            p
        ))
        .height(Length::Fill),
    ];
    if app.wifi_secured(app.form.wifi_ssid.as_deref()) {
        col = col.push(Space::new().height(space::LG)).push(secret_field(
            app,
            "Wi-Fi password",
            F::WifiPsk,
            p,
        ));
    }
    col.into()
}

/// The Show/Hide toggle for secrets: small, on the right of a field's label.
fn show_toggle<'a>(app: &'a App, p: &Palette) -> El<'a> {
    crate::widgets::show_toggle(
        app.show_secrets,
        Message::Click(F::ShowSecrets),
        app.is_focused(F::ShowSecrets),
        p,
    )
}

use crate::widgets::{confirm_note, mismatched};

/// Width of the Wi-Fi rows' lock/"Open" column.
const WIFI_TRAIL: f32 = 40.0;

fn secret_field<'a>(app: &'a App, lbl: &'a str, f: F, p: &Palette) -> El<'a> {
    field_with(
        lbl,
        Some(show_toggle(app, p)),
        input(app, f, "", None, p),
        None,
        p,
    )
}

fn strength_meter<'a>(pw: &str, p: &Palette) -> El<'a> {
    let s = model::strength(pw);
    let color = match s {
        0 => p.outline_variant,
        1 => p.error,
        2 => mix(p.error, p.primary, 0.5),
        3 => p.primary,
        _ => p.tertiary,
    };
    let mut segs = row![].spacing(4).width(112);
    for i in 1..=4u8 {
        let c = if s >= i {
            color
        } else {
            alpha(p.outline_variant, 0.8)
        };
        segs = segs.push(
            container(Space::new())
                .width(Length::Fill)
                .height(4)
                .style(move |_t| {
                    container::Style::default()
                        .background(c)
                        .border(Border::default().rounded(2))
                }),
        );
    }
    let label = match s {
        1 => "Weak. A toddler could guess it.",
        _ => model::strength_label(s),
    };
    row![segs, text(label).size(size::SM).color(color)]
        .spacing(space::MD)
        .align_y(Alignment::Center)
        .into()
}

fn account<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let f = &app.form;
    let mismatch = mismatched(&f.password, &f.password_confirm);
    let payload = app.ready_probe().and_then(|p| p.payload.as_ref());
    let baked = payload.is_some_and(|m| m.username == f.username);
    let bad_name =
        !f.username.is_empty() && !kubaos_core::engine::validate::valid_username(&f.username);
    let username = input_err(app, F::Username, "you", None, bad_name, p);
    let username_note = if f.username.is_empty() {
        "Your login and home folder.".to_string()
    } else if bad_name {
        "Lowercase: a-z first, then a-z, 0-9, - or _.".to_string()
    } else if baked {
        format!(
            "/home/{}. The image's home config is built for {} already.",
            f.username, f.username
        )
    } else {
        format!(
            "/home/{}. The install builds the home config for {}: a few seconds more.",
            f.username, f.username
        )
    };
    let bad_host =
        !f.hostname.is_empty() && !kubaos_core::engine::validate::valid_hostname(&f.hostname);
    let hostname = input_err(app, F::Hostname, "this-machine", None, bad_host, p);
    let note = |s: &'static str, c: Color| -> El<'a> { text(s).size(size::SM).color(c).into() };
    let hostname_note = if bad_host {
        note(model::HOSTNAME_RULE, p.error)
    } else if f.hostname.is_empty() || f.username.is_empty() {
        note(
            "This machine's name on the network and in your prompt.",
            p.outline,
        )
    } else {
        row![
            note("You'll be", p.outline),
            text(format!("{}@{}", f.username, f.hostname))
                .font(theme::MONO)
                .size(size::SM)
                .color(p.on_surface_variant),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    };
    let half = |e: El<'a>| container(e).width(Length::FillPortion(1));
    let names = row![
        half(field(
            "Full name",
            input(app, F::FullName, "Your name", None, p),
            None,
            p
        )),
        half(field(
            "Username",
            username,
            Some(
                text(username_note)
                    .size(size::SM)
                    .color(if bad_name { p.error } else { p.outline })
                    .into()
            ),
            p
        )),
    ]
    .spacing(space::XL);
    let host = row![
        half(field("Hostname", hostname, Some(hostname_note), p)),
        half(Space::new().into()),
    ]
    .spacing(space::XL);
    let passwords = row![
        half(field(
            "Password",
            column![
                input(app, F::Password, "", None, p),
                strength_meter(&f.password, p)
            ]
            .spacing(space::SM),
            None,
            p
        )),
        half(field_with(
            "Confirm password",
            Some(show_toggle(app, p)),
            input_err(app, F::PasswordConfirm, "", None, mismatch, p),
            confirm_note(&f.password, &f.password_confirm, p),
            p
        )),
    ]
    .spacing(space::XL);
    // The title stays put; only the fields scroll (src/reveal.rs).
    column![
        header(
            jp::ACCOUNT,
            "You",
            "Your login, this machine's name and your sudo password. Pick one you'll still remember at 2am.",
            p
        ),
        Space::new().height(space::XXL),
        scroll(
            ids::ACCOUNT_SCROLL,
            column![
                container(names).id(ids::REVEAL_NAMES),
                Space::new().height(space::XL),
                container(host).id(ids::REVEAL_HOST),
                Space::new().height(space::XL),
                container(passwords).id(ids::REVEAL_PASSWORDS),
            ]
            .padding(Padding::default().right(14)),
            p,
        )
        .height(Length::Fill),
    ]
    .into()
}

/// Some disks have no model string (virtio, SD cards): name them by kind.
fn disk_model(d: &DiskInfo) -> &str {
    let m = d.model.trim();
    let dev = d.path.trim_start_matches("/dev/");
    match m {
        "" if dev.starts_with("vd") || dev.starts_with("xvd") => "Virtual disk",
        "" if dev.starts_with("mmcblk") => "SD card",
        "" if dev.starts_with("nvme") => "NVMe disk",
        "" => "Unnamed disk",
        m => m,
    }
}

/// Colors for partitions in the layout bar.
fn part_color(hint: Option<&str>, i: usize, p: &Palette) -> Color {
    match hint {
        Some(h) if h.contains("Windows") => mix(p.tertiary_container, p.surface_bright, 0.35),
        Some(h) if h.contains("EFI") => p.secondary_container,
        _ => [p.surface_bright, p.surface_container_highest][i % 2],
    }
}

fn disk_row<'a>(d: &'a DiskInfo, sel: bool, small: bool, p: &Palette) -> El<'a> {
    let pp = *p;
    let fg = if sel {
        p.on_secondary_container
    } else {
        p.on_surface
    };
    let mut chips = row![].spacing(6);
    if let Some(os) = d.other_os() {
        chips = chips.push(chip(
            None,
            os.to_string(),
            p.tertiary,
            alpha(p.tertiary_container, 0.35),
        ));
    }
    if small {
        chips = chips.push(chip(
            None,
            "Too small",
            p.error,
            alpha(p.error_container, 0.35),
        ));
    }
    if d.removable {
        chips = chips.push(chip(
            None,
            "USB",
            p.on_surface_variant,
            alpha(p.surface_container_highest, 0.9),
        ));
    }
    let content = row![
        icon(
            Icon::HardDrive,
            26.0,
            if sel {
                p.on_secondary_container
            } else {
                p.primary
            }
        ),
        column![
            elide(
                disk_model(d).to_string(),
                theme::SANS_SEMIBOLD,
                size::MD,
                fg
            )
            .width(Length::Fill),
            text(format!("{} · {}", d.path, human_bytes(d.size_bytes)))
                .font(theme::MONO)
                .size(size::XS)
                .color(alpha(fg, 0.7)),
        ]
        .spacing(2)
        .width(Length::Fill),
        chips,
        check_if(sel, p),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);
    button(content)
        .width(Length::Fill)
        .padding(Padding::from([10.0, 14.0]))
        .on_press(Message::Pick(List::Disk, d.path.clone()))
        .style(move |_t: &Theme, status| {
            let bg = if sel {
                pp.secondary_container
            } else {
                match status {
                    button::Status::Hovered => alpha(pp.on_surface, theme::STATE_HOVER),
                    _ => Color::TRANSPARENT,
                }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: pp.on_surface,
                border: Border::default().rounded(radius::MD),
                shadow: Shadow::default(),
                snap: true,
            }
        })
        .into()
}

/// The disk after the install: kept partitions, then ours.
fn layout_bar<'a>(d: &'a DiskInfo, mode: DiskMode, encrypt: bool, p: &Palette) -> El<'a> {
    let total = d.size_bytes.max(1) as f32;
    let portion = |b: u64| ((b as f32 / total) * 1000.0).max(14.0) as u16;
    let ours_label = format!("KubaOS{}", if encrypt { " · encrypted" } else { "" });
    let pp = *p;
    let seg = move |c: Color, w: u16, first: bool, last: bool, glow: bool| -> El<'a> {
        let mut r = Radius::default();
        if first {
            r = r.left(radius::SM);
        }
        if last {
            r = r.right(radius::SM);
        }
        container(Space::new())
            .width(Length::FillPortion(w))
            .height(36)
            .style(move |_t| container::Style {
                background: Some(Background::Color(c)),
                border: Border {
                    radius: r,
                    ..Border::default()
                },
                shadow: if glow {
                    Shadow {
                        color: alpha(pp.primary, 0.35),
                        offset: Vector::ZERO,
                        blur_radius: 16.0,
                    }
                } else {
                    Shadow::default()
                },
                ..container::Style::default()
            })
            .into()
    };
    let mut bar = row![].spacing(3);
    let mut legend = row![].spacing(space::LG);
    let legend_item = |c: Color, name: String, sz: u64| -> El<'a> {
        row![
            container(Space::new())
                .width(10)
                .height(10)
                .style(move |_t| container::Style::default()
                    .background(c)
                    .border(Border::default().rounded(3))),
            text(name).size(size::SM).color(pp.on_surface),
            text(human_bytes(sz))
                .font(theme::MONO)
                .size(size::XS)
                .color(pp.on_surface_variant),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
    };
    match mode {
        DiskMode::Whole => {
            bar = bar.push(seg(p.primary, 1000, true, true, true));
            legend = legend.push(legend_item(p.primary, ours_label, d.size_bytes));
        }
        DiskMode::FreeSpace => {
            let free = d.largest_free();
            let mut items: Vec<(u64, Option<usize>)> = vec![];
            // Partitions in order, with our region slotted after the last
            // partition that ends before it (good enough for a picture).
            let mut placed = false;
            let mut offset = 0u64;
            for (i, part) in d.partitions.iter().enumerate() {
                if let (Some(r), false) = (free, placed)
                    && offset >= r.start
                {
                    items.push((r.end - r.start, None));
                    placed = true;
                }
                offset += part.size_bytes;
                items.push((part.size_bytes, Some(i)));
            }
            if let (Some(r), false) = (free, placed) {
                items.push((r.end - r.start, None));
            }
            let n = items.len();
            for (k, (sz, which)) in items.into_iter().enumerate() {
                match which {
                    Some(i) => {
                        let part = &d.partitions[i];
                        let c = part_color(part.os_hint.as_deref(), i, p);
                        bar = bar.push(seg(c, portion(sz), k == 0, k + 1 == n, false));
                        if sz as f32 / total > 0.05
                            || part.os_hint.as_deref().is_some_and(|h| h.contains("EFI"))
                        {
                            let name =
                                part.label
                                    .clone()
                                    .or(part.os_hint.clone())
                                    .unwrap_or_else(|| {
                                        part.path.rsplit('/').next().unwrap_or("").to_string()
                                    });
                            legend = legend.push(legend_item(c, name, sz));
                        }
                    }
                    None => {
                        bar = bar.push(seg(p.primary, portion(sz), k == 0, k + 1 == n, true));
                        legend = legend.push(legend_item(p.primary, ours_label.clone(), sz));
                    }
                }
            }
        }
    }
    column![bar, legend].spacing(space::MD).into()
}

fn mode_card<'a>(
    ic: Icon,
    t: &'a str,
    sub: String,
    selected: bool,
    on_press: Option<Message>,
    p: &Palette,
) -> El<'a> {
    let pp = *p;
    let enabled = on_press.is_some();
    let a = if enabled { 1.0 } else { 0.4 };
    let fg = if selected {
        p.on_secondary_container
    } else {
        p.on_surface
    };
    button(
        row![
            icon(
                if selected {
                    Icon::RadioChecked
                } else {
                    Icon::RadioUnchecked
                },
                20.0,
                alpha(if selected { p.primary } else { p.outline }, a)
            ),
            column![
                row![
                    icon(ic, 18.0, alpha(fg, a)),
                    text(t)
                        .font(theme::SANS_SEMIBOLD)
                        .size(size::MD)
                        .color(alpha(fg, a))
                ]
                .spacing(space::SM)
                .align_y(Alignment::Center),
                text(sub).size(size::SM).color(alpha(
                    if selected { fg } else { p.on_surface_variant },
                    0.85 * a
                )),
            ]
            .spacing(4),
        ]
        .spacing(space::MD),
    )
    .width(Length::FillPortion(1))
    .padding(space::LG)
    .on_press_maybe(on_press)
    .style(move |_t: &Theme, status| {
        let bg = if selected {
            alpha(pp.secondary_container, 0.85)
        } else if matches!(status, button::Status::Hovered) {
            alpha(pp.on_surface, theme::STATE_HOVER)
        } else {
            alpha(pp.surface_container_low, 0.6)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pp.on_surface,
            border: Border {
                color: if selected {
                    alpha(pp.primary, 0.8)
                } else {
                    alpha(pp.outline_variant, 0.7)
                },
                width: 1.0,
                radius: radius::LG.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    })
    .into()
}

fn setting_row<'a>(ic: Icon, t: &'a str, sub: &'a str, control: El<'a>, p: &Palette) -> El<'a> {
    row![
        icon(ic, 22.0, p.primary),
        column![
            text(t)
                .font(theme::SANS_SEMIBOLD)
                .size(size::MD)
                .color(p.on_surface),
            text(sub).size(size::SM).color(p.on_surface_variant)
        ]
        .spacing(2)
        .width(Length::Fill),
        control,
    ]
    .spacing(space::LG)
    .align_y(Alignment::Center)
    .into()
}

fn disk<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let Some(probe) = app.ready_probe() else {
        return Space::new().into();
    };
    let f = &app.form;
    let mut disks = column![].spacing(2);
    for d in probe.disks.iter().filter(|d| d.is_target()) {
        disks = disks.push(disk_row(
            d,
            f.disk.as_deref() == Some(&d.path),
            !d.fits(probe),
            p,
        ));
    }
    let disks = container(disks)
        .id(ids::REVEAL_DISKS)
        .padding(6)
        .style(well_ring(*p, app.is_focused(F::DiskList)));

    let mut col = column![disks];
    if let Some(d) = f.selected_disk(probe) {
        let mode = f.disk_mode.unwrap_or(DiskMode::Whole);
        let free = d.largest_free().map(|r| r.end - r.start).unwrap_or(0);
        let others = d.other_os().map(str::to_string);
        let free_sub = if d.fits_dual_boot(probe) {
            format!(
                "Keep {} and use the {} of free space.",
                others.as_deref().unwrap_or("everything"),
                human_bytes(free)
            )
        } else if free > 0 {
            format!("Only {} free. Shrink something first.", human_bytes(free))
        } else {
            "No free space on this disk.".to_string()
        };
        let whole_sub = match d.partitions.len() {
            _ if !d.fits(probe) => format!(
                "Too small: KubaOS needs {}.",
                human_bytes(model::needed(probe))
            ),
            0 => "The disk is empty anyway.".to_string(),
            n => format!(
                "Wipes all {n} partition{}{}.",
                if n == 1 { "" } else { "s" },
                others
                    .map(|o| format!(", {o} included"))
                    .unwrap_or_default()
            ),
        };
        let modes = row![
            mode_card(
                Icon::Storage,
                "Install alongside",
                free_sub,
                mode == DiskMode::FreeSpace,
                d.fits_dual_boot(probe)
                    .then_some(Message::DiskModePick(DiskMode::FreeSpace)),
                p
            ),
            mode_card(
                Icon::Warning,
                "Erase the whole disk",
                whole_sub,
                mode == DiskMode::Whole,
                Some(Message::DiskModePick(DiskMode::Whole)),
                p
            ),
        ]
        .spacing(space::LG);
        col = col
            .push(Space::new().height(space::LG))
            .push(
                container(ring(modes, app.is_focused(F::DiskMode), radius::LG, p))
                    .id(ids::REVEAL_MODE),
            )
            .push(Space::new().height(space::LG))
            .push(label("After the install", p))
            .push(Space::new().height(space::SM))
            .push(layout_bar(d, mode, f.encrypt, p));
    }

    let mut enc = column![
        container(setting_row(
            if f.encrypt {
                Icon::Lock
            } else {
                Icon::LockOpen
            },
            "Encrypt the disk",
            "LUKS2. Lose the laptop, keep your secrets. You type the passphrase once at boot.",
            switch_anim(
                f.encrypt,
                app.switch_progress(F::Encrypt),
                Some(Message::Click(F::Encrypt)),
                app.is_focused(F::Encrypt),
                p
            ),
            p
        ))
        .id(ids::REVEAL_ENCRYPT)
    ]
    .spacing(space::LG);
    if f.encrypt {
        enc = enc.push(
            container(setting_row(
                Icon::Key,
                "Use my account password",
                "One password for boot and login.",
                switch_anim(
                    f.luks_same_as_password,
                    app.switch_progress(F::LuksSame),
                    Some(Message::Click(F::LuksSame)),
                    app.is_focused(F::LuksSame),
                    p,
                ),
                p,
            ))
            .id(ids::REVEAL_LUKS_SAME),
        );
        if !f.luks_same_as_password {
            let mismatch = mismatched(&f.luks, &f.luks_confirm);
            enc = enc.push(
                container(
                    row![
                        container(field(
                            "Encryption passphrase",
                            column![input(app, F::Luks, "", None, p), strength_meter(&f.luks, p)]
                                .spacing(space::SM),
                            None,
                            p
                        ))
                        .width(Length::FillPortion(1)),
                        container(field_with(
                            "Confirm the passphrase",
                            Some(show_toggle(app, p)),
                            input_err(app, F::LuksConfirm, "", None, mismatch, p),
                            confirm_note(&f.luks, &f.luks_confirm, p),
                            p
                        ))
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(space::LG),
                )
                .id(ids::REVEAL_LUKS),
            );
        }
    }
    col = col
        .push(Space::new().height(space::LG))
        .push(container(enc).padding(space::LG).style(well(*p)));
    // The title stays put; only what's under it scrolls (src/reveal.rs).
    column![
        header(
            jp::DISK,
            "Where it goes",
            "Pick a disk, then decide whether whatever's on it gets to live.",
            p
        ),
        Space::new().height(space::XL),
        scroll(
            ids::DISK_SCROLL,
            col.padding(Padding::default().right(14)),
            p
        )
        .height(Length::Fill),
    ]
    .into()
}

fn git<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let f = &app.form;
    // Not while it's still being typed: only once you've left the field,
    // or pressed Enter on it.
    let email = f.git_email.trim();
    let email_bad = !email.is_empty()
        && !model::looks_like_email(email)
        && (!app.is_focused(F::GitEmail) || app.nudged.is_some());
    let pp = *p;
    let name = if f.git_name.trim().is_empty() {
        "…"
    } else {
        f.git_name.trim()
    };
    let email = if f.git_email.trim().is_empty() {
        "…"
    } else {
        f.git_email.trim()
    };
    let k = |s: &'a str| text(s).font(theme::MONO).size(size::MD).color(pp.tertiary);
    let v = |s: String| {
        text(s)
            .font(theme::MONO)
            .size(size::MD)
            .color(pp.on_surface)
    };
    let preview = container(
        column![
            text("~/.gitconfig")
                .font(theme::MONO)
                .size(size::XS)
                .color(p.outline),
            Space::new().height(space::SM),
            text("[user]")
                .font(theme::MONO)
                .size(size::MD)
                .color(p.primary),
            row![k("    name"), v(format!(" = {name}"))],
            row![k("    email"), v(format!(" = {email}"))],
        ]
        .spacing(2),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .style(move |_t| container::Style {
        background: Some(Background::Color(alpha(pp.surface_container_lowest, 0.7))),
        border: Border {
            color: alpha(pp.outline_variant, 0.55),
            width: 1.0,
            radius: radius::LG.into(),
        },
        ..container::Style::default()
    });
    column![
        header(
            jp::GIT,
            "Git identity",
            "Who your commits say they're from. No email means you deal with it later.",
            p
        ),
        Space::new().height(space::XXL),
        row![
            container(field(
                "Name",
                input(app, F::GitName, "Your name", None, p),
                None,
                p
            ))
            .width(Length::FillPortion(1)),
            container(field(
                "Email",
                input_err(app, F::GitEmail, "you@example.com", None, email_bad, p),
                email_bad.then(|| {
                    text("That isn't an email address.")
                        .size(size::SM)
                        .color(p.error)
                        .into()
                }),
                p
            ))
            .width(Length::FillPortion(1)),
        ]
        .spacing(space::XL),
        Space::new().height(space::XL),
        preview,
    ]
    .into()
}

/// `cyberpunk` -> "Cyberpunk", `neo-tokyo` -> "Neo tokyo".
fn set_name(id: &str) -> String {
    let s = id.replace(['-', '_'], " ");
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// `kabukicho-red-gate.jpg` -> "kabukicho red gate", as the shell's picker labels it.
fn wallpaper_label(name: &str) -> String {
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    stem.replace(['-', '_'], " ")
}

/// The Summary's line for `[wallpapers]`: the sets picked whole by name, the others with how many
/// of theirs.
pub fn wallpapers_summary(
    w: Option<&kubaos_core::config::Wallpapers>,
    sets: &[kubaos_core::probe::WallpaperSet],
) -> String {
    let Some(w) = w else {
        return "Every set".into();
    };
    if w.sets.is_empty() && w.files.is_empty() {
        return "Only your own".into();
    }
    let mut parts = Vec::new();
    for set in sets {
        let picked = w
            .files
            .iter()
            .filter(|f| f.split_once('/').is_some_and(|(id, _)| id == set.id))
            .count();
        if w.sets.contains(&set.id) {
            parts.push(set_name(&set.id));
        } else if picked > 0 {
            parts.push(format!("{} ({picked} of {})", set_name(&set.id), set.files.len()));
        }
    }
    parts.join(", ")
}

/// A wallpaper card's shape: the photo is cropped to it, whatever its own.
const CARD_ASPECT: f32 = 16.0 / 10.0;

/// Space between the cards, both ways.
const CARD_GAP: f32 = space::LG;

/// How far the label and badges sit from the card's edge: the corner radius, so text never runs
/// into a rounded corner.
const CARD_INSET: f32 = radius::MD;

/// How far a card's glow reaches past its edge. The list is padded by it on the left, where the
/// scroll area would otherwise cut the first column's glow off straight.
const CARD_GLOW: f32 = 10.0;

fn wallpapers<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let sets = app.wallpaper_sets();
    let mut sections = column![].spacing(space::XXXL);
    for (s, set) in sets.iter().enumerate() {
        let cards = (0..set.files.len()).map(|i| wallpaper_card(app, s, i, p));
        let grid = iced::widget::Grid::with_children(cards)
            .columns(WALLPAPER_COLUMNS)
            .spacing(CARD_GAP - 2.0 * HALO)
            .height(iced::widget::grid::aspect_ratio(CARD_ASPECT, 1.0));
        sections = sections.push(column![wallpaper_set_header(app, s, set, p), grid].spacing(space::SM));
    }
    let total: u32 = sets.iter().map(|s| s.images).sum();
    let picked = app.form.wallpapers_picked(sets);
    let note = match picked {
        0 => "None picked. The picker needs at least one to start on.".to_string(),
        n if n == total => format!("All {total} picked."),
        n => format!("{n} of {total} picked."),
    };
    column![
        header(
            jp::WALLPAPERS,
            "Wallpapers",
            "Pick the ones the wallpaper picker shows; a set's name takes all of it. Every one gets installed either way (later: ~/.config/kubaos/desktop/wallpapers/sets). Your own images go in ~/Pictures/wallpapers.",
            p
        ),
        Space::new().height(space::XL),
        scroll(
            ids::WALLPAPER_SCROLL,
            container(sections).padding(Padding::ZERO.left(CARD_GLOW - HALO).right(space::LG).bottom(space::LG)),
            p
        )
        .height(Length::Fill),
        Space::new().height(space::MD),
        text(note).size(size::SM).color(p.on_surface_variant),
    ]
    .into()
}

/// A set's header: its check, its name and a pill with how many of its wallpapers are picked.
/// Space or a click picks the rest of it, or drops it when it's all picked.
fn wallpaper_set_header<'a>(
    app: &'a App,
    s: usize,
    set: &'a kubaos_core::probe::WallpaperSet,
    p: &Palette,
) -> El<'a> {
    let on = app.form.wallpapers_on(set);
    let tick = Tick::of(on, set.images);
    let pp = *p;
    let count = match (tick, set.images) {
        (Tick::Full | Tick::Empty, 1) => "1 wallpaper".to_string(),
        (Tick::Full, n) => format!("all {n}"),
        (Tick::Empty, n) => format!("{n} wallpapers"),
        (Tick::Partial, n) => format!("{on} of {n}"),
    };
    let (pill_fg, pill_bg) = match tick {
        Tick::Full => (p.primary, alpha(p.primary, 0.14)),
        Tick::Partial => (p.on_surface, alpha(p.on_surface, 0.08)),
        Tick::Empty => (p.on_surface_variant, alpha(p.on_surface, 0.05)),
    };
    let pill = container(text(count).font(theme::SANS_MEDIUM).size(size::SM).color(pill_fg))
        .padding([2.0, space::MD - 2.0])
        .style(move |_t| container::Style {
            background: Some(Background::Color(pill_bg)),
            border: Border::default().rounded(radius::FULL),
            ..container::Style::default()
        });
    let b = button(
        row![
            check_badge(tick, 20.0, p.outline, Color::TRANSPARENT, p),
            text(set_name(&set.id))
                .font(theme::SANS_SEMIBOLD)
                .size(size::LG)
                .color(p.on_surface),
            pill,
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([space::XS, space::SM])
    .on_press(Message::Click(F::WallpaperSet(s)))
    .style(move |_t: &Theme, status| button::Style {
        background: matches!(status, button::Status::Hovered)
            .then(|| Background::Color(alpha(pp.on_surface, theme::STATE_HOVER))),
        text_color: pp.on_surface,
        border: Border::default().rounded(radius::MD),
        shadow: Shadow::default(),
        snap: true,
    });
    let f = F::WallpaperSet(s);
    container(halo(b, app.is_focused(f), radius::MD, p))
        .id(wallpaper_item(f).unwrap_or_default())
        .into()
}

/// A wallpaper card: the photo edge to edge, its name over a scrim along the bottom, a check
/// badge (and a play badge for videos) top right. Picked ones glow amber; the others are dimmed
/// with an empty badge. Until (or unless) its picture is decoded, a placeholder.
fn wallpaper_card<'a>(app: &'a App, s: usize, i: usize, p: &Palette) -> El<'a> {
    let set = &app.wallpaper_sets()[s];
    let file = &set.files[i];
    let on = app.form.wallpaper_on(set, file);
    let f = F::Wallpaper(s, i);
    let focused = app.is_focused(f);
    let pp = *p;
    let fill = |el: El<'a>| container(el).width(Length::Fill).height(Length::Fill);
    let picture: El = match app.thumbs.get(&model::wallpaper_key(set, file)) {
        Some(h) => image(h.clone())
            .content_fit(ContentFit::Cover)
            .width(Length::Fill)
            .height(Length::Fill)
            .border_radius(radius::MD)
            .into(),
        None => center(icon(Icon::Wallpaper, 28.0, alpha(p.on_surface_variant, 0.6))).into(),
    };
    let dim = fill(Space::new().into()).style(move |_t| container::Style {
        background: (!on).then(|| Background::Color(alpha(DARK.scrim, 0.38))),
        border: Border::default().rounded(radius::MD),
        ..container::Style::default()
    });
    let scrim = fill(Space::new().into()).style(move |_t| {
        let g = Linear::new(Radians(PI))
            .add_stop(0.45, alpha(DARK.surface, 0.0))
            .add_stop(0.75, alpha(DARK.surface, 0.4))
            .add_stop(1.0, alpha(DARK.surface, 0.78));
        container::Style {
            background: Some(Background::Gradient(Gradient::Linear(g))),
            border: Border::default().rounded(radius::MD),
            ..container::Style::default()
        }
    });
    let name = elide(
        wallpaper_label(&file.name),
        theme::SANS_MEDIUM,
        size::SM,
        if on { p.on_surface } else { alpha(p.on_surface, 0.72) },
    )
    .width(Length::Fill);
    let label = fill(name.into())
        .padding(Padding::new(CARD_INSET).top(0))
        .align_y(Alignment::End);
    let mut badges = row![].spacing(space::XS + 2.0).align_y(Alignment::Center);
    if file.is_video() {
        badges = badges.push(play_badge(22.0, Color::WHITE, alpha(Color::BLACK, 0.55)));
    }
    let tick = if on { Tick::Full } else { Tick::Empty };
    badges = badges.push(check_badge(
        tick,
        22.0,
        alpha(Color::WHITE, 0.85),
        alpha(Color::BLACK, 0.3),
        p,
    ));
    let badges = fill(badges.into())
        .padding(space::SM + 2.0)
        .align_x(Alignment::End)
        .align_y(Alignment::Start);
    // The border is a layer of its own on top: a button's border is drawn under its content,
    // so the photo would cover it.
    let edge = fill(Space::new().into()).style(move |_t| container::Style {
        border: if on {
            Border {
                color: pp.primary,
                width: 2.0,
                radius: radius::MD.into(),
            }
        } else {
            Border {
                color: alpha(Color::WHITE, 0.08),
                width: 1.0,
                radius: radius::MD.into(),
            }
        },
        ..container::Style::default()
    });
    let b = button(stack![picture, dim, scrim, label, badges, edge])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .on_press(Message::Click(f))
        .style(move |_t: &Theme, status| button::Style {
            background: Some(Background::Color(pp.surface_container_highest)),
            text_color: pp.on_surface,
            border: Border::default().rounded(radius::MD),
            shadow: card_shadow(on, focused, matches!(status, button::Status::Hovered), &pp),
            snap: true,
        });
    container(halo(b, focused, radius::MD, p))
        .id(wallpaper_item(f).unwrap_or_default())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// A card's shadow: a soft amber glow when picked, and a lift under it when focused (deeper) or
/// hovered (lighter).
fn card_shadow(on: bool, focused: bool, hovered: bool, p: &Palette) -> Shadow {
    let color = |a: f32| if on { alpha(p.primary, a) } else { alpha(DARK.scrim, a + 0.15) };
    match (focused, hovered) {
        (true, _) => Shadow {
            color: color(0.4),
            offset: Vector::new(0.0, 6.0),
            blur_radius: CARD_GLOW + 4.0,
        },
        (false, true) => Shadow {
            color: color(0.3),
            offset: Vector::new(0.0, 3.0),
            blur_radius: CARD_GLOW,
        },
        (false, false) if on => Shadow {
            color: color(0.24),
            offset: Vector::ZERO,
            blur_radius: CARD_GLOW,
        },
        (false, false) => Shadow::default(),
    }
}

/// A summary row with a second, quieter line (paths, emails, versions):
/// those break badly when wrapped mid-token, so they get their own line.
fn kv2<'a>(k: &'a str, v: String, sub: String, p: &Palette) -> El<'a> {
    row![
        text(k).size(size::SM).color(p.on_surface_variant).width(96),
        column![
            text(v).size(size::MD).color(p.on_surface),
            elide(sub, theme::MONO, size::XS, p.on_surface_variant).width(Length::Fill),
        ]
        .spacing(2)
        .width(Length::Fill),
    ]
    .spacing(space::SM)
    .into()
}

fn kv<'a>(k: &'a str, v: String, p: &Palette) -> El<'a> {
    row![
        text(k).size(size::SM).color(p.on_surface_variant).width(96),
        text(v)
            .size(size::MD)
            .color(p.on_surface)
            .width(Length::Fill),
    ]
    .spacing(space::SM)
    .into()
}

/// A card on Last chance, sized by the grid it sits in.
fn block<'a>(id: &'static str, ic: Icon, t: &'a str, rows: Vec<El<'a>>, p: &Palette) -> El<'a> {
    container(
        column![
            row![
                icon(ic, 18.0, p.primary),
                text(t)
                    .font(theme::SANS_SEMIBOLD)
                    .size(size::MD)
                    .color(p.on_surface),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center)
            .height(30),
            column(rows).spacing(6),
        ]
        .spacing(space::SM),
    )
    .id(id)
    .padding(space::LG)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(well(*p))
    .into()
}

fn summary<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let Some(probe) = app.ready_probe() else {
        return Space::new().into();
    };
    let f = &app.form;
    let cfg = f.to_config(probe);
    let m = probe.payload.as_ref();
    let disk = f.selected_disk(probe);
    let whole = f.disk_mode != Some(DiskMode::FreeSpace);

    let lang = block(
        ids::SUMMARY_LANGUAGE,
        Icon::Language,
        "Language",
        vec![
            kv("Language", data::locale_name(&f.locale).into_owned(), p),
            kv(
                "Keyboard",
                f.keymaps
                    .iter()
                    .map(|k| data::keymap_name(k))
                    .collect::<Vec<_>>()
                    .join(", "),
                p,
            ),
            kv(
                "Time zone",
                {
                    let (city, region) = data::split_tz(&f.timezone);
                    if region.is_empty() {
                        city
                    } else {
                        format!("{city} ({region})")
                    }
                },
                p,
            ),
        ],
        p,
    );
    let acct = block(
        ids::SUMMARY_ACCOUNT,
        Icon::Person,
        "You",
        vec![
            kv("Name", f.full_name.clone(), p),
            kv(
                "Login",
                format!("{}@{}", cfg.user.username, cfg.host.hostname),
                p,
            ),
            match cfg.git.as_ref() {
                Some(g) => kv2("Git", g.name.clone(), g.email.clone(), p),
                None => kv("Git", "Later".into(), p),
            },
        ],
        p,
    );
    let disk_block = block(
        ids::SUMMARY_DISK,
        Icon::HardDrive,
        "Disk",
        vec![
            match disk {
                Some(d) => kv2(
                    "Disk",
                    disk_model(d).to_string(),
                    format!("{} · {}", d.path, human_bytes(d.size_bytes)),
                    p,
                ),
                None => kv("Disk", "None picked".into(), p),
            },
            kv(
                "Mode",
                if whole {
                    "Erase everything".into()
                } else {
                    format!(
                        "Alongside, in {} free",
                        human_bytes(f.free_region(probe).map(|r| r.end - r.start).unwrap_or(0))
                    )
                },
                p,
            ),
            kv(
                "Encryption",
                match (f.encrypt, f.luks_same_as_password) {
                    (false, _) => "Off. Brave.".into(),
                    (true, true) => "LUKS2, account password".into(),
                    (true, false) => "LUKS2, its own passphrase".into(),
                },
                p,
            ),
        ],
        p,
    );
    let sys = block(
        ids::SUMMARY_SYSTEM,
        Icon::DeployedCode,
        "System",
        vec![
            kv("Release", format!("KubaOS {}", app.codename()), p),
            kv2(
                "Image",
                m.map(|m| {
                    format!(
                        "{} · {}",
                        m.version,
                        m.commit.chars().take(7).collect::<String>()
                    )
                })
                .unwrap_or_default(),
                m.map(|m| format!("Linux {}", m.kernel)).unwrap_or_default(),
                p,
            ),
            kv(
                "Wallpapers",
                wallpapers_summary(cfg.wallpapers.as_ref(), app.wallpaper_sets()),
                p,
            ),
            kv(
                "Network",
                f.wifi_ssid
                    .clone()
                    .map(|s| format!("Wi-Fi “{s}”"))
                    .unwrap_or_else(|| "None".into()),
                p,
            ),
        ],
        p,
    );

    // The confirm strip. Erasing says so in red and names the disk, with no
    // jokes in that sentence (docs/BRAND.md, voice rule 4).
    let pp = *p;
    let hold_t = app
        .hold
        .map(|s| ((app.now - s).as_secs_f32() / HOLD.as_secs_f32()).min(1.0))
        .unwrap_or(0.0);
    let disk_name = disk
        .map(|d| {
            format!(
                "{} ({}, {})",
                disk_model(d),
                d.path.trim_start_matches("/dev/"),
                model::human_bytes_nb(d.size_bytes)
            )
        })
        .unwrap_or_default();
    let (tone, tone_bg, msg, sub, cta) = if whole {
        let n = disk.map(|d| d.partitions.len()).unwrap_or(0);
        let others = disk
            .and_then(|d| d.other_os())
            .map(|o| format!(", {o} included"))
            .unwrap_or_default();
        (
            p.error,
            alpha(p.error_container, 0.30),
            format!("Last chance to not fuck this up. Everything on {disk_name} gets erased."),
            if n == 0 {
                "There's no undo.".to_string()
            } else {
                format!(
                    "All {n} partition{}{others}. There's no undo.",
                    if n == 1 { "" } else { "s" }
                )
            },
            "Hold to erase and install",
        )
    } else {
        (
            p.primary,
            alpha(p.primary_container, 0.22),
            format!("KubaOS goes into the free space on {disk_name}."),
            format!(
                "{} stays where it is.",
                disk.and_then(|d| d.other_os()).unwrap_or("Everything else")
            ),
            "Hold to install",
        )
    };
    // The fill is a pill inside the border. iced doesn't clip children to a
    // rounded container, and a pill narrower than it is tall squashes its
    // radius, so it starts as a circle at the left end and fades in.
    const HOLD_W: f32 = 270.0;
    const HOLD_H: f32 = 48.0;
    const HOLD_INSET: f32 = 3.0;
    let inner_h = HOLD_H - 2.0 * HOLD_INSET;
    let mut fill = row![].height(Length::Fill);
    if hold_t > 0.0 {
        let w = inner_h + hold_t * (HOLD_W - 2.0 * HOLD_INSET - inner_h);
        let fade = (hold_t * 12.0).min(1.0);
        fill = fill.push(
            container(Space::new())
                .width(w)
                .height(Length::Fill)
                .style(move |_t| {
                    container::Style::default()
                        .background(alpha(tone, 0.55 * fade))
                        .border(Border::default().rounded(radius::FULL))
                }),
        );
    }
    let label_txt = if app.hold.is_some() {
        "Keep holding…"
    } else {
        cta
    };
    let hold_btn = mouse_area(
        container(stack![
            container(fill)
                .padding(HOLD_INSET)
                .width(Length::Fill)
                .height(Length::Fill),
            center(
                row![
                    icon(Icon::Download, 18.0, pp.on_surface),
                    text(label_txt)
                        .font(theme::SANS_SEMIBOLD)
                        .size(size::MD)
                        .color(pp.on_surface)
                ]
                .spacing(space::SM)
                .align_y(Alignment::Center)
            )
        ])
        .width(HOLD_W)
        .height(HOLD_H)
        .style(move |_t| container::Style {
            background: Some(Background::Color(alpha(tone, 0.16))),
            border: Border {
                color: alpha(tone, 0.9),
                width: 1.5,
                radius: radius::FULL.into(),
            },
            ..container::Style::default()
        }),
    )
    .on_press(Message::HoldStart)
    .on_release(Message::HoldEnd)
    .on_exit(Message::HoldEnd)
    .interaction(iced::mouse::Interaction::Pointer);

    let confirm = container(
        row![
            icon(
                if whole { Icon::Warning } else { Icon::Storage },
                28.0,
                tone
            ),
            column![
                text(msg)
                    .font(theme::SANS_SEMIBOLD)
                    .size(size::LG)
                    .color(p.on_surface),
                text(sub).size(size::SM).color(p.on_surface_variant),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![
                ring(hold_btn, app.is_focused(F::Hold), radius::FULL, p),
                text("or hold Enter").size(size::XS).color(p.outline)
            ]
            .spacing(4)
            .align_x(Alignment::Center),
        ]
        .spacing(space::LG)
        .align_y(Alignment::Center),
    )
    .padding(space::XL)
    .style(move |_t| container::Style {
        background: Some(Background::Color(tone_bg)),
        border: Border {
            color: alpha(tone, 0.45),
            width: 1.0,
            radius: radius::LG.into(),
        },
        ..container::Style::default()
    });

    let mut top = column![header(
        jp::SUMMARY,
        "Last chance",
        "Nothing's been written yet. Check it over, then hold the button.",
        p
    )];
    if let Some(note) = low_battery_note(app) {
        top = top.push(Space::new().height(space::MD)).push(
            row![
                icon(Icon::Warning, 18.0, p.error),
                text(note)
                    .font(theme::SANS_MEDIUM)
                    .size(size::MD)
                    .color(p.error)
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        );
    }

    column![
        top,
        Space::new().height(space::XL),
        scroll(
            "summary-scroll",
            container(grid(2, space::LG, [lang, acct, disk_block, sys]))
                .padding(Padding::default().right(14)),
            p
        )
        .height(Length::Fill),
        Space::new().height(space::LG),
        confirm,
    ]
    .into()
}

/// Asks for the charger on Last chance when the battery is low. It warns only: an install on a
/// low battery is the owner's call.
pub fn low_battery_note(app: &App) -> Option<String> {
    let b = app.battery.filter(|b| b.is_low())?;
    Some(format!(
        "{}% left. Plug the damn thing in.",
        b.percent
    ))
}

fn step_lead<'a>(state: StepState, t: f32, p: &Palette) -> El<'a> {
    let pp = *p;
    let inner: El = match state {
        StepState::Done => icon(Icon::Check, 16.0, p.on_primary).into(),
        StepState::Running => spinner(t, 20.0, p.primary),
        StepState::Failed => icon(Icon::Close, 16.0, p.on_error).into(),
        StepState::Pending => Space::new().into(),
    };
    container(inner)
        .center(22)
        .style(move |_t| {
            let (bg, border) = match state {
                StepState::Done => (pp.primary, Color::TRANSPARENT),
                StepState::Failed => (pp.error, Color::TRANSPARENT),
                StepState::Running => (Color::TRANSPARENT, Color::TRANSPARENT),
                StepState::Pending => (Color::TRANSPARENT, alpha(pp.outline, 0.6)),
            };
            container::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    color: border,
                    width: 1.5,
                    radius: radius::FULL.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

fn steps_list<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let Some(inst) = &app.install else {
        return Space::new().into();
    };
    let t = (app.now - app.epoch).as_secs_f32();
    let rows: Vec<El> = inst
        .steps
        .iter()
        .map(|s| {
            let timing = match (s.state, s.elapsed, s.started) {
                (_, Some(e), _) => fmt_duration(e),
                (StepState::Running, None, Some(st)) => {
                    fmt_duration(app.now.saturating_duration_since(st))
                }
                _ => String::new(),
            };
            let (c, f) = match s.state {
                StepState::Running => (p.on_surface, theme::SANS_SEMIBOLD),
                StepState::Done => (p.on_surface, theme::SANS),
                StepState::Failed => (p.error, theme::SANS_SEMIBOLD),
                StepState::Pending => (alpha(p.on_surface_variant, 0.7), theme::SANS),
            };
            let mut r = row![
                step_lead(s.state, t, p),
                text(s.info.title.clone()).font(f).size(size::MD).color(c)
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center)
            .height(30);
            r = r.push(Space::new().width(Length::Fill));
            if s.state == StepState::Running && s.fraction > 0.0 && s.fraction < 1.0 {
                r = r.push(
                    text(format!("{:.0}%", s.fraction * 100.0))
                        .font(theme::MONO)
                        .size(size::SM)
                        .color(p.primary),
                );
            }
            r.push(
                text(timing)
                    .font(theme::MONO)
                    .size(size::SM)
                    .color(p.on_surface_variant)
                    .width(64)
                    .align_x(Alignment::End),
            )
            .into()
        })
        .collect();
    // Two columns: the list is short and the panel is wide.
    let half = rows.len().div_ceil(2);
    let mut left = column![].spacing(4).width(Length::FillPortion(1));
    let mut right = column![].spacing(4).width(Length::FillPortion(1));
    for (i, r) in rows.into_iter().enumerate() {
        if i < half {
            left = left.push(r);
        } else {
            right = right.push(r);
        }
    }
    row![left, right].spacing(space::XXXL).into()
}

fn log_view<'a>(app: &'a App, p: &Palette, height: Length) -> El<'a> {
    let Some(inst) = &app.install else {
        return Space::new().into();
    };
    let skip = inst.log.len().saturating_sub(LOG_VIEW_LINES);
    let lines: Vec<El> = inst.log[skip..]
        .iter()
        .map(|l| {
            let c = match l.level {
                Level::Error => p.error,
                Level::Warn => p.primary,
                Level::Info if l.text.starts_with("==>") => p.tertiary,
                Level::Info => p.on_surface,
                Level::Debug => p.on_surface_variant,
            };
            text(l.text.clone())
                .font(theme::MONO)
                .size(size::SM)
                .color(c)
                .into()
        })
        .collect();
    let pp = *p;
    container(
        scroll(
            ids::LOG,
            column(lines)
                .spacing(1)
                .padding(Padding::default().right(14)),
            p,
        )
        .anchor_bottom()
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .padding(Padding::from([10.0, 14.0]))
    .height(height)
    .width(Length::Fill)
    .style(move |_t| container::Style {
        background: Some(Background::Color(alpha(pp.surface_container_lowest, 0.85))),
        border: Border {
            color: alpha(pp.outline_variant, 0.55),
            width: 1.0,
            radius: radius::MD.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// 0..=1, how far the log panel is open.
fn log_openness(app: &App) -> f32 {
    let open_t = ((app.now - app.log_open.1).as_secs_f32() / motion::NORMAL.as_secs_f32()).min(1.0);
    let eased = motion::bezier(motion::EMPHASIZED_DECEL, open_t);
    if app.log_open.0 { eased } else { 1.0 - eased }
}

fn log_toggle<'a>(app: &'a App, p: &Palette) -> El<'a> {
    btn(
        if app.log_open.0 {
            "Hide log"
        } else {
            "Show log"
        },
        Some(Icon::Terminal),
        false,
        Kind::Text,
        Some(Message::Click(F::ToggleLog)),
        app.is_focused(F::ToggleLog),
        p,
    )
}

fn installing<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let Some(inst) = &app.install else {
        return Space::new().into();
    };
    let pct = inst.shown * 100.0;
    let current = inst
        .current()
        .map(|s| s.info.title.clone())
        .unwrap_or_else(|| "Starting…".into());
    let detail = inst.detail.clone().unwrap_or_default();
    let elapsed = fmt_duration(app.now.saturating_duration_since(inst.started));
    let log_h = (log_openness(app) * 230.0).round();

    let big = row![
        text(format!("{pct:.0}"))
            .font(theme::SANS_LIGHT)
            .size(size::DISPLAY)
            .color(p.on_surface)
            .line_height(1.0),
        text("%")
            .font(theme::SANS_LIGHT)
            .size(28)
            .color(p.on_surface_variant),
    ]
    .align_y(Alignment::End);

    let mut col = column![
        row![
            column![
                accent(jp::INSTALLING, p),
                Space::new().height(2),
                title("Installing", p),
                Space::new().height(space::SM),
                body("Go stare at a wall, it'll be done before you get bored.", p),
            ]
            .width(Length::Fill),
            big,
        ]
        .align_y(Alignment::End),
        Space::new().height(space::XL),
        bar(
            inst.shown,
            8.0,
            p.primary,
            alpha(p.surface_container_highest, 0.9)
        ),
        Space::new().height(space::MD),
        row![
            text(current)
                .font(theme::SANS_MEDIUM)
                .size(size::MD)
                .color(p.on_surface),
            text(detail)
                .font(theme::MONO)
                .size(size::SM)
                .color(p.primary),
            Space::new().width(Length::Fill),
            icon(Icon::Timer, 16.0, p.on_surface_variant),
            text(elapsed)
                .font(theme::MONO)
                .size(size::SM)
                .color(p.on_surface_variant),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
        Space::new().height(space::XXL),
        container(steps_list(app, p))
            .height(Length::Fill)
            .clip(true),
    ];
    if log_h > 0.5 {
        col = col.push(log_view(app, p, Length::Fixed(log_h)));
        col = col.push(Space::new().height(space::MD));
    }
    let last = inst
        .log
        .last()
        .filter(|_| !app.log_open.0)
        .map(|l| l.text.clone())
        .unwrap_or_default();
    col = col.push(
        row![
            log_toggle(app, p),
            text(last)
                .font(theme::MONO)
                .size(size::SM)
                .color(alpha(p.on_surface_variant, 0.7))
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    );
    col.into()
}

fn failed<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let Some(inst) = &app.install else {
        return Space::new().into();
    };
    let (step, error, hint) = match &inst.outcome {
        Some(Outcome::Failed { step, error, hint }) => (*step, error.clone(), hint.clone()),
        _ => (None, "No idea what happened".to_string(), None),
    };
    let at = step
        .and_then(|id| inst.steps.iter().find(|s| s.info.id == id))
        .map(|s| {
            format!(
                "“{}” died{}. Nothing after it ran.",
                s.info.title,
                s.elapsed
                    .map(|e| format!(" after {}", fmt_duration(e)))
                    .unwrap_or_default()
            )
        })
        .unwrap_or_else(|| "The install stopped before it got anywhere.".into());
    let pp = *p;
    let log_h = log_openness(app);
    let mut col =
        column![
            accent(jp::FAILED, p).color(p.error),
            Space::new().height(2),
            title("Fuck.", p),
            Space::new().height(space::SM),
            body(at, p),
            Space::new().height(space::XL),
            container(
                row![
                    icon(Icon::Error, 24.0, p.error),
                    column![
                    // A whole grub-install command line in 16 px semibold
                    // is a wall; long errors drop to body size.
                    text(error.clone())
                        .font(theme::SANS_SEMIBOLD)
                        .size(if error.chars().count() > 90 { size::MD } else { size::LG })
                        .color(p.on_surface),
                    text(hint.unwrap_or_else(|| {
                        "The log has the gory details. Try again, or save it and go read it.".into()
                    }))
                    .size(size::MD)
                    .color(p.on_surface_variant),
                ]
                    .spacing(4),
                ]
                .spacing(space::LG)
                .align_y(Alignment::Center)
            )
            .padding(space::XL)
            .width(Length::Fill)
            .style(move |_t| container::Style {
                background: Some(Background::Color(alpha(pp.error_container, 0.30))),
                border: Border {
                    color: alpha(pp.error, 0.45),
                    width: 1.0,
                    radius: radius::LG.into()
                },
                ..container::Style::default()
            }),
            Space::new().height(space::LG),
        ];
    if log_h > 0.01 {
        col = col.push(
            container(log_view(app, p, Length::Fill))
                .height(Length::FillPortion((log_h * 100.0) as u16 + 1)),
        );
    }
    col = col.push(Space::new().height(Length::FillPortion(((1.0 - log_h) * 100.0) as u16 + 1)));
    col = col.push(Space::new().height(space::LG));
    col = col.push(
        row![
            btn(
                "Back to the review",
                Some(Icon::ArrowBack),
                false,
                Kind::Text,
                Some(Message::Click(F::Back)),
                app.is_focused(F::Back),
                p
            ),
            log_toggle(app, p),
            Space::new().width(Length::Fill),
            btn(
                "Save log",
                Some(Icon::Save),
                false,
                Kind::Tonal,
                Some(Message::Click(F::SaveLog)),
                app.is_focused(F::SaveLog),
                p
            ),
            btn(
                "Try again",
                Some(Icon::Restart),
                false,
                Kind::Primary,
                Some(Message::Click(F::Retry)),
                app.is_focused(F::Retry),
                p
            ),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    );
    col.into()
}

// ----------------------------------------------------------------- hero --

fn hero(app: &App) -> El<'_> {
    let f = app.fade();
    let p = DARK.faded(f);
    let dy = (1.0 - f) * 24.0;
    let inner: El = match app.screen {
        Screen::Welcome => welcome(app, &p),
        _ => done(app, &p),
    };
    let pad = if app.size.width >= 1100.0 { 88.0 } else { 40.0 };
    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_bottom(Length::Fill)
        .padding(Padding {
            top: pad,
            right: pad,
            bottom: (pad - dy).max(0.0),
            left: pad,
        })
        .into()
}

fn display<'a>(s: &'a str, p: &Palette) -> El<'a> {
    text(s)
        .font(theme::SANS_SEMIBOLD)
        .size(88)
        .color(p.on_surface)
        .line_height(1.0)
        .into()
}

fn lede<'a>(s: impl text::IntoFragment<'a>, p: &Palette) -> El<'a> {
    text(s)
        .size(size::LG)
        .line_height(1.5)
        .color(p.on_surface_variant)
        .width(Length::Fixed(580.0))
        .into()
}

fn welcome<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let t = (app.now - app.epoch).as_secs_f32();
    let status: El = match &app.probe {
        ProbeState::Loading => row![
            spinner(t, 18.0, p.primary),
            text("Poking at the hardware…").size(size::MD).color(p.on_surface_variant)
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center)
        .into(),
        ProbeState::Failed(e) => row![
            icon(Icon::Error, 20.0, p.error),
            text(format!("Couldn't read this machine: {e}. Enter tries again."))
                .size(size::MD)
                .color(p.error)
                .width(Length::Fixed(520.0)),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center)
        .into(),
        ProbeState::Ready(pr) if pr.payload.is_none() => row![
            icon(Icon::Error, 20.0, p.error),
            text("No system image on this stick, so I've got nothing to install. Rebuild the ISO and flash it again.")
                .size(size::MD)
                .color(p.error)
                .width(Length::Fixed(520.0)),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center)
        .into(),
        ProbeState::Ready(_) => Space::new().into(),
    };
    let mut meta = row![].spacing(space::SM);
    if let Some(pr) = app.ready_probe() {
        let c = |s: String| {
            chip(
                None,
                s,
                p.on_surface_variant,
                alpha(p.surface_container_high, 0.65),
            )
        };
        if let Some(m) = &pr.payload {
            meta = meta
                .push(chip(
                    Some(Icon::DeployedCode),
                    format!("v{}", m.version),
                    p.primary,
                    alpha(p.primary_container, 0.3),
                ))
                .push(c(m.commit.chars().take(7).collect()))
                .push(c(format!(
                    "built {}",
                    m.built_at.get(..10).unwrap_or(&m.built_at)
                )));
        }
        meta = meta.push(c(format!(
            "{}{}",
            pr.arch,
            if pr.uefi { " · UEFI" } else { " · BIOS" }
        )));
    }
    let failed = matches!(app.probe, ProbeState::Failed(_));
    let action = if failed {
        btn(
            "Try again",
            Some(Icon::Restart),
            false,
            Kind::Primary,
            Some(Message::Click(F::RetryProbe)),
            app.is_focused(F::RetryProbe),
            p,
        )
    } else {
        btn(
            "Let's go",
            Some(Icon::ArrowForward),
            true,
            Kind::Primary,
            app.blocker().is_none().then_some(Message::Click(F::Start)),
            app.is_focused(F::Start),
            p,
        )
    };
    column![
        accent(jp::WELCOME, p).size(18),
        Space::new().height(space::SM),
        row![
            sign(540.0),
            container(
                text(app.codename().to_uppercase()).font(theme::SANS_MEDIUM).size(size::MD).color(p.tertiary)
            )
            .padding(Padding::from([4.0, 12.0]))
            .style({
                let pp = *p;
                move |_t| container::Style {
                    border: Border { color: alpha(pp.tertiary, 0.6), width: 1.0, radius: radius::FULL.into() },
                    ..container::Style::default()
                }
            }),
        ]
        .spacing(space::XL)
        .align_y(Alignment::Center),
        Space::new().height(space::XL),
        text("Whatever's on here, it's had its chance. Let's fix this shit.")
            .font(theme::SANS_SEMIBOLD)
            .size(30)
            .color(p.on_surface)
            .width(Length::Fixed(640.0)),
        Space::new().height(space::LG),
        lede(
            "Kuba's own Linux: a fast, keyboard-driven desktop for writing Rust all day without the OS \
             getting in the way. I update when you say so, I never nag, and if you break me there's always \
             a way back.",
            p
        ),
        Space::new().height(space::XXXL),
        row![action, status].spacing(space::XL).align_y(Alignment::Center),
        Space::new().height(space::LG),
        // No image, nothing to start: no "Enter: start" either.
        if app.ready_probe().is_some_and(|p| p.payload.is_none()) {
            Space::new().height(18).into()
        } else {
            hint_row(app, p)
        },
        Space::new().height(space::XXL),
        meta,
    ]
    .into()
}

/// The Done screen's first line: the time the install took and a quip picked by it, so a quick
/// install brags and a slow one owns up.
pub fn done_line(took: Duration) -> String {
    let quip = match took.as_secs() {
        0..120 => "Damn, that was quick.",
        120..300 => "Not bad. Windows would still be on “Getting ready”.",
        300..900 => "Took its sweet time, but it's on there.",
        _ => "Slow as hell. Blame the USB stick, I do.",
    };
    format!("Done in {}. {quip} Pull the stick and reboot.", fmt_duration(took))
}

fn done<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let took = match app.install.as_ref().and_then(|i| i.outcome.clone()) {
        Some(Outcome::Done { elapsed }) => elapsed,
        _ => Duration::ZERO,
    };
    let pp = *p;
    let badge = container(icon(Icon::Check, 44.0, p.on_primary))
        .center(84)
        .style(move |_t| container::Style {
            background: Some(Background::Color(pp.primary)),
            border: Border::default().rounded(radius::FULL),
            shadow: Shadow {
                color: alpha(pp.primary, 0.5),
                offset: Vector::ZERO,
                blur_radius: 36.0,
            },
            ..container::Style::default()
        });
    let user = app.form.username.clone();
    // tty1 logs in by itself and starts the desktop: the only thing to type
    // at boot is the disk passphrase.
    let boot = if app.form.encrypt {
        format!("Type the disk passphrase at boot and you're on the desktop as {user}.")
    } else {
        format!("You land on the desktop as {user}.")
    };
    column![
        badge,
        Space::new().height(space::XXL),
        accent(jp::DONE, p).size(18),
        Space::new().height(space::SM),
        display("That's it.", p),
        Space::new().height(space::XL),
        lede(format!("{} {boot}", done_line(took)), p),
        Space::new().height(space::XXXL),
        row![
            if let Some(action) = app.shutting_down {
                row![
                    spinner((app.now - app.epoch).as_secs_f32(), 18.0, p.primary),
                    text(going_down(action))
                        .size(size::MD)
                        .color(p.on_surface_variant)
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center)
                .padding(Padding::from([13.0, 4.0]))
                .into()
            } else {
                btn(
                    "Reboot",
                    Some(Icon::Power),
                    false,
                    Kind::Primary,
                    Some(Message::Click(F::Reboot)),
                    app.is_focused(F::Reboot),
                    p,
                )
            },
            btn(
                "Stay in the live system",
                None,
                false,
                Kind::Text,
                Some(Message::Click(F::Stay)),
                app.is_focused(F::Stay),
                p
            ),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
        Space::new().height(space::LG),
        hint_row(app, p),
    ]
    .into()
}

// ---------------------------------------------------------------- power --

/// The button that opens the power menu, on every screen.
fn power_button(app: &App) -> El<'_> {
    quiet_btn(
        "Power",
        Icon::Power,
        Some(Message::Click(F::Power)),
        app.is_focused(F::Power),
        &DARK,
    )
}

/// What the power menu says, for where the install stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerCopy {
    pub title: &'static str,
    pub body: String,
    /// Going down now loses work: the menu is drawn in the error colour.
    pub danger: bool,
    pub off: &'static str,
    pub restart: &'static str,
    /// Shown in place of the choices once one is taken.
    pub status: Option<String>,
}

pub fn power_copy(app: &App) -> PowerCopy {
    let disk = app.form.disk.as_deref().unwrap_or("The disk");
    let running = app.install_running();
    let body = match app.install.as_ref().map(|i| &i.outcome) {
        None => "Nothing has been written to any disk yet.".to_string(),
        Some(None) => format!(
            "The install stops where it is. {disk} is left partly written and won't boot: \
             install KubaOS on it again to use it."
        ),
        Some(Some(Outcome::Done { .. })) => {
            "KubaOS is installed. Pull the USB stick first, or the machine starts this installer \
             again."
                .to_string()
        }
        Some(Some(Outcome::Failed { .. })) => format!(
            "The install didn't finish, so {disk} won't boot until KubaOS is installed on it again."
        ),
    };
    let stopping = app.power.is_some_and(|m| m.stopping.is_some());
    let status = match app.shutting_down {
        Some(action) => Some(going_down(action).to_string()),
        None if stopping => Some("Stopping the install and unmounting the disk…".to_string()),
        None => None,
    };
    PowerCopy {
        title: if running {
            "Stop the install?"
        } else {
            "Power off or restart?"
        },
        body,
        danger: running,
        off: if running {
            "Stop and power off"
        } else {
            "Power off"
        },
        restart: if running {
            "Stop and restart"
        } else {
            "Restart"
        },
        status,
    }
}

fn going_down(action: PowerAction) -> &'static str {
    match action {
        PowerAction::PowerOff => "Powering off…",
        PowerAction::Restart => "Restarting…",
    }
}

/// The power menu over the dimmed screen. It takes every click: one outside the card cancels,
/// and nothing behind it reacts.
fn power_menu(app: &App) -> El<'_> {
    let p = DARK;
    let copy = power_copy(app);
    let tint = if copy.danger { p.error } else { p.primary };
    let busy = copy.status.is_some();
    let lead = icon(
        if copy.danger {
            Icon::Warning
        } else {
            Icon::Power
        },
        24.0,
        tint,
    );
    let actions: El = match &copy.status {
        Some(status) => row![
            spinner((app.now - app.epoch).as_secs_f32(), 18.0, tint),
            text(status.clone())
                .size(size::MD)
                .color(p.on_surface_variant)
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center)
        .padding(Padding::from([13.0, 4.0]))
        .into(),
        None => row![
            btn(
                "Cancel",
                None,
                false,
                Kind::Text,
                Some(Message::Click(F::PowerCancel)),
                app.is_focused(F::PowerCancel),
                &p
            ),
            Space::new().width(Length::Fill),
            btn(
                copy.restart,
                Some(Icon::Restart),
                false,
                Kind::Tonal,
                Some(Message::Click(F::PowerRestart)),
                app.is_focused(F::PowerRestart),
                &p
            ),
            btn(
                copy.off,
                Some(Icon::Power),
                false,
                Kind::Primary,
                Some(Message::Click(F::PowerOff)),
                app.is_focused(F::PowerOff),
                &p
            ),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center)
        .into(),
    };
    let card = container(
        column![
            row![lead, title(copy.title, &p).size(24)]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            Space::new().height(space::MD),
            body(copy.body, &p),
            Space::new().height(space::XXL),
            actions,
        ]
        .width(Length::Fill),
    )
    .width(Length::Fixed(if copy.danger { 560.0 } else { 500.0 }))
    .padding(space::XXL)
    .style(move |_t| container::Style {
        background: Some(Background::Color(p.surface_container_high)),
        border: Border {
            color: alpha(tint, if copy.danger { 0.6 } else { 0.25 }),
            width: 1.0,
            radius: radius::PANEL.into(),
        },
        shadow: Shadow {
            color: alpha(p.scrim, 0.6),
            offset: Vector::new(0.0, 18.0),
            blur_radius: 48.0,
        },
        ..container::Style::default()
    });
    let scrim = container(opaque(card))
        .center(Length::Fill)
        .style(|_t| container::Style::default().background(alpha(DARK.scrim, 0.6)));
    let mut area = mouse_area(scrim);
    if !busy {
        area = area.on_press(Message::Click(F::PowerCancel));
    }
    opaque(area)
}
