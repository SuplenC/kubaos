//! Everything on screen. The panel screens share the glass panel with the step rail; Welcome and
//! Done are full-bleed over the wallpaper. The power menu opens over either. The battery sits
//! beside the power button on both. First-boot setup is its own app (`setup/`).
//!
//! Every view is built from `kubaos-ui`'s layout and components, with its tokens: no length,
//! colour, duration or font is written here.
//!
//! Copy follows docs/BRAND.md: plain facts first, attitude around them, at most one swear per
//! screen, none near the disk that gets erased or the passphrase prompts.

use std::f32::consts::PI;
use std::sync::LazyLock;
use std::time::Duration;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::ContentFit;
use iced::Gradient;
use iced::Length;
use iced::Radians;
use iced::Rectangle;
use iced::Shadow;
use iced::Size as WindowSize;
use iced::Vector;
use iced::gradient::Linear;
use iced::widget;
use iced::widget::container;
use iced::widget::image;
use iced::widget::svg;
use kubaos_core::InstallConfig;
use kubaos_core::Level;
use kubaos_core::probe::DiskInfo;
use kubaos_core::probe::Probe;
use kubaos_ui::Look;
use kubaos_ui::Scale;
use kubaos_ui::color;
use kubaos_ui::component::banner::BannerTone;
use kubaos_ui::component::banner::banner;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::card::card;
use kubaos_ui::component::card::card_sized;
use kubaos_ui::component::card::choice_card;
use kubaos_ui::component::check::Tick;
use kubaos_ui::component::check::checkbox;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::description::Fact;
use kubaos_ui::component::description::description_list;
use kubaos_ui::component::dialog::Tone;
use kubaos_ui::component::dialog::dialog;
use kubaos_ui::component::dialog::scrim;
use kubaos_ui::component::error::Level as LogLevel;
use kubaos_ui::component::error::error_block;
use kubaos_ui::component::error::log;
use kubaos_ui::component::field::State;
use kubaos_ui::component::field::field;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::component::keycap::keycap;
use kubaos_ui::component::list::list;
use kubaos_ui::component::list::list_row;
use kubaos_ui::component::list::scrolling_list;
use kubaos_ui::component::media::Aspect;
use kubaos_ui::component::picture::NamePlace;
use kubaos_ui::component::picture::picture_choice;
use kubaos_ui::component::progress::danger_timed_bar;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::component::progress::timed_bar;
use kubaos_ui::component::step::StepState as RailState;
use kubaos_ui::component::step::step;
use kubaos_ui::component::switch::switch;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::flow;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::page_clear;
use kubaos_ui::layout::row;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::look::radius;
use kubaos_ui::motion::Motion;
use kubaos_ui::motion::ease;
use kubaos_ui::text::line;
use kubaos_ui::text::text;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Mode;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Radius;
use kubaos_ui::tokens::Rgb;
use kubaos_ui::tokens::Role;
use kubaos_ui::tokens::Shadow as ShadowToken;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;
use kubaos_ui::tokens::palette;

use crate::app::App;
use crate::app::F;
use crate::app::HOLD;
use crate::app::List;
use crate::app::Message;
use crate::app::PowerAction;
use crate::app::ProbeState;
use crate::app::Screen;
use crate::app::WALLPAPER_COLUMNS;
use crate::app::ids;
use crate::app::wallpaper_item;
use crate::data;
use crate::done;
use crate::model;
use crate::model::DiskExt;
use crate::model::DiskMode;
use crate::model::human_bytes;
use crate::progress::LOG_VIEW_LINES;
use crate::progress::Outcome;
use crate::progress::StepState;
use crate::progress::fmt_duration;
use crate::theme;
use crate::theme::jp;
use crate::ui;
use crate::ui::El;

/// The room between a screen's body and the footer under it.
const FOOTER_GAP: Space = Space::S16;

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
fn wide(win: WindowSize) -> bool {
    win.width >= 2.0 * f32::from(Size::Column.px())
}

pub fn view(app: &App) -> El<'_> {
    let look = app.look();
    let fg: El = if app.screen.in_panel() {
        panel(app, &look)
    } else {
        hero(app, &look)
    };
    let mut layers = widget::stack![background(app, &look), fg]
        .width(Length::Fill)
        .height(Length::Fill);
    // The panel screens carry the power button in the rail; the full-bleed ones in the corner.
    let mut corner = row(&look, Space::S16).center();
    if app.backend.is_demo() {
        corner = corner.push(chip(&look, ChipKind::Cold, "Demo", Some(Icon::Bolt), None, None));
    }
    if !app.screen.in_panel() {
        if let Some(b) = app.battery {
            corner = corner.push(ui::battery(&look, b));
        }
        corner = corner.push(power_button(app, &look));
    }
    layers = layers.push(
        widget::container(corner)
            .width(Length::Fill)
            .align_right(Length::Fill)
            .padding(Inset::all(Space::S16).padding()),
    );
    if app.power.is_some() {
        layers = layers.push(power_menu(app, &look));
    }
    if let Some(toast) = ui::toast(&look, app.toast.as_ref(), app.now) {
        layers = layers.push(toast);
    }
    crate::pointer::idle(layers, app.pointer_idle).into()
}

fn background<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let base = widget::container(widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style::default().background(c.surface_container_lowest));
    let mut layers = widget::stack![base];
    if let Some((handle, at)) = &app.wallpaper {
        let shown = Motion::default().progress(tokens::Duration::Slow, Easing::Standard, app.now - *at);
        layers = layers.push(
            image(handle.clone())
                .content_fit(ContentFit::Cover)
                .width(Length::Fill)
                .height(Length::Fill)
                .opacity(shown),
        );
    }
    let hero = !app.screen.in_panel();
    let scrim = move |_theme: &iced::Theme| {
        let dimmed = if hero {
            // Dark behind the words on the left, the city glowing on the right.
            let dark = color::alpha(c.surface_container_lowest, Opacity::Backdrop.value());
            Background::Gradient(Gradient::Linear(
                Linear::new(Radians(PI / 2.0))
                    .add_stop(0.0, dark)
                    .add_stop(0.5, dark)
                    .add_stop(1.0, color::alpha(c.surface_container_lowest, 0.0)),
            ))
        } else {
            Background::Color(color::alpha(c.scrim, Opacity::Scrim.value()))
        };
        container::Style {
            background: Some(dimmed),
            ..container::Style::default()
        }
    };
    layers
        .push(
            widget::container(widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(scrim),
        )
        .into()
}

/// クバ beside "KubaOS", the mark a symbol tall and the wordmark a line of title text.
fn lockup<'a>(look: &Look) -> El<'a> {
    let (mark, word) = LOGO[usize::from(look.mode == Mode::Light)].clone();
    // The mark's drawing keeps room for its glow, so it is the larger box of the two.
    let mark = svg(mark)
        .width(f32::from(IconSize::S32.px()))
        .height(f32::from(IconSize::S32.px()));
    let word = svg(word).height(f32::from(IconSize::S24.px())).width(Length::Shrink);
    row(look, Gap::NONE).center().push(mark).push(word).into()
}

/// The mark's and the wordmark's handles in dark and in light, made once, as the sign's is.
static LOGO: LazyLock<[(svg::Handle, svg::Handle); 2]> = LazyLock::new(|| [Mode::Dark, Mode::Light].map(logo_in));

/// The logo's drawings in `mode`'s colours. They are written in the dark palette's `on_surface`
/// (the wordmark's "Kuba"), `primary` (the mark, "OS") and `cold` (the dakuten); in light those
/// would be pale on the pale surface, under 1.2:1, so each takes the light palette's role.
fn logo_in(mode: Mode) -> (svg::Handle, svg::Handle) {
    let handle = |drawing: &[u8]| svg::Handle::from_memory(logo_drawing(drawing, mode).into_bytes());
    (handle(theme::LOGO_MARK), handle(theme::LOGO_WORDMARK))
}

/// A logo drawing's SVG with its dark palette roles swapped for `mode`'s.
fn logo_drawing(drawing: &[u8], mode: Mode) -> String {
    let (written, shown) = (palette(Mode::Dark), palette(mode));
    let hex = |c: Rgb| format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
    let mut text = String::from_utf8_lossy(drawing).into_owned();
    for role in [Role::OnSurface, Role::Primary, Role::Cold] {
        text = text.replace(&hex(role.of(&written)), &hex(role.of(&shown)));
    }
    text
}

/// The neon sign's handle, made once: a handle made from bytes gets a new id every time, so one
/// made in `view` would be decoded and uploaded again on every frame.
static SIGN: LazyLock<image::Handle> = LazyLock::new(|| image::Handle::from_bytes(theme::LOGO_SIGN));

