use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use super::Leftover;
use super::LeftoverKind;
use super::Migrated;
use super::Pin;
use super::PinError;
use super::Pins;
use super::Tangle;
use super::main_tool;
use crate::mise::Mise;
use crate::mise::fake::FakeMise;
use crate::state::Paths;
use crate::suites::OptionsChange;
use crate::suites::Suite;
use crate::suites::Suites;
use crate::suites::Toolchain;
use crate::suites::catalog::TEST;

const TOOL: &str = "kubaos-test-tool";

struct Home {
    _tmp: tempfile::TempDir,
    paths: Paths,
    mise: Arc<FakeMise>,
    pins: Pins,
}

fn home() -> Home {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::from_vars(Some(tmp.path().join("home").into_os_string()), None, None).unwrap();
    fs::create_dir_all(paths.flake()).unwrap();
    let mise = Arc::new(FakeMise::new(paths.mise_config().join("conf.d")));
    let pins = Pins::with_mise(&paths, Arc::clone(&mise) as Arc<dyn Mise>);
    Home {
        _tmp: tmp,
        paths,
        mise,
        pins,
    }
}

impl Home {
    /// A folder under the Projects root, made.
    fn project(&self, rel: &str) -> PathBuf {
        let dir = self.paths.projects_root().join(rel);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Installs the test Suite at its default version, 1.
    fn install_test_suite(&self) {
        Suites::with_mise(&self.paths, Arc::clone(&self.mise) as Arc<dyn Mise>)
            .add(&TEST, &OptionsChange::default())
            .unwrap();
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
    fs::write(path, contents).expect("write the file");
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("read the file")
}

fn pin(version: &str, config: &Path) -> Pin {
    Pin {
        suite: "test".to_owned(),
        tool: TOOL.to_owned(),
        version: version.to_owned(),
        config: config.to_path_buf(),
    }
}

#[test]
fn a_pin_of_a_suite_on_mise_is_its_main_tool_and_any_other_suite_is_refused() {
    let nix = Suite {
        name: "nix-only",
        mise: &[],
        toolchain: Toolchain::Nix,
        ..TEST
    };
    let unversioned = Suite {
        name: "unversioned",
        versions: &[],
        mise: &[],
        ..TEST
    };

    assert_eq!(main_tool(&TEST).unwrap(), TOOL);
    assert!(matches!(main_tool(&nix), Err(PinError::NotOnMise { suite }) if suite == "nix-only"));
    assert!(matches!(main_tool(&unversioned), Err(PinError::Unversioned { .. })));
}

#[test]
fn pinning_installs_the_version_then_writes_a_new_mise_toml() {
    let h = home();
    let acme = h.project("acme");

    let pinned = h.pins.pin(&TEST, "3", &acme).unwrap();

    assert_eq!(pinned.config, acme.join("mise.toml"));
    assert!(pinned.created);
    assert_eq!(pinned.previous, None);
    assert_eq!(pinned.applies, Some(pin("3", &acme.join("mise.toml"))));
    assert_eq!(read(&acme.join("mise.toml")), "[tools]\nkubaos-test-tool = \"3\"\n");
    assert_eq!(h.mise.installed(), ["kubaos-test-tool@3.0.0"]);
    assert_eq!(h.mise.tracked(), [acme.join("mise.toml")]);
}

#[test]
fn pinning_again_replaces_the_version_and_keeps_the_rest_of_the_file() {
    let h = home();
    let acme = h.project("acme");
    let config = acme.join("mise.toml");
    write(
        &config,
        "# acme's tools\n[tools]\npython = \"3.12\"\nkubaos-test-tool = \"2\" # the old API\n\n[env]\nPORT = \"8080\"\n",
    );

    let pinned = h.pins.pin(&TEST, "3", &acme).unwrap();

    assert!(!pinned.created);
    assert_eq!(pinned.previous.as_deref(), Some("2"));
    assert_eq!(
        read(&config),
        "# acme's tools\n[tools]\npython = \"3.12\"\nkubaos-test-tool = \"3\" # the old API\n\n[env]\nPORT = \"8080\"\n"
    );
}

#[test]
fn pinning_adds_to_an_existing_file_without_tools() {
    let h = home();
    let acme = h.project("acme");
    let config = acme.join("mise.toml");
    write(&config, "[env]\nPORT = \"8080\"\n");

    h.pins.pin(&TEST, "2", &acme).unwrap();

    assert_eq!(
        read(&config),
        "[env]\nPORT = \"8080\"\n\n[tools]\nkubaos-test-tool = \"2\"\n"
    );
}

#[test]
fn pinning_edits_the_folders_config_that_already_names_the_tool() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join(".mise.toml"), "[tools]\nkubaos-test-tool = \"1\"\n");

