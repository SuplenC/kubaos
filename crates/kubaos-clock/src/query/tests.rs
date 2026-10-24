use std::sync::LazyLock;

use jiff::Zoned;
use jiff::civil::date;

use super::Answer;
use super::Context;
use super::evaluate;
use crate::holidays::Countries;
use crate::holidays::Holidays;
use crate::zones::Zones;

static ZONES: LazyLock<Zones> = LazyLock::new(Zones::new);
static PL: LazyLock<Holidays> = LazyLock::new(|| Countries::bundled().holidays("pl").expect("Poland"));

/// Saturday 24 October 2026, 10:00 in Warsaw: the day before the clocks go back.
fn now() -> Zoned {
    "2026-10-24T10:00:00[Europe/Warsaw]".parse().expect("a zoned time")
}

fn ask(query: &str) -> Option<Answer> {
    let now = now();
    evaluate(query, &Context::new(&now, &PL, &ZONES))
}

#[track_caller]
fn title(query: &str) -> String {
    ask(query).unwrap_or_else(|| panic!("no answer for {query:?}")).title
}

#[track_caller]
fn detail(query: &str) -> String {
    ask(query).unwrap_or_else(|| panic!("no answer for {query:?}")).detail
}

#[test]
fn weekdays_name_the_next_one_and_today_counts_for_a_bare_weekday() {
    assert_eq!(title("next friday"), "Friday 30 October 2026");
    assert_eq!(title("friday"), "Friday 30 October 2026");
    assert_eq!(title("Fri"), "Friday 30 October 2026");
    assert_eq!(title("saturday"), "Saturday 24 October 2026");
    assert_eq!(title("this saturday"), "Saturday 24 October 2026");
    assert_eq!(title("next saturday"), "Saturday 31 October 2026");
    assert_eq!(title("last friday"), "Friday 23 October 2026");
    assert_eq!(title("last saturday"), "Saturday 17 October 2026");
}

#[test]
fn a_date_says_how_far_it_is_its_weekday_and_its_week() {
    assert_eq!(title("2027-03-01"), "Monday 1 March 2027");
    assert_eq!(detail("2027-03-01"), "in 128 days, a Monday · week 9");
    assert_eq!(detail("next friday"), "in 6 days, a Friday · week 44");
    assert_eq!(detail("today"), "today, a Saturday · week 43");
}

#[test]
fn a_day_and_month_without_a_year_is_the_next_one() {
    assert_eq!(title("24 dec"), "Thursday 24 December 2026");
    assert_eq!(title("dec 24"), "Thursday 24 December 2026");
    assert_eq!(title("24th December"), "Thursday 24 December 2026");
    assert_eq!(title("24 of december"), "Thursday 24 December 2026");
    assert_eq!(title("1 mar"), "Monday 1 March 2027");
    assert_eq!(title("24 oct"), "Saturday 24 October 2026");
    assert_eq!(title("24 december 2027"), "Friday 24 December 2027");
    assert_eq!(title("dec 24, 2027"), "Friday 24 December 2027");
    assert_eq!(title("24.12.2026"), "Thursday 24 December 2026");
}

#[test]
fn a_holiday_on_the_day_is_named() {
    assert!(detail("24 dec").ends_with("· Christmas Eve"), "{}", detail("24 dec"));
}

#[test]
fn relative_days_count_from_today() {
    assert_eq!(title("today"), "Saturday 24 October 2026");
    assert_eq!(title("tomorrow"), "Sunday 25 October 2026");
    assert_eq!(title("yesterday"), "Friday 23 October 2026");
    assert_eq!(title("day after tomorrow"), "Monday 26 October 2026");
    assert_eq!(title("in 3 weeks"), "Saturday 14 November 2026");
    assert_eq!(title("in a week"), "Saturday 31 October 2026");
    assert_eq!(title("in 10d"), "Tuesday 3 November 2026");
    assert_eq!(title("3 days ago"), "Wednesday 21 October 2026");
    assert_eq!(title("2 weeks from now"), "Saturday 7 November 2026");
    assert_eq!(title("end of month"), "Saturday 31 October 2026");
    assert_eq!(title("end of the year"), "Thursday 31 December 2026");
}

#[test]
fn named_days_and_the_countrys_holidays_are_dates() {
    assert_eq!(title("christmas"), "Friday 25 December 2026");
    assert_eq!(title("Christmas Eve"), "Thursday 24 December 2026");
    assert_eq!(title("new year's eve"), "Thursday 31 December 2026");
    assert_eq!(title("new year"), "Friday 1 January 2027");
    assert_eq!(title("easter"), "Sunday 28 March 2027");
    assert_eq!(title("halloween"), "Saturday 31 October 2026");
    assert_eq!(title("independence day"), "Wednesday 11 November 2026");
    assert_eq!(title("corpus christi"), "Thursday 27 May 2027");
}

#[test]
fn days_until_and_since_count_whole_days() {
    assert_eq!(title("days until christmas"), "62 days");
    assert_eq!(
        detail("days until christmas"),
        "until Friday 25 December 2026 · 8 weeks 6 days"
    );
    assert_eq!(title("until christmas"), "62 days");
    assert_eq!(title("days till 1 jan"), "69 days");
    assert_eq!(title("days since 2020-01-01"), "2488 days");
    assert_eq!(
        detail("days since 2020-01-01"),
        "since Wednesday 1 January 2020 · 355 weeks 3 days"
    );
    assert_eq!(title("weeks until christmas"), "8 weeks 6 days");
    assert_eq!(title("how many days until easter"), "155 days");
    assert_eq!(title("days until tomorrow"), "1 day");
}

