//! The startup apps in the menu: "Start on login" on every app's Result that doesn't start on
//! login, and "Don't start on login" on every one that does. Both are targeted Contributed actions
//! (`manifest.toml`), offered on the Results of the Identities this Extension names, `app:` and
//! the desktop id, as the Apps Extension names its Items. Each runs `kubaos autostart on` or `off`,
//! which writes the entry; its change comes back through the watch in `main.rs`, which sends both
//! sets again.
//!
//! # Owns
//!
//! - Which apps each Action is offered on, and what it runs.
//!
//! # Not here
//!
//! - Which entries start, and changing them: `kubaos_core::autostart`, through `kubaos autostart`
//!   (`crates/kubaos-cli`).

use std::collections::BTreeSet;

use kotodama_extension::Chain;
use kotodama_protocol::ItemKind;
use kubaos_core::autostart::Dirs;

use crate::run::Runner;
use crate::toggles::Ran;

/// The Action that starts an app on login.
pub const START: &str = "start-on-login";
/// The Action that keeps an app from starting on login.
pub const STOP: &str = "dont-start-on-login";

/// What the Apps Extension's Identities start with, before the desktop id.
const IDENTITY_PREFIX: &str = "app:";

/// The apps a launcher shows and the ones that start on login, by desktop id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StartupApps {
    apps: BTreeSet<String>,
    starting: BTreeSet<String>,
}

impl StartupApps {
    /// The apps and the startup apps in `dirs`.
    pub fn read(dirs: &Dirs) -> Self {
        StartupApps {
            apps: dirs.apps().into_iter().collect(),
            starting: dirs.startup_apps().into_iter().map(|app| app.id).collect(),
        }
    }

    /// Each Action and the Identities it is offered on, whole: "Start on login" on the apps that
    /// don't start, "Don't start on login" on every startup app.
    pub fn targets(&self) -> [(&'static str, Vec<String>); 2] {
        let identities = |ids: &mut dyn Iterator<Item = &String>| -> Vec<String> {
            ids.map(|id| format!("{IDENTITY_PREFIX}{id}")).collect()
        };
        [
            (START, identities(&mut self.apps.difference(&self.starting))),
            (STOP, identities(&mut self.starting.iter())),
        ]
    }
}

/// Whether `action` is one of the startup apps' Actions.
pub fn is_action(action: &str) -> bool {
    action == START || action == STOP
}

/// Runs `action` on the app `kind`, by its desktop id; the error is what the user reads.
///
/// # Errors
///
/// When `kind` isn't an app, `action` isn't one of the startup apps' Actions, or
/// `kubaos autostart` fails.
pub fn run(action: &str, kind: &ItemKind, runner: &impl Runner) -> Result<Ran, String> {
    let ItemKind::App { desktop_id } = kind else {
        return Err(format!("`{action}` is for apps, not a {}", kind.name()));
    };
    let verb = match action {
        START => "on",
        STOP => "off",
        _ => return Err(format!("`{desktop_id}` has no Action `{action}`")),
    };
    let argv = ["kubaos", "autostart", verb, desktop_id].map(str::to_owned);
    runner.output(&argv).map_err(|err| Chain(&err).to_string())?;
    Ok(Ran::Close)
}

#[cfg(test)]
mod tests;
