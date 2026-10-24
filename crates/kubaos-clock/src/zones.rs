//! Time zones by the names people type: an IANA name (`America/New_York`), the city it ends in
//! (`tokyo`, `new york`), a city or country the database doesn't name (`krakow`, `japan`), or an
//! abbreviation (`PT`, `CET`). The zones themselves come from the system's database.
//!
//! # Owns
//!
//! - The index of names, the aliases and abbreviations, and a zone's label.
//!
//! # Not here
//!
//! - Converting times between zones: [`crate::query`].

use std::collections::HashMap;

use jiff::tz;
use jiff::tz::TimeZone;

use crate::words;

/// Names the database doesn't have, and the zone each means. An abbreviation means the zone it
/// is used in all year, so `PST` in July is Pacific time, as people mean it.
const ALIASES: &[(&str, &str)] = &[
    ("utc", "UTC"),
    ("gmt", "UTC"),
    ("z", "UTC"),
    ("pt", "America/Los_Angeles"),
    ("pst", "America/Los_Angeles"),
    ("pdt", "America/Los_Angeles"),
    ("mt", "America/Denver"),
    ("mst", "America/Denver"),
    ("mdt", "America/Denver"),
    ("ct", "America/Chicago"),
    ("cst", "America/Chicago"),
    ("cdt", "America/Chicago"),
    ("et", "America/New_York"),
    ("est", "America/New_York"),
    ("edt", "America/New_York"),
    ("akst", "America/Anchorage"),
    ("hst", "Pacific/Honolulu"),
    ("bst", "Europe/London"),
    ("wet", "Europe/Lisbon"),
    ("west", "Europe/Lisbon"),
    ("cet", "Europe/Paris"),
    ("cest", "Europe/Paris"),
    ("eet", "Europe/Athens"),
    ("eest", "Europe/Athens"),
    ("msk", "Europe/Moscow"),
    ("ist", "Asia/Kolkata"),
    ("jst", "Asia/Tokyo"),
    ("kst", "Asia/Seoul"),
    ("hkt", "Asia/Hong_Kong"),
    ("sgt", "Asia/Singapore"),
    ("aest", "Australia/Sydney"),
    ("aedt", "Australia/Sydney"),
    ("awst", "Australia/Perth"),
    ("nzst", "Pacific/Auckland"),
    ("nzdt", "Pacific/Auckland"),
    ("brt", "America/Sao_Paulo"),
];

/// Cities and countries the database names no zone after, with the zone they keep.
const PLACES: &[(&str, &str)] = &[
    ("ny", "America/New_York"),
    ("nyc", "America/New_York"),
    ("washington", "America/New_York"),
    ("dc", "America/New_York"),
    ("boston", "America/New_York"),
    ("miami", "America/New_York"),
    ("atlanta", "America/New_York"),
    ("montreal", "America/Toronto"),
    ("la", "America/Los_Angeles"),
    ("sf", "America/Los_Angeles"),
    ("san francisco", "America/Los_Angeles"),
    ("seattle", "America/Los_Angeles"),
    ("dallas", "America/Chicago"),
    ("houston", "America/Chicago"),
    ("austin", "America/Chicago"),
    ("hawaii", "Pacific/Honolulu"),
    ("rio", "America/Sao_Paulo"),
    ("rio de janeiro", "America/Sao_Paulo"),
    ("uk", "Europe/London"),
    ("england", "Europe/London"),
    ("edinburgh", "Europe/London"),
    ("manchester", "Europe/London"),
    ("munich", "Europe/Berlin"),
    ("frankfurt", "Europe/Berlin"),
    ("hamburg", "Europe/Berlin"),
    ("cologne", "Europe/Berlin"),
    ("germany", "Europe/Berlin"),
    ("milan", "Europe/Rome"),
    ("naples", "Europe/Rome"),
    ("italy", "Europe/Rome"),
    ("barcelona", "Europe/Madrid"),
    ("spain", "Europe/Madrid"),
    ("france", "Europe/Paris"),
    ("geneva", "Europe/Zurich"),
    ("krakow", "Europe/Warsaw"),
    ("cracow", "Europe/Warsaw"),
    ("gdansk", "Europe/Warsaw"),
    ("wroclaw", "Europe/Warsaw"),
    ("poznan", "Europe/Warsaw"),
    ("lodz", "Europe/Warsaw"),
    ("poland", "Europe/Warsaw"),
    ("kiev", "Europe/Kyiv"),
    ("st petersburg", "Europe/Moscow"),
    ("saint petersburg", "Europe/Moscow"),
    ("beijing", "Asia/Shanghai"),
    ("china", "Asia/Shanghai"),
    ("mumbai", "Asia/Kolkata"),
    ("bombay", "Asia/Kolkata"),
    ("delhi", "Asia/Kolkata"),
    ("new delhi", "Asia/Kolkata"),
    ("bangalore", "Asia/Kolkata"),
    ("india", "Asia/Kolkata"),
    ("osaka", "Asia/Tokyo"),
    ("kyoto", "Asia/Tokyo"),
    ("japan", "Asia/Tokyo"),
    ("korea", "Asia/Seoul"),
    ("canberra", "Australia/Sydney"),
];

