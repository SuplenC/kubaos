//! The wallpaper-depth settings, in the user layer:
//! `~/.config/kubaos/desktop/wallpapers/depth.toml`.
//!
//! ```toml
//! enabled = true
//! push = 0.025
//! skip = [
//!   "/home/kuba/Pictures/wallpapers/harbour.jpg",
//! ]
//! ```
//!
//! `enabled` is depth for the user's own pictures, `push` how far the wallpaper comes toward you
//! in the lock motion, and `skip` the pictures that never get a cut-out, by the path the theme
//! records them by. The skip list lives here rather than beside the results, because
//! `kubaos suite remove --purge-caches` deletes those. `kubaos depth` changes the file, editing
//! only the value it changes, so the user's comments and the rest of the file stay as they are.
//!
//! # Owns
//!
//! - The settings, their defaults and bounds, reading the file and changing a value in it.
//! - Which pictures the wallpaper-depth Suite cuts out ([`Settings::refusal`]), and the current
//!   wallpaper as the theme records it ([`current_wallpaper`]).
//!
//! # Not here
//!
//! - Queuing and stopping the runs: `kubaos depth` in the CLI, which runs the Suite's
//!   `kubaos-depth-queue` (`home/suites/wallpaper-depth-queue.sh`). The seed with its comments:
//!   `user-layer/desktop/wallpapers/depth.toml`. The cut-outs themselves: `crates/kubaos-depth`.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use kubaos_user_layer::App;
use serde::Deserialize;
use toml_edit::Array;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Value;

/// Depth is on for the user's own pictures as soon as the Suite is installed: installing it is
/// the choice.
pub const DEFAULT_ENABLED: bool = true;
/// The share the picture grows by in the lock motion: the Lock page's `scale.push`, 1.025, a
/// gentle dolly the mockups were approved with.
pub const DEFAULT_PUSH: f64 = 0.025;
/// The smallest push: the picture holds still.
pub const MIN_PUSH: f64 = 0.0;
/// The largest push, twice the default; the cut-out's lead over the picture grows with it, and so
/// does the band of the background plate the city uncovers.
pub const MAX_PUSH: f64 = 0.05;

/// The settings file in the wallpapers' folder of the user layer.
pub const FILE: &str = "depth.toml";

/// The theme's record of the current wallpaper under the home, which `scripts/dot-theme` writes.
const WALLPAPER_FILE: &str = ".local/state/theme/wallpaper";

/// The picture formats `scripts/dot-theme` treats as a video (`VIDEO_EXTS`): a cut-out of one
/// frame doesn't hold over a moving picture.
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "mkv", "mov"];

const ENABLED: &str = "enabled";
const PUSH: &str = "push";
const SKIP: &str = "skip";

/// The settings file for the config home `config_home`, usually `~/.config`.
pub fn path(config_home: &Path) -> PathBuf {
    kubaos_user_layer::dir(config_home, App::Wallpapers).join(FILE)
}

/// The file the theme records the current wallpaper in, for the home `home`.
pub fn wallpaper_file(home: &Path) -> PathBuf {
    home.join(WALLPAPER_FILE)
}

/// The current wallpaper as the theme records it in `file` (see [`wallpaper_file`]): `None` while
/// none is set.
///
/// # Errors
///
/// When the file exists but can't be read.
pub fn current_wallpaper(file: &Path) -> io::Result<Option<PathBuf>> {
    match fs::read_to_string(file) {
        Ok(text) => Ok(text.lines().next().filter(|line| !line.is_empty()).map(PathBuf::from)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// The wallpaper-depth settings.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Whether the Suite cuts out the user's own pictures, and the lock draws those cut-outs.
    pub enabled: bool,
    /// The share the picture grows by in the lock motion, [`MIN_PUSH`] to [`MAX_PUSH`].
    pub push: f64,
    /// The pictures that never get a cut-out, shipped or the user's own.
    pub skip: Vec<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            enabled: DEFAULT_ENABLED,
            push: DEFAULT_PUSH,
            skip: Vec::new(),
        }
    }
}

/// Why the settings file couldn't be read or changed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SettingsError {
    #[error("reading the wallpaper-depth settings")]
    Read(#[source] io::Error),
    #[error("parsing the wallpaper-depth settings")]
    Parse(#[source] toml::de::Error),
    #[error("editing the wallpaper-depth settings")]
    Edit(#[source] toml_edit::TomlError),
    #[error("push {0} isn't a share from {MIN_PUSH} to {MAX_PUSH}")]
    Push(f64),
    #[error("writing the wallpaper-depth settings")]
    Write(#[source] io::Error),
}

/// Why the Suite doesn't cut a picture out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Depth for the user's own pictures is off.
    Off,
    /// The picture is in the skip list.
    Skipped,
    /// The picture is a video.
    Video,
    /// The picture is one of KubaOS's sets, which ship their depth beside it.
    Shipped,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Refusal::Off => "wallpaper depth is off",
            Refusal::Skipped => "the picture has no cut-out by choice",
            Refusal::Video => "a video gets no cut-out",
            Refusal::Shipped => "KubaOS's wallpapers ship their own depth",
        })
    }
}

