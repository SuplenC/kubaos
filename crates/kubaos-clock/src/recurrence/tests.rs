use jiff::Zoned;
use jiff::civil::Time;
use jiff::civil::Weekday;

use super::*;

fn at(text: &str) -> Zoned {
    format!("{text}[Europe/Warsaw]").parse().expect("a zoned time")
}

fn read(text: &str) -> Recurrence {
    Recurrence::parse(text).unwrap_or_else(|err| panic!("{text}: {err}"))
}

fn next(rule: &str, after: &str) -> String {
    read(rule).next_after(&at(after)).expect("a next time").strftime("%Y-%m-%dT%H:%M%:z").to_string()
}

fn nine() -> Time {
    Time::constant(9, 0, 0, 0)
}

#[test]
fn days_read_with_a_time_or_at_midnight() {
    assert_eq!(read("every day at 9"), Recurrence::Daily(nine()));
    assert_eq!(read("Every day at 9:00"), Recurrence::Daily(nine()));
    assert_eq!(read("daily"), Recurrence::Daily(Time::midnight()));
    assert_eq!(read("daily at 9am"), Recurrence::Daily(nine()));
    assert_eq!(read("9am every day"), Recurrence::Daily(nine()));
    assert_eq!(read("at 21:30"), Recurrence::Daily(Time::constant(21, 30, 0, 0)));
}

#[test]
fn weekdays_read_singular_plural_and_short() {
    let monday_eight = Recurrence::Weekly(Weekday::Monday, Time::constant(8, 0, 0, 0));
    assert_eq!(read("mondays 8am"), monday_eight);
    assert_eq!(read("every Monday at 8"), monday_eight);
    assert_eq!(read("on mon at 8:00"), monday_eight);
    assert_eq!(read("every monday"), Recurrence::Weekly(Weekday::Monday, Time::midnight()));
    assert_eq!(read("every week"), Recurrence::Weekly(Weekday::Monday, Time::midnight()));
    assert_eq!(read("weekly on fridays at 17:00"), Recurrence::Weekly(Weekday::Friday, Time::constant(17, 0, 0, 0)));
}

#[test]
fn days_of_the_month_read_as_ordinals_or_after_the() {
    assert_eq!(read("1st of the month"), Recurrence::Monthly(1, Time::midnight()));
    assert_eq!(read("every month on the 15th"), Recurrence::Monthly(15, Time::midnight()));
    assert_eq!(read("on the 15 of each month at 9"), Recurrence::Monthly(15, nine()));
    assert_eq!(read("monthly"), Recurrence::Monthly(1, Time::midnight()));
    assert_eq!(read("the 31st"), Recurrence::Monthly(31, Time::midnight()));
}

#[test]
fn intervals_read_hours_and_minutes_up_to_a_day() {
    assert_eq!(read("every 2 hours"), Recurrence::Every(120));
    assert_eq!(read("every 30 minutes"), Recurrence::Every(30));
    assert_eq!(read("every 1h30m"), Recurrence::Every(90));
    assert_eq!(read("hourly"), Recurrence::Every(60));
    assert_eq!(read("every hour"), Recurrence::Every(60));
    assert_eq!(read("every minute"), Recurrence::Every(1));
    assert_eq!(read("every 24 hours"), Recurrence::Every(1440));
}

#[test]
fn what_it_cannot_read_names_examples() {
    for text in [
        "",
        "sometimes",
        "every 2 days",
        "every 25 hours",
        "every 0 minutes",
        "every 2 hours at 9",
        "mondays and tuesdays",
        "monday the 15th",
        "every day on mondays",
        "the 32nd",
        "at 25",
    ] {
        let err = Recurrence::parse(text).expect_err(text);
        let message = err.to_string();
        assert!(message.contains("every day at 9") && message.contains("1st of the month"), "{message}");
    }
}

#[test]
fn it_says_itself_back_in_words() {
    assert_eq!(read("every day at 9").words(None), "Every day at 9:00");
    assert_eq!(read("mondays 8am").words(None), "Every Monday at 8:00");
    assert_eq!(read("1st of the month").words(None), "Every month on the 1st at 0:00");
    assert_eq!(read("every month on the 22nd").words(None), "Every month on the 22nd at 0:00");
    assert_eq!(read("the 13th").words(None), "Every month on the 13th at 0:00");
    assert_eq!(read("every 2 hours").words(None), "Every 2 hours");
    assert_eq!(read("hourly").words(None), "Every hour");
    assert_eq!(read("every 90 minutes").words(None), "Every 90 minutes");
    assert_eq!(read("every 30 minutes").words(None), "Every 30 minutes");
    assert_eq!(
        read("every day at 21:30").words(Some(TimeFormat::TwelveHour)),
        "Every day at 9:30 PM"
    );
    assert_eq!(read("every day at 9").phrase(None), "every day at 9:00");
}

