use super::*;
use crate::fixture;

#[test]
fn the_three_placeholders_take_the_styles_colours() {
    let svg = r##"<path fill="#00FF00" stroke="#0000FF"/><circle fill="#FF0000"/>"##;

    let out = recolour(svg, &fixture::DARK);

    assert_eq!(out, r##"<path fill="#1E0F0E" stroke="#FADBD9"/><circle fill="#1E0F0E"/>"##);
}

#[test]
fn a_placeholder_in_lowercase_is_replaced_too() {
    assert_eq!(recolour(r##"fill="#00ff00""##, &fixture::DARK), r##"fill="#1E0F0E""##);
}

#[test]
fn the_spinner_segments_take_the_styles_spinner_tones_in_order() {
    let svg = r##"<path fill="#F05024"/><path fill="#FCB813"/><path fill="#7eba41"/><path fill="#32A0DA"/>"##;

    let out = recolour(svg, &fixture::DARK);

    assert_eq!(
        out,
        r##"<path fill="#FFB3AE"/><path fill="#FFDAD6"/><path fill="#AB3537"/><path fill="#891D22"/>"##
    );
}

#[test]
fn no_rainbow_colour_is_left_in_a_recoloured_busy_cursor_or_corner() {
    let corners = r##"<path fill="#F1613A"/><path fill="#FDBE2A"/><path fill="#96C865"/><path fill="#4FADDF"/>"##;

    for svg in [recolour(fixture::WATCH, &fixture::DARK), recolour(corners, &fixture::DARK)] {
        for colour in fixture::RAINBOW {
            assert!(!svg.to_uppercase().contains(colour), "{colour} left in {svg}");
        }
    }
}

#[test]
fn every_other_colour_and_a_short_or_trailing_hash_stay() {
    let svg = r##"<path fill="#06B231" filter="url(#filter0)"/><use href="#0000F"/>#"##;

    assert_eq!(recolour(svg, &fixture::DARK), svg);
}
