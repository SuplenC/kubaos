use std::path::Path;

use super::Settings;
use super::is_ini_name;
use super::is_ini_value;
use super::written;

const ZOOM: &str = "/home/u/.config/zoomus.conf";

#[test]
fn an_ini_setting_round_trips_and_its_undo_takes_the_section_it_made() {
    let path = Path::new(ZOOM);
    let mut settings = Settings::parse(path, Some("General"), "[AS]\nshowframewindow=true\n").unwrap();
    let value = toml::Value::Boolean(false);
    let tables = settings.tables(path, "xwayland").unwrap();

    let text = written(&settings, &value);
    settings.set(path, "xwayland", Some(&text), usize::MAX).unwrap();
    assert!(settings.has(path, "xwayland", &value).unwrap());
    settings.set(path, "xwayland", None, tables).unwrap();

    assert_eq!(tables, 0);
    assert_eq!(settings.render(), "[AS]\nshowframewindow=true\n");
}

#[test]
fn an_ini_undo_writes_the_old_value_back_as_the_file_had_it() {
    let path = Path::new(ZOOM);
    let mine = "[General]\nxwayland=\"1860,990\" ;mine\n";
    let mut settings = Settings::parse(path, Some("General"), mine).unwrap();
    let old = settings.get(path, "xwayland").unwrap();

    settings.set(path, "xwayland", Some("false"), usize::MAX).unwrap();
    settings.set(path, "xwayland", old.as_deref(), 1).unwrap();

    assert_eq!(old.as_deref(), Some("\"1860,990\""));
    assert_eq!(settings.render(), mine);
}

#[test]
fn an_ini_value_or_name_qt_would_write_otherwise_is_refused() {
    for value in [
        "\"quoted\"",
        "a,b",
        "a;b",
        "a=b",
        "a\\b",
        "@ByteArray(x)",
        " padded",
        "two\nlines",
        "tab\there",
        "del\u{7f}",
    ] {
        assert!(!is_ini_value(&toml::Value::String(value.to_owned())), "{value:?}");
    }
    assert!(is_ini_value(&toml::Value::Boolean(false)));
    assert!(is_ini_value(&toml::Value::String("https://zoom.us/#x".to_owned())));
    for name in ["", "a b", "a/b", "a%2B", "é", "[x"] {
        assert!(!is_ini_name(name), "{name:?}");
    }
    assert!(is_ini_name("conf.webserver-2_x"));
}

#[test]
fn a_conf_without_a_section_stays_flat_and_refuses_a_header() {
    let path = Path::new("/home/u/.config/sunshine/sunshine.conf");

    assert!(Settings::parse(path, None, "[General]\nkey = pc\n").is_err());
    assert!(Settings::parse(path, Some("General"), "[General]\nkey = pc\n").is_ok());
}
