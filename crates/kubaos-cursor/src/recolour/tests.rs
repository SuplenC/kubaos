use super::*;
use crate::fixture;

#[test]
fn the_three_placeholders_take_the_styles_colours() {
    let svg = r##"<path fill="#00FF00" stroke="#0000FF"/><circle fill="#FF0000"/>"##;

    let out = recolour(svg, &fixture::DARK);

    assert_eq!(out, r##"<path fill="#1E0F0E" stroke="#FADBD9"/><circle fill="#FFB3AE"/>"##);
}

#[test]
fn a_placeholder_in_lowercase_is_replaced_too() {
    assert_eq!(recolour(r##"fill="#00ff00""##, &fixture::DARK), r##"fill="#1E0F0E""##);
}

#[test]
fn every_other_colour_and_a_short_or_trailing_hash_stay() {
    let svg = r##"<path fill="#F05024" filter="url(#filter0)"/><use href="#0000F"/>#"##;

    assert_eq!(recolour(svg, &fixture::DARK), svg);
}
