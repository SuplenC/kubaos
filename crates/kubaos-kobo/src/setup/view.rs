//! The setup screen: the window is one panel, the cards on the left, the selected card on the
//! right. Built from `kubaos-ui`'s layout and components, with its tokens.
//!
//! Copy per docs/BRAND.md: attitude around the facts, never in the sentence that matters (the
//! MOK password, the blue screen).

mod bitwarden;
mod detail;
mod explain;
mod form;
mod phone;
mod side;
mod tuxedo;

use iced::Length;
use iced::widget;
use iced::widget::container;
use kubaos_core::setup::CardId;
use kubaos_core::setup::State;
use kubaos_ui::Look;
use kubaos_ui::component::banner::BannerTone;
use kubaos_ui::component::banner::banner;
use kubaos_ui::component::card::card;
use kubaos_ui::component::check::radio;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::field::State as FieldState;
use kubaos_ui::component::field::field;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::flow;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use self::detail::detail;
use self::side::left;
use super::CARDS_SCROLL;
use super::Msg;
use super::Run;
use super::SF;
use super::Setup;
use super::card_item;
use crate::app::Message;
use crate::ui;
use crate::ui::El;

fn click(f: SF) -> Message {
    Message::Setup(super::Msg::Click(f))
}

pub fn view(s: &Setup) -> El<'_> {
    let look = s.look();
    let mut layers = widget::stack![panel(s, &look)].width(Length::Fill).height(Length::Fill);
    if s.backend.is_demo() {
        layers = layers.push(
            widget::container(chip(ChipKind::Cold, "Demo").icon(Icon::Bolt).view(&look))
                .width(Length::Fill)
                .align_right(Length::Fill)
                .padding(Inset::all(Space::S16).padding()),
        );
    }
    if let Some(toast) = ui::toast(&look, s.toast.as_ref(), s.now) {
        layers = layers.push(toast);
    }
    crate::pointer::idle(layers, s.pointer_idle).into()
}

/// The window is the panel. Setup runs as a floating window in Hyprland, which draws the border,
/// the rounding and the shadow, with the desktop right there around it. So no photo and no card
/// inside a card: a solid surface, edge to edge.
fn panel<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let surface = look.colors.surface;
    widget::container(row(look, Gap::NONE).push(left(s, look)).push(detail(s, look)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style::default().background(surface))
        .into()
}

fn card_icon(id: CardId) -> Icon {
    match id {
        CardId::Tuxedo => Icon::Computer,
        CardId::Fingerprint => Icon::Fingerprint,
        CardId::Yubikey => Icon::Key,
        CardId::Bitwarden => Icon::Lock,
        CardId::Git => Icon::Code,
        // The link runs over the Wi-Fi.
        CardId::Phone => Icon::Wifi,
    }
}

/// A card's state as a chip, as its row in the list shows it: to do plain, done tonal with a
/// check, a reboot on the accent.
fn state_chip<'a>(look: &Look, st: State) -> El<'a> {
    let (kind, glyph, word) = match st {
        State::Done => (ChipKind::Accent, Some(Icon::Check), "Done"),
        State::Needed => (ChipKind::Neutral, None, "To do"),
        State::Reboot => (ChipKind::Filled, Some(Icon::RestartAlt), "Reboot"),
        State::Waiting => (ChipKind::Neutral, Some(Icon::Schedule), "Waiting"),
        State::Absent => (ChipKind::Neutral, None, "Not here"),
    };
    chip(kind, word).icon_maybe(glyph).view(look)
}

fn para<'a>(look: &Look, words: impl Into<String>) -> El<'a> {
    text(words.into(), Type::Body, look.colors.on_surface_variant)
}

/// A warning: something that goes wrong unless it is done right, as a banner with its symbol, its
/// first line and the rest under it.
fn warning<'a>(look: &Look, glyph: Icon, head: &'a str, lines: Vec<String>) -> El<'a> {
    let mut lines = lines.into_iter();
    banner(BannerTone::Warning, glyph, head, lines.next().unwrap_or_default())
        .paragraphs(lines)
        .view(look)
}

/// The serious kind of note: no jokes, a card with its symbol and its first line.
fn notice<'a>(look: &Look, glyph: Icon, head: &'a str, lines: Vec<String>) -> El<'a> {
    let c = look.colors;
    card(
        look,
        stack(look, Space::S8)
            .push(
                row(look, Space::S8)
                    .center()
                    .push(icon(look, glyph, IconSize::S20, c.primary))
                    .push(text(head, Type::Subheading, c.on_surface)),
            )
            .extend(lines.into_iter().map(|line| para(look, line))),
    )
}

/// A field of the card's form.
fn input<'a>(s: &'a Setup, f: SF, placeholder: &'a str, glyph: Option<Icon>, look: &Look) -> El<'a> {
    input_state(s, f, placeholder, glyph, FieldState::Normal, look)
}

/// [`input`] in `state`: invalid shows the error colour, and the reason goes under it. While a card
/// runs, every field is locked.
fn input_state<'a>(
    s: &'a Setup,
    f: SF,
    placeholder: &'a str,
    glyph: Option<Icon>,
    state: FieldState,
    look: &Look,
) -> El<'a> {
    let state = if s.running() { FieldState::Locked } else { state };
    let mut input = field(s.field_value(f), placeholder)
        .state(state)
        .blink_from(s.focus_at)
        .on_pointer(move |pointer| Message::Setup(super::Msg::Pointer(f, pointer)));
    if let Some(glyph) = glyph {
        input = input.icon(glyph);
    }
    if f.is_secret() && !s.show_secrets {
        input = input.secret();
    }
    if s.is_focused(f) {
        input = input.caret(s.caret);
    }
    input.view(look)
}

/// One of a few choices, as radio buttons: Left and Right (or Space) move between them, a click
/// picks one. The picked one has the focus tube while the choice has the keyboard.
fn choice<'a>(s: &'a Setup, f: SF, options: Vec<(&'a str, bool, Message)>, look: &Look) -> El<'a> {
    let focused = s.is_focused(f);
    let radios = options
        .into_iter()
        .map(|(label, on, pick)| radio(look, label, on, (!s.running()).then_some(pick), focused && on));
    flow(look, Space::S20, radios)
}