#[test]
fn how_long_until_a_time_counts_days_and_hours() {
    assert_eq!(title("how long until friday 17:00"), "6 days 7 hours");
    assert_eq!(title("time until 18:30"), "8 hours 30 minutes");
}

#[test]
fn age_is_whole_years_with_the_next_birthday() {
    assert_eq!(title("age 1993-05-12"), "33 years");
    assert_eq!(
        detail("age 1993-05-12"),
        "33 years 5 months 12 days · born on a Wednesday · next birthday in 200 days"
    );
    assert_eq!(title("how old is 12 may 1993"), "33 years");
}

#[test]
fn a_range_counts_days_weeks_months_and_working_days() {
    assert_eq!(title("12 mar to 4 jul"), "114 days");
    assert_eq!(
        detail("12 mar to 4 jul"),
        "12 March 2027 to 4 July 2027 · 16 weeks 2 days · 3 months 22 days · 78 working days"
    );
    assert_eq!(title("from 2026-12-21 to 2027-01-08"), "18 days");
    assert!(
        detail("2026-12-21 - 2027-01-08").ends_with("· 11 working days"),
        "{}",
        detail("2026-12-21 - 2027-01-08")
    );
    assert_eq!(title("between 1 nov and 30 nov"), "29 days");
}

#[test]
fn a_span_moves_a_date_or_a_time() {
    assert_eq!(title("today + 90d"), "Friday 22 January 2027");
    assert_eq!(title("today+90d"), "Friday 22 January 2027");
    assert_eq!(title("today plus 2 weeks"), "Saturday 7 November 2026");
    assert_eq!(title("24 dec - 2 weeks"), "Thursday 10 December 2026");
    assert_eq!(title("2027-03-01 + 1 month"), "Thursday 1 April 2027");
    assert_eq!(title("christmas + 1 week 2 days"), "Sunday 3 January 2027");
    assert_eq!(title("3 weeks before christmas"), "Friday 4 December 2026");
    assert_eq!(title("90 days after 2027-01-01"), "Thursday 1 April 2027");
}

#[test]
fn hours_move_the_clock_across_the_dst_change() {
    // The clocks go back an hour at 03:00 on Sunday 25 October: 36 real hours end at 21:00.
    assert_eq!(title("now + 36h"), "Sunday 25 October 2026, 21:00");
    assert_eq!(detail("now + 36h"), "in 1 day 11 hours");
    assert_eq!(title("now + 1h30m"), "Saturday 24 October 2026, 11:30");
    assert_eq!(title("friday 17:00 - 3h"), "Friday 30 October 2026, 14:00");
    assert_eq!(title("tomorrow 9am"), "Sunday 25 October 2026, 09:00");
}

#[test]
fn a_time_in_a_zone_is_shown_here() {
    assert_eq!(title("15:00 tokyo"), "08:00 in Warsaw");
    assert_eq!(
        detail("15:00 tokyo"),
        "15:00 in Tokyo · Saturday 24 October · Warsaw is 7 h behind"
    );
    assert_eq!(title("3pm Tokyo"), "08:00 in Warsaw");
}

#[test]
fn a_time_converts_from_one_zone_to_another() {
    assert_eq!(title("9am NY in Warsaw"), "15:00 in Warsaw");
    assert_eq!(title("9am new york to warsaw"), "15:00 in Warsaw");
    assert_eq!(title("17:00 in new york"), "11:00 in New York");
    assert_eq!(
        detail("17:00 in new york"),
        "17:00 in Warsaw · Saturday 24 October · New York is 6 h behind"
    );
}

#[test]
fn a_conversion_past_the_dst_change_lands_on_the_next_day() {
    assert_eq!(title("9pm pt to cet"), "05:00 in CET");
    assert_eq!(
        detail("9pm pt to cet"),
        "21:00 in PT · Sunday 25 October · CET is 8 h ahead"
    );
}

#[test]
fn the_time_in_a_zone_is_now_there() {
    assert_eq!(title("time in tokyo"), "17:00 in Tokyo");
    assert_eq!(title("now in tokyo"), "17:00 in Tokyo");
    assert_eq!(title("tokyo time"), "17:00 in Tokyo");
    assert_eq!(detail("tokyo time"), "Saturday 24 October · Tokyo is 7 h ahead");
}

#[test]
fn an_answer_carries_an_iso_form_and_the_date_to_show() {
    let date_answer = ask("next friday").expect("an answer");
    assert_eq!(date_answer.iso, "2026-10-30");
    assert_eq!(date_answer.date, Some(date(2026, 10, 30)));

    let count = ask("days until christmas").expect("an answer");
    assert_eq!(count.iso, "P62D");
    assert_eq!(count.date, Some(date(2026, 12, 25)));

    let moved = ask("now + 36h").expect("an answer");
    assert_eq!(moved.iso, "2026-10-25T21:00:00+01:00");

    let zoned = ask("15:00 tokyo").expect("an answer");
    assert_eq!(zoned.iso, "2026-10-24T08:00:00+02:00");

    let range = ask("12 mar to 4 jul").expect("an answer");
    assert_eq!(range.iso, "2027-03-12/2027-07-04");
}

#[test]
fn queries_that_are_not_dates_get_no_answer() {
    for query in [
        "",
        "firefox",
        "next",
        "12",
        "15:00",
        "volume 40",
        "days until",
        "age",
        "tokyo",
        "in",
        "may",
    ] {
        assert!(ask(query).is_none(), "{query:?} answered {:?}", ask(query));
    }
}

#[test]
fn a_question_mark_and_when_is_are_ignored() {
    assert_eq!(title("when is easter?"), "Sunday 28 March 2027");
    assert_eq!(title("days until christmas?"), "62 days");
}
