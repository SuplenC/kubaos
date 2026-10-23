//! Logging in and unlocking. On a terminal bw asks for the master password and the two-step
//! code itself; from Kōbō's form they go to bw's environment and its `--code`, never further.

use super::Config;
use super::Error;
use super::Machine;
use super::Paths;
use super::Progress;
use super::Sensitive;
use super::Session;
use crate::setup::parse_bw_status;
use crate::setup::server_url;

/// How a terminal login goes when bw is logged out (`--login`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Email, master password and the authenticator app's code. bw can't use a FIDO2 key.
    Totp,
    /// The personal API key, for an account with no authenticator app as its two-step method.
    ApiKey,
    /// The organization's identity provider, in the browser.
    Sso,
}

/// The second step Kōbō's form offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMethod {
    Totp,
    YubikeyOtp,
    ApiKey,
}

/// What Kōbō's form holds. The secrets are [`Sensitive`], so `Debug` never prints them.
#[derive(Debug, Clone)]
pub struct Form {
    pub server: String,
    pub email: String,
    pub password: Sensitive,
    pub method: FormMethod,
    /// The one-time code. It goes on bw's command line, where it is spent before anyone could
    /// read it.
    pub code: String,
    pub client_id: String,
    pub client_secret: Sensitive,
}

/// What a login starts from.
#[derive(Debug, Clone, Default)]
pub struct LoginRequest {
    /// A refresh asks nothing: it fails when bw is logged out or no server is set.
    pub refresh: bool,
    /// `--server`: com, eu or a URL.
    pub server: Option<String>,
    pub method: Option<Method>,
    /// From Kōbō: everything is in the form, and nothing is asked.
    pub form: Option<Form>,
    /// `BW_SESSION`, when the person exported one.
    pub session: Option<Session>,
}

/// One `bw login`, as [`super::Bw::login`] runs it.
#[derive(Debug, Clone)]
pub enum Login {
    /// bw asks for the master password and the authenticator code on the terminal.
    Totp { email: String },
    /// bw asks for client_id and client_secret on the terminal.
    ApiKeyPrompt,
    Sso,
    /// From the form: the password in bw's environment, `--method 0` or `3`.
    Password {
        email: String,
        password: Sensitive,
        method: &'static str,
        code: String,
    },
    /// From the form: the key in bw's environment.
    ApiKey { client_id: String, client_secret: Sensitive },
}

/// Questions for the person at the terminal.
pub trait Ask {
    /// The answer, or `None` when stdin isn't a terminal.
    fn ask(&mut self, prompt: &str) -> Option<String>;
    fn tty(&self) -> bool;
}

/// Nobody to ask: Kōbō, a refresh, or a pipe.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoTerminal;

impl Ask for NoTerminal {
    fn ask(&mut self, _prompt: &str) -> Option<String> {
        None
    }

    fn tty(&self) -> bool {
        false
    }
}

fn ask(a: &mut dyn Ask, prompt: &str) -> Result<String, Error> {
    a.ask(prompt).ok_or_else(|| Error::NoTerminal(prompt.to_string()))
}

/// Logs in when bw is logged out, unlocks, and hands back the session for this run.
///
/// # Errors
///
/// bw refused, the person picked nothing, or a refresh found bw logged out.
pub fn login(
    m: &mut dyn Machine,
    paths: &Paths,
    req: &LoginRequest,
    asker: &mut dyn Ask,
    p: &mut Progress,
) -> Result<Session, Error> {
    p.step("Bitwarden login");
    let json = m.status(req.session.as_ref()).map_err(Error::Status)?;
    let status = parse_bw_status(&json).ok_or_else(|| Error::UnknownStatus(super::one_line(json.trim())))?;
    let raw = status.server.as_deref().unwrap_or("https://vault.bitwarden.com");
    let current = server_url(raw).unwrap_or_else(|_| raw.to_string());
    let email = status.email.clone().unwrap_or_default();
    let logged_out = status.status == "unauthenticated";

    let mut config = Config::read(m, paths);
    let mut server = config.get("server").map(str::to_string);
    if let Some(form) = req.form.as_ref().filter(|_| logged_out) {
        server = Some(server_url(&form.server).map_err(Error::Server)?);
    }
    if let Some(s) = &req.server {
        server = Some(server_url(s).map_err(Error::Server)?);
    }
    if server.is_none() && !logged_out {
        server = Some(current.clone());
    }
    let server = match server {
        Some(s) => s,
        None if req.refresh => return Err(Error::NotSetUp),
        None => ask_server(asker, p)?,
    };
    if config.get("server") != Some(server.as_str()) {
        config.set("server", &server);
        config.write(m, paths)?;
    }
    if !logged_out && current != server {
        return Err(Error::OtherServer { current, wanted: server });
    }

    let mut session = None;
    match status.status.as_str() {
        "unlocked" if req.session.is_some() => {
            p.ok(format!("unlocked ({email}, session from BW_SESSION)"));
            return req.session.clone().ok_or(Error::NoSession);
        }
        // A BW_SESSION bw doesn't take: unlock again.
        "unlocked" | "locked" => p.ok(format!("logged in as {email} on {server}")),
        "unauthenticated" => {
            if req.refresh {
                return Err(Error::LoggedOut);
            }
            if current != server {
                m.set_server(&server).map_err(Error::SetServer)?;
            }
            session = match &req.form {
                Some(form) => form_login(m, paths, &mut config, form)?,
                None => terminal_login(m, paths, &mut config, req.method, asker, p)?,
            };
            p.ok(format!("logged in on {server}"));
        }
        other => return Err(Error::UnknownStatus(super::one_line(other))),
    }

    let session = match session {
        Some(s) => s,
        None => {
            let password = req.form.as_ref().map(|f| &f.password);
            if password.is_some_and(|pw| pw.0.is_empty()) {
                return Err(Error::Incomplete("the vault is locked"));
            }
            // bw prompts for the master password itself; the 2026.10 bw can also ask the
            // desktop app to unlock with system authentication, when the server allows it.
            m.unlock(password).map_err(Error::Unlock)?
        }
    };
    if session.expose().is_empty() {
        return Err(Error::NoSession);
    }
    p.ok("vault unlocked for this run");
    Ok(session)
}

