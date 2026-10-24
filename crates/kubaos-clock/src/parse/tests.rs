use jiff::SignedDuration;
use jiff::Zoned;
use jiff::civil::Time;

use super::Units;
use super::alarm;
use super::duration;
use super::span;
use super::time;
use crate::zones::Zones;

#[test]
fn times_of_day_read_in_24_and_12_hour_forms() {
    assert_eq!(time(&["15:00"]), Some((Time::constant(15, 0, 0, 0), 1)));
    assert_eq!(time(&["9am"]), Some((Time::constant(9, 0, 0, 0), 1)));
    assert_eq!(time(&["9:30", "pm"]), Some((Time::constant(21, 30, 0, 0), 2)));
    assert_eq!(time(&["12am"]), Some((Time::constant(0, 0, 0, 0), 1)));
    assert_eq!(time(&["12pm"]), Some((Time::constant(12, 0, 0, 0), 1)));
    assert_eq!(time(&["noon"]), Some((Time::constant(12, 0, 0, 0), 1)));
    assert_eq!(time(&["midnight"]), Some((Time::constant(0, 0, 0, 0), 1)));
}

#[test]
fn numbers_and_ordinals_are_not_times() {
    assert_eq!(time(&["12"]), None);
    assert_eq!(time(&["24th"]), None);
    assert_eq!(time(&["25:00"]), None);
    assert_eq!(time(&["13pm"]), None);
    assert_eq!(time(&["90d"]), None);
}

#[test]
fn spans_read_words_compact_forms_and_articles() {
    let (units, used) = span(&["1", "week", "and", "2", "days", "later"]).expect("a span");
    assert_eq!((units.weeks, units.days, used), (1, 2, 5));

    let (units, used) = span(&["1h30m"]).expect("a span");
    assert_eq!((units.hours, units.minutes, used), (1, 30, 1));

    let (units, _) = span(&["a", "month"]).expect("a span");
    assert_eq!(
        units,
        Units {
            months: 1,
            ..Units::default()
        }
    );

    let (units, _) = span(&["3mo"]).expect("a span");
    assert_eq!(units.months, 3);

    assert!(span(&["90x"]).is_none());
    assert!(span(&["friday"]).is_none());
}

#[test]
fn a_timer_takes_spans_minutes_alone_or_a_clock() {
    assert_eq!(duration("5m"), Some(SignedDuration::from_mins(5)));
    assert_eq!(duration("1h30m"), Some(SignedDuration::from_mins(90)));
    assert_eq!(duration("90 seconds"), Some(SignedDuration::from_secs(90)));
    assert_eq!(duration("25"), Some(SignedDuration::from_mins(25)));
    assert_eq!(duration("1:30"), Some(SignedDuration::from_secs(90)));
    assert_eq!(duration("1:00:00"), Some(SignedDuration::from_hours(1)));
    assert_eq!(duration("2 hours 15 min"), Some(SignedDuration::from_mins(135)));
}

#[test]
fn a_timer_refuses_nothing_months_and_words() {
    assert_eq!(duration(""), None);
    assert_eq!(duration("0"), None);
    assert_eq!(duration("1 month"), None);
    assert_eq!(duration("next friday"), None);
}

#[test]
fn an_alarm_rings_the_next_time_its_time_comes_round() {
    let now: Zoned = "2026-10-24T10:00:00[Europe/Warsaw]".parse().expect("zoned");
    let zones = Zones::new();

    let ring = |text: &str| alarm(text, &now, &zones).map(|at| at.strftime("%Y-%m-%d %H:%M").to_string());

    assert_eq!(ring("7:30").as_deref(), Some("2026-10-25 07:30"));
    assert_eq!(ring("11am").as_deref(), Some("2026-10-24 11:00"));
    assert_eq!(ring("friday 17:00").as_deref(), Some("2026-10-30 17:00"));
    assert_eq!(ring("yesterday 9am"), None);
    assert_eq!(ring("soon"), None);
}