/// The neon sign, a column of text wide. Nearly half its height is the glow on the wall around the
/// glass.
fn sign<'a>(look: &Look) -> El<'a> {
    let width = f32::from(Size::Column.px());
    image(SIGN.clone())
        .width(width)
        .height(look.scale.snap(width / theme::LOGO_SIGN_ASPECT))
        .into()
}

// ---------------------------------------------------------------- panel --

fn panel<'a>(app: &'a App, look: &Look) -> El<'a> {
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

fn content<'a>(app: &'a App, look: &Look) -> El<'a> {
    let shown = app.fade();
    let look = look.faded(shown);
    let look = &look;
    let body: El = match app.screen {
        Screen::Language => language(app, look),
        Screen::Timezone => timezone(app, look),
        Screen::Wifi => wifi(app, look),
        Screen::Account => account(app, look),
        Screen::Disk => disk(app, look),
        Screen::Git => git(app, look),
        Screen::Wallpapers => wallpapers(app, look),
        Screen::Summary => summary(app, look),
        Screen::Installing => installing(app, look),
        Screen::Failed => failed(app, look),
        Screen::Welcome | Screen::Done => widget::Space::new().into(),
    };
    let mut page = stack(look, Gap::NONE)
        .height(Extent::Fill)
        .push(widget::container(body).height(Length::Fill));
    if !matches!(app.screen, Screen::Installing | Screen::Failed | Screen::Summary) {
        page = page.push(gap(FOOTER_GAP)).push(footer(app, look));
    }
    // The page slides in from the side it comes from, a whole device pixel at a time.
    let travel = look
        .scale
        .snap((1.0 - shown) * f32::from(Space::S32.px()) * if app.transition.1 { 1.0 } else { -1.0 });
    widget::container(page_clear(page))
        .padding(iced::Padding::ZERO.left(travel.max(0.0)).right((-travel).max(0.0)))
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .into()
}

fn footer<'a>(app: &'a App, look: &Look) -> El<'a> {
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

// -------------------------------------------------------------- screens --

fn input<'a>(app: &'a App, f: F, placeholder: &'a str, glyph: Option<Icon>, look: &Look) -> El<'a> {
    input_state(app, f, placeholder, glyph, State::Normal, look)
}

/// [`input`] in `state`: invalid shows the error colour, and the reason goes under it.
fn input_state<'a>(app: &'a App, f: F, placeholder: &'a str, glyph: Option<Icon>, state: State, look: &Look) -> El<'a> {
    let mut input = field(app.field_value(f), placeholder)
        .state(state)
        .blink_from(app.focus_at)
        .on_pointer(move |pointer| Message::FieldPointer(f, pointer));
    if let Some(glyph) = glyph {
        input = input.icon(glyph);
    }
    if f.is_secret() && !app.show_secrets {
        input = input.secret();
    }
    if app.is_focused(f) {
        input = input.caret(app.caret);
    }
    input.view(look)
}

fn invalid(wrong: bool) -> State {
    if wrong { State::Invalid } else { State::Normal }
}

/// A list of the screen's choices, scrolled in its well to the selection.
fn choices<'a>(id: &'static str, rows: Vec<El<'a>>, focused: bool, look: &Look) -> El<'a> {
    if rows.is_empty() {
        return kubaos_ui::component::list::empty_list(look, "Nothing matches. Typo?");
    }
    scrolling_list(rows, focused)
        .id(id)
        .on_scroll(move |v| Message::Scrolled(id, v.absolute_offset().y, v.bounds().height))
        .view(look)
}

/// The keymap list: every layout, a check mark on the picked ones, the highlight on the row Space
/// acts on.
fn keymap_rows<'a>(app: &'a App, look: &Look) -> Vec<El<'a>> {
    let at = app.keymap_row();
    let default = app.form.keymaps.first();
    app.list_items(List::Keymap)
        .into_iter()
        .map(|v| {
            let here = v == at;
            let mut row = list_row(data::keymap_name(&v).into_owned())
                .detail(v.clone())
                .selected(here)
                .checked(app.form.keymap_picked(&v));
            if default == Some(&v) && app.form.keymaps.len() > 1 {
                row = row.chip(ChipKind::Accent, "Default");
            }
            row.on_press(Message::Pick(List::Keymap, v)).view(look)
        })
        .collect()
}

/// The picked layouts, the default first. A click makes one the default; its close button unpicks
/// it, all but the last one.
fn keymap_chips<'a>(app: &'a App, look: &Look) -> El<'a> {
    let keymaps = &app.form.keymaps;
    let chips = keymaps.iter().enumerate().map(|(i, k)| {
        let lead = i == 0;
        chip(
            look,
            if lead { ChipKind::Selected } else { ChipKind::Neutral },
            data::keymap_name(k).into_owned(),
            lead.then_some(Icon::Check),
            (!lead).then(|| Message::KeymapDefault(k.clone())),
            (keymaps.len() > 1).then(|| Message::KeymapDrop(k.clone())),
        )
    });
    let hint = if keymaps.len() > 1 {
        text(
            "The first is the default. Click one to make it the default.",
            Type::Caption,
            look.colors.on_surface_variant,
        )
    } else {
        ui::hint_row(
            look,
            &[("Space", "picks more"), ("Shift+Space", "makes one the default")],
        )
    };
    stack(look, Space::S8)
        .push(flow(look, Space::S8, chips))
        .push(hint)
        .into()
}

fn language<'a>(app: &'a App, look: &Look) -> El<'a> {
    let rows = |list: List, cur: &str| -> Vec<El<'a>> {
        app.list_items(list)
            .into_iter()
            .map(|v| {
                let sel = v == cur;
                list_row(data::locale_name(&v).into_owned())
                    .detail(v.clone())
                    .selected(sel)
                    .checked(sel)
                    .on_press(Message::Pick(list, v))
                    .view(look)
            })
            .collect()
    };
    let languages = stack(look, Space::S8)
        .height(Extent::Fill)
        .push(labelled(
            look,
            "Language",
            None,
            input(app, F::LocaleQ, "Search languages", Some(Icon::Search), look),
            None,
        ))
        .push(choices(
            ids::LOCALE_LIST,
            rows(List::Locale, &app.form.locale),
            app.is_focused(F::LocaleQ),
            look,
        ));
    let keyboards = stack(look, Space::S8)
        .height(Extent::Fill)
        .push(labelled(
            look,
            "Keyboard",
            None,
            input(app, F::KeymapQ, "Search layouts", Some(Icon::Search), look),
            None,
        ))
        .push(choices(
            ids::KEYMAP_LIST,
            keymap_rows(app, look),
            app.is_focused(F::KeymapQ),
            look,
        ))
        .push(keymap_chips(app, look))
        .push(input(
            app,
            F::KeyboardTest,
            "Mash some keys to check the layout",
            Some(Icon::Keyboard),
            look,
        ));
    stack(look, Space::S24)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::LANGUAGE,
            "Language and keyboard",
            "The language you want to read, and the layouts your fingers actually know. Type to search, arrows \
             to move, Space to pick a layout.",
        ))
        .push(grid(look, 2, Space::S20).fill_height().push(languages).push(keyboards))
        .into()
}

fn timezone<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let rows: Vec<El> = app
        .list_items(List::Timezone)
        .into_iter()
        .map(|tz| {
            let sel = tz == app.form.timezone;
            let (city, region) = data::split_tz(&tz);
            list_row(city)
                .detail(region)
                .selected(sel)
                .checked(sel)
                .on_press(Message::Pick(List::Timezone, tz))
                .view(look)
        })
        .collect();
    let (city, region) = data::split_tz(&app.form.timezone);
    let side = card_sized(
        look,
        stack(look, Gap::NONE)
            .height(Extent::Fill)
            .push(icon(look, Icon::Public, IconSize::S48, c.primary))
            .push(gap(Space::S16))
            .push(text(city, Type::Headline, c.on_surface))
            .push(text(region, Type::BodyLarge, c.on_surface_variant))
            .push(widget::Space::new().height(Length::Fill))
            .push(text(
                "Your clock, calendar and every timestamp in your logs follow this.",
                Type::Small,
                c.on_surface_variant,
            ))
            .push(gap(Space::S8))
            .push(text(app.form.timezone.clone(), Type::Mono, c.on_surface_variant)),
        Extent::Fill,
    );
    let search = stack(look, Space::S8)
        .height(Extent::Fill)
        .push(input(app, F::TzQ, "Search cities or regions", Some(Icon::Search), look))
        .push(choices(ids::TZ_LIST, rows, app.is_focused(F::TzQ), look));
    stack(look, Space::S24)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::TIMEZONE,
            "Time zone",
            "Where your clock lives. Start typing a city.",
        ))
        .push(split(look, &[2, 1], Space::S20).fill_height().push(search).push(side))
        .into()
}