    let pinned = h.pins.pin(&TEST, "2", &acme).unwrap();

    assert_eq!(pinned.config, acme.join(".mise.toml"));
    assert_eq!(pinned.previous.as_deref(), Some("1"));
    assert!(!acme.join("mise.toml").exists());
}

#[test]
fn a_local_config_that_pins_the_tool_too_is_named_as_the_one_that_applies() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join("mise.local.toml"), "[tools]\nkubaos-test-tool = \"1\"\n");
    write(&acme.join("mise.toml"), "[env]\nPORT = \"8080\"\n");

    let pinned = h.pins.pin(&TEST, "3", &acme).unwrap();

    assert_eq!(pinned.config, acme.join("mise.local.toml"));
    assert_eq!(pinned.applies, Some(pin("3", &acme.join("mise.local.toml"))));
}

#[test]
fn a_version_mise_cannot_install_writes_nothing() {
    let h = home();
    let acme = h.project("acme");

    let err = h.pins.pin(&TEST, "9", &acme).unwrap_err();

    assert!(
        matches!(err, PinError::Install { ref version, .. } if version == "9"),
        "{err:?}"
    );
    assert!(!acme.join("mise.toml").exists());
}

#[test]
fn a_version_that_is_not_one_word_is_refused_before_mise_runs() {
    let h = home();
    let acme = h.project("acme");

    for version in ["", "1 2", "../x", "\"3\"", "-1"] {
        let err = h.pins.pin(&TEST, version, &acme).unwrap_err();
        assert!(matches!(err, PinError::InvalidVersion { .. }), "{version:?}: {err:?}");
    }
    assert_eq!(h.mise.calls(), [] as [String; 0]);
}

#[test]
fn a_pin_with_options_is_left_to_the_user() {
    let h = home();
    let acme = h.project("acme");
    let text = "[tools]\nkubaos-test-tool = { version = \"2\", os = [\"linux\"] }\n";
    write(&acme.join("mise.toml"), text);

    let pin_err = h.pins.pin(&TEST, "3", &acme).unwrap_err();
    let unpin_err = h.pins.unpin("test", &acme).unwrap_err();

    for err in [pin_err, unpin_err] {
        assert!(
            matches!(
                err,
                PinError::Tangled {
                    why: Tangle::NotAString,
                    ..
                }
            ),
            "{err:?}"
        );
    }
    assert_eq!(read(&acme.join("mise.toml")), text);
    assert_eq!(h.mise.installed(), [] as [String; 0]);
}

#[test]
fn a_file_that_is_not_toml_is_left_alone() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join("mise.toml"), "[tools\n");

    let err = h.pins.pin(&TEST, "3", &acme).unwrap_err();

    assert!(matches!(err, PinError::Parse { .. }), "{err:?}");
    assert_eq!(read(&acme.join("mise.toml")), "[tools\n");
}

#[test]
fn pinning_a_folder_that_does_not_exist_is_an_error() {
    let h = home();

    let err = h
        .pins
        .pin(&TEST, "2", &h.paths.projects_root().join("nope"))
        .unwrap_err();

    assert!(matches!(err, PinError::NotAFolder { .. }), "{err:?}");
}

