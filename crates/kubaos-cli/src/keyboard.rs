//! `kubaos keyboard`: the keyboard layouts, the first the default.
//!
//!   kubaos keyboard show                      print the layouts, one a line, the default first
//!   kubaos keyboard set <us,it(intl)>         replace them all
//!   kubaos keyboard add <layout>              add one at the end
//!   kubaos keyboard remove <layout>           take one out; the last one stays
//!   kubaos keyboard default <layout>          move one to the front
//!   kubaos keyboard move <layout> <place>     move one to a place, 1 the front (the default)
//!   kubaos keyboard list                      every layout and variant xkb knows, a line each:
//!                                             the spelling, a tab, the name; sorted by name
//!   kubaos keyboard accents [on | off | apply]  the Mac-style accents on Left Alt: print them,
//!                                             turn them on or off, or apply the setting
//!
//! A layout is spelled `it` or `us(intl)`. A change is checked against xkb's rules list, written
//! by `localectl set-x11-keymap` (systemd-localed writes `/etc/default/keyboard` on Ubuntu, and
//! asks polkit, so it runs as the desktop user), then reaches Hyprland with `hyprctl reload`.
//! The Command menu and the control center's keyboard list run these. `KUBAOS_KEYBOARD_FILE`
//! reads another file instead (the harness).
//!
//! The accents are keyd, which `system/50-hyprland.sh` installs and enables. `on` and `off` write
//! the setting to `~/.config/kubaos/keyboard/keyboard.toml`, then apply it: keyd runs while they
//! are on and is stopped while they are off (a polkit rule lets the desktop user start and stop
//! `keyd.service`), and Hyprland reloads, so the Shortcuts View's cheat sheet follows. `apply`
//! runs at login (`config/hypr/autostart.lua`), as keyd starts with the machine.
//! Exit codes: 0 ok, 1 `localectl` or `systemctl` failed, 2 usage or a change the rules refuse.
//!
//! # Owns
//!
//! - Telling localed, systemd and Hyprland.
//!
//! # Not here
//!
//! - The layouts, the file's fields, the rules and the accents setting: `kubaos_core::keyboard`.
//! - What keyd does with Left Alt: `system/files/keyd/kubaos.conf`.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

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

/// The model localed is given when the file names none: the one the install writes.
const DEFAULT_MODEL: &str = "pc105";

/// The unit `system/50-hyprland.sh` installs with keyd.
const KEYD_UNIT: &str = "keyd.service";

#[derive(Subcommand)]
pub enum KeyboardCmd {
    /// Print the layouts, one a line, the default first.
    Show,
    /// Replace the layouts: a comma-separated list, the default first (us,it(intl)).
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
    let path = env::var_os("KUBAOS_KEYBOARD_FILE")
        .map_or_else(|| PathBuf::from(keyboard::DEFAULT_KEYBOARD), PathBuf::from);
    let file = KeyboardFile::parse(&fs::read_to_string(&path).unwrap_or_default());
    let change = match cmd {
        KeyboardCmd::Show => {
            for layout in &file.layouts {
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
        KeyboardCmd::Accents { change } => return accents(change),
        KeyboardCmd::Set { layouts } => Change::Set(keyboard::parse_list(&layouts)),
        KeyboardCmd::Add { layout } => Change::Add(Layout::parse(&layout)),
        KeyboardCmd::Remove { layout } => Change::Remove(Layout::parse(&layout)),
        KeyboardCmd::Default { layout } => Change::MakeDefault(Layout::parse(&layout)),
        KeyboardCmd::Move { layout, place } => {
            let Some(index) = place.checked_sub(1) else {
                return usage_error(LayoutsError::Place(file.layouts.len()));
            };
            Change::Move(Layout::parse(&layout), index)
        }
    };
    let known = parse_xkb_layouts(&rules_lst());
    let layouts = match change.apply(&file.layouts, &known) {
        Ok(layouts) => layouts,
        Err(e) => return usage_error(e),
    };
    if let Err(msg) = tell_localed(&file, &layouts) {
        eprintln!("kubaos: {msg}");
        return ExitCode::FAILURE;
    }
    reload_hyprland();
    println!("{}", keyboard::spell_list(&layouts));
    ExitCode::SUCCESS
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
            MacAccents::On => Err(format!("{KEYD_UNIT} isn't installed; system/50-hyprland.sh installs it")),
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

/// Writes `layouts` through localed, keeping the file's model and options.
fn tell_localed(file: &KeyboardFile, layouts: &[Layout]) -> Result<(), String> {
    let (layout, variant) = keyboard::to_fields(layouts);
    // No variants at all go as one empty field, as a single layout's do, not as bare commas.
    let variant = if variant.chars().all(|c| c == ',') {
        String::new()
    } else {
        variant
    };
    let model = if file.model.is_empty() {
        DEFAULT_MODEL
    } else {
        &file.model
    };
    let status = Command::new("localectl")
        .args(["set-x11-keymap", &layout, model, &variant, &file.options])
        .status()
        .map_err(|e| format!("running localectl: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "localectl set-x11-keymap {layout} failed ({status})"
        ))
    }
}
