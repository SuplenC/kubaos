//! Putting a plan in place and taking it away, every part or none.
//!
//! `on` puts each part up in order; when one fails, it takes that one and every part before it
//! away again, so the Proxy is either all on or all off. `off` takes the parts down in the
//! reverse order; when one can't go, it puts the ones it took down back. A part is written to the
//! applied file ([`Paths::applied`]) before it goes up, in the state directory: a login after a
//! crash or a reboot finds it and takes it away ([`reconcile`]), since nothing switches on by
//! itself. Taking away a part that never came up is harmless; a part up that nothing records
//! would stay forever.
//!
//! Every part's removal takes away whatever KubaOS put there, whatever the profile says now, so
//! a profile edited while the Proxy is on still comes off whole, and puts back what the user had
//! before (GNOME's proxy, Docker's, the session's variables).
//!
//! # Owns
//!
//! - [`on`], [`off`], [`reconcile`] and the applied file.
//! - Putting the kill switch up again alone when the local networks it lets through change
//!   ([`kill_switch_again`]), and the record of what it went up with.
//!
//! # Not here
//!
//! - Which parts a profile has: [`crate::plan`]. Each part on the machine: `apply/session.rs` (the
//!   session's), `apply/root.rs` (the root helper's), `apply/endpoints.rs` (the tunnel's endpoint
//!   addresses). The root side: `system/files/proxy/kubaos-proxy-write`.

mod endpoints;
mod root;
mod session;

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub use self::root::handshake;
use crate::local_network;
use crate::machine::Cmd;
use crate::machine::Machine;
use crate::machine::Ran;
use crate::machine::ResolveError;
use crate::machine::Said;
use crate::machine::TunnelError;
use crate::paths::HELPER;
use crate::paths::Paths;
use crate::plan::KillSwitch;
use crate::plan::Kind;
use crate::plan::Part;
use crate::plan::Plan;
use crate::plan::RootArg;
use crate::profile::LoadError;
use crate::profile::Profile;
use crate::profile::write_atomic;
use crate::secrets;
use crate::tunnel;
use crate::wireguard::Cidr;

/// Why one part couldn't be put in place or taken away.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StepError {
    /// The cause's message alone: [`Failed`] has named the tunnel already.
    #[error(transparent)]
    Keys(secrets::Error),
    /// The cause's message alone, as with `Keys`.
    #[error(transparent)]
    Tunnel(TunnelError),
    #[error("the tunnel's endpoint, with no address known for it")]
    Endpoint(#[source] ResolveError),
    #[error("the root helper, {HELPER}")]
    Root(#[source] Said),
    /// pkexec never ran the helper: polkit refused, the prompt was dismissed, or the helper isn't
    /// installed. Nothing was changed as root.
    #[error("pkexec didn't run {HELPER}: not allowed, or not installed (kubaos update installs it)")]
    NotRun,
    /// The helper changed nothing: what it was asked isn't what an admin pinned.
    #[error("the proxy, its CA or its PAC file isn't what an admin pinned")]
    NotPinned,
    #[error("{program}")]
    Program {
        program: String,
        #[source]
        said: Said,
    },
    #[error("{}", .0.display())]
    File(PathBuf, #[source] io::Error),
    #[error("{} isn't a JSON object: fix or remove it", .0.display())]
    NotJson(PathBuf),
}

/// Why `on` or `off` didn't go through, and what undoing the rest left behind.
#[derive(Debug)]
pub struct Failed {
    pub kind: Kind,
    pub going_up: bool,
    pub cause: StepError,
    /// What couldn't be undone, a sentence each; empty when the Proxy is wholly on or off again.
    pub left: Vec<String>,
}

impl std::fmt::Display for Failed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what = if self.going_up { "put in place" } else { "taken away" };
        write!(f, "{} couldn't be {what}", capital(self.kind.title()))
    }
}