#[test]
fn its_words_read_back_to_the_same_rule() {
    for text in [
        "every day at 9",
        "mondays 8am",
        "every sunday at 23:59",
        "1st of the month",
        "every month on the 31st at 18:15",
        "every 2 hours",
        "hourly",
        "every minute",
        "every 45 minutes",
    ] {
        let rule = read(text);
        assert_eq!(read(&rule.to_string()), rule, "{text}");
        assert_eq!(read(&rule.words(Some(TimeFormat::TwelveHour))), rule, "{text}");
    }
}

#[test]
fn a_day_comes_later_today_or_tomorrow() {
    assert_eq!(next("every day at 9", "2026-11-12T08:59:00"), "2026-11-12T09:00+01:00");
    assert_eq!(next("every day at 9", "2026-11-12T09:00:00"), "2026-11-13T09:00+01:00");
}

#[test]
fn a_weekday_comes_this_week_or_next() {
    // 12 November 2026 is a Thursday.
    assert_eq!(next("mondays 8am", "2026-11-12T10:00:00"), "2026-11-16T08:00+01:00");
    assert_eq!(next("thursdays 8am", "2026-11-12T07:00:00"), "2026-11-12T08:00+01:00");
    assert_eq!(next("thursdays 8am", "2026-11-12T08:00:00"), "2026-11-19T08:00+01:00");
}

#[test]
fn a_day_of_the_month_past_its_end_is_the_last_day() {
    assert_eq!(next("the 31st", "2026-01-31T00:00:00"), "2026-02-28T00:00+01:00");
    assert_eq!(next("the 31st", "2028-01-31T12:00:00"), "2028-02-29T00:00+01:00");
    assert_eq!(next("the 31st", "2026-04-01T00:00:00"), "2026-04-30T00:00+02:00");
    assert_eq!(next("the 31st", "2026-04-30T00:00:00"), "2026-05-31T00:00+02:00");
    assert_eq!(next("1st of the month", "2026-12-15T00:00:00"), "2027-01-01T00:00+01:00");
}

#[test]
fn intervals_fall_on_the_clock_from_midnight() {
    assert_eq!(next("every 2 hours", "2026-11-12T09:15:00"), "2026-11-12T10:00+01:00");
    assert_eq!(next("every 30 minutes", "2026-11-12T09:30:00"), "2026-11-12T10:00+01:00");
    assert_eq!(next("every 5 hours", "2026-11-12T20:00:00"), "2026-11-13T00:00+01:00");
    assert_eq!(next("every 24 hours", "2026-11-12T00:00:00"), "2026-11-13T00:00+01:00");
}

#[test]
fn a_time_skipped_by_summer_time_comes_an_hour_later() {
    // Warsaw goes from 2:00 to 3:00 on 29 March 2026.
    assert_eq!(next("every day at 2:30", "2026-03-29T00:00:00"), "2026-03-29T03:30+02:00");
    assert_eq!(next("every day at 2:30", "2026-03-29T03:30:00"), "2026-03-30T02:30+02:00");
    assert_eq!(next("hourly", "2026-03-29T01:30:00"), "2026-03-29T03:00+02:00");
}

#[test]
fn a_time_that_comes_twice_in_autumn_counts_once() {
    // Warsaw goes from 3:00 back to 2:00 on 25 October 2026, so 2:30 comes twice.
    let first = at("2026-10-25T00:00:00");
    let rule = read("every day at 2:30");
    let once = rule.next_after(&first).expect("a next time");
    assert_eq!(once.strftime("%H:%M%:z").to_string(), "02:30+02:00");
    let then = rule.next_after(&once).expect("a next time");
    assert_eq!(then.strftime("%Y-%m-%dT%H:%M%:z").to_string(), "2026-10-26T02:30+01:00");
}

#[test]
fn midnight_rules_on_a_day_without_midnight_come_at_the_first_moment() {
    // Santiago moves its clocks at midnight: 6 September 2026 starts at 1:00.
    let after: Zoned = "2026-09-05T23:30:00[America/Santiago]".parse().expect("a zoned time");
    let at = read("daily").next_after(&after).expect("a next time");
    assert_eq!(at.strftime("%Y-%m-%dT%H:%M").to_string(), "2026-09-06T01:00");
}
