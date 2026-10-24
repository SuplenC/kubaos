use jiff::civil::Date;
use jiff::civil::date;

use super::Countries;
use super::Holidays;
use super::easter;

fn names(holidays: &Holidays, year: i16) -> Vec<(Date, String)> {
    holidays
        .in_year(year)
        .into_iter()
        .map(|day| (day.date, day.name))
        .collect()
}

fn country(code: &str) -> Holidays {
    Countries::bundled().holidays(code).expect("a bundled country")
}

#[test]
fn easter_falls_on_the_known_sundays() {
    assert_eq!(easter(2000), date(2000, 4, 23));
    assert_eq!(easter(2024), date(2024, 3, 31));
    assert_eq!(easter(2025), date(2025, 4, 20));
    assert_eq!(easter(2026), date(2026, 4, 5));
    assert_eq!(easter(2027), date(2027, 3, 28));
    assert_eq!(easter(2038), date(2038, 4, 25));
}

#[test]
fn every_bundled_country_parses_and_has_new_years_day() {
    let countries = Countries::bundled();

    let listed: Vec<&str> = countries.list().map(|(code, _)| code).collect();

    assert!(listed.len() >= 10, "{listed:?}");
    for code in listed {
        let holidays = countries.holidays(code).expect("listed");
        assert_eq!(
            holidays.on(date(2026, 1, 1)).as_deref(),
            Some("New Year's Day"),
            "{code}"
        );
    }
}

#[test]
fn poland_moves_its_easter_holidays_and_keeps_christmas_eve_from_2025() {
    let pl = country("pl");

    assert_eq!(pl.on(date(2026, 4, 6)).as_deref(), Some("Easter Monday"));
    assert_eq!(pl.on(date(2026, 6, 4)).as_deref(), Some("Corpus Christi"));
    assert_eq!(pl.on(date(2025, 12, 24)).as_deref(), Some("Christmas Eve"));
    assert_eq!(pl.on(date(2024, 12, 24)), None);
    assert_eq!(pl.on(date(2026, 10, 24)), None);
}

#[test]
fn the_us_keeps_a_weekend_holiday_on_the_nearest_weekday() {
    let us = country("us");

    let days = names(&us, 2026);

    assert!(days.contains(&(date(2026, 7, 4), "Independence Day".into())));
    assert!(days.contains(&(date(2026, 7, 3), "Independence Day (observed)".into())));
    assert!(days.contains(&(date(2026, 11, 26), "Thanksgiving".into())));
    assert!(days.contains(&(date(2026, 5, 25), "Memorial Day".into())));
    assert!(days.contains(&(date(2026, 1, 19), "Martin Luther King Jr. Day".into())));
}

#[test]
fn the_uk_gives_substitute_days_after_a_weekend_christmas() {
    let gb = country("gb");

    let days = names(&gb, 2027);

    assert!(days.contains(&(date(2027, 12, 27), "Christmas Day (substitute day)".into())));
    assert!(days.contains(&(date(2027, 12, 28), "Boxing Day (substitute day)".into())));
}

#[test]
fn kings_day_on_a_sunday_is_kept_on_the_saturday() {
    let nl = country("nl");

    assert_eq!(nl.on(date(2025, 4, 26)).as_deref(), Some("King's Day"));
    assert_eq!(nl.on(date(2025, 4, 27)), None);
    assert_eq!(nl.on(date(2026, 4, 27)).as_deref(), Some("King's Day"));
}

#[test]
fn a_year_comes_sorted_by_date() {
    let days = names(&country("de"), 2026);

    let dates: Vec<Date> = days.iter().map(|(date, _)| *date).collect();
    let mut sorted = dates.clone();
    sorted.sort();
    assert_eq!(dates, sorted);
    assert_eq!(days.first().map(|(_, name)| name.as_str()), Some("New Year's Day"));
}

#[test]
fn a_users_country_replaces_the_bundled_one_whole() {
    let mut countries = Countries::bundled();

    countries
        .add_user("[pl]\nname = \"Poland\"\ndays = [{ name = \"Kuba's Day\", date = \"10-24\" }]\n")
        .expect("parses");

    let pl = countries.holidays("pl").expect("still there");
    assert_eq!(pl.on(date(2026, 10, 24)).as_deref(), Some("Kuba's Day"));
    assert_eq!(pl.on(date(2026, 1, 1)), None);
    assert!(countries.holidays("de").is_some());
}

#[test]
fn a_user_file_that_is_not_holidays_is_an_error() {
    let mut countries = Countries::bundled();

    assert!(countries.add_user("[pl]\ndays = [{ name = \"x\" }]\n").is_err());
}

#[test]
fn country_codes_are_found_in_any_case_and_unknown_ones_are_none() {
    let countries = Countries::bundled();

    assert!(countries.holidays("PL").is_some());
    assert!(countries.holidays("xx").is_none());
}

#[test]
fn no_country_means_no_holidays() {
    let none = Holidays::none();

    assert_eq!(none.on(date(2026, 12, 25)), None);
    assert!(none.in_year(2026).is_empty());
}

#[test]
fn a_holiday_is_found_by_name_from_a_day_on() {
    let pl = country("pl");

    assert_eq!(
        pl.next_named("corpus christi", date(2026, 10, 24)),
        Some(date(2027, 5, 27))
    );
    assert_eq!(
        pl.next_named("independence day", date(2026, 10, 24)),
        Some(date(2026, 11, 11))
    );
    assert_eq!(pl.next_named("all saints", date(2026, 10, 24)), Some(date(2026, 11, 1)));
    assert_eq!(pl.next_named("nothing like it", date(2026, 10, 24)), None);
}
