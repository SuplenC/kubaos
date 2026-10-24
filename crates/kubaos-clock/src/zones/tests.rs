use super::Zones;
use super::city;

fn name(zones: &Zones, typed: &str) -> Option<(String, String)> {
    zones.get(typed).map(|zone| (zone.name, zone.label))
}

#[test]
fn a_zone_is_found_by_its_iana_name_its_city_or_an_alias() {
    let zones = Zones::new();

    assert_eq!(
        name(&zones, "America/New_York"),
        Some(("America/New_York".into(), "New York".into()))
    );
    assert_eq!(name(&zones, "tokyo"), Some(("Asia/Tokyo".into(), "Tokyo".into())));
    assert_eq!(
        name(&zones, "New York"),
        Some(("America/New_York".into(), "New York".into()))
    );
    assert_eq!(name(&zones, "NY"), Some(("America/New_York".into(), "New York".into())));
    assert_eq!(name(&zones, "krakow"), Some(("Europe/Warsaw".into(), "Krakow".into())));
    assert_eq!(
        name(&zones, "san francisco"),
        Some(("America/Los_Angeles".into(), "San Francisco".into()))
    );
}

#[test]
fn an_abbreviation_keeps_its_letters_as_its_label() {
    let zones = Zones::new();

    assert_eq!(name(&zones, "pt"), Some(("America/Los_Angeles".into(), "PT".into())));
    assert_eq!(name(&zones, "UTC"), Some(("UTC".into(), "UTC".into())));
    assert_eq!(name(&zones, "cet"), Some(("Europe/Paris".into(), "CET".into())));
}

#[test]
fn the_longest_name_at_the_front_wins() {
    let zones = Zones::new();

    let (zone, used) = zones.find(&["new", "york", "in", "warsaw"]).expect("found");

    assert_eq!(zone.name, "America/New_York");
    assert_eq!(used, 2);
}

#[test]
fn words_that_name_no_zone_find_nothing() {
    let zones = Zones::new();

    assert!(zones.get("friday").is_none());
    assert!(zones.find(&["in", "warsaw"]).is_none());
}

#[test]
fn a_city_is_written_with_spaces() {
    assert_eq!(city("America/Argentina/Buenos_Aires"), "Buenos Aires");
    assert_eq!(city("UTC"), "UTC");
}
