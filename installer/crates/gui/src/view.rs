//! Everything on screen. Panel screens share the glass panel with the step
//! rail; Welcome and Done are full-bleed over the wallpaper. First-boot
//! setup is its own app (setup/).
//!
//! Copy follows docs/BRAND.md: plain facts first, attitude around them, at
//! most one swear per screen, none near the disk that gets erased or the
//! passphrase prompts.

use std::f32::consts::PI;
use std::time::Duration;

use iced::border::Radius;
use iced::gradient::Linear;
use iced::widget::{
    Space, button, center, column, container, image, mouse_area, row, stack, svg, text,
};
use iced::{
    Alignment, Background, Border, Color, ContentFit, Element, Gradient, Length, Padding, Radians,
    Rectangle, Shadow, Size, Theme, Vector,
};

use dotinstall_core::Level;
use dotinstall_core::probe::DiskInfo;

use crate::app::{App, F, HOLD, List, Message, ProbeState, Screen, ids};
use crate::data;
use crate::elide::elide;
use crate::model::{self, DiskExt, DiskMode, human_bytes};
use crate::progress::{LOG_VIEW_LINES, Outcome, StepState, fmt_duration};
use crate::theme::{self, DARK, Icon, Palette, alpha, jp, mix, motion, radius, size, space};
use crate::widgets::{
    FieldLook, Kind, ROW_GAP, accent, bar, body, btn, chip, field, field_with, header, icon, label,
    list_row, ring, scroll, spinner, switch_anim, text_field, title, well, well_ring,
};

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
    if app.backend.is_demo() {
        layers = layers.push(
            container(chip(
                Some(Icon::Bolt),
                "Demo",
                DARK.tertiary,
                alpha(DARK.tertiary_container, 0.45),
            ))
            .width(Length::Fill)
            .align_right(Length::Fill)
            .padding(14),
        );
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
            .on_press_maybe(clickable.then_some(Message::Click(F::Edit(target))))
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
        text(format!("{} · installer {version}", app.codename()))
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
            keys
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
        Screen::Language | Screen::Timezone => {
            v.extend([("A–Z", "search"), ("↑ ↓", "pick"), ("Tab", "next field")]);
        }
        Screen::Wifi => v.extend([("↑ ↓", "pick"), ("Tab", "next field")]),
        Screen::Account | Screen::Git => v.push(("Tab", "next field")),
        Screen::Disk => v.extend([
            ("↑ ↓", "pick a disk"),
            ("← →", "keep or erase"),
            ("Space", "switch"),
            ("Tab", "next field"),
        ]),
        Screen::Summary => v.extend([("Enter", "hold to install"), ("Tab", "edit")]),
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
    let (c1, c2) = if selected {
        (
            p.on_secondary_container,
            alpha(p.on_secondary_container, 0.75),
        )
    } else {
        (p.on_surface, p.on_surface_variant)
    };
    row![
        elide(main, theme::SANS_MEDIUM, size::MD, c1).width(Length::Fill),
        Space::new().width(space::MD),
        text(sub).font(theme::MONO).size(size::XS).color(c2),
        Space::new().width(space::MD),
        check_if(selected, p),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn language<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let rows = |list: List, cur: &str| -> Vec<El<'a>> {
        app.list_items(list)
            .into_iter()
            .map(|v| {
                let sel = v == cur;
                let name = match list {
                    List::Locale => data::locale_name(&v),
                    _ => data::keymap_name(&v),
                }
                .into_owned();
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
            "The language you want to read, and the layout your fingers actually know. Type to search, arrows to pick.",
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
                    list_well(ids::KEYMAP_LIST, rows(List::Keymap, &app.form.keymap), key_focus, p),
                ),
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
                "No Wi-Fi. I'll plug in a cable like an adult.",
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
            "Optional. The install doesn't need the internet; this only saves a network for your first boot.",
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
    let host = payload.map(|m| m.hostname.clone()).unwrap_or_default();
    let baked = payload.is_some_and(|m| m.username == f.username);
    let bad_name =
        !f.username.is_empty() && !dotinstall_core::engine::validate::valid_username(&f.username);
    let username = input_err(app, F::Username, "you", None, bad_name, p);
    let username_note = if f.username.is_empty() {
        "Your login and home folder.".to_string()
    } else if bad_name {
        "Lowercase: a-z first, then a-z, 0-9, - or _.".to_string()
    } else if baked {
        format!(
            "/home/{}. The image's home config is built for {}@{host} already.",
            f.username, f.username
        )
    } else {
        format!(
            "/home/{}. The install builds the home config for {}@{host}: a few seconds more.",
            f.username, f.username
        )
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
            "Your login and your sudo password. Pick one you'll still remember at 2am.",
            p
        ),
        Space::new().height(space::XXL),
        scroll(
            ids::ACCOUNT_SCROLL,
            column![
                container(names).id(ids::REVEAL_NAMES),
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

fn block<'a>(
    app: &'a App,
    ic: Icon,
    t: &'a str,
    rows: Vec<El<'a>>,
    to: Option<Screen>,
    p: &Palette,
) -> El<'a> {
    let pp = *p;
    let edit: El = match to {
        Some(s) => {
            let b = button(
                text("Edit")
                    .font(theme::SANS_MEDIUM)
                    .size(size::SM)
                    .color(p.primary),
            )
            .padding(Padding::from([4.0, 10.0]))
            .on_press(Message::Click(F::Edit(s)))
            .style(move |_t: &Theme, status| button::Style {
                background: matches!(status, button::Status::Hovered)
                    .then_some(Background::Color(alpha(pp.primary, 0.12))),
                text_color: pp.primary,
                border: Border::default().rounded(radius::FULL),
                shadow: Shadow::default(),
                snap: true,
            });
            ring(b, app.is_focused(F::Edit(s)), radius::FULL, p)
        }
        None => Space::new().into(),
    };
    container(
        column![
            row![
                icon(ic, 18.0, p.primary),
                text(t)
                    .font(theme::SANS_SEMIBOLD)
                    .size(size::MD)
                    .color(p.on_surface),
                Space::new().width(Length::Fill),
                edit
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center)
            .height(30),
            column(rows).spacing(6),
        ]
        .spacing(space::SM),
    )
    .padding(space::LG)
    .width(Length::Fill)
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
        app,
        Icon::Language,
        "Language",
        vec![
            kv("Language", data::locale_name(&f.locale).into_owned(), p),
            kv("Keyboard", data::keymap_name(&f.keymap).into_owned(), p),
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
        Some(Screen::Language),
        p,
    );
    let acct = block(
        app,
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
        Some(Screen::Account),
        p,
    );
    let disk_block = block(
        app,
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
        Some(Screen::Disk),
        p,
    );
    let sys = block(
        app,
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
                "Network",
                f.wifi_ssid
                    .clone()
                    .map(|s| format!("Wi-Fi “{s}”"))
                    .unwrap_or_else(|| "None".into()),
                p,
            ),
        ],
        app.flow().contains(&Screen::Wifi).then_some(Screen::Wifi),
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
            format!("Everything on {disk_name} gets erased."),
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
    let fill_portion = (hold_t * 1000.0) as u16;
    let mut fill = row![].height(Length::Fill);
    if fill_portion > 0 {
        fill = fill.push(
            container(Space::new())
                .width(Length::FillPortion(fill_portion))
                .height(Length::Fill)
                .style(move |_t| {
                    container::Style::default()
                        .background(alpha(tone, 0.55))
                        .border(Border::default().rounded(radius::FULL))
                }),
        );
    }
    if fill_portion < 1000 {
        fill = fill.push(Space::new().width(Length::FillPortion(1000 - fill_portion)));
    }
    let label_txt = if app.hold.is_some() {
        "Keep holding…"
    } else {
        cta
    };
    let hold_btn = mouse_area(
        container(stack![
            fill,
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
        .width(270)
        .height(48)
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

    column![
        header(
            jp::SUMMARY,
            "Last chance",
            "Nothing's been written yet. Check it over, then hold the button. Tab gets you to Edit.",
            p
        ),
        Space::new().height(space::XL),
        scroll(
            "summary-scroll",
            column![row![lang, acct].spacing(space::LG), row![disk_block, sys].spacing(space::LG)]
                .spacing(space::LG)
                .padding(Padding::default().right(14)),
            p
        )
        .height(Length::Fill),
        Space::new().height(space::LG),
        confirm,
    ]
    .into()
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
                body(
                    "Go get a coffee. Actually, don't bother, it's faster than that.",
                    p
                ),
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
        .unwrap_or_else(|| "The installer stopped before it got anywhere.".into());
    let pp = *p;
    let log_h = log_openness(app);
    let mut col =
        column![
            accent(jp::FAILED, p).color(p.error),
            Space::new().height(2),
            title("Well, shit.", p),
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
            text("No system image on this stick, so there's nothing to install. Rebuild the ISO and flash it again.")
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
            lockup(76.0),
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
        text("Your machine. Your rules. No bullshit.").font(theme::SANS_SEMIBOLD).size(30).color(p.on_surface),
        Space::new().height(space::LG),
        lede(
            "Kuba's own Linux: a fast, keyboard-driven desktop for writing Rust all day without the OS \
             getting in the way. It updates when you say so, never nags, and if you break it there's always \
             a way back. Let's get rid of whatever's on this thing.",
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

fn done<'a>(app: &'a App, p: &Palette) -> El<'a> {
    let took = match app.install.as_ref().and_then(|i| i.outcome.clone()) {
        Some(Outcome::Done { elapsed }) => fmt_duration(elapsed),
        _ => fmt_duration(Duration::ZERO),
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
        format!("type the disk passphrase, and you're on the desktop as {user}")
    } else {
        format!("and you're on the desktop as {user}")
    };
    column![
        badge,
        Space::new().height(space::XXL),
        accent(jp::DONE, p).size(18),
        Space::new().height(space::SM),
        display("Done.", p),
        Space::new().height(space::XL),
        lede(
            format!(
                "Installed in {took}. Pull the USB stick, reboot, {boot}. \
                 Enjoy a computer that minds its own business."
            ),
            p
        ),
        Space::new().height(space::XXXL),
        row![
            if app.rebooting {
                row![
                    spinner((app.now - app.epoch).as_secs_f32(), 18.0, p.primary),
                    text("Rebooting…")
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
