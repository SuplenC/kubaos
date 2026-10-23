//! What each card does. Every action ends with one `Finished` (the card
//! detected again) or one `Failed`.
//!
//! Secrets: the MOK password goes to the helper on stdin; the master
//! password and API key go to `bw` in its environment (`--passwordenv`,
//! `BW_CLIENTID`), and the session stays in this process, going to each `bw`
//! call as `BW_SESSION` ([`crate::bitwarden`]), the way docs/SECRETS.md has
//! it. None of them is logged. The one exception to "no secrets in argv" is
//! the two-step code (`bw login --code`): bw has no other way to take it
//! without a terminal, and it's spent the moment bw uses it.

use super::detect;
use super::parse::*;
use super::{
    BwLogin, BwMethod, CardId, Cmd, Env, Facts, Inputs, MokState, PromptKind, SetupEvent, Stream,
    Sys,
};
use crate::bitwarden;
use crate::bitwarden::FormMethod;
use crate::bitwarden::Line;
use crate::bitwarden::LoginRequest;
use crate::bitwarden::NoTerminal;
use crate::bitwarden::Parts;
use crate::bitwarden::Progress;
use crate::bitwarden::Sensitive;
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
        )?;
        key_queued(ctx)
    } else {
        ctx.privileged(&["tuxedo"], None, "Installing TUXEDO's drivers…")
    }
}