/// A network's signal as one of the Wi-Fi symbols.
fn signal(strength: u8) -> Icon {
    match strength {
        75.. => Icon::SignalWifi4Bar,
        50.. => Icon::NetworkWifi3Bar,
        25.. => Icon::NetworkWifi2Bar,
        _ => Icon::NetworkWifi1Bar,
    }
}

fn wifi<'a>(app: &'a App, look: &Look) -> El<'a> {
    let Some(probe) = app.ready_probe() else {
        return widget::Space::new().into();
    };
    let none = app.form.wifi_ssid.is_none();
    let mut rows: Vec<El> = vec![
        list_row("Fuck Wi-Fi, I've got a cable")
            .icon(Icon::WifiOff)
            .selected(none)
            .checked(none)
            .on_press(Message::WifiNone)
            .view(look),
    ];
    for w in &probe.wifi {
        let sel = app.form.wifi_ssid.as_deref() == Some(&w.ssid);
        rows.push(
            list_row(w.ssid.clone())
                .icon(signal(w.signal))
                .detail(if w.secured { "Secured" } else { "Open" })
                .selected(sel)
                .checked(sel)
                .on_press(Message::Pick(List::Wifi, w.ssid.clone()))
                .view(look),
        );
    }
    let mut page = stack(look, Space::S24)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::NETWORK,
            "Wi-Fi",
            "Optional. I don't need the internet to install; this only saves a network for your first boot.",
        ))
        .push(choices(ids::WIFI_LIST, rows, app.is_focused(F::WifiList), look));
    if app.wifi_secured(app.form.wifi_ssid.as_deref()) {
        page = page.push(secret_field(app, "Wi-Fi password", F::WifiPsk, look));
    }
    page.into()
}

/// The Show/Hide toggle for secrets: small, at the right of a field's label.
fn show_toggle<'a>(app: &'a App, look: &Look) -> El<'a> {
    let (label, glyph) = if app.show_secrets {
        ("Hide", Icon::VisibilityOff)
    } else {
        ("Show", Icon::Visibility)
    };
    button(label)
        .kind(Kind::Text)
        .size(ButtonSize::Sm)
        .icon(glyph)
        .on_press(Message::Click(F::ShowSecrets))
        .focused(app.is_focused(F::ShowSecrets))
        .view(look)
}

fn secret_field<'a>(app: &'a App, label: &'a str, f: F, look: &Look) -> El<'a> {
    labelled(
        look,
        label,
        Some(show_toggle(app, look)),
        input(app, f, "", None, look),
        None,
    )
}

/// The note under the username: what it is for, the rule it breaks, or the home it gets. `baked`:
/// the image's home config is built for this name already.
fn username_note(username: &str, bad_name: bool, baked: bool) -> String {
    if username.is_empty() {
        "Your login and home folder.".to_string()
    } else if bad_name {
        "Lowercase: a-z first, then a-z, 0-9, - or _.".to_string()
    } else if baked {
        format!("/home/{username}. The image's home config is built for {username} already.")
    } else {
        format!("/home/{username}. The install builds the home config for {username}: a few seconds more.")
    }
}

fn account<'a>(app: &'a App, look: &Look) -> El<'a> {
    let f = &app.form;
    let mismatch = ui::mismatched(&f.password, &f.password_confirm);
    let payload = app.ready_probe().and_then(|p| p.payload.as_ref());
    let baked = payload.is_some_and(|m| m.username == f.username);
    let bad_name = !f.username.is_empty() && !kubaos_core::engine::validate::valid_username(&f.username);
    let name_note = username_note(&f.username, bad_name, baked);
    let bad_host = !f.hostname.is_empty() && !kubaos_core::engine::validate::valid_hostname(&f.hostname);
    let hostname_note = if bad_host {
        note(look, model::HOSTNAME_RULE, true)
    } else if f.hostname.is_empty() || f.username.is_empty() {
        note(look, "This machine's name on the network and in your prompt.", false)
    } else {
        let c = look.colors;
        row(look, Space::S4)
            .push(text("You'll be", Type::Small, c.on_surface_variant))
            .push(text(
                format!("{}@{}", f.username, f.hostname),
                Type::Mono,
                c.on_surface_variant,
            ))
            .into()
    };
    let pair = |left: El<'a>, right: El<'a>| -> El<'a> { grid(look, 2, Space::S20).push(left).push(right).into() };
    let names = pair(
        labelled(
            look,
            "Full name",
            None,
            input(app, F::FullName, "Your name", None, look),
            None,
        ),
        labelled(
            look,
            "Username",
            None,
            input_state(app, F::Username, "you", None, invalid(bad_name), look),
            Some(note(look, name_note, bad_name)),
        ),
    );
    let host = pair(
        labelled(
            look,
            "Hostname",
            None,
            input_state(app, F::Hostname, "this-machine", None, invalid(bad_host), look),
            Some(hostname_note),
        ),
        widget::Space::new().into(),
    );
    let passwords = pair(
        labelled(
            look,
            "Password",
            None,
            stack(look, Space::S8)
                .push(input(app, F::Password, "", None, look))
                .push(ui::strength(look, &f.password)),
            None,
        ),
        labelled(
            look,
            "Confirm password",
            Some(show_toggle(app, look)),
            input_state(app, F::PasswordConfirm, "", None, invalid(mismatch), look),
            ui::confirm_note(look, &f.password, &f.password_confirm),
        ),
    );
    // The title stays put; only the fields scroll (src/reveal.rs).
    let apart = Space::S32;
    stack(look, apart)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::ACCOUNT,
            "You",
            "Your login, this machine's name and your sudo password. Pick one you'll still remember at 2am.",
        ))
        .push(
            scroll(
                look,
                stack(look, Space::S20)
                    .padding(Inset::NONE.right(Space::S16))
                    .push(widget::container(names).id(ids::REVEAL_NAMES))
                    .push(widget::container(host).id(ids::REVEAL_HOST))
                    .push(widget::container(passwords).id(ids::REVEAL_PASSWORDS)),
            )
            .id(ids::ACCOUNT_SCROLL)
            .height(Extent::Fill)
            .room_above(apart)
            .room_below(FOOTER_GAP),
        )
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

/// A partition's colour in the layout bar.
fn part_color(hint: Option<&str>, i: usize, look: &Look) -> Color {
    let c = look.colors;
    match hint {
        Some(h) if h.contains("Windows") => c.cold_container,
        Some(h) if h.contains("EFI") => c.secondary_container,
        _ => [c.surface_bright, c.surface_container_highest][i % 2],
    }
}

fn disk_row<'a>(d: &'a DiskInfo, sel: bool, small: bool, look: &Look) -> El<'a> {
    let mut row = list_row(disk_model(d).to_string())
        .subtitle(format!("{} · {}", d.path, human_bytes(d.size_bytes)))
        .icon(Icon::HardDrive);
    if let Some(os) = d.other_os() {
        row = row.chip(ChipKind::Neutral, os.to_string());
    }
    if small {
        row = row.chip(ChipKind::Neutral, "Too small");
    }
    if d.removable {
        row = row.chip(ChipKind::Neutral, "USB");
    }
    row.selected(sel)
        .checked(sel)
        .on_press(Message::Pick(List::Disk, d.path.clone()))
        .view(look)
}

