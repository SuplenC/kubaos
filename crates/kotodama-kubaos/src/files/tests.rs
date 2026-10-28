use std::fs;

use super::*;
use crate::fake::FakeRunner;

const HOME: &str = "/home/kuba";

fn every_folder_exists(_: &Path) -> bool {
    true
}

fn titles(folders: &UserFolders) -> Vec<String> {
    folders.items().into_iter().map(|item| item.title).collect()
}

#[test]
fn without_user_dirs_the_english_folders_that_exist_are_items() {
    let folders = UserFolders::parse(Path::new(HOME), None, |path| {
        path.ends_with("Downloads") || path.ends_with("Pictures")
    });

    assert_eq!(titles(&folders), ["Files: Downloads", "Files: Pictures"]);
    let item = &folders.items()[0];
    assert_eq!(item.id, "files:download");
    assert_eq!(item.kind.name(), KIND);
    assert_eq!(item.subtitle.as_deref(), Some("~/Downloads"));
}

#[test]
fn user_dirs_names_each_folder_under_the_home_or_anywhere() {
    let text = "# This file is written by xdg-user-dirs-update\n\
                XDG_DOWNLOAD_DIR=\"$HOME/Scaricati\"\n\
                XDG_PICTURES_DIR=\"/data/photos\"\n";

    let folders = UserFolders::parse(Path::new(HOME), Some(text), every_folder_exists);

    assert_eq!(titles(&folders), ["Files: Scaricati", "Files: photos"]);
    let items = folders.items();
    assert!(items[0].keywords.contains(&"Downloads".to_owned()));
    assert_eq!(items[1].subtitle.as_deref(), Some("/data/photos"));
}

#[test]
fn a_folder_set_to_the_home_is_turned_off() {
    let text = "XDG_DESKTOP_DIR=\"$HOME/\"\nXDG_MUSIC_DIR=\"$HOME\"\nXDG_VIDEOS_DIR=\"$HOME/Videos\"\n";

    let folders = UserFolders::parse(Path::new(HOME), Some(text), every_folder_exists);

    assert_eq!(titles(&folders), ["Files: Videos"]);
}

#[test]
fn a_folder_user_dirs_leaves_out_is_no_item() {
    let folders = UserFolders::parse(Path::new(HOME), Some(""), every_folder_exists);

    assert_eq!(folders.items(), [] as [kotodama_protocol::Item; 0]);
}

#[test]
fn open_starts_files_on_the_folder_and_closes_the_menu() {
    let folders = UserFolders::parse(Path::new(HOME), None, every_folder_exists);
    let runner = FakeRunner::default();

    let ran = folders.run("files:documents", OPEN, &runner);

    assert!(matches!(ran, Ok(Ran::Close)));
    let launched = runner.launched.borrow();
    assert_eq!(launched[0].0, ["nautilus", "--new-window", "/home/kuba/Documents"]);
}

#[test]
fn the_second_action_opens_the_folder_in_yazi() {
    let folders = UserFolders::parse(Path::new(HOME), None, every_folder_exists);
    let runner = FakeRunner::default();

    let ran = folders.run("files:documents", YAZI, &runner);

    assert!(matches!(ran, Ok(Ran::Close)));
    assert_eq!(
        runner.launched.borrow()[0].0,
        ["kubaos-files", "open", "/home/kuba/Documents"]
    );
    let actions: Vec<String> = folders.items()[0]
        .actions
        .iter()
        .map(|action| action.id.clone())
        .collect();
    assert_eq!(actions, [OPEN, YAZI]);
}

#[test]
fn an_unknown_folder_or_action_is_refused() {
    let folders = UserFolders::parse(Path::new(HOME), None, |path| path.ends_with("Music"));
    let runner = FakeRunner::default();

    assert!(folders.run("files:documents", OPEN, &runner).is_err());
    assert!(folders.run("files:music", "delete", &runner).is_err());
    assert!(runner.launched.borrow().is_empty());
}

#[test]
fn read_keeps_only_the_folders_on_disk() {
    let home = tempfile::tempdir().unwrap();
    let config = home.path().join(".config");
    fs::create_dir_all(home.path().join("Downloads")).unwrap();
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join(USER_DIRS),
        "XDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\nXDG_DOCUMENTS_DIR=\"$HOME/Documents\"\n",
    )
    .unwrap();

    let folders = UserFolders::read(home.path(), &config);

    assert_eq!(titles(&folders), ["Files: Downloads"]);
}
