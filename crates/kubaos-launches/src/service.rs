//! The launch watcher: it follows the systemd user manager for new units and tells Hyprland of
//! each that runs an app's launch, while the setting is on.
//!
//! The manager sends its signals only to a client that asked with `Subscribe`; the match rule is
//! in place before that, so no unit made in between is missed. It sends `UnitNew` again for a unit
//! it loads again, as stopping one does, so each unit is told once ([`Seen`]).
//!
//! # Owns
//!
//! - [`run`], the loop.
//!
//! # Not here
//!
//! - Which units are an app's launch: [`crate::unit`].
//! - The request to Hyprland: [`crate::hyprland`].

use std::collections::VecDeque;

use zbus::MatchRule;
use zbus::blocking::Connection;
use zbus::blocking::MessageIterator;
use zbus::message::Type;
use zbus::zvariant::OwnedObjectPath;

use crate::hyprland::Hyprland;
pub use crate::hyprland::TellError;
use crate::settings::Settings;
use crate::unit;

const SYSTEMD: &str = "org.freedesktop.systemd1";
const MANAGER: &str = "org.freedesktop.systemd1.Manager";
const MANAGER_PATH: &str = "/org/freedesktop/systemd1";

/// How many units [`Seen`] remembers. Each launch's unit has a name of its own, and a unit is
/// loaded again within seconds of its launch, so the last few dozen are plenty.
const REMEMBERED: usize = 64;

/// The service couldn't start, or lost the bus.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("connecting to the session bus")]
    Session(#[source] zbus::Error),
    #[error("following the systemd user manager")]
    Follow(#[source] zbus::Error),
}

/// Follows the user manager until the bus goes, telling `hyprland` of each app's launch while
/// `settings` has it on. A launch Hyprland can't be told of goes to `failed`, and the next is
/// told as usual.
///
/// # Errors
///
/// When the session bus can't be reached, or the manager can't be followed.
pub fn run(settings: &Settings, hyprland: &Hyprland, mut failed: impl FnMut(&TellError)) -> Result<(), RunError> {
    let bus = Connection::session().map_err(RunError::Session)?;
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender(SYSTEMD)
        .and_then(|rule| rule.interface(MANAGER))
        .and_then(|rule| rule.member("UnitNew"))
        .map_err(RunError::Follow)?
        .build();
    let signals = MessageIterator::for_match_rule(rule, &bus, None).map_err(RunError::Follow)?;
    bus.call_method(Some(SYSTEMD), MANAGER_PATH, Some(MANAGER), "Subscribe", &())
        .map_err(RunError::Follow)?;
    let mut seen = Seen::default();
    for message in signals {
        let message = message.map_err(RunError::Follow)?;
        let Ok((name, _)) = message.body().deserialize::<(String, OwnedObjectPath)>() else {
            continue;
        };
        let Some(app) = unit::app_id(&name) else {
            continue;
        };
        if !seen.first_time(&name) || !settings.open_where_launched() {
            continue;
        }
        if let Err(err) = hyprland.started(&name, &app) {
            failed(&err);
        }
    }
    Ok(())
}

/// The units told last, so a unit loaded again isn't told twice.
#[derive(Debug, Default)]
struct Seen {
    units: VecDeque<String>,
}

impl Seen {
    /// Whether `unit` is new, remembering it.
    fn first_time(&mut self, unit: &str) -> bool {
        if self.units.iter().any(|seen| seen == unit) {
            return false;
        }
        if self.units.len() == REMEMBERED {
            self.units.pop_front();
        }
        self.units.push_back(unit.to_owned());
        true
    }
}

#[cfg(test)]
mod tests;
