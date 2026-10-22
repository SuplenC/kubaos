//! What each card does. Every action ends with one `Finished` (the card
//! detected again) or one `Failed`.
//!
//! Secrets: the MOK password goes to the helper on stdin; the master
//! password and API key go to `bw` in its environment (`--passwordenv`,
//! `BW_CLIENTID`), the session to `dot-bw-setup` as `BW_SESSION`, the way
//! docs/SECRETS.md has it. None of them is logged. The one exception to "no
//! secrets in argv" is the two-step code (`bw login --code`): bw has no other
//! way to take it without a terminal, and it's spent the moment bw uses it.

use super::detect::{self, BW};
use super::parse::*;
use super::{
    BwMethod, CardId, Cmd, Env, Facts, Inputs, MokState, PromptKind, SetupEvent, Stream, Sys,
};
use crate::event::Level;

struct Fail {
    error: String,
    hint: Option<String>,
}

fn fail(error: impl Into<String>, hint: impl Into<String>) -> Fail {
    Fail {
        error: error.into(),
        hint: Some(hint.into()),
    }
}

impl From<String> for Fail {
    fn from(error: String) -> Self {
        Fail { error, hint: None }
    }
}

type R<T> = Result<T, Fail>;

struct Ctx<'a> {
    sys: &'a mut dyn Sys,
    env: &'a Env,
    card: CardId,
    emit: &'a mut dyn FnMut(SetupEvent),
}

impl Ctx<'_> {
    fn log(&mut self, level: Level, line: impl Into<String>) {
        (self.emit)(SetupEvent::Log {
            card: self.card,
            level,
            line: line.into(),
        });
    }

    fn prompt(&mut self, kind: PromptKind, text: impl Into<String>, count: Option<(u32, u32)>) {
        (self.emit)(SetupEvent::Prompt {
            card: self.card,
            kind,
            text: text.into(),
            count,
        });
    }

    /// One verb of the root helper, through pkexec. polkit's dialog asks
    /// for the password (or the enrolled key, or the finger) first.
    fn privileged(&mut self, verb: &[&str], stdin: Option<&str>, busy: &str) -> R<()> {
        self.prompt(
            PromptKind::Password,
            "Your password, in the dialog that just opened.",
            None,
        );
        let mut cmd = self.env.privileged(verb[0]).args(verb[1..].iter().copied());
        if let Some(s) = stdin {
            cmd = cmd.secret_stdin(s);
        }
        self.log(
            Level::Debug,
            format!("$ {} {}", self.env.privileged.join(" "), verb.join(" ")),
        );
        let card = self.card;
        let mut authed = false;
        let busy = busy.to_string();
        let emit = &mut *self.emit;
        let r = self.sys.stream(&cmd, &mut |s| {
            let Stream::Line(l) = s else { return };
            if !authed {
                // Output means pkexec let us through.
                authed = true;
                emit(SetupEvent::Prompt {
                    card,
                    kind: PromptKind::Wait,
                    text: busy.clone(),
                    count: None,
                });
            }
            let t = l.trim_end();
            let (level, line) = if let Some(s) = t.strip_prefix("==> ") {
                emit(SetupEvent::Prompt {
                    card,
                    kind: PromptKind::Wait,
                    text: format!("{s}…"),
                    count: None,
                });
                (Level::Info, s.to_string())
            } else if let Some(w) = t.trim_start().strip_prefix("!!") {
                (Level::Warn, w.trim().to_string())
            } else if let Some(e) = t.trim_start().strip_prefix("xx") {
                (Level::Error, e.trim().to_string())
            } else {
                (Level::Info, t.to_string())
            };
            if !line.trim().is_empty() {
                emit(SetupEvent::Log { card, level, line });
            }
        });
        r.map_err(|e| pkexec_error(&e))
    }
}

/// pkexec's exit codes: 126 the dialog was dismissed, 127 not authorized
/// (or no agent to ask).
fn pkexec_error(e: &str) -> Fail {
    if e.contains("status: 126") {
        fail(
            "the password dialog was closed",
            "Nothing changed. Run it again when you're ready.",
        )
    } else if e.contains("status: 127") {
        fail(
            "polkit said no",
            "Wrong password three times, or no password dialog at all: then the polkit agent \
             isn't running (systemctl --user start hyprpolkitagent).",
        )
    } else if e.starts_with("pkexec:") && e.contains("No such file") {
        fail("pkexec isn't installed", "sudo apt install pkexec")
    } else {
        let last = e.lines().last().unwrap_or(e).trim();
        Fail {
            error: last
                .trim_start_matches("xx")
                .trim()
                .trim_end_matches('.')
                .to_string(),
            hint: Some("The details are in the log below.".into()),
        }
    }
}

