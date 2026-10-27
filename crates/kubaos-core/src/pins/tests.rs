use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use super::Fetch;
use super::Hidden;
use super::Pin;
use super::PinError;
use super::Pins;
use super::Tangle;
use crate::state::Paths;
use crate::suites::OptionsChange;
use crate::suites::Suites;
use crate::suites::catalog::TEST;

struct Home {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    paths: Paths,
    pins: Pins,
    /// What the fake `direnv` was run with, one line per run.
    direnv_log: PathBuf,
}

/// A home in a temporary folder whose `direnv` is a script that logs its arguments. `sh` runs the
/// script, so no test executes a file another test's fork may still hold open for writing.
fn home() -> Home {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    let paths = Paths::from_vars(Some(root.join("home").into_os_string()), None, None).unwrap();
    fs::create_dir_all(paths.flake()).unwrap();
    let direnv_log = root.join("direnv.log");
    let direnv = root.join("direnv");
    write(
        &direnv,
        &format!("#!/bin/sh\necho \"$*\" >>'{}'\n", direnv_log.display()),
    );
    let pins = Pins::new(&paths).with_direnv([OsString::from("sh"), direnv.into_os_string()]);
    Home {
        _tmp: tmp,
        root,
        paths,
        pins,
        direnv_log,
    }
}

impl Home {
    /// A folder under the Projects root, made.
    fn project(&self, rel: &str) -> PathBuf {
        let dir = self.paths.projects_root().join(rel);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The toolchain entry `home/pins.nix` writes for the test Suite's `version`, with its
    /// output at `out`.
    fn toolchain(&self, version: &str, out: &Path) {
        let entry = self.paths.toolchains().join("test").join(version);
        write(
            &entry,
            &format!(
                "/nix/store/aaaa-kubaos-test-suite-{version}.drv!out\n{}\n",
                out.display()
            ),
        );
    }

    fn direnv_runs(&self) -> Vec<String> {
        fs::read_to_string(&self.direnv_log)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
    fs::write(path, contents).expect("write the file");
}

fn pin(suite: &str, version: &str, envrc: &Path) -> Pin {
    Pin {
        suite: suite.to_owned(),
        version: version.to_owned(),
        envrc: envrc.to_path_buf(),
    }
}

#[test]
fn a_folder_with_no_envrc_above_it_has_no_pins() {
    let h = home();
    let app = h.project("acme/old-app");

    let resolved = h.pins.resolve(&app).unwrap();

    assert_eq!(resolved.pins, Vec::<Pin>::new());
    assert_eq!(resolved.hidden, Vec::<Hidden>::new());
    assert_eq!(resolved.nearest, None);
}

#[test]
fn a_pin_covers_its_folder_and_everything_below_it() {
    let h = home();
    let acme = h.project("acme");
    let deep = h.project("acme/old-app/src/lib");
    write(&acme.join(".envrc"), "use suite test 1\n");

    let here = h.pins.resolve(&acme).unwrap();
    let below = h.pins.resolve(&deep).unwrap();

    assert_eq!(here.pins, [pin("test", "1", &acme.join(".envrc"))]);
    assert_eq!(below.pins, here.pins);
    assert_eq!(below.nearest, Some(acme.join(".envrc")));
}

#[test]
fn the_nearest_pin_wins_over_one_it_sources_from_above() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/new-app");
    write(&acme.join(".envrc"), "use suite test 1\n");
    write(&app.join(".envrc"), "source_up\nuse suite test 2\n");

    let resolved = h.pins.resolve(&app).unwrap();

    assert_eq!(resolved.pins, [pin("test", "2", &app.join(".envrc"))]);
    assert_eq!(resolved.hidden, Vec::<Hidden>::new());
}

#[test]
fn a_nested_envrc_without_source_up_hides_the_pin_above() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/other-app");
    write(&acme.join(".envrc"), "use suite test 1\n");
    write(&app.join(".envrc"), "export APP=other\n");

