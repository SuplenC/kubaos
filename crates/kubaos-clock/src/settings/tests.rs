use std::fs;

use jiff::civil::Weekday;

use super::*;

const SEED: &str = "\
# Extra clocks.
zones = [\"Asia/Tokyo\"]
# Holidays.
holidays = \"pl\"
";

fn file(text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("clock").join(FILE);
    if !text.is_empty() {
        fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        fs::write(&path, text).expect("write the settings");
    }
    (dir, path)
}

fn load(path: &Path) -> Settings {
    Settings::load(path).expect("the settings load")
}

#[test]
fn a_missing_file_is_the_defaults_with_monday_first() {
    let (_dir, path) = file("");

    let settings = load(&path);

    assert_eq!(settings, Settings::default());
    assert_eq!(settings.first_weekday, Weekday::Monday);
}

#[test]
fn every_key_is_read() {
    let text = "zones = [\"Asia/Tokyo\"]\nholidays = \"\"\nfirst_weekday = \"Sunday\"\nanalog = true\n\
                bar_countdown = \"Trip\"\n[[countdowns]]\nname = \"Trip\"\nat = \"2026-12-19\"\n";

    let settings = Settings::parse(text).expect("parses");

    assert_eq!(settings.zones, ["Asia/Tokyo"]);
    assert_eq!(settings.holidays.as_deref(), Some(""));
    assert_eq!(settings.first_weekday, Weekday::Sunday);
    assert!(settings.analog);
    assert_eq!(settings.bar_countdown, "Trip");
    assert_eq!(
        settings.countdowns,
        [Countdown {
            name: "Trip".into(),
            at: "2026-12-19".into()
        }]
    );
}

#[test]
fn an_unknown_key_or_weekday_is_an_error() {
    assert!(matches!(Settings::parse("colour = 1"), Err(SettingsError::Parse(_))));
    assert!(matches!(
        Settings::parse("first_weekday = \"someday\""),
        Err(SettingsError::Weekday(_))
    ));
}

#[test]
fn an_empty_holiday_setting_means_none_and_a_code_is_kept() {
    let countries = Countries::bundled();
    let none = Settings {
        holidays: Some(String::new()),
        ..Settings::default()
    };
    let us = Settings {
        holidays: Some("US".into()),
        ..Settings::default()
    };

    assert_eq!(none.holiday_country(&countries), None);
    assert_eq!(us.holiday_country(&countries).as_deref(), Some("us"));
}

#[test]
fn adding_a_zone_keeps_the_comments_and_skips_one_already_there() {
    let (_dir, path) = file(SEED);

    add_zone(&path, "America/New_York").expect("added");
    add_zone(&path, "Asia/Tokyo").expect("already there");

    let text = fs::read_to_string(&path).expect("read back");
    assert!(
        text.contains("# Extra clocks.") && text.contains("# Holidays."),
        "{text}"
    );
    assert_eq!(load(&path).zones, ["Asia/Tokyo", "America/New_York"]);
}

#[test]
fn a_fifth_zone_is_refused() {
    let (_dir, path) = file("zones = [\"UTC\", \"Asia/Tokyo\", \"Europe/London\", \"America/Denver\"]\n");

    assert!(matches!(
        add_zone(&path, "America/New_York"),
        Err(SettingsError::TooManyZones)
    ));
}

#[test]
fn removing_a_zone_takes_it_out_and_an_unknown_one_is_an_error() {
    let (_dir, path) = file(SEED);

    remove_zone(&path, "Asia/Tokyo").expect("removed");

    assert_eq!(load(&path).zones, [] as [std::string::String; 0]);
    assert!(matches!(
        remove_zone(&path, "Asia/Tokyo"),
        Err(SettingsError::NoZone(_))
    ));
}

#[test]
fn a_countdown_is_added_replaced_and_removed_with_the_bar() {
    let (_dir, path) = file("");

    add_countdown(&path, "Trip", "2026-12-19").expect("added");
    add_countdown(&path, "Trip", "2026-12-20").expect("replaced");
    set(&path, "bar-countdown", "Trip", &Countries::bundled()).expect("in the bar");

    let settings = load(&path);
    assert_eq!(
        settings.countdowns,
        [Countdown {
            name: "Trip".into(),
            at: "2026-12-20".into()
        }]
    );
    assert_eq!(settings.bar_countdown, "Trip");

    remove_countdown(&path, "Trip").expect("removed");

    let settings = load(&path);
    assert_eq!(settings.countdowns, [] as [super::Countdown; 0]);
    assert_eq!(settings.bar_countdown, "");
}

#[test]
fn the_plain_settings_take_their_values_and_refuse_others() {
    let (_dir, path) = file(SEED);
    let countries = Countries::bundled();

    set(&path, "holidays", "DE", &countries).expect("a country");
    set(&path, "first-weekday", "sun", &countries).expect("a weekday");
    set(&path, "analog", "toggle", &countries).expect("toggled");

    let settings = load(&path);
    assert_eq!(settings.holidays.as_deref(), Some("de"));
    assert_eq!(settings.first_weekday, Weekday::Sunday);
    assert!(settings.analog);

    set(&path, "holidays", "none", &countries).expect("none");
    assert_eq!(load(&path).holidays.as_deref(), Some(""));
    assert!(matches!(
        set(&path, "holidays", "xx", &countries),
        Err(SettingsError::NoCountry(..))
    ));
    assert!(matches!(
        set(&path, "analog", "maybe", &countries),
        Err(SettingsError::Value { .. })
    ));
    assert!(matches!(
        set(&path, "bar-countdown", "Nope", &countries),
        Err(SettingsError::NoCountdown(_))
    ));
    assert!(matches!(
        set(&path, "colour", "red", &countries),
        Err(SettingsError::NoSetting(_))
    ));
}

#[test]
fn a_users_holiday_file_beside_the_settings_is_read_and_a_broken_one_reported() {
    let dir = tempfile::tempdir().expect("a temp dir");
    fs::write(
        dir.path().join(HOLIDAYS_FILE),
        "[xx]\nname = \"Mine\"\ndays = [{ name = \"Day\", date = \"10-24\" }]\n",
    )
    .expect("write");

    let (countries, problem) = countries(Some(dir.path()));

    assert!(problem.is_none());
    assert!(countries.holidays("xx").is_some());

    fs::write(dir.path().join(HOLIDAYS_FILE), "not toml [").expect("write");
    let (countries, problem) = super::countries(Some(dir.path()));
    assert!(problem.is_some());
    assert!(countries.holidays("pl").is_some());
}

#[test]
fn the_time_format_and_date_order_follow_the_locale_until_set() {
    let settings = Settings::parse("").expect("parses");

    assert_eq!(settings.time_format, None);
    assert_eq!(settings.date_order, None);

    let settings = Settings::parse("time_format = \"24h\"\ndate_order = \"day-month\"\n").expect("parses");

    assert_eq!(settings.time_format, Some(TimeFormat::TwentyFourHour));
    assert_eq!(settings.date_order, Some(DateOrder::DayMonth));
}

#[test]
fn a_time_format_or_date_order_it_does_not_know_is_an_error() {
    assert!(matches!(
        Settings::parse("time_format = \"HH:mm\""),
        Err(SettingsError::Value { key: "time_format", .. })
    ));
    assert!(matches!(
        Settings::parse("date_order = \"year-first\""),
        Err(SettingsError::Value { key: "date_order", .. })
    ));
}

#[test]
fn the_time_format_and_date_order_are_set_and_given_back_to_the_locale() {
    let (_dir, path) = file(SEED);
    let countries = Countries::bundled();

    set(&path, "time-format", "12h", &countries).expect("12-hour");
    set(&path, "date-order", "month-day", &countries).expect("month first");

    let settings = load(&path);
    assert_eq!(settings.time_format, Some(TimeFormat::TwelveHour));
    assert_eq!(settings.date_order, Some(DateOrder::MonthDay));

    set(&path, "time-format", "locale", &countries).expect("the locale's");
    set(&path, "date-order", "locale", &countries).expect("the locale's");

    let text = fs::read_to_string(&path).expect("read back");
    assert!(!text.contains("time_format") && !text.contains("date_order"), "{text}");
    assert!(text.contains("# Holidays."), "{text}");
    assert!(matches!(
        set(&path, "time-format", "25h", &countries),
        Err(SettingsError::Value { .. })
    ));
    assert!(matches!(
        set(&path, "date-order", "sideways", &countries),
        Err(SettingsError::Value { .. })
    ));
}
