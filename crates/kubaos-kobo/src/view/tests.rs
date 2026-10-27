use kubaos_ui::color;
use kubaos_ui::color::Contrast;
use kubaos_ui::tokens::Mode;
use kubaos_ui::tokens::Rgb;
use kubaos_ui::tokens::palette;

use super::logo_drawing;
use crate::theme;

/// Every `#rrggbb` a drawing strokes or fills with.
fn colours(drawing: &str) -> Vec<Rgb> {
    drawing
        .match_indices('#')
        .filter_map(|(at, _)| drawing.get(at + 1..at + 7))
        .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(|hex| {
            let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex digits");
            Rgb {
                r: channel(0),
                g: channel(2),
                b: channel(4),
            }
        })
        .collect()
}

#[test]
fn the_wordmark_reads_on_the_surface_in_both_modes() {
    for mode in [Mode::Dark, Mode::Light] {
        let surface = color::rgb(palette(mode).surface);

        let drawn = logo_drawing(theme::LOGO_WORDMARK, mode);

        for colour in colours(&drawn) {
            assert!(
                Contrast::Graphic.holds(color::rgb(colour), surface),
                "{mode:?}: {colour:?} on the surface"
            );
        }
    }
}

#[test]
fn the_dark_logo_is_drawn_as_written() {
    let drawn = logo_drawing(theme::LOGO_MARK, Mode::Dark);

    assert_eq!(drawn.as_bytes(), theme::LOGO_MARK);
}
