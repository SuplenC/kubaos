//! `kubaos keyboard`: the keyboard layouts, the first the default.
//!
//!   kubaos keyboard show                      print the layouts, one a line, the default first
//!   kubaos keyboard set <us,de(nodeadkeys)>   replace them all
//!   kubaos keyboard add <layout>              add one at the end
//!   kubaos keyboard remove <layout>           take one out; the last one stays
//!   kubaos keyboard default <layout>          move one to the front
//!   kubaos keyboard move <layout> <place>     move one to a place, 1 the front (the default)
//!   kubaos keyboard list                      every layout and variant xkb knows, a line each:
//!                                             the spelling, a tab, the name; sorted by name
//!   kubaos keyboard seed                      keep the system's layouts as the user's, unless
//!                                             the user has a list already (a user migration)
//!   kubaos keyboard accents [on | off | apply]  the Mac-style accents on Left Alt: print them,
//!                                             turn them on or off, or apply the setting
//!
//! A layout is spelled `it` or `us(intl)`. The layouts are the user's: `layouts` in
//! `~/.config/kubaos/desktop/keyboard/keyboard.toml`, which Hyprland reads
//! (`config/hypr/input.lua`). Until the user has a list, it is the one in `/etc/default/keyboard`,
//! which the install wrote. A change is checked against xkb's rules list, written to the user's
//! file and reaches Hyprland with `hyprctl reload`, with no privilege asked.
//!
//! `/etc/default/keyboard` is the system's: the console and the disk's passphrase prompt type
//! with its layout. It follows the default alone. A new default is written there by a root helper
//! that pkexec runs (polkit decides, so `kubaos` runs as the desktop user). Ubuntu's
//! systemd-localed can't do that writing: a D-Bus policy shipped with systemd refuses its
//! SetX11Keyboard to everyone, root included, and Debian's localed no longer writes that file.
//! When the helper fails the user's layouts have changed all the same; the last line says that the
//! console keeps the old default and why, which the control center shows as is, after a line with
//! everything pkexec said.
//! The Command menu and the control center's keyboard list run these. `KUBAOS_KEYBOARD_FILE`
//! reads another system file instead (the harness).
//!
//! The accents are keyd, which `system/50-hyprland.sh` installs and enables. `on` and `off` write
//! the setting to `~/.config/kubaos/desktop/keyboard/keyboard.toml`, then apply it: keyd runs while
//! they are on and is stopped while they are off (a polkit rule lets the desktop user start and
//! stop `keyd.service`), and Hyprland reloads, so the Shortcuts View's cheat sheet follows. `apply`
//! runs at login (`config/hypr/autostart.lua`), as keyd starts with the machine.
//! Exit codes: 0 ok, 1 writing the user's file, pkexec, the helper or `systemctl` failed, 2 usage
//! or a change the rules refuse.
//!
//! # Owns
//!
//! - Writing the user's layouts, running the root helper for a new default, and telling systemd and
//!   Hyprland.
//!
//! # Not here
//!
//! - The layouts, the file's fields, the rules and the accents setting: `kubaos_core::keyboard`.
//! - Writing the file as root: `system/files/keyboard/kubaos-keyboard-write`.
//! - What keyd does with Left Alt: `system/files/keyd/kubaos.conf`.

use std::env;
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;

use clap::Subcommand;
use clap::ValueEnum;
use kubaos_core::engine::machine::parse_xkb_layouts;
use kubaos_core::keyboard;
use kubaos_core::keyboard::Change;
use kubaos_core::keyboard::KeyboardFile;
use kubaos_core::keyboard::Layout;
use kubaos_core::keyboard::LayoutsError;
use kubaos_core::keyboard::settings;
use kubaos_core::keyboard::settings::MacAccents;
use kubaos_core::keyboard::settings::Settings;

use crate::job::chain;
use crate::usage_error;

/// The model written when the file names none: the one the install writes.
const DEFAULT_MODEL: &str = "pc105";

