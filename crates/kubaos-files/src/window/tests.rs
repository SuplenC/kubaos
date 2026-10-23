use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use super::*;

#[test]
fn yazi_runs_in_ghostty_with_every_entry_after_the_end_of_its_flags() {
    let entries = [
        PathBuf::from("/home/kuba/Downloads"),
        PathBuf::from("/home/kuba/-notes.md"),
    ];

    assert_eq!(
        argv(&entries),
        [
            "ghostty",
            "-e",
            "yazi",
            "--",
            "/home/kuba/Downloads",
            "/home/kuba/-notes.md"
        ]
    );
}

#[test]
fn a_path_that_isnt_utf8_opens_its_nearest_folder_that_is() {
    let path = Path::new(OsStr::from_bytes(b"/home/kuba/\xFF/file"));

    assert_eq!(argv(&[path.to_path_buf()])[4], "/home/kuba");
}
