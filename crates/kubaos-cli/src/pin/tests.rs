use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;

use kubaos_core::pins::Fetch;
use kubaos_core::pins::Pin;
use kubaos_core::pins::PinStatus;
use kubaos_core::pins::Resolution;
use kubaos_core::state::Paths;

use super::expand_tilde;
use super::normalize;
use super::resolution_json;
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

#[test]
fn json_carries_a_path_that_is_not_utf8() {
    let folder = PathBuf::from(OsStr::from_bytes(b"/p/caf\xe9"));
    let envrc = folder.join(".envrc");
    let pin = Pin {
        suite: "test".to_owned(),
        version: "1".to_owned(),
        envrc: envrc.clone(),
    };
    let status = PinStatus {
        pin: pin.clone(),
        suite: None,
        installed: None,
        toolchain: Fetch::Missing,
    };
    let resolution = Resolution {
        nearest: Some(envrc),
        pins: vec![pin],
        hidden: Vec::new(),
    };

    let one = resolution_json(&folder, &resolution, std::slice::from_ref(&status));
    let row = status_json(&status);

    assert_eq!(one["folder"], "/p/caf\u{fffd}");
    assert_eq!(one["pins"][0]["envrc"], "/p/caf\u{fffd}/.envrc");
    assert_eq!(row["envrc"], "/p/caf\u{fffd}/.envrc");
}