/// A zone, with the name it is shown by.
#[derive(Clone, Debug)]
pub struct Zone {
    pub tz: TimeZone,
    /// The IANA name, as the settings keep it.
    pub name: String,
    /// What the popup and the answers call it: the city (`New York`), or an abbreviation as
    /// typed (`CET`).
    pub label: String,
}

/// The names a zone is found by.
#[derive(Clone, Debug, Default)]
pub struct Zones {
    /// Plain name (see [`words::plain`]) to IANA name.
    names: HashMap<String, String>,
}

impl Zones {
    /// The index of the system's zones, with the aliases above.
    #[must_use]
    pub fn new() -> Self {
        let mut names = HashMap::new();
        for name in tz::db().available() {
            let name = name.as_str();
            names.entry(words::plain(name)).or_insert_with(|| name.to_owned());
            if let Some((_, city)) = name.rsplit_once('/') {
                names.entry(words::plain(city)).or_insert_with(|| name.to_owned());
            }
        }
        for (place, name) in PLACES {
            names.insert((*place).to_owned(), (*name).to_owned());
        }
        Zones { names }
    }

    /// The zone named by `name` whole.
    pub fn get(&self, name: &str) -> Option<Zone> {
        let plain = words::plain(name);
        if let Some(&(_, iana)) = ALIASES.iter().find(|(alias, _)| *alias == plain) {
            return zone(iana, name.trim().to_uppercase());
        }
        let iana = self.names.get(&plain)?;
        let label = if PLACES.iter().any(|(place, _)| *place == plain && place.len() > 3) {
            title_case(&plain)
        } else {
            city(iana)
        };
        zone(iana, label)
    }

    /// The zone the first words of `words` name, and how many words that took: three words at
    /// most, the longest name first, so `new york` wins over `new`.
    pub fn find(&self, words: &[&str]) -> Option<(Zone, usize)> {
        (1..=words.len().min(3)).rev().find_map(|n| Some((self.get(&words[..n].join(" "))?, n)))
    }

    /// The zone `tz` as the popup shows it, by its IANA name.
    pub fn of(tz: &TimeZone) -> Zone {
        let name = tz.iana_name().unwrap_or("UTC").to_owned();
        Zone {
            tz: tz.clone(),
            label: city(&name),
            name,
        }
    }
}

fn zone(iana: &str, label: String) -> Option<Zone> {
    Some(Zone {
        tz: TimeZone::get(iana).ok()?,
        name: iana.to_owned(),
        label,
    })
}

/// The city an IANA name ends in, as written: `America/New_York` is `New York`.
pub fn city(iana: &str) -> String {
    iana.rsplit('/').next().unwrap_or(iana).replace('_', " ")
}

fn title_case(plain: &str) -> String {
    plain
        .split(' ')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests;
