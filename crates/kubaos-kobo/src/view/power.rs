//! The power button, and the menu it opens over any screen.

use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::dialog::Tone;
use kubaos_ui::component::dialog::dialog;
use kubaos_ui::component::dialog::scrim;
use kubaos_ui::component::progress::spinner;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::row;
use kubaos_ui::text::text;
use kubaos_ui::tokens::IconSize;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::app::PowerAction;
use crate::progress::Outcome;
use crate::ui::El;

/// The button that opens the power menu, on every screen.
pub(super) fn power_button<'a>(app: &'a App, look: &Look) -> El<'a> {
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

pub(super) fn going_down(action: PowerAction) -> &'static str {
    match action {
        PowerAction::PowerOff => "Powering off…",
        PowerAction::Restart => "Restarting…",
    }
}

/// The power menu, a dialog over the dimmed screen. It takes every click: one outside it cancels,
/// and nothing behind it reacts.
pub(super) fn power_menu<'a>(app: &'a App, look: &Look) -> El<'a> {
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