/// The root helper that writes `/etc/default/keyboard`, which `system/50-hyprland.sh` installs.
const WRITER: &str = "/usr/local/libexec/kubaos-keyboard-write";

/// The unit `system/50-hyprland.sh` installs with keyd.
const KEYD_UNIT: &str = "keyd.service";

#[derive(Subcommand)]
pub enum KeyboardCmd {
    /// Print the layouts, one a line, the default first.
    Show,
    /// Replace the layouts: a comma-separated list, the default first (us,de(nodeadkeys)).
    Set { layouts: String },
    /// Add a layout at the end (it, or us(intl)).
    Add { layout: String },
    /// Take a layout out; the last one stays.
    Remove { layout: String },
    /// Make a layout the default, the first one.
    Default { layout: String },
    /// Move a layout to a place, 1 the front, where it is the default.
    Move { layout: String, place: usize },
    /// Print every layout and variant xkb knows: the spelling, a tab, the name.
    List,
    /// Keep the system's layouts as the user's, unless the user has a list already.
    Seed,
    /// The Mac-style accents on Left Alt: print whether they are on, or change them.
    Accents { change: Option<AccentsChange> },
}

/// What `kubaos keyboard accents` does besides printing the setting.
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AccentsChange {
    /// Turn them on and start keyd.
    On,
    /// Turn them off and stop keyd.
    Off,
    /// Start or stop keyd as the setting says.
    Apply,
}

pub fn main(cmd: KeyboardCmd) -> ExitCode {
    let KeyboardCmd::Accents { change } = cmd else {
        return layouts(cmd);
    };
    accents(change)
}

/// Every `kubaos keyboard` but `accents`: prints the layouts or changes them.
fn layouts(cmd: KeyboardCmd) -> ExitCode {
    let Some(config_home) = config_home() else {
        return usage_error("HOME isn't set");
    };
    let mine = settings::path(&config_home);
    let user = match Settings::load(&mine) {
        Ok(user) => user,
        Err(e) => return usage_error(chain(&e)),
    };
    let system_path =
        env::var_os("KUBAOS_KEYBOARD_FILE").map_or_else(|| PathBuf::from(keyboard::DEFAULT_KEYBOARD), PathBuf::from);
    let system = KeyboardFile::parse(&fs::read_to_string(&system_path).unwrap_or_default());
    let current = keyboard::layouts(&user, &system);
    let change = match cmd {
        KeyboardCmd::Show => {
            for layout in &current {
                println!("{layout}");
            }
            return ExitCode::SUCCESS;
        }
        KeyboardCmd::List => {
            for (layout, name) in keyboard::catalogue(&rules_lst()) {
                println!("{layout}\t{name}");
            }
            return ExitCode::SUCCESS;
        }
        KeyboardCmd::Seed => {
            if user.layouts.is_empty()
                && !current.is_empty()
                && let Err(e) = settings::set_layouts(&mine, &current)
            {
                eprintln!("kubaos: {}", chain(&e));
                return ExitCode::FAILURE;
            }
            return ExitCode::SUCCESS;
        }
        KeyboardCmd::Accents { .. } => unreachable!("`main` runs the accents"),
        KeyboardCmd::Set { layouts } => Change::Set(keyboard::parse_list(&layouts)),
        KeyboardCmd::Add { layout } => Change::Add(Layout::parse(&layout)),
        KeyboardCmd::Remove { layout } => Change::Remove(Layout::parse(&layout)),
        KeyboardCmd::Default { layout } => Change::MakeDefault(Layout::parse(&layout)),
        KeyboardCmd::Move { layout, place } => {
            let Some(index) = place.checked_sub(1) else {
                return usage_error(LayoutsError::Place(current.len()));
            };
            Change::Move(Layout::parse(&layout), index)
        }
    };
    let known = parse_xkb_layouts(&rules_lst());
    let layouts = match change.apply(&current, &known) {
        Ok(layouts) => layouts,
        Err(e) => return usage_error(e),
    };
    if let Err(e) = settings::set_layouts(&mine, &layouts) {
        eprintln!("kubaos: the keyboard layouts weren't changed: {}", chain(&e));
        return ExitCode::FAILURE;
    }
    reload_hyprland();
    println!("{}", keyboard::spell_list(&layouts));
    // The console and the disk's passphrase prompt type with the default alone, so only a new
    // default asks for an admin.
    let default = &layouts[0];
    let Some(old) = system.layouts.first().filter(|old| *old != default) else {
        return ExitCode::SUCCESS;
    };
    match write_system_default(&system, default) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "kubaos: the keyboard layouts changed, but the console and the disk's passphrase \
                 prompt still type with {old}: {}",
                chain(&e)
            );
            ExitCode::FAILURE
        }
    }
}