/// Runs one card's action. `true` when it finished.
pub fn run(
    sys: &mut dyn Sys,
    env: &Env,
    card: CardId,
    inputs: &Inputs,
    emit: &mut dyn FnMut(SetupEvent),
) -> bool {
    emit(SetupEvent::Started { card });
    let mut ctx = Ctx {
        sys,
        env,
        card,
        emit,
    };
    let r = match card {
        CardId::Tuxedo => tuxedo(&mut ctx, inputs),
        CardId::Fingerprint => fingerprint(&mut ctx, inputs),
        CardId::Yubikey => yubikey(&mut ctx),
        CardId::Bitwarden => bitwarden(&mut ctx, inputs),
        CardId::Git => git(&mut ctx, inputs),
    };
    let Ctx { sys, emit, .. } = ctx;
    match r {
        Ok(()) => {
            let online = detect::online(sys);
            let status = detect::card_status(sys, env, card, online);
            emit(SetupEvent::Finished { card, status });
            true
        }
        Err(Fail { error, hint }) => {
            emit(SetupEvent::Failed { card, error, hint });
            false
        }
    }
}

/// MokManager is a firmware app with a US keyboard and no layout switch:
/// keep the password to what's printed on every key cap.
pub fn mok_password_ok(pw: &str) -> Result<(), &'static str> {
    if pw.len() < 8 {
        Err("At least 8 characters.")
    } else if pw.len() > 16 {
        Err("16 characters at most: you'll type it on a blue screen.")
    } else if !pw.chars().all(|c| c.is_ascii_alphanumeric()) {
        Err("Letters and digits only: MokManager assumes a US keyboard.")
    } else {
        Ok(())
    }
}

// ------------------------------------------------------------------ TUXEDO

fn tuxedo(ctx: &mut Ctx, inputs: &Inputs) -> R<()> {
    let c = detect::card_status(ctx.sys, ctx.env, CardId::Tuxedo, true);
    let Facts::Tuxedo {
        secure_boot,
        drivers,
        mok,
        ..
    } = c.facts
    else {
        return Err("no TUXEDO facts".to_string().into());
    };
    if c.state == super::State::Absent {
        return Err(fail(c.summary, "Nothing to install here."));
    }
    // A key to enroll: Secure Boot on, and the drivers not in yet (DKMS
    // makes the key while it builds them) or in with the key not enrolled.
    let wants_mok = secure_boot == Some(true)
        && (!drivers || matches!(mok, Some(MokState::NotEnrolled | MokState::NoKey)));
    let pw = inputs.mok_password.as_deref().unwrap_or("");
    if wants_mok {
        if let Err(why) = mok_password_ok(pw) {
            return Err(fail(
                "Secure Boot is on, so the drivers' signing key needs a one-time password",
                why,
            ));
        }
        let line = format!("{pw}\n");
        ctx.privileged(
            &["tuxedo", "--mok-password-stdin"],
            Some(&line),
            "Installing TUXEDO's drivers…",
        )
    } else {
        ctx.privileged(&["tuxedo"], None, "Installing TUXEDO's drivers…")
    }
}

// ------------------------------------------------------------- fingerprint

