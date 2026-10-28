use super::*;

#[test]
fn the_accents_are_on_without_a_file_or_a_line() {
    let dir = tempfile::tempdir().unwrap();

    assert_eq!(
        Settings::load(&dir.path().join("keyboard.toml")).unwrap(),
        Settings::default()
    );
    assert!(Settings::default().mac_accents);
    assert!(Settings::parse("# mac_accents = false\n").unwrap().mac_accents);
}

#[test]
fn the_seed_in_the_user_layer_leaves_the_accents_on() {
    let seed = include_str!("../../../../../user-layer/desktop/keyboard/keyboard.toml");

    assert_eq!(Settings::parse(seed).unwrap(), Settings::default());
}

#[test]
fn the_file_turns_the_accents_off() {
    assert!(!Settings::parse("mac_accents = false\n").unwrap().mac_accents);
}

#[test]
fn an_unknown_key_or_a_wrong_type_is_refused() {
    assert!(matches!(
        Settings::parse("accents = false\n"),
        Err(SettingsError::Parse(_))
    ));
    assert!(matches!(
        Settings::parse("mac_accents = \"no\"\n"),
        Err(SettingsError::Parse(_))
    ));
}

#[test]
fn turning_the_accents_off_and_on_keeps_the_comments() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keyboard.toml");
    fs::write(&path, "# Mine.\n# mac_accents = false\n").unwrap();

    set_mac_accents(&path, MacAccents::Off).unwrap();
    let off = fs::read_to_string(&path).unwrap();
    set_mac_accents(&path, MacAccents::On).unwrap();
    let on = fs::read_to_string(&path).unwrap();

    assert_eq!(off, "# Mine.\n# mac_accents = false\nmac_accents = false\n");
    assert_eq!(on, "# Mine.\n# mac_accents = false\nmac_accents = true\n");
    assert!(Settings::load(&path).unwrap().mac_accents);
}

#[test]
fn turning_the_accents_off_creates_the_file_and_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keyboard").join("keyboard.toml");

    set_mac_accents(&path, MacAccents::Off).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "mac_accents = false\n");
}

#[test]
fn the_file_lives_in_the_user_layer() {
    assert_eq!(
        path(Path::new("/home/kuba/.config")),
        Path::new("/home/kuba/.config/kubaos/desktop/keyboard/keyboard.toml")
    );
}

#[test]
fn the_setting_prints_as_on_or_off() {
    let off = Settings {
        mac_accents: false,
        ..Settings::default()
    };

    assert_eq!(MacAccents::from(Settings::default()).to_string(), "on");
    assert_eq!(MacAccents::from(off).to_string(), "off");
}

#[test]
fn the_layouts_are_unset_until_the_file_lists_them() {
    assert!(Settings::default().layouts.is_empty());

    let settings = Settings::parse("layouts = [\"us\", \"it(intl)\"]\n").unwrap();

    assert_eq!(settings.layouts, ["us", "it(intl)"]);
}

#[test]
fn writing_the_layouts_keeps_the_comments_and_the_accents() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keyboard.toml");
    fs::write(&path, "# Mine.\nmac_accents = false\n").unwrap();

    set_layouts(&path, &[Layout::parse("us"), Layout::parse("it(intl)")]).unwrap();
    set_layouts(&path, &[Layout::parse("de")]).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# Mine.\nmac_accents = false\nlayouts = [\"de\"]\n"
    );
    let settings = Settings::load(&path).unwrap();
    assert!(!settings.mac_accents);
    assert_eq!(settings.layouts, ["de"]);
}

#[test]
fn writing_the_layouts_creates_the_file_and_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keyboard").join("keyboard.toml");

    set_layouts(&path, &[Layout::parse("us"), Layout::parse("it")]).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "layouts = [\"us\", \"it\"]\n");
}