    let resolved = h.pins.resolve(&app).unwrap();

    assert_eq!(resolved.pins, Vec::<Pin>::new());
    assert_eq!(resolved.hidden.len(), 1);
    assert_eq!(resolved.hidden[0].pin, pin("test", "1", &acme.join(".envrc")));
    assert_eq!(resolved.hidden[0].by, app.join(".envrc"));
}

#[test]
fn source_up_reaches_every_envrc_up_the_chain() {
    let h = home();
    let projects = h.project("");
    let acme = h.project("acme");
    let app = h.project("acme/new-app/web");
    write(&projects.join(".envrc"), "use suite demo 7\n");
    write(&acme.join(".envrc"), "source_up_if_exists\nuse suite test 1\n");
    write(&app.join(".envrc"), "source_up\n");

    let resolved = h.pins.resolve(&app).unwrap();

    assert_eq!(
        resolved.pins,
        [
            pin("demo", "7", &projects.join(".envrc")),
            pin("test", "1", &acme.join(".envrc")),
        ]
    );
}

#[test]
fn a_pin_before_source_up_loses_to_the_one_sourced_after_it() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/app");
    write(&acme.join(".envrc"), "use suite test 1\n");
    write(&app.join(".envrc"), "use suite test 2\nsource_up\n");

    let resolved = h.pins.resolve(&app).unwrap();

    assert_eq!(resolved.pins, [pin("test", "1", &acme.join(".envrc"))]);
}

#[test]
fn pin_lines_are_read_as_direnv_runs_them() {
    let h = home();
    let app = h.project("app");
    write(
        &app.join(".envrc"),
        "# use suite test 3\n\
         use flake\n\
         use suite \"test\" '1'  # the legacy API\n\
         use suite test\n\
         use suite test 2 extra\n\
         use suite demo 4\n\
         use suite demo 5\n",
    );

    let resolved = h.pins.resolve(&app).unwrap();

    assert_eq!(
        resolved.pins,
        [
            pin("demo", "5", &app.join(".envrc")),
            pin("test", "1", &app.join(".envrc"))
        ]
    );
}

#[test]
fn resolving_a_folder_that_does_not_exist_is_an_error() {
    let h = home();

    let err = h.pins.resolve(&h.root.join("nowhere")).unwrap_err();

    assert!(matches!(err, PinError::NotAFolder { .. }), "{err:?}");
}

#[test]
fn a_pinned_version_that_is_not_installed_is_fetched_on_first_use() {
    let h = home();
    let acme = h.project("acme");
    Suites::new(&h.paths).add(&TEST, &OptionsChange::default()).unwrap();
    h.toolchain("3", &h.root.join("store/kubaos-test-suite-3"));

    let status = h.pins.status(pin("test", "3", &acme.join(".envrc"))).unwrap();

    assert_eq!(status.toolchain, Fetch::OnFirstUse);
    assert_eq!(status.global_version(), Some("1"));
    assert!(!status.is_global());
    assert!(status.suite.is_some());
}

#[test]
fn a_pinned_version_already_in_the_store_is_used_from_there() {
    let h = home();
    let acme = h.project("acme");
    let out = h.root.join("store/kubaos-test-suite-1");
    fs::create_dir_all(&out).unwrap();
    Suites::new(&h.paths).add(&TEST, &OptionsChange::default()).unwrap();
    h.toolchain("1", &out);

    let status = h.pins.status(pin("test", "1", &acme.join(".envrc"))).unwrap();

    assert_eq!(status.toolchain, Fetch::InStore { path: out });
    assert!(status.is_global());
}

#[test]
fn a_suite_that_is_not_installed_can_still_be_pinned() {
    let h = home();
    let acme = h.project("acme");
    h.toolchain("2", &h.root.join("store/kubaos-test-suite-2"));

    let status = h.pins.status(pin("test", "2", &acme.join(".envrc"))).unwrap();

    assert_eq!(status.installed, None);
    assert_eq!(status.toolchain, Fetch::OnFirstUse);
}