/// The disk after the install: kept partitions, then ours, each as wide as its share of the disk
/// (a sliver at least, so a small one still shows), with a legend under it.
fn layout_bar<'a>(d: &'a DiskInfo, mode: DiskMode, encrypt: bool, look: &Look) -> El<'a> {
    let c = look.colors;
    let total = d.size_bytes.max(1) as f32;
    // Shares in thousandths of the disk, a hundredth and a half at least.
    let share = |bytes: u64| ((bytes as f32 / total) * 1000.0).max(14.0) as u16;
    let ours = format!("KubaOS{}", if encrypt { " · encrypted" } else { "" });
    let mut parts: Vec<(Color, u16, bool)> = vec![];
    let mut legend: Vec<El<'a>> = vec![];
    let legend_item = |colour: Color, name: String, bytes: u64| -> El<'a> {
        row(look, Space::S8)
            .center()
            .push(
                widget::container(widget::Space::new())
                    .width(f32::from(Size::RadioDot.px()))
                    .height(f32::from(Size::RadioDot.px()))
                    .style(move |_theme| {
                        container::Style::default()
                            .background(colour)
                            .border(Border::default().rounded(radius(Radius::Full)))
                    }),
            )
            .push(text(name, Type::Small, c.on_surface))
            .push(text(human_bytes(bytes), Type::Mono, c.on_surface_variant))
            .into()
    };
    match mode {
        DiskMode::Whole => {
            parts.push((c.primary, 1000, true));
            legend.push(legend_item(c.primary, ours, d.size_bytes));
        }
        DiskMode::FreeSpace => {
            let free = d.largest_free();
            let mut items: Vec<(u64, Option<usize>)> = vec![];
            // Partitions in order, with our region slotted after the last partition that ends
            // before it: good enough for a picture.
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
            for (bytes, which) in items {
                if let Some(i) = which {
                    let part = &d.partitions[i];
                    let colour = part_color(part.os_hint.as_deref(), i, look);
                    parts.push((colour, share(bytes), false));
                    if bytes as f32 / total > 0.05 || part.os_hint.as_deref().is_some_and(|h| h.contains("EFI")) {
                        let name = part
                            .label
                            .clone()
                            .or(part.os_hint.clone())
                            .unwrap_or_else(|| part.path.rsplit('/').next().unwrap_or("").to_string());
                        legend.push(legend_item(colour, name, bytes));
                    }
                } else {
                    parts.push((c.primary, share(bytes), true));
                    legend.push(legend_item(c.primary, ours.clone(), bytes));
                }
            }
        }
    }
    let weights: Vec<u16> = parts.iter().map(|&(_, w, _)| w).collect();
    let glow = color::alpha(c.primary, ShadowToken::Glow.opacity());
    let segments = parts.into_iter().map(|(colour, _, ours)| -> El<'a> {
        widget::container(widget::Space::new())
            .width(Length::Fill)
            .height(f32::from(Size::ControlSm.px()))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(colour)),
                border: Border::default().rounded(radius(Radius::Small)),
                shadow: if ours {
                    Shadow {
                        color: glow,
                        offset: Vector::new(0.0, f32::from(ShadowToken::Glow.y())),
                        blur_radius: f32::from(ShadowToken::Glow.blur()),
                    }
                } else {
                    Shadow::default()
                },
                ..container::Style::default()
            })
            .into()
    });
    stack(look, Space::S12)
        .push(split(look, &weights, Space::S4).extend(segments))
        .push(flow(look, Space::S16, legend))
        .into()
}

/// One of the two ways to use the disk: its symbol and name, and what it means for this disk.
fn mode_card<'a>(
    app: &'a App,
    glyph: Icon,
    title: &'a str,
    sub: String,
    mode: DiskMode,
    on_pick: Option<Message>,
    look: &Look,
) -> El<'a> {
    let c = look.colors;
    let picked = app.form.disk_mode.unwrap_or(DiskMode::Whole) == mode;
    let dim = |colour: Color| {
        if on_pick.is_some() {
            colour
        } else {
            color::alpha(colour, Opacity::DisabledContent.value())
        }
    };
    let (main, muted) = if picked {
        (
            c.on_secondary_container,
            color::alpha(c.on_secondary_container, Opacity::SubduedText.value()),
        )
    } else {
        (c.on_surface, c.on_surface_variant)
    };
    let content = stack(look, Space::S4)
        .push(
            row(look, Space::S8)
                .center()
                .push(icon(look, glyph, IconSize::S20, dim(main)))
                .push(text(title, Type::Subheading, dim(main))),
        )
        .push(text(sub, Type::Small, dim(muted)));
    choice_card(look, content, picked, on_pick, picked && app.is_focused(F::DiskMode))
}

/// A setting with a switch: its symbol, its name and what it does, the switch at the end.
fn setting<'a>(app: &'a App, glyph: Icon, title: &'a str, sub: &'a str, on: bool, f: F, look: &Look) -> El<'a> {
    let c = look.colors;
    row(look, Space::S16)
        .center()
        .push(icon(look, glyph, IconSize::S24, c.primary))
        .push(
            stack(look, Space::S4)
                .width(Extent::Fill)
                .push(text(title, Type::BodyStrong, c.on_surface))
                .push(text(sub, Type::Small, c.on_surface_variant)),
        )
        .push(switch(
            look,
            on,
            app.switch_progress(f),
            Some(Message::Click(f)),
            app.is_focused(f),
        ))
        .into()
}

fn disk<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let Some(probe) = app.ready_probe() else {
        return widget::Space::new().into();
    };
    let f = &app.form;
    let rows: Vec<El> = probe
        .disks
        .iter()
        .filter(|d| d.is_target())
        .map(|d| disk_row(d, f.disk.as_deref() == Some(&d.path), !d.fits(probe), look))
        .collect();
    let disks = widget::container(list(look, rows, app.is_focused(F::DiskList))).id(ids::REVEAL_DISKS);

    let mut sections = stack(look, Space::S16)
        .padding(Inset::NONE.right(Space::S16))
        .push(disks);
    if let Some(d) = f.selected_disk(probe) {
        let mode = f.disk_mode.unwrap_or(DiskMode::Whole);
        sections = sections.push(disk_modes(app, d, probe, look)).push(
            stack(look, Space::S8)
                .push(text("After the install", Type::Label, c.on_surface_variant))
                .push(layout_bar(d, mode, f.encrypt, look)),
        );
    }
    sections = sections.push(disk_encryption(app, look));
    // The title stays put; only what's under it scrolls (src/reveal.rs).
    let apart = Space::S20;
    stack(look, apart)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::DISK,
            "Where it goes",
            "Pick a disk, then decide whether whatever's on it gets to live.",
        ))
        .push(
            scroll(look, sections)
                .id(ids::DISK_SCROLL)
                .height(Extent::Fill)
                .room_above(apart)
                .room_below(FOOTER_GAP),
        )
        .into()
}

/// The two ways to use the selected disk: beside what's on it, or all of it.
fn disk_modes<'a>(app: &'a App, d: &DiskInfo, probe: &Probe, look: &Look) -> El<'a> {
    let free = d.largest_free().map_or(0, |r| r.end - r.start);
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
        _ if !d.fits(probe) => format!("Too small: KubaOS needs {}.", human_bytes(model::needed(probe))),
        0 => "The disk is empty anyway.".to_string(),
        n => format!(
            "Wipes all {n} partition{}{}.",
            if n == 1 { "" } else { "s" },
            others.map(|o| format!(", {o} included")).unwrap_or_default()
        ),
    };
    let modes = grid(look, 2, Space::S16)
        .push(mode_card(
            app,
            Icon::Storage,
            "Install alongside",
            free_sub,
            DiskMode::FreeSpace,
            d.fits_dual_boot(probe)
                .then_some(Message::DiskModePick(DiskMode::FreeSpace)),
            look,
        ))
        .push(mode_card(
            app,
            Icon::Warning,
            "Erase the whole disk",
            whole_sub,
            DiskMode::Whole,
            Some(Message::DiskModePick(DiskMode::Whole)),
            look,
        ));
    widget::container(modes).id(ids::REVEAL_MODE).into()
}

fn disk_encryption<'a>(app: &'a App, look: &Look) -> El<'a> {
    let f = &app.form;
    let mut encryption = stack(look, Space::S16).push(
        widget::container(setting(
            app,
            if f.encrypt { Icon::Lock } else { Icon::LockOpen },
            "Encrypt the disk",
            "LUKS2. Lose the laptop, keep your secrets. You type the passphrase once at boot.",
            f.encrypt,
            F::Encrypt,
            look,
        ))
        .id(ids::REVEAL_ENCRYPT),
    );
    if f.encrypt {
        encryption = encryption.push(
            widget::container(setting(
                app,
                Icon::Key,
                "Use my account password",
                "One password for boot and login.",
                f.luks_same_as_password,
                F::LuksSame,
                look,
            ))
            .id(ids::REVEAL_LUKS_SAME),
        );
        if !f.luks_same_as_password {
            let mismatch = ui::mismatched(&f.luks, &f.luks_confirm);
            encryption = encryption.push(
                widget::container(
                    grid(look, 2, Space::S16)
                        .push(labelled(
                            look,
                            "Encryption passphrase",
                            None,
                            stack(look, Space::S8)
                                .push(input(app, F::Luks, "", None, look))
                                .push(ui::strength(look, &f.luks)),
                            None,
                        ))
                        .push(labelled(
                            look,
                            "Confirm the passphrase",
                            Some(show_toggle(app, look)),
                            input_state(app, F::LuksConfirm, "", None, invalid(mismatch), look),
                            ui::confirm_note(look, &f.luks, &f.luks_confirm),
                        )),
                )
                .id(ids::REVEAL_LUKS),
            );
        }
    }
    card(look, encryption)
}