/// xkb's rules list; empty when it can't be read, and then any layout name passes.
fn rules_lst() -> String {
    fs::read_to_string(keyboard::RULES_LST).unwrap_or_default()
}

/// Outside a Hyprland session there is nothing to reload; the next login reads the files.
fn reload_hyprland() {
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        let _ = Command::new("hyprctl").arg("reload").status();
    }
}

/// `kubaos keyboard accents [on | off | apply]`: prints the setting, or turns the accents on or
/// off and applies that, or applies the setting as it is.
fn accents(change: Option<AccentsChange>) -> ExitCode {
    let Some(config_home) = config_home() else {
        return usage_error("HOME isn't set");
    };
    let path = settings::path(&config_home);
    let wanted = match change {
        Some(AccentsChange::On) => MacAccents::On,
        Some(AccentsChange::Off) => MacAccents::Off,
        None | Some(AccentsChange::Apply) => match Settings::load(&path) {
            Ok(settings) => MacAccents::from(settings),
            Err(e) => return usage_error(chain(&e)),
        },
    };
    let Some(change) = change else {
        println!("{wanted}");
        return ExitCode::SUCCESS;
    };
    if change != AccentsChange::Apply
        && let Err(e) = settings::set_mac_accents(&path, wanted)
    {
        eprintln!("kubaos: {}", chain(&e));
        return ExitCode::FAILURE;
    }
    if let Err(msg) = tell_keyd(wanted) {
        eprintln!("kubaos: {msg}");
        return ExitCode::FAILURE;
    }
    // At login Hyprland has just read the setting; after a change its cheat sheet follows.
    if change != AccentsChange::Apply {
        reload_hyprland();
    }
    println!("{wanted}");
    ExitCode::SUCCESS
}

/// `~/.config`, or `XDG_CONFIG_HOME` when it is set.
fn config_home() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
}

