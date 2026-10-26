use std::fs;

use super::*;

const SEED: &str = include_str!("../../../../user-layer/desktop/idle/idle.toml");

fn file(text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("idle").join(FILE);
    if !text.is_empty() {
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(&path, text).expect("write the settings");
    }
    (dir, path)
}

fn mins(minutes: u32) -> Wait {
    Wait::from_secs(minutes * 60)
}

#[test]
fn the_defaults_are_lock_5_screens_off_10_and_suspend_30_on_battery_only() {
    let settings = Settings::default();

    assert_eq!(settings.waits(Timer::Lock), (mins(5), mins(5)));
    assert_eq!(settings.waits(Timer::ScreensOff), (mins(10), mins(10)));
    assert_eq!(settings.waits(Timer::Suspend), (mins(30), Wait::NEVER));
    assert_eq!(
        (settings.battery.lid, settings.ac.lid),
        (LidAction::Suspend, LidAction::Nothing)
    );
}

#[test]
fn a_missing_file_and_the_seed_are_the_defaults() {
    let (_dir, path) = file("");

    assert_eq!(Settings::load(&path).expect("loads"), Settings::default());
    assert_eq!(Settings::parse(SEED).expect("the seed parses"), Settings::default());
}

#[test]
fn the_seeds_commented_values_are_the_defaults() {
    let uncommented: String = SEED
        .lines()
        .filter_map(|line| line.strip_prefix("# "))
        .filter(|line| line.starts_with('[') || line.contains(" = \""))
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(Settings::parse(&uncommented).expect("parses"), Settings::default());
}

#[test]
fn waits_read_as_the_clock_reads_a_timer() {
    assert_eq!(Wait::parse("2m").expect("2m"), mins(2));
    assert_eq!(Wait::parse("90s").expect("90s"), Wait::from_secs(90));
    assert_eq!(Wait::parse("1h30m").expect("1h30m"), mins(90));
    assert_eq!(Wait::parse("5h").expect("5h"), mins(300));
    assert_eq!(Wait::parse("25").expect("25"), mins(25));
    assert_eq!(Wait::parse(" 2 hours 15 min ").expect("words"), mins(135));
    assert_eq!(Wait::parse("never").expect("never"), Wait::NEVER);
    assert_eq!(Wait::parse("0").expect("0"), Wait::NEVER);
    assert_eq!(Wait::parse("Off").expect("off"), Wait::NEVER);
}

#[test]
fn a_wait_that_isnt_one_is_refused_with_examples() {
    for typed in ["", "soon", "1 month", "friday", "-5m"] {
        let err = Wait::parse(typed).expect_err(typed);
        assert!(
            err.to_string().contains("try 90s, 2m, 1h30m or never"),
            "{typed}: {err}"
        );
    }
}

#[test]
fn waits_are_written_compactly() {
    assert_eq!(mins(5).canonical(), "5m");
    assert_eq!(Wait::from_secs(90).canonical(), "1m30s");
    assert_eq!(mins(90).canonical(), "1h30m");
    assert_eq!(mins(300).canonical(), "5h");
    assert_eq!(Wait::NEVER.canonical(), "never");
    for wait in [mins(5), Wait::from_secs(90), mins(90), Wait::from_secs(3601)] {
        assert_eq!(Wait::parse(&wait.canonical()).expect("reads back"), wait);
    }
}

#[test]
fn a_lock_under_30_seconds_or_over_a_day_is_refused_saying_why() {
    let soon = Timer::Lock.check(Wait::from_secs(10)).expect_err("too soon");
    let long = Timer::Lock.check(mins(25 * 60)).expect_err("too long");

    assert_eq!(
        soon.to_string(),
        "The lock can't come sooner than 30 s idle, not after 10 s"
    );
    assert_eq!(
        long.to_string(),
        "The lock can wait a day at most, not 25 h; for longer, say never"
    );
    assert!(Timer::Lock.check(Wait::from_secs(30)).is_ok());
    assert!(Timer::Lock.check(Wait::NEVER).is_ok());
    assert!(Timer::Suspend.check(Wait::from_secs(30)).is_err());
    assert!(Timer::Suspend.check(mins(1)).is_ok());
}

#[test]
fn every_key_is_read_strings_and_minutes() {
    let text = "[battery]\nlock = \"2m\"\nscreens_off = 3\nsuspend = \"never\"\nlid = \"lock\"\n\
                [ac]\nlock = \"1h30m\"\nlid = \"ignore\"\n";

    let settings = Settings::parse(text).expect("parses");

    assert_eq!(settings.waits(Timer::Lock), (mins(2), mins(90)));
    assert_eq!(settings.waits(Timer::ScreensOff), (mins(3), mins(10)));
    assert_eq!(settings.waits(Timer::Suspend), (Wait::NEVER, Wait::NEVER));
    assert_eq!(
        (settings.battery.lid, settings.ac.lid),
        (LidAction::Lock, LidAction::Nothing)
    );
}