#[test]
fn pinning_keeps_the_mode_and_writes_through_a_link() {
    let h = home();
    let acme = h.project("acme");
    let real = h.project("dotfiles").join("acme.toml");
    write(&real, "[tools]\n");
    fs::set_permissions(&real, fs::Permissions::from_mode(0o600)).unwrap();
    std::os::unix::fs::symlink(&real, acme.join("mise.toml")).unwrap();

    h.pins.pin(&TEST, "2", &acme).unwrap();

    assert!(fs::symlink_metadata(acme.join("mise.toml")).unwrap().is_symlink());
    assert_eq!(read(&real), "[tools]\nkubaos-test-tool = \"2\"\n");
    assert_eq!(fs::metadata(&real).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn the_written_pin_is_reported_when_mise_cannot_read_the_folder() {
    let h = home();
    let acme = h.project("acme");
    h.mise.fail_current();

    let err = h.pins.pin(&TEST, "2", &acme).unwrap_err();

    assert!(matches!(err, PinError::Resolve { .. }), "{err:?}");
    assert_eq!(read(&acme.join("mise.toml")), "[tools]\nkubaos-test-tool = \"2\"\n");
}

#[test]
fn a_pin_covers_its_folder_and_everything_below_it_and_the_nearest_wins() {
    let h = home();
    h.install_test_suite();
    let acme = h.project("acme");
    let src = h.project("acme/old-app/src");
    let new_app = h.project("acme/new-app");
    h.pins.pin(&TEST, "3", &acme).unwrap();
    h.pins.pin(&TEST, "2", &new_app).unwrap();

    let below = h.pins.resolve(&src).unwrap();
    let nested = h.pins.resolve(&new_app).unwrap();
    let outside = h.pins.resolve(&h.project("other")).unwrap();

    let only = |r: &super::Resolution| r.pins.iter().map(|s| s.pin.clone()).collect::<Vec<_>>();
    assert_eq!(only(&below), [pin("3", &acme.join("mise.toml"))]);
    assert_eq!(below.pins[0].selected.as_deref(), Some("3.0.0"));
    assert!(below.pins[0].fetched);
    assert_eq!(below.pins[0].global_version(), Some("1"));
    assert_eq!(only(&nested), [pin("2", &new_app.join("mise.toml"))]);
    assert_eq!(
        outside.pins.len(),
        0,
        "the Suite's fragment is no pin: {:?}",
        outside.pins
    );
}

#[test]
fn a_tool_of_no_suite_is_no_pin() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join("mise.toml"), "[tools]\ndemo-lang = \"2\"\n");

    assert_eq!(h.pins.resolve(&acme).unwrap().pins.len(), 0);
    assert_eq!(h.pins.all().unwrap().pins, [] as [Pin; 0]);
}

#[test]
fn a_suite_that_is_not_installed_can_still_be_pinned() {
    let h = home();
    let acme = h.project("acme");
    h.pins.pin(&TEST, "2", &acme).unwrap();

    let status = &h.pins.resolve(&acme).unwrap().pins[0];

    assert_eq!(status.installed, None);
    assert_eq!(status.selected.as_deref(), Some("2.0.0"));
}

#[test]
fn a_pin_mise_has_not_installed_says_so() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join("mise.toml"), "[tools]\nkubaos-test-tool = \"3\"\n");

    let status = &h.pins.resolve(&acme).unwrap().pins[0];

    assert_eq!(status.selected.as_deref(), Some("3.0.0"));
    assert!(!status.fetched);
}

#[test]
fn resolving_a_folder_that_does_not_exist_is_an_error() {
    let h = home();

    let err = h.pins.resolve(&h.paths.projects_root().join("nope")).unwrap_err();

    assert!(matches!(err, PinError::NotAFolder { .. }), "{err:?}");
}

#[test]
fn unpinning_removes_the_entry_keeps_the_rest_and_prunes() {
    let h = home();
    h.install_test_suite();
    let acme = h.project("acme");
    let config = acme.join("mise.toml");
    write(&config, "[tools]\npython = \"3.12\"\n");
    h.pins.pin(&TEST, "3", &acme).unwrap();

    let unpinned = h.pins.unpin("test", &acme).unwrap();

    assert_eq!(unpinned.version, "3");
    assert!(!unpinned.deleted);
    assert_eq!(read(&config), "[tools]\npython = \"3.12\"\n");
    assert_eq!(h.mise.installed(), ["kubaos-test-tool@1.0.1"]);
}

#[test]
fn unpinning_the_last_entry_deletes_the_file() {
    let h = home();
    let acme = h.project("acme");
    h.pins.pin(&TEST, "3", &acme).unwrap();

    let unpinned = h.pins.unpin("test", &acme).unwrap();

    assert!(unpinned.deleted);
    assert!(!acme.join("mise.toml").exists());
}

