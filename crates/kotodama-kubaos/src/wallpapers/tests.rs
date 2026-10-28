use std::fs;

use super::*;
use crate::fake::FakeRunner;

fn sets(file: Option<&str>) -> WallpaperSets {
    WallpaperSets::parse(["shinjuku", "cyberpunk", "japanese"].map(str::to_owned), file)
}

fn titles(sets: &WallpaperSets) -> Vec<String> {
    sets.items().into_iter().map(|item| item.title).collect()
}

fn item(sets: &WallpaperSets, id: &str) -> Item {
    sets.items().into_iter().find(|item| item.id == id).expect("the Item")
}

fn actions(sets: &WallpaperSets, id: &str) -> Vec<String> {
    item(sets, id).actions.into_iter().map(|action| action.id).collect()
}

#[test]
fn every_set_on_disk_is_an_item_by_its_name_in_order() {
    let sets = sets(None);

    assert_eq!(
        titles(&sets),
        [
            "Wallpaper set: Cyberpunk",
            "Wallpaper set: Japanese",
            "Wallpaper set: Shinjuku"
        ]
    );
    let items = sets.items();
    assert!(items.iter().all(|item| item.kind.name() == KIND));
    assert_eq!(items[0].id, "wallpaper-set:cyberpunk");
}

#[test]
fn without_the_sets_file_every_set_is_shown_whole() {
    let sets = sets(None);

    let shinjuku = item(&sets, "wallpaper-set:shinjuku");

    assert_eq!(shinjuku.subtitle.as_deref(), Some("In the wallpaper picker, all of it"));
    assert_eq!(actions(&sets, "wallpaper-set:shinjuku"), [HIDE]);
}

#[test]
fn a_set_out_of_the_file_is_hidden_and_can_be_shown() {
    let sets = sets(Some("# a comment\nshinjuku\n"));

    let japanese = item(&sets, "wallpaper-set:japanese");

    assert_eq!(japanese.subtitle.as_deref(), Some("Hidden from the wallpaper picker"));
    assert_eq!(actions(&sets, "wallpaper-set:japanese"), [SHOW]);
}

#[test]
fn a_set_picked_one_wallpaper_at_a_time_counts_them_and_can_be_shown_whole() {
    let sets = sets(Some("shinjuku/a.jpg\nshinjuku/b.jpg\njapanese\n"));

    let shinjuku = item(&sets, "wallpaper-set:shinjuku");

    assert_eq!(
        shinjuku.subtitle.as_deref(),
        Some("In the wallpaper picker, 2 of its wallpapers")
    );
    assert_eq!(actions(&sets, "wallpaper-set:shinjuku"), [SHOW, HIDE]);
}

#[test]
fn a_set_listed_whole_is_whole_whatever_else_is_picked() {
    let sets = sets(Some("shinjuku/a.jpg\nshinjuku\n"));

    assert_eq!(actions(&sets, "wallpaper-set:shinjuku"), [HIDE]);
}

#[test]
fn without_sets_on_disk_there_are_no_items() {
    let sets = WallpaperSets::parse(Vec::new(), Some("shinjuku\n"));

    assert_eq!(sets.items(), [] as [kotodama_protocol::Item; 0]);
}

#[test]
fn show_and_hide_run_dot_theme_and_keep_the_menu_open() {
    let runner = FakeRunner::default();
    let sets = sets(Some("shinjuku\ncyberpunk/a.jpg\n"));

    assert_eq!(sets.run("wallpaper-set:japanese", SHOW, &runner), Ok(Ran::KeepOpen));
    assert_eq!(sets.run("wallpaper-set:shinjuku", HIDE, &runner), Ok(Ran::KeepOpen));
    assert_eq!(sets.run("wallpaper-set:cyberpunk", SHOW, &runner), Ok(Ran::KeepOpen));

    assert_eq!(
        runner.ran(),
        [
            "dot-theme show-set japanese",
            "dot-theme hide-set shinjuku",
            "dot-theme show-set cyberpunk"
        ]
    );
}

#[test]
fn the_last_set_in_the_picker_is_not_hidden() {
    let runner = FakeRunner::default();
    let sets = sets(Some("japanese/a.jpg\n"));

    let ran = sets.run("wallpaper-set:japanese", HIDE, &runner);

    assert_eq!(
        ran,
        Err("At least one wallpaper set stays in the picker. Nice try".to_owned())
    );
    assert_eq!(runner.ran(), [] as [std::string::String; 0]);
}

#[test]
fn an_action_the_set_does_not_offer_is_refused() {
    let runner = FakeRunner::default();
    let sets = sets(Some("shinjuku\n"));

    assert!(sets.run("wallpaper-set:shinjuku", SHOW, &runner).is_err());
    assert!(sets.run("wallpaper-set:japanese", HIDE, &runner).is_err());
    assert!(sets.run("wallpaper-set:gone", SHOW, &runner).is_err());
    assert!(sets.run("shinjuku", SHOW, &runner).is_err());
    assert_eq!(runner.ran(), [] as [std::string::String; 0]);
}

#[test]
fn a_failing_dot_theme_says_why() {
    let runner = FakeRunner {
        failing: Some("dot-theme: no set 'japanese'".to_owned()),
        ..FakeRunner::default()
    };

    let ran = sets(Some("shinjuku\n")).run("wallpaper-set:japanese", SHOW, &runner);

    assert!(ran.is_err_and(|message| message.contains("no set 'japanese'")));
}

#[test]
fn read_takes_the_set_folders_but_yours_and_the_sets_file() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let on_disk = dir.path().join("wallpapers");
    for set in ["shinjuku", "japanese", "yours"] {
        fs::create_dir_all(on_disk.join(set)).expect("make a set folder");
    }
    fs::write(on_disk.join("README.md"), "").expect("write a file beside the sets");
    let file = dir.path().join("sets");
    fs::write(&file, "japanese\n").expect("write the sets file");

    let sets = WallpaperSets::read(&on_disk, &file);

    assert_eq!(titles(&sets), ["Wallpaper set: Japanese", "Wallpaper set: Shinjuku"]);
    assert_eq!(actions(&sets, "wallpaper-set:shinjuku"), [SHOW]);
}

#[test]
fn read_without_the_sets_file_shows_every_set() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    fs::create_dir_all(dir.path().join("shinjuku")).expect("make a set folder");

    let sets = WallpaperSets::read(dir.path(), &dir.path().join("missing"));

    assert_eq!(actions(&sets, "wallpaper-set:shinjuku"), [HIDE]);
}
