use std::fs;

use super::Metric;
use super::Settings;
use super::SettingsError;
use super::Style;
use super::Thresholds;
use super::set;

#[test]
fn an_empty_file_is_the_defaults() {
    let settings = Settings::parse("").expect("the defaults");

    assert_eq!(settings, Settings::default());
    assert!(settings.pill);
    assert!(settings.alerts);
    assert_eq!(settings.style, Style::Numbers);
    assert_eq!(settings.metrics, [Metric::Cpu, Metric::Ram]);
    assert_eq!(
        settings.thresholds,
        Thresholds {
            cpu_temp: 90.0,
            ram: 90.0,
            disk: 95.0,
            battery_draw: 30.0
        }
    );
}

#[test]
fn every_key_is_read() {
    let settings = Settings::parse(
        "pill = false\nstyle = \"sparklines\"\nmetrics = [\"temp\", \"net\", \"power\"]\nalerts = false\n\
         [thresholds]\ncpu_temp = 85\nram = 80.5\ndisk = 90\nbattery_draw = 25\n",
    )
    .expect("settings");

    assert!(!settings.pill);
    assert_eq!(settings.style, Style::Sparklines);
    assert_eq!(settings.metrics, [Metric::Temp, Metric::Net, Metric::Power]);
    assert!(!settings.alerts);
    assert_eq!(
        settings.thresholds,
        Thresholds {
            cpu_temp: 85.0,
            ram: 80.5,
            disk: 90.0,
            battery_draw: 25.0
        }
    );
}

#[test]
fn an_unknown_metric_or_key_is_refused() {
    assert!(matches!(
        Settings::parse("metrics = [\"fps\"]"),
        Err(SettingsError::Parse(_))
    ));
    assert!(matches!(
        Settings::parse("colour = \"red\""),
        Err(SettingsError::Parse(_))
    ));
}

#[test]
fn the_pill_toggles_and_the_users_comments_stay() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("sysmon.toml");
    fs::write(&path, "# mine\npill = true\n").expect("write the settings");

    set(&path, "pill", "toggle").expect("toggle the pill");

    let text = fs::read_to_string(&path).expect("read the settings");
    assert!(text.starts_with("# mine\n"), "{text}");
    assert!(!Settings::parse(&text).expect("settings").pill);
}

#[test]
fn metrics_are_set_from_words_in_order() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("sysmon.toml");

    set(&path, "metrics", "ram, gpu temp").expect("set the metrics");

    let settings = Settings::load(&path).expect("settings");
    assert_eq!(settings.metrics, [Metric::Ram, Metric::Gpu, Metric::Temp]);
}

#[test]
fn a_bad_value_names_what_it_may_be() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("sysmon.toml");

    let style = set(&path, "style", "pie");
    let metrics = set(&path, "metrics", "cpu fps");
    let threshold = set(&path, "ram", "120");
    let key = set(&path, "colour", "red");

    assert!(matches!(style, Err(SettingsError::Value { key: "style", .. })));
    assert!(matches!(metrics, Err(SettingsError::Value { key: "metrics", .. })));
    assert!(matches!(threshold, Err(SettingsError::Value { key: "ram", .. })));
    assert!(matches!(key, Err(SettingsError::NoSetting(_))));
    assert!(!path.exists());
}

#[test]
fn thresholds_go_in_their_table() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("sysmon.toml");

    set(&path, "cpu-temp", "85").expect("set the CPU threshold");
    set(&path, "battery-draw", "40").expect("set the draw threshold");

    let settings = Settings::load(&path).expect("settings");
    assert_eq!(
        settings.thresholds,
        Thresholds {
            cpu_temp: 85.0,
            battery_draw: 40.0,
            ..Thresholds::default()
        }
    );
}
