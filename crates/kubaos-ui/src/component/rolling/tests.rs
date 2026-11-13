use super::Slot;
use super::changed;
use super::figures;

fn slots(text: &str) -> Vec<Option<char>> {
    text.chars().map(|c| (c != ' ').then_some(c)).collect()
}

#[test]
fn a_count_is_its_plain_figures() {
    assert_eq!(figures(0), vec!['0']);
    assert_eq!(figures(1000), vec!['1', '0', '0', '0']);
}

#[test]
fn only_the_figures_that_change_roll() {
    let rolled = changed(&slots("109"), &slots("110"), false);

    let rolls: Vec<bool> = rolled.iter().map(|slot| slot.rolls).collect();
    assert_eq!(rolls, [false, true, true]);
    assert_eq!(rolled[1].leaving, Some('0'));
    assert_eq!(rolled[2].figure, Some('0'));
}

#[test]
fn a_figure_the_count_grows_rolls_in_from_nothing() {
    let rolled = changed(&slots(" 99"), &slots("100"), false);

    assert_eq!(
        rolled[0],
        Slot {
            figure: Some('1'),
            leaving: None,
            rolls: true
        }
    );
}

#[test]
fn reduced_motion_changes_figures_in_place() {
    let rolled = changed(&slots("9"), &slots("8"), true);

    assert_eq!(
        rolled[0],
        Slot {
            figure: Some('8'),
            leaving: None,
            rolls: false
        }
    );
}
