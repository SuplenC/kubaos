//! Display names for locales and keymaps, and a time zone list for the demo
//! probe (the real one comes from `kubaos_core::engine::probe`).
//!
//! The names come from Ubuntu 26.04's own glibc and xkb data
//! (`assets/make-tables.sh` writes `assets/locales.tsv` and `keymaps.tsv`).
//! Anything the tables don't know still gets a readable name, built from the
//! parts they do know ("en_SE.UTF-8" -> "English (Sweden)"), so no list ever
//! shows a bare code as its name.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

const LOCALES_TSV: &str = include_str!("../assets/locales.tsv");
const KEYMAPS_TSV: &str = include_str!("../assets/keymaps.tsv");

struct Tables {
    locales: HashMap<&'static str, &'static str>,
    keymaps: HashMap<&'static str, &'static str>,
    /// "en" -> "English", from the locale names.
    languages: HashMap<&'static str, &'static str>,
    /// "SE" -> "Sweden".
    territories: HashMap<&'static str, &'static str>,
}

fn tsv(s: &'static str) -> HashMap<&'static str, &'static str> {
    s.lines().filter_map(|l| l.split_once('\t')).collect()
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let locales = tsv(LOCALES_TSV);
        let mut languages = HashMap::new();
        let mut territories = HashMap::new();
        for (code, name) in &locales {
            let (lang, terr, _) = split_locale(code);
            let (lname, rest) = match name.split_once(" (") {
                Some((l, r)) => (l, r.trim_end_matches(')')),
                None => (*name, ""),
            };
            languages.entry(lang).or_insert(lname);
            if let (Some(t), Some(tname)) = (terr, rest.split(", ").next())
                && !tname.is_empty()
            {
                territories.entry(t).or_insert(tname);
            }
        }
        Tables {
            locales,
            keymaps: tsv(KEYMAPS_TSV),
            languages,
            territories,
        }
    })
}

/// "sr_RS.UTF-8@latin" -> ("sr", Some("RS"), Some("latin")).
fn split_locale(code: &str) -> (&str, Option<&str>, Option<&str>) {
    let (main, modifier) = match code.split_once('@') {
        Some((m, x)) => (m, Some(x)),
        None => (code, None),
    };
    let main = main.split('.').next().unwrap_or(main);
    match main.split_once('_') {
        Some((l, t)) => (l, Some(t), modifier),
        None => (main, None, modifier),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// glibc locale -> what the picker shows, e.g. "English (Sweden)".
pub fn locale_name(locale: &str) -> Cow<'static, str> {
    let t = tables();
    if let Some(n) = t.locales.get(locale) {
        return Cow::Borrowed(n);
    }
    // The same locale under another spelling of its charset.
    let (lang, terr, modifier) = split_locale(locale);
    let plain = match (terr, modifier) {
        (Some(tr), Some(m)) => format!("{lang}_{tr}.UTF-8@{m}"),
        (Some(tr), None) => format!("{lang}_{tr}.UTF-8"),
        _ => String::new(),
    };
    if let Some(n) = t.locales.get(plain.as_str()) {
        return Cow::Borrowed(n);
    }
    let lname = t
        .languages
        .get(lang)
        .map(|s| s.to_string())
        .unwrap_or_else(|| lang.to_uppercase());
    let mut extra: Vec<String> = vec![];
    if let Some(tr) = terr {
        extra.push(
            t.territories
                .get(tr)
                .map_or(tr.to_string(), |s| s.to_string()),
        );
    }
    if let Some(m) = modifier {
        extra.push(capitalize(m));
    }
    if extra.is_empty() {
        Cow::Owned(lname)
    } else {
        Cow::Owned(format!("{lname} ({})", extra.join(", ")))
    }
}

/// XKB layout (optionally `layout(variant)`) -> what the picker shows.
pub fn keymap_name(keymap: &str) -> Cow<'static, str> {
    let t = tables();
    if let Some(n) = t.keymaps.get(keymap) {
        return Cow::Borrowed(n);
    }
    let (layout, variant) = split_keymap(keymap);
    let base = t
        .keymaps
        .get(layout.as_str())
        .map_or_else(|| layout.to_uppercase(), |s| s.to_string());
    if variant.is_empty() {
        Cow::Owned(base)
    } else {
        Cow::Owned(format!("{base} ({variant})"))
    }
}

