use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::App;
use super::Group;
use super::config_home_from;
use super::dir;
use super::root;

/// The seeds, `user-layer/` in the repo: the tree a fresh user layer starts as.
fn seeds() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../user-layer")
}

fn names(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .expect("read a seeds folder")
        .map(|entry| entry.expect("a seeds entry").file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn an_app_lives_in_its_group_under_its_own_name() {
    assert_eq!(App::Clock.relative(), Path::new("desktop/clock"));
    assert_eq!(App::Ghostty.relative(), Path::new("terminal/ghostty"));
    assert_eq!(App::Nvim.relative(), Path::new("editor/nvim"));
    assert_eq!(App::Yazi.relative(), Path::new("tools/yazi"));
    assert_eq!(App::Kotodama.relative(), Path::new("menu/kotodama"));
    assert_eq!(App::Phone.relative(), Path::new("desktop/phone"));
}

#[test]
fn every_app_has_its_own_folder() {
    let folders: BTreeSet<PathBuf> = App::ALL.iter().map(|app| app.relative()).collect();
    let names: BTreeSet<&str> = App::ALL.iter().map(|app| app.name()).collect();

    assert_eq!(folders.len(), App::ALL.len());
    assert_eq!(names.len(), App::ALL.len());
}

#[test]
fn an_app_is_found_by_its_name() {
    for app in App::ALL {
        assert_eq!(App::from_name(app.name()), Some(app));
    }
    assert_eq!(App::from_name("home"), None);
}

#[test]
fn the_seeds_are_grouped_as_the_table_says() {
    let seeds = seeds();
    let groups: BTreeSet<String> = Group::ALL.iter().map(|group| group.name().to_owned()).collect();
    let mut top = names(&seeds);
    top.remove("README.md");
    assert_eq!(top, groups, "the seeds' top level is the groups and the map");

    for group in Group::ALL {
        let mut found = names(&seeds.join(group.name()));
        found.remove("README.md");
        for app in found {
            let known = App::from_name(&app).unwrap_or_else(|| panic!("user-layer/{}/{app} is no App", group.name()));
            assert_eq!(
                known.group(),
                group,
                "user-layer/{}/{app} is in another group",
                group.name()
            );
        }
    }
}

#[test]
fn every_group_has_a_readme() {
    for group in Group::ALL {
        assert!(
            seeds().join(group.name()).join("README.md").is_file(),
            "no README in {}",
            group.name()
        );
    }
    assert!(seeds().join("README.md").is_file());
}

#[test]
fn the_user_layer_is_kubaos_under_the_config_home() {
    let config = Path::new("/home/kuba/.config");

    assert_eq!(root(config), Path::new("/home/kuba/.config/kubaos"));
    assert_eq!(
        dir(config, App::Keyboard),
        Path::new("/home/kuba/.config/kubaos/desktop/keyboard")
    );
}

#[test]
fn the_config_home_is_xdg_config_home_else_dot_config_in_the_home() {
    let xdg = Some(OsString::from("/cfg"));
    let home = Some(OsString::from("/home/kuba"));

    assert_eq!(config_home_from(xdg, home.clone()), Some(PathBuf::from("/cfg")));
    assert_eq!(
        config_home_from(Some(OsString::new()), home.clone()),
        Some(PathBuf::from("/home/kuba/.config"))
    );
    assert_eq!(config_home_from(None, home), Some(PathBuf::from("/home/kuba/.config")));
    assert_eq!(config_home_from(None, None), None);
}
