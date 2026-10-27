//! Kōbō's blocks, composed from `kubaos-ui`'s components and tokens, that both the install screens
//! (`view.rs`) and setup (`setup/view.rs`) use.
//!
//! # Owns
//!
//! - The look Kōbō draws with ([`look`]) and the iced theme under it ([`theme`]).
//! - Key hints, the battery, the strength of a password, the note under a confirmation, a fact
//!   beside its label, and the toast.
//!
//! # Not here
//!
//! - The components themselves: `kubaos-ui`.

use std::time::Instant;

use iced::Element;
use iced::Length;
use iced::Theme;
use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::Mode;
use kubaos_ui::Scale;
use kubaos_ui::component::field::note;
use kubaos_ui::component::progress::bar;
use kubaos_ui::component::toast::DEFAULT_STAY;
use kubaos_ui::component::toast::toast_at;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::row;
use kubaos_ui::layout::split;
use kubaos_ui::layout::stack;
use kubaos_ui::snap::centre_x;
use kubaos_ui::text::line;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::app::Message;
use crate::battery::Battery;
use crate::model;

pub type El<'a> = Element<'a, Message>;

/// Kōbō's look in `mode` at `scale`: the fallback palette, since no wallpaper's colours are known
/// on the live ISO.
pub fn look(mode: Mode, scale: f32) -> Look {
    Look::new(mode, Scale::new(scale))
}

/// The theme iced's own widgets fall back to, from `look`'s colours.
pub fn theme(look: &Look) -> Theme {
    let c = look.colors;
    Theme::custom(
        "KubaOS",
        iced::theme::Palette {
            background: c.surface,
            text: c.on_surface,
            primary: c.primary,
            success: c.cold,
            warning: c.primary,
            danger: c.error,
        },
    )
}

/// One key hint: the key in mono, what it does in a caption.
pub fn hint<'a>(look: &Look, key: &'a str, what: &'a str) -> El<'a> {
    let c = look.colors;
    row(look, Space::S8)
        .center()
        .push(text(key, Type::Mono, c.on_surface_variant))
        .push(text(what, Type::Caption, c.outline))
        .into()
}

/// Key hints one under the other, the keys in a column of their own so what each does lines up.
pub fn hint_column<'a>(look: &Look, hints: &[(&'a str, &'a str)]) -> El<'a> {
    let c = look.colors;
    let mut lines = split(look, &[2, 5], Space::S4);
    for &(key, what) in hints {
        lines = lines
            .push(text(key, Type::Mono, c.on_surface_variant))
            .push(text(what, Type::Caption, c.outline));
    }
    lines.into()
}

/// Key hints side by side.
pub fn hint_row<'a>(look: &Look, hints: &[(&'a str, &'a str)]) -> El<'a> {
    row(look, Space::S20)
        .center()
        .extend(hints.iter().map(|&(key, what)| hint(look, key, what)))
        .into()
}

/// The battery: its symbol, charging or not, and the percent, muted; low, in the error colour.
pub fn battery<'a>(look: &Look, battery: Battery) -> El<'a> {
    let c = look.colors;
    let colour = if battery.is_low() {
        c.error
    } else {
        c.on_surface_variant
    };
    let glyph = if battery.on_charger() {
        Icon::BatteryChargingFull
    } else {
        Icon::BatteryFull
    };
    row(look, Space::S4)
        .center()
        .push(icon(look, glyph, IconSize::S16, colour))
        .push(text(format!("{}%", battery.percent), Type::Label, colour))
        .into()
}

/// How strong `password` is: a bar a quarter full per step, and what that means under it, in the
/// error colour while it is weak.
pub fn strength<'a>(look: &Look, password: &str) -> El<'a> {
    let steps = model::strength(password);
    let label = match steps {
        1 => "Weak. A toddler could guess it.",
        _ => model::strength_label(steps),
    };
    stack(look, Space::S8)
        .push(bar(look, f32::from(steps) / 4.0))
        .push(note(look, label, steps == 1))
        .into()
}

/// Under a confirmation field: nothing while it is still the start of the original (you're typing
/// it), "Doesn't match." once it can't be, "Matches." when it does.
pub fn confirm_note<'a>(look: &Look, original: &str, confirm: &str) -> Option<El<'a>> {
    if confirm.is_empty() || (confirm != original && original.starts_with(confirm)) {
        return None;
    }
    Some(if confirm == original {
        note(look, "Matches.", false)
    } else {
        note(look, "Doesn't match.", true)
    })
}

/// The confirmation is wrong, not just unfinished.
pub fn mismatched(original: &str, confirm: &str) -> bool {
    !confirm.is_empty() && confirm != original && !original.starts_with(confirm)
}

/// A fact beside its label: the label a quarter of the width, the value the rest.
pub fn fact<'a>(look: &Look, label: &'a str, value: String) -> El<'a> {
    let c = look.colors;
    split(look, &[1, 3], Space::S8)
        .push(text(label, Type::Small, c.on_surface_variant))
        .push(text(value, Type::Body, c.on_surface))
        .into()
}

/// [`fact`] with a second, quieter line under the value, in mono: a path, an email, a version,
/// which break badly wrapped mid-token, so they get a line of their own.
pub fn fact_with<'a>(look: &Look, label: &'a str, value: String, sub: String) -> El<'a> {
    let c = look.colors;
    split(look, &[1, 3], Space::S8)
        .push(text(label, Type::Small, c.on_surface_variant))
        .push(
            stack(look, Space::S4)
                .push(text(value, Type::Body, c.on_surface))
                .push(line(sub, Type::Mono, c.on_surface_variant)),
        )
        .into()
}

/// The toast, if one shows: at the bottom, in the middle.
pub fn toast<'a>(look: &Look, toast: Option<&'a (String, Instant)>, now: Instant) -> Option<El<'a>> {
    let (message, at) = toast?;
    let shown = toast_at(look, message, None, now.saturating_duration_since(*at), DEFAULT_STAY);
    Some(
        widget::container(centre_x(look.scale, shown))
            .width(Length::Fill)
            .align_bottom(Length::Fill)
            .padding(Inset::all(Space::S24).padding())
            .into(),
    )
}