#[test]
fn a_version_another_folder_pins_survives_the_prune() {
    let h = home();
    let acme = h.project("acme");
    let other = h.project("other");
    h.pins.pin(&TEST, "3", &acme).unwrap();
    h.pins.pin(&TEST, "3", &other).unwrap();
    h.pins.pin(&TEST, "2", &h.project("third")).unwrap();

    h.pins.unpin("test", &acme).unwrap();

    assert_eq!(h.mise.installed(), ["kubaos-test-tool@2.0.0", "kubaos-test-tool@3.0.0"]);
}

#[test]
fn unpinning_where_no_pin_is_written_names_the_config_whose_pin_applies() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/app");
    h.pins.pin(&TEST, "3", &acme).unwrap();

    let err = h.pins.unpin("test", &app).unwrap_err();

    assert!(
        matches!(&err, PinError::NotPinned { applies_from: Some(config), .. } if *config == acme.join("mise.toml")),
        "{err:?}"
    );
}

#[test]
fn unpinning_a_suite_that_is_unknown_or_has_no_versions_is_refused() {
    let h = home();
    let acme = h.project("acme");

    let unknown = h.pins.unpin("cobol", &acme).unwrap_err();
    let unversioned = h.pins.unpin("wallpaper-depth", &acme).unwrap_err();

    assert!(matches!(unknown, PinError::UnknownSuite { .. }), "{unknown:?}");
    assert!(matches!(unversioned, PinError::Unversioned { .. }), "{unversioned:?}");
}

#[test]
fn every_pin_under_the_projects_root_is_listed_but_not_in_hidden_or_dependency_folders() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/app");
    write(
        &acme.join("mise.toml"),
        "[tools]\nkubaos-test-tool = \"3\"\npython = \"3.12\"\n",
    );
    write(&app.join(".mise.toml"), "[tools]\nkubaos-test-tool = \"2\"\n");
    write(
        &acme.join("node_modules/x/mise.toml"),
        "[tools]\nkubaos-test-tool = \"1\"\n",
    );
    write(&acme.join(".git/mise.toml"), "[tools]\nkubaos-test-tool = \"1\"\n");
    write(
        &h.paths.projects_root().join("a/b/c/d/e/mise.toml"),
        "[tools]\nkubaos-test-tool = \"1\"\n",
    );

    let found = h.pins.all().unwrap();
    let statuses = h.pins.statuses(found.pins.clone()).unwrap();

    assert_eq!(
        found.pins,
        [pin("2", &app.join(".mise.toml")), pin("3", &acme.join("mise.toml"))]
    );
    assert_eq!(statuses[0].selected.as_deref(), Some("2.0.0"));
}

#[test]
fn use_suite_and_use_bwenv_lines_left_in_an_envrc_are_reported() {
    let h = home();
    let acme = h.project("acme");
    let src = h.project("acme/src");
    write(
        &acme.join(".envrc"),
        "source_up\nuse suite node 22\nexport A=1\nuse bwenv web api\n",
    );

    let here = h.pins.resolve(&src).unwrap();
    let all = h.pins.all().unwrap();

    let expected = [
        Leftover {
            envrc: acme.join(".envrc"),
            line: 2,
            kind: LeftoverKind::Suite {
                suite: "node".into(),
                version: "22".into(),
            },
        },
        Leftover {
            envrc: acme.join(".envrc"),
            line: 4,
            kind: LeftoverKind::Bwenv {
                projects: vec!["web".into(), "api".into()],
            },
        },
    ];
    assert_eq!(here.leftovers, expected);
    assert_eq!(all.leftovers, expected);
}

