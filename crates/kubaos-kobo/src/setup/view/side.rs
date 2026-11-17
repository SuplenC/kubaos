//! The window's side: the header, the cards as a list, whether the machine is online, Done for
//! now and the keys, giving way in turn as the window gets shorter.

use iced::Background;
use iced::Length;
use iced::widget;
use iced::widget::container;
use kubaos_core::setup::CardStatus;
use kubaos_core::setup::State;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::list::scrolling_list;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::component::step::StepState;
use kubaos_ui::component::step::step;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::flow;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::header;
use kubaos_ui::layout::page_clear;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Opacity;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::card_icon;
use super::click;
use crate::app::Message;
use crate::setup::SF;
use crate::setup::Setup;
use crate::theme::jp;
use crate::ui;
use crate::ui::El;

pub(super) fn left<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let c = look.colors;
    let seconds = (s.now - s.epoch).as_secs_f32();
    let cards = left_cards(s, seconds, look);
    let online = left_online(s, look);
    let footer = left_footer(s, look);
    let detecting: El = if s.detecting() && s.status.is_some() {
        row(look, Space::S8)
            .center()
            .push(spinner(look, IconSize::S16, seconds, c.primary))
            .push(text("Checking again…", Type::Small, c.on_surface_variant))
            .into()
    } else {
        widget::Space::new().into()
    };
    let rail = color::alpha(c.surface_container_low, Opacity::Well.value());
    widget::container(page_clear(
        stack(look, Gap::NONE)
            .height(Extent::Fill)
            .push(header(
                look,
                jp::SETUP,
                "First boot",
                "The things the install couldn't do without you or the hardware. Each one checks itself, so done \
                 stays done.",
            ))
            .push(gap(Space::S20))
            .push(cards)
            .push(gap(Space::S12))
            .push(detecting)
            .push(gap(Space::S12))
            .push(online)
            .push(gap(Space::S16))
            .push(footer)
            .push(gap(Space::S16))
            .push(flow(
                look,
                Space::S16,
                [
                    ("↑ ↓", "pick a card"),
                    ("Tab", "next control"),
                    ("Enter", "open, press"),
                    ("Esc", "back to the cards"),
                ]
                .map(|(key, what)| ui::hint(look, key, what)),
            )),
    ))
    .width(side_width(s, look))
    .height(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(rail)),
        ..container::Style::default()
    })
    .into()
}

/// The side panel's width: a third of the window, so the card's page beside it keeps the room, and
/// never wider than a column of text. On a whole device pixel.
fn side_width(s: &Setup, look: &Look) -> f32 {
    let third = s.size.width / 3.0;
    look.scale.snap(third.min(f32::from(Size::Column.px())))
}

/// The cards, or a spinner while the first detection runs. The cards take the room the panel has
/// left and scroll in their well past it: their summaries wrap, so six of them can outgrow it.
fn left_cards<'a>(s: &'a Setup, seconds: f32, look: &Look) -> El<'a> {
    let c = look.colors;
    match &s.status {
        None => widget::container(
            row(look, Space::S12)
                .center()
                .padding(Inset::xy(Space::S12, Space::S12))
                .push(spinner(look, IconSize::S20, seconds, c.primary))
                .push(text(
                    "Checking what this machine has…",
                    Type::Body,
                    c.on_surface_variant,
                )),
        )
        .height(Length::Fill)
        .into(),
        Some(st) => scrolling_list(
            st.cards
                .iter()
                .map(|card| {
                    widget::container(card_row(s, card, look))
                        .id(super::card_item(card.id))
                        .into()
                })
                .collect::<Vec<_>>(),
            s.is_focused(SF::Cards),
        )
        .id(super::CARDS_SCROLL)
        .height(Extent::Fill)
        .view(look),
    }
}

/// Whether the machine is online, once detection has said.
fn left_online<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    match &s.status {
        Some(st) if st.online => chip(ChipKind::Neutral, "Online").icon(Icon::Wifi).view(look),
        Some(_) => chip(
            ChipKind::Neutral,
            "Offline: drivers, fprintd and Bitwarden need the internet",
        )
        .icon(Icon::WifiOff)
        .view(look),
        None => widget::Space::new().into(),
    }
}

/// The way out, with "Reboot to finish" beside it while a card waits on a reboot.
fn left_footer<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let c = look.colors;
    let reboot = s.reboot_pending();
    // Beside "Reboot to finish" the way out stays quiet.
    let (finish_label, kind) = if reboot {
        ("Done for now", Kind::Text)
    } else if s.settled() {
        ("All set", Kind::Filled)
    } else {
        ("Done for now", Kind::Tonal)
    };
    let finish = button(finish_label)
        .kind(kind)
        .icon(Icon::CheckCircle)
        .on_press(click(SF::Finish))
        .focused(s.is_focused(SF::Finish))
        .view(look);
    if reboot {
        stack(look, Space::S12)
            .push(text(
                s.reboot_nudge().unwrap_or_default(),
                Type::Small,
                c.on_surface_variant,
            ))
            .push(
                row(look, Space::S12)
                    .center()
                    .push(
                        button("Reboot to finish")
                            .icon(Icon::RestartAlt)
                            .on_press(click(SF::RebootAll))
                            .focused(s.is_focused(SF::RebootAll))
                            .view(look),
                    )
                    .push(finish),
            )
            .into()
    } else {
        finish
    }
}

/// A card as a step: to do, done, or waiting on a reboot, each its own look, the reboot on the
/// accent. Its summary wraps, so it is read whole.
fn card_row<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> El<'a> {
    let running = s.run.as_ref().is_some_and(|r| r.card == c.id && r.running());
    let row = step(c.id.title())
        .subtitle(c.summary.clone())
        .icon(card_icon(c.id))
        .selected(c.id == s.sel)
        .on_press_maybe(Some(Message::Setup(super::Msg::Pick(c.id))));
    let row = if running {
        row.words("Running", Some(Icon::ProgressActivity))
    } else {
        match c.state {
            State::Done => row.state(StepState::Done),
            State::Needed => row,
            State::Reboot => row.state(REBOOT),
            State::Waiting => row.words("Waiting", Some(Icon::Schedule)),
            State::Absent => row.words("Not here", None),
        }
    };
    row.view(look)
}

/// A card waiting on a reboot: the action it needs, on the accent.
const REBOOT: StepState = StepState::Action {
    label: "Reboot",
    glyph: Icon::RestartAlt,
};
