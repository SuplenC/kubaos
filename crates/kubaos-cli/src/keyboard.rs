//! `kubaos keyboard`: the keyboard layouts, the first the default.
//!
//!   kubaos keyboard show                      print the layouts, one a line, the default first
//!   kubaos keyboard set <us,it(intl)>         replace them all
//!   kubaos keyboard add <layout>              add one at the end
//!   kubaos keyboard remove <layout>           take one out; the last one stays
//!   kubaos keyboard default <layout>          move one to the front
//!
//! A layout is spelled `it` or `us(intl)`. A change is checked against xkb's rules list, written
//! by `localectl set-x11-keymap` (systemd-localed writes `/etc/default/keyboard` on Ubuntu, and
//! asks polkit, so it runs as the desktop user), then reaches Hyprland with `hyprctl reload`.
//! The Command menu runs these. `KUBAOS_KEYBOARD_FILE` reads another file instead (the harness).
//! Exit codes: 0 ok, 1 `localectl` failed, 2 usage or a change the rules refuse.
//!
//! # Owns
//!
//! - Telling localed and Hyprland.
//!
//! # Not here
//!
//! - The layouts, the file's fields and the rules: `kubaos_core::keyboard`.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

use clap::Subcommand;
use kubaos_core::engine::machine::parse_xkb_layouts;
use kubaos_core::keyboard;
use kubaos_core::keyboard::Change;
use kubaos_core::keyboard::KeyboardFile;
use kubaos_core::keyboard::Layout;

use crate::usage_error;

/// The model localed is given when the file names none: the one the install writes.
const DEFAULT_MODEL: &str = "pc105";

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
        KeyboardCmd::Set { layouts } => Change::Set(keyboard::parse_list(&layouts)),
        KeyboardCmd::Add { layout } => Change::Add(Layout::parse(&layout)),
        KeyboardCmd::Remove { layout } => Change::Remove(Layout::parse(&layout)),
        KeyboardCmd::Default { layout } => Change::MakeDefault(Layout::parse(&layout)),
    };
    let known = fs::read_to_string(keyboard::RULES_LST)
        .map(|lst| parse_xkb_layouts(&lst))
        .unwrap_or_default();
    let layouts = match change.apply(&file.layouts, &known) {
        Ok(layouts) => layouts,
        Err(e) => return usage_error(e),
    };
    if let Err(msg) = tell_localed(&file, &layouts) {
        eprintln!("kubaos: {msg}");
        return ExitCode::FAILURE;
    }
    // Outside a Hyprland session there is nothing to reload; the next login reads the file.
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        let _ = Command::new("hyprctl").arg("reload").status();
    }
    println!("{}", keyboard::spell_list(&layouts));
    ExitCode::SUCCESS
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
