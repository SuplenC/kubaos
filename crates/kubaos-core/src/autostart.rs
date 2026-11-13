//! The startup apps: the XDG autostart entries the session starts at login.
//!
//! An entry is a desktop file in `~/.config/autostart` or in `autostart/` under a system config
//! folder (`/etc/xdg/autostart`). Entries are known by their file name, the app's desktop id
//! (`solaar.desktop`), and the user's entry replaces a system one of the same name, as the XDG
//! Autostart spec says. An entry starts unless it is `Hidden`, turned off with
//! `X-GNOME-Autostart-enabled=false`, left to a unit with `X-GNOME-HiddenUnderSystemd`, kept from
//! this desktop by `OnlyShowIn` or `NotShowIn`, or its `TryExec` isn't there: the rules
//! systemd-xdg-autostart-generator follows when it writes each one's unit.
//!
//! Turning an app on writes its entry to `~/.config/autostart`, from the app's own desktop file, or
//! removes the user's entry when that was what kept a system one from starting. Turning it off
//! removes the user's entry, or writes one with `Hidden=true` over a system entry, which the spec
//! has for it.
//!
//! <https://specifications.freedesktop.org/autostart-spec/latest/>
//!
//! # Owns
//!
//! - Where the entries and the apps' desktop files are, which entries start, and turning an app's
//!   start on login on and off.
//!
//! # Not here
//!
//! - Starting them at login: `config/hypr/autostart.lua` starts `xdg-desktop-autostart.target`.
//! - Reloading systemd's units after a change: `kubaos autostart` (`crates/kubaos-cli`).
//! - The Command menu's Actions: `crates/kotodama-kubaos`.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::state::write_atomic;

/// The suffix of every desktop id.
const SUFFIX: &str = ".desktop";
/// The group of a desktop file that holds the entry's keys.
const GROUP: &str = "[Desktop Entry]";

/// The folders the entries and the apps are read from, and what the session is, which decide
/// what starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dirs {
    /// The user's entries, `~/.config/autostart`, where every change is written.
    pub user: PathBuf,
    /// The system's entries, `/etc/xdg/autostart` and the others of `XDG_CONFIG_DIRS`, the first
    /// the most important.
    pub system: Vec<PathBuf>,
    /// The apps' desktop files, `~/.local/share/applications` first, then each of
    /// `XDG_DATA_DIRS`.
    pub applications: Vec<PathBuf>,
    /// The session's desktop names, from `XDG_CURRENT_DESKTOP`, for `OnlyShowIn` and `NotShowIn`.
    pub desktops: Vec<String>,
    /// The folders a bare `TryExec` is looked for in, from `PATH`.
    pub path: Vec<PathBuf>,
}

/// An app that starts on login.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartupApp {
    /// Its desktop id, the entry's file name: `solaar.desktop`.
    pub id: String,
    /// The entry's name, or the id without `.desktop` when it has none.
    pub name: String,
    /// The entry's file, the user's or a system one.
    pub file: PathBuf,
}

