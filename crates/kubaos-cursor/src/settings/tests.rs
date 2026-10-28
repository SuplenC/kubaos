use super::*;

#[test]
fn an_empty_file_is_the_dark_style_at_24() {
    assert_eq!(
        Settings::parse("").unwrap(),
        Settings {
            style: Style::Dark,
            size: 24
        }
    );
}

#[test]
fn a_missing_file_is_the_defaults() {
    let dir = tempfile::tempdir().unwrap();

    assert_eq!(
        Settings::load(&dir.path().join("cursor.toml")).unwrap(),
        Settings::default()
    );
}

#[test]
fn the_file_sets_the_style_and_the_size() {
    let settings = Settings::parse("style = \"accent\"\nsize = 32\n").unwrap();

    assert_eq!(
        settings,
        Settings {
            style: Style::Accent,
            size: 32
        }
    );
}

#[test]
fn an_unknown_style_or_a_size_out_of_range_is_an_error() {
    assert!(matches!(
        Settings::parse("style = \"neon\""),
        Err(SettingsError::Parse(_))
    ));
    assert!(matches!(Settings::parse("size = 8"), Err(SettingsError::Size(8))));
    assert!(matches!(Settings::parse("size = 200"), Err(SettingsError::Size(200))));
}

#[test]
fn a_style_parses_from_its_name() {
    assert_eq!("classic".parse::<Style>().unwrap(), Style::Classic);
    assert!("Dark".parse::<Style>().is_err());
}

#[test]
fn setting_a_value_replaces_its_line_and_keeps_the_comments() {
    let text = "# The pointer.\n# style = \"accent\"\nstyle = \"dark\"\nsize = 24\n";

    let new = with_value(text, "style", "\"classic\"");

    assert_eq!(
        new,
        "# The pointer.\n# style = \"accent\"\nstyle = \"classic\"\nsize = 24\n"
    );
}

#[test]
fn setting_a_value_the_file_lacks_appends_it() {
    assert_eq!(
        with_value("# Only comments.\n", "size", "48"),
        "# Only comments.\nsize = 48\n"
    );
}

#[test]
fn set_size_writes_a_new_file_and_refuses_a_size_out_of_range() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cursor/cursor.toml");

    set_size(&path, 48).unwrap();
    set_style(&path, Style::Accent).unwrap();

    assert_eq!(
        Settings::load(&path).unwrap(),
        Settings {
            style: Style::Accent,
            size: 48
        }
    );
    assert!(matches!(set_size(&path, 4), Err(SettingsError::Size(4))));
    assert_eq!(Settings::load(&path).unwrap().size, 48);
}
