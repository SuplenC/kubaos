use jiff::Zoned;
use jiff::civil::Weekday;
use jiff::civil::date;
use jiff::tz::TimeZone;

use super::*;
use crate::holidays::Countries;

fn now() -> Zoned {
    "2026-10-24T10:00:00[Europe/Warsaw]".parse().expect("zoned")
}

fn pl() -> Holidays {
    Countries::bundled().holidays("pl").expect("Poland")
}

#[test]
fn a_month_is_six_weeks_starting_on_monday_with_iso_week_numbers() {
    let october = month(2026, 10, Weekday::Monday, date(2026, 10, 24), &pl()).expect("a month");

    assert_eq!(october.title, "October 2026");
    assert_eq!(october.weekdays, ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]);
    assert_eq!(october.weeks.len(), WEEKS);
    assert_eq!(october.weeks[0].days[0].date, date(2026, 9, 28));
    assert!(!october.weeks[0].days[0].in_month);
    assert_eq!(october.weeks[0].number, 40);
    assert_eq!(october.weeks[3].number, 43);
}

#[test]
fn today_weekends_and_holidays_are_marked() {
    let november = month(2026, 11, Weekday::Monday, date(2026, 11, 11), &pl()).expect("a month");

    let days: Vec<&Day> = november.weeks.iter().flat_map(|week| &week.days).collect();
    let eleventh = days
        .iter()
        .find(|day| day.date == date(2026, 11, 11))
        .expect("the 11th");
    assert!(eleventh.today);
    assert_eq!(eleventh.holiday.as_deref(), Some("Independence Day"));
    assert!(
        days.iter()
            .find(|day| day.date == date(2026, 11, 1))
            .expect("the 1st")
            .weekend
    );
}

#[test]
fn a_sunday_first_month_starts_its_rows_on_sunday() {
    let october = month(2026, 10, Weekday::Sunday, date(2026, 10, 24), &Holidays::none()).expect("a month");

    assert_eq!(october.weekdays[0], "Sun");
    assert_eq!(october.weeks[0].days[0].date, date(2026, 9, 27));
    assert_eq!(october.weeks[0].number, 40);
}

#[test]
fn a_january_shows_the_new_years_holiday_of_its_own_year() {
    let january = month(2027, 1, Weekday::Monday, date(2026, 10, 24), &pl()).expect("a month");

    let first = january
        .weeks
        .iter()
        .flat_map(|week| &week.days)
        .find(|day| day.date == date(2027, 1, 1));
    assert_eq!(first.and_then(|day| day.holiday.as_deref()), Some("New Year's Day"));
    assert_eq!(january.weeks[0].number, 53);
}

#[test]
fn today_knows_its_day_and_week_of_the_year() {
    let today = today(&now());

    assert_eq!((today.day_of_year, today.week), (297, 43));
}

#[test]
fn clocks_give_each_zone_its_offset_and_skip_unknown_names() {
    let names = vec![
        "Asia/Tokyo".to_owned(),
        "America/New_York".to_owned(),
        "Atlantis".to_owned(),
    ];

    let clocks = clocks(&names, &Zones::new(), &now());

    assert_eq!(clocks.len(), 2);
    assert_eq!(
        (clocks[0].label.as_str(), clocks[0].offset, clocks[0].relative.as_str()),
        ("Tokyo", 9 * 3600, "+7 h")
    );
    assert_eq!(clocks[1].relative, "\u{2212}6 h");
}

#[test]
fn a_days_details_say_how_far_it_is_and_the_zones_time_that_day() {
    let tokyo = Zones::new().get("tokyo").expect("Tokyo");

    let christmas = day(date(2026, 12, 25), &now(), &pl(), &[tokyo]);

    assert_eq!(christmas.title, "Friday 25 December 2026");
    assert_eq!(christmas.detail, "in 62 days, a Friday · week 52 · day 359");
    assert_eq!(christmas.holiday.as_deref(), Some("Christmas Day"));
    // In winter Warsaw is 8 hours behind Tokyo.
    assert_eq!(christmas.zones[0].time, "18:00");
}

#[test]
fn a_zone_on_another_day_is_marked() {
    let la = Zones::new().get("la").expect("Los Angeles");
    let late: Zoned = "2026-10-24T05:00:00[Europe/Warsaw]".parse().expect("zoned");

    let details = day(date(2026, 10, 24), &late, &Holidays::none(), &[la]);

    assert_eq!(details.zones[0].time, "20:00 \u{2212}1");
}

#[test]
fn a_countdown_to_a_date_ends_at_its_start_here() {
    let tz = TimeZone::get("Europe/Warsaw").expect("Warsaw");

    let end = countdown_end("2026-12-19", &tz).expect("a date");

    assert_eq!(end.to_string(), "2026-12-18T23:00:00Z");
    assert!(countdown_end("2026-12-19T17:00:00+01:00", &tz).is_some());
    assert!(countdown_end("soon", &tz).is_none());
}