impl std::error::Error for Failed {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

fn capital(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// The error and its causes, each once.
pub fn chain(err: &dyn std::error::Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

/// The parts up now, in the order they went up.
pub fn applied(paths: &Paths) -> Vec<Kind> {
    fs::read_to_string(paths.applied())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| Kind::from_name(line.trim()))
        .collect()
}

fn record(paths: &Paths, kinds: &[Kind]) -> Result<(), StepError> {
    let path = paths.applied();
    if kinds.is_empty() {
        return remove_file(&path);
    }
    let text: String = kinds.iter().flat_map(|kind| [kind.name(), "\n"]).collect();
    write_atomic(&path, &text, 0o644).map_err(|err| StepError::File(path, err))
}

fn remove_file(path: &Path) -> Result<(), StepError> {
    match fs::remove_file(path) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(StepError::File(path.to_path_buf(), err)),
        _ => Ok(()),
    }
}

/// Whether the user wants the Proxy on in this session.
pub fn wanted(paths: &Paths) -> bool {
    paths.wanted().exists()
}

/// Sets whether the user wants the Proxy on.
///
/// # Errors
///
/// The session's folder can't be written.
pub fn set_wanted(paths: &Paths, on: bool) -> io::Result<()> {
    if on {
        write_atomic(&paths.wanted(), "on\n", 0o644)
    } else {
        match fs::remove_file(paths.wanted()) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        }
    }
}

/// Who runs this, for the tunnel's owner.
pub fn user() -> Option<String> {
    ["USER", "LOGNAME"]
        .iter()
        .find_map(|var| std::env::var(var).ok())
        .filter(|name| !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)))
}

/// Whether undoing keeps the kill switch: a tunnel that went down while the user wants it up
/// leaves nothing to go out in the clear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Keep {
    Nothing,
    KillSwitch,
}

/// Puts every part of `plan` in place, the ones already up again, and takes away a part that is
/// up but no longer in the plan. When one fails, every part is taken away.
///
/// # Errors
///
/// The part that failed and why; [`Failed::left`] says what couldn't be undone.
pub fn on(m: &mut dyn Machine, paths: &Paths, plan: &Plan, user: Option<&str>) -> Result<(), Failed> {
    on_keeping(m, paths, plan, user, Keep::Nothing)
}

fn on_keeping(m: &mut dyn Machine, paths: &Paths, plan: &Plan, user: Option<&str>, keep: Keep) -> Result<(), Failed> {
    let plan = &with_local_networks(m, plan);
    let mut up = applied(paths);
    let guarded = up.contains(&Kind::Root) && plan.has_kill_switch();
    let wanted: Vec<Kind> = plan.kinds();
    let fail = |m: &mut dyn Machine, kind, going_up, cause| {
        let left = undo_keeping(m, paths, keep);
        Err(Failed {
            kind,
            going_up,
            cause,
            left,
        })
    };
    for kind in up.clone().into_iter().rev().filter(|kind| !wanted.contains(kind)) {
        if let Err(cause) = remove(m, paths, kind, Keep::Nothing) {
            return fail(m, kind, false, cause);
        }
        up.retain(|k| *k != kind);
        let _ = record(paths, &up);
    }
    for part in &plan.parts {
        let kind = part.kind();
        let fresh = !up.contains(&kind);
        if fresh {
            up.push(kind);
        }
        if let Err(cause) = record(paths, &up) {
            return fail(m, kind, true, cause);
        }
        if let Err(cause) = put(m, paths, part, user, guarded) {
            if fresh && matches!(cause, StepError::NotRun) {
                up.retain(|k| *k != kind);
                let _ = record(paths, &up);
            }
            return fail(m, kind, true, cause);
        }
    }
    Ok(())
}