fn fingerprint(ctx: &mut Ctx, inputs: &Inputs) -> R<()> {
    let c = detect::card_status(ctx.sys, ctx.env, CardId::Fingerprint, true);
    let Facts::Fingerprint {
        fprintd,
        pam,
        fingers,
        ..
    } = c.facts
    else {
        return Err("no fingerprint facts".to_string().into());
    };
    if c.state == super::State::Absent {
        return Err(fail(c.summary, "Nothing to enroll here."));
    }
    if !fprintd || !pam {
        // fprintd, its PAM module, and pam-auth-update --enable fprintd,
        // as 30-hardware.sh does it. Safe before enrolling: with no finger
        // enrolled the module steps aside at once.
        ctx.privileged(&["fingerprint"], None, "Installing fprintd…")?;
    }
    let finger = inputs
        .finger
        .clone()
        .unwrap_or_else(|| next_finger(&fingers).to_string());
    if !FINGERS.contains(&finger.as_str()) {
        return Err(format!("{finger} isn't a finger fprintd knows").into());
    }
    let stages = ctx
        .sys
        .run(&Cmd::new("busctl").args([
            "get-property",
            "net.reactivated.Fprint",
            "/net/reactivated/Fprint/Device/0",
            "net.reactivated.Fprint.Device",
            "num-enroll-stages",
        ]))
        .ok()
        .and_then(|o| o.stdout.trim().strip_prefix("i ")?.parse::<u32>().ok())
        .filter(|n| *n > 0);
    let name = finger_name(&finger);
    let total = stages.unwrap_or(0);
    ctx.prompt(
        PromptKind::Finger,
        format!("Put your {name} finger on the reader."),
        stages.map(|n| (0, n)),
    );
    // stdbuf: fprintd-enroll's stdio is block-buffered into a pipe, and the
    // touches have to show up as they happen.
    let cmd = Cmd::new("stdbuf").args(["-oL", "fprintd-enroll", "-f", &finger]);
    let card = ctx.card;
    let mut k = 0u32;
    let mut done = false;
    let mut why: Option<String> = None;
    let emit = &mut *ctx.emit;
    let r = ctx.sys.stream(&cmd, &mut |s| {
        let Stream::Line(l) = s else { return };
        let prompt = |text: String, k: u32| SetupEvent::Prompt {
            card,
            kind: PromptKind::Finger,
            text,
            count: stages.map(|n| (k.min(n), n)),
        };
        match enroll_line(&l) {
            Enroll::Stage => {
                k += 1;
                let left = total.saturating_sub(k);
                let text = if total > 0 && left == 1 {
                    "Lift it. Once more.".to_string()
                } else {
                    "Lift it, touch again. Shift it a little each time.".to_string()
                };
                emit(prompt(text, k));
            }
            Enroll::Retry(m) => emit(prompt(m.to_string(), k)),
            Enroll::Completed => done = true,
            Enroll::Failed(m) => why = Some(m),
            Enroll::Other => {}
        }
        emit(SetupEvent::Log {
            card,
            level: Level::Debug,
            line: l,
        });
    });
    match (r, done, why) {
        (_, true, _) => Ok(()),
        (_, false, Some(m)) => Err(fail(
            format!("enrolling your {name} finger failed: {m}"),
            "Run it again. A different finger works too.",
        )),
        (Err(e), false, None) if e.contains("PermissionDenied") || e.contains("Not Authorized") => {
            Err(fail(
                "fprintd wasn't allowed to enroll",
                "Answer the password dialog, then try again.",
            ))
        }
        (Err(e), false, None) => Err(fail(
            format!("fprintd-enroll stopped: {}", e.lines().last().unwrap_or(&e)),
            "Run it again. If the reader is busy, wait a few seconds first.",
        )),
        (Ok(()), false, None) => Err(fail(
            "fprintd-enroll ended without finishing",
            "Run it again and keep touching until it says done.",
        )),
    }
}

// ------------------------------------------------------------------ YubiKey

/// What to do in a terminal when the key isn't ready for enrolment.
pub const YKMAN_PREP: &str =
    "ykman fido access change-pin, then ykman fido fingerprints add right-index";

fn yubikey(ctx: &mut Ctx) -> R<()> {
    let devs = ctx
        .sys
        .run(&Cmd::new("fido2-token").arg("-L"))
        .map(|o| o.stdout.lines().filter(|l| !l.trim().is_empty()).count())
        .map_err(|_| {
            fail(
                "fido2-token isn't working",
                "system/75-yubikey.sh installs fido2-tools.",
            )
        })?;
    match devs {
        0 => {
            return Err(fail(
                "no FIDO key found",
                "Plug it in. Still nothing? ls -l /dev/hidraw* should show a + on the key's node.",
            ));
        }
        1 => {}
        n => {
            return Err(fail(
                format!("{n} FIDO keys plugged in"),
                "Leave only the one to enroll, then try again.",
            ));
        }
    }
    let info = ctx
        .sys
        .run(&Cmd::new(ctx.env.script("dot-u2f-enroll")).arg("--info"))
        .map(|o| parse_key_info(&o.stdout))
        .map_err(|e| {
            fail(
                format!("dot-u2f-enroll --info failed: {e}"),
                "Is the key plugged in all the way?",
            )
        })?;
    if info.enrolled_here {
        ctx.log(Level::Info, "This key is enrolled already. Nothing to do.");
        return Ok(());
    }
    match (info.pin, info.fingerprints) {
        (Some(false), _) | (_, Some(false)) => {
            return Err(fail(
                "this key needs its own PIN and a fingerprint first",
                format!("Set them up in a terminal: {YKMAN_PREP}."),
            ));
        }
        (_, None) => {
            return Err(fail(
                "this key has no fingerprint reader",
                "Enroll it in a terminal with dot-u2f-enroll --pin: sudo asks for its PIN, the lock screen skips it.",
            ));
        }
        _ => {}
    }
    ctx.prompt(
        PromptKind::Key,
        "Put an enrolled finger on the key's sensor when it blinks.",
        None,
    );
    // -V: user verification (the finger), no +pin: the lock screen can't
    // ask for a PIN (dot-u2f-enroll, docs/SECRETS.md).
    let out = ctx
        .sys
        .run(&Cmd::new("pamu2fcfg").args(["-n", "-u", &ctx.env.user, "-V"]))
        .map_err(|_| {
            fail(
                "the key didn't confirm",
                "Wrong finger, or too slow. After 3 bad scans the key wants its PIN: unplug it, plug it back, try again.",
            )
        })?;
    let cred = out.stdout.trim().trim_start_matches(':').to_string();
    if !valid_credential(&cred) {
        return Err(fail(
            "pamu2fcfg printed something that isn't a credential",
            "Try dot-u2f-enroll in a terminal; it says more.",
        ));
    }
    ctx.privileged(
        &["u2f-add"],
        Some(&format!("{cred}\n")),
        "Adding the key to /etc/u2f_mappings…",
    )
}

