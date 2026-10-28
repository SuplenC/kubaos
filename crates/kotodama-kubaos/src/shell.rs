//! The shell's settings the Command menu changes: the focus stutter, a key of the user layer's
//! `desktop/quickshell/settings.json`. The Extension writes the file itself, keeping the user's
//! comments and other keys, so the Item works with the shell down; the shell reads the file again
//! as soon as it changes (`config/quickshell/settings/Settings.qml`).
//!
//! # Owns
//!
//! - The "Focus stutter" Item and its Actions.
//! - Reading and setting one key of the file, in place.
//!
//! # Not here
//!
//! - What the stutter does: the focus tube in `KubaOS.Components`' `FocusRing` and `kubaos-ui`'s
//!   `focus`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kubaos_user_layer::App;

use crate::toggles::Ran;

/// The Item kind of every shell setting.
pub const KIND: &str = "kubaos.shell-setting";
/// The focus stutter's Item.
pub const STUTTER: &str = "focus-stutter";
/// The focus stutter's key in the file.
pub const STUTTER_KEY: &str = "focusStutter";

/// The primary Action: on if off, off if on.
pub const TOGGLE: &str = "toggle";
pub const ON: &str = "on";
pub const OFF: &str = "off";

/// The stutter when the file doesn't set it: on, as `Settings.qml` has it.
const DEFAULT_STUTTER: bool = true;

/// Why the settings file couldn't be changed.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("reading the shell's settings")]
    Read(#[source] io::Error),
    #[error("the shell's settings aren't JSON, so they were left as they are")]
    Invalid(#[source] serde_json::Error),
    #[error("the shell's settings put more than one key on a line, so they were left as they are")]
    Crowded,
    #[error("writing the shell's settings")]
    Write(#[source] io::Error),
}

/// The file in the user layer, `~/.config/kubaos/desktop/quickshell/settings.json`.
pub fn path() -> Option<PathBuf> {
    kubaos_user_layer::config_home().map(|home| kubaos_user_layer::dir(&home, App::Quickshell).join("settings.json"))
}

/// The Items.
pub fn items() -> Vec<Item> {
    vec![Item {
        id: STUTTER.to_owned(),
        kind: kind(),
        identity: None,
        title: "Focus stutter: on/off".to_owned(),
        subtitle: Some("The focus tube strikes on like a neon sign, or simply fades in".to_owned()),
        icon: Some("preferences-desktop-accessibility".to_owned()),
        keyword: None,
        keywords: ["focus ring", "neon", "flicker", "flash", "animation", "motion", "glow"]
            .map(str::to_owned)
            .to_vec(),
        hidden: false,
        app_only: false,
        actions: vec![
            Action::new(TOGGLE, "Turn the focus stutter on or off"),
            Action::new(ON, "Turn on"),
            Action::new(OFF, "Turn off"),
        ],
    }]
}

/// Every id of an Item this module sends, which no action of the registry may take.
pub fn ids() -> [&'static str; 1] {
    [STUTTER]
}

/// Runs `action` on the Item `id`, writing `file`; the error is what the user reads.
///
/// # Errors
///
/// When there is no such Item or Action, or the file can't be read, isn't JSON or can't be
/// written.
pub fn run(id: &str, action: &str, file: &Path) -> Result<Ran, String> {
    if id != STUTTER {
        return Err(format!("`{id}` isn't a shell setting"));
    }
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(Chain(&SettingsError::Read(err)).to_string()),
    };
    let on = match action {
        ON => true,
        OFF => false,
        TOGGLE => !stutter(&text).unwrap_or(DEFAULT_STUTTER),
        _ => return Err(format!("`{id}` has no Action `{action}`")),
    };
    let set = with(&text, STUTTER_KEY, on).map_err(|err| Chain(&err).to_string())?;
    write(file, &set).map_err(|err| Chain(&err).to_string())?;
    Ok(Ran::Close)
}

fn write(file: &Path, text: &str) -> Result<(), SettingsError> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(SettingsError::Write)?;
    }
    // HAZARD: written in place, not through a rename: the shell watches this path, and a rename
    // would replace the file it watches.
    fs::write(file, text).map_err(SettingsError::Write)
}

/// The focus stutter as `text` sets it, or `None` where it doesn't, or doesn't read.
pub fn stutter(text: &str) -> Option<bool> {
    let value: serde_json::Value = serde_json::from_str(&uncommented(text)).ok()?;
    value.get(STUTTER_KEY)?.as_bool()
}

/// `text` with the boolean `key` set to `value`: the key's line changed in place when it is set,
/// else a line of its own after the opening brace. Every comment and other key stays.
///
/// # Errors
///
/// When `text`, or what it would become, isn't JSON with whole-line comments, or the edit would
/// change another key: the key shares a line with another.
pub fn with(text: &str, key: &str, value: bool) -> Result<String, SettingsError> {
    let mut wanted = if text.trim().is_empty() {
        serde_json::Map::new()
    } else {
        match serde_json::from_str(&uncommented(text)).map_err(SettingsError::Invalid)? {
            serde_json::Value::Object(map) => map,
            _ => return Err(SettingsError::Crowded),
        }
    };
    wanted.insert(key.to_owned(), serde_json::Value::Bool(value));
    let quoted = format!("\"{key}\"");
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let open = lines.iter().enumerate().find_map(|(at, line)| {
        let (before, after) = line.split_once('{').filter(|_| !is_comment(line))?;
        Some((at, before.to_owned(), after.trim().to_owned()))
    });
    let set = lines
        .iter_mut()
        .find(|line| !is_comment(line) && line.trim_start().starts_with(&quoted));
    if let Some(line) = set {
        let indent = line.len() - line.trim_start().len();
        let comma = if line.trim_end().ends_with(',') { "," } else { "" };
        *line = format!("{}{quoted}: {value}{comma}", &line[..indent]);
    } else if let Some((open, before, after)) = open {
        let others = !after.trim_start_matches('}').trim().is_empty()
            || lines
                .iter()
                .skip(open + 1)
                .any(|line| !is_comment(line) && line.contains('"'));
        let comma = if others { "," } else { "" };
        let mut replaced = vec![format!("{before}{{"), format!("  {quoted}: {value}{comma}")];
        if !after.is_empty() {
            replaced.push(format!("  {after}"));
        }
        lines.splice(open..=open, replaced);
    } else {
        lines.extend(["{".to_owned(), format!("  {quoted}: {value}"), "}".to_owned()]);
    }
    let mut out = lines.join("\n");
    out.push('\n');
    let got: serde_json::Value = serde_json::from_str(&uncommented(&out)).map_err(SettingsError::Invalid)?;
    // A line holding the key and another, or the key twice, edits the wrong thing: the result must
    // be every key as it was with this one set, or the file stays as it is.
    if got != serde_json::Value::Object(wanted) {
        return Err(SettingsError::Crowded);
    }
    Ok(out)
}

fn kind() -> ItemKind {
    ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid"))
}

fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// `text` without its whole-line comments, as the shell reads it.
fn uncommented(text: &str) -> String {
    text.lines()
        .filter(|line| !is_comment(line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