fn git<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let f = &app.form;
    // Not while it's still being typed: only once you've left the field, or pressed Enter on it.
    let email = f.git_email.trim();
    let email_bad =
        !email.is_empty() && !model::looks_like_email(email) && (!app.is_focused(F::GitEmail) || app.nudged.is_some());
    let name = if f.git_name.trim().is_empty() {
        "…"
    } else {
        f.git_name.trim()
    };
    let email = if email.is_empty() { "…" } else { email };
    let entry = |key: &'a str, value: String| -> El<'a> {
        row(look, Gap::NONE)
            .push(text(key, Type::MonoLarge, c.cold))
            .push(text(value, Type::MonoLarge, c.on_surface))
            .into()
    };
    let preview = card(
        look,
        stack(look, Space::S4)
            .push(text("~/.gitconfig", Type::Mono, c.on_surface_variant))
            .push(gap(Space::S4))
            .push(text("[user]", Type::MonoLarge, c.primary))
            .push(entry("    name", format!(" = {name}")))
            .push(entry("    email", format!(" = {email}"))),
    );
    stack(look, Space::S32)
        .push(header(
            look,
            jp::GIT,
            "Git identity",
            "Who your commits say they're from. No email means you deal with it later.",
        ))
        .push(
            grid(look, 2, Space::S20)
                .push(labelled(
                    look,
                    "Name",
                    None,
                    input(app, F::GitName, "Your name", None, look),
                    None,
                ))
                .push(labelled(
                    look,
                    "Email",
                    None,
                    input_state(app, F::GitEmail, "you@example.com", None, invalid(email_bad), look),
                    email_bad.then(|| note(look, "That isn't an email address.", true)),
                )),
        )
        .push(preview)
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

fn wallpapers<'a>(app: &'a App, look: &Look) -> El<'a> {
    let sets = app.wallpaper_sets();
    let mut sections = stack(look, Space::S40).padding(Inset::NONE.right(Space::S16));
    for (s, set) in sets.iter().enumerate() {
        let cards = (0..set.files.len()).map(|i| wallpaper_card(app, s, i, look));
        sections = sections.push(
            stack(look, Space::S12)
                .push(wallpaper_set_header(app, s, set, look))
                .push(grid(look, WALLPAPER_COLUMNS as u16, Space::S16).extend(cards)),
        );
    }
    let apart = Space::S20;
    stack(look, apart)
        .height(Extent::Fill)
        .push(header(
            look,
            jp::WALLPAPERS,
            "Wallpapers",
            "Pick the ones the wallpaper picker shows; a set's name takes all of it. Every one gets installed \
             either way (later: ~/.config/kubaos/desktop/wallpapers/sets). Your own images go in \
             ~/Pictures/wallpapers.",
        ))
        .push(
            scroll(look, sections)
                .id(ids::WALLPAPER_SCROLL)
                .height(Extent::Fill)
                .room_above(apart)
                .room_below(FOOTER_GAP),
        )
        .into()
}

/// A set's header: its checkbox with its name, and a chip with how many of its wallpapers are
/// picked. Space or a click picks the rest of it, or drops it when it's all picked.
fn wallpaper_set_header<'a>(app: &'a App, s: usize, set: &'a kubaos_core::probe::WallpaperSet, look: &Look) -> El<'a> {
    let on = app.form.wallpapers_on(set);
    let tick = Tick::of(on, set.images);
    let count = match (tick, set.images) {
        (Tick::Full | Tick::Empty, 1) => "1 wallpaper".to_string(),
        (Tick::Full, n) => format!("all {n}"),
        (Tick::Empty, n) => format!("{n} wallpapers"),
        (Tick::Partial, n) => format!("{on} of {n}"),
    };
    let kind = match tick {
        Tick::Full => ChipKind::Selected,
        Tick::Partial | Tick::Empty => ChipKind::Neutral,
    };
    let f = F::WallpaperSet(s);
    widget::container(
        row(look, Space::S12)
            .center()
            .push(checkbox(
                look,
                set_name(&set.id),
                tick,
                Some(Message::Click(f)),
                app.is_focused(f),
            ))
            .push(chip(look, kind, count, None, None, None)),
    )
    .id(wallpaper_item(f).unwrap_or_default())
    .into()
}

/// A wallpaper to pick: the picture over the whole tile, its name over its foot; picked, the tube
/// and a check. Its box keeps the wallpaper's aspect until the thumbnail is decoded (or for good,
/// when it can't be), so the grid never jumps.
fn wallpaper_card<'a>(app: &'a App, s: usize, i: usize, look: &Look) -> El<'a> {
    let set = &app.wallpaper_sets()[s];
    let file = &set.files[i];
    let f = F::Wallpaper(s, i);
    let mut tile = picture_choice(
        app.thumbs.get(&model::wallpaper_key(set, file)).cloned(),
        Aspect::Wallpaper,
        wallpaper_label(&file.name),
    )
    .name(NamePlace::Over)
    .picked(app.form.wallpaper_on(set, file))
    .mark(Icon::Check, "Picked")
    .focused(app.is_focused(f))
    .on_press(Message::Click(f));
    if file.is_video() {
        tile = tile.badge(Icon::PlayArrow, "Video");
    }
    widget::container(tile.view(look))
        .id(wallpaper_item(f).unwrap_or_default())
        .into()
}

/// A card on Last chance, as tall as the one beside it.
fn block<'a>(id: &'static str, glyph: Icon, title: &'a str, facts: Vec<Fact<'a>>, look: &Look) -> El<'a> {
    let c = look.colors;
    widget::container(card_sized(
        look,
        stack(look, Space::S12)
            .push(
                row(look, Space::S8)
                    .center()
                    .push(icon(look, glyph, IconSize::S20, c.primary))
                    .push(text(title, Type::Subheading, c.on_surface)),
            )
            .push(description_list(look, facts)),
        Extent::Fill,
    ))
    .id(id)
    .height(Length::Fill)
    .into()
}

fn summary<'a>(app: &'a App, look: &Look) -> El<'a> {
    let Some(probe) = app.ready_probe() else {
        return widget::Space::new().into();
    };
    let f = &app.form;
    let cfg = f.to_config(probe);
    let disk = f.selected_disk(probe);
    let whole = f.disk_mode != Some(DiskMode::FreeSpace);

    let lang = summary_language(app, look);
    let acct = summary_account(app, &cfg, look);
    let disk_block = summary_disk(app, probe, disk, whole, look);
    let sys = summary_system(app, probe, &cfg, look);

    let mut top = stack(look, Space::S12).push(header(
        look,
        jp::SUMMARY,
        "Last chance",
        "Nothing's been written yet. Check it over, then hold the button.",
    ));
    // One line, so the warning takes no more of the cards' room than it needs, ending where the
    // cards do, short of their scrollbar's gutter.
    if let Some(note) = low_battery_note(app) {
        top = top.push(
            widget::container(banner(BannerTone::Warning, Icon::Warning, note, "").view(look))
                .padding(Inset::NONE.right(Space::S16).padding()),
        );
    }

    let apart = Space::S20;
    stack(look, apart)
        .height(Extent::Fill)
        .push(top)
        .push(
            scroll(
                look,
                widget::container(
                    grid(look, 2, Space::S16)
                        .uniform_rows()
                        .push(lang)
                        .push(acct)
                        .push(disk_block)
                        .push(sys),
                )
                .padding(Inset::NONE.right(Space::S16).padding()),
            )
            .id(ids::SUMMARY_SCROLL)
            .height(Extent::Fill)
            .room_above(apart)
            .room_below(apart),
        )
        // The cards keep a gutter for their scrollbar; the banner ends where they do.
        .push(widget::container(confirm(app, disk, whole, look)).padding(Inset::NONE.right(Space::S16).padding()))
        .into()
}

fn summary_language<'a>(app: &'a App, look: &Look) -> El<'a> {
    let f = &app.form;
    block(
        ids::SUMMARY_LANGUAGE,
        Icon::Language,
        "Language",
        vec![
            Fact::new("Language", data::locale_name(&f.locale).into_owned()),
            Fact::new(
                "Keyboard",
                f.keymaps
                    .iter()
                    .map(|k| data::keymap_name(k))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Fact::new("Time zone", {
                let (city, region) = data::split_tz(&f.timezone);
                if region.is_empty() {
                    city
                } else {
                    format!("{city} ({region})")
                }
            }),
        ],
        look,
    )
}

