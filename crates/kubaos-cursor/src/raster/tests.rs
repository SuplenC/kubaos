use super::*;
use crate::fixture;
use crate::recolour::recolour;

#[test]
fn the_fill_lands_in_the_middle_and_the_corners_stay_clear() {
    let drawing = Drawing::parse(&recolour(fixture::ARROW, &fixture::DARK)).unwrap();

    let pixels = drawing.render(32);

    assert_eq!(pixels.len(), 32 * 32);
    assert_eq!(pixels[16 * 32 + 16], 0xff1e_0f0e);
    assert_eq!(pixels[0], 0);
}

#[test]
fn the_outline_takes_the_outline_colour() {
    let drawing = Drawing::parse(&recolour(fixture::ARROW, &fixture::DARK)).unwrap();

    let pixels = drawing.render(256);

    assert_eq!(pixels[128 * 256 + 64], 0xfffa_dbd9);
}

#[test]
fn text_that_isnt_an_svg_is_an_error() {
    assert!(Drawing::parse("not an svg").is_err());
}
