use super::*;

#[test]
fn a_missing_or_broken_file_is_power_saver_on_battery_and_balanced_plugged_in() {
    let dir = tempfile::tempdir().expect("a temp dir");

    assert_eq!(Memory::load(&dir.path().join(FILE)), Memory::default());
    assert_eq!(Memory::parse("{not json"), Memory::default());
    assert_eq!(Memory::default().battery, Stop::PowerSaver);
    assert_eq!(Memory::default().ac, Stop::Balanced);
}

#[test]
fn the_memory_survives_a_write_and_a_read() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("kubaos/power").join(FILE);
    let memory = Memory {
        battery: Stop::SuperSaver,
        ac: Stop::MaxPerformance,
        last: Some((Source::Battery, Stop::PowerSaver)),
        dropped_from: Some(Stop::SuperSaver),
        undone: true,
    };

    memory.save(&path).expect("saves");

    assert_eq!(Memory::load(&path), memory);
}

#[test]
fn an_unknown_stop_takes_the_sources_default() {
    let memory = Memory::parse(r#"{"battery":"turbo","ac":"performance","last":{"source":"moon","stop":"balanced"}}"#);

    assert_eq!(memory.battery, Stop::PowerSaver);
    assert_eq!(memory.ac, Stop::Performance);
    assert_eq!(memory.last, None);
}