impl Settings {
    /// Parses the settings file's text; a key left out takes its default.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Parse`] for malformed TOML, an unknown key or a value of the wrong type,
    /// and [`SettingsError::Push`] for a push outside [`MIN_PUSH`]..=[`MAX_PUSH`].
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let settings: Self = toml::from_str(text).map_err(SettingsError::Parse)?;
        check_push(settings.push)?;
        Ok(settings)
    }

    /// Reads the settings file; a missing file is the defaults.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Read`] when the file exists but can't be read, else as in
    /// [`Settings::parse`].
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        match fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(SettingsError::Read(e)),
        }
    }

    /// Whether `picture` is in the skip list.
    pub fn skips(&self, picture: &Path) -> bool {
        self.skip.iter().any(|skipped| skipped == picture)
    }

    /// Why the Suite doesn't cut `picture` out, `None` when it does. `sets` is the folder of
    /// KubaOS's sets, `/usr/share/kubaos/wallpapers`.
    pub fn refusal(&self, picture: &Path, sets: &Path) -> Option<Refusal> {
        if self.skips(picture) {
            Some(Refusal::Skipped)
        } else if picture.starts_with(sets) {
            Some(Refusal::Shipped)
        } else if is_video(picture) {
            Some(Refusal::Video)
        } else if !self.enabled {
            Some(Refusal::Off)
        } else {
            None
        }
    }
}

fn is_video(picture: &Path) -> bool {
    picture
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| VIDEO_EXTENSIONS.iter().any(|video| ext.eq_ignore_ascii_case(video)))
}

fn check_push(push: f64) -> Result<(), SettingsError> {
    if (MIN_PUSH..=MAX_PUSH).contains(&push) {
        Ok(())
    } else {
        Err(SettingsError::Push(push))
    }
}

/// Turns depth for the user's own pictures on or off in the settings file at `path`, creating it
/// when missing.
///
/// # Errors
///
/// [`SettingsError::Read`] or [`SettingsError::Write`] when the file can't be read or replaced,
/// and [`SettingsError::Edit`] when it isn't TOML.
pub fn set_enabled(path: &Path, enabled: bool) -> Result<(), SettingsError> {
    edit(path, |doc| {
        set(doc, ENABLED, Value::from(enabled));
        true
    })
    .map(|_| ())
}

/// Sets the push in the settings file at `path`, creating it when missing.
///
/// # Errors
///
/// [`SettingsError::Push`] for a push outside [`MIN_PUSH`]..=[`MAX_PUSH`], else as in
/// [`set_enabled`].
pub fn set_push(path: &Path, push: f64) -> Result<(), SettingsError> {
    check_push(push)?;
    edit(path, |doc| {
        set(doc, PUSH, Value::from(push));
        true
    })
    .map(|_| ())
}

/// Adds `picture` to the skip list in the settings file at `path`; whether it wasn't there.
///
/// # Errors
///
/// As in [`set_enabled`].
pub fn skip(path: &Path, picture: &Path) -> Result<bool, SettingsError> {
    edit(path, |doc| {
        let mut list = skip_list(doc);
        if list.iter().any(|skipped| Path::new(skipped) == picture) {
            return false;
        }
        list.push(picture.to_string_lossy().into_owned());
        set(doc, SKIP, Value::Array(array(&list)));
        true
    })
}

/// Takes `picture` out of the skip list in the settings file at `path`; whether it was there.
///
/// # Errors
///
/// As in [`set_enabled`].
pub fn unskip(path: &Path, picture: &Path) -> Result<bool, SettingsError> {
    edit(path, |doc| {
        let list = skip_list(doc);
        let kept: Vec<String> = list
            .iter()
            .filter(|skipped| Path::new(skipped) != picture)
            .cloned()
            .collect();
        if kept.len() == list.len() {
            return false;
        }
        set(doc, SKIP, Value::Array(array(&kept)));
        true
    })
}

/// The skip list's paths in `doc`, empty without the key.
fn skip_list(doc: &DocumentMut) -> Vec<String> {
    doc.get(SKIP)
        .and_then(Item::as_array)
        .map(|list| list.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
        .unwrap_or_default()
}

/// `paths` as a TOML array, one a line, so a long list reads as a list.
fn array(paths: &[String]) -> Array {
    let mut array: Array = paths.iter().map(String::as_str).collect();
    if array.is_empty() {
        return array;
    }
    for value in array.iter_mut() {
        value.decor_mut().set_prefix("\n  ");
    }
    array.set_trailing_comma(true);
    array.set_trailing("\n");
    array
}

/// Sets `key` to `value` in `doc`. An existing key keeps its place and the comments above it; a
/// new one goes at the end of the file, after the comments there, as an appended line would.
fn set(doc: &mut DocumentMut, key: &str, value: Value) {
    if let Some(item) = doc.get_mut(key) {
        let decor = item.as_value().map(|old| old.decor().clone());
        let mut value = value;
        if let Some(decor) = decor {
            *value.decor_mut() = decor;
        }
        *item = Item::Value(value);
        return;
    }
    let trailing = doc.trailing().as_str().unwrap_or_default().to_owned();
    doc.insert(key, Item::Value(value));
    if let Some(mut new) = doc.key_mut(key) {
        new.leaf_decor_mut().set_prefix(trailing);
    }
    doc.set_trailing("");
}

/// Reads the file at `path` (empty when missing), lets `change` edit it, and writes it back
/// through a temporary file and a rename when `change` says it changed; returns what it said.
fn edit(path: &Path, change: impl FnOnce(&mut DocumentMut) -> bool) -> Result<bool, SettingsError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(SettingsError::Read(e)),
    };
    let mut doc: DocumentMut = text.parse().map_err(SettingsError::Edit)?;
    if !change(&mut doc) {
        return Ok(false);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(SettingsError::Write)?;
    }
    let tmp = path.with_extension("toml.new");
    fs::write(&tmp, doc.to_string()).map_err(SettingsError::Write)?;
    fs::rename(&tmp, path).map_err(SettingsError::Write)?;
    Ok(true)
}

#[cfg(test)]
mod tests;
