//! Each card's form: its fields, in one match over the fields a card asks for.

use iced::widget;
use kubaos_core::setup::BwMethod;
use kubaos_ui::Look;
use kubaos_ui::component::button::Kind;
use kubaos_ui::component::button::button;
use kubaos_ui::component::field::State as FieldState;
use kubaos_ui::component::field::labelled;
use kubaos_ui::component::field::note;
use kubaos_ui::icon::Icon;
use kubaos_ui::layout::Gap;
use kubaos_ui::layout::fill;
use kubaos_ui::layout::row;
use kubaos_ui::layout::stack;
use kubaos_ui::tokens::Size;
use kubaos_ui::tokens::Space;

use super::bitwarden::folder_ticks;
use super::choice;
use super::click;
use super::input;
use super::input_state;
use crate::app::Message;
use crate::setup::SF;
use crate::setup::Server;
use crate::setup::Setup;
use crate::setup::method_label;
use crate::ui;
use crate::ui::El;

#[expect(
    clippy::too_many_lines,
    reason = "One arm per field, each its labelled input: every form field of setup, read in one place."
)]
pub(super) fn form<'a>(s: &'a Setup, fields: &[SF], look: &Look) -> El<'a> {
    let f = &s.form;
    let mut col = stack(look, Space::S16);
    for &sf in fields {
        let el: El = match sf {
            SF::Mok => labelled(
                look,
                "One-time password for the blue screen",
                None,
                input(s, sf, "8 to 16 letters and digits", Some(Icon::Key), look),
                None,
            ),
            SF::MokAgain => labelled(
                look,
                "Same again",
                None,
                input_state(
                    s,
                    sf,
                    "",
                    Some(Icon::Key),
                    if ui::mismatched(&f.mok, &f.mok_again) {
                        FieldState::Invalid
                    } else {
                        FieldState::Normal
                    },
                    look,
                ),
                ui::confirm_note(look, &f.mok, &f.mok_again),
            ),
            SF::Server => labelled(
                look,
                "Server",
                None,
                choice(
                    s,
                    sf,
                    [Server::Com, Server::Eu, Server::Custom]
                        .into_iter()
                        .map(|x| (x.label(), x == f.server, Message::Setup(super::Msg::Server(x))))
                        .collect(),
                    look,
                ),
                None,
            ),
            SF::Url => labelled(
                look,
                "Server address",
                None,
                input(s, sf, "https://vault.example.com", Some(Icon::Public), look),
                None,
            ),
            SF::Email => labelled(
                look,
                "Email",
                None,
                input(s, sf, "you@example.com", Some(Icon::Mail), look),
                None,
            ),
            SF::Folders => labelled(
                look,
                "SSH keys from",
                None,
                folder_ticks(s, look),
                Some(note(
                    look,
                    if f.folders.is_empty() {
                        "The whole vault. Tick folders to take keys from those only; Pull again to apply."
                    } else {
                        "Only the ticked folders. Untick them all for the whole vault; Pull again to apply."
                    },
                    false,
                )),
            ),
            SF::Password => labelled(
                look,
                "Master password",
                None,
                input(s, sf, "", Some(Icon::Lock), look),
                s.bw_logged_in()
                    .then(|| note(look, "Logged in already: this only unlocks the vault.", false)),
            ),
            SF::Method => labelled(
                look,
                "Two-step login",
                None,
                choice(
                    s,
                    sf,
                    [BwMethod::Totp, BwMethod::YubikeyOtp, BwMethod::ApiKey]
                        .into_iter()
                        .map(|m| (method_label(m), m == f.method, Message::Setup(super::Msg::Method(m))))
                        .collect(),
                    look,
                ),
                Some(note(
                    look,
                    match f.method {
                        BwMethod::Totp => "The 6-digit code from your authenticator app.",
                        BwMethod::YubikeyOtp => "A YubiKey 5 types its code when touched. The Bio can't.",
                        BwMethod::ApiKey => {
                            "Web vault: Settings, Security, Keys, View API key. No two-step code; bw keeps a refresh \
                             token."
                        }
                    },
                    false,
                )),
            ),
            SF::Code => labelled(
                look,
                "Two-step code",
                None,
                input(s, sf, "123456", Some(Icon::Timer), look),
                None,
            ),
            SF::ClientId => labelled(
                look,
                "client_id",
                None,
                input(s, sf, "user.…", Some(Icon::Badge), look),
                None,
            ),
            SF::ClientSecret => labelled(
                look,
                "client_secret",
                None,
                input(s, sf, "", Some(Icon::Key), look),
                None,
            ),
            SF::GitName => labelled(
                look,
                "Name",
                None,
                input(s, sf, "Your name", Some(Icon::Person), look),
                None,
            ),
            SF::GitEmail => labelled(
                look,
                "Email",
                None,
                input(s, sf, "you@example.com", Some(Icon::Mail), look),
                Some(note(
                    look,
                    "Public on every commit you push. GitHub's noreply address works too.",
                    false,
                )),
            ),
            _ => continue,
        };
        col = col.push(el);
    }
    if fields.iter().any(|f| f.is_secret()) {
        // The same small toggle as the install screens', under the fields: it comes after them in
        // the Tab order too.
        let (label, glyph) = if s.show_secrets {
            ("Hide", Icon::VisibilityOff)
        } else {
            ("Show", Icon::Visibility)
        };
        col = col.push(
            row(look, Gap::NONE).push(fill()).push(
                button(label)
                    .kind(Kind::Text)
                    .size(kubaos_ui::component::button::ButtonSize::Sm)
                    .icon(glyph)
                    .on_press(click(SF::ShowSecrets))
                    .focused(s.is_focused(SF::ShowSecrets))
                    .view(look),
            ),
        );
    }
    widget::container(col).max_width(f32::from(Size::Column.px())).into()
}
