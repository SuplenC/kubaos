//! The mounted removable drives in the menu: one Item each, "Eject PHOTOS", that unmounts it and
//! powers it down. The list is the drives file the automount service keeps,
//! `$XDG_RUNTIME_DIR/kubaos-drives`; no file, as with the service stopped, means no Items.
//!
//! # Owns
//!
//! - Reading the drives file, the Eject Items, and what their Action runs.
//!
//! # Not here
//!
//! - Writing the file, its format, and ejecting: `crates/kubaos-automount`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;

use crate::run::Runner;
use crate::toggles::Ran;

/// The Item kind of every Eject Item.
pub const KIND: &str = "kubaos.drive";
/// Ejects the drive.
pub const EJECT: &str = "eject";
/// A drive's Item is this and the last part of its udisks2 object path, `drive:SanDisk_Ultra_1234`.
const PREFIX: &str = "drive:";
/// The drives file's name in `$XDG_RUNTIME_DIR` (`kubaos_automount::drives::FILE_NAME`).
pub const FILE_NAME: &str = "kubaos-drives";
/// What ejects a drive and says how it went in a notification.
const EJECTOR: [&str; 2] = ["kubaos-automount", "eject"];

const ICON: &str = "media-eject";

/// The mounted removable drives, in the file's order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Drives {
    drives: Vec<Drive>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Drive {
    /// Its udisks2 object path.
    path: String,
    name: String,
    mounts: Vec<String>,
}

impl Drives {
    /// Reads the drives file `file`; none without it.
    pub fn read(file: &Path) -> Self {
        Self::parse(&fs::read_to_string(file).unwrap_or_default())
    }

    /// The drives in the file's text: a line each, its udisks2 object path, its name and its
    /// mount points, split by tabs. A line without all three is skipped.
    pub fn parse(text: &str) -> Self {
        let drives = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\t');
                let path = fields.next().filter(|path| path.starts_with('/'))?;
                let name = fields.next().filter(|name| !name.is_empty())?;
                let mounts: Vec<String> = fields.filter(|mount| !mount.is_empty()).map(str::to_owned).collect();
                (!mounts.is_empty()).then(|| Drive {
                    path: path.to_owned(),
                    name: name.to_owned(),
                    mounts,
                })
            })
            .collect();
        Drives { drives }
    }

    /// One Item a drive.
    pub fn items(&self) -> Vec<Item> {
        self.drives.iter().map(item).collect()
    }

    /// Runs `action` on the drive Item `id`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, or `kubaos-automount` can't be started.
    pub fn run(&self, id: &str, action: &str, runner: &impl Runner) -> Result<Ran, String> {
        let unknown = || format!("`{id}` has no Action `{action}`");
        if action != EJECT {
            return Err(unknown());
        }
        let drive = self
            .drives
            .iter()
            .find(|drive| item_id(drive) == id)
            .ok_or_else(unknown)?;
        let argv = [EJECTOR[0], EJECTOR[1], &drive.path].map(str::to_owned);
        runner
            .launch(&argv, Duration::ZERO)
            .map_err(|err| Chain(&err).to_string())?;
        Ok(Ran::Close)
    }
}

/// The drives file, `$XDG_RUNTIME_DIR/kubaos-drives`; none when the variable isn't set.
pub fn file() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .filter(|dir| !dir.is_empty())
        .map(|dir| PathBuf::from(dir).join(FILE_NAME))
}

fn item_id(drive: &Drive) -> String {
    let last = drive.path.rsplit('/').next().unwrap_or(&drive.path);
    format!("{PREFIX}{last}")
}

fn item(drive: &Drive) -> Item {
    Item {
        id: item_id(drive),
        kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
        identity: None,
        title: format!("Eject {}", drive.name),
        subtitle: Some(format!("Unmount {} and power it off", drive.mounts.join(", "))),
        icon: Some(ICON.to_owned()),
        keyword: None,
        keywords: ["eject drive", "unmount", "safely remove", "usb", "disk", "card"]
            .map(str::to_owned)
            .to_vec(),
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        yields: false,
        actions: vec![Action::new(EJECT, "Eject")],
    }
}

#[cfg(test)]
mod tests;
