//! The file indexer under a saver stop: whether it runs and is paused, and the hold that pauses it.
//!
//! Nothing in a Hyprland session starts `localsearch-3`: its autostart entry is for other desktops
//! and its user unit is `static`, so D-Bus activates it the first time a program asks for it.
//! `localsearch inhibit` refuses to run while the indexer's bus name has no owner, and its pause
//! lives in the indexer, gone when the indexer goes while `sleep infinity` runs on. So a hold only
//! ever exists while the name has an owner, and the service follows the name.
//!
//! ```text
//! name appears, stop is quiet ─▶ hold starts      name goes ─▶ hold stops, whatever the stop
//! stop turns quiet, no name   ─▶ nothing yet      stop turns normal ─▶ hold stops
//! ```
//!
//! # Owns
//!
//! - [`State`], read from the indexer itself by [`read`], never from the hold's unit.
//! - [`hold`], the decision, and [`keep`], which carries it out. [`follow`], the bus name's watch.
//!
//! # Not here
//!
//! - The row's words around it: [`crate::status`]. When to ask: `main.rs`'s service.

use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use zbus::blocking::Connection;
use zbus::blocking::Proxy;
use zbus::blocking::fdo::DBusProxy;
use zbus::proxy::CacheProperties;
use zbus::proxy::MethodFlags;

use crate::live;

/// The transient user unit that holds the file indexer paused.
pub const QUIET_UNIT: &str = "kubaos-power-quiet";
/// The indexer's name on the session bus.
pub const BUS_NAME: &str = "org.freedesktop.LocalSearch3";
const MINER_PATH: &str = "/org/freedesktop/Tracker3/Miner/Files";
const MINER_INTERFACE: &str = "org.freedesktop.Tracker3.Miner";
/// How long `status` waits for an indexer that owns its name to answer.
const ANSWER: Duration = Duration::from_secs(2);

/// What the file indexer is doing, as it says itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Nothing owns its bus name: it does no work, and D-Bus starts it when a program asks.
    NotRunning,
    Running,
    /// It runs, and some program's pause is on it.
    Paused,
}

impl State {
    /// Its words in `status`.
    pub fn words(self) -> &'static str {
        match self {
            State::NotRunning => "indexer not running",
            State::Running => "indexer running",
            State::Paused => "indexer paused",
        }
    }

    /// Whether it is what a stop asks for, `quiet` or not. An indexer that isn't running does no
    /// work in either: a quiet stop's hold comes the moment it starts.
    pub fn agrees(self, quiet: bool) -> bool {
        match self {
            State::NotRunning => true,
            State::Running => !quiet,
            State::Paused => quiet,
        }
    }
}

/// What to do with the hold's unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    Start,
    Stop,
}

/// What the hold needs, with the stop `quiet` or not, the indexer `running` or not and the unit
/// `held` or not. A hold left on an indexer that has gone pauses nothing, and would keep the next
/// one from getting a hold of its own.
pub fn hold(quiet: bool, running: bool, held: bool) -> Option<Hold> {
    match (quiet && running, held) {
        (true, false) => Some(Hold::Start),
        (false, true) => Some(Hold::Stop),
        _ => None,
    }
}

/// Starts or stops the hold as a stop `quiet` or not asks, for an indexer `running` or not. The
/// hold is a transient unit, so stopping it lets the indexer go and a crash of this service doesn't
/// leave it paused.
pub fn keep(quiet: bool, running: bool) {
    if !installed() {
        return;
    }
    match hold(quiet, running, live::unit_active(QUIET_UNIT)) {
        Some(Hold::Start) => {
            let _ = Command::new("systemd-run")
                .args([
                    "--user",
                    "--quiet",
                    "--collect",
                    &format!("--unit={QUIET_UNIT}"),
                    "--",
                    "localsearch",
                    "inhibit",
                    "sleep",
                    "infinity",
                ])
                .stdin(Stdio::null())
                .status();
        }
        Some(Hold::Stop) => {
            let _ = Command::new("systemctl")
                .args(["--user", "stop", &format!("{QUIET_UNIT}.service")])
                .stdin(Stdio::null())
                .status();
        }
        None => {}
    }
}

/// Whether the machine has the file indexer, `localsearch`.
pub fn installed() -> bool {
    Command::new("sh")
        .args(["-c", "command -v localsearch"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Whether something owns the indexer's bus name; `false` without a session bus to ask.
pub fn running() -> bool {
    let owned = || -> zbus::Result<bool> {
        let bus = Connection::session()?;
        Ok(DBusProxy::new(&bus)?.name_has_owner(BUS_NAME.try_into()?)?)
    };
    owned().unwrap_or(false)
}

/// What the indexer is doing; `None` without one, or without a session bus or an answer from it.
pub fn read() -> Option<State> {
    if !installed() {
        return None;
    }
    let bus = zbus::blocking::connection::Builder::session()
        .ok()?
        .method_timeout(ANSWER)
        .build()
        .ok()?;
    let owned = DBusProxy::new(&bus)
        .ok()?
        .name_has_owner(BUS_NAME.try_into().ok()?)
        .ok()?;
    if !owned {
        return Some(State::NotRunning);
    }
    let pauses = pauses(&bus).ok()?;
    Some(if pauses.is_empty() {
        State::Running
    } else {
        State::Paused
    })
}

/// The programs pausing the indexer.
///
/// HAZARD: no auto start. Asking the name the plain way starts the indexer through D-Bus, so a
/// `status` on a saver stop would set off the work it reports on, as `localsearch status` does.
fn pauses(bus: &Connection) -> zbus::Result<Vec<String>> {
    let proxy: Proxy<'_> = zbus::blocking::proxy::Builder::new(bus)
        .destination(BUS_NAME)?
        .path(MINER_PATH)?
        .interface(MINER_INTERFACE)?
        .cache_properties(CacheProperties::No)
        .build()?;
    let details: Option<(Vec<String>, Vec<String>)> =
        proxy.call_with_flags("GetPauseDetails", MethodFlags::NoAutoStart.into(), &())?;
    Ok(details.map(|(programs, _reasons)| programs).unwrap_or_default())
}

/// Calls `changed` with whether the indexer runs: once at the start, so nothing falls between a
/// first look and the watch, then each time its bus name changes hands.
///
/// # Errors
///
/// A session bus that can't be reached or is lost.
pub fn follow(changed: impl Fn(bool)) -> zbus::Result<()> {
    let bus = Connection::session()?;
    let dbus = DBusProxy::new(&bus)?;
    let owners = dbus.receive_name_owner_changed_with_args(&[(0, BUS_NAME)])?;
    changed(dbus.name_has_owner(BUS_NAME.try_into()?)?);
    for owner in owners {
        changed(owner.args()?.new_owner().is_some());
    }
    Ok(())
}

/// [`follow`] on a thread of its own, its failure in the journal: the hold then comes at the next
/// stop only, and `status` still tells.
pub fn follow_in_background(changed: impl Fn(bool) + Send + 'static) {
    thread::spawn(move || {
        if let Err(err) = follow(changed) {
            log::warn!("couldn't follow the file indexer on the session bus: {err}");
        }
    });
}

#[cfg(test)]
mod tests;
