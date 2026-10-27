use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::*;
use crate::fake::FakeRunner;

fn no_arguments() -> BTreeMap<String, String> {
    BTreeMap::new()
}

const SCREENS: &str = "0\teDP-1\tpanel\tBOE 0x0BCA\n\
                       1\t\t3\tBNQ BenQ GW2480 ET00000000SL0\n\
                       3\tDP-8\t2\tDell Inc. DELL U2724D 1A2B3C4\n\
                       4\tDP-9\t1\tDell Inc. DELL U2724D 5D6E7F8\n";

fn titles(desk: &Desk) -> Vec<String> {
    desk.items().into_iter().map(|item| item.title).collect()
}

#[test]
fn each_position_is_an_item_the_panel_first_then_from_the_left() {
    let desk = Desk::parse(SCREENS);

    assert_eq!(
        titles(&desk),
        [
            "Workspace group: laptop panel",
            "Workspace group 1",
            "Workspace group 2",
            "Workspace group 3",
        ]
    );
    let items = desk.items();
    assert_eq!(items[1].id, "desk:1");
    assert_eq!(items[1].kind.name(), KIND);
    assert!(items[1].keywords.contains(&"workspace group".to_owned()));
}

#[test]
fn an_item_names_its_workspaces_and_the_screen_standing_there() {
    let items = Desk::parse(SCREENS).items();

    assert_eq!(
        items[0].subtitle.as_deref(),
        Some("Workspaces 1-10, on BOE 0x0BCA (eDP-1)")
    );
    assert_eq!(
        items[1].subtitle.as_deref(),
        Some("Workspaces 41-50, on Dell Inc. DELL U2724D 5D6E7F8 (DP-9)")
    );
}

#[test]
fn a_position_no_screen_holds_says_its_group_waits_and_where_it_was_last() {
    let items = Desk::parse(SCREENS).items();

    assert_eq!(
        items[3].subtitle.as_deref(),
        Some("Workspaces 11-20, waiting for a screen here; last on BNQ BenQ GW2480 ET00000000SL0")
    );
}

#[test]
fn a_file_keyed_by_screen_gives_no_items_until_hyprland_migrates_it() {
    let desk = Desk::parse("0\teDP-1\tBOE 0x0BCA\n1\t\tDell Inc. DELL U2724D 5D6E7F8\n");

    assert!(desk.items().is_empty());
}

#[test]
fn no_screens_file_means_no_items() {
    assert!(
        Desk::read(Path::new("/nonexistent/workspace-screens"))
            .items()
            .is_empty()
    );
}

#[test]
fn reading_the_file_gives_its_positions() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("workspace-screens");
    fs::write(&file, SCREENS).unwrap();

    assert_eq!(Desk::read(&file).items().len(), 4);
}

#[test]
fn going_to_a_position_asks_hyprland_and_closes_the_menu() {
    let runner = FakeRunner::default();

    let ran = Desk::parse(SCREENS).run("desk:2", SHOW, &no_arguments(), &runner);

    assert!(matches!(ran, Ok(Ran::Close)));
    assert_eq!(
        runner.launched.borrow()[0].0,
        ["hyprctl", "dispatch", "dot.workspaces.go_to_position(\"2\")"]
    );
}

#[test]
fn an_unknown_position_or_action_is_refused() {
    let runner = FakeRunner::default();

    assert!(
        Desk::parse(SCREENS)
            .run("desk:9", SHOW, &no_arguments(), &runner)
            .is_err()
    );
    assert!(
        Desk::parse(SCREENS)
            .run("desk:1", "unplug", &no_arguments(), &runner)
            .is_err()
    );
    assert!(runner.launched.borrow().is_empty());
}

// The file Kuba's laptop had: each external named at the position it stands at and again waiting
// at a position the old migration invented for it.
const DOUBLED: &str = "0\teDP-1\tpanel\tCSOT MNE007ZA3-2\n\
                       1\t\t3\tLG ULTRAGEAR+ 510NTCZN7787\n\
                       2\t\t4\tBNQ ZOWIE XL G4L01663SL0\n\
                       3\tHDMI-A-1\t2\tLG ULTRAGEAR+ 510NTCZN7787\n\
                       4\tDP-2\t1\tBNQ ZOWIE XL G4L01663SL0\n";

#[test]
fn a_screen_standing_at_a_position_gets_no_waiting_item_elsewhere() {
    let desk = Desk::parse(DOUBLED);

    assert_eq!(
        titles(&desk),
        [
            "Workspace group: laptop panel",
            "Workspace group 1",
            "Workspace group 2",
        ]
    );
    assert_eq!(
        desk.items()[1].subtitle.as_deref(),
        Some("Workspaces 41-50, on BNQ ZOWIE XL G4L01663SL0 (DP-2)")
    );
}

