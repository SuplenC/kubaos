//! A made-up machine for GUI work and screenshots: a TUXEDO with Secure Boot
//! on, a fingerprint reader, a YubiKey plugged in, Bitwarden logged out and
//! no git identity.
//! Actions play the real event shapes with fake timings and flip the card.

use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use super::{
    BwMethod, CardId, CardStatus, Facts, Inputs, MokState, PromptKind, SetupEvent, State, Status,
};
use crate::event::Level;

/// Which machine the demo pretends to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Machine {
    /// Everything there, nothing done.
    Tuxedo,
    /// The bench VM: no TUXEDO, no reader, no key.
    Vm,
}

pub struct Demo {
    status: Mutex<Status>,
    /// 1.0 is human speed; the tour runs faster.
    pub speed: f32,
}

impl Demo {
    pub fn new(machine: Machine, speed: f32) -> Self {
        Self {
            status: Mutex::new(initial(machine)),
            speed,
        }
    }

    pub fn status(&self) -> Status {
        self.status.lock().expect("demo lock").clone()
    }

    fn wait(&self, ms: u64) {
        sleep(Duration::from_millis(ms).div_f32(self.speed.max(0.01)));
    }

    pub fn run(&self, card: CardId, inputs: &Inputs, emit: &mut dyn FnMut(SetupEvent)) -> bool {
        emit(SetupEvent::Started { card });
        let p = |kind, text: &str, count| SetupEvent::Prompt {
            card,
            kind,
            text: text.into(),
            count,
        };
        let log = |line: &str| SetupEvent::Log {
            card,
            level: Level::Info,
            line: line.into(),
        };
        let fail = |error: &str, hint: &str| SetupEvent::Failed {
            card,
            error: error.into(),
            hint: Some(hint.into()),
        };
        let mut next = self.status().card(card).cloned().expect("demo card");
        match card {
            CardId::Tuxedo => {
                let pw = inputs.mok_password.as_deref().unwrap_or("");
                if let Err(why) = super::actions::mok_password_ok(pw) {
                    emit(fail(
                        "Secure Boot is on, so the drivers' signing key needs a one-time password",
                        why,
                    ));
                    return false;
                }
                emit(p(
                    PromptKind::Password,
                    "Your password, in the dialog that just opened.",
                    None,
                ));
                self.wait(1400);
                for (step, lines) in [
                    (
                        "TUXEDO apt repository (resolute)",
                        &[
                            "+ apt-get update",
                            "Get:1 https://deb.tuxedocomputers.com/ubuntu resolute InRelease [4,512 B]",
                        ][..],
                    ),
                    (
                        "Kernel headers + DKMS (tuxedo-drivers is a DKMS package)",
                        &[
                            "+ apt-get install -y --no-install-recommends dkms mokutil linux-headers-generic",
                        ][..],
                    ),
                    (
                        "tuxedo-drivers (NOT tuxedo-control-center)",
                        &[
                            "+ apt-get install -y --no-install-recommends tuxedo-drivers udev-hid-bpf",
                            "Building for 7.0.0-14-generic",
                            "Signing module /var/lib/dkms/tuxedo-drivers/4.15.4/build/tuxedo_keyboard.ko",
                        ][..],
                    ),
                    (
                        "Secure Boot: DKMS signing key (MOK)",
                        &[
                            "+ mokutil --import /var/lib/dkms/mok.pub",
                            "  ok  key queued for the next boot",
                        ][..],
                    ),
                ] {
                    emit(p(PromptKind::Wait, &format!("{step}…"), None));
                    emit(log(step));
                    for l in lines {
                        self.wait(500);
                        emit(log(l));
                    }
                    self.wait(700);
                }
                next.state = State::Reboot;
                next.summary =
                    "Key queued. Reboot and enroll it on the blue MokManager screen.".into();
                if let Facts::Tuxedo { drivers, mok, .. } = &mut next.facts {
                    *drivers = true;
                    *mok = Some(MokState::Pending);
                }
            }
            CardId::Fingerprint => {
                emit(p(
                    PromptKind::Password,
                    "Your password, in the dialog that just opened.",
                    None,
                ));
                self.wait(1200);
                emit(p(PromptKind::Wait, "Fingerprint reader (fprintd)…", None));
                emit(log(
                    "+ apt-get install -y --no-install-recommends fprintd libpam-fprintd",
                ));
                self.wait(1500);
                emit(log("+ pam-auth-update --enable fprintd"));
                let n = 8;
                emit(p(
                    PromptKind::Finger,
                    "Put your right index finger on the reader.",
                    Some((0, n)),
                ));
                for k in 1..=n {
                    self.wait(900);
                    if k == 3 {
                        emit(p(
                            PromptKind::Finger,
                            "Off-centre. Put it flat in the middle.",
                            Some((k - 1, n)),
                        ));
                        self.wait(900);
                    }
                    let text = if k == n - 1 {
                        "Lift it. Once more."
                    } else {
                        "Lift it, touch again. Shift it a little each time."
                    };
                    emit(p(PromptKind::Finger, text, Some((k, n))));
                }
                self.wait(500);
                next.state = State::Done;
                next.summary = "One finger: right index. sudo and password dialogs take it.".into();
                if let Facts::Fingerprint {
                    fingers,
                    fprintd,
                    pam,
                    ..
                } = &mut next.facts
                {
                    fingers.push("right-index-finger".into());
                    *fprintd = true;
                    *pam = true;
                }
            }
            CardId::Yubikey => {
                emit(p(
                    PromptKind::Key,
                    "Put an enrolled finger on the key's sensor when it blinks.",
                    None,
                ));
                self.wait(2600);
                emit(p(
                    PromptKind::Password,
                    "Your password, in the dialog that just opened.",
                    None,
                ));
                self.wait(1400);
                emit(p(
                    PromptKind::Wait,
                    "Adding the key to /etc/u2f_mappings…",
                    None,
                ));
                emit(log("  ok  /etc/u2f_mappings: kuba has 1 key"));
                self.wait(600);
                next.state = State::Done;
                next.summary =
                    "This key is enrolled (one key in total). Enroll a backup key too: keys get lost."
                        .into();
                if let Facts::Yubikey {
                    enrolled,
                    this_key_enrolled,
                    ..
                } = &mut next.facts
                {
                    *enrolled += 1;
                    *this_key_enrolled = true;
                }
            }
            CardId::Bitwarden => {
                let Some(l) = &inputs.bw else {
                    emit(fail("nothing to log in with", "Fill in the form first."));
                    return false;
                };
                let empty = match l.method {
                    BwMethod::ApiKey => l.client_id.is_empty() || l.client_secret.is_empty(),
                    _ => l.email.is_empty() || l.password.is_empty() || l.code.is_empty(),
                };
                if empty {
                    emit(fail(
                        "email, master password and code, all three",
                        "Fill in the empty one.",
                    ));
                    return false;
                }
                emit(p(PromptKind::Wait, "Logging in…", None));
                emit(log(&format!("bw login {} --method 0", l.email)));
                self.wait(2200);
                if l.code == "000000" {
                    emit(fail(
                        "Bitwarden didn't take that code",
                        "Codes last 30 seconds. Type the next one, quicker.",
                    ));
                    return false;
                }
                for (step, lines) in [
                    (
                        "Reading the vault",
                        &["  ok  57 items, 2 SSH key(s) in \"dotfiles\""][..],
                    ),
                    (
                        "SSH: public keys and Host blocks",
                        &[
                            "  ok  ~/.ssh/github.pub  (SHA256:4f1x…)",
                            "      Host github.com",
                            "  ok  ~/.ssh/config.d/bitwarden (2 Host block(s))",
                        ][..],
                    ),
                    (
                        "git: commit signing with an SSH key from the vault",
                        &["  ok  signing with ~/.ssh/github.pub (GitHub); git config includes it"]
                            [..],
                    ),
                    (
                        "gh: token from the vault into the system keyring",
                        &["  ok  gh logged in to github.com, token in the keyring"][..],
                    ),
                    (
                        "Project env: vault -> GNOME keyring (dot-bwenv)",
                        &["arms 2 variable(s)"][..],
                    ),
                    (
                        "Bitwarden desktop SSH agent (~/.bitwarden-ssh-agent.sock)",
                        &[
                            "  !!  no agent socket. Start Bitwarden desktop, log in, then Settings -> SSH agent -> turn on \"Enable SSH agent\"",
                        ][..],
                    ),
                ] {
                    emit(p(PromptKind::Wait, &format!("{step}…"), None));
                    emit(log(step));
                    for line in lines {
                        self.wait(450);
                        let (level, line) = match line.trim_start().strip_prefix("!!") {
                            Some(w) => (Level::Warn, w.trim()),
                            None => (Level::Info, *line),
                        };
                        emit(SetupEvent::Log {
                            card,
                            level,
                            line: line.into(),
                        });
                    }
                }
                next.state = State::Done;
                next.summary = format!(
                    "Logged in as {}. SSH keys, git signing and project env come from the vault.",
                    l.email
                );
                if let Facts::Bitwarden {
                    login,
                    email,
                    configured,
                    ..
                } = &mut next.facts
                {
                    *login = "unlocked".into();
                    *email = Some(l.email.clone());
                    *configured = true;
                }
            }
            CardId::Git => {
                let g = inputs.git.clone().unwrap_or_default();
                if let Err(why) = super::git_identity_ok(&g.name, &g.email) {
                    emit(fail("no name and email to commit with", why));
                    return false;
                }
                let (name, email) = (g.name.trim().to_string(), g.email.trim().to_string());
                self.wait(300);
                emit(log(&format!(
                    "  ok  ~/{}: {name} <{email}>",
                    crate::engine::files::GIT_IDENTITY
                )));
                next.state = State::Done;
                next.summary = format!("Commits as {name} <{email}>.");
                if let Facts::Git {
                    name: n, email: e, ..
                } = &mut next.facts
                {
                    *n = Some(name);
                    *e = Some(email);
                }
            }
        }
        {
            let mut s = self.status.lock().expect("demo lock");
            if let Some(c) = s.cards.iter_mut().find(|c| c.id == card) {
                *c = next.clone();
            }
        }
        emit(SetupEvent::Finished { card, status: next });
        true
    }
}

