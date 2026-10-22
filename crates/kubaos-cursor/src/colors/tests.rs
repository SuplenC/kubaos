use super::*;

const SHINJUKU: &str = r##"
[dark]
fill = "#1e0f0e"
outline = "#fadbd9"
watch = "#ffb3ae"

[accent]
fill = "#ffb3ae"
outline = "#68000c"
watch = "#ffb3ae"
"##;

#[test]
fn a_colour_parses_from_rrggbb_in_either_case() {
    assert_eq!("#ffB3ae".parse::<Rgb>().unwrap(), Rgb([0xff, 0xb3, 0xae]));
}

#[test]
fn a_colour_without_the_hash_or_six_digits_is_refused() {
    for bad in ["ffb3ae", "#ffb3a", "#ffb3aeff", "#ggb3ae", ""] {
        assert!(bad.parse::<Rgb>().is_err(), "{bad:?} parsed");
    }
}

#[test]
fn a_colour_prints_as_uppercase_rrggbb() {
    assert_eq!(Rgb([0x1e, 0x0f, 0x0e]).to_string(), "#1E0F0E");
}

#[test]
fn the_palette_file_gives_each_style_its_colours() {
    let palette = Palette::parse(SHINJUKU).unwrap();

    assert_eq!(palette.colors(Style::Dark).unwrap().fill, Rgb([0x1e, 0x0f, 0x0e]));
    assert_eq!(palette.colors(Style::Accent).unwrap().outline, Rgb([0x68, 0x00, 0x0c]));
    assert_eq!(palette.colors(Style::Classic), None);
}

#[test]
fn a_palette_missing_a_colour_is_an_error() {
    let text = SHINJUKU.replace("watch = \"#ffb3ae\"\n\n[accent]", "\n[accent]");

    assert!(matches!(Palette::parse(&text), Err(PaletteError::Parse(_))));
}
