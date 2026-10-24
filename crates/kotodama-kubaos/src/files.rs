//! The user folders in the menu: one Item each, "Files: Downloads", that opens it in Files
//! (Nautilus), or in yazi as its second Action. The folders are the XDG user folders
//! `xdg-user-dirs` writes to `~/.config/user-dirs.dirs`, or their English names without the file,
//! and only the ones that exist.
//!
//! # Owns
//!
//! - Reading `user-dirs.dirs`, the folder Items, and what their Action runs.
//!
//! # Not here
//!
//! - "Files", "Yazi", "Open folder…" and "Recent files", which are actions in
//!   `config/kotodama/kubaos.toml`.
//! - Opening yazi: `crates/kubaos-files`.

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

/// The Item kind of every folder Item.
pub const KIND: &str = "kubaos.files";
/// Opens the folder in Files.
pub const OPEN: &str = "open";
/// Opens the folder in yazi.
pub const YAZI: &str = "yazi";
/// A folder's Item is this and its key in lower case, `files:download`.
const PREFIX: &str = "files:";
/// What opens a folder in Files, a window of its own each time, as `SUPER + SHIFT + F` does.
const FILE_MANAGER: [&str; 2] = ["nautilus", "--new-window"];
/// What opens a folder in yazi, in a terminal window: `crates/kubaos-files`.
const YAZI_OPENER: [&str; 2] = ["kubaos-files", "open"];
/// The file `xdg-user-dirs` writes, under the config folder.
const USER_DIRS: &str = "user-dirs.dirs";

const ICON: &str = "folder";

/// Each user folder's key in `user-dirs.dirs` (`XDG_<key>_DIR`) and its English name, which it
/// has without the file. In the order the Items are sent.
const KNOWN: [(&str, &str); 8] = [
    ("DOWNLOAD", "Downloads"),
    ("DOCUMENTS", "Documents"),
    ("PICTURES", "Pictures"),
    ("VIDEOS", "Videos"),
    ("MUSIC", "Music"),
    ("DESKTOP", "Desktop"),
    ("TEMPLATES", "Templates"),
    ("PUBLICSHARE", "Public"),
];

/// The user folders on this machine.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserFolders {
    home: PathBuf,
    folders: Vec<Folder>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Folder {
    /// Its key from [`KNOWN`].
    key: &'static str,
    english: &'static str,
    /// Its path, which is UTF-8: the launcher takes text.
    path: String,
}

impl UserFolders {
    /// Reads `user-dirs.dirs` in `config` and keeps the folders under `home` that exist.
    pub fn read(home: &Path, config: &Path) -> Self {
        let text = fs::read_to_string(config.join(USER_DIRS)).ok();
        Self::parse(home, text.as_deref(), Path::is_dir)
    }

    /// The user folders from `HOME` and `XDG_CONFIG_HOME` (else `~/.config`); none without a home.
    pub fn from_env() -> Self {
        let var = |name| {
            std::env::var_os(name)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        };
        let Some(home) = var("HOME") else {
            return UserFolders::default();
        };
        let config = var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
        Self::read(&home, &config)
    }

    /// The folders `user-dirs.dirs`'s `text` names (the English ones without it) that `exists`
    /// says are there. A folder set to the home itself, as `xdg-user-dirs` turns one off, is left
    /// out.
    pub fn parse(home: &Path, text: Option<&str>, exists: impl Fn(&Path) -> bool) -> Self {
        let folders = KNOWN
            .into_iter()
            .filter_map(|(key, english)| {
                let path = match text {
                    Some(text) => configured(text, key, home)?,
                    None => home.join(english),
                };
                if path == home || !exists(&path) {
                    return None;
                }
                Some(Folder {
                    key,
                    english,
                    path: path.into_os_string().into_string().ok()?,
                })
            })
            .collect();
        UserFolders {
            home: home.to_path_buf(),
            folders,
        }
    }

    /// One Item a folder.
    pub fn items(&self) -> Vec<Item> {
        self.folders.iter().map(|folder| self.item(folder)).collect()
    }

    /// Runs `action` on the folder Item `id`; the error is what the user reads.
    ///
    /// # Errors
    ///
    /// When there is no such Item or Action, or Nautilus or `kubaos-files` can't be started.
    pub fn run(&self, id: &str, action: &str, runner: &impl Runner) -> Result<Ran, String> {
        let unknown = || format!("`{id}` has no Action `{action}`");
        let opener = match action {
            OPEN => FILE_MANAGER,
            YAZI => YAZI_OPENER,
            _ => return Err(unknown()),
        };
        let folder = self
            .folders
            .iter()
            .find(|folder| item_id(folder) == id)
            .ok_or_else(unknown)?;
        let argv = [opener[0], opener[1], &folder.path].map(str::to_owned);
        runner
            .launch(&argv, Duration::ZERO)
            .map_err(|err| Chain(&err).to_string())?;
        Ok(Ran::Close)
    }

    fn item(&self, folder: &Folder) -> Item {
        let shown = Path::new(&folder.path)
            .strip_prefix(&self.home)
            .map_or_else(|_| folder.path.clone(), |rest| format!("~/{}", rest.display()));
        let name = Path::new(&folder.path)
            .file_name()
            .map_or_else(|| folder.english.to_owned(), |name| name.to_string_lossy().into_owned());
        Item {
            id: item_id(folder),
            kind: ItemKind::Namespaced(NamespacedKind::new(KIND).expect("the Extension's kinds are valid")),
            identity: None,
            title: format!("Files: {name}"),
            subtitle: Some(shown),
            icon: Some(ICON.to_owned()),
            keyword: None,
            keywords: [folder.english, "folder", "file manager", "nautilus", "yazi"]
                .map(str::to_owned)
                .to_vec(),
            hidden: false,
            app_only: false,
            actions: vec![Action::new(OPEN, "Open"), Action::new(YAZI, "Open in yazi")],
        }
    }
}

fn item_id(folder: &Folder) -> String {
    format!("{PREFIX}{}", folder.key.to_lowercase())
}

/// The path `text` gives `XDG_<key>_DIR`: `"$HOME/…"` or an absolute path, in double quotes, as
/// `xdg-user-dirs` writes it. The last line for the key wins, as when the shell sources the file.
fn configured(text: &str, key: &str, home: &Path) -> Option<PathBuf> {
    let name = format!("XDG_{key}_DIR");
    let value = text
        .lines()
        .rev()
        .filter_map(|line| line.trim().split_once('='))
        .filter(|(var, _)| *var == name)
        .map(|(_, value)| value.trim().trim_matches('"'))
        .next()?;
    if let Some(rest) = value.strip_prefix("$HOME") {
        return Some(home.join(rest.trim_start_matches('/')));
    }
    value.starts_with('/').then(|| PathBuf::from(value))
}

#[cfg(test)]
mod tests;
