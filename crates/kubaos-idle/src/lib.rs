//! KubaOS's idle settings: when the screen locks, turns off and suspends by itself, on battery
//! and plugged in; what closing the lid does; Keep awake; and a lock scheduled for a time.
//!
//! ```text
//! idle.toml ─kubaos-idle set─▶ ~/.local/state/kubaos/idle/hypridle.conf ◀─source─ hypridle.conf
//!                          └─▶ a lid holder (systemd-inhibit handle-lid-switch), when the lid
//!                              setting isn't logind's own
//! kubaos-idle awake 1h ─▶ systemd-inhibit --what=idle, which hypridle honours, for an hour
//! kubaos-idle lock at 17:00 ─▶ systemd --user transient timers: two heads-ups, then the lock
//! ```
//!
//! # Owns
//!
//! - The settings file and its changes ([`settings`]), the words the toasts say ([`words`]),
//!   hypridle's listeners ([`hypridle`]), the lid's decision ([`lid`]), Keep awake's state
//!   ([`awake`]) and the scheduled action's ([`schedule`]).
//! - What they read from the machine: the power source, logind's lid and dock, the session
//!   ([`machine`]).
//!
//! # Not here
//!
//! - Running programs, the holders' loops and the command line: `main.rs`.
//! - Reading durations and times of day: `kubaos_clock::duration` and `kubaos_clock::alarm`.
//! - The menu's Items: `crates/kotodama-kubaos/src/idle.rs` and `config/kotodama/kubaos.toml`. The
//!   bar's markers: `config/quickshell/services/Idle.qml`.

#![forbid(unsafe_code)]

pub mod awake;
pub mod hypridle;
pub mod lid;
pub mod machine;
pub mod schedule;
pub mod settings;
pub mod words;

use std::path::PathBuf;

/// `$XDG_RUNTIME_DIR`, where the state that ends with the login lives; `None` without it.
pub fn runtime_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
}
