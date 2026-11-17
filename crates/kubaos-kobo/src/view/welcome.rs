//! The Welcome screen: the lockup, what the probe found, and the way in.

use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::button;
use kubaos_ui::component::chip::ChipKind;
use kubaos_ui::component::chip::chip;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::icon::Icon;
use kubaos_ui::icon::icon;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::chrome::hints;
use super::lede;
use super::sign;
use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::app::ProbeState;
use crate::theme::jp;
use crate::ui;
use crate::ui::El;

pub(super) fn welcome<'a>(app: &'a App, look: &Look) -> El<'a> {
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
        .push(
            row(look, Space::S20)
                .center()
                .push(sign(look))
                .push(chip(ChipKind::Neutral, app.codename().to_uppercase()).view(look)),
        )
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
            meta.push(
                chip(ChipKind::Accent, format!("v{}", m.version))
                    .icon(Icon::DeployedCode)
                    .view(look),
            );
            meta.push(chip(ChipKind::Neutral, m.commit.chars().take(7).collect::<String>()).view(look));
            meta.push(
                chip(
                    ChipKind::Neutral,
                    format!("built {}", m.built_at.get(..10).unwrap_or(&m.built_at)),
                )
                .view(look),
            );
        }
        meta.push(
            chip(
                ChipKind::Neutral,
                format!("{}{}", pr.arch, if pr.uefi { " · UEFI" } else { " · BIOS" }),
            )
            .view(look),
        );
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
