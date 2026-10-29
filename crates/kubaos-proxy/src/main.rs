//! `kubaos-proxy`: the Proxy from a terminal, the control center and the Command menu, and its
//! session service.
//!
//! ```text
//! kubaos-proxy on                     every part of the profile up, or none
//! kubaos-proxy off                    every part down
//! kubaos-proxy lan on|off             whether the kill switch lets the local networks through
//! kubaos-proxy status [--json] [--live]
//!                                     the parts and which are up; --live adds the VPN's last
//!                                     handshake and bytes moved
//! kubaos-proxy import FILE            IT's VPN config, WireGuard or OpenVPN: its secrets to the
//!                                     vault and the keyring, the rest beside proxy.toml
//! kubaos-proxy pull                   the secrets from the vault to the keyring again
//! kubaos-proxy serve                  the session service: at its start, after a resume and when
//!                                     the network changes, makes the machine what the user wants
//! ```
//!
//! # Owns
//!
//! - The command line and its exit codes: 2 for a usage error, 1 when the command failed, with the
//!   reason on stderr. `on` and `off` say a failure in a notification too, since the shell and the
//!   menu run them with no terminal.
//! - The service's loop: logind's resume and NetworkManager's connections on the system bus, the
//!   lock, and its log.
//!
//! # Not here
//!
//! - Everything else: the library's modules.

use std::fs;
use std::fs::File;
use std::io;
use std::io::IsTerminal as _;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use kubaos_core::bitwarden::Bw as _;
use kubaos_core::bitwarden::Session;
use kubaos_proxy::Paths;
use kubaos_proxy::Plan;
use kubaos_proxy::Profile;
use kubaos_proxy::apply;
use kubaos_proxy::apply::Reconciled;
use kubaos_proxy::apply::chain;
use kubaos_proxy::openvpn;
use kubaos_proxy::profile;
use kubaos_proxy::profile::LoadError;
use kubaos_proxy::real::Real;
use kubaos_proxy::secrets;
use kubaos_proxy::secrets::Keys;
use kubaos_proxy::secrets::Secret;
use kubaos_proxy::secrets::Secrets;
use kubaos_proxy::status;
use kubaos_proxy::status::Status;
use kubaos_proxy::vpn;
use kubaos_proxy::vpn::Type;
use kubaos_proxy::watch;
use kubaos_proxy::watch::Event;
use kubaos_proxy::watch::Response;

const USAGE: &str = "\
usage: kubaos-proxy on|off
       kubaos-proxy lan on|off
       kubaos-proxy status [--json] [--live]
       kubaos-proxy import FILE
       kubaos-proxy pull
       kubaos-proxy serve

on       the work network: every part of ~/.config/kubaos/desktop/network/proxy.toml up, or none
off      every part down again
lan      whether the kill switch lets the network you're on through (on by default): the phone
         link, printers, casting and SSH there; a range the VPN routes stays behind it
status   the parts and which are up; --live adds the VPN's last handshake and bytes moved
import   IT's VPN config, a WireGuard .conf or an OpenVPN .ovpn, its type read from the file: its
         secrets go to the vault and the keyring, the rest beside proxy.toml; a username and
         password it asks for are asked once, here
pull     the VPN's secrets from the vault to the keyring again (a new machine, a new keyring)
serve    the session service (home/proxy.nix)";

const ICON: &str = "network-vpn";

