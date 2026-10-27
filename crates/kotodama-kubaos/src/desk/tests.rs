use std::fs;
use std::path::Path;

use super::*;
use crate::fake::FakeRunner;

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
            "Workspace group: position 1 from the left",
            "Workspace group: position 2 from the left",
            "Workspace group: position 3 from the left",
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

    let ran = Desk::parse(SCREENS).run("desk:2", SHOW, &runner);

    assert!(matches!(ran, Ok(Ran::Close)));
    assert_eq!(
        runner.launched.borrow()[0].0,
        ["hyprctl", "dispatch", "dot.workspaces.go_to_position(\"2\")"]
    );
}

#[test]
fn an_unknown_position_or_action_is_refused() {
    let runner = FakeRunner::default();

    assert!(Desk::parse(SCREENS).run("desk:9", SHOW, &runner).is_err());
    assert!(Desk::parse(SCREENS).run("desk:1", "unplug", &runner).is_err());
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
            "Workspace group: position 1 from the left",
            "Workspace group: position 2 from the left",
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

    assert_eq!(
        titles(&desk),
        [
            "Workspace group: laptop panel",
            "Workspace group: position 2 from the left",
        ]
    );
}

#[test]
fn a_position_named_twice_keeps_its_first_line() {
    let items = Desk::parse("0\teDP-1\tpanel\tBOE 0x0BCA\n3\tDP-1\t1\tA\n5\tDP-2\t1\tB\n").items();

    assert_eq!(items.len(), 2);
    assert_eq!(items[1].subtitle.as_deref(), Some("Workspaces 31-40, on A (DP-1)"));
}
