use super::Settings;

#[test]
fn it_is_on_until_turned_off_and_back_on() {
    let dir = tempfile::tempdir().unwrap();
    let settings = Settings::new(dir.path());

    assert!(settings.open_where_launched());
    settings.set_open_where_launched(false).unwrap();
    assert!(!settings.open_where_launched());
    assert!(
        dir.path()
            .join("kubaos/desktop/workspaces/no-open-where-launched")
            .exists()
    );
    settings.set_open_where_launched(true).unwrap();
    assert!(settings.open_where_launched());
}

#[test]
fn turning_it_on_twice_is_fine() {
    let dir = tempfile::tempdir().unwrap();
    let settings = Settings::new(dir.path());

    settings.set_open_where_launched(true).unwrap();
    settings.set_open_where_launched(true).unwrap();

    assert!(settings.open_where_launched());
}