/// How long NetworkManager gets to wake after a resume before the tunnel goes up again.
const WAKE_TIMEOUT: Duration = Duration::from_mins(1);
/// `NM_STATE_CONNECTED_LOCAL`: NetworkManager is awake and has a network.
const NM_CONNECTED: u32 = 50;
/// How long a burst of events gets to settle before the service acts once on it.
const SETTLE: Duration = Duration::from_secs(2);
const NM: &str = "org.freedesktop.NetworkManager";
const NM_PATH: &str = "/org/freedesktop/NetworkManager";

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The help, the usage and the missing home are the command's output, for whoever ran it."
)]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let Some(paths) = Paths::from_env() else {
        eprintln!("kubaos-proxy: neither HOME nor XDG_CONFIG_HOME is set");
        return ExitCode::FAILURE;
    };
    match args.as_slice() {
        ["on"] => switch(&paths, true),
        ["off"] => switch(&paths, false),
        ["lan", setting @ ("on" | "off")] => local_network(&paths, *setting == "on"),
        ["status", flags @ ..] if flags.iter().all(|f| matches!(*f, "--json" | "--live")) => {
            show(&paths, flags.contains(&"--json"), flags.contains(&"--live"))
        }
        ["import", file] => import(&paths, &typed_path(file)),
        ["pull"] => pull(&paths),
        ["serve"] => {
            if log::set_logger(&Journal).is_ok() {
                log::set_max_level(log::LevelFilter::Info);
            }
            serve(&paths)
        }
        ["-h" | "--help"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Holds the session's lock, so the service and a command never change the Proxy at once.
fn lock(paths: &Paths) -> io::Result<File> {
    fs::create_dir_all(paths.session())?;
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(paths.lock())?;
    file.lock()?;
    Ok(file)
}

#[expect(
    clippy::print_stderr,
    reason = "The commands and the service both publish; the line goes to stderr, which the terminal or the journal shows, as the service's log lines do."
)]
fn publish(m: &mut Real, paths: &Paths, profile: &Result<Profile, LoadError>, busy: bool, problem: Option<String>) {
    let mut now = status::read(m, paths, profile, false);
    now.busy = busy;
    if problem.is_some() {
        now.problem = problem;
    }
    if let Err(err) = status::publish(paths, &now) {
        eprintln!("kubaos-proxy: couldn't write {}: {err}", paths.shell().display());
    }
}

/// `on` or `off`.
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The summary, its notes and the lock's refusal are the command's answer to whoever ran it."
)]
fn switch(paths: &Paths, on: bool) -> ExitCode {
    let _lock = match lock(paths) {
        Ok(lock) => lock,
        Err(err) => {
            eprintln!("kubaos-proxy: couldn't take {}: {err}", paths.lock().display());
            return ExitCode::FAILURE;
        }
    };
    let mut m = Real::new();
    let profile = Profile::load(&paths.network);
    publish(&mut m, paths, &profile, true, None);
    let user = apply::user();
    let plan = profile.as_ref().ok().map(Plan::for_profile);
    let done = if on {
        let plan = match (&profile, &plan) {
            (Ok(_), Some(plan)) if !plan.is_empty() => plan,
            (Ok(_), _) => {
                return fail(
                    &mut m,
                    paths,
                    &profile,
                    "Proxy not set up",
                    "Nothing to switch on: import IT's VPN config (kubaos proxy import FILE) or edit \
                     ~/.config/kubaos/desktop/network/proxy.toml"
                        .to_owned(),
                );
            }
            (Err(err), _) => return fail(&mut m, paths, &profile, "Proxy settings", chain(err)),
        };
        if let Err(err) = apply::set_wanted(paths, true) {
            return fail(&mut m, paths, &profile, "Proxy", err.to_string());
        }
        let done = apply::on(&mut m, paths, plan, user.as_deref());
        if done.is_err() {
            let _ = apply::set_wanted(paths, false);
        }
        done
    } else {
        let done = apply::off(&mut m, paths, plan.as_ref(), user.as_deref());
        if done.is_ok() {
            let _ = apply::set_wanted(paths, false);
        }
        done
    };
    match done {
        Ok(()) => {
            publish(&mut m, paths, &profile, false, None);
            let now = status::read(&mut m, paths, &profile, false);
            println!("{}", summary(&now));
            for warning in &now.warnings {
                eprintln!("kubaos-proxy: note: {warning}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            let mut body = chain(&err);
            if err.left.is_empty() {
                body.push_str(if on {
                    ". The Proxy is off."
                } else {
                    ". The Proxy is still on."
                });
            } else {
                body.push_str(". ");
                body.push_str(&err.left.join(". "));
            }
            let title = if on {
                "Proxy couldn't go on"
            } else {
                "Proxy couldn't go off"
            };
            fail(&mut m, paths, &profile, title, body)
        }
    }
}

/// `lan on|off`: writes the setting into the profile and, while the kill switch is up, puts it up
/// again with the setting.
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "What the kill switch lets through now, or the lock's refusal, is the command's answer to whoever ran it."
)]
fn local_network(paths: &Paths, allowed: bool) -> ExitCode {
    let _lock = match lock(paths) {
        Ok(lock) => lock,
        Err(err) => {
            eprintln!("kubaos-proxy: couldn't take {}: {err}", paths.lock().display());
            return ExitCode::FAILURE;
        }
    };
    let mut m = Real::new();
    if let Err(err) = profile::set_local_network(&paths.profile(), allowed) {
        let profile = Profile::load(&paths.network);
        return fail(&mut m, paths, &profile, "Proxy settings", chain(&err));
    }
    let profile = Profile::load(&paths.network);
    let plan = match &profile {
        Ok(profile) => Plan::for_profile(profile),
        Err(err) => {
            let body = chain(err);
            return fail(&mut m, paths, &profile, "Proxy settings", body);
        }
    };
    if let Err(err) = apply::kill_switch_again(&mut m, paths, &plan) {
        let body = format!("{}. The kill switch lets through what it did before.", chain(&err));
        return fail(&mut m, paths, &profile, "Proxy's local network", body);
    }
    publish(&mut m, paths, &profile, false, None);
    println!(
        "{}",
        if allowed {
            "The kill switch lets the local network through."
        } else {
            "The kill switch blocks the local network."
        }
    );
    if !plan.has_kill_switch() {
        eprintln!("kubaos-proxy: note: the setting is for a full tunnel's kill switch, and this profile has none");
    }
    ExitCode::SUCCESS
}

