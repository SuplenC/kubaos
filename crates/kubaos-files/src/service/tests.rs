use std::fs;

use super::*;

#[test]
fn a_downloaded_file_is_revealed_and_a_folder_opened_with_escapes_undone() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("my report.pdf");
    fs::write(&file, "").unwrap();
    let resolver = Resolver::new(dir.path(), dir.path(), None);
    let base = format!("file://{}", dir.path().display());

    let entries = entries([format!("{base}/my%20report.pdf").as_str(), base.as_str()], &resolver);

    assert_eq!(entries.paths, [file, dir.path().to_path_buf()]);
    assert_eq!(entries.refused, [] as [entry::EntryError; 0]);
}

#[test]
fn a_file_that_is_gone_opens_its_folder_once() {
    let dir = tempfile::tempdir().unwrap();
    let resolver = Resolver::new(dir.path(), dir.path(), None);
    let base = format!("file://{}", dir.path().display());

    let entries = entries(
        [format!("{base}/a.part").as_str(), format!("{base}/b.part").as_str()],
        &resolver,
    );

    assert_eq!(entries.paths, [dir.path().to_path_buf()]);
}

#[test]
fn uris_that_name_no_local_path_are_refused_and_the_rest_still_open() {
    let resolver = Resolver::new("/", "/", None);

    let entries = entries(["smb://nas/share", "file://nas/share/a.txt", "file:///"], &resolver);

    assert_eq!(entries.paths, [PathBuf::from("/")]);
    assert_eq!(
        entries.refused,
        [
            EntryError::Scheme("smb://nas/share".to_owned()),
            EntryError::Remote {
                uri: "file://nas/share/a.txt".to_owned(),
                host: "nas".to_owned(),
            },
        ]
    );
}

#[test]
fn the_introspection_names_every_method_the_service_answers() {
    for method in METHODS {
        assert!(
            INTROSPECTION.contains(&format!("<method name=\"{method}\">")),
            "{method}"
        );
    }
}
