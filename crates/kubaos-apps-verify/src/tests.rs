use super::*;
use crate::check::Outcome;

fn args(words: &[&str]) -> Result<Option<Args>, String> {
    parse(words.iter().map(|word| (*word).to_owned()))
}

#[test]
fn no_argument_checks_every_app_for_the_default_release() {
    let parsed = args(&[]).unwrap().unwrap();

    assert!(parsed.ids.is_empty());
    assert_eq!(parsed.changed, None);
    assert_eq!(parsed.release, DEFAULT_RELEASE);
}

#[test]
fn ids_and_flags_are_read_in_any_order() {
    let parsed = args(&["docker", "--root", "/r", "bitwarden", "--release", "plucky"])
        .unwrap()
        .unwrap();

    assert_eq!(parsed.ids, ["docker", "bitwarden"]);
    assert_eq!(parsed.root, PathBuf::from("/r"));
    assert_eq!(parsed.release, "plucky");
}

#[test]
fn changed_with_ids_or_an_unknown_flag_is_a_usage_error() {
    assert!(args(&["--changed", "origin/main", "docker"]).is_err());
    assert!(args(&["--fast"]).is_err());
    assert!(args(&["--root"]).is_err());
}

#[test]
fn the_table_pads_every_column_but_the_last() {
    let rows = [Row {
        app: "docker".to_owned(),
        check: "key",
        subject: "docker".to_owned(),
        outcome: Outcome::Ok,
        detail: "9DC8".to_owned(),
    }];

    let text = table(&rows);

    assert_eq!(
        text,
        "APP     CHECK  WHAT    RESULT  DETAIL\ndocker  key    docker  ok      9DC8\n"
    );
}