#[expect(
    clippy::print_stderr,
    reason = "The failure is the command's answer to whoever ran it, beside its notification."
)]
fn fail(m: &mut Real, paths: &Paths, profile: &Result<Profile, LoadError>, title: &str, body: String) -> ExitCode {
    eprintln!("kubaos-proxy: {body}");
    notify(title, &body);
    publish(m, paths, profile, false, Some(body));
    ExitCode::FAILURE
}

/// One line on what is up.
fn summary(now: &Status) -> String {
    if !now.on {
        return "Proxy off.".to_owned();
    }
    let mut parts = Vec::new();
    if let Some(tunnel) = &now.tunnel {
        let kind = if tunnel.full { "full tunnel" } else { "split tunnel" };
        let to = tunnel
            .endpoint
            .as_deref()
            .map(|e| format!(" to {e}"))
            .unwrap_or_default();
        let guard = if tunnel.kill_switch { ", kill switch on" } else { "" };
        parts.push(format!("{} VPN, {kind}{to}{guard}", tunnel.type_name));
    }
    if !now.dns_servers.is_empty() {
        parts.push("work DNS".to_owned());
    }
    if let Some(proxy) = &now.proxy {
        parts.push(format!("proxy {proxy}"));
    } else if now.pac.is_some() {
        parts.push("PAC file".to_owned());
    }
    if now.ca.is_some() {
        parts.push("company CA".to_owned());
    }
    format!("Proxy on: {}.", parts.join(", "))
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The status is the command's output, for the person or the shell that asked."
)]
fn show(paths: &Paths, json: bool, live: bool) -> ExitCode {
    let mut m = Real::new();
    let profile = Profile::load(&paths.network);
    let now = status::read(&mut m, paths, &profile, live);
    if json {
        match serde_json::to_string_pretty(&now) {
            Ok(text) => println!("{text}"),
            Err(err) => {
                eprintln!("kubaos-proxy: {err}");
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
    }
    for row in status::rows(&now, status::now()) {
        println!("{}: {}", row.label, row.value);
    }
    ExitCode::SUCCESS
}

/// The vault's session: `BW_SESSION` when exported, else bw's own prompt for the master
/// password, on a terminal.
#[expect(
    clippy::print_stderr,
    reason = "It tells the person at the terminal why bw asks for the master password."
)]
fn session() -> Result<Session, String> {
    if let Ok(session) = std::env::var("BW_SESSION")
        && !session.trim().is_empty()
    {
        return Ok(Session::new(session.trim()));
    }
    let mut bw = kubaos_core::bitwarden::Real::new();
    if !bw.installed() {
        return Err(chain(&secrets::Error::NoBw));
    }
    if !io::stdin().is_terminal() {
        return Err("the vault is locked and there's no terminal to unlock it: run this in one".to_owned());
    }
    eprintln!("kubaos-proxy: unlocking the vault for the VPN's secrets");
    bw.unlock(None)
        .map_err(|err| format!("bw couldn't unlock the vault ({err}): kubaos bitwarden setup logs in"))
}

