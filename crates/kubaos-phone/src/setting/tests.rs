use super::Setting;

#[test]
fn the_link_is_off_until_the_file_is_there() {
    let config = tempfile::tempdir().expect("a config folder");
    let setting = Setting::new(config.path());

    assert!(!setting.is_on());
    assert_eq!(setting.path(), config.path().join("kubaos/desktop/phone/phone-link"));
}

#[test]
fn turning_it_on_makes_the_folder_and_off_takes_the_file_away() {
    let config = tempfile::tempdir().expect("a config folder");
    let setting = Setting::new(config.path());

    setting.set(true).expect("turn it on");
    assert!(setting.is_on());

    setting.set(false).expect("turn it off");
    setting.set(false).expect("turn it off again");
    assert!(!setting.is_on());
}
