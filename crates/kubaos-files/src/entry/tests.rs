use std::fs;

use super::*;

fn resolver() -> Resolver {
    Resolver::new("/home/kuba", "/home/kuba/Projects", Some("tuxedo".to_owned()))
}

fn path(arg: &str) -> Result<PathBuf, EntryError> {
    resolver().path(OsStr::new(arg))
}

#[test]
fn a_file_uri_with_an_empty_host_is_its_path() {
    assert_eq!(
        path("file:///home/kuba/Downloads"),
        Ok(PathBuf::from("/home/kuba/Downloads"))
    );
}

#[test]
fn percent_escapes_are_unescaped() {
    assert_eq!(
        path("file:///home/kuba/My%20Files/caf%C3%A9%23menu.pdf"),
        Ok(PathBuf::from("/home/kuba/My Files/café#menu.pdf"))
    );
}

#[test]
fn an_escape_may_name_bytes_that_are_no_utf8() {
    let path = path("file:///tmp/%FF.bin").unwrap();

    assert_eq!(path.as_os_str().as_bytes(), b"/tmp/\xFF.bin");
}

#[test]
fn localhost_and_this_machines_name_are_local_in_any_case() {
    assert_eq!(path("file://localhost/etc/hosts"), Ok(PathBuf::from("/etc/hosts")));
    assert_eq!(path("FILE://Tuxedo/etc/hosts"), Ok(PathBuf::from("/etc/hosts")));
}

#[test]
fn a_file_uri_on_another_host_is_refused() {
    assert_eq!(
        path("file://nas/share/movie.mkv"),
        Err(EntryError::Remote {
            uri: "file://nas/share/movie.mkv".to_owned(),
            host: "nas".to_owned(),
        })
    );
}

#[test]
fn a_single_slash_file_uri_is_its_path() {
    assert_eq!(path("file:/var/log"), Ok(PathBuf::from("/var/log")));
}

#[test]
fn a_query_or_a_fragment_is_no_part_of_the_path() {
    assert_eq!(path("file:///srv/a.txt?x=1#top"), Ok(PathBuf::from("/srv/a.txt")));
}

#[test]
fn a_broken_escape_or_a_relative_file_uri_is_malformed() {
    assert!(matches!(path("file:///tmp/%G1"), Err(EntryError::Malformed(_))));
    assert!(matches!(path("file:///tmp/%4"), Err(EntryError::Malformed(_))));
    assert!(matches!(path("file:///tmp/%+5"), Err(EntryError::Malformed(_))));
    assert!(matches!(path("file:notes.txt"), Err(EntryError::Malformed(_))));
}

#[test]
fn a_host_needs_an_absolute_path_after_it() {
    assert!(matches!(path("file://localhost"), Err(EntryError::Malformed(_))));
    assert_eq!(path("file://localhost/"), Ok(PathBuf::from("/")));
}

#[test]
fn another_scheme_is_refused() {
    assert_eq!(
        path("sftp://nas/home"),
        Err(EntryError::Scheme("sftp://nas/home".to_owned()))
    );
    assert!(matches!(resolver().uri("/home/kuba"), Err(EntryError::Scheme(_))));
}

#[test]
fn a_colon_without_slashes_is_part_of_a_path() {
    assert_eq!(
        path("notes:draft.md"),
        Ok(PathBuf::from("/home/kuba/Projects/notes:draft.md"))
    );
}

#[test]
fn a_tilde_is_the_home() {
    assert_eq!(path("~"), Ok(PathBuf::from("/home/kuba")));
    assert_eq!(path("~/Downloads"), Ok(PathBuf::from("/home/kuba/Downloads")));
    assert_eq!(path("~kuba"), Ok(PathBuf::from("/home/kuba/Projects/~kuba")));
}

#[test]
fn a_relative_path_is_under_the_working_folder_and_an_absolute_one_stays() {
    assert_eq!(
        path("kubaos/README.md"),
        Ok(PathBuf::from("/home/kuba/Projects/kubaos/README.md"))
    );
    assert_eq!(path("/etc"), Ok(PathBuf::from("/etc")));
}

#[test]
fn a_path_that_exists_is_kept_and_a_missing_one_falls_back_to_its_nearest_folder() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("report.pdf");
    fs::write(&file, "").unwrap();

    assert_eq!(existing(&file), file);
    assert_eq!(existing(&dir.path().join("gone/deeper/file.txt")), dir.path());
}
