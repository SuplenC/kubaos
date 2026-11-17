//! Everything on screen. The panel screens share the glass panel with the step rail; Welcome and
//! Done are full-bleed over the wallpaper. The power menu opens over either. The battery sits
//! beside the power button on both. First-boot setup is its own app (`setup/`).
//!
//! Every view is built from `kubaos-ui`'s layout and components, with its tokens: no length,
//! colour, duration or font is written here.
//!
//! Copy follows docs/BRAND.md: plain facts first, attitude around them, at most one swear per
//! screen, none near the disk that gets erased or the passphrase prompts.

mod account;
mod chrome;
mod disk;
mod done;
mod git;
mod installing;
mod language;
mod network;
mod power;
mod review;
mod timezone;
mod wallpapers;
mod welcome;

use std::f32::consts::PI;
use std::sync::LazyLock;

use iced::Background;
use iced::ContentFit;
use iced::Gradient;
use iced::Length;
use iced::Radians;
use iced::gradient::Linear;
use iced::widget;
use iced::widget::container;
use iced::widget::image;
use iced::widget::svg;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::field::State;
use kubaos_ui::component::field::field;
use kubaos_ui::component::list::scrolling_list;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::page_clear;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::motion::Motion;
use kubaos_ui::text::text;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Mode;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Rgb;
use kubaos_ui::tokens::Role;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;
use kubaos_ui::tokens::palette;

use self::account::account;
use self::chrome::FOOTER_GAP;
use self::chrome::footer;
#[cfg(test)]
pub use self::chrome::hints;
use self::chrome::panel;
pub use self::chrome::panel_rect;
use self::chrome::wide;
use self::disk::disk;
use self::done::done;
#[cfg(test)]
pub use self::done::done_line;
use self::git::git;
use self::installing::failed;
use self::installing::installing;
#[cfg(test)]
pub use self::installing::low_battery_note;
use self::language::language;
use self::network::wifi;
use self::power::power_button;
#[cfg(test)]
pub use self::power::power_copy;
use self::power::power_menu;
use self::review::summary;
use self::timezone::timezone;
use self::wallpapers::wallpapers;
use self::welcome::welcome;
use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::app::Screen;
use crate::theme;
use crate::ui;
use crate::ui::El;

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
        corner = corner.push(chip(ChipKind::Cold, "Demo").icon(Icon::Bolt).view(&look));
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

#[cfg(test)]
mod tests;