/// Why turning an app's start on login on or off failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AutostartError {
    /// The id isn't a desktop id: a file name ending in `.desktop`.
    #[error("`{0}` isn't an app's desktop id, such as `solaar.desktop`")]
    InvalidId(String),
    /// There is no app with this id to start, and no entry of its own.
    #[error("there's no app `{0}` to start on login")]
    NoApp(String),
    /// Writing the user's entry failed.
    #[error("writing {}", .0.display())]
    Write(PathBuf, #[source] io::Error),
    /// Removing the user's entry failed.
    #[error("removing {}", .0.display())]
    Remove(PathBuf, #[source] io::Error),
}

impl Dirs {
    /// The folders as the environment names them, with the XDG base directory defaults; none
    /// without a home.
    pub fn from_env() -> Option<Self> {
        let home = env::var_os("HOME").filter(|home| !home.is_empty()).map(PathBuf::from)?;
        let var = |name: &str| env::var(name).ok().filter(|value| !value.is_empty());
        let config_home = var("XDG_CONFIG_HOME").map_or_else(|| home.join(".config"), PathBuf::from);
        let data_home = var("XDG_DATA_HOME").map_or_else(|| home.join(".local/share"), PathBuf::from);
        let list = |name: &str, default: &str| -> Vec<PathBuf> {
            var(name)
                .unwrap_or_else(|| default.to_owned())
                .split(':')
                .filter(|dir| !dir.is_empty())
                .map(PathBuf::from)
                .collect()
        };
        let applications = std::iter::once(data_home)
            .chain(list("XDG_DATA_DIRS", "/usr/local/share:/usr/share"))
            .map(|dir| dir.join("applications"))
            .collect();
        Some(Dirs {
            user: config_home.join("autostart"),
            system: list("XDG_CONFIG_DIRS", "/etc/xdg")
                .into_iter()
                .map(|dir| dir.join("autostart"))
                .collect(),
            applications,
            desktops: var("XDG_CURRENT_DESKTOP")
                .map(|desktops| desktops.split(':').map(str::to_owned).collect())
                .unwrap_or_default(),
            path: env::var_os("PATH")
                .map(|path| env::split_paths(&path).collect())
                .unwrap_or_default(),
        })
    }

    /// Every app that starts on login, by name.
    pub fn startup_apps(&self) -> Vec<StartupApp> {
        let mut ids: Vec<String> = std::iter::once(&self.user)
            .chain(&self.system)
            .flat_map(|dir| desktop_ids(dir))
            .collect();
        ids.sort();
        ids.dedup();
        let mut apps: Vec<StartupApp> = ids
            .into_iter()
            .filter_map(|id| {
                let (file, entry) = self.entry(&id)?;
                self.starts_entry(&entry).then(|| StartupApp {
                    name: entry.get("Name").unwrap_or(id.trim_end_matches(SUFFIX)).to_owned(),
                    id,
                    file,
                })
            })
            .collect();
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.id.cmp(&b.id)));
        apps
    }

    /// Whether the app `id` starts on login.
    pub fn starts(&self, id: &str) -> bool {
        self.entry(id).is_some_and(|(_, entry)| self.starts_entry(&entry))
    }

    /// The desktop ids of the apps a launcher shows, in no order: a desktop file of each id, the
    /// first in [`Dirs::applications`], that is an Application, neither `Hidden` nor `NoDisplay`,
    /// and shown on this desktop.
    pub fn apps(&self) -> Vec<String> {
        let mut seen: BTreeMap<String, bool> = BTreeMap::new();
        for dir in &self.applications {
            for id in desktop_ids(dir) {
                if seen.contains_key(&id) {
                    continue;
                }
                let shown = Entry::read(&dir.join(&id)).is_some_and(|entry| {
                    entry.get("Type") == Some("Application")
                        && !entry.is_true("Hidden")
                        && !entry.is_true("NoDisplay")
                        && entry.shown_in(&self.desktops)
                });
                seen.insert(id, shown);
            }
        }
        seen.into_iter().filter(|(_, shown)| *shown).map(|(id, _)| id).collect()
    }

    /// Makes the app `id` start on login. An app that starts already is left alone.
    ///
    /// # Errors
    ///
    /// When `id` isn't a desktop id, there is no app of that id and no system entry that would
    /// start, or the user's entry can't be written or removed.
    pub fn turn_on(&self, id: &str) -> Result<(), AutostartError> {
        check_id(id)?;
        if self.starts(id) {
            return Ok(());
        }
        let mine = self.user.join(id);
        if self.system_entry(id).is_some_and(|entry| self.starts_entry(&entry)) {
            return fs::remove_file(&mine).map_err(|err| AutostartError::Remove(mine, err));
        }
        let app = self.app(id).ok_or_else(|| AutostartError::NoApp(id.to_owned()))?;
        write(&mine, &app.autostart_entry())
    }

    /// Keeps the app `id` from starting on login. An app that doesn't start is left alone.
    ///
    /// # Errors
    ///
    /// When `id` isn't a desktop id, or the user's entry can't be written or removed.
    pub fn turn_off(&self, id: &str) -> Result<(), AutostartError> {
        check_id(id)?;
        if !self.starts(id) {
            return Ok(());
        }
        let mine = self.user.join(id);
        match self.system_entry(id) {
            Some(system) if self.starts_entry(&system) => write(&mine, &system.hidden_entry()),
            _ => fs::remove_file(&mine).map_err(|err| AutostartError::Remove(mine, err)),
        }
    }

    /// The entry of `id` that counts, the user's or else the first system one, and its file.
    fn entry(&self, id: &str) -> Option<(PathBuf, Entry)> {
        std::iter::once(&self.user)
            .chain(&self.system)
            .map(|dir| dir.join(id))
            .find_map(|file| Entry::read(&file).map(|entry| (file, entry)))
    }

    /// The first system entry of `id`, as it would count without the user's.
    fn system_entry(&self, id: &str) -> Option<Entry> {
        self.system.iter().find_map(|dir| Entry::read(&dir.join(id)))
    }

    /// The app's own desktop file of `id`, the first in [`Dirs::applications`].
    fn app(&self, id: &str) -> Option<Entry> {
        self.applications.iter().find_map(|dir| Entry::read(&dir.join(id)))
    }

    /// Whether `entry` starts, as systemd-xdg-autostart-generator decides it.
    fn starts_entry(&self, entry: &Entry) -> bool {
        entry.get("Exec").is_some_and(|exec| !exec.trim().is_empty())
            && !entry.is_true("Hidden")
            && entry.get("X-GNOME-Autostart-enabled") != Some("false")
            && !entry.is_true("X-GNOME-HiddenUnderSystemd")
            && entry.shown_in(&self.desktops)
            && entry.get("TryExec").is_none_or(|program| self.finds(program))
    }

    /// Whether `program`, a path or a name looked for in [`Dirs::path`], is there.
    fn finds(&self, program: &str) -> bool {
        let program = Path::new(program);
        if program.is_absolute() {
            return program.is_file();
        }
        self.path.iter().any(|dir| dir.join(program).is_file())
    }
}

