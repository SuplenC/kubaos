use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;

use kubaos_core::pins::Leftover;
use kubaos_core::pins::LeftoverKind;
use kubaos_core::pins::Pin;
use kubaos_core::pins::PinStatus;
use kubaos_core::state::Paths;
use kubaos_core::suites::catalog;

use super::describe;
use super::expand_tilde;
use super::leftover_json;
use super::normalize;
use super::status_json;

#[test]
fn a_folder_typed_with_a_tilde_is_under_the_home() {
    let paths = Paths::from_vars(Some("/home/kuba".into()), None, None).unwrap();

    assert_eq!(expand_tilde(&paths, Path::new("~")), Path::new("/home/kuba"));
    assert_eq!(
        expand_tilde(&paths, Path::new("~/Projects/acme")),
        Path::new("/home/kuba/Projects/acme")
    );
    assert_eq!(expand_tilde(&paths, Path::new("~bob/x")), Path::new("~bob/x"));
    assert_eq!(expand_tilde(&paths, Path::new("acme/~")), Path::new("acme/~"));
}

#[test]
fn dots_in_a_folder_are_worked_out_from_the_text_as_cd_does() {
    assert_eq!(normalize(Path::new("/p/acme/y/..")), Path::new("/p/acme"));
    assert_eq!(normalize(Path::new("/p/acme/./y/../../z")), Path::new("/p/z"));
    assert_eq!(normalize(Path::new("/..")), Path::new("/"));
    assert_eq!(normalize(Path::new("/p/acme/")), Path::new("/p/acme"));
}

fn status(config: PathBuf, selected: Option<&str>, fetched: bool) -> PinStatus {
    PinStatus {
        pin: Pin {
            suite: "test".to_owned(),
            tool: "kubaos-test-tool".to_owned(),
            version: "3".to_owned(),
            config,
        },
        suite: catalog::find("go").expect("a shipped Suite"),
        installed: None,
        selected: selected.map(str::to_owned),
        fetched,
        unresolved: None,
    }
}

#[test]
fn json_carries_a_path_that_is_not_utf8() {
    let folder = PathBuf::from(OsStr::from_bytes(b"/p/caf\xe9"));
    let left = Leftover {
        envrc: folder.join(".envrc"),
        line: 2,
        kind: LeftoverKind::Bwenv {
            projects: vec!["web".into()],
        },
    };

    let row = status_json(&status(folder.join("mise.toml"), Some("3.0.0"), true));
    let left = leftover_json(&left);

    assert_eq!(row["config"], "/p/caf\u{fffd}/mise.toml");
    assert_eq!(row["selected"], "3.0.0");
    assert_eq!(left["envrc"], "/p/caf\u{fffd}/.envrc");
    assert_eq!(left["kind"], "bwenv");
}

#[test]
fn a_pin_reads_as_the_suites_state_and_the_version_mise_selects() {
    let config = PathBuf::from("/p/mise.toml");

    assert_eq!(
        describe(&status(config.clone(), Some("3.0.0"), true)),
        "Suite not installed; 3.0.0"
    );
    assert_eq!(
        describe(&status(config.clone(), Some("3.0.0"), false)),
        "Suite not installed; 3.0.0 not installed: `mise install` in the folder"
    );
    assert_eq!(
        describe(&status(config.clone(), None, false)),
        "Suite not installed; another config in the folder wins"
    );
    assert_eq!(
        describe(&PinStatus {
            unresolved: Some("Config files in /p/mise.toml are not trusted.".into()),
            ..status(config, None, false)
        }),
        "Suite not installed; unresolved: Config files in /p/mise.toml are not trusted."
    );
}
