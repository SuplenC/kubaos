//! What each card says, for its state: every card's words in one match, read in one place.

use kubaos_core::setup::CardId;
use kubaos_core::setup::CardStatus;
use kubaos_core::setup::Facts;
use kubaos_core::setup::MokState;
use kubaos_core::setup::State;
use kubaos_core::setup::finger_name;
use kubaos_ui::Look;
use kubaos_ui::icon::Icon;

use super::bitwarden::keys_overview;
use super::para;
use super::phone::phone_explain;
use super::tuxedo::mok_notice;
use super::warning;
use crate::setup::Setup;
use crate::ui::El;

/// What the card is about, for its state. Facts first.
#[expect(
    clippy::too_many_lines,
    reason = "One arm per card and state, each its few sentences: every card's words, read in one place."
)]
pub(super) fn explain<'a>(s: &'a Setup, c: &'a CardStatus, look: &Look) -> Vec<El<'a>> {
    let mut v: Vec<El> = vec![];
    match (&c.facts, c.state) {
        (_, State::Absent) => v.push(para(
            look,
            match c.id {
                CardId::Tuxedo => "Nothing to install. The TUXEDO drivers only make sense on a TUXEDO.",
                CardId::Fingerprint => {
                    "No reader that fprintd knows. Plugged one in? Check again from a fresh start of setup."
                }
                _ => "Nothing to do here.",
            },
        )),
        (
            Facts::Tuxedo {
                product,
                secure_boot,
                drivers,
                mok,
                ..
            },
            st,
        ) => {
            let machine = if product.is_empty() {
                "This TUXEDO".to_string()
            } else {
                format!("Your {product}")
            };
            if matches!(st, State::Needed | State::Waiting) && !drivers {
                v.push(para(
                    look,
                    format!(
                        "{machine} wants TUXEDO's own kernel drivers for the keyboard backlight, the fans and the \
                         power limits. Setup adds TUXEDO's apt repository and installs tuxedo-drivers. No Control \
                         Center: KubaOS runs power profiles instead, one less app phoning home."
                    ),
                ));
            }
            if s.wants_mok() || matches!(mok, Some(MokState::Pending)) {
                v.push(mok_notice(c.state, look));
            } else if *secure_boot == Some(false) && c.state != State::Done {
                v.push(para(
                    look,
                    "Secure Boot is off, so the drivers load as they are. No key to enroll.",
                ));
            }
            if c.state == State::Reboot && !matches!(mok, Some(MokState::Pending)) {
                v.push(para(look, "They load on the next boot. Reboot when it suits you."));
            }
        }
        (Facts::Fingerprint { reader, fingers, .. }, st) => {
            let reader = reader.clone().unwrap_or_else(|| "The reader".into());
            v.push(para(
                look,
                if st == State::Done {
                    format!(
                        "{reader}. sudo, polkit dialogs and the tty login take your finger first and wait 10 \
                         seconds before they ask for the password. Enrolled: {}.",
                        fingers.iter().map(|f| finger_name(f)).collect::<Vec<_>>().join(", ")
                    )
                } else {
                    format!(
                        "{reader}. Enroll a finger and sudo and password dialogs take it instead of your password. \
                         The password keeps working, so a cut finger is a bad day, not a lockout. You'll touch the \
                         reader eight-ish times; move the finger a little each time."
                    )
                },
            ));
        }
        (
            Facts::Yubikey {
                plugged,
                enrolled,
                pin,
                fingerprints,
                ..
            },
            st,
        ) => {
            v.push(para(
                look,
                "Your YubiKey and the finger on its sensor stand in for the password at sudo, polkit and the lock \
                 screen. The password always keeps working. Enroll a second key as the backup: keys get lost.",
            ));
            if st == State::Needed && (*pin == Some(false) || *fingerprints == Some(false)) {
                v.push(warning(
                    look,
                    Icon::Key,
                    "The key needs its own setup first",
                    vec![
                        "It has no FIDO2 PIN or no fingerprint yet, and it won't enroll without both. That part \
                         talks to the key directly, so it runs in a terminal:"
                            .into(),
                        "ykman fido access change-pin, then ykman fido fingerprints add right-index (and a finger \
                         from the other hand)."
                            .into(),
                        "Come back and press Check again.".into(),
                    ],
                ));
            } else if st == State::Waiting && plugged.is_empty() && *enrolled == 0 {
                v.push(para(look, "Plug it in, then Check again. Or skip it; I'll be here."));
            }
        }
        (
            Facts::Bitwarden {
                desktop,
                configured,
                server,
                email,
                login,
                folders,
                overview,
                agent,
                ..
            },
            st,
        ) => {
            if let (Some(server), true) = (server, s.bw_logged_in()) {
                let who = email.as_deref().unwrap_or("someone");
                v.push(para(
                    look,
                    format!("Logged in to {server} as {who}, the vault {login}."),
                ));
            }
            if st == State::Done {
                v.push(para(
                    look,
                    "SSH public keys and Host blocks, git commit signing, the gh token and every project's env came \
                     from the vault. Changed the vault? Pull again; kubaos bitwarden refresh in a terminal does the \
                     same.",
                ));
            }
            if *configured {
                v.push(keys_overview(folders, overview, *desktop, agent.as_ref(), look));
            }
            if st != State::Done {
                v.push(para(
                    look,
                    "Log in once and setup pulls what the dotfiles expect from the vault: SSH public keys and Host \
                     blocks, git commit signing, the gh token and your projects' env. The private keys never touch \
                     the disk.",
                ));
                v.push(para(
                    look,
                    "bw can't use the YubiKey Bio for two-step login: use the authenticator app's code. Only the \
                     email and server are saved. The password and the vault session live in memory until this is \
                     done.",
                ));
            }
            if *desktop && (*configured || st == State::Done) {
                v.push(para(
                    look,
                    "Bitwarden desktop starts when you close setup. Last bit by hand: log in there, Settings, turn \
                     on \"Enable SSH agent\". It has no config file for that.",
                ));
            }
        }
        (Facts::Phone { link }, st) => v.extend(phone_explain(s, link, st, look)),
        (Facts::Git { .. }, st) => v.push(para(
            look,
            if st == State::Done {
                "git commits with this name and email. Change them here any time; they live in \
                 ~/.config/git-identity/identity.gitconfig, outside the dotfiles repo."
            } else {
                "The name and email git puts on every commit. The dotfiles don't carry one, so the repo stays \
                 yours to share. Saved to ~/.config/git-identity/identity.gitconfig, which the git config \
                 includes."
            },
        )),
        _ => {}
    }
    v
}