fn vault_item(paths: &Paths) -> String {
    Profile::load(&paths.network).map_or_else(|_| profile::DEFAULT_VAULT_ITEM.to_owned(), |p| p.vault_item)
}

/// A path as typed into the menu, which runs no shell: `~/` is the home.
fn typed_path(typed: &str) -> PathBuf {
    match (typed.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => PathBuf::from(typed),
    }
}

/// Asks the person at the terminal for `what`, its typing hidden when `hidden`.
#[expect(
    clippy::print_stderr,
    reason = "The question is for the person at the terminal, on stderr beside bw's own prompt."
)]
fn ask(what: &str, hidden: bool) -> io::Result<String> {
    let echo = |on: bool| {
        Command::new("stty")
            .arg(if on { "echo" } else { "-echo" })
            .stdin(Stdio::inherit())
            .status()
            .map(drop)
    };
    eprint!("{what}: ");
    io::stderr().flush()?;
    if hidden {
        echo(false)?;
    }
    let mut line = String::new();
    let read = io::stdin().read_line(&mut line);
    if hidden {
        let _ = echo(true);
        eprintln!();
    }
    read?;
    Ok(line.trim_end_matches(['\n', '\r']).to_owned())
}

/// The secrets of an imported config, what an OpenVPN one lacks asked for at the terminal: the
/// username and password `auth-user-pass` wants, and an encrypted key's passphrase.
fn secrets_of(config: &vpn::Config, file: &Path) -> Result<Secrets, String> {
    let config = match config {
        vpn::Config::WireGuard(config) => {
            return Keys::of(config).map(Secrets::WireGuard).ok_or_else(|| {
                format!(
                    "{} has no PrivateKey: import the config IT gave you, with its keys",
                    file.display()
                )
            });
        }
        vpn::Config::OpenVpn(config) => config,
    };
    let mut credentials = config.credentials.clone();
    let mut questions = Vec::new();
    if config.auth_user_pass {
        if credentials.username.is_none() {
            questions.push(("username", "The VPN's username", false));
        }
        if credentials.password.is_none() {
            questions.push(("password", "The VPN's password", true));
        }
    }
    if credentials
        .key
        .as_ref()
        .is_some_and(|key| openvpn::is_encrypted(key.expose()))
        && credentials.cert_pass.is_none()
    {
        questions.push(("cert-pass", "The private key's passphrase", true));
    }
    if !questions.is_empty() && !io::stdin().is_terminal() {
        return Err(format!(
            "{} asks for a password, and there's no terminal to ask in: run this in one",
            file.display()
        ));
    }
    for (name, question, hidden) in questions {
        let answer = ask(question, hidden).map_err(|err| format!("couldn't read the answer: {err}"))?;
        if !answer.is_empty() {
            credentials.set(name, Secret::new(answer));
        }
    }
    let missing = credentials.missing(config);
    if !missing.is_empty() {
        return Err(format!(
            "{} has no {}: import the config IT gave you, with its certificate and key or a login",
            file.display(),
            missing.join(", ")
        ));
    }
    Ok(Secrets::OpenVpn(credentials))
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "What was imported, or why not, is the command's answer to whoever ran it."
)]
fn import(paths: &Paths, file: &Path) -> ExitCode {
    let config = match profile::read_conf(file) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("kubaos-proxy: {}", chain(&err));
            return ExitCode::FAILURE;
        }
    };
    let secrets = match secrets_of(&config, file) {
        Ok(secrets) => secrets,
        Err(err) => {
            eprintln!("kubaos-proxy: {err}");
            return ExitCode::FAILURE;
        }
    };
    let session = match session() {
        Ok(session) => session,
        Err(err) => {
            eprintln!("kubaos-proxy: {err}");
            return ExitCode::FAILURE;
        }
    };
    let mut m = Real::new();
    let item = vault_item(paths);
    if let Err(err) = secrets::to_vault(&mut m, &session, &item, &secrets) {
        eprintln!("kubaos-proxy: {}", chain(&err));
        return ExitCode::FAILURE;
    }
    println!("The VPN's secrets are in the vault, in {item:?}.");
    if let Err(err) = secrets::store(&mut m, &secrets) {
        eprintln!("kubaos-proxy: {}; kubaos proxy pull tries again", chain(&err));
        return ExitCode::FAILURE;
    }
    let kind = config.kind();
    let conf = paths.network.join(kind.file_name());
    // The file's name goes in a comment line: a newline in it would start a line of config.
    let from: String = file
        .file_name()
        .map(|name| name.to_string_lossy().chars().filter(|c| !c.is_control()).collect())
        .unwrap_or_default();
    let text = format!(
        "# IT's {kind} config without its secrets, from {from} (kubaos-proxy import).\n\
         # The secrets are in the vault, in {item:?}, and the GNOME keyring.\n{}",
        config.sanitized().render()
    );
    if let Err(err) = profile::write_atomic(&conf, &text, 0o644) {
        eprintln!("kubaos-proxy: couldn't write {}: {err}", conf.display());
        return ExitCode::FAILURE;
    }
    let before = profile::conf_named(&paths.profile());
    if let Err(err) = profile::set_conf(&paths.profile(), kind, kind.file_name()) {
        eprintln!("kubaos-proxy: {}", chain(&err));
        return ExitCode::FAILURE;
    }
    // The other type's file, an import's own, is the config this one replaces.
    if let Some(before) = before
        && before != kind.file_name()
        && Type::ALL.iter().any(|other| other.file_name() == before)
    {
        let _ = fs::remove_file(paths.network.join(before));
    }
    report_import(&config, &conf);
    publish(&mut m, paths, &Profile::load(&paths.network), false, None);
    ExitCode::SUCCESS
}