#[test]
fn a_screen_waiting_at_two_positions_gets_one_item_the_first_from_the_left() {
    let desk = Desk::parse("0\teDP-1\tpanel\tBOE 0x0BCA\n1\t\t4\tDell U2724D 1A2B3C4\n2\t\t2\tDell U2724D 1A2B3C4\n");

    assert_eq!(titles(&desk), ["Workspace group: laptop panel", "Workspace group 2",]);
}

#[test]
fn a_position_named_twice_keeps_its_first_line() {
    let items = Desk::parse("0\teDP-1\tpanel\tBOE 0x0BCA\n3\tDP-1\t1\tA\n5\tDP-2\t1\tB\n").items();

    assert_eq!(items.len(), 2);
    assert_eq!(items[1].subtitle.as_deref(), Some("Workspaces 31-40, on A (DP-1)"));
}

#[test]
fn a_position_named_twice_apart_keeps_its_first_line() {
    let items = Desk::parse("3\tDP-1\t1\tA\n1\t\t2\tB\n5\tDP-2\t1\tC\n").items();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].subtitle.as_deref(), Some("Workspaces 31-40, on A (DP-1)"));
    assert_eq!(items[1].id, "desk:2");
}

// Kuba at home, with his groups: the BenQ left, the LG in the centre, the panel; the left group
// named.
const HOME: &str = "0\teDP-1\tpanel\tCSOT MNE007ZA3-2\n\
                    3\tHDMI-A-1\t2\tLG ULTRAGEAR+ 510NTCZN7787\n\
                    4\tDP-2\t1\tBNQ ZOWIE XL G4L01663SL0\tLeft\n";
const HOME_GROUPS: &str = "# Mine.\n[group.1]\nname = \"Left\"\nscreens = [\"BNQ ZOWIE XL G4L01663SL0\"]\n";

fn home() -> (tempfile::TempDir, PathBuf, Desk) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(groups::FILE);
    fs::write(&file, HOME_GROUPS).unwrap();
    let desk = Desk::parse(HOME).with_groups_file(file.clone());
    (dir, file, desk)
}

fn ids(desk: &Desk) -> Vec<String> {
    desk.items().into_iter().map(|item| item.id).collect()
}

fn find(desk: &Desk, id: &str) -> Item {
    desk.items()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap_or_else(|| panic!("no Item {id}"))
}

#[test]
fn a_named_group_says_its_name() {
    let desk = Desk::parse(HOME);

    assert_eq!(
        titles(&desk),
        [
            "Workspace group: laptop panel",
            "Workspace group 1: Left",
            "Workspace group 2",
        ]
    );
}

#[test]
fn each_screen_plugged_in_can_be_pinned_to_each_group_and_a_new_one() {
    let (_dir, _file, desk) = home();

    let pins: Vec<String> = ids(&desk)
        .into_iter()
        .filter(|id| id.starts_with("desk-pin:"))
        .collect();

    assert_eq!(
        pins,
        [
            "desk-pin:1:CSOT MNE007ZA3-2",
            "desk-pin:2:CSOT MNE007ZA3-2",
            "desk-pin:3:CSOT MNE007ZA3-2",
            "desk-pin:2:BNQ ZOWIE XL G4L01663SL0",
            "desk-pin:3:BNQ ZOWIE XL G4L01663SL0",
            "desk-pin:1:LG ULTRAGEAR+ 510NTCZN7787",
            "desk-pin:2:LG ULTRAGEAR+ 510NTCZN7787",
            "desk-pin:3:LG ULTRAGEAR+ 510NTCZN7787",
        ]
    );
    assert_eq!(
        find(&desk, "desk-pin:1:LG ULTRAGEAR+ 510NTCZN7787").title,
        "Pin LG ULTRAGEAR+ 510NTCZN7787 to group 1: Left"
    );
    assert_eq!(
        find(&desk, "desk-pin:3:LG ULTRAGEAR+ 510NTCZN7787").title,
        "Pin LG ULTRAGEAR+ 510NTCZN7787 to a new group 3"
    );
    assert_eq!(
        find(&desk, "desk-pin:2:LG ULTRAGEAR+ 510NTCZN7787").subtitle.as_deref(),
        Some("On HDMI-A-1; it shows this group whenever it is plugged in, on any port, wherever it stands")
    );
}

#[test]
fn a_pinned_screen_can_be_unpinned_and_each_group_named() {
    let (_dir, _file, desk) = home();

    let unpin = find(&desk, "desk-unpin:BNQ ZOWIE XL G4L01663SL0");
    let name = find(&desk, "desk-name:1");

    assert_eq!(unpin.title, "Unpin BNQ ZOWIE XL G4L01663SL0");
    assert_eq!(
        unpin.subtitle.as_deref(),
        Some("From group 1: Left; it takes the next group left to right again")
    );
    assert_eq!(name.title, "Name group 1…");
    assert_eq!(name.actions[0].arguments[0].placeholder.as_deref(), Some("Left"));
    assert!(name.actions[0].arguments[0].optional);
    assert_eq!(find(&desk, "desk-name:2").title, "Name group 2…");
}

