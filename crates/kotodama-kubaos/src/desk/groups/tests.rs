use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::*;

const BENQ: &str = "BNQ ZOWIE XL LCD G4L01663SL0";
const LG: &str = "LG Electronics LG ULTRAGEAR+ 510NTCZN7787";
const DELL: &str = "Dell Inc. DELL U2724D 5D6E7F8";

/// The seed `dot-config apply` writes, which is comments only.
fn seed() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../user-layer/desktop/workspaces/groups.toml"))
        .expect("read the seed")
}

fn file_with(text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(FILE);
    fs::write(&path, text).unwrap();
    (dir, path)
}

fn text(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn the_seed_pins_nothing() {
    assert_eq!(parse(&seed()).unwrap(), []);
}

#[test]
fn groups_read_in_number_order_with_their_names_and_screens() {
    let groups = parse(&format!(
        "[group.2]\nscreens = [\"{LG}\"]\n\n[group.1]\nname = \" Left \"\nscreens = [\"{BENQ}\", \"{DELL}\"]\n"
    ))
    .unwrap();

    assert_eq!(
        groups,
        [
            Group {
                number: 1,
                name: Some("Left".to_owned()),
                screens: vec![BENQ.to_owned(), DELL.to_owned()],
            },
            Group {
                number: 2,
                name: None,
                screens: vec![LG.to_owned()],
            },
        ]
    );
}

#[test]
fn a_file_hyprland_refuses_is_refused() {
    for (text, problem) in [
        ("screens = [\"A\"]\n", "`screens` isn't a group"),
        ("[groups.1]\n", "`groups` isn't a group"),
        ("[group]\n[group.1]\n", "`group` isn't a group"),
        ("[group.0]\n", "[group.0] isn't a group"),
        ("[group.1]\nscreen = [\"A\"]\n", "`screen` isn't a setting of a group"),
        ("[group.1]\nname = 3\n", "group 1's name isn't text"),
        ("[group.1]\nscreens = \"A\"\n", "group 1's screens aren't a list"),
        ("group = { 1 = { screens = [\"A\"] } }\n", "`group` isn't a group"),
        (
            "[group.1]\nscreens = [\"A\"]\n[group.2]\nscreens = [\"A\"]\n",
            "A is in group 1 and group 2",
        ),
        ("[group.1]\nscreens = [\"A\", \"A\"]\n", "group 1 lists A twice"),
    ] {
        let err = parse(text).expect_err(text);
        assert!(
            matches!(&err, GroupsError::Invalid(message) if message.contains(problem)),
            "{text:?}: {err}"
        );
    }
    assert!(matches!(parse("[group.1"), Err(GroupsError::Toml(_))));
}

#[test]
fn no_file_is_no_groups() {
    assert_eq!(read(Path::new("/nonexistent/groups.toml")).unwrap(), []);
}

#[test]
fn pinning_into_the_seed_adds_the_group_below_its_comments() {
    let (_dir, path) = file_with(&seed());

    pin(&path, BENQ, 1).unwrap();

    assert_eq!(text(&path), format!("{}\n[group.1]\nscreens = [\"{BENQ}\"]\n", seed()));
    assert_eq!(read(&path).unwrap()[0].screens, [BENQ]);
}

#[test]
fn pinning_with_no_file_makes_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspaces").join(FILE);

    pin(&path, LG, 2).unwrap();

    assert_eq!(text(&path), format!("[group.2]\nscreens = [\"{LG}\"]\n"));
}

#[test]
fn a_screen_pinned_to_a_group_goes_last_so_the_ones_listed_keep_winning() {
    let (_dir, path) = file_with(&format!("[group.1]\nscreens = [\"{BENQ}\"]\n"));

    pin(&path, DELL, 1).unwrap();

    assert_eq!(text(&path), format!("[group.1]\nscreens = [\"{BENQ}\", \"{DELL}\"]\n"));
}

#[test]
fn a_list_written_one_a_line_stays_one_a_line() {
    let (_dir, path) = file_with(&format!("[group.1]\nscreens = [\n  \"{BENQ}\", # home\n]\n"));

    pin(&path, DELL, 1).unwrap();

    assert_eq!(
        text(&path),
        format!("[group.1]\nscreens = [\n  \"{BENQ}\", # home\n  \"{DELL}\",\n]\n")
    );
}

#[test]
fn pinning_a_screen_to_another_group_moves_it_and_keeps_every_comment() {
    let (_dir, path) = file_with(&format!(
        "# My desks.\n[group.1]\n# Left.\nname = \"Left\" # the BenQ side\nscreens = [\"{BENQ}\", \"{LG}\"]\n\n# The end.\n"
    ));

    pin(&path, LG, 2).unwrap();

    assert_eq!(
        text(&path),
        format!(
            "# My desks.\n[group.1]\n# Left.\nname = \"Left\" # the BenQ side\nscreens = [\"{BENQ}\"]\n\n# The end.\n\n[group.2]\nscreens = [\"{LG}\"]\n"
        )
    );
}

#[test]
fn a_new_group_after_others_is_set_apart_by_a_blank_line() {
    let (_dir, path) = file_with(&format!("[group.1]\nscreens = [\"{BENQ}\"]\n"));

    pin(&path, LG, 2).unwrap();

    assert_eq!(
        text(&path),
        format!("[group.1]\nscreens = [\"{BENQ}\"]\n\n[group.2]\nscreens = [\"{LG}\"]\n")
    );
}

#[test]
fn unpinning_takes_the_screen_out_and_leaves_its_group() {
    let (_dir, path) = file_with(&format!(
        "[group.1]\nname = \"Left\"\nscreens = [\"{BENQ}\", \"{DELL}\"]\n"
    ));

    unpin(&path, BENQ).unwrap();
    unpin(&path, DELL).unwrap();

    assert_eq!(text(&path), "[group.1]\nname = \"Left\"\nscreens = []\n");
}

#[test]
fn naming_a_group_puts_the_name_above_its_screens() {
    let (_dir, path) = file_with(&format!("[group.1]\nscreens = [\"{BENQ}\"]\n"));

    name(&path, 1, "  Left   side ").unwrap();

    assert_eq!(
        text(&path),
        format!("[group.1]\nname = \"Left side\"\nscreens = [\"{BENQ}\"]\n")
    );
}

#[test]
fn renaming_keeps_the_comment_on_the_name_line() {
    let (_dir, path) = file_with("[group.1]\nname = \"Left\" # by the window\n");

    name(&path, 1, "Window").unwrap();

    assert_eq!(text(&path), "[group.1]\nname = \"Window\" # by the window\n");
}

#[test]
fn a_blank_name_takes_the_name_away() {
    let (_dir, path) = file_with(&format!("[group.1]\nname = \"Left\"\nscreens = [\"{BENQ}\"]\n"));

    name(&path, 1, " ").unwrap();

    assert_eq!(text(&path), format!("[group.1]\nscreens = [\"{BENQ}\"]\n"));
}

#[test]
fn naming_a_group_with_no_table_makes_it() {
    let (_dir, path) = file_with("");

    name(&path, 3, "Right").unwrap();

    assert_eq!(text(&path), "[group.3]\nname = \"Right\"\n");
}

#[test]
fn a_file_that_doesnt_read_is_left_alone() {
    let broken = "[group.1]\nscreen = [\"A\"]\n";
    let (_dir, path) = file_with(broken);

    assert!(matches!(pin(&path, BENQ, 1), Err(GroupsError::Invalid(_))));
    assert!(matches!(name(&path, 1, "Left"), Err(GroupsError::Invalid(_))));
    assert_eq!(text(&path), broken);
}
