use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use super::Month;
use super::Placement;
use super::Report;
use super::dcim;
use super::import;
use super::is_media;
use super::month_of;
use super::parse_exif_month;
use super::pictures_dir;
use super::place;

/// A JPEG holding only an EXIF block whose `DateTimeOriginal` is `date` (19 characters).
fn jpeg_taken(date: &str) -> Vec<u8> {
    let mut tiff = b"MM\0\x2a\0\0\0\x08".to_vec();
    // IFD0: one entry, the pointer to the Exif IFD at offset 26.
    tiff.extend([0, 1, 0x87, 0x69, 0, 4, 0, 0, 0, 1, 0, 0, 0, 26, 0, 0, 0, 0]);
    // The Exif IFD: one entry, DateTimeOriginal, 20 ASCII bytes at offset 44.
    tiff.extend([0, 1, 0x90, 0x03, 0, 2, 0, 0, 0, 20, 0, 0, 0, 44, 0, 0, 0, 0]);
    tiff.extend(date.as_bytes());
    tiff.push(0);
    let mut app1 = b"Exif\0\0".to_vec();
    app1.extend(tiff);
    let length = u16::try_from(app1.len() + 2).unwrap();
    let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
    jpeg.extend(length.to_be_bytes());
    jpeg.extend(app1);
    jpeg.extend([0xff, 0xd9]);
    jpeg
}

fn modified_at(path: &Path, secs: u64) {
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(UNIX_EPOCH + Duration::from_secs(secs))
        .unwrap();
}

/// 2023-11-20, a Monday, in seconds since the epoch.
const NOVEMBER_2023: u64 = 1_700_438_400;
/// Noon on 2024-02-29, in seconds since the epoch.
const LEAP_DAY_2024: u64 = 1_709_208_000;

#[test]
fn an_exif_date_gives_its_year_and_month() {
    assert_eq!(
        parse_exif_month("2024:07:15 10:00:00"),
        Some(Month { year: 2024, month: 7 })
    );
    assert_eq!(parse_exif_month("    :  :     :  :  "), None);
    assert_eq!(parse_exif_month("0000:00:00 00:00:00"), None);
}

#[test]
fn a_time_falls_in_its_utc_month() {
    assert_eq!(month_of(UNIX_EPOCH), Month { year: 1970, month: 1 });
    assert_eq!(
        month_of(UNIX_EPOCH + Duration::from_secs(NOVEMBER_2023)),
        Month { year: 2023, month: 11 }
    );
    assert_eq!(
        month_of(UNIX_EPOCH + Duration::from_secs(LEAP_DAY_2024)),
        Month { year: 2024, month: 2 }
    );
}

#[test]
fn a_month_is_a_year_folder_and_a_two_digit_month_folder() {
    assert_eq!(Month { year: 2024, month: 7 }.folder(), PathBuf::from("2024/07"));
}

#[test]
fn photos_and_videos_are_media_and_the_rest_are_not() {
    for name in ["IMG_0001.JPG", "PXL_1.heic", "DSC_1.NEF", "VID_1.mp4", "C0001.MOV"] {
        assert!(is_media(Path::new(name)), "{name}");
    }
    for name in ["notes.txt", "MISC", "INDEX.DAT"] {
        assert!(!is_media(Path::new(name)), "{name}");
    }
}

#[test]
fn a_file_goes_under_its_own_name_when_it_is_free() {
    let placement = place(Path::new("/p/2024/07"), OsStr::new("IMG_1.JPG"), 10, |_| None);

    assert_eq!(placement, Placement::Copy(PathBuf::from("/p/2024/07/IMG_1.JPG")));
}

#[test]
fn a_file_of_the_same_name_and_size_is_already_imported() {
    let placement = place(Path::new("/p"), OsStr::new("IMG_1.JPG"), 10, |_| Some(10));

    assert_eq!(placement, Placement::Already(PathBuf::from("/p/IMG_1.JPG")));
}

#[test]
fn another_file_under_the_name_pushes_it_to_a_numbered_one() {
    let taken = |path: &Path| match path.to_str() {
        Some("/p/IMG_1.JPG") => Some(99),
        Some("/p/IMG_1 (1).JPG") => Some(98),
        _ => None,
    };

    let placement = place(Path::new("/p"), OsStr::new("IMG_1.JPG"), 10, taken);

    assert_eq!(placement, Placement::Copy(PathBuf::from("/p/IMG_1 (2).JPG")));
}

#[test]
fn the_dcim_folder_is_found_in_any_case() {
    let card = tempfile::tempdir().unwrap();
    fs::create_dir(card.path().join("dcim")).unwrap();

    assert_eq!(dcim(card.path()), Some(card.path().join("dcim")));
    assert_eq!(dcim(&card.path().join("dcim")), None);
}

#[test]
fn photos_are_copied_by_capture_date_and_a_second_import_copies_nothing() {
    let card = tempfile::tempdir().unwrap();
    let pictures = tempfile::tempdir().unwrap();
    let camera = card.path().join("DCIM/100CANON");
    fs::create_dir_all(&camera).unwrap();
    fs::write(camera.join("IMG_0001.JPG"), jpeg_taken("2019:03:07 12:00:00")).unwrap();
    fs::write(camera.join("MVI_0002.MP4"), b"video").unwrap();
    modified_at(&camera.join("MVI_0002.MP4"), NOVEMBER_2023);
    fs::write(camera.join("INDEX.DAT"), b"not a photo").unwrap();
    fs::create_dir(camera.join(".thumbnails")).unwrap();
    fs::write(camera.join(".thumbnails/IMG_0001.JPG"), b"thumb").unwrap();

    let first = import(&card.path().join("DCIM"), pictures.path()).unwrap();
    let second = import(&card.path().join("DCIM"), pictures.path()).unwrap();

    assert_eq!(
        first,
        Report {
            copied: 2,
            already: 0,
            failed: 0
        }
    );
    assert!(pictures.path().join("2019/03/IMG_0001.JPG").is_file());
    let video = pictures.path().join("2023/11/MVI_0002.MP4");
    assert_eq!(fs::read(&video).unwrap(), b"video");
    let kept = fs::metadata(&video).unwrap().modified().unwrap();
    assert_eq!(kept, UNIX_EPOCH + Duration::from_secs(NOVEMBER_2023));
    assert_eq!(
        second,
        Report {
            copied: 0,
            already: 2,
            failed: 0
        }
    );
    assert!(camera.join("IMG_0001.JPG").is_file(), "the device keeps its photos");
}

#[test]
fn the_pictures_folder_follows_user_dirs() {
    let config = tempfile::tempdir().unwrap();
    let home = Path::new("/home/kuba");

    assert_eq!(pictures_dir(home, config.path()), home.join("Pictures"));
    fs::write(
        config.path().join("user-dirs.dirs"),
        "XDG_PICTURES_DIR=\"$HOME/Immagini\"\n",
    )
    .unwrap();
    assert_eq!(pictures_dir(home, config.path()), home.join("Immagini"));
}
