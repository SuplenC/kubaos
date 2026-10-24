use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;

use super::DEFAULT_MIN_AGE;
use super::Entry;
use super::Kind;
use super::Setting;
use super::downloads_dir;
use super::kind;
use super::plan;
use super::tidy;

const TWO_DAYS: Duration = Duration::from_hours(48);
const AN_HOUR: Duration = Duration::from_hours(1);

fn file(name: &str, age: Duration) -> Entry {
    Entry {
        name: OsString::from(name),
        file: true,
        age,
    }
}

#[test]
fn each_extension_has_its_kind_in_any_case() {
    assert_eq!(kind(OsStr::new("photo.JPG")), Some(Kind::Images));
    assert_eq!(kind(OsStr::new("invoice.pdf")), Some(Kind::Documents));
    assert_eq!(kind(OsStr::new("src.tar.gz")), Some(Kind::Archives));
    assert_eq!(kind(OsStr::new("chrome.deb")), Some(Kind::Installers));
    assert_eq!(kind(OsStr::new("talk.mkv")), Some(Kind::Video));
    assert_eq!(kind(OsStr::new("song.flac")), Some(Kind::Audio));
}

#[test]
fn a_partial_download_or_an_unknown_kind_has_none() {
    for name in ["movie.mkv.crdownload", "big.iso.part", "Makefile", "notes.unknownext"] {
        assert_eq!(kind(OsStr::new(name)), None, "{name}");
    }
}

#[test]
fn only_files_of_a_known_kind_a_day_old_move() {
    let entries = [
        file("old.pdf", TWO_DAYS),
        file("today.pdf", AN_HOUR),
        file(".hidden.pdf", TWO_DAYS),
        file("mystery.bin", TWO_DAYS),
        Entry {
            name: OsString::from("Projects.zip"),
            file: false,
            age: TWO_DAYS,
        },
    ];

    let moves = plan(&entries, DEFAULT_MIN_AGE);

    assert_eq!(moves, vec![(OsString::from("old.pdf"), Kind::Documents)]);
}

#[test]
fn tidying_moves_old_files_into_their_folders_and_keeps_both_on_a_clash() {
    let downloads = tempfile::tempdir().unwrap();
    let dir = downloads.path();
    fs::write(dir.join("report.pdf"), "new").unwrap();
    fs::write(dir.join("shot.png"), "png").unwrap();
    fs::create_dir(dir.join("Documents")).unwrap();
    fs::write(dir.join("Documents/report.pdf"), "old").unwrap();
    let later = SystemTime::now() + TWO_DAYS;

    let moved = tidy(dir, later, DEFAULT_MIN_AGE).unwrap();

    assert_eq!(moved, 2);
    assert_eq!(fs::read_to_string(dir.join("Documents/report.pdf")).unwrap(), "old");
    assert_eq!(fs::read_to_string(dir.join("Documents/report (1).pdf")).unwrap(), "new");
    assert!(dir.join("Images/shot.png").is_file());
    assert!(!dir.join("report.pdf").exists());
}

#[test]
fn a_fresh_download_stays_where_it_landed() {
    let downloads = tempfile::tempdir().unwrap();
    fs::write(downloads.path().join("fresh.zip"), "zip").unwrap();

    let moved = tidy(downloads.path(), SystemTime::now(), DEFAULT_MIN_AGE).unwrap();

    assert_eq!(moved, 0);
    assert!(downloads.path().join("fresh.zip").is_file());
    assert!(!downloads.path().join("Archives").exists());
}

#[test]
fn the_daily_tidy_is_off_until_turned_on() {
    let config = tempfile::tempdir().unwrap();
    let setting = Setting::new(config.path());

    assert!(!setting.daily());
    setting.set_daily(true).unwrap();
    assert!(setting.daily());
    setting.set_daily(false).unwrap();
    setting.set_daily(false).unwrap();
    assert!(!setting.daily());
}

#[test]
fn the_downloads_folder_follows_user_dirs() {
    let config = tempfile::tempdir().unwrap();
    let home = Path::new("/home/kuba");

    assert_eq!(downloads_dir(home, config.path()), home.join("Downloads"));
    fs::write(config.path().join("user-dirs.dirs"), "XDG_DOWNLOAD_DIR=\"$HOME/Scaricati\"\n").unwrap();
    assert_eq!(downloads_dir(home, config.path()), home.join("Scaricati"));
}