fn summary_account<'a>(app: &'a App, cfg: &InstallConfig, look: &Look) -> El<'a> {
    let f = &app.form;
    block(
        ids::SUMMARY_ACCOUNT,
        Icon::Person,
        "You",
        vec![
            Fact::new("Name", f.full_name.clone()),
            Fact::new("Login", format!("{}@{}", cfg.user.username, cfg.host.hostname)),
            match cfg.git.as_ref() {
                Some(g) => Fact::new("Git", g.name.clone()).detail(g.email.clone()),
                None => Fact::new("Git", "Later".to_owned()),
            },
        ],
        look,
    )
}

fn summary_disk<'a>(app: &'a App, probe: &Probe, disk: Option<&DiskInfo>, whole: bool, look: &Look) -> El<'a> {
    let f = &app.form;
    block(
        ids::SUMMARY_DISK,
        Icon::HardDrive,
        "Disk",
        vec![
            match disk {
                Some(d) => {
                    Fact::new("Disk", disk_model(d)).detail(format!("{} · {}", d.path, human_bytes(d.size_bytes)))
                }
                None => Fact::new("Disk", "None picked".to_owned()),
            },
            Fact::new(
                "Mode",
                if whole {
                    "Erase everything".to_owned()
                } else {
                    format!(
                        "Alongside, in {} free",
                        human_bytes(f.free_region(probe).map_or(0, |r| r.end - r.start))
                    )
                },
            ),
            Fact::new(
                "Encryption",
                match (f.encrypt, f.luks_same_as_password) {
                    (false, _) => "Off. Brave.".to_owned(),
                    (true, true) => "LUKS2, account password".to_owned(),
                    (true, false) => "LUKS2, its own passphrase".to_owned(),
                },
            ),
        ],
        look,
    )
}

fn summary_system<'a>(app: &'a App, probe: &Probe, cfg: &InstallConfig, look: &Look) -> El<'a> {
    let m = probe.payload.as_ref();
    block(
        ids::SUMMARY_SYSTEM,
        Icon::DeployedCode,
        "System",
        vec![
            Fact::new("Release", format!("KubaOS {}", app.codename())),
            Fact::new(
                "Image",
                m.map(|m| format!("{} · {}", m.version, m.commit.chars().take(7).collect::<String>()))
                    .unwrap_or_default(),
            )
            .detail(m.map(|m| format!("Linux {}", m.kernel)).unwrap_or_default()),
            Fact::new(
                "Wallpapers",
                wallpapers_summary(cfg.wallpapers.as_ref(), app.wallpaper_sets()),
            ),
            Fact::new(
                "Network",
                app.form
                    .wifi_ssid
                    .clone()
                    .map_or_else(|| "None".to_owned(), |s| format!("Wi-Fi “{s}”")),
            ),
        ],
        look,
    )
}

/// What the install does to the disk, as a banner holding Back and the button to hold, the hold's
/// bar along its foot. Erasing is a danger banner, really red, naming the disk, with no jokes in
/// that sentence (docs/BRAND.md, voice rule 4); going into free space is an info banner.
fn confirm<'a>(app: &'a App, disk: Option<&'a DiskInfo>, whole: bool, look: &Look) -> El<'a> {
    let c = look.colors;
    let held = app
        .hold
        .map_or(0.0, |s| ((app.now - s).as_secs_f32() / HOLD.as_secs_f32()).min(1.0));
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
    let (tone, glyph, kind, message, sub, cta, ink) = if whole {
        let n = disk.map_or(0, |d| d.partitions.len());
        let others = disk
            .and_then(|d| d.other_os())
            .map(|o| format!(", {o} included"))
            .unwrap_or_default();
        (
            BannerTone::Danger,
            Icon::Warning,
            Kind::Danger,
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
            color::alpha(c.on_error_container, Opacity::SubduedText.value()),
        )
    } else {
        (
            BannerTone::Info,
            Icon::Storage,
            Kind::Filled,
            format!("KubaOS goes into the free space on {disk_name}."),
            format!(
                "{} stays where it is.",
                disk.and_then(|d| d.other_os()).unwrap_or("Everything else")
            ),
            "Hold to install",
            c.on_surface_variant,
        )
    };
    let label = if app.hold.is_some() { "Keep holding…" } else { cta };
    let on_banner = kubaos_ui::component::banner::inside(look, tone);
    let back = button("Back")
        .kind(Kind::Text)
        .icon(Icon::ArrowBack)
        .on_press(Message::Click(F::Back))
        .focused(app.is_focused(F::Back))
        .view(&on_banner);
    let hold_button = stack(look, Space::S8)
        .push(
            button(label)
                .kind(kind)
                .size(ButtonSize::Lg)
                .icon(Icon::Download)
                .on_hold(Message::HoldStart, Message::HoldEnd)
                .focused(app.is_focused(F::Hold))
                .view(look),
        )
        .push(
            row(look, Space::S4)
                .center()
                .push(text("or hold", Type::Caption, ink))
                .push(keycap(look, "Enter")),
        );
    banner(tone, glyph, message, sub)
        .actions([back, hold_button.into()])
        .foot(if tone == BannerTone::Danger {
            danger_timed_bar(look, held)
        } else {
            timed_bar(look, held)
        })
        .view(look)
}

/// Asks for the charger on Last chance when the battery is low. It warns only: an install on a low
/// battery is the owner's call.
pub fn low_battery_note(app: &App) -> Option<String> {
    let b = app.battery.filter(|b| b.is_low())?;
    Some(format!("{}% left. Plug the damn thing in.", b.percent))
}

fn step_lead<'a>(state: StepState, seconds: f32, look: &Look) -> El<'a> {
    let c = look.colors;
    match state {
        StepState::Done => icon(look, Icon::CheckCircle, IconSize::S20, c.primary),
        StepState::Running => spinner(look, IconSize::S20, seconds, c.primary),
        StepState::Failed => icon(look, Icon::Error, IconSize::S20, c.error),
        StepState::Pending => icon(look, Icon::Circle, IconSize::S20, c.outline_variant),
    }
}

fn steps_list<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let seconds = (app.now - app.epoch).as_secs_f32();
    let rows: Vec<El> = inst
        .steps
        .iter()
        .map(|s| {
            let timing = match (s.state, s.elapsed, s.started) {
                (_, Some(e), _) => fmt_duration(e),
                (StepState::Running, None, Some(st)) => fmt_duration(app.now.saturating_duration_since(st)),
                _ => String::new(),
            };
            let (colour, style) = match s.state {
                StepState::Running => (c.on_surface, Type::BodyStrong),
                StepState::Done => (c.on_surface, Type::Body),
                StepState::Failed => (c.error, Type::BodyStrong),
                StepState::Pending => (c.on_surface_variant, Type::Body),
            };
            let mut r = row(look, Space::S12)
                .center()
                .push(step_lead(s.state, seconds, look))
                .push(text(s.info.title.clone(), style, colour))
                .push(fill());
            if s.state == StepState::Running && s.fraction > 0.0 && s.fraction < 1.0 {
                r = r.push(text(format!("{:.0}%", s.fraction * 100.0), Type::Mono, c.primary));
            }
            r.push(text(timing, Type::Mono, c.on_surface_variant)).into()
        })
        .collect();
    // Two columns, down the first then the second: the list is short and the panel is wide.
    let half = rows.len().div_ceil(2);
    let mut first = stack(look, Space::S8);
    let mut second = stack(look, Space::S8);
    for (i, r) in rows.into_iter().enumerate() {
        if i < half {
            first = first.push(r);
        } else {
            second = second.push(r);
        }
    }
    grid(look, 2, Space::S40).push(first).push(second).into()
}

/// The install's log, its newest lines, `height` tall.
fn log_view<'a>(app: &'a App, look: &Look, height: Extent) -> El<'a> {
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let skip = inst.log.len().saturating_sub(LOG_VIEW_LINES);
    let lines: Vec<(LogLevel, &str)> = inst.log[skip..]
        .iter()
        .map(|l| {
            let level = match l.level {
                Level::Error => LogLevel::Error,
                Level::Warn => LogLevel::Warn,
                Level::Info if l.text.starts_with("==>") => LogLevel::Step,
                Level::Info => LogLevel::Info,
                Level::Debug => LogLevel::Debug,
            };
            (level, l.text.as_str())
        })
        .collect();
    log(look, &lines, height)
}

