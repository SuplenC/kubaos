//! The Account screen: the name, the username and the password, with the toggle that shows what
//! was typed.

use iced::widget;
use kubaos_ui::Look;
use kubaos_ui::component::button::ButtonSize;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Extent;
use kubaos_ui::layout::Inset;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::row;
use kubaos_ui::layout::scroll;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::chrome::FOOTER_GAP;
use super::input;
use super::input_state;
use super::invalid;
use crate::app::App;
use crate::app::F;
use crate::app::Message;
use crate::app::ids;
use crate::model;
use crate::theme::jp;
use crate::ui;
use crate::ui::El;

/// The Show/Hide toggle for secrets: small, at the right of a field's label.
pub(super) fn show_toggle<'a>(app: &'a App, look: &Look) -> El<'a> {
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

pub(super) fn secret_field<'a>(app: &'a App, label: &'a str, f: F, look: &Look) -> El<'a> {
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

pub(super) fn account<'a>(app: &'a App, look: &Look) -> El<'a> {
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