/// The helper exits 0 when it has nothing to enroll, so success alone would send the card
/// back to an empty password form with no word of what went wrong.
fn key_queued(ctx: &mut Ctx) -> R<()> {
    let c = detect::card_status(ctx.sys, ctx.env, CardId::Tuxedo, true);
    match c.facts {
        Facts::Tuxedo {
            mok: Some(MokState::NoKey),
            ..
        } => Err(fail(
            "the drivers are in, but DKMS made no signing key to enroll",
            "Rebuild them so DKMS makes one (sudo dpkg-reconfigure tuxedo-drivers), then try again.",
        )),
        Facts::Tuxedo {
            mok: Some(MokState::NotEnrolled),
            ..
        } => Err(fail(
            "the drivers' key isn't queued for enrollment",
            "mokutil --import didn't take it. The details are in the log below.",
        )),
        _ => Ok(()),
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

/// Logs in or unlocks with the form, then pulls everything from the vault, the way
/// `kubaos bitwarden setup` does on a terminal. Or logs out, for another account or server.
fn bitwarden(ctx: &mut Ctx, inputs: &Inputs) -> R<()> {
    let paths = ctx.env.bitwarden.clone();
    let card = ctx.card;
    let emit = &mut *ctx.emit;
    let m = ctx.sys.bitwarden();
    let wait = |text: String| SetupEvent::Prompt {
        card,
        kind: PromptKind::Wait,
        text,
        count: None,
    };
    let log = |level, line: String| SetupEvent::Log { card, level, line };
    if inputs.bw_logout {
        emit(wait("Logging out…".into()));
        bitwarden::logout(m).map_err(bw_failure)?;
        emit(log(Level::Info, "logged out of Bitwarden".into()));
        return Ok(());
    }
    let Some(login) = &inputs.bw else {
        return Err(fail("nothing to log in with", "Fill in the form first."));
    };
    if let Some(folders) = &inputs.bw_folders {
        bitwarden::set_folders(m, &paths, folders).map_err(bw_failure)?;
    }
    bitwarden::preflight(m, Parts::ALL).map_err(bw_failure)?;
    let mut on_line = |l: &Line| match l {
        Line::Step(s) => {
            emit(wait(format!("{s}…")));
            emit(log(Level::Info, s.clone()));
        }
        Line::Ok(s) => emit(log(Level::Info, format!("ok  {s}"))),
        Line::Info(s) => emit(log(Level::Info, s.clone())),
        Line::Warn(s) => emit(log(Level::Warn, s.clone())),
    };
    let mut p = Progress::new(&mut on_line);
    let req = LoginRequest {
        form: Some(form_of(login)),
        ..LoginRequest::default()
    };
    let session = bitwarden::login(m, &paths, &req, &mut NoTerminal, &mut p).map_err(bw_failure)?;
    bitwarden::pull(m, &paths, &session, Parts::ALL, &mut p).map_err(bw_failure)?;
    drop(session);
    let report = p.finish();
    for line in report.summary {
        emit(log(Level::Info, line));
    }
    if !report.warnings.is_empty() {
        emit(log(
            Level::Warn,
            format!(
                "{} thing(s) need you; they're in the lines marked above.",
                report.warnings.len()
            ),
        ));
    }
    Ok(())
}

/// The card's form, for [`bitwarden::login`].
fn form_of(l: &BwLogin) -> bitwarden::Form {
    bitwarden::Form {
        server: l.server.clone(),
        email: l.email.clone(),
        password: Sensitive(l.password.clone()),
        method: match l.method {
            BwMethod::Totp => FormMethod::Totp,
            BwMethod::YubikeyOtp => FormMethod::YubikeyOtp,
            BwMethod::ApiKey => FormMethod::ApiKey,
        },
        code: l.code.clone(),
        client_id: l.client_id.clone(),
        client_secret: Sensitive(l.client_secret.clone()),
    }
}

/// What stopped a Bitwarden run, in the card's words: bw's own reason when it gave one.
fn bw_failure(e: bitwarden::Error) -> Fail {
    use bitwarden::Error as E;
    match e {
        E::Login { cause, .. } | E::Unlock(cause) | E::NotUnlocked(cause) => {
            let (error, hint) = bw_error(cause.detail());
            Fail { error, hint }
        }
        E::Incomplete(what) => {
            let hint = match what {
                "the vault is locked" => "Type the master password.",
                "the API key needs client_id and client_secret" => {
                    "Web vault: Settings, Security, Keys, View API key."
                }
                _ => "Fill in the empty one.",
            };
            fail(what, hint)
        }
        E::Server(why) => fail(why, "com, eu, or your server's https:// address."),
        other => {
            let error = match std::error::Error::source(&other) {
                Some(cause) => format!("{other}: {cause}"),
                None => other.to_string(),
            };
            fail(
                error,
                "The log below has the whole story. kubaos bitwarden setup in a terminal says the same.",
            )
        }
    }
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

#[cfg(test)]
mod tests {
    use super::super::fake::FakeSys;
    use super::super::{GitIdentity, State};
    use super::*;
    use crate::bitwarden::Login;
    use crate::bitwarden::fake::Fake;
    use crate::bitwarden::fake::SESSION;
    use std::path::PathBuf;

    fn env() -> Env {
        Env {
            home: PathBuf::from("/home/kuba"),
            user: "kuba".into(),
            privileged: vec!["pkexec".into(), "/usr/lib/kubaos/setup-helper".into()],
            bitwarden: crate::bitwarden::Paths::under(std::path::Path::new("/home/kuba")),
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

    fn tuxedo_drivers_in_key_not_enrolled() -> FakeSys {
        tuxedo_sb_on()
            .file("/var/lib/shim-signed/mok/MOK.der", "cert")
            .ok("dpkg-query", "install ok installed")
            .ok(
                "sh -c mokutil",
                "/var/lib/shim-signed/mok/MOK.der is not enrolled\n",
            )
            .ok(
                "pkexec /usr/lib/kubaos/setup-helper tuxedo --mok-password-stdin",
                "==> Secure Boot: DKMS signing key (MOK)\n  ok  queued\n",
            )
    }

    fn kuba2026() -> Inputs {
        Inputs {
            mok_password: Some("kuba2026".into()),
            ..Default::default()
        }
    }

    #[test]
    fn a_queued_key_finishes_waiting_on_the_reboot() {
        // The helper's `mokutil --import` leaves MokNew, which only root can read.
        let mut s = tuxedo_drivers_in_key_not_enrolled().creates(
            "pkexec /usr/lib/kubaos/setup-helper tuxedo --mok-password-stdin",
            detect::MOK_NEW_VAR,
        );

        let (ok, evs) = go(&mut s, CardId::Tuxedo, &kuba2026());

        assert!(ok, "{}", all_text(&evs));
        match evs.last() {
            Some(SetupEvent::Finished { status, .. }) => assert_eq!(status.state, State::Reboot),
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn a_helper_run_that_leaves_the_key_unqueued_fails_instead_of_finishing() {
        let mut s = tuxedo_drivers_in_key_not_enrolled();

        let (ok, evs) = go(&mut s, CardId::Tuxedo, &kuba2026());

        assert!(!ok);
        match evs.last() {
            Some(SetupEvent::Failed { error, .. }) => {
                assert!(error.contains("isn't queued"), "{error}")
            }
            e => panic!("{e:?}"),
        }
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

    /// A vault with one SSH key in the folder Work, the agent serving it, git answering.
    fn bw_machine(bw: Fake) -> FakeSys {
        let folders = r#"[{"id":"f-work","name":"Work"},{"id":null,"name":"No Folder"}]"#;
        let items = r#"[{"id":"i1","type":5,"name":"GitHub","folderId":"f-work","creationDate":"2026-01-01",
            "fields":[{"name":"hosts","value":"github.com","type":0}],
            "sshKey":{"privateKey":"-----BEGIN OPENSSH PRIVATE KEY-----","publicKey":"ssh-ed25519 AAAAgh",
            "keyFingerprint":"SHA256:gh"}}]"#;
        let mut bw = bw
            .vault(folders, items)
            .answer("timeout 20 ssh-add", 0, "256 SHA256:gh GitHub (ED25519)\n");
        bw.sockets.insert("/home/kuba/.bitwarden-ssh-agent.sock".into());
        let mut s = FakeSys::new().ok("nmcli", "full\n");
        s.bw = bw;
        s
    }

    #[test]
    fn bitwarden_logs_in_and_pulls_the_vault() {
        let mut s = bw_machine(Fake::new());

        let (ok, evs) = go(&mut s, CardId::Bitwarden, &bw_login());

        assert!(ok, "{}", all_text(&evs));
        // The master password only in bw's environment, the session only in this process:
        // neither in a command line or an event.
        assert!(matches!(&s.bw.logins[0], Login::Password { password, .. } if password.0 == "hunter2-master"));
        let lines = s.bw.lines().join("\n");
        assert!(!lines.contains("hunter2") && !lines.contains(SESSION));
        let text = all_text(&evs);
        assert!(!text.contains("hunter2") && !text.contains(SESSION));
        assert!(text.contains("signing=true"), "{text}");
        assert!(text.contains("1 SSH key(s) in your vault"));
        assert_eq!(
            s.bw.text("/home/kuba/.config/dot-bw/config"),
            Some("server=https://vault.bitwarden.eu\nemail=k@x.eu\n")
        );
        match evs.last() {
            Some(SetupEvent::Finished { status, .. }) => {
                assert_eq!(status.state, State::Done);
                let Facts::Bitwarden { overview, .. } = &status.facts else {
                    panic!("{:?}", status.facts)
                };
                assert_eq!(overview.keys[0].hosts, ["github.com"]);
            }
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn bitwarden_locked_only_unlocks() {
        let mut s = bw_machine(Fake::new().locked("https://vault.bitwarden.eu", "k@x.eu"));

        let (ok, evs) = go(&mut s, CardId::Bitwarden, &bw_login());

        assert!(ok, "{}", all_text(&evs));
        assert!(!s.bw.bw.iter().any(|c| c == "login"));
        assert!(s.bw.bw.iter().any(|c| c == "unlock"));
    }

    #[test]
    fn bitwarden_wrong_password_names_the_cause() {
        let mut bw = Fake::new();
        bw.server = Some("https://vault.bitwarden.eu".into());
        bw.login_error = Some("Username or password is incorrect. Try again.".into());
        let mut s = bw_machine(bw);

        let (ok, evs) = go(&mut s, CardId::Bitwarden, &bw_login());

        assert!(!ok);
        match evs.last() {
            Some(SetupEvent::Failed { error, .. }) => {
                assert_eq!(error, "wrong email or master password")
            }
            e => panic!("{e:?}"),
        }
        assert!(!s.bw.bw.iter().any(|c| c == "sync"));
        // Same server: no bw config call.
        assert!(!s.bw.bw.iter().any(|c| c.starts_with("config server")));
    }

    #[test]
    fn bitwarden_ticked_folders_narrow_the_ssh_keys() {
        let mut s = bw_machine(Fake::new().locked("https://vault.bitwarden.eu", "k@x.eu"));
        let inputs = Inputs {
            bw_folders: Some(vec!["Work".into()]),
            ..bw_login()
        };

        let (ok, evs) = go(&mut s, CardId::Bitwarden, &inputs);

        assert!(ok, "{}", all_text(&evs));
        assert!(s.bw.text("/home/kuba/.config/dot-bw/config").unwrap().contains("folders=Work\n"));
        assert!(all_text(&evs).contains(r#"1 SSH key(s) in the folder \"Work\""#));
        match evs.last() {
            Some(SetupEvent::Finished { status, .. }) => {
                assert!(matches!(&status.facts, Facts::Bitwarden { folders, .. } if folders == &["Work"]))
            }
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn bitwarden_logs_out_and_asks_for_a_login_again() {
        let mut s = bw_machine(Fake::new().locked("https://vault.bitwarden.eu", "k@x.eu"));
        let inputs = Inputs {
            bw_logout: true,
            ..Inputs::default()
        };

        let (ok, evs) = go(&mut s, CardId::Bitwarden, &inputs);

        assert!(ok, "{}", all_text(&evs));
        assert!(s.bw.bw.iter().any(|c| c == "logout"));
        match evs.last() {
            Some(SetupEvent::Finished { status, .. }) => assert_eq!(status.state, State::Needed),
            e => panic!("{e:?}"),
        }
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