// ---------------------------------------------------------------- Bitwarden

fn bitwarden(ctx: &mut Ctx, inputs: &Inputs) -> R<()> {
    let Some(login) = &inputs.bw else {
        return Err(fail("nothing to log in with", "Fill in the form first."));
    };
    let status = ctx
        .sys
        .run(&Cmd::new(BW).arg("status").env("BW_NOINTERACTION", "true"))
        .ok()
        .and_then(|o| parse_bw_status(&o.stdout))
        .ok_or_else(|| fail("bw status didn't answer", "Is /usr/local/bin/bw installed?"))?;
    let bw = |args: &[&str]| {
        Cmd::new(BW)
            .args(args.iter().copied())
            .env("BW_NOINTERACTION", "true")
    };
    let mut session = String::new();
    if status.status == "unauthenticated" {
        let url = server_url(&login.server)
            .map_err(|e| fail(e, "com, eu, or your server's https:// address."))?;
        let current = status
            .server
            .as_deref()
            .map(|s| server_url(s).unwrap_or_default())
            .unwrap_or_else(|| "https://vault.bitwarden.com".into());
        if current != url {
            ctx.log(Level::Info, format!("bw config server {url}"));
            ctx.sys.run(&bw(&["config", "server", &url])).map_err(|e| {
                fail(
                    format!("bw config server failed: {e}"),
                    "Check the address.",
                )
            })?;
        }
        ctx.prompt(PromptKind::Wait, "Logging in…", None);
        match login.method {
            BwMethod::Totp | BwMethod::YubikeyOtp => {
                let (email, pw, code) = (login.email.trim(), &login.password, login.code.trim());
                if email.is_empty() || pw.is_empty() || code.is_empty() {
                    return Err(fail(
                        "email, master password and code, all three",
                        "Fill in the empty one.",
                    ));
                }
                let method = if login.method == BwMethod::Totp {
                    "0"
                } else {
                    "3"
                };
                ctx.log(Level::Info, format!("bw login {email} --method {method}"));
                let out = ctx
                    .sys
                    .run(
                        &bw(&[
                            "login",
                            email,
                            "--passwordenv",
                            "BW_PASSWORD",
                            "--method",
                            method,
                            "--code",
                            code,
                            "--raw",
                        ])
                        .env("BW_PASSWORD", pw),
                    )
                    .map_err(|e| bw_fail(&e))?;
                session = out.stdout.trim().to_string();
            }
            BwMethod::ApiKey => {
                if login.client_id.trim().is_empty() || login.client_secret.trim().is_empty() {
                    return Err(fail(
                        "the API key needs client_id and client_secret",
                        "Web vault: Settings, Security, Keys, View API key.",
                    ));
                }
                ctx.log(Level::Info, "bw login --apikey");
                ctx.sys
                    .run(
                        &bw(&["login", "--apikey"])
                            .env("BW_CLIENTID", login.client_id.trim())
                            .env("BW_CLIENTSECRET", login.client_secret.trim()),
                    )
                    .map_err(|e| bw_fail(&e))?;
            }
        }
        remember(ctx, "server", &url)?;
        if !login.email.trim().is_empty() {
            remember(ctx, "email", login.email.trim())?;
        }
        ctx.log(Level::Info, format!("logged in on {url}"));
    }
    if session.is_empty() {
        if login.password.is_empty() {
            return Err(fail("the vault is locked", "Type the master password."));
        }
        ctx.prompt(PromptKind::Wait, "Unlocking the vault…", None);
        ctx.log(Level::Info, "bw unlock");
        let out = ctx
            .sys
            .run(
                &bw(&["unlock", "--passwordenv", "BW_PASSWORD", "--raw"])
                    .env("BW_PASSWORD", &login.password),
            )
            .map_err(|e| bw_fail(&e))?;
        session = out.stdout.trim().to_string();
    }
    if session.is_empty() {
        return Err(fail(
            "bw returned no session",
            "Try again; bw sometimes has a moment.",
        ));
    }

    ctx.prompt(
        PromptKind::Wait,
        "Reading the vault: SSH keys, git signing, gh, project env…",
        None,
    );
    let card = ctx.card;
    let mut warnings = 0;
    let emit = &mut *ctx.emit;
    let cmd = Cmd::new(ctx.env.script("dot-bw-setup")).env("BW_SESSION", &session);
    let r = ctx.sys.stream(&cmd, &mut |s| {
        let Stream::Line(l) = s else { return };
        let t = l.trim();
        if t.is_empty() {
            return;
        }
        let (level, line) = if let Some(s) = t.strip_prefix("==> ") {
            emit(SetupEvent::Prompt {
                card,
                kind: PromptKind::Wait,
                text: format!("{s}…"),
                count: None,
            });
            (Level::Info, s.to_string())
        } else if let Some(w) = t.strip_prefix("!!") {
            warnings += 1;
            (Level::Warn, w.trim().to_string())
        } else if let Some(e) = t.strip_prefix("xx") {
            (Level::Error, e.trim().to_string())
        } else {
            (Level::Info, t.to_string())
        };
        emit(SetupEvent::Log { card, level, line });
    });
    drop(session);
    r.map_err(|e| {
        fail(
            format!(
                "dot-bw-setup failed: {}",
                e.lines()
                    .last()
                    .unwrap_or(&e)
                    .trim_start_matches("xx")
                    .trim()
            ),
            "The log below has the whole story. dot-bw-setup in a terminal says the same.",
        )
    })?;
    if warnings > 0 {
        ctx.log(
            Level::Warn,
            format!("{warnings} thing(s) need you; they're in the lines marked above."),
        );
    }
    Ok(())
}

