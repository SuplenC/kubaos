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

fn ids(apps: &[&Folder]) -> Vec<String> {
    apps.iter().map(|app| app.id.clone()).collect()
}

#[test]
fn a_change_reaches_the_apps_whose_folder_or_vendor_repository_it_touches() {
    let catalog = Catalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps")).unwrap();

    assert_eq!(ids(&touched("apps/penpot/app.toml\nREADME.md\n", &catalog)), ["penpot"]);
    assert_eq!(
        ids(&touched("system/files/apt/sources.list.d/docker.sources\n", &catalog)),
        ["docker"]
    );
    assert_eq!(
        ids(&touched("system/packages.toml\n", &catalog)),
        ["claude-code", "docker", "google-chrome"]
    );
    assert!(
        touched(
            "apps/README.md\nsystem/files/apt/sources.list.d/tuxedo.sources\n",
            &catalog
        )
        .is_empty()
    );
}
