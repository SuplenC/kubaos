use super::REMEMBERED;
use super::Seen;

#[test]
fn a_unit_loaded_again_is_told_once() {
    let mut seen = Seen::default();

    assert!(seen.first_time("app-kotodama-foot@1.service"));
    assert!(!seen.first_time("app-kotodama-foot@1.service"));
    assert!(seen.first_time("app-kotodama-foot@2.service"));
}

#[test]
fn only_the_last_units_are_remembered() {
    let mut seen = Seen::default();

    for n in 0..=REMEMBERED {
        seen.first_time(&format!("app-kotodama-foot@{n}.service"));
    }

    assert!(seen.first_time("app-kotodama-foot@0.service"));
    assert!(!seen.first_time(&format!("app-kotodama-foot@{REMEMBERED}.service")));
}
