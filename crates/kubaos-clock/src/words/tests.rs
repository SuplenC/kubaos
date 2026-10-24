use jiff::civil::Weekday;
use jiff::civil::date;

use super::*;

#[test]
fn weekdays_and_months_are_read_from_three_letters_on() {
    assert_eq!(weekday("fri"), Some(Weekday::Friday));
    assert_eq!(weekday("thurs"), Some(Weekday::Thursday));
    assert_eq!(weekday("Wednesday".to_lowercase().as_str()), Some(Weekday::Wednesday));
    assert_eq!(weekday("fr"), None);
    assert_eq!(month("dec"), Some(12));
    assert_eq!(month("sept"), Some(9));
    assert_eq!(month("may"), Some(5));
    assert_eq!(month("ma"), None);
    assert_eq!(month("monday"), None);
}

#[test]
fn a_query_splits_on_signs_but_not_inside_dates_or_names() {
    assert_eq!(split("today+90d"), ["today", "+", "90d"]);
    assert_eq!(split("2027-03-01 - 3d"), ["2027-03-01", "-", "3d"]);
    assert_eq!(split("12 Mar - 4 Jul"), ["12", "mar", "-", "4", "jul"]);
    assert_eq!(split("15:00 Port-au-Prince"), ["15:00", "port-au-prince"]);
    assert_eq!(split("dec 25, 2026"), ["dec", "25", "2026"]);
    assert_eq!(split("now-36h"), ["now", "-", "36h"]);
}

#[test]
fn dates_and_distances_read_as_people_say_them() {
    assert_eq!(long_date(date(2026, 12, 25)), "Friday 25 December 2026");
    assert_eq!(relative_days(0), "today");
    assert_eq!(relative_days(1), "tomorrow");
    assert_eq!(relative_days(-1), "yesterday");
    assert_eq!(relative_days(61), "in 61 days");
    assert_eq!(relative_days(-3), "3 days ago");
    assert_eq!(count(1, "week"), "1 week");
    assert_eq!(a_weekday(Weekday::Thursday), "a Thursday");
}

#[test]
fn offsets_are_hours_with_a_true_minus() {
    assert_eq!(offset_label(0), "same time");
    assert_eq!(offset_label(7 * 3600), "+7 h");
    assert_eq!(offset_label(-6 * 3600), "\u{2212}6 h");
    assert_eq!(offset_label(3 * 3600 + 1800), "+3:30 h");
    assert_eq!(offset_label(-(9 * 3600 + 1800)), "\u{2212}9:30 h");
}

#[test]
fn plain_names_compare_as_typed() {
    assert_eq!(plain("St. Stephen's Day"), "st stephens day");
    assert_eq!(plain("  New_York "), "new york");
}