/// How far the log is open, 0 to 1: it opens over medium as things arrive, and closes the same way.
fn log_openness(app: &App) -> f32 {
    let shown = Motion::default().progress(
        tokens::Duration::Medium,
        Easing::EmphasizedDecel,
        app.now - app.log_open.1,
    );
    if app.log_open.0 { shown } else { 1.0 - shown }
}

fn log_toggle<'a>(app: &'a App, look: &Look) -> El<'a> {
    button(if app.log_open.0 { "Hide log" } else { "Show log" })
        .kind(Kind::Text)
        .icon(Icon::Terminal)
        .on_press(Message::Click(F::ToggleLog))
        .focused(app.is_focused(F::ToggleLog))
        .view(look)
}

fn installing<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let current = inst
        .current()
        .map_or_else(|| "Starting…".into(), |s| s.info.title.clone());
    let detail = inst.detail.clone().unwrap_or_default();
    let elapsed = fmt_duration(app.now.saturating_duration_since(inst.started));
    let open = look.scale.snap(log_openness(app) * f32::from(Size::Log.px()));

    let mut page = stack(look, Gap::NONE)
        .height(Extent::Fill)
        .push(
            row(look, Space::S16)
                .bottom()
                .push(header(
                    look,
                    jp::INSTALLING,
                    "Installing",
                    "Go stare at a wall, it'll be done before you get bored.",
                ))
                .push(fill())
                .push(text(
                    format!("{:.0}%", inst.shown * 100.0),
                    Type::HeroNumber,
                    c.on_surface,
                )),
        )
        .push(gap(Space::S20))
        // `shown` already eases toward the work on every tick.
        .push(timed_bar(look, inst.shown))
        .push(gap(Space::S12))
        .push(
            row(look, Space::S12)
                .center()
                .push(text(current, Type::BodyStrong, c.on_surface))
                .push(text(detail, Type::Mono, c.primary))
                .push(fill())
                .push(icon(look, Icon::Timer, IconSize::S16, c.on_surface_variant))
                .push(text(elapsed, Type::Mono, c.on_surface_variant)),
        )
        .push(gap(Space::S32))
        .push(widget::container(steps_list(app, look)).height(Length::Fill).clip(true));
    if open > 0.0 {
        page = page
            .push(
                widget::container(log_view(app, look, Extent::Fill))
                    .height(open)
                    .clip(true),
            )
            .push(gap(Space::S12));
    }
    let last = inst
        .log
        .last()
        .filter(|_| !app.log_open.0)
        .map(|l| l.text.clone())
        .unwrap_or_default();
    page.push(row(look, Space::S12).center().push(log_toggle(app, look)).push(line(
        last,
        Type::Mono,
        c.on_surface_variant,
    )))
    .into()
}

fn failed<'a>(app: &'a App, look: &Look) -> El<'a> {
    let Some(inst) = &app.install else {
        return widget::Space::new().into();
    };
    let (step, error, hint) = match &inst.outcome {
        Some(Outcome::Failed { step, error, hint }) => (*step, error.as_str(), hint.clone()),
        _ => (None, "No idea what happened", None),
    };
    let at = step
        .and_then(|id| inst.steps.iter().find(|s| s.info.id == id))
        .map_or_else(
            || "The install stopped before it got anywhere.".into(),
            |s| {
                format!(
                    "“{}” died{}. Nothing after it ran.",
                    s.info.title,
                    s.elapsed
                        .map(|e| format!(" after {}", fmt_duration(e)))
                        .unwrap_or_default()
                )
            },
        );
    let hint = hint.unwrap_or_else(|| "The log has the gory details. Try again, or save it and go read it.".into());
    let open = log_openness(app);
    let mut page = stack(look, Gap::NONE)
        .height(Extent::Fill)
        .push(header(look, jp::FAILED, "Fuck.", at))
        .push(gap(Space::S20))
        .push(error_block(look, error, hint))
        .push(gap(Space::S16));
    // The log takes its share of the room as it opens, and gives it back as it closes. Open or
    // closed, one of the two has all of it, so its edges are whole device pixels.
    let share = (open * 100.0) as u16;
    let (log_room, rest) = match share {
        0 => (None, Length::Fill),
        100.. => (Some(Length::Fill), Length::Shrink),
        _ => (Some(Length::FillPortion(share)), Length::FillPortion(100 - share)),
    };
    if let Some(room) = log_room {
        page = page.push(widget::container(log_view(app, look, Extent::Fill)).height(room));
    }
    page.push(widget::Space::new().height(rest))
        .push(gap(Space::S16))
        .push(
            row(look, Space::S12)
                .center()
                .push(
                    button("Back to the review")
                        .kind(Kind::Text)
                        .icon(Icon::ArrowBack)
                        .on_press(Message::Click(F::Back))
                        .focused(app.is_focused(F::Back))
                        .view(look),
                )
                .push(log_toggle(app, look))
                .push(fill())
                .push(
                    button("Save log")
                        .kind(Kind::Tonal)
                        .icon(Icon::Save)
                        .on_press(Message::Click(F::SaveLog))
                        .focused(app.is_focused(F::SaveLog))
                        .view(look),
                )
                .push(
                    button("Try again")
                        .icon(Icon::RestartAlt)
                        .on_press(Message::Click(F::Retry))
                        .focused(app.is_focused(F::Retry))
                        .view(look),
                ),
        )
        .into()
}

// ----------------------------------------------------------------- hero --

fn hero<'a>(app: &'a App, look: &Look) -> El<'a> {
    // The done screen brings each part in itself (`crate::done`).
    let shown = if app.screen == Screen::Done { 1.0 } else { app.fade() };
    let look = look.faded(shown);
    let inner: El = match app.screen {
        Screen::Welcome => welcome(app, &look),
        _ => done(app, &look),
    };
    let edge = f32::from(if wide(app.size) { Space::S64 } else { Space::S40 }.px());
    // The words rise into place as they fade in, a whole device pixel at a time.
    let rise = look.scale.snap((1.0 - shown) * f32::from(Space::S24.px()));
    widget::container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_bottom(Length::Fill)
        .padding(iced::Padding {
            top: edge,
            right: edge,
            bottom: (edge - rise).max(0.0),
            left: edge,
        })
        .into()
}

/// A lede under a full-screen line: as wide as a column of text at most.
fn lede<'a>(words: impl Into<String>, look: &Look) -> El<'a> {
    widget::container(text(words.into(), Type::BodyLarge, look.colors.on_surface_variant))
        .max_width(f32::from(Size::Column.px()))
        .into()
}

fn welcome<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let status = welcome_status(app, look);
    let meta = welcome_meta(app, look);
    let action = welcome_action(app, look);
    // No image, nothing to start: no "Enter: start" either, and the line keeps its room.
    let keys = if app.ready_probe().is_some_and(|p| p.payload.is_none()) {
        Vec::new()
    } else {
        hints(app)
    };
    stack(look, Gap::NONE)
        .push(text(jp::WELCOME, Type::Accent, c.primary))
        .push(gap(Space::S8))
        .push(row(look, Space::S20).center().push(sign(look)).push(chip(
            look,
            ChipKind::Neutral,
            app.codename().to_uppercase(),
            None,
            None,
            None,
        )))
        .push(gap(Space::S20))
        .push(text("Let's fix this shit.", Type::Hero, c.on_surface))
        .push(gap(Space::S16))
        .push(lede(
            "Whatever's on here, it's had its chance. Kuba's own Linux: a fast, keyboard-driven desktop for \
             writing Rust all day without the OS getting in the way. I update when you say so, I never nag, and \
             if you break me there's always a way back.",
            look,
        ))
        .push(gap(Space::S40))
        .push(row(look, Space::S20).center().push(action).push(status))
        .push(gap(Space::S16))
        .push(ui::hint_row(look, &keys))
        .push(gap(Space::S32))
        .push(row(look, Space::S8).center().extend(meta))
        .into()
}

/// The probe's progress beside the start button, or why there's nothing to start.
fn welcome_status<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let seconds = (app.now - app.epoch).as_secs_f32();
    let refused = |words: String| -> El<'a> {
        row(look, Space::S12)
            .center()
            .push(icon(look, Icon::Error, IconSize::S20, c.error))
            .push(widget::container(text(words, Type::Body, c.error)).max_width(f32::from(Size::Column.px())))
            .into()
    };
    match &app.probe {
        ProbeState::Loading => row(look, Space::S12)
            .center()
            .push(spinner(look, IconSize::S20, seconds, c.primary))
            .push(text("Poking at the hardware…", Type::Body, c.on_surface_variant))
            .into(),
        ProbeState::Failed(e) => refused(format!("Couldn't read this machine: {e}.")),
        ProbeState::Ready(pr) if pr.payload.is_none() => refused(
            "No system image on this stick, so I've got nothing to install. Rebuild the ISO and flash it again.".into(),
        ),
        ProbeState::Ready(_) => widget::Space::new().into(),
    }
}

