//! KubaOS's counters: named counts, each with an auto reset (a rule such as "every day at 9", or a
//! stop at N), kept in order, with every change logged forever.
//!
//! ```text
//! ~/.local/state/kubaos/counters.toml   the counters ([`store`])
//! ~/.local/state/kubaos/counters.log    every change, appended ([`history`])
//! $XDG_RUNTIME_DIR/kubaos-counters.json what the bar watches, written after each change ([`live`])
//!
//! menu, key, shell ─kubaos-counters add Pushups─▶ lock ─▶ due resets ─▶ the change ─▶ the files
//! kubaos-counters-reset.timer (transient, at the next reset) ─▶ kubaos-counters tick ─┘
//! ```
//!
//! # Owns
//!
//! - The counters and their changes ([`store`]), the change log and the history it gives
//!   ([`history`]), and the JSON the shell reads ([`live`]); where the files are.
//!
//! # Not here
//!
//! - The command line, the lock, the reset timer and the toasts: `main.rs`. Reading rules:
//!   `kubaos_clock::recurrence`. The menu's Items: `crates/kotodama-kubaos/src/counters.rs`.

pub mod history;
pub mod live;
pub mod store;

use std::path::Path;
use std::path::PathBuf;

/// Where the counters' files are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    /// `counters.toml`.
    pub counters: PathBuf,
    /// `counters.log`.
    pub log: PathBuf,
    /// `counters.lock`, held while a command reads and writes the other two.
    pub lock: PathBuf,
}

impl Paths {
    /// The files in `dir`.
    #[must_use]
    pub fn in_dir(dir: &Path) -> Self {
        Paths {
            counters: dir.join("counters.toml"),
            log: dir.join("counters.log"),
            lock: dir.join("counters.lock"),
        }
    }

    /// The files in KubaOS's state folder, `$XDG_STATE_HOME/kubaos` or `~/.local/state/kubaos`;
    /// `None` with neither variable set.
    pub fn from_env() -> Option<Self> {
        let var = |name| std::env::var_os(name).filter(|value| !value.is_empty()).map(PathBuf::from);
        let state = var("XDG_STATE_HOME").or_else(|| Some(var("HOME")?.join(".local/state")))?;
        Some(Paths::in_dir(&state.join("kubaos")))
    }
}

/// The shell's state file, `$XDG_RUNTIME_DIR/kubaos-counters.json`; `None` without the variable.
pub fn live_file() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(|dir| PathBuf::from(dir).join(live::FILE))
}