/// Takes every part away, in the reverse order they went up. When one can't go and `plan` is
/// there, the parts taken down go back up, so the Proxy stays wholly on.
///
/// # Errors
///
/// The part that couldn't go and why; [`Failed::left`] says what couldn't be put back.
pub fn off(m: &mut dyn Machine, paths: &Paths, plan: Option<&Plan>, user: Option<&str>) -> Result<(), Failed> {
    let mut up = applied(paths);
    if !up.contains(&Kind::Tunnel) && m.tunnel_active().unwrap_or(false) {
        up.insert(0, Kind::Tunnel);
    }
    let mut taken: Vec<Kind> = Vec::new();
    for kind in up.clone().into_iter().rev() {
        if let Err(cause) = remove(m, paths, kind, Keep::Nothing) {
            let left = match plan {
                Some(plan) => put_back(m, paths, plan, &taken, user),
                None => vec![format!("{} stayed in place", capital(kind.title()))],
            };
            return Err(Failed {
                kind,
                going_up: false,
                cause,
                left,
            });
        }
        taken.push(kind);
        up.retain(|k| *k != kind);
        let _ = record(paths, &up);
    }
    Ok(())
}

/// Takes away whatever is up, past any part that fails: for a login, and after a failure. What
/// couldn't go, a sentence each.
pub fn undo_all(m: &mut dyn Machine, paths: &Paths) -> Vec<String> {
    undo_keeping(m, paths, Keep::Nothing)
}

fn undo_keeping(m: &mut dyn Machine, paths: &Paths, keep: Keep) -> Vec<String> {
    let mut up = applied(paths);
    if !up.contains(&Kind::Tunnel) && m.tunnel_active().unwrap_or(false) {
        up.insert(0, Kind::Tunnel);
    }
    let mut left = Vec::new();
    let mut kept = Vec::new();
    for kind in up.into_iter().rev() {
        match remove(m, paths, kind, keep) {
            Ok(()) if kind == Kind::Root && keep == Keep::KillSwitch => kept.insert(0, kind),
            Ok(()) => {}
            Err(err) => {
                left.push(format!("{} stayed in place: {}", capital(kind.title()), chain(&err)));
                kept.insert(0, kind);
            }
        }
    }
    let _ = record(paths, &kept);
    left
}

fn put_back(m: &mut dyn Machine, paths: &Paths, plan: &Plan, taken: &[Kind], user: Option<&str>) -> Vec<String> {
    let plan = &with_local_networks(m, plan);
    let mut up = applied(paths);
    let mut left = Vec::new();
    for part in plan.parts.iter().filter(|part| taken.contains(&part.kind())) {
        if !up.contains(&part.kind()) {
            up.push(part.kind());
            let _ = record(paths, &up);
        }
        if let Err(err) = put(m, paths, part, user, false) {
            left.push(format!(
                "{} couldn't go back up: {}",
                capital(part.kind().title()),
                chain(&err)
            ));
        }
    }
    let order = |kind: &Kind| Kind::ALL.iter().position(|k| k == kind);
    up.sort_by_key(order);
    let _ = record(paths, &up);
    left
}