/// The chips under the welcome: the image's version, commit and build date, and the machine.
fn welcome_meta<'a>(app: &'a App, look: &Look) -> Vec<El<'a>> {
    let mut meta: Vec<El> = vec![];
    if let Some(pr) = app.ready_probe() {
        if let Some(m) = &pr.payload {
            meta.push(chip(
                look,
                ChipKind::Accent,
                format!("v{}", m.version),
                Some(Icon::DeployedCode),
                None,
                None,
            ));
            meta.push(chip(
                look,
                ChipKind::Neutral,
                m.commit.chars().take(7).collect::<String>(),
                None,
                None,
                None,
            ));
            meta.push(chip(
                look,
                ChipKind::Neutral,
                format!("built {}", m.built_at.get(..10).unwrap_or(&m.built_at)),
                None,
                None,
                None,
            ));
        }
        meta.push(chip(
            look,
            ChipKind::Neutral,
            format!("{}{}", pr.arch, if pr.uefi { " · UEFI" } else { " · BIOS" }),
            None,
            None,
            None,
        ));
    }
    meta
}

fn welcome_action<'a>(app: &'a App, look: &Look) -> El<'a> {
    if matches!(app.probe, ProbeState::Failed(_)) {
        button("Try again")
            .size(ButtonSize::Lg)
            .icon(Icon::RestartAlt)
            .on_press(Message::Click(F::RetryProbe))
            .focused(app.is_focused(F::RetryProbe))
            .view(look)
    } else {
        button("Let's go")
            .size(ButtonSize::Lg)
            .trailing_icon(Icon::ArrowForward)
            .on_press_maybe(app.blocker().is_none().then_some(Message::Click(F::Start)))
            .focused(app.is_focused(F::Start))
            .view(look)
    }
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

fn done<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let took = match app.install.as_ref().and_then(|i| i.outcome.clone()) {
        Some(Outcome::Done { elapsed }) => elapsed,
        _ => Duration::ZERO,
    };
    let user = app.form.username.clone();
    // tty1 logs in by itself and starts the desktop: the only thing to type at boot is the disk
    // passphrase.
    let boot = if app.form.encrypt {
        format!("Type the disk passphrase at boot and you're on the desktop as {user}.")
    } else {
        format!("You land on the desktop as {user}.")
    };
    // Every part arrives on the sequence's cues, rising onto whole device pixels; none of them
    // waits for it, so Reboot works from the first frame.
    let seq = done::Sequence::new(app.transition.0, app.now, app.motion);
    let rising = |reveal: done::Reveal, part: El<'a>| -> El<'a> {
        done::shift(part, Vector::new(0.0, look.scale.snap(reveal.below)))
    };
    let headline = DONE_HEADLINE.split(' ').zip(0..).map(|(word, i)| {
        let reveal = seq.word(i);
        // Each word carries the space before it, so the line keeps its own word spacing.
        let word = if i == 0 { word.to_owned() } else { format!(" {word}") };
        rising(reveal, text(word, Type::Hero, color::alpha(c.on_surface, reveal.shown)))
    });
    let lede_reveal = seq.lede();
    let says = lede(
        format!("That's it. {} {boot}", done_line(took)),
        &look.faded(lede_reveal.shown),
    );
    let actions = seq.actions();
    let hints_reveal = seq.hints();
    let column = stack(look, Gap::NONE)
        .push(done::sign_off(look, app.transition.0, app.now, app.motion))
        .push(gap(Space::S24))
        .push(row(look, Gap::NONE).extend(headline))
        .push(gap(Space::S16))
        .push(rising(lede_reveal, says))
        .push(gap(Space::S40))
        .push(rising(actions, done_actions(app, &look.faded(actions.shown))))
        .push(gap(Space::S16))
        .push(ui::hint_row(&look.faded(hints_reveal.shown), &hints(app)));
    let jolt = seq.jolt();
    done::shift(column, Vector::new(look.scale.snap(jolt.x), look.scale.snap(jolt.y)))
}

/// The done screen's headline, the words of the draft Kuba picked.
const DONE_HEADLINE: &str = "Fixed. You're welcome.";

/// Reboot, or how far the machine is in going down, beside staying in the live system.
fn done_actions<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let reboot: El = if let Some(action) = app.shutting_down {
        row(look, Space::S12)
            .center()
            .push(spinner(
                look,
                IconSize::S20,
                (app.now - app.epoch).as_secs_f32(),
                c.primary,
            ))
            .push(text(going_down(action), Type::Body, c.on_surface_variant))
            .into()
    } else {
        button("Reboot")
            .size(ButtonSize::Lg)
            .icon(Icon::PowerSettingsNew)
            .on_press(Message::Click(F::Reboot))
            .focused(app.is_focused(F::Reboot))
            .view(look)
    };
    row(look, Space::S12)
        .center()
        .push(reboot)
        .push(
            button("Stay in the live system")
                .kind(Kind::Text)
                .size(ButtonSize::Lg)
                .on_press(Message::Click(F::Stay))
                .focused(app.is_focused(F::Stay))
                .view(look),
        )
        .into()
}

// ---------------------------------------------------------------- power --

/// The button that opens the power menu, on every screen.
fn power_button<'a>(app: &'a App, look: &Look) -> El<'a> {
    button("Power")
        .kind(Kind::Text)
        .size(ButtonSize::Sm)
        .icon(Icon::PowerSettingsNew)
        .on_press(Message::Click(F::Power))
        .focused(app.is_focused(F::Power))
        .view(look)
}

/// What the power menu says, for where the install stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerCopy {
    pub title: &'static str,
    pub body: String,
    /// Going down now loses work: the menu is a danger dialog.
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
            "The install stops where it is. {disk} is left partly written and won't boot: install KubaOS on it \
             again to use it."
        ),
        Some(Some(Outcome::Done { .. })) => {
            "KubaOS is installed. Pull the USB stick first, or the machine starts this installer again.".to_string()
        }
        Some(Some(Outcome::Failed { .. })) => {
            format!("The install didn't finish, so {disk} won't boot until KubaOS is installed on it again.")
        }
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
        off: if running { "Stop and power off" } else { "Power off" },
        restart: if running { "Stop and restart" } else { "Restart" },
        status,
    }
}

fn going_down(action: PowerAction) -> &'static str {
    match action {
        PowerAction::PowerOff => "Powering off…",
        PowerAction::Restart => "Restarting…",
    }
}

/// The power menu, a dialog over the dimmed screen. It takes every click: one outside it cancels,
/// and nothing behind it reacts.
fn power_menu<'a>(app: &'a App, look: &Look) -> El<'a> {
    let c = look.colors;
    let copy = power_copy(app);
    let busy = copy.status.is_some();
    let tone = if copy.danger { Tone::Danger } else { Tone::Normal };
    let (dismiss, actions): (Option<El>, Vec<El>) = match copy.status {
        Some(status) => (
            None,
            vec![
                row(look, Space::S12)
                    .center()
                    .push(spinner(
                        look,
                        IconSize::S20,
                        (app.now - app.epoch).as_secs_f32(),
                        if copy.danger { c.error } else { c.primary },
                    ))
                    .push(text(status, Type::Body, c.on_surface_variant))
                    .into(),
            ],
        ),
        None => (
            Some(
                button("Cancel")
                    .kind(Kind::Text)
                    .on_press(Message::Click(F::PowerCancel))
                    .focused(app.is_focused(F::PowerCancel))
                    .view(look),
            ),
            vec![
                button(copy.restart)
                    .kind(Kind::Tonal)
                    .icon(Icon::RestartAlt)
                    .on_press(Message::Click(F::PowerRestart))
                    .focused(app.is_focused(F::PowerRestart))
                    .view(look),
                button(copy.off)
                    .kind(if copy.danger { Kind::Danger } else { Kind::Filled })
                    .icon(Icon::PowerSettingsNew)
                    .on_press(Message::Click(F::PowerOff))
                    .focused(app.is_focused(F::PowerOff))
                    .view(look),
            ],
        ),
    };
    let menu = dialog(
        look,
        tone,
        Icon::PowerSettingsNew,
        copy.title,
        copy.body,
        dismiss,
        actions,
    );
    let mut area = widget::mouse_area(scrim(look, widget::opaque(menu)));
    if !busy {
        area = area.on_press(Message::Click(F::PowerCancel));
    }
    widget::opaque(area)
}

#[cfg(test)]
mod tests;
