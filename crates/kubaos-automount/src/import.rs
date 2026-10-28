//! Importing photos from a camera, a phone or a card: every photo and video under its `DCIM`
//! folder is copied into `~/Pictures/<year>/<month>/` by the month it was taken. Nothing on the
//! device is changed or deleted.
//!
//! The month comes from the photo's EXIF `DateTimeOriginal` (then `DateTime`), and from the
//! file's modification time for videos and photos without one: cameras set it when they write
//! the file. A file whose name is already in its month folder at the same size counts as
//! imported, so importing the same card twice copies only what's new.
//!
//! # Owns
//!
//! - Finding the `DCIM` folder, the month of each file, where it goes, and copying it.
//!
//! # Not here
//!
//! - Offering the import when a drive is mounted: [`crate::service`].
//! - The notification with the count: `main.rs`.

use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use kotodama_extension::log;

/// The folder cameras keep their photos in (Design rule for Camera File system).
pub const DCIM: &str = "DCIM";

/// The extensions of the files imported, photos and videos, in lower case.
const MEDIA: [&str; 25] = [
    "jpg", "jpeg", "heic", "heif", "avif", "png", "webp", "gif", "dng", "cr2", "cr3", "nef", "nrw", "arw", "raf",
    "orf", "rw2", "pef", "srw", "mp4", "mov", "m4v", "3gp", "mts", "avi",
];

/// The suffix of a file being copied, renamed to its name once whole.
const PARTIAL: &str = ".kubaos-import";

/// A calendar month, the folder a file is imported into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Month {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
}

impl Month {
    /// The folder under the pictures folder: `2024/07`.
    pub fn folder(self) -> PathBuf {
        PathBuf::from(format!("{:04}", self.year)).join(format!("{:02}", self.month))
    }
}

/// How an import went.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub copied: usize,
    /// Files found already imported.
    pub already: usize,
    pub failed: usize,
}

/// Why nothing could be imported.
#[derive(Debug, thiserror::Error)]
#[error("reading {}", path.display())]
pub struct ImportError {
    path: PathBuf,
    #[source]
    source: io::Error,
}

impl ImportError {
    fn read(path: &Path) -> impl FnOnce(io::Error) -> ImportError + '_ {
        move |source| ImportError {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// The `DCIM` folder at the top of `mount`, in any case, if there is one.
pub fn dcim(mount: &Path) -> Option<PathBuf> {
    fs::read_dir(mount)
        .ok()?
        .flatten()
        .find(|entry| entry.file_name().eq_ignore_ascii_case(DCIM) && entry.path().is_dir())
        .map(|entry| entry.path())
}

/// Copies every photo and video under `source` into `pictures`, by month.
///
/// # Errors
///
/// When `source` can't be read. A file that fails is counted and logged, and the rest go on.
pub fn import(source: &Path, pictures: &Path) -> Result<Report, ImportError> {
    let mut files = Vec::new();
    collect(source, &mut files).map_err(ImportError::read(source))?;
    files.sort();
    let mut report = Report::default();
    for file in files {
        match import_one(&file, pictures) {
            Ok(true) => report.copied += 1,
            Ok(false) => report.already += 1,
            Err(err) => {
                log!("importing {}: {err}", file.display());
                report.failed += 1;
            }
        }
    }
    Ok(report)
}

/// Copies `file` into its month folder under `pictures`; false when it was already there.
fn import_one(file: &Path, pictures: &Path) -> io::Result<bool> {
    let metadata = fs::metadata(file)?;
    let modified = metadata.modified()?;
    let month = exif_month(file).unwrap_or_else(|| month_of(modified));
    let folder = pictures.join(month.folder());
    let name = file
        .file_name()
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    let target = match place(&folder, name, metadata.len(), |path| {
        fs::metadata(path).ok().map(|meta| meta.len())
    }) {
        Placement::Already(_) => return Ok(false),
        Placement::Copy(target) => target,
    };
    fs::create_dir_all(&folder)?;
    let mut partial = target.clone().into_os_string();
    partial.push(PARTIAL);
    let partial = PathBuf::from(partial);
    fs::copy(file, &partial)?;
    File::options().write(true).open(&partial)?.set_modified(modified)?;
    fs::rename(&partial, &target)?;
    Ok(true)
}

/// Where a file goes in a month folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Placement {
    /// Copy it here, a name not taken.
    Copy(PathBuf),
    /// A file of its size is already here under its name, or the name a clash gave it.
    Already(PathBuf),
}

/// Where the file `name` of `size` bytes goes in `folder`, `size_of` giving the size of what is
/// at a path: its name; else, when a file of another size has it (two cameras' `IMG_0001.JPG`),
/// `IMG_0001 (1).JPG`, `(2)`, and so on.
pub fn place(folder: &Path, name: &OsStr, size: u64, size_of: impl Fn(&Path) -> Option<u64>) -> Placement {
    let name = Path::new(name);
    let stem = name.file_stem().unwrap_or_default().to_string_lossy();
    let extension = name
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()));
    let mut clashes = 0_u64;
    loop {
        let candidate = if clashes == 0 {
            folder.join(name)
        } else {
            folder.join(format!(
                "{stem} ({clashes}){}",
                extension.as_deref().unwrap_or_default()
            ))
        };
        match size_of(&candidate) {
            None => return Placement::Copy(candidate),
            Some(taken) if taken == size => return Placement::Already(candidate),
            Some(_) => clashes += 1,
        }
    }
}

