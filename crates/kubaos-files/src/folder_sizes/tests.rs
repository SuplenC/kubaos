use super::Setting;

#[test]
fn folder_sizes_are_on_until_turned_off() {
    let config = tempfile::tempdir().unwrap();
    let setting = Setting::new(config.path());

    assert!(setting.is_on());
    setting.set(false).unwrap();
    assert!(!setting.is_on());
    assert!(config.path().join("kubaos/desktop/files/no-folder-sizes").exists());
    setting.set(true).unwrap();
    setting.set(true).unwrap();
    assert!(setting.is_on());
}
