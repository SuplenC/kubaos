use std::path::Path;

use super::*;
use crate::settings::LidAction;

/// The listeners alone, without the comments at the top.
fn listeners(settings: &Settings) -> String {
    config(settings)
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_defaults_are_the_three_listeners_kubaos_always_had() {
    let expected = r#"
listener {
    timeout = 300
    on-timeout = loginctl lock-session
}

listener {
    timeout = 600
    on-timeout = hyprctl dispatch 'hl.dsp.dpms({ action = "disable" })'
    on-resume = hyprctl dispatch 'hl.dsp.dpms({ action = "enable" })'
}

listener {
    timeout = 1800
    condition_cmd = ! dot-hypr-on-ac
    condition_retry = 60
    on-timeout = systemctl suspend
}"#;

    assert_eq!(listeners(&Settings::default()), expected);
}

#[test]
fn free_form_waits_become_seconds() {
    let mut settings = Settings::default();
    settings.battery.lock = Wait::parse("90s").expect("90s");
    settings.ac.lock = Wait::parse("90s").expect("90s");
    settings.battery.screens_off = Wait::parse("1h30m").expect("1h30m");
    settings.ac.screens_off = Wait::parse("1h30m").expect("1h30m");

    let text = listeners(&settings);

    assert!(
        text.contains("timeout = 90\n    on-timeout = loginctl lock-session"),
        "{text}"
    );
    assert!(
        text.contains("timeout = 5400\n    on-timeout = hyprctl dispatch"),
        "{text}"
    );
}

#[test]
fn two_waits_are_two_listeners_each_asking_the_power_source() {
    let mut settings = Settings::default();
    settings.battery.lock = Wait::from_secs(15 * 60);
    settings.ac.lock = Wait::from_secs(30 * 60);

    let text = listeners(&settings);

    assert!(
        text.contains(
            "timeout = 900\n    condition_cmd = ! dot-hypr-on-ac\n    condition_retry = 60\n    \
             on-timeout = loginctl lock-session"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "timeout = 1800\n    condition_cmd = dot-hypr-on-ac\n    condition_retry = 60\n    \
             on-timeout = loginctl lock-session"
        ),
        "{text}"
    );
    assert_eq!(text.matches("listener {").count(), 4);
}

#[test]
fn never_leaves_the_listener_out() {
    let mut settings = Settings::default();
    for on in [&mut settings.battery, &mut settings.ac] {
        on.lock = Wait::NEVER;
        on.screens_off = Wait::NEVER;
    }
    settings.battery.suspend = Wait::NEVER;

    assert_eq!(listeners(&settings), "");
}

#[test]
fn plugged_in_alone_asks_for_ac() {
    let mut settings = Settings::default();
    settings.battery.suspend = Wait::NEVER;
    settings.ac.suspend = Wait::from_secs(3600);
    settings.battery.lid = LidAction::Lock;

    let text = listeners(&settings);

    assert!(
        text.contains("timeout = 3600\n    condition_cmd = dot-hypr-on-ac\n"),
        "{text}"
    );
    assert!(!text.contains("! dot-hypr-on-ac"), "{text}");
}

#[test]
fn the_header_says_where_the_settings_are_and_what_they_come_to() {
    let text = config(&Settings::default());

    assert!(text.starts_with("# Written by kubaos-idle from ~/.config/kubaos/desktop/idle/idle.toml"));
    assert!(
        text.contains("# Lock after 5 min idle, on battery and plugged in."),
        "{text}"
    );
}

#[test]
fn the_file_is_where_the_shipped_config_sources_it() {
    let shipped = include_str!("../../../../config/hypr/hypridle.conf");

    assert!(shipped.contains(&format!("source = ~/{FILE}\n")), "{shipped}");
    assert_eq!(
        path(Path::new("/home/kuba")),
        Path::new("/home/kuba/.local/state/kubaos/idle/hypridle.conf")
    );
}

#[test]
fn writing_makes_the_folder_and_replaces_the_file() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let file = path(dir.path());
    let mut settings = Settings::default();

    write(&file, &settings).expect("first write");
    settings.ac.lock = Wait::from_secs(60);
    write(&file, &settings).expect("second write");

    let text = std::fs::read_to_string(&file).expect("read back");
    assert!(text.contains("timeout = 60\n"), "{text}");
}