/// Splits `layout(variant)` into XKB layout and variant.
pub fn split_keymap(keymap: &str) -> (String, String) {
    match keymap.split_once('(') {
        Some((layout, rest)) => (layout.to_string(), rest.trim_end_matches(')').to_string()),
        None => (keymap.to_string(), String::new()),
    }
}

/// Canonical IANA zones (tzdata zone1970.tab).
/// "Europe/Rome" -> ("Rome", "Europe"); "America/Argentina/Buenos_Aires" ->
/// ("Buenos Aires", "America, Argentina").
pub fn split_tz(tz: &str) -> (String, String) {
    match tz.rsplit_once('/') {
        Some((region, city)) => (
            city.replace('_', " "),
            region.replace(['_'], " ").replace('/', ", "),
        ),
        None => (tz.replace('_', " "), String::new()),
    }
}

/// Canonical IANA zones (tzdata zone1970.tab).
pub const TIMEZONES: &[&str] = &[
    "Africa/Abidjan",
    "Africa/Accra",
    "Africa/Addis_Ababa",
    "Africa/Algiers",
    "Africa/Asmara",
    "Africa/Bamako",
    "Africa/Bangui",
    "Africa/Banjul",
    "Africa/Bissau",
    "Africa/Blantyre",
    "Africa/Brazzaville",
    "Africa/Bujumbura",
    "Africa/Cairo",
    "Africa/Casablanca",
    "Africa/Ceuta",
    "Africa/Conakry",
    "Africa/Dakar",
    "Africa/Dar_es_Salaam",
    "Africa/Djibouti",
    "Africa/Douala",
    "Africa/El_Aaiun",
    "Africa/Freetown",
    "Africa/Gaborone",
    "Africa/Harare",
    "Africa/Johannesburg",
    "Africa/Juba",
    "Africa/Kampala",
    "Africa/Khartoum",
    "Africa/Kigali",
    "Africa/Kinshasa",
    "Africa/Lagos",
    "Africa/Libreville",
    "Africa/Lome",
    "Africa/Luanda",
    "Africa/Lubumbashi",
    "Africa/Lusaka",
    "Africa/Malabo",
    "Africa/Maputo",
    "Africa/Maseru",
    "Africa/Mbabane",
    "Africa/Mogadishu",
    "Africa/Monrovia",
    "Africa/Nairobi",
    "Africa/Ndjamena",
    "Africa/Niamey",
    "Africa/Nouakchott",
    "Africa/Ouagadougou",
    "Africa/Porto-Novo",
    "Africa/Sao_Tome",
    "Africa/Tripoli",
    "Africa/Tunis",
    "Africa/Windhoek",
    "America/Adak",
    "America/Anchorage",
    "America/Anguilla",
    "America/Antigua",
    "America/Araguaina",
    "America/Argentina/Buenos_Aires",
    "America/Argentina/Catamarca",
    "America/Argentina/Cordoba",
    "America/Argentina/Jujuy",
    "America/Argentina/La_Rioja",
    "America/Argentina/Mendoza",
    "America/Argentina/Rio_Gallegos",
    "America/Argentina/Salta",
    "America/Argentina/San_Juan",
    "America/Argentina/San_Luis",
    "America/Argentina/Tucuman",
    "America/Argentina/Ushuaia",
    "America/Aruba",
    "America/Asuncion",
    "America/Atikokan",
    "America/Bahia",
    "America/Bahia_Banderas",
    "America/Barbados",
    "America/Belem",
    "America/Belize",
    "America/Blanc-Sablon",
    "America/Boa_Vista",
    "America/Bogota",
    "America/Boise",
    "America/Cambridge_Bay",
    "America/Campo_Grande",
    "America/Cancun",
    "America/Caracas",
    "America/Cayenne",
    "America/Cayman",
    "America/Chicago",
    "America/Chihuahua",
    "America/Ciudad_Juarez",
    "America/Costa_Rica",
    "America/Coyhaique",
    "America/Creston",
    "America/Cuiaba",
    "America/Curacao",
    "America/Danmarkshavn",
    "America/Dawson",
    "America/Dawson_Creek",
    "America/Denver",
    "America/Detroit",
    "America/Dominica",
    "America/Edmonton",
    "America/Eirunepe",
    "America/El_Salvador",
    "America/Fort_Nelson",
    "America/Fortaleza",
    "America/Glace_Bay",
    "America/Goose_Bay",
    "America/Grand_Turk",
    "America/Grenada",
    "America/Guadeloupe",
    "America/Guatemala",
    "America/Guayaquil",
    "America/Guyana",
    "America/Halifax",
    "America/Havana",
    "America/Hermosillo",
    "America/Indiana/Indianapolis",
    "America/Indiana/Knox",
    "America/Indiana/Marengo",
    "America/Indiana/Petersburg",
    "America/Indiana/Tell_City",
    "America/Indiana/Vevay",
    "America/Indiana/Vincennes",
    "America/Indiana/Winamac",
    "America/Inuvik",
    "America/Iqaluit",
    "America/Jamaica",
    "America/Juneau",
    "America/Kentucky/Louisville",
    "America/Kentucky/Monticello",
    "America/Kralendijk",
    "America/La_Paz",
    "America/Lima",
    "America/Los_Angeles",
    "America/Lower_Princes",
    "America/Maceio",
    "America/Managua",
    "America/Manaus",
    "America/Marigot",
    "America/Martinique",
    "America/Matamoros",
    "America/Mazatlan",
    "America/Menominee",
    "America/Merida",
    "America/Metlakatla",
    "America/Mexico_City",
    "America/Miquelon",
    "America/Moncton",
    "America/Monterrey",
    "America/Montevideo",
    "America/Montserrat",
    "America/Nassau",
    "America/New_York",
    "America/Nome",
    "America/Noronha",
    "America/North_Dakota/Beulah",
    "America/North_Dakota/Center",
    "America/North_Dakota/New_Salem",
    "America/Nuuk",
    "America/Ojinaga",
    "America/Panama",
    "America/Paramaribo",
    "America/Phoenix",
    "America/Port-au-Prince",
    "America/Port_of_Spain",
    "America/Porto_Velho",
    "America/Puerto_Rico",
    "America/Punta_Arenas",
    "America/Rankin_Inlet",
    "America/Recife",
    "America/Regina",
    "America/Resolute",
    "America/Rio_Branco",
    "America/Santarem",
    "America/Santiago",
    "America/Santo_Domingo",
    "America/Sao_Paulo",
    "America/Scoresbysund",
    "America/Sitka",
    "America/St_Barthelemy",
    "America/St_Johns",
    "America/St_Kitts",
    "America/St_Lucia",
    "America/St_Thomas",
    "America/St_Vincent",
    "America/Swift_Current",
    "America/Tegucigalpa",
    "America/Thule",
    "America/Tijuana",
    "America/Toronto",
    "America/Tortola",
    "America/Vancouver",
    "America/Whitehorse",
    "America/Winnipeg",
    "America/Yakutat",
    "Antarctica/Casey",
    "Antarctica/Davis",
    "Antarctica/DumontDUrville",
    "Antarctica/Macquarie",
    "Antarctica/Mawson",
    "Antarctica/McMurdo",
    "Antarctica/Palmer",
    "Antarctica/Rothera",
    "Antarctica/Syowa",
    "Antarctica/Troll",
    "Antarctica/Vostok",
    "Arctic/Longyearbyen",
    "Asia/Aden",
    "Asia/Almaty",
    "Asia/Amman",
    "Asia/Anadyr",
    "Asia/Aqtau",
    "Asia/Aqtobe",
    "Asia/Ashgabat",
    "Asia/Atyrau",
    "Asia/Baghdad",
    "Asia/Bahrain",
    "Asia/Baku",
    "Asia/Bangkok",
    "Asia/Barnaul",
    "Asia/Beirut",
    "Asia/Bishkek",
    "Asia/Brunei",
    "Asia/Chita",
    "Asia/Colombo",
    "Asia/Damascus",
    "Asia/Dhaka",
    "Asia/Dili",
    "Asia/Dubai",
    "Asia/Dushanbe",
    "Asia/Famagusta",
    "Asia/Gaza",
    "Asia/Hebron",
    "Asia/Ho_Chi_Minh",
    "Asia/Hong_Kong",
    "Asia/Hovd",
    "Asia/Irkutsk",
    "Asia/Jakarta",
    "Asia/Jayapura",
    "Asia/Jerusalem",
    "Asia/Kabul",
    "Asia/Kamchatka",
    "Asia/Karachi",
    "Asia/Kathmandu",
    "Asia/Khandyga",
    "Asia/Kolkata",
    "Asia/Krasnoyarsk",
    "Asia/Kuala_Lumpur",
    "Asia/Kuching",
    "Asia/Kuwait",
    "Asia/Macau",
    "Asia/Magadan",
    "Asia/Makassar",
    "Asia/Manila",
    "Asia/Muscat",
    "Asia/Nicosia",
    "Asia/Novokuznetsk",
    "Asia/Novosibirsk",
    "Asia/Omsk",
    "Asia/Oral",
    "Asia/Phnom_Penh",
    "Asia/Pontianak",
    "Asia/Pyongyang",
    "Asia/Qatar",
    "Asia/Qostanay",
    "Asia/Qyzylorda",
    "Asia/Riyadh",
    "Asia/Sakhalin",
    "Asia/Samarkand",
    "Asia/Seoul",
    "Asia/Shanghai",
    "Asia/Singapore",
    "Asia/Srednekolymsk",
    "Asia/Taipei",
    "Asia/Tashkent",
    "Asia/Tbilisi",
    "Asia/Tehran",
    "Asia/Thimphu",
    "Asia/Tokyo",
    "Asia/Tomsk",
    "Asia/Ulaanbaatar",
    "Asia/Urumqi",
    "Asia/Ust-Nera",
    "Asia/Vientiane",
    "Asia/Vladivostok",
    "Asia/Yakutsk",
    "Asia/Yangon",
    "Asia/Yekaterinburg",
    "Asia/Yerevan",
    "Atlantic/Azores",
    "Atlantic/Bermuda",
    "Atlantic/Canary",
    "Atlantic/Cape_Verde",
    "Atlantic/Faroe",
    "Atlantic/Madeira",
    "Atlantic/Reykjavik",
    "Atlantic/South_Georgia",
    "Atlantic/St_Helena",
    "Atlantic/Stanley",
    "Australia/Adelaide",
    "Australia/Brisbane",
    "Australia/Broken_Hill",
    "Australia/Darwin",
    "Australia/Eucla",
    "Australia/Hobart",
    "Australia/Lindeman",
    "Australia/Lord_Howe",
    "Australia/Melbourne",
    "Australia/Perth",
    "Australia/Sydney",
    "Europe/Amsterdam",
    "Europe/Andorra",
    "Europe/Astrakhan",
    "Europe/Athens",
    "Europe/Belgrade",
    "Europe/Berlin",
    "Europe/Bratislava",
    "Europe/Brussels",
    "Europe/Bucharest",
    "Europe/Budapest",
    "Europe/Busingen",
    "Europe/Chisinau",
    "Europe/Copenhagen",
    "Europe/Dublin",
    "Europe/Gibraltar",
    "Europe/Guernsey",
    "Europe/Helsinki",
    "Europe/Isle_of_Man",
    "Europe/Istanbul",
    "Europe/Jersey",
    "Europe/Kaliningrad",
    "Europe/Kirov",
    "Europe/Kyiv",
    "Europe/Lisbon",
    "Europe/Ljubljana",
    "Europe/London",
    "Europe/Luxembourg",
    "Europe/Madrid",
    "Europe/Malta",
    "Europe/Mariehamn",
    "Europe/Minsk",
    "Europe/Monaco",
    "Europe/Moscow",
    "Europe/Oslo",
    "Europe/Paris",
    "Europe/Podgorica",
    "Europe/Prague",
    "Europe/Riga",
    "Europe/Rome",
    "Europe/Samara",
    "Europe/San_Marino",
    "Europe/Sarajevo",
    "Europe/Saratov",
    "Europe/Simferopol",
    "Europe/Skopje",
    "Europe/Sofia",
    "Europe/Stockholm",
    "Europe/Tallinn",
    "Europe/Tirane",
    "Europe/Ulyanovsk",
    "Europe/Vaduz",
    "Europe/Vatican",
    "Europe/Vienna",
    "Europe/Vilnius",
    "Europe/Volgograd",
    "Europe/Warsaw",
    "Europe/Zagreb",
    "Europe/Zurich",
    "Indian/Antananarivo",
    "Indian/Chagos",
    "Indian/Christmas",
    "Indian/Cocos",
    "Indian/Comoro",
    "Indian/Kerguelen",
    "Indian/Mahe",
    "Indian/Maldives",
    "Indian/Mauritius",
    "Indian/Mayotte",
    "Indian/Reunion",
    "Pacific/Apia",
    "Pacific/Auckland",
    "Pacific/Bougainville",
    "Pacific/Chatham",
    "Pacific/Chuuk",
    "Pacific/Easter",
    "Pacific/Efate",
    "Pacific/Fakaofo",
    "Pacific/Fiji",
    "Pacific/Funafuti",
    "Pacific/Galapagos",
    "Pacific/Gambier",
    "Pacific/Guadalcanal",
    "Pacific/Guam",
    "Pacific/Honolulu",
    "Pacific/Kanton",
    "Pacific/Kiritimati",
    "Pacific/Kosrae",
    "Pacific/Kwajalein",
    "Pacific/Majuro",
    "Pacific/Marquesas",
    "Pacific/Midway",
    "Pacific/Nauru",
    "Pacific/Niue",
    "Pacific/Norfolk",
    "Pacific/Noumea",
    "Pacific/Pago_Pago",
    "Pacific/Palau",
    "Pacific/Pitcairn",
    "Pacific/Pohnpei",
    "Pacific/Port_Moresby",
    "Pacific/Rarotonga",
    "Pacific/Saipan",
    "Pacific/Tahiti",
    "Pacific/Tarawa",
    "Pacific/Tongatapu",
    "Pacific/Wake",
    "Pacific/Wallis",
    "UTC",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_gets_a_name() {
        assert_eq!(locale_name("en_US.UTF-8"), "English (United States)");
        assert_eq!(locale_name("en_SE.UTF-8"), "English (Sweden)");
        assert_eq!(locale_name("sr_RS@latin"), "Serbian (Serbia, Latin)");
        assert_eq!(locale_name("pl_PL.utf8"), "Polish (Poland)");
        // Not in any table: built from the parts.
        assert_eq!(locale_name("pl_SE.UTF-8"), "Polish (Sweden)");
        assert_eq!(locale_name("xx_YY.UTF-8"), "XX (YY)");
        for (code, _) in tables().locales.iter() {
            assert!(!locale_name(code).is_empty());
        }
    }

    #[test]
    fn every_keymap_gets_a_name() {
        assert_eq!(keymap_name("us"), "English (US)");
        assert_eq!(keymap_name("pl"), "Polish");
        assert_eq!(
            keymap_name("us(intl)"),
            "English (US, intl., with dead keys)"
        );
        assert_eq!(keymap_name("pl(nope)"), "Polish (nope)");
        assert_eq!(keymap_name("zz"), "ZZ");
    }

    #[test]
    fn timezones_split() {
        assert_eq!(split_tz("Europe/Rome"), ("Rome".into(), "Europe".into()));
        assert_eq!(
            split_tz("America/Argentina/Buenos_Aires"),
            ("Buenos Aires".into(), "America, Argentina".into())
        );
        assert_eq!(split_tz("UTC"), ("UTC".into(), String::new()));
    }
}