/// Why [`reconcile`] couldn't bring the Proxy up.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReconcileError {
    #[error("the profile can't be used")]
    Profile(#[source] LoadError),
    #[error("the Proxy couldn't come back")]
    On(#[source] Failed),
}

/// The outcome of [`reconcile`].
#[derive(Debug)]
pub enum Reconciled {
    /// The machine already was as the user wants it.
    Nothing,
    /// The Proxy is off, as wanted, and what was left up is gone; anything that stayed, a
    /// sentence each.
    TookDown(Vec<String>),
    /// The Proxy is on again, as wanted.
    BroughtUp,
    /// Every part was up, and the kill switch went up again to let through what it now lets
    /// through: the local networks changed.
    KillSwitchAgain(KillSwitch),
    /// Every part is up, but the kill switch couldn't go up again for the local networks as they
    /// are now, so it lets through what it did before.
    KillSwitchKept(StepError),
    /// The Proxy was wanted on and couldn't be. Every part is off but, when it was up, the kill
    /// switch: then the Proxy stays wanted, so the next change of network tries again, and "Proxy
    /// off" lifts it. Without it, the Proxy is no longer wanted.
    Failed {
        cause: ReconcileError,
        kill_switch_kept: bool,
    },
}

/// Makes the machine what the user wants: at the service's start, after a resume, when the
/// network changes. Wanted off (and a login starts so), whatever is up goes. Wanted on, a part
/// that isn't up, or a tunnel NetworkManager dropped, goes up again with the rest.
pub fn reconcile(
    m: &mut dyn Machine,
    paths: &Paths,
    profile: Result<Profile, LoadError>,
    user: Option<&str>,
) -> Reconciled {
    let up = applied(paths);
    let tunnel_up = m.tunnel_active().unwrap_or(false);
    if !wanted(paths) {
        if up.is_empty() && !tunnel_up {
            return Reconciled::Nothing;
        }
        return Reconciled::TookDown(undo_all(m, paths));
    }
    let guarded = up.contains(&Kind::Root);
    let keep = if guarded { Keep::KillSwitch } else { Keep::Nothing };
    let profile = match profile {
        Ok(profile) => profile,
        Err(err) => {
            undo_keeping(m, paths, keep);
            if !guarded {
                let _ = set_wanted(paths, false);
            }
            return Reconciled::Failed {
                cause: ReconcileError::Profile(err),
                kill_switch_kept: guarded,
            };
        }
    };
    let plan = Plan::for_profile(&profile);
    let missing = plan.kinds().iter().any(|kind| !up.contains(kind));
    let dropped = plan.kinds().contains(&Kind::Tunnel) && !tunnel_up;
    if !missing && !dropped {
        return match kill_switch_again(m, paths, &plan) {
            Ok(None) => Reconciled::Nothing,
            Ok(Some(kill_switch)) => Reconciled::KillSwitchAgain(kill_switch),
            Err(err) => Reconciled::KillSwitchKept(err),
        };
    }
    let guarded = guarded && plan.has_kill_switch();
    let keep = if guarded { Keep::KillSwitch } else { Keep::Nothing };
    match on_keeping(m, paths, &plan, user, keep) {
        Ok(()) => Reconciled::BroughtUp,
        Err(err) => {
            if !guarded {
                let _ = set_wanted(paths, false);
            }
            Reconciled::Failed {
                cause: ReconcileError::On(err),
                kill_switch_kept: guarded,
            }
        }
    }
}

/// `plan`, its kill switch letting through the machine's local networks as they are now, when it
/// lets them through at all.
fn with_local_networks(m: &mut dyn Machine, plan: &Plan) -> Plan {
    if matches!(plan.kill_switch(), Some(KillSwitch::LocalNetwork(_))) {
        plan.with_local_networks(&local_network::read(m))
    } else {
        plan.clone()
    }
}

/// The kill switch's argument as it last went up, `None` when it isn't recorded.
fn kill_switch_recorded(paths: &Paths) -> Option<String> {
    fs::read_to_string(paths.kill_switch())
        .ok()
        .map(|text| text.trim().to_owned())
}

/// Records the kill switch among `args` as it went up, or that none did.
fn record_kill_switch(paths: &Paths, args: &[RootArg]) -> Result<(), StepError> {
    let path = paths.kill_switch();
    match args.iter().find(|arg| matches!(arg, RootArg::KillSwitch(_))) {
        Some(arg) => write_atomic(&path, &format!("{}\n", arg.arg()), 0o644).map_err(|err| StepError::File(path, err)),
        None => remove_file(&path),
    }
}

/// The local networks the kill switch lets through now, as it last went up: none while it is
/// strict or down.
pub fn local_networks_let_through(paths: &Paths) -> Vec<Cidr> {
    if !applied(paths).contains(&Kind::Root) {
        return Vec::new();
    }
    kill_switch_recorded(paths)
        .and_then(|arg| arg.strip_prefix("killswitch=lan|").map(str::to_owned))
        .map(|list| list.split(',').filter_map(Cidr::parse).collect())
        .unwrap_or_default()
}

/// Puts the kill switch up again, alone, when what `plan`'s lets through differs from what it
/// last went up with: the local networks changed, or whether it lets them through. Nothing while
/// the root part isn't up or the plan has no kill switch. What it lets through now, when it went
/// up again.
///
/// # Errors
///
/// The helper couldn't put it up; it lets through what it did before.
pub fn kill_switch_again(m: &mut dyn Machine, paths: &Paths, plan: &Plan) -> Result<Option<KillSwitch>, StepError> {
    if !applied(paths).contains(&Kind::Root) || !plan.has_kill_switch() {
        return Ok(None);
    }
    let plan = with_local_networks(m, plan);
    let Some(kill_switch) = plan.kill_switch() else {
        return Ok(None);
    };
    let arg = RootArg::KillSwitch(kill_switch.clone());
    if kill_switch_recorded(paths).as_deref() == Some(arg.arg().as_str()) {
        return Ok(None);
    }
    root::put_alone(m, &arg)?;
    record_kill_switch(paths, std::slice::from_ref(&arg))?;
    Ok(Some(kill_switch.clone()))
}

fn run(m: &mut dyn Machine, cmd: &Cmd) -> Result<Ran, StepError> {
    let program = cmd.program.rsplit('/').next().unwrap_or(&cmd.program).to_owned();
    match m.run(cmd) {
        Ok(ran) if ran.ok() => Ok(ran),
        Ok(ran) => {
            let said = ran.stderr.trim();
            Err(StepError::Program {
                program: program.clone(),
                said: Said(if said.is_empty() {
                    format!("{program} failed")
                } else {
                    said.to_owned()
                }),
            })
        }
        Err(err) => Err(StepError::Program {
            program,
            said: err.said,
        }),
    }
}

/// Puts one part in place. `guarded`: the kill switch is up, so the endpoint's last address
/// goes first, with no lookup outside the tunnel.
fn put(m: &mut dyn Machine, paths: &Paths, part: &Part, user: Option<&str>, guarded: bool) -> Result<(), StepError> {
    match part {
        Part::Tunnel(plan) => {
            let keys = secrets::load(m, &plan.config).map_err(StepError::Keys)?;
            let config = endpoints::resolved(m, paths, &plan.config, guarded)?;
            let connection = tunnel::connection(&config, &keys, &plan.dns, user);
            m.tunnel_up(&connection).map_err(StepError::Tunnel)
        }
        Part::Root(args) => {
            root::put(m, args)?;
            record_kill_switch(paths, args)
        }
        Part::Env(vars) => session::env_on(m, paths, vars),
        Part::Gnome(gnome) => session::gnome_on(m, paths, gnome),
        Part::DockerClient(docker) => session::docker_on(paths, docker),
        Part::Git(proxy) => session::git_on(paths, proxy),
    }
}

/// Takes one part away, whatever of it KubaOS put there, and puts back what the user had.
fn remove(m: &mut dyn Machine, paths: &Paths, kind: Kind, keep: Keep) -> Result<(), StepError> {
    match kind {
        Kind::Tunnel => m.tunnel_down().map_err(StepError::Tunnel),
        Kind::Root if keep == Keep::KillSwitch => root::remove(m, true),
        Kind::Root => {
            root::remove(m, false)?;
            remove_file(&paths.kill_switch())
        }
        Kind::Env => session::env_off(m, paths),
        Kind::Gnome => session::gnome_off(m, paths),
        Kind::DockerClient => session::docker_off(paths),
        Kind::Git => remove_file(&paths.git()),
    }
}

#[cfg(test)]
mod tests;
