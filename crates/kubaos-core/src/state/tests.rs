use std::fs;
use std::path::Path;

use super::Origin;
use super::Paths;
use super::ReadError;
use super::StateDir;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
    fs::write(path, contents).expect("write the file");
}

#[test]
fn a_missing_state_directory_declares_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let state = StateDir::new(tmp.path().join("kubaos.d"));

    let declared = state.declared().unwrap();

    assert!(declared.is_empty());
}

#[test]
fn packages_toml_lists_the_packages() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        &tmp.path().join("packages.toml"),
        "packages = [\"ripgrep\", \"hello\"]\n",
    );

    let declared = StateDir::new(tmp.path()).declared().unwrap();

    assert_eq!(declared.names(), ["hello", "ripgrep"]);
    assert_eq!(declared.origins("ripgrep"), [Origin::Packages]);
}

#[test]
fn a_file_without_a_packages_key_declares_none() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        &tmp.path().join("suites/go.toml"),
        "version = \"1.25\"\n[options]\nlsp = true\n",
    );

    let declared = StateDir::new(tmp.path()).declared().unwrap();

    assert!(declared.is_empty());
}

#[test]
fn two_files_listing_the_same_package_give_the_union() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("packages.toml"), "packages = [\"jq\", \"gopls\"]\n");
    write(
        &tmp.path().join("suites/go.toml"),
        "version = \"1.25\"\npackages = [\"go\", \"gopls\"]\n",
    );
    write(&tmp.path().join("suites/node.toml"), "packages = [\"nodejs\"]\n");

    let declared = StateDir::new(tmp.path()).declared().unwrap();

    assert_eq!(declared.names(), ["go", "gopls", "jq", "nodejs"]);
    assert_eq!(
        declared.origins("gopls"),
        [Origin::Packages, Origin::Suite("go".into())]
    );
    assert_eq!(declared.origins("nodejs"), [Origin::Suite("node".into())]);
}

#[test]
fn only_toml_files_in_suites_are_read() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("suites/go.toml.bak"), "packages = [\"go\"]\n");
    write(&tmp.path().join("suites/README"), "not toml at all");
    write(&tmp.path().join("other.toml"), "packages = [\"hello\"]\n");

    let declared = StateDir::new(tmp.path()).declared().unwrap();

    assert!(declared.is_empty());
}

#[test]
fn a_malformed_file_is_a_parse_error_naming_it() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("suites/go.toml"), "packages = [\"go\"\n");

    let err = StateDir::new(tmp.path()).declared().unwrap_err();

    let ReadError::Parse { path, .. } = &err else {
        panic!("{err:?}")
    };
    assert!(path.ends_with("suites/go.toml"), "{path:?}");
}

#[test]
fn a_packages_key_of_the_wrong_type_is_a_parse_error() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("packages.toml"), "packages = \"ripgrep\"\n");

    let err = StateDir::new(tmp.path()).declared().unwrap_err();

    assert!(matches!(err, ReadError::Parse { .. }), "{err:?}");
}

#[test]
fn paths_follow_the_xdg_variables_and_fall_back_to_home() {
    let defaults = Paths::from_vars(Some("/home/kuba".into()), None, None).unwrap();
    let custom = Paths::from_vars(Some("/home/kuba".into()), Some("/cfg".into()), Some("/st".into())).unwrap();

    assert_eq!(defaults.flake(), Path::new("/home/kuba/.config/kubaos/home"));
    assert_eq!(
        defaults.state_dir().root(),
        Path::new("/home/kuba/.config/kubaos/home/kubaos.d")
    );
    assert_eq!(
        defaults.home_nix(),
        Path::new("/home/kuba/.config/kubaos/home/home.nix")
    );
    assert_eq!(defaults.jobs(), Path::new("/home/kuba/.local/state/kubaos/jobs"));
    assert_eq!(custom.flake(), Path::new("/cfg/kubaos/home"));
    assert_eq!(custom.jobs(), Path::new("/st/kubaos/jobs"));
}