fn ask_server(asker: &mut dyn Ask, p: &mut Progress) -> Result<String, Error> {
    p.info("Which Bitwarden server holds your vault?");
    p.info("  1) bitwarden.com (US cloud)   2) bitwarden.eu (EU cloud)   3) self-hosted");
    match ask(asker, "Choice [1-3]:")?.trim() {
        "1" => Ok("https://vault.bitwarden.com".into()),
        "2" => Ok("https://vault.bitwarden.eu".into()),
        "3" => server_url(&ask(asker, "Server URL (https://...):")?).map_err(Error::Server),
        _ => Err(Error::Choice),
    }
}

fn remember_email(m: &mut dyn Machine, paths: &Paths, config: &mut Config, email: &str) -> Result<(), Error> {
    if email.is_empty() || config.get("email") == Some(email) {
        return Ok(());
    }
    config.set("email", email);
    config.write(m, paths)
}

fn form_login(m: &mut dyn Machine, paths: &Paths, config: &mut Config, f: &Form) -> Result<Option<Session>, Error> {
    let email = f.email.trim();
    let session = match f.method {
        FormMethod::Totp | FormMethod::YubikeyOtp => {
            let code = f.code.trim();
            if email.is_empty() || f.password.0.is_empty() || code.is_empty() {
                return Err(Error::Incomplete("email, master password and code, all three"));
            }
            let how = Login::Password {
                email: email.to_string(),
                password: f.password.clone(),
                method: if f.method == FormMethod::Totp { "0" } else { "3" },
                code: code.to_string(),
            };
            m.login(&how).map_err(|cause| Error::Login { what: "login failed", cause })?
        }
        FormMethod::ApiKey => {
            let (id, secret) = (f.client_id.trim(), f.client_secret.0.trim());
            if id.is_empty() || secret.is_empty() {
                return Err(Error::Incomplete("the API key needs client_id and client_secret"));
            }
            let how = Login::ApiKey {
                client_id: id.to_string(),
                client_secret: Sensitive(secret.to_string()),
            };
            m.login(&how)
                .map_err(|cause| Error::Login { what: "API key login failed", cause })?;
            None
        }
    };
    remember_email(m, paths, config, email)?;
    Ok(session)
}

fn terminal_login(
    m: &mut dyn Machine,
    paths: &Paths,
    config: &mut Config,
    method: Option<Method>,
    asker: &mut dyn Ask,
    p: &mut Progress,
) -> Result<Option<Session>, Error> {
    let mut chosen = match method {
        Some(m) => m,
        None => {
            p.info("How do you log in? bw can't use the YubiKey (FIDO2) for two-step login;");
            p.info("it does authenticator-app codes, email codes and YubiKey OTP.");
            p.info("  1) email + master password + authenticator-app code (recommended, Enter)");
            p.info("  2) personal API key (web vault: Settings -> Security -> Keys -> View API key),");
            p.info("     if you have no authenticator app as two-step method; asked now, never stored");
            p.info("  3) SSO (your organization's identity provider, in the browser)");
            match ask(asker, "Choice [1-3]:")?.trim() {
                "" | "1" => Method::Totp,
                "2" => Method::ApiKey,
                "3" => Method::Sso,
                _ => return Err(Error::Choice),
            }
        }
    };
    if chosen == Method::Totp {
        let email = match config.get("email") {
            Some(e) => e.to_string(),
            None => {
                let e = ask(asker, "Email:")?.trim().to_string();
                remember_email(m, paths, config, &e)?;
                e
            }
        };
        // --method 0: the authenticator app, even when a YubiKey is the account's preferred
        // method. bw prompts on stderr and prints only the session on stdout.
        match m.login(&Login::Totp { email }) {
            Ok(s) => return Ok(s),
            Err(cause) => {
                p.warn("authenticator-app login failed (wrong code, or no authenticator app on the account?)");
                if !asker.tty() || method.is_some() {
                    return Err(Error::Login {
                        what: "try again, or: kubaos bitwarden setup --login apikey",
                        cause,
                    });
                }
                let again = ask(asker, "Log in with the personal API key instead? [y/N]")?;
                if !again.trim().to_ascii_lowercase().starts_with('y') {
                    return Err(Error::NotLoggedIn);
                }
                p.unwarn();
                chosen = Method::ApiKey;
            }
        }
    }
    let (how, what) = match chosen {
        Method::Sso => (Login::Sso, "SSO login failed"),
        _ => (Login::ApiKeyPrompt, "API key login failed"),
    };
    m.login(&how).map_err(|cause| Error::Login { what, cause })?;
    Ok(None)
}

/// Logs bw out, for another account or another server.
///
/// # Errors
///
/// bw failed.
pub fn logout(m: &mut dyn Machine) -> Result<(), Error> {
    m.logout().map_err(Error::Logout)
}