// --------------------------------------------------------------------- Git

/// Why git can't commit with this name and email.
pub fn git_identity_ok(name: &str, email: &str) -> Result<(), &'static str> {
    let (name, email) = (name.trim(), email.trim());
    if name.is_empty() {
        Err("Your name, the way commits should show it.")
    } else if name.contains(['\n', '\r']) || email.contains(['\n', '\r']) {
        Err("One line each, please.")
    } else if email.contains(char::is_whitespace)
        || !email
            .split_once('@')
            .is_some_and(|(u, d)| !u.is_empty() && !d.is_empty())
    {
        Err("That doesn't look like an email.")
    } else {
        Ok(())
    }
}

/// Writes the identity file config/git/config includes. Whatever was there
/// is replaced: the file is setup's (and the install's), nobody else's.
fn git(ctx: &mut Ctx, inputs: &Inputs) -> R<()> {
    let g = inputs.git.clone().unwrap_or_default();
    git_identity_ok(&g.name, &g.email)
        .map_err(|why| fail("no name and email to commit with", why))?;
    let path = ctx.env.home_path(crate::engine::files::GIT_IDENTITY);
    let (name, email) = (g.name.trim(), g.email.trim());
    ctx.sys
        .write(&path, &crate::engine::files::gitconfig(name, email))
        .map_err(|e| Fail::from(format!("can't write {path}: {e}")))?;
    ctx.log(Level::Info, format!("  ok  {path}: {name} <{email}>"));
    // git reads ~/.gitconfig after the include, so a [user] there wins.
    for (key, want) in [("user.name", name), ("user.email", email)] {
        let got = ctx
            .sys
            .run(&Cmd::new(detect::GIT).args(["config", "--global", "--includes", "--get", key]))
            .ok()
            .map(|o| o.stdout.trim().to_string());
        if let Some(got) = got.filter(|g| g != want) {
            return Err(fail(
                format!("git still says {key} = {got}"),
                "~/.gitconfig sets it and wins over the include. Delete the [user] lines there, \
                 or change them there.",
            ));
        }
    }
    Ok(())
}

fn bw_fail(e: &str) -> Fail {
    let (error, hint) = bw_error(e);
    Fail { error, hint }
}