#[test]
fn pinning_from_the_menu_writes_the_groups_file_and_closes_it() {
    let (_dir, file, desk) = home();
    let runner = FakeRunner::default();

    let ran = desk.run("desk-pin:2:LG ULTRAGEAR+ 510NTCZN7787", PIN, &no_arguments(), &runner);

    assert!(matches!(ran, Ok(Ran::Close)));
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        format!("{HOME_GROUPS}\n[group.2]\nscreens = [\"LG ULTRAGEAR+ 510NTCZN7787\"]\n")
    );
    assert_eq!(launched(&runner), [RELOAD]);
}

const RELOAD: [&str; 3] = ["hyprctl", "dispatch", "dot.workspaces.reload_groups()"];

fn launched(runner: &FakeRunner) -> Vec<Vec<String>> {
    runner.launched.borrow().iter().map(|(argv, _)| argv.clone()).collect()
}

// Regression: with the shell down, a pin from the menu waited for the next `hyprctl reload`.
#[test]
fn every_edit_from_the_menu_tells_hyprland_itself() {
    let (_dir, _file, desk) = home();
    let runner = FakeRunner::default();
    let named = BTreeMap::from([(NAME_ARGUMENT.to_owned(), "Centre".to_owned())]);

    desk.run("desk-name:2", NAME, &named, &runner).unwrap();
    desk.run("desk-unpin:BNQ ZOWIE XL G4L01663SL0", UNPIN, &no_arguments(), &runner)
        .unwrap();

    assert_eq!(launched(&runner), [RELOAD, RELOAD]);
}

#[test]
fn an_edit_refused_tells_hyprland_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(groups::FILE);
    fs::write(&file, "[group.1]\nscreens = [\"A\"]\n").unwrap();
    let desk = Desk::parse(HOME).with_groups_file(file.clone());
    fs::write(&file, "[group.1]\nscreen = []\n").unwrap();
    let runner = FakeRunner::default();

    assert!(desk.run("desk-name:1", NAME, &no_arguments(), &runner).is_err());
    assert!(runner.launched.borrow().is_empty());
}

// Two screens with no serial report one description, and a pin names a description, so it can't
// tell them apart.
#[test]
fn twins_reporting_one_description_get_no_pin_items() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(groups::FILE);
    fs::write(&file, "").unwrap();
    let twins = "0\teDP-1\tpanel\tCSOT MNE007ZA3-2\n1\tDP-1\t1\tGeneric Monitor\n2\tDP-2\t2\tGeneric Monitor\n\
                 3\tDP-3\t3\tGeneric Monitor #2\n";

    let desk = Desk::parse(twins).with_groups_file(file);

    let pins: Vec<String> = ids(&desk)
        .into_iter()
        .filter(|id| id.starts_with("desk-pin:"))
        .collect();
    assert_eq!(
        pins,
        [
            "desk-pin:1:CSOT MNE007ZA3-2",
            "desk-pin:2:CSOT MNE007ZA3-2",
            "desk-pin:3:CSOT MNE007ZA3-2",
            "desk-pin:4:CSOT MNE007ZA3-2"
        ]
    );
}

#[test]
fn unpinning_and_naming_from_the_menu_write_the_groups_file() {
    let (_dir, file, desk) = home();
    let runner = FakeRunner::default();
    let named = BTreeMap::from([(NAME_ARGUMENT.to_owned(), "Centre".to_owned())]);

    desk.run("desk-name:2", NAME, &named, &runner).unwrap();
    desk.run("desk-unpin:BNQ ZOWIE XL G4L01663SL0", UNPIN, &no_arguments(), &runner)
        .unwrap();

    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "# Mine.\n[group.1]\nname = \"Left\"\nscreens = []\n\n[group.2]\nname = \"Centre\"\n"
    );
}

#[test]
fn a_groups_file_that_doesnt_read_gives_no_pin_or_name_items() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(groups::FILE);
    fs::write(&file, "[group.1]\nscreen = []\n").unwrap();

    let desk = Desk::parse(HOME).with_groups_file(file);

    assert_eq!(ids(&desk), ["desk:panel", "desk:1", "desk:2"]);
}

#[test]
fn an_action_that_isnt_the_items_is_refused() {
    let (_dir, file, desk) = home();
    let runner = FakeRunner::default();

    assert!(
        desk.run("desk-pin:2:LG ULTRAGEAR+ 510NTCZN7787", UNPIN, &no_arguments(), &runner)
            .is_err()
    );
    assert!(
        desk.run("desk-pin:9:LG ULTRAGEAR+ 510NTCZN7787", PIN, &no_arguments(), &runner)
            .is_err()
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), HOME_GROUPS);
}
