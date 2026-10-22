use super::*;
use crate::fixture;

#[test]
fn every_cursor_of_the_config_loads_sorted_by_name() {
    let dir = fixture::bibata();

    let shapes = load(dir.path()).unwrap();

    let names: Vec<&str> = shapes.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["hand2", "left_ptr", "wait"]);
}

#[test]
fn a_cursor_keeps_its_hotspot_and_aliases() {
    let dir = fixture::bibata();

    let shapes = load(dir.path()).unwrap();

    let arrow = &shapes[1];
    assert_eq!(arrow.hotspot, (55, 17));
    assert_eq!(arrow.aliases, ["arrow", "default"]);
    assert_eq!(arrow.frames, [fixture::ARROW]);
}

#[test]
fn an_animated_cursor_takes_its_frames_in_order_and_the_fallback_hotspot_and_delay() {
    let dir = fixture::bibata();
    fs::write(dir.path().join("svg/modern/wait/wait-02.svg"), "second").unwrap();

    let shapes = load(dir.path()).unwrap();

    let wait = &shapes[2];
    assert_eq!(wait.frames, [fixture::WATCH, "second"]);
    assert_eq!(wait.hotspot, (128, 128));
    assert_eq!(wait.delay_ms, 40);
}

#[test]
fn a_missing_svg_names_its_path() {
    let dir = fixture::bibata();
    fs::remove_file(dir.path().join("svg/modern/hand2.svg")).unwrap();

    let err = load(dir.path()).unwrap_err();

    assert!(matches!(err, SourceError::ReadSvg(path, _) if path.ends_with("svg/modern/hand2.svg")));
}
