use super::Settings;

#[test]
fn automount_is_on_until_turned_off_and_back_on() {
    let dir = tempfile::tempdir().unwrap();
    let settings = Settings::new(dir.path());

    assert!(settings.automount());
    settings.set_automount(false).unwrap();
    assert!(!settings.automount());
    assert!(dir.path().join("kubaos/files/no-automount").exists());
    settings.set_automount(true).unwrap();
    assert!(settings.automount());
}

#[test]
fn turning_it_on_twice_is_fine() {
    let dir = tempfile::tempdir().unwrap();
    let settings = Settings::new(dir.path());

    settings.set_automount(true).unwrap();
    settings.set_automount(true).unwrap();

    assert!(settings.automount());
}