#[test]
fn a_bad_value_names_its_table_and_key() {
    let err = Settings::parse("[ac]\nlock = \"5s\"\n").expect_err("too soon");
    assert_eq!(
        chain(&err),
        "`lock` in [ac]: The lock can't come sooner than 30 s idle, not after 5 s"
    );

    assert!(Settings::parse("[battery]\nlid = \"explode\"\n").is_err());
    assert!(Settings::parse("[battery]\ndim = \"1m\"\n").is_err());
    assert!(Settings::parse("lock = \"5m\"\n").is_err());
}

fn chain(err: &SettingsError) -> String {
    match err {
        SettingsError::Value { power, key, problem } => format!("`{key}` in [{power}]: {problem}"),
        other => other.to_string(),
    }
}

#[test]
fn setting_both_writes_both_tables_and_keeps_the_comments() {
    let (_dir, path) = file(SEED);

    let settings = set_wait(&path, Timer::Lock, mins(15), Sources::Both).expect("sets");

    assert_eq!(settings.waits(Timer::Lock), (mins(15), mins(15)));
    let text = fs::read_to_string(&path).expect("read back");
    assert!(text.starts_with("# Your idle settings"), "{text}");
    assert!(text.contains("[battery]\nlock = \"15m\"\n"), "{text}");
    assert!(text.contains("[ac]\nlock = \"15m\"\n"), "{text}");
    assert_eq!(Settings::load(&path).expect("loads"), settings);
}

#[test]
fn setting_one_source_leaves_the_other() {
    let (_dir, path) = file("");

    set_wait(&path, Timer::Lock, mins(15), Sources::Only(Power::Battery)).expect("battery");
    let settings = set_wait(&path, Timer::Lock, mins(30), Sources::Only(Power::Ac)).expect("ac");

    assert_eq!(settings.waits(Timer::Lock), (mins(15), mins(30)));
    assert_eq!(settings.waits(Timer::Suspend), (mins(30), Wait::NEVER));
}

#[test]
fn setting_a_value_again_replaces_its_line() {
    let (_dir, path) = file("[battery] # mine\nlock = \"2m\" # short\nsuspend = \"1h\"\n");

    set_wait(&path, Timer::Lock, mins(5), Sources::Only(Power::Battery)).expect("sets");

    let text = fs::read_to_string(&path).expect("read back");
    assert_eq!(text.matches("lock =").count(), 1, "{text}");
    assert!(text.contains("[battery] # mine"), "{text}");
    assert!(text.contains("suspend = \"1h\""), "{text}");
}

#[test]
fn a_refused_change_leaves_the_file_alone() {
    let (_dir, path) = file("[battery]\nlock = \"2m\"\n");

    let err = set_wait(&path, Timer::ScreensOff, Wait::from_secs(5), Sources::Both).expect_err("too soon");

    assert!(matches!(err, SettingsError::TooSoon { .. }));
    assert_eq!(fs::read_to_string(&path).expect("read"), "[battery]\nlock = \"2m\"\n");
}

#[test]
fn the_lid_is_set_by_name() {
    let (_dir, path) = file("");

    let settings = set_lid(&path, LidAction::Lock, Sources::Only(Power::Ac)).expect("sets");

    assert_eq!(
        (settings.battery.lid, settings.ac.lid),
        (LidAction::Suspend, LidAction::Lock)
    );
    assert_eq!(LidAction::from_name("Do-Nothing"), Some(LidAction::Nothing));
    assert_eq!(LidAction::from_name("hibernate"), None);
}

#[test]
fn reset_takes_the_tables_out_and_keeps_the_comments() {
    let (_dir, path) = file("# mine\n[battery]\nlock = \"2m\"\n[ac]\nlid = \"lock\"\n");

    let settings = reset(&path).expect("resets");

    assert_eq!(settings, Settings::default());
    assert!(fs::read_to_string(&path).expect("read").starts_with("# mine"));
}

#[test]
fn the_sources_word_reads_battery_ac_and_plugged_in() {
    assert_eq!(Sources::from_word("battery"), Some(Sources::Only(Power::Battery)));
    assert_eq!(Sources::from_word("AC"), Some(Sources::Only(Power::Ac)));
    assert_eq!(Sources::from_word("plugged-in"), Some(Sources::Only(Power::Ac)));
    assert_eq!(Sources::from_word("both"), Some(Sources::Both));
    assert_eq!(Sources::from_word("5m"), None);
}

#[test]
fn every_choice_the_menu_offers_is_within_the_limits() {
    for timer in Timer::ALL {
        for &wait in timer.choices() {
            assert!(timer.check(wait).is_ok(), "{} {}", timer.name(), wait.canonical());
        }
        assert_eq!(timer.choices().last(), Some(&Wait::NEVER));
        assert_eq!(Timer::from_name(timer.name()), Some(timer));
        assert_eq!(Timer::from_name(timer.key()), Some(timer));
    }
}