#[expect(
    clippy::print_stdout,
    reason = "The report of the imported VPN is the command's output."
)]
fn report_import(config: &vpn::Config, conf: &Path) {
    match config {
        vpn::Config::WireGuard(config) => {
            let ranges: Vec<String> = config
                .peers
                .iter()
                .flat_map(|peer| peer.allowed_ips.iter().map(ToString::to_string))
                .collect();
            if config.is_full() {
                println!(
                    "A WireGuard VPN, a full tunnel (AllowedIPs {}): everything goes through it, and the kill \
                     switch blocks what would leave outside it.",
                    ranges.join(", ")
                );
            } else {
                println!(
                    "A WireGuard VPN, a split tunnel: only {} go through it.",
                    ranges.join(", ")
                );
            }
        }
        vpn::Config::OpenVpn(config) => {
            if config.is_full() {
                println!(
                    "An OpenVPN VPN, a full tunnel (redirect-gateway): everything goes through it, and the kill \
                     switch blocks what would leave outside it."
                );
            } else {
                let own: Vec<String> = config.routes.iter().map(ToString::to_string).collect();
                let also = if own.is_empty() {
                    String::new()
                } else {
                    format!(", and {},", own.join(", "))
                };
                println!(
                    "An OpenVPN VPN, a split tunnel: only the ranges the server pushes{also} go through it. Add \
                     redirect-gateway def1 to {} for a full one.",
                    conf.display()
                );
            }
        }
    }
    if let Some(endpoint) = config.endpoint() {
        println!("Server: {endpoint}.");
    }
    if !config.dns_servers().is_empty() {
        let servers: Vec<String> = config.dns_servers().iter().map(ToString::to_string).collect();
        println!("Work DNS: {}.", servers.join(", "));
    }
    if !config.ignored().is_empty() {
        let why = match config.kind() {
            Type::WireGuard => "since KubaOS runs no wg-quick scripts",
            Type::OpenVpn => "since NetworkManager's OpenVPN plugin runs it",
        };
        println!("Left out, {why}: {}.", config.ignored().join(", "));
    }
    println!(
        "Written to {} and named in proxy.toml. The Proxy tile in the control center, or kubaos proxy on, \
         brings it up.",
        conf.display()
    );
}

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "The outcome is the command's answer to whoever ran it."
)]
fn pull(paths: &Paths) -> ExitCode {
    let session = match session() {
        Ok(session) => session,
        Err(err) => {
            eprintln!("kubaos-proxy: {err}");
            return ExitCode::FAILURE;
        }
    };
    let mut m = Real::new();
    let item = vault_item(paths);
    let done = secrets::from_vault(&mut m, &session, &item).and_then(|secrets| secrets::store(&mut m, &secrets));
    match done {
        Ok(()) => {
            println!("The VPN's secrets are in the keyring again, from {item:?} in the vault.");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("kubaos-proxy: {}", chain(&err));
            ExitCode::FAILURE
        }
    }
}

