//! Tidying Downloads: each file at its top goes into a folder for its kind, Images, Documents,
//! Archives, Installers, Video or Audio, once it is a day old. A newer file stays where the
//! browser put it, as it is likely still being opened from there; a file of no known kind, a
//! hidden one and a folder stay too.
//!
//! It runs from the Command menu, and daily when the setting is on: the file
//! `~/.config/kubaos/desktop/files/tidy-downloads` exists. Off by default.
//!
//! # Owns
//!
//! - The kinds and their extensions, which files move and where, and the setting.
//!
//! # Not here
//!
//! - The daily timer: `home/files.nix`. The menu's Items: `config/kotodama/kubaos.toml`.

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use kubaos_user_layer::App;

/// How old a file must be to move: a day, so today's downloads stay at hand.
pub const DEFAULT_MIN_AGE: Duration = Duration::from_hours(24);

/// The file that turns the daily tidy on, in the Files folder of the user layer.
const ON: &str = "tidy-downloads";

/// A kind of file, and the folder it goes into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Images,
    Documents,
    Archives,
    Installers,
    Video,
    Audio,
}

impl Kind {
    pub const ALL: [Kind; 6] = [
        Kind::Images,
        Kind::Documents,
        Kind::Archives,
        Kind::Installers,
        Kind::Video,
        Kind::Audio,
    ];

    /// The folder's name in Downloads.
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Images => "Images",
            Kind::Documents => "Documents",
            Kind::Archives => "Archives",
            Kind::Installers => "Installers",
            Kind::Video => "Video",
            Kind::Audio => "Audio",
        }
    }

    /// The extensions of this kind, in lower case.
    fn extensions(self) -> &'static [&'static str] {
        match self {
            Kind::Images => &[
                "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "avif", "bmp", "tif", "tiff", "svg", "ico",
                "dng", "cr2", "nef", "arw", "raw",
            ],
            Kind::Documents => &[
                "pdf", "doc", "docx", "odt", "rtf", "txt", "md", "xls", "xlsx", "ods", "csv", "ppt", "pptx", "odp",
                "epub",
            ],
            Kind::Archives => &["zip", "7z", "rar", "tar", "gz", "tgz", "bz2", "tbz2", "xz", "txz", "zst", "lz"],
            Kind::Installers => &[
                "deb", "rpm", "appimage", "flatpakref", "snap", "run", "msi", "exe", "dmg", "pkg", "iso",
            ],
            Kind::Video => &["mp4", "mkv", "webm", "mov", "avi", "m4v", "wmv", "flv"],
            Kind::Audio => &["mp3", "flac", "ogg", "opus", "wav", "m4a", "aac", "wma"],
        }
    }
}

/// The kind of the file `name`, by its extension; none for one of no known kind. A partial
/// download (`.crdownload`, `.part`) has none, so it never moves while it is being written.
pub fn kind(name: &OsStr) -> Option<Kind> {
    let extension = Path::new(name).extension()?.to_str()?;
    Kind::ALL
        .into_iter()
        .find(|kind| kind.extensions().iter().any(|known| known.eq_ignore_ascii_case(extension)))
}

/// A file at the top of Downloads, as [`plan`] reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: OsString,
    /// Whether it is a regular file, not a folder or a link.
    pub file: bool,
    /// How long since it arrived: its newest change, so a file a browser renamed into place
    /// counts from then.
    pub age: Duration,
}

/// Which of `entries` move, each with its kind: the regular files of a known kind, not hidden,
/// at least `min_age` old.
pub fn plan(entries: &[Entry], min_age: Duration) -> Vec<(OsString, Kind)> {
    entries
        .iter()
        .filter(|entry| entry.file && entry.age >= min_age)
        .filter(|entry| !entry.name.as_encoded_bytes().starts_with(b"."))
        .filter_map(|entry| Some((entry.name.clone(), kind(&entry.name)?)))
        .collect()
}

/// Moves the files [`plan`] picks in `dir` into their folders, and returns how many moved. A
/// name taken in the folder becomes `name (1).ext`, `(2)` and so on, so nothing is replaced.
///
/// # Errors
///
/// When `dir` can't be read, or a file can't be moved.
pub fn tidy(dir: &Path, now: SystemTime, min_age: Duration) -> io::Result<usize> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push(Entry {
            name: entry.file_name(),
            file: metadata.file_type().is_file(),
            age: now.duration_since(arrived(&metadata)).unwrap_or_default(),
        });
    }
    let moves = plan(&entries, min_age);
    for (name, kind) in &moves {
        let folder = dir.join(kind.folder());
        fs::create_dir_all(&folder)?;
        fs::rename(dir.join(name), free_name(&folder, name))?;
    }
    Ok(moves.len())
}

/// The newer of a file's modification and status change times.
fn arrived(metadata: &fs::Metadata) -> SystemTime {
    let changed = u64::try_from(metadata.ctime()).map_or(UNIX_EPOCH, |secs| UNIX_EPOCH + Duration::from_secs(secs));
    metadata.modified().map_or(changed, |modified| modified.max(changed))
}

/// `name` in `folder`, or the first `name (n).ext` free there.
fn free_name(folder: &Path, name: &OsStr) -> PathBuf {
    let candidate = folder.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(name);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    let mut n = 1_u64;
    loop {
        let candidate = folder.join(format!("{stem} ({n}){extension}"));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

/// Where the daily tidy's setting is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    on: PathBuf,
}

impl Setting {
    /// The setting under the config folder `config`.
    #[must_use]
    pub fn new(config: impl AsRef<Path>) -> Self {
        Setting {
            on: kubaos_user_layer::dir(config.as_ref(), App::Files).join(ON),
        }
    }

    pub fn daily(&self) -> bool {
        self.on.exists()
    }

    /// Turns the daily tidy on or off.
    ///
    /// # Errors
    ///
    /// When the file can't be made or removed.
    pub fn set_daily(&self, on: bool) -> io::Result<()> {
        if !on {
            return match fs::remove_file(&self.on) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
                other => other,
            };
        }
        if let Some(dir) = self.on.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&self.on, "")
    }
}

/// The Downloads folder: `XDG_DOWNLOAD_DIR` from `user-dirs.dirs` in `config`, else
/// `home/Downloads`.
pub fn downloads_dir(home: &Path, config: &Path) -> PathBuf {
    let text = fs::read_to_string(config.join("user-dirs.dirs")).unwrap_or_default();
    let value = text
        .lines()
        .rev()
        .filter_map(|line| line.trim().split_once('='))
        .find(|(name, _)| *name == "XDG_DOWNLOAD_DIR")
        .map(|(_, value)| value.trim().trim_matches('"'));
    match value {
        Some(value) if value.starts_with("$HOME/") => home.join(&value["$HOME/".len()..]),
        Some(value) if value.starts_with('/') => PathBuf::from(value),
        _ => home.join("Downloads"),
    }
}

#[cfg(test)]
mod tests;