/// Starts keyd for the accents, or stops it. Without its unit (a machine `system/50-hyprland.sh`
/// never ran on) there is nothing to start, and nothing to stop.
fn tell_keyd(accents: MacAccents) -> Result<(), String> {
    let listed = Command::new("systemctl")
        .args(["list-unit-files", "--no-legend", KEYD_UNIT])
        .output()
        .map_err(|e| format!("running systemctl: {e}"))?;
    if String::from_utf8_lossy(&listed.stdout).trim().is_empty() {
        return match accents {
            MacAccents::On => Err(format!(
                "{KEYD_UNIT} isn't installed; system/50-hyprland.sh installs it"
            )),
            MacAccents::Off => Ok(()),
        };
    }
    let verb = match accents {
        MacAccents::On => "start",
        MacAccents::Off => "stop",
    };
    let status = Command::new("systemctl")
        .args([verb, "--no-ask-password", KEYD_UNIT])
        .status()
        .map_err(|e| format!("running systemctl: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("systemctl {verb} {KEYD_UNIT} failed ({status})"))
    }
}

/// Writes `default` to `/etc/default/keyboard` as its one layout through the root helper, keeping
/// the file's model and options; the helper then brings the console and the initramfs in line.
///
/// pkexec asks polkit, which lets an active admin at the machine through
/// (`system/files/polkit/50-kubaos-keyboard.rules`) and asks anyone else for a password through
/// the session's agent. Everything pkexec and the helper wrote to stderr is logged on a line of its
/// own, then read for the refusal, so the last line is one sentence.
fn write_system_default(file: &KeyboardFile, default: &Layout) -> Result<(), SystemError> {
    let model = if file.model.is_empty() {
        DEFAULT_MODEL
    } else {
        &file.model
    };
    let fields = [default.layout.as_str(), model, &default.variant, &file.options];
    // stdin stays the caller's: in a terminal, pkexec's text agent asks for the password there.
    let out = Command::new("pkexec")
        .args(pkexec_args(io::stdin().is_terminal(), fields))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()
        .map_err(SystemError::Run)?;
    if out.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    eprintln!("kubaos: {}", logged(&stderr, out.status));
    Err(refusal(&stderr, out.status))
}

/// pkexec's arguments for the helper and its four `fields`. Without a terminal (the Keyboard tab,
/// the Command menu) only the session's agent may ask: pkexec's own text agent would fail on the
/// missing tty with words of its own, where polkit's refusal reads as one.
fn pkexec_args(terminal: bool, fields: [&str; 4]) -> Vec<&str> {
    let agent = (!terminal).then_some("--disable-internal-agent");
    agent.into_iter().chain([WRITER]).chain(fields).collect()
}

/// Why the system's keyboard wasn't written.
#[derive(Debug, thiserror::Error)]
enum SystemError {
    #[error("running pkexec")]
    Run(#[source] io::Error),
    /// No rule let the caller through and no agent asked for a password.
    #[error("polkit refused it")]
    Refused,
    /// polkit wanted a password and no agent was there to ask for it.
    #[error("no polkit agent is running to ask for the password")]
    NoAgent,
    #[error("the password prompt was dismissed")]
    Dismissed,
    /// A machine installed before the helper shipped, until the update's migration installs it.
    #[error("{WRITER} isn't installed; system/50-hyprland.sh installs it")]
    NoWriter,
    #[error(transparent)]
    Failed(HelperSaid),
}

/// The helper's or pkexec's reason for a failure: the last line it wrote.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct HelperSaid(String);

/// What pkexec prints when polkit refuses.
const REFUSED: &str = "Not authorized";

/// What pkexec prints when polkit wants a password and no agent is registered for the session.
const NO_AGENT: &str = "No authentication agent found";

/// What pkexec prints when the password prompt is dismissed.
const DISMISSED: &str = "Request dismissed";

/// What pkexec starts a line with when the program it should run isn't there: it can't look at
/// it, or can't run it.
const NO_PROGRAM: [&str; 2] = ["Error accessing", "Cannot run program"];

/// The non-empty lines of `stderr`, trimmed.
fn said_lines(stderr: &str) -> impl Iterator<Item = &str> {
    stderr.lines().map(str::trim).filter(|line| !line.is_empty())
}

/// The log line for a failed write: the exit status, then everything pkexec and the helper said,
/// on one line.
fn logged(stderr: &str, status: impl Display) -> String {
    let said: Vec<&str> = said_lines(stderr).collect();
    format!("pkexec kubaos-keyboard-write failed ({status}): {}", said.join(" / "))
}

/// The error for a failed write, from pkexec's stderr and exit status.
fn refusal(stderr: &str, status: impl Display) -> SystemError {
    let said = |words: &str| said_lines(stderr).any(|line| line.contains(words));
    if said(REFUSED) {
        return SystemError::Refused;
    }
    if said(NO_AGENT) {
        return SystemError::NoAgent;
    }
    if said(DISMISSED) {
        return SystemError::Dismissed;
    }
    let missing = |line: &str| line.contains(WRITER) && NO_PROGRAM.iter().any(|start| line.starts_with(start));
    if said_lines(stderr).any(missing) {
        return SystemError::NoWriter;
    }
    let reason = said_lines(stderr)
        .last()
        .map_or_else(|| format!("pkexec failed ({status})"), str::to_owned);
    SystemError::Failed(HelperSaid(reason))
}

#[cfg(test)]
mod tests;
