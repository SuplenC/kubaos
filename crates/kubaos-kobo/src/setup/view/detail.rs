//! The picked card's page: its head, its words and form in a scroll, the run's prompt and log, and
//! its actions.

use iced::Length;
use iced::widget;
use kubaos_core::Level;
use kubaos_core::setup::CardStatus;
use kubaos_core::setup::PromptKind;
use kubaos_core::setup::State;
use kubaos_ui::Look;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::card::card;
use kubaos_ui::component::error::Level as LogLevel;
use kubaos_ui::component::error::error_block;
use kubaos_ui::component::error::log;
use kubaos_ui::component::progress::bar;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::page_clear;
use kubaos_ui::layout::row;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::stack;
use kubaos_ui::motion::Motion;
use kubaos_ui::text::text;
use kubaos_ui::tokens;
use kubaos_ui::tokens::Easing;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::card_icon;
use super::click;
use super::explain::explain;
use super::form::form;
use super::state_chip;
use crate::setup::Act;
use crate::setup::FLASH;
use crate::setup::Outcome;
use crate::setup::SF;
use crate::setup::Setup;
use crate::ui::El;

pub(super) fn detail<'a>(s: &'a Setup, look: &Look) -> El<'a> {
    let shown = Motion::default().progress(tokens::Duration::Long, Easing::EmphasizedDecel, s.now - s.changed);
    let look = look.faded(shown);
    // The card's page rises into place as it fades in, a whole device pixel at a time.
    let rise = look.scale.snap((1.0 - shown) * f32::from(Space::S16.px()));
    let inner: El = match s.selected() {
        None => widget::Space::new().into(),
        Some(c) => card_detail(s, c, &look),
    };
    widget::container(page_clear(inner))
        .padding(iced::Padding::ZERO.top(rise))
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .into()
}

fn card_detail<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> El<'a> {
    let colours = look.colors;
    let head = row(look, Space::S16)
        .center()
        .push(icon(
            look,
            card_icon(c.id),
            IconSize::S48,
            if c.state == State::Absent {
                colours.outline
            } else {
                colours.primary
            },
        ))
        .push(
            stack(look, Space::S4)
                .push(
                    row(look, Space::S12)
                        .center()
                        .push(text(c.id.title(), Type::Title, colours.on_surface))
                        .push(state_chip(look, c.state)),
                )
                .push(text(c.summary.as_str(), Type::Body, colours.on_surface_variant)),
        );

    let mut parts = stack(look, Space::S16).padding(Inset::NONE.right(Space::S16));
    // What's happening right now goes first: the touch prompt has to be on screen, not under the
    // explanations.
    if let Some(run) = s.shown_run() {
        parts = parts.push(run_view(s, run, look));
    }
    parts = parts.extend(explain(s, c, look));
    let fields = s.fields();
    if !fields.is_empty() && !s.running() {
        parts = parts.push(form(s, &fields, look));
    }
    if let Some(run) = s.shown_run() {
        parts = parts.push(log_view(s, run, look));
    }
    let apart = Space::S24;
    stack(look, apart)
        .height(Extent::Fill)
        .push(head)
        .push(
            scroll(look, parts)
                .id("setup-detail")
                .height(Extent::Fill)
                .room_above(apart)
                .room_below(apart),
        )
        .push(actions(s, c, look))
        .into()
}

fn prompt_icon(k: PromptKind) -> Icon {
    match k {
        PromptKind::Password => Icon::Lock,
        PromptKind::Finger => Icon::Fingerprint,
        PromptKind::Key => Icon::Key,
        PromptKind::Wait => Icon::Schedule,
    }
}