#[test]
fn a_version_with_no_toolchain_in_the_home_has_none_to_fetch() {
    let h = home();
    let acme = h.project("acme");

    let unknown = h.pins.status(pin("test", "9", &acme.join(".envrc"))).unwrap();
    let foreign = h.pins.status(pin("cobol", "85", &acme.join(".envrc"))).unwrap();

    assert_eq!(unknown.toolchain, Fetch::Missing);
    assert_eq!(foreign.toolchain, Fetch::Missing);
    assert!(foreign.suite.is_none());
}

#[test]
fn pinning_writes_a_new_envrc_and_allows_it() {
    let h = home();
    let acme = h.project("acme");

    let pinned = h.pins.pin(&TEST, "2", &acme).unwrap();

    assert_eq!(pinned.envrc, acme.join(".envrc"));
    assert!(pinned.created);
    assert_eq!(pinned.previous, None);
    assert_eq!(fs::read_to_string(acme.join(".envrc")).unwrap(), "use suite test 2\n");
    assert_eq!(h.direnv_runs(), [format!("allow {}", acme.join(".envrc").display())]);
}

#[test]
fn pinning_again_replaces_the_line_and_keeps_its_comment_and_the_rest() {
    let h = home();
    let acme = h.project("acme");
    write(
        &acme.join(".envrc"),
        "source_up\nuse suite test 1 # the old API\nexport A=1\nuse suite demo 4",
    );

    let pinned = h.pins.pin(&TEST, "3", &acme).unwrap();

    assert!(!pinned.created);
    assert_eq!(pinned.previous.as_deref(), Some("1"));
    assert_eq!(
        fs::read_to_string(acme.join(".envrc")).unwrap(),
        "source_up\nuse suite test 3 # the old API\nexport A=1\nuse suite demo 4\n"
    );
}

#[test]
fn a_pin_the_envrc_repeats_indents_or_puts_in_a_block_is_left_to_the_user() {
    let h = home();
    let cases = [
        (
            "if-else",
            "if has nvm; then\n  use suite test 1\nelse\n  use suite test 2\nfi\n",
            4,
            Tangle::Repeated,
        ),
        ("repeated", "use suite test 1\nuse suite test 2\n", 2, Tangle::Repeated),
        ("indented", "export A=1\n  use suite test 1\n", 2, Tangle::Indented),
        ("block", "if true; then\nuse suite test 1\nfi\n", 2, Tangle::InBlock),
        (
            "function",
            "legacy() {\nuse suite test 1\n}\nlegacy\n",
            2,
            Tangle::InBlock,
        ),
    ];

    for (name, text, at, tangle) in cases {
        let dir = h.project(name);
        write(&dir.join(".envrc"), text);

        let pinned = h.pins.pin(&TEST, "3", &dir).unwrap_err();
        let unpinned = h.pins.unpin("test", &dir).unwrap_err();

        for err in [pinned, unpinned] {
            let PinError::Tangled { line, why, .. } = err else {
                panic!("{name}: {err:?}");
            };
            assert_eq!((line, why), (at, tangle), "{name}");
        }
        assert_eq!(fs::read_to_string(dir.join(".envrc")).unwrap(), text, "{name}");
    }
    assert_eq!(h.direnv_runs(), Vec::<String>::new());
}

#[test]
fn a_one_line_block_before_the_pin_leaves_it_editable() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join(".envrc"), "if true; then export A=1; fi\nuse suite test 1\n");

    h.pins.pin(&TEST, "2", &acme).unwrap();

    assert_eq!(
        fs::read_to_string(acme.join(".envrc")).unwrap(),
        "if true; then export A=1; fi\nuse suite test 2\n"
    );
}