#[test]
fn paths_name_the_user_layer_the_projects_root_and_the_backups() {
    let defaults = Paths::from_vars(Some("/home/kuba".into()), None, None).unwrap();
    let custom = Paths::from_vars(Some("/home/kuba".into()), Some("/cfg".into()), Some("/st".into())).unwrap();

    assert_eq!(defaults.home(), Path::new("/home/kuba"));
    assert_eq!(defaults.user_layer(), Path::new("/home/kuba/.config/kubaos"));
    assert_eq!(defaults.projects_root(), Path::new("/home/kuba/Projects"));
    assert_eq!(defaults.backups(), Path::new("/home/kuba/.local/state/kubaos/backups"));
    assert_eq!(custom.user_layer(), Path::new("/cfg/kubaos"));
    assert_eq!(custom.backups(), Path::new("/st/kubaos/backups"));
    assert!(!defaults.backups().starts_with(defaults.flake()));
}

#[test]
fn the_projects_root_moves_to_an_absolute_path_or_one_under_the_home() {
    let home = || Paths::from_vars(Some("/home/kuba".into()), None, None).unwrap();

    let absolute = home().with_projects_root("/srv/code");
    let relative = home().with_projects_root("src");

    assert_eq!(absolute.projects_root(), Path::new("/srv/code"));
    assert_eq!(relative.projects_root(), Path::new("/home/kuba/src"));
    assert_eq!(relative.flake(), home().flake());
}

#[test]
fn the_suite_toolchains_live_in_the_data_home() {
    let home = || Paths::from_vars(Some("/home/kuba".into()), None, None).unwrap();

    let moved = home().with_data_home("/data");

    assert_eq!(
        home().toolchains(),
        Path::new("/home/kuba/.local/share/kubaos/toolchains")
    );
    assert_eq!(moved.toolchains(), Path::new("/data/kubaos/toolchains"));
}

#[test]
fn paths_need_a_home() {
    assert!(Paths::from_vars(None, None, None).is_err());
}

#[test]
fn restoring_from_a_generation_brings_back_its_state_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let generation = tmp.path().join("home-manager-41-link");
    let saved = generation.join("home-files/.local/share/kubaos/kubaos.d");
    write(&saved.join("packages.toml"), "packages = [\"hello\"]\n");
    write(&saved.join("suites/go.toml"), "packages = [\"go\"]\n");
    let state = StateDir::new(tmp.path().join("kubaos.d"));
    write(&state.root().join("packages.toml"), "packages = [\"ripgrep\"]\n");
    write(&state.root().join("suites/node.toml"), "packages = [\"nodejs\"]\n");

    state.restore_from_generation(&generation).unwrap();

    assert_eq!(state.declared().unwrap().names(), ["go", "hello"]);
    assert!(!state.root().join("suites/node.toml").exists());
    let mode = fs::metadata(state.root().join("packages.toml")).unwrap().permissions();
    assert!(!mode.readonly(), "a restored file stays writable for the next add");
}

#[test]
fn restoring_a_generation_that_had_no_state_directory_empties_it() {
    let tmp = tempfile::tempdir().unwrap();
    let generation = tmp.path().join("home-manager-3-link");
    fs::create_dir_all(generation.join("home-files")).unwrap();
    let state = StateDir::new(tmp.path().join("kubaos.d"));
    write(&state.root().join("packages.toml"), "packages = [\"ripgrep\"]\n");

    state.restore_from_generation(&generation).unwrap();

    assert!(state.declared().unwrap().is_empty());
    assert!(state.root().is_dir());
}

#[test]
fn restoring_from_something_that_is_not_a_generation_changes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let state = StateDir::new(tmp.path().join("kubaos.d"));
    write(&state.root().join("packages.toml"), "packages = [\"ripgrep\"]\n");

    let result = state.restore_from_generation(&tmp.path().join("nope"));

    assert!(result.is_err());
    assert_eq!(state.declared().unwrap().names(), ["ripgrep"]);
}