/// Whether `path` is a photo or a video, by its extension.
pub fn is_media(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| MEDIA.iter().any(|media| media.eq_ignore_ascii_case(extension)))
}

/// The month of an EXIF date, `2024:07:15 10:00:00`; none for a blank or broken one.
pub fn parse_exif_month(text: &str) -> Option<Month> {
    let year: i32 = text.get(0..4)?.parse().ok()?;
    let month: u32 = text.get(5..7)?.parse().ok()?;
    ((1..=12).contains(&month) && year > 0).then_some(Month { year, month })
}

/// The month `time` falls in, in UTC: KubaOS reads no time zone database here, and a photo taken
/// in the last hours of a month in another zone lands in the next one at worst.
pub fn month_of(time: SystemTime) -> Month {
    let days = time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() / 86_400);
    civil(i64::try_from(days).unwrap_or(0))
}

/// The month of day `days` since 1970-01-01, by Howard Hinnant's `civil_from_days`.
/// <https://howardhinnant.github.io/date_algorithms.html#civil_from_days>
fn civil(days: i64) -> Month {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    Month {
        year: i32::try_from(year).unwrap_or(i32::MAX),
        month: u32::try_from(month).unwrap_or(1),
    }
}

/// The month in `file`'s EXIF, if it has a date there.
fn exif_month(file: &Path) -> Option<Month> {
    let mut reader = BufReader::new(File::open(file).ok()?);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    [exif::Tag::DateTimeOriginal, exif::Tag::DateTime]
        .into_iter()
        .filter_map(|tag| exif.get_field(tag, exif::In::PRIMARY))
        .find_map(|field| match &field.value {
            exif::Value::Ascii(values) => values
                .first()
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
                .and_then(parse_exif_month),
            _ => None,
        })
}

/// Every photo and video under `dir`, not following links, leaving out hidden files and folders
/// (a phone's `.thumbnails`).
fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name().as_encoded_bytes().starts_with(b".") {
            continue;
        }
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            collect(&path, files)?;
        } else if kind.is_file() && is_media(&path) {
            files.push(path);
        }
    }
    Ok(())
}

/// The pictures folder: `XDG_PICTURES_DIR` from `user-dirs.dirs` in `config`, else
/// `home/Pictures`.
pub fn pictures_dir(home: &Path, config: &Path) -> PathBuf {
    let text = fs::read_to_string(config.join("user-dirs.dirs")).unwrap_or_default();
    let value = text
        .lines()
        .rev()
        .filter_map(|line| line.trim().split_once('='))
        .find(|(name, _)| *name == "XDG_PICTURES_DIR")
        .map(|(_, value)| value.trim().trim_matches('"'));
    match value {
        Some(value) if value.starts_with("$HOME/") => home.join(&value["$HOME/".len()..]),
        Some(value) if value.starts_with('/') => PathBuf::from(value),
        _ => home.join("Pictures"),
    }
}

#[cfg(test)]
mod tests;