/// The keys of a desktop file's `[Desktop Entry]` group, as written: values keep their escapes,
/// and a localised key (`Name[pl]`) is a key of its own.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Entry {
    keys: HashMap<String, String>,
}

impl Entry {
    /// The entry in the desktop file `file`; none when it can't be read.
    fn read(file: &Path) -> Option<Self> {
        fs::read_to_string(file).ok().map(|text| Self::parse(&text))
    }

    fn parse(text: &str) -> Self {
        let mut keys = HashMap::new();
        let mut inside = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                inside = line == GROUP;
            } else if inside
                && !line.starts_with('#')
                && let Some((key, value)) = line.split_once('=')
            {
                keys.entry(key.trim().to_owned())
                    .or_insert_with(|| value.trim().to_owned());
            }
        }
        Entry { keys }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.keys.get(key).map(String::as_str)
    }

    fn is_true(&self, key: &str) -> bool {
        self.get(key) == Some("true")
    }

    /// Whether `OnlyShowIn` and `NotShowIn` let it show on a session of `desktops`.
    fn shown_in(&self, desktops: &[String]) -> bool {
        let names = |key: &str| -> Option<Vec<&str>> {
            self.get(key)
                .map(|list| list.split(';').filter(|name| !name.is_empty()).collect())
        };
        let ours = |names: &[&str]| desktops.iter().any(|desktop| names.contains(&desktop.as_str()));
        if let Some(only) = names("OnlyShowIn")
            && !ours(&only)
        {
            return false;
        }
        names("NotShowIn").is_none_or(|not| !ours(&not))
    }

    /// The autostart entry that starts this app: its name, icon and what it runs, and nothing a
    /// launcher alone reads, such as its MIME types and its own Actions.
    fn autostart_entry(&self) -> String {
        const KEPT: [&str; 6] = ["Name", "Icon", "TryExec", "Exec", "Path", "Terminal"];
        let kept: String = KEPT
            .into_iter()
            .filter_map(|key| self.get(key).map(|value| format!("{key}={value}\n")))
            .collect();
        format!("{GROUP}\nType=Application\n{kept}")
    }

    /// The user's entry that keeps this system entry from starting.
    fn hidden_entry(&self) -> String {
        let name = self
            .get("Name")
            .map(|name| format!("Name={name}\n"))
            .unwrap_or_default();
        format!("{GROUP}\nType=Application\n{name}Hidden=true\n")
    }
}

/// The desktop ids in `dir`: its files named `*.desktop`; none when it can't be read.
fn desktop_ids(dir: &Path) -> Vec<String> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    read.filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| is_desktop_id(name))
        .collect()
}

fn is_desktop_id(id: &str) -> bool {
    id.len() > SUFFIX.len() && id.ends_with(SUFFIX) && !id.starts_with('.') && !id.contains('/')
}

fn check_id(id: &str) -> Result<(), AutostartError> {
    if is_desktop_id(id) {
        Ok(())
    } else {
        Err(AutostartError::InvalidId(id.to_owned()))
    }
}

fn write(file: &Path, text: &str) -> Result<(), AutostartError> {
    file.parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| write_atomic(file, text))
        .map_err(|err| AutostartError::Write(file.to_owned(), err))
}

#[cfg(test)]
mod tests;