/// Makes the machine what the user wants, under the lock, and says so when it had to act.
/// Whether a reconnect failed.
fn reconcile(m: &mut Real, paths: &Paths, why: &str) -> bool {
    let _lock = match lock(paths) {
        Ok(lock) => lock,
        Err(err) => {
            log::error!("couldn't take {}: {err}", paths.lock().display());
            return false;
        }
    };
    let mut problem = None;
    let mut failed = false;
    match apply::reconcile(m, paths, Profile::load(&paths.network), apply::user().as_deref()) {
        Reconciled::Nothing => {}
        Reconciled::TookDown(left) if left.is_empty() => log::info!("{why}: took away what was left of the Proxy"),
        Reconciled::TookDown(left) => {
            let body = format!("Parts of the Proxy stayed up: {}", left.join(". "));
            log::warn!("{why}: {body}");
            notify("Proxy", &body);
        }
        Reconciled::BroughtUp => log::info!("{why}: the Proxy is up again"),
        Reconciled::KillSwitchAgain(kill_switch) => {
            log::info!("{why}: the kill switch went up again, {kill_switch:?}");
        }
        Reconciled::KillSwitchKept(err) => {
            log::warn!(
                "{why}: the kill switch couldn't let the local networks as they are now through, and lets through \
                 what it did before: {}",
                chain(&err)
            );
        }
        Reconciled::Failed {
            cause,
            kill_switch_kept,
        } => {
            let body = if kill_switch_kept {
                format!(
                    "{}. The kill switch stays up, so nothing leaves outside the tunnel; it tries again when the \
                     network changes, and \"Proxy off\" lifts it.",
                    chain(&cause)
                )
            } else {
                format!(
                    "{}. Everything is off; turn it on again once the network is back.",
                    chain(&cause)
                )
            };
            log::error!("{why}: {body}");
            notify(
                if kill_switch_kept {
                    "Proxy is down"
                } else {
                    "Proxy went off"
                },
                &body,
            );
            problem = Some(body);
            failed = true;
        }
    }
    publish(m, paths, &Profile::load(&paths.network), false, problem);
    failed
}