#[test]
fn the_migration_moves_pins_and_bwenv_to_mise_toml_and_deletes_an_envrc_left_empty() {
    let h = home();
    let acme = h.project("acme");
    write(
        &acme.join(".envrc"),
        "# acme\nsource_up\nuse suite test 3\nuse bwenv web api\n",
    );
    fs::create_dir_all(acme.join(".direnv/suites/test/3-bin")).unwrap();

    let migrated = h.pins.migrate().unwrap();

    assert_eq!(migrated.len(), 1);
    assert_eq!(migrated[0].pins, [("test".to_owned(), "3".to_owned())]);
    assert_eq!(migrated[0].bwenv, ["web", "api"]);
    assert!(migrated[0].envrc_deleted);
    assert!(!acme.join(".envrc").exists());
    assert!(!acme.join(".direnv").exists());
    assert_eq!(
        read(&acme.join("mise.toml")),
        "[tools]\nkubaos-test-tool = \"3\"\n\n[env]\nKUBAOS_BWENV = \"web api\"\n_.source = { path = \"~/.local/share/kubaos/bwenv.sh\", redact = true }\n"
    );
    assert!(
        h.mise
            .calls()
            .contains(&format!("mise trust {}", acme.join("mise.toml").display()))
    );
    let backed_up: Vec<_> = fs::read_dir(h.paths.backups()).unwrap().flatten().collect();
    assert_eq!(backed_up.len(), 1);
    let backup = backed_up[0].path().join(acme.strip_prefix("/").unwrap()).join(".envrc");
    assert_eq!(
        read(&backup),
        "# acme\nsource_up\nuse suite test 3\nuse bwenv web api\n"
    );
}

#[test]
fn the_migration_leaves_what_it_cannot_move_and_the_rest_of_the_envrc() {
    let h = home();
    let acme = h.project("acme");
    let twice = h.project("twice");
    write(
        &acme.join(".envrc"),
        "use flake\nuse suite rust 1\nif true; then\n  use suite test 2\nfi\nuse bwenv web\n",
    );
    write(&twice.join(".envrc"), "use suite test 2\nuse suite test 3\n");

    let migrated = h.pins.migrate().unwrap();

    assert_eq!(migrated.len(), 1);
    assert_eq!(migrated[0].bwenv, ["web"]);
    assert_eq!(migrated[0].pins, [] as [(String, String); 0]);
    assert!(!migrated[0].envrc_deleted);
    assert_eq!(
        read(&acme.join(".envrc")),
        "use flake\nuse suite rust 1\nif true; then\n  use suite test 2\nfi\n"
    );
    assert!(!twice.join("mise.toml").exists());
    let left: Vec<_> = h
        .pins
        .all()
        .unwrap()
        .leftovers
        .into_iter()
        .map(|l| (l.envrc, l.line))
        .collect();
    assert_eq!(
        left,
        [
            (acme.join(".envrc"), 2),
            (acme.join(".envrc"), 4),
            (twice.join(".envrc"), 1),
            (twice.join(".envrc"), 2),
        ]
    );
}

#[test]
fn the_migration_keeps_a_pin_mise_toml_already_has() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join("mise.toml"), "[tools]\nkubaos-test-tool = \"2\"\n");
    write(&acme.join(".envrc"), "use suite test 3\n");

    let migrated = h.pins.migrate().unwrap();

    assert_eq!(migrated[0].kept, [("test".to_owned(), "3".to_owned())]);
    assert_eq!(migrated[0].pins, [] as [(String, String); 0]);
    assert_eq!(read(&acme.join("mise.toml")), "[tools]\nkubaos-test-tool = \"2\"\n");
    assert!(!acme.join(".envrc").exists());
}

#[test]
fn the_migration_runs_again_without_changing_anything() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join(".envrc"), "use suite test 3\nexport A=1\n");
    h.pins.migrate().unwrap();
    let after = (read(&acme.join(".envrc")), read(&acme.join("mise.toml")));

    let again = h.pins.migrate().unwrap();

    assert_eq!(again, [] as [Migrated; 0]);
    assert_eq!((read(&acme.join(".envrc")), read(&acme.join("mise.toml"))), after);
}

#[test]
fn the_migration_leaves_a_bwenv_when_the_config_already_sources_a_script() {
    let h = home();
    let acme = h.project("acme");
    let config = "[env]\n_.source = \"env.sh\"\n";
    write(&acme.join("mise.toml"), config);
    write(&acme.join(".envrc"), "use bwenv web\n");

    let migrated = h.pins.migrate().unwrap();

    assert_eq!(migrated, [] as [Migrated; 0]);
    assert_eq!(read(&acme.join("mise.toml")), config);
    assert_eq!(read(&acme.join(".envrc")), "use bwenv web\n");
}