fn run_view<'a>(s: &'a Setup, run: &'a super::Run, look: &Look) -> El<'a> {
    let c = look.colors;
    let seconds = (s.now - s.epoch).as_secs_f32();
    match (&run.outcome, &run.prompt) {
        (None, prompt) => {
            let (kind, message, count) = prompt.clone().unwrap_or((PromptKind::Wait, "Starting…".into(), None));
            let lead: El = if kind == PromptKind::Wait {
                spinner(look, IconSize::S48, seconds, c.primary)
            } else {
                icon(look, prompt_icon(kind), IconSize::S48, c.primary)
            };
            let mut words = stack(look, Space::S8)
                .width(Extent::Fill)
                .push(text(message, Type::Heading, c.on_surface));
            if let Some((touched, of)) = count {
                words = words.push(
                    stack(look, Space::S4)
                        .push(bar(look, touched as f32 / of.max(1) as f32))
                        .push(text(format!("{touched} of {of}"), Type::Small, c.on_surface_variant)),
                );
            }
            let sub = match kind {
                PromptKind::Password => "polkit's dialog. Your password, or your finger on the enrolled key.",
                PromptKind::Finger => "On the laptop's reader.",
                PromptKind::Key => "On the YubiKey's own sensor, not the laptop's.",
                PromptKind::Wait => "Nothing for you to do. Watch the log if you like that sort of thing.",
            };
            words = words.push(text(sub, Type::Small, c.on_surface_variant));
            card(look, row(look, Space::S20).center().push(lead).push(words))
        }
        (Some(Outcome::Done(summary)), _) => card(
            look,
            row(look, Space::S12)
                .center()
                .push(icon(look, Icon::CheckCircle, IconSize::S24, c.primary))
                .push(
                    stack(look, Space::S4)
                        .push(text("Done.", Type::Subheading, c.on_surface))
                        .push(text(summary.as_str(), Type::Body, c.on_surface_variant)),
                ),
        ),
        (Some(Outcome::Failed { error, hint }), _) => error_block(
            look,
            format!("Fuck. {}.", capital(error)),
            hint.clone().unwrap_or_default(),
        ),
    }
}

/// The run's log, under everything else: the facts and the serious notes come first.
fn log_view<'a>(s: &'a Setup, run: &'a super::Run, look: &Look) -> El<'a> {
    if run.log.is_empty() {
        return widget::Space::new().into();
    }
    let mut col = stack(look, Space::S12).push(
        button(if s.log_open { "Hide the log" } else { "Show the log" })
            .kind(Kind::Text)
            .icon(Icon::Terminal)
            .on_press(click(SF::Log))
            .focused(s.is_focused(SF::Log))
            .view(look),
    );
    if s.log_open {
        let shown: Vec<(LogLevel, &str)> = run
            .log
            .iter()
            .filter(|(level, _)| *level != Level::Debug)
            .map(|(level, line)| {
                let level = match level {
                    Level::Warn => LogLevel::Warn,
                    Level::Error => LogLevel::Error,
                    _ => LogLevel::Debug,
                };
                (level, line.as_str())
            })
            .collect();
        col = col.push(log(look, &shown, Extent::Fixed(Size::Log)));
    }
    col.into()
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

fn actions<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> El<'a> {
    let colours = look.colors;
    let running = s.running();
    let mut r = row(look, Space::S12).center().push(
        button(if c.state.open() { "Skip" } else { "Next" })
            .kind(Kind::Text)
            .trailing_icon(Icon::ArrowForward)
            .on_press(click(SF::Skip))
            .focused(s.is_focused(SF::Skip))
            .view(look),
    );
    r = r.push(fill());
    let blocker = s.blocker().filter(|_| {
        s.primary().is_some_and(|(_, a)| a == Act::Run) || s.secondary().is_some_and(|(_, a)| a == Act::Run)
    });
    if let Some(b) = blocker
        && !running
    {
        let flashing = s.flash.is_some_and(|f| s.now - f < FLASH);
        r = r.push(text(
            b,
            Type::Small,
            if flashing {
                colours.primary
            } else {
                colours.on_surface_variant
            },
        ));
    }
    if s.can_log_out() {
        r = r.push(
            button("Log out")
                .kind(Kind::Text)
                .on_press_maybe((!running).then(|| click(SF::LogOut)))
                .focused(s.is_focused(SF::LogOut))
                .view(look),
        );
    }
    if s.can_turn_off() {
        r = r.push(
            button("Turn the phone link off")
                .kind(Kind::Text)
                .on_press_maybe((!running).then(|| click(SF::TurnOff)))
                .focused(s.is_focused(SF::TurnOff))
                .view(look),
        );
    }
    if let Some((label, act)) = s.secondary() {
        let ok = !(running || act == Act::Run && s.blocker().is_some());
        r = r.push(
            button(label)
                .kind(Kind::Tonal)
                .on_press_maybe(ok.then(|| click(SF::Secondary)))
                .focused(s.is_focused(SF::Secondary))
                .view(look),
        );
    }
    if let Some((label, act)) = s.primary() {
        let ok = !(running || act == Act::Run && s.blocker().is_some());
        r = r.push(
            button(label)
                .on_press_maybe(ok.then(|| click(SF::Primary)))
                .focused(s.is_focused(SF::Primary))
                .view(look),
        );
    }
    r.into()
}