/// Waits until NetworkManager is awake with a network, at most [`WAKE_TIMEOUT`].
fn wait_for_network(bus: &zbus::blocking::Connection) {
    let Ok(nm) = zbus::blocking::Proxy::new(bus, NM, NM_PATH, NM) else {
        return;
    };
    let start = Instant::now();
    while start.elapsed() < WAKE_TIMEOUT {
        if nm.get_property::<u32>("State").is_ok_and(|state| state >= NM_CONNECTED) {
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }
}

/// Sends an event each time the machine wakes.
fn follow_sleep(bus: &zbus::blocking::Connection, events: &Sender<Event>) -> zbus::Result<()> {
    let login = zbus::blocking::Proxy::new(
        bus,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )?;
    for signal in login.receive_signal("PrepareForSleep")? {
        let sleeping: bool = signal.body().deserialize()?;
        if !sleeping && events.send(Event::Resumed).is_err() {
            break;
        }
    }
    Ok(())
}

/// Sends an event each time NetworkManager's active connections change: the tunnel among them.
fn follow_network(bus: &zbus::blocking::Connection, events: &Sender<Event>) -> zbus::Result<()> {
    let nm = zbus::blocking::Proxy::new(bus, NM, NM_PATH, NM)?;
    for _ in nm.receive_property_changed::<Vec<zbus::zvariant::OwnedObjectPath>>("ActiveConnections") {
        if events.send(Event::NetworkChanged).is_err() {
            break;
        }
    }
    Ok(())
}

/// Sends an event each time NetworkManager's state changes. A network joined is an active
/// connection before it has an address; the state changes once it has one, so the kill switch
/// lets its range through then.
fn follow_state(bus: &zbus::blocking::Connection, events: &Sender<Event>) -> zbus::Result<()> {
    let nm = zbus::blocking::Proxy::new(bus, NM, NM_PATH, NM)?;
    for _ in nm.receive_property_changed::<u32>("State") {
        if events.send(Event::NetworkChanged).is_err() {
            break;
        }
    }
    Ok(())
}

fn serve(paths: &Paths) -> ExitCode {
    let mut m = Real::new();
    reconcile(&mut m, paths, "at the start");
    let bus = match zbus::blocking::Connection::system() {
        Ok(bus) => bus,
        Err(err) => {
            log::error!("the system bus: {err}");
            return ExitCode::FAILURE;
        }
    };
    let (sender, events) = mpsc::channel();
    for (name, follow) in [
        (
            "logind",
            follow_sleep as fn(&zbus::blocking::Connection, &Sender<Event>) -> zbus::Result<()>,
        ),
        ("NetworkManager", follow_network),
        ("NetworkManager's state", follow_state),
    ] {
        let (bus, sender) = (bus.clone(), sender.clone());
        thread::spawn(move || {
            if let Err(err) = follow(&bus, &sender) {
                log::error!("following {name} on the system bus: {err}");
            }
        });
    }
    drop(sender);
    let mut failed_at: Option<Instant> = None;
    while let Ok(event) = events.recv() {
        let event = settle(event, &events);
        match watch::respond(event, apply::wanted(paths), failed_at.map(|at| at.elapsed())) {
            Response::Publish => publish(&mut m, paths, &Profile::load(&paths.network), false, None),
            Response::Reconcile => {
                if event == Event::Resumed {
                    wait_for_network(&bus);
                }
                let why = if event == Event::Resumed {
                    "after a resume"
                } else {
                    "the network changed"
                };
                let failed = reconcile(&mut m, paths, why);
                failed_at = failed.then(Instant::now);
            }
        }
    }
    log::error!("logind and NetworkManager both went from the system bus");
    ExitCode::FAILURE
}

/// The event and those that follow it within [`SETTLE`], as one: a resume outweighs a change of
/// network. NetworkManager changes its connections several times in a row as a network comes.
fn settle(first: Event, events: &mpsc::Receiver<Event>) -> Event {
    let mut event = first;
    while let Ok(next) = events.recv_timeout(SETTLE) {
        if next == Event::Resumed {
            event = Event::Resumed;
        }
    }
    event
}

/// A notification through `notify-send` (libnotify-bin, `system/packages.toml`). Without it the
/// notification only goes unseen; the reason is in the log or on stderr too.
fn notify(summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["--app-name=Proxy", &format!("--icon={ICON}"), summary, body])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// The service's log: a line on stderr each, with the priority prefix the journal reads
/// (`sd-daemon(3)`).
struct Journal;

impl log::Log for Journal {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let priority = match record.level() {
            log::Level::Error => 3,
            log::Level::Warn => 4,
            log::Level::Info => 6,
            log::Level::Debug | log::Level::Trace => 7,
        };
        let _ = writeln!(io::stderr(), "<{priority}>{}", record.args());
    }

    fn flush(&self) {}
}
