//! The Done screen's copy and actions; its sign-off is `crate::done`.

use std::time::Duration;

use iced::Vector;
use kubaos_ui::Look;
use kubaos_ui::color;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::chrome::hints;
use super::lede;
use super::power::going_down;
use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::done;
use crate::progress::Outcome;
use crate::progress::fmt_duration;
use crate::ui;
use crate::ui::El;

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

pub(super) fn done<'a>(app: &'a App, look: &Look) -> El<'a> {
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