fn initial(machine: Machine) -> Status {
    let absent = |id, summary: &str, facts| CardStatus {
        id,
        state: State::Absent,
        summary: summary.into(),
        facts,
    };
    let bitwarden = CardStatus {
        id: CardId::Bitwarden,
        state: State::Needed,
        summary: "Not logged in.".into(),
        facts: Facts::Bitwarden {
            bw: true,
            login: "unauthenticated".into(),
            email: Some("kuba@example.com".into()),
            server: Some("https://vault.bitwarden.eu".into()),
            configured: false,
            desktop: machine == Machine::Tuxedo,
        },
    };
    let git = CardStatus {
        id: CardId::Git,
        state: State::Needed,
        summary: "No name and email for commits yet.".into(),
        facts: Facts::Git {
            name: None,
            email: None,
            full_name: Some("Kuba Example".into()),
        },
    };
    let cards = match machine {
        Machine::Tuxedo => vec![
            CardStatus {
                id: CardId::Tuxedo,
                state: State::Needed,
                summary: "Keyboard backlight, fans and power limits need TUXEDO's drivers.".into(),
                facts: Facts::Tuxedo {
                    vendor: "TUXEDO".into(),
                    product: "InfinityBook Pro AMD Gen10".into(),
                    secure_boot: Some(true),
                    drivers: false,
                    modules: vec![],
                    mok: None,
                },
            },
            CardStatus {
                id: CardId::Fingerprint,
                state: State::Needed,
                summary: "Goodix reader (27c6:609c). No finger enrolled yet.".into(),
                facts: Facts::Fingerprint {
                    reader: Some("Goodix reader (27c6:609c)".into()),
                    fprintd: false,
                    fingers: vec![],
                    pam: false,
                },
            },
            CardStatus {
                id: CardId::Yubikey,
                state: State::Needed,
                summary: "YubiKey C Bio - FIDO Edition plugged in, not enrolled yet.".into(),
                facts: Facts::Yubikey {
                    plugged: vec!["YubiKey C Bio - FIDO Edition".into()],
                    enrolled: 0,
                    this_key_enrolled: false,
                    pin: Some(true),
                    fingerprints: Some(true),
                },
            },
            bitwarden,
            git,
        ],
        Machine::Vm => vec![
            absent(
                CardId::Tuxedo,
                "Not on this machine: QEMU isn't a TUXEDO.",
                Facts::Tuxedo {
                    vendor: "QEMU".into(),
                    product: String::new(),
                    secure_boot: None,
                    drivers: false,
                    modules: vec![],
                    mok: None,
                },
            ),
            absent(
                CardId::Fingerprint,
                "Not on this machine: no fingerprint reader.",
                Facts::Fingerprint {
                    reader: None,
                    fprintd: false,
                    fingers: vec![],
                    pam: false,
                },
            ),
            CardStatus {
                id: CardId::Yubikey,
                state: State::Waiting,
                summary: "Plug in your YubiKey to use it for sudo, polkit and the lock screen."
                    .into(),
                facts: Facts::Yubikey {
                    plugged: vec![],
                    enrolled: 0,
                    this_key_enrolled: false,
                    pin: None,
                    fingerprints: None,
                },
            },
            bitwarden,
            git,
        ],
    };
    Status {
        online: true,
        cards,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::{BwLogin, GitIdentity};

    #[test]
    fn demo_cards_flip() {
        let d = Demo::new(Machine::Tuxedo, 1000.0);
        let mut evs = vec![];
        assert!(!d.run(CardId::Tuxedo, &Inputs::default(), &mut |e| evs.push(e)));
        let inputs = Inputs {
            mok_password: Some("kuba2026".into()),
            bw: Some(BwLogin {
                email: "a@b.c".into(),
                password: "x".into(),
                code: "123456".into(),
                ..Default::default()
            }),
            git: Some(GitIdentity {
                name: "Kuba Example".into(),
                email: "kuba@example.com".into(),
            }),
            ..Default::default()
        };
        for c in CardId::ALL {
            assert!(d.run(c, &inputs, &mut |_| {}), "{c:?}");
        }
        let s = d.status();
        assert!(s.settled());
        assert!(s.reboot_pending());
    }
}