#[test]
fn pinning_the_version_already_pinned_leaves_the_file_and_allows_it() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join(".envrc"), "use suite test 2\n");
    let before = fs::metadata(acme.join(".envrc")).unwrap().modified().unwrap();

    let pinned = h.pins.pin(&TEST, "2", &acme).unwrap();

    assert_eq!(pinned.previous.as_deref(), Some("2"));
    assert_eq!(fs::metadata(acme.join(".envrc")).unwrap().modified().unwrap(), before);
    assert_eq!(h.direnv_runs().len(), 1);
}

#[test]
fn pinning_appends_after_what_the_envrc_already_does() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join(".envrc"), "source_up\nuse flake");

    h.pins.pin(&TEST, "1", &acme).unwrap();

    assert_eq!(
        fs::read_to_string(acme.join(".envrc")).unwrap(),
        "source_up\nuse flake\nuse suite test 1\n"
    );
}

#[test]
fn pinning_keeps_the_envrc_mode_and_writes_through_a_link() {
    let h = home();
    let acme = h.project("acme");
    let shared = h.root.join("shared.envrc");
    write(&shared, "export A=1\n");
    fs::set_permissions(&shared, fs::Permissions::from_mode(0o600)).unwrap();
    std::os::unix::fs::symlink(&shared, acme.join(".envrc")).unwrap();

    h.pins.pin(&TEST, "1", &acme).unwrap();

    assert!(fs::symlink_metadata(acme.join(".envrc")).unwrap().is_symlink());
    assert_eq!(fs::read_to_string(&shared).unwrap(), "export A=1\nuse suite test 1\n");
    assert_eq!(fs::metadata(&shared).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn pinning_a_version_the_suite_does_not_have_writes_nothing() {
    let h = home();
    let acme = h.project("acme");

    let err = h.pins.pin(&TEST, "9", &acme).unwrap_err();

    assert!(matches!(err, PinError::UnknownVersion { .. }), "{err:?}");
    assert!(!acme.join(".envrc").exists());
    assert_eq!(h.direnv_runs(), Vec::<String>::new());
}

#[test]
fn pinning_a_folder_that_does_not_exist_is_an_error() {
    let h = home();

    let err = h.pins.pin(&TEST, "1", &h.root.join("nowhere")).unwrap_err();

    assert!(matches!(err, PinError::NotAFolder { .. }), "{err:?}");
}

#[test]
fn a_pin_that_direnv_cannot_allow_is_written_and_reported() {
    let h = home();
    let acme = h.project("acme");
    let pins = Pins::new(&h.paths).with_direnv([h.root.join("no-direnv-here")]);

    let err = pins.pin(&TEST, "1", &acme).unwrap_err();

    assert!(matches!(err, PinError::Allow { .. }), "{err:?}");
    assert_eq!(fs::read_to_string(acme.join(".envrc")).unwrap(), "use suite test 1\n");
}

#[test]
fn unpinning_removes_the_line_and_keeps_the_rest() {
    let h = home();
    let acme = h.project("acme");
    write(&acme.join(".envrc"), "source_up\nuse suite test 2\nuse suite demo 4\n");

    let unpinned = h.pins.unpin("test", &acme).unwrap();

    assert_eq!(unpinned.version, "2");
    assert!(!unpinned.deleted);
    assert_eq!(
        fs::read_to_string(acme.join(".envrc")).unwrap(),
        "source_up\nuse suite demo 4\n"
    );
    assert_eq!(h.direnv_runs(), [format!("allow {}", acme.join(".envrc").display())]);
}

#[test]
fn unpinning_the_last_line_deletes_the_envrc() {
    let h = home();
    let acme = h.project("acme");
    h.pins.pin(&TEST, "2", &acme).unwrap();

    let unpinned = h.pins.unpin("test", &acme).unwrap();

    assert!(unpinned.deleted);
    assert!(!acme.join(".envrc").exists());
}

#[test]
fn unpinning_keeps_a_source_up_that_does_more() {
    let h = home();
    for (name, rest) in [
        ("and", "source_up && dotenv_if_exists .env.local"),
        ("other-file", "source_up_if_exists .envrc.secrets"),
    ] {
        let dir = h.project(name);
        write(&dir.join(".envrc"), &format!("{rest}\nuse suite test 1\n"));

        let unpinned = h.pins.unpin("test", &dir).unwrap();

        assert!(!unpinned.deleted, "{name}");
        assert_eq!(fs::read_to_string(dir.join(".envrc")).unwrap(), format!("{rest}\n"));
    }
}

#[test]
fn unpinning_deletes_the_suites_gc_roots_and_the_folders_left_empty() {
    let h = home();
    let acme = h.project("acme");
    let roots = acme.join(".direnv/suites");
    write(&acme.join(".envrc"), "use suite test 2\nuse suite demo 4\n");
    for root in ["test/1-bin", "test/2-bin", "demo/4"] {
        write(&roots.join(root), "");
    }

    h.pins.unpin("test", &acme).unwrap();
    let kept = roots.join("demo/4").exists();
    h.pins.unpin("demo", &acme).unwrap();

    assert!(kept, "another Suite's roots stay");
    assert!(!roots.join("test").exists());
    assert!(!acme.join(".direnv").exists());
}

#[test]
fn a_source_up_of_another_file_does_not_load_the_envrc_above() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/app");
    write(&acme.join(".envrc"), "use suite test 1\n");
    write(&app.join(".envrc"), "source_up_if_exists .envrc.secrets\n");

    let resolved = h.pins.resolve(&app).unwrap();

    assert_eq!(resolved.pins, Vec::<Pin>::new());
    assert_eq!(resolved.hidden.len(), 1);
}

#[test]
fn a_new_envrc_below_another_sources_it_and_goes_away_whole_on_unpin() {
    let h = home();
    let app = h.project("acme/old-app");
    let src = h.project("acme/old-app/src");
    write(&app.join(".envrc"), "use flake\nuse suite test 1\n");

    h.pins.pin(&TEST, "2", &src).unwrap();
    let pinned = fs::read_to_string(src.join(".envrc")).unwrap();
    let resolved = h.pins.resolve(&src).unwrap();
    let unpinned = h.pins.unpin("test", &src).unwrap();

    assert_eq!(pinned, "source_up\nuse suite test 2\n");
    assert_eq!(resolved.pins, [pin("test", "2", &src.join(".envrc"))]);
    assert!(unpinned.deleted);
    assert!(!src.join(".envrc").exists());
}

#[test]
fn unpinning_where_no_pin_is_written_names_the_folder_whose_pin_applies() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/old-app");
    write(&acme.join(".envrc"), "use suite test 1\n");

    let err = h.pins.unpin("test", &app).unwrap_err();

    let PinError::NotPinned { applies_from, .. } = err else {
        panic!("{err:?}");
    };
    assert_eq!(applies_from, Some(acme.join(".envrc")));
    assert_eq!(fs::read_to_string(acme.join(".envrc")).unwrap(), "use suite test 1\n");
}

#[test]
fn every_pin_under_the_projects_root_is_listed_but_not_in_hidden_or_dependency_folders() {
    let h = home();
    let acme = h.project("acme");
    let app = h.project("acme/new-app");
    let skipped = [h.project("acme/new-app/node_modules/x"), h.project("acme/.git/x")];
    write(&acme.join(".envrc"), "use suite test 1\n");
    write(&app.join(".envrc"), "source_up\nuse suite test 2\n");
    for dir in &skipped {
        write(&dir.join(".envrc"), "use suite test 3\n");
    }
    write(&h.project("plain").join(".envrc"), "use flake\n");

    let all = h.pins.all().unwrap();

    assert_eq!(
        all,
        [
            pin("test", "1", &acme.join(".envrc")),
            pin("test", "2", &app.join(".envrc"))
        ]
    );
}
