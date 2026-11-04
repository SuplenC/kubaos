use image::Rgb;

use super::*;

fn picture(colour: [u8; 3]) -> RgbImage {
    RgbImage::from_pixel(40, 20, Rgb(colour))
}

const ALL: Rect = Rect {
    x0: 0.0,
    y0: 0.0,
    x1: 40.0,
    y1: 20.0,
};

#[test]
fn a_grey_picture_gives_a_light_grey_tint() {
    let tint = tint_under(&picture([60, 60, 60]), ALL);

    assert_eq!(&tint[1..3], &tint[3..5]);
    assert_eq!(&tint[3..5], &tint[5..7]);
    assert!((to_lch(hex(&tint)).l - LIGHTNESS).abs() < 0.01, "{tint}");
}

#[test]
fn a_purple_sky_gives_a_light_purple_tint() {
    let tint = tint_under(&picture([110, 70, 190]), ALL);

    let lch = to_lch(hex(&tint));
    let sky = to_lch([110, 70, 190]);
    assert!((lch.l - LIGHTNESS).abs() < 0.01, "{tint}");
    assert!((lch.h - sky.h).abs() < 3.0, "{tint}: hue {} against {}", lch.h, sky.h);
}

#[test]
fn a_vivid_colour_outweighs_grey_haze_around_it() {
    let mut mixed = picture([90, 90, 90]);
    for x in 0..8 {
        for y in 0..20 {
            mixed.put_pixel(x, y, Rgb([240, 60, 40]));
        }
    }

    let tint = tint_under(&mixed, ALL);

    let red = to_lch([240, 60, 40]);
    assert!((to_lch(hex(&tint)).h - red.h).abs() < 3.0, "{tint}");
}

#[test]
fn a_tint_out_of_srgb_is_fitted_by_its_chroma() {
    let hex_out = to_hex(Lch {
        l: LIGHTNESS,
        c: 0.5,
        h: 140.0,
    });

    let fitted = to_lch(hex(&hex_out));
    assert!((fitted.l - LIGHTNESS).abs() < 0.01);
    assert!(fitted.c < 0.5);
}

#[test]
fn the_tint_matches_the_lock_pages_rule() {
    // The Lock page's `tintFor` and `toHex` (`design/mockups/src/lib/lock-tint.ts`) give these for
    // a flat picture of each colour.
    assert_eq!(tint_under(&picture([0x6a, 0x4b, 0xc0]), ALL), "#cac1ff");
    assert_eq!(tint_under(&picture([0x3c, 0x3c, 0x3c]), ALL), "#cacaca");
}

fn hex(colour: &str) -> [u8; 3] {
    let byte = |at: usize| u8::from_str_radix(&colour[at..at + 2], 16).unwrap();
    [byte(1), byte(3), byte(5)]
}