/// `~/.config/dot-bw/config`: dot-bw-setup's non-secret settings.
fn remember(ctx: &mut Ctx, key: &str, value: &str) -> R<()> {
    let path = ctx.env.home_path(".config/dot-bw/config");
    let old = ctx.sys.read(&path).unwrap_or_default();
    let mut lines: Vec<String> = old
        .lines()
        .filter(|l| !l.starts_with(&format!("{key}=")) && !l.trim().is_empty())
        .map(str::to_string)
        .collect();
    lines.push(format!("{key}={value}"));
    ctx.sys
        .write(&path, &(lines.join("\n") + "\n"))
        .map_err(|e| Fail::from(format!("can't write {path}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::super::fake::FakeSys;
    use super::super::{BwLogin, GitIdentity, State};
    use super::*;
    use std::path::PathBuf;

    fn env() -> Env {
        Env {
            home: PathBuf::from("/home/kuba"),
            user: "kuba".into(),
            privileged: vec!["pkexec".into(), "/usr/lib/kubaos/setup-helper".into()],
        }
    }

    fn go(s: &mut FakeSys, card: CardId, inputs: &Inputs) -> (bool, Vec<SetupEvent>) {
        let mut evs = vec![];
        let ok = run(s, &env(), card, inputs, &mut |e| evs.push(e));
        (ok, evs)
    }

    fn all_text(evs: &[SetupEvent]) -> String {
        evs.iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn tuxedo_sb_on() -> FakeSys {
        FakeSys::new()
            .file("/sys/class/dmi/id/sys_vendor", "TUXEDO\n")
            .file(
                "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c",
                "\u{6}\0\0\0\u{1}",
            )
            .ok("dpkg --print-architecture", "amd64\n")
            .ok("nmcli", "full\n")
    }

    #[test]
    fn tuxedo_with_secure_boot_wants_a_sane_mok_password() {
        let mut s = tuxedo_sb_on().fail("dpkg-query", "not installed");
        let (ok, evs) = go(&mut s, CardId::Tuxedo, &Inputs::default());
        assert!(!ok);
        assert!(matches!(evs.last(), Some(SetupEvent::Failed { .. })));
        assert!(!s.ran_any("pkexec"));

        let inputs = Inputs {
            mok_password: Some("kuba2026".into()),
            ..Default::default()
        };
        let mut s = tuxedo_sb_on().fail("dpkg-query", "not installed").ok(
            "pkexec /usr/lib/kubaos/setup-helper tuxedo --mok-password-stdin",
            "==> TUXEDO apt repository (resolute)\n  ok  done\n",
        );
        let (ok, evs) = go(&mut s, CardId::Tuxedo, &inputs);
        assert!(ok, "{}", all_text(&evs));
        let i = s.ran.iter().position(|l| l.starts_with("pkexec")).unwrap();
        assert_eq!(s.stdin[i].as_deref(), Some("kuba2026\n"));
        // The password is in no event.
        assert!(!all_text(&evs).contains("kuba2026"));
        assert!(matches!(evs.last(), Some(SetupEvent::Finished { .. })));
    }

    #[test]
    fn mok_password_rules() {
        assert!(mok_password_ok("kuba2026").is_ok());
        assert!(mok_password_ok("short").is_err());
        assert!(mok_password_ok("zażółć12").is_err());
        assert!(mok_password_ok("with space1").is_err());
    }

    #[test]
    fn pkexec_dismissed_is_a_plain_message() {
        let mut s = FakeSys::new()
            .file("/sys/class/dmi/id/sys_vendor", "TUXEDO\n")
            .ok("dpkg --print-architecture", "amd64\n")
            .fail("dpkg-query", "x")
            .fail("pkexec", "pkexec exited with exit status: 126: ");
        let (ok, evs) = go(&mut s, CardId::Tuxedo, &Inputs::default());
        assert!(!ok);
        match evs.last() {
            Some(SetupEvent::Failed { error, .. }) => assert!(error.contains("dialog")),
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn fingerprint_counts_touches() {
        let mut s = FakeSys::new()
            .file("/usr/bin/fprintd-list", "")
            .file("/etc/pam.d/common-auth", "pam_fprintd.so\n")
            .ok("fprintd-list", "User kuba has no fingers enrolled for Goodix MOC Fingerprint Sensor.\n")
            .ok("busctl get-property", "i 4\n")
            .ok(
                "stdbuf -oL fprintd-enroll -f right-index-finger",
                "Using device /net/reactivated/Fprint/Device/0\nEnrolling right-index-finger finger.\nEnroll result: enroll-stage-passed\nEnroll result: enroll-retry-scan\nEnroll result: enroll-stage-passed\nEnroll result: enroll-stage-passed\nEnroll result: enroll-stage-passed\nEnroll result: enroll-completed\n",
            )
            .ok("nmcli", "full\n");
        let (ok, evs) = go(&mut s, CardId::Fingerprint, &Inputs::default());
        assert!(ok, "{}", all_text(&evs));
        let counts: Vec<(u32, u32)> = evs
            .iter()
            .filter_map(|e| match e {
                SetupEvent::Prompt { count: Some(c), .. } => Some(*c),
                _ => None,
            })
            .collect();
        assert_eq!(counts, [(0, 4), (1, 4), (1, 4), (2, 4), (3, 4), (4, 4)]);
        // fprintd and PAM were there already: no root needed.
        assert!(!s.ran_any("pkexec"));
    }

    #[test]
    fn fingerprint_installs_fprintd_first() {
        let mut s = FakeSys::new()
            .file("/sys/bus/usb/devices/3-3/idVendor", "27c6\n")
            .ok(
                "pkexec /usr/lib/kubaos/setup-helper fingerprint",
                "==> Fingerprint reader (fprintd)\n",
            )
            .fail("busctl", "no")
            .ok("stdbuf", "Enroll result: enroll-duplicate\n");
        let (ok, evs) = go(&mut s, CardId::Fingerprint, &Inputs::default());
        assert!(!ok);
        assert!(s.ran_any("pkexec /usr/lib/kubaos/setup-helper fingerprint"));
        match evs.last() {
            Some(SetupEvent::Failed { error, .. }) => assert!(error.contains("enrolled already")),
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn yubikey_enrolls_through_the_helper() {
        let cred = "owBYtd,pDtKMa,es256,+presence+verification";
        let mut s = FakeSys::new()
            .ok(
                "fido2-token -L",
                "/dev/hidraw3: vendor=0x1050, product=0x0402 (Yubico YubiKey FIDO)\n",
            )
            .ok(
                "/home/kuba/.local/bin/dot-u2f-enroll --info",
                "  PIN: set\n  fingerprints: enrolled\n  this host:    not enrolled (pam://vm)\n",
            )
            .ok("pamu2fcfg -n -u kuba -V", &format!(":{cred}\n"))
            .ok(
                "pkexec /usr/lib/kubaos/setup-helper u2f-add",
                "  ok  added\n",
            );
        let (ok, evs) = go(&mut s, CardId::Yubikey, &Inputs::default());
        assert!(ok, "{}", all_text(&evs));
        let i = s.ran.iter().position(|l| l.contains("u2f-add")).unwrap();
        assert_eq!(s.stdin[i].as_deref(), Some(&*format!("{cred}\n")));
        assert!(evs.iter().any(|e| matches!(
            e,
            SetupEvent::Prompt {
                kind: PromptKind::Key,
                ..
            }
        )));
    }

    #[test]
    fn yubikey_without_pin_says_how() {
        let mut s = FakeSys::new().ok("fido2-token -L", "/dev/hidraw3: x\n").ok(
            "/home/kuba/.local/bin/dot-u2f-enroll --info",
            "  PIN: NOT set\n  fingerprints: none yet\n",
        );
        let (ok, evs) = go(&mut s, CardId::Yubikey, &Inputs::default());
        assert!(!ok);
        match evs.last() {
            Some(SetupEvent::Failed { hint: Some(h), .. }) => assert!(h.contains("ykman")),
            e => panic!("{e:?}"),
        }
        assert!(!s.ran_any("pamu2fcfg"));
    }

    fn bw_login() -> Inputs {
        Inputs {
            bw: Some(BwLogin {
                server: "eu".into(),
                email: "k@x.eu".into(),
                password: "hunter2-master".into(),
                method: BwMethod::Totp,
                code: "123456".into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn bitwarden_logs_in_and_runs_dot_bw_setup() {
        let mut s = FakeSys::new()
            .file(BW, "")
            .ok("/usr/local/bin/bw status", r#"{"serverUrl":null,"status":"unauthenticated"}"#)
            .ok("/usr/local/bin/bw config server https://vault.bitwarden.eu", "")
            .ok("/usr/local/bin/bw login k@x.eu --passwordenv BW_PASSWORD --method 0 --code 123456 --raw", "SESSIONKEY==\n")
            .ok(
                "/home/kuba/.local/bin/dot-bw-setup",
                "==> Bitwarden login\n  ok  unlocked\n  !!  no folder \"dotfiles\" in the vault\n",
            )
            .ok("nmcli", "full\n");
        let (ok, evs) = go(&mut s, CardId::Bitwarden, &bw_login());
        assert!(ok, "{}", all_text(&evs));
        // The master password only in bw's environment, the session only in
        // dot-bw-setup's; neither in argv or an event.
        let login = s.ran.iter().position(|l| l.contains("bw login")).unwrap();
        assert!(s.env[login].contains(&("BW_PASSWORD".into(), "hunter2-master".into())));
        assert!(
            !s.ran
                .iter()
                .any(|l| l.contains("hunter2") || l.contains("SESSIONKEY"))
        );
        let setup = s
            .ran
            .iter()
            .position(|l| l.contains("dot-bw-setup"))
            .unwrap();
        assert!(s.env[setup].contains(&("BW_SESSION".into(), "SESSIONKEY==".into())));
        let text = all_text(&evs);
        assert!(!text.contains("hunter2") && !text.contains("SESSIONKEY"));
        assert!(text.contains("no folder"));
        assert_eq!(
            s.files
                .get("/home/kuba/.config/dot-bw/config")
                .map(String::as_str),
            Some("server=https://vault.bitwarden.eu\nemail=k@x.eu\n")
        );
    }

    #[test]
    fn bitwarden_locked_only_unlocks() {
        let mut s = FakeSys::new()
            .file(BW, "")
            .ok(
                "/usr/local/bin/bw status",
                r#"{"serverUrl":"https://vault.bitwarden.eu","userEmail":"k@x.eu","status":"locked"}"#,
            )
            .ok("/usr/local/bin/bw unlock --passwordenv BW_PASSWORD --raw", "S\n")
            .ok("/home/kuba/.local/bin/dot-bw-setup", "")
            .ok("nmcli", "full\n");
        let (ok, _) = go(&mut s, CardId::Bitwarden, &bw_login());
        assert!(ok);
        assert!(!s.ran_any("/usr/local/bin/bw login"));
    }

    #[test]
    fn bitwarden_wrong_password_names_the_cause() {
        let mut s = FakeSys::new()
            .file(BW, "")
            .ok(
                "/usr/local/bin/bw status",
                r#"{"serverUrl":"https://vault.bitwarden.eu","status":"unauthenticated"}"#,
            )
            .fail(
                "/usr/local/bin/bw login",
                "bw exited with exit status: 1: Username or password is incorrect. Try again.",
            );
        let (ok, evs) = go(&mut s, CardId::Bitwarden, &bw_login());
        assert!(!ok);
        match evs.last() {
            Some(SetupEvent::Failed { error, .. }) => {
                assert_eq!(error, "wrong email or master password")
            }
            e => panic!("{e:?}"),
        }
        assert!(!s.ran_any("/home/kuba/.local/bin/dot-bw-setup"));
        // Same server: no bw config call.
        assert!(!s.ran_any("/usr/local/bin/bw config"));
    }

    #[test]
    fn git_writes_the_identity_file_quoted() {
        let mut s = FakeSys::new();
        let inputs = Inputs {
            git: Some(GitIdentity {
                name: "  Kuba \"K\" W\\asil ".into(),
                email: " kuba@example.com ".into(),
            }),
            ..Default::default()
        };
        let (ok, evs) = go(&mut s, CardId::Git, &inputs);
        assert!(ok, "{}", all_text(&evs));
        assert_eq!(
            s.files["/home/kuba/.config/git-identity/identity.gitconfig"],
            "[user]\n\tname = \"Kuba \\\"K\\\" W\\\\asil\"\n\temail = \"kuba@example.com\"\n"
        );
        assert!(matches!(evs.last(), Some(SetupEvent::Finished { .. })));
    }

    #[test]
    fn git_says_when_gitconfig_wins() {
        // ~/.gitconfig is read after the include, so its [user] wins.
        let mut s = FakeSys::new().ok(
            "/usr/bin/git config --global --includes --get user.name",
            "Old Name\n",
        );
        let inputs = Inputs {
            git: Some(GitIdentity {
                name: "Kuba Example".into(),
                email: "kuba@example.com".into(),
            }),
            ..Default::default()
        };
        let (ok, evs) = go(&mut s, CardId::Git, &inputs);
        assert!(!ok);
        match evs.last() {
            Some(SetupEvent::Failed { hint: Some(h), .. }) => {
                assert!(h.contains("~/.gitconfig"), "{h}")
            }
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn git_wants_a_name_and_a_real_email() {
        for (name, email) in [("", "a@b.c"), ("A", "nope"), ("A\nB", "a@b.c")] {
            let mut s = FakeSys::new();
            let inputs = Inputs {
                git: Some(GitIdentity {
                    name: name.into(),
                    email: email.into(),
                }),
                ..Default::default()
            };
            let (ok, evs) = go(&mut s, CardId::Git, &inputs);
            assert!(!ok, "{name:?} {email:?}");
            assert!(matches!(evs.last(), Some(SetupEvent::Failed { .. })));
            assert!(s.files.is_empty());
        }
    }

    #[test]
    fn finished_carries_the_fresh_state() {
        let mut s = FakeSys::new()
            .ok("fido2-token -L", "/dev/hidraw3: x\n")
            .file("/sys/bus/usb/devices/1-2/idVendor", "1050\n")
            .file("/etc/u2f_mappings", "kuba:a,b,es256,+presence\n")
            .ok(
                "/home/kuba/.local/bin/dot-u2f-enroll --info",
                "  PIN: set\n  fingerprints: enrolled\n  this host:    enrolled as key 1 (pam://vm)\n",
            );
        let (ok, evs) = go(&mut s, CardId::Yubikey, &Inputs::default());
        assert!(ok);
        match evs.last() {
            Some(SetupEvent::Finished { status, .. }) => assert_eq!(status.state, State::Done),
            e => panic!("{e:?}"),
        }
        assert!(!s.ran_any("pamu2fcfg"));
    }
}
