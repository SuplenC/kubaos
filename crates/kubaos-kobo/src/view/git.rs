//! The Git screen: the name and email commits are signed with.

use kubaos_ui::Look;
use kubaos_ui::component::card::card;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::gap;
use kubaos_ui::layout::grid;
use kubaos_ui::layout::header;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::text::text;
use kubaos_ui::tokens::Space;
use kubaos_ui::tokens::Type;

use super::input;
use super::input_state;
use super::invalid;
use crate::app::App;
use crate::app::F;
use crate::model;
use crate::theme::jp;
use crate::ui::El;

pub(super) fn git<'a>(app: &'a App, look: &Look) -> El<'a> {
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
