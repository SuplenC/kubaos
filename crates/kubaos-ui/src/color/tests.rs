use iced::Color;

use super::Contrast;
use super::colors;
use super::contrast_ratio;
use super::derived;
use super::over;
use crate::tokens;
use crate::tokens::Mode;

#[test]
fn black_on_white_is_the_largest_contrast() {
    assert!((contrast_ratio(Color::BLACK, Color::WHITE) - 21.0).abs() < 0.01);
}

#[test]
fn a_colour_on_itself_has_no_contrast() {
    let amber = Color::from_rgb8(0xff, 0xb7, 0x79);

    assert!((contrast_ratio(amber, amber) - 1.0).abs() < 1e-6);
}

#[test]
fn contrast_is_the_same_both_ways_round() {
    let (a, b) = (Color::from_rgb8(0x8b, 0x4c, 0x00), Color::from_rgb8(0xff, 0xf8, 0xf5));

    assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-6);
}

#[test]
fn a_known_pair_has_its_published_ratio() {
    // WebAIM's checker gives #777777 on white 4.48:1, just under AA.
    let grey = Color::from_rgb8(0x77, 0x77, 0x77);

    assert!((contrast_ratio(grey, Color::WHITE) - 4.48).abs() < 0.01);
    assert!(!Contrast::Text.holds(grey, Color::WHITE));
    assert!(Contrast::LargeText.holds(grey, Color::WHITE));
}

#[test]
fn a_translucent_colour_is_judged_as_drawn_over_its_background() {
    let half_black = Color { a: 0.5, ..Color::BLACK };

    let seen = over(half_black, Color::WHITE);

    assert!((seen.r - 0.5).abs() < 1e-6 && (seen.a - 1.0).abs() < 1e-6);
}

#[test]
fn every_contrast_pair_the_tokens_list_meets_its_minimum_in_both_modes() {
    for mode in [Mode::Dark, Mode::Light] {
        let c = colors(mode);
        for pair in crate::tokens::CONTRAST {
            let ratio = super::pair_ratio(&pair, &c);
            let need = Contrast::from(pair.need);

            assert!(
                ratio >= need.minimum(),
                "{mode:?}: {} on {} ({}) is {ratio:.2}:1",
                pair.fg.name(),
                pair.bg.name(),
                pair.purpose
            );
        }
    }
}

#[test]
fn the_fallback_palettes_meet_the_contrast_minimums() {
    for mode in [Mode::Dark, Mode::Light] {
        let c = colors(mode);
        let pairs = [
            ("on_surface on surface", c.on_surface, c.surface, Contrast::Text),
            (
                "on_surface_variant on surface_container",
                c.on_surface_variant,
                c.surface_container,
                Contrast::Text,
            ),
            (
                "on_surface_variant on surface_container_highest",
                c.on_surface_variant,
                c.surface_container_highest,
                Contrast::Text,
            ),
            ("primary on surface", c.primary, c.surface, Contrast::Text),
            ("on_primary on primary", c.on_primary, c.primary, Contrast::Text),
            (
                "on_secondary_container on secondary_container",
                c.on_secondary_container,
                c.secondary_container,
                Contrast::Text,
            ),
            ("error on surface", c.error, c.surface, Contrast::Text),
            ("on_error on error", c.on_error, c.error, Contrast::Text),
            ("cold on surface", c.cold, c.surface, Contrast::Text),
            ("warning on surface", c.warning, c.surface, Contrast::Text),
            (
                "sharing on surface_container_highest",
                c.sharing,
                c.surface_container_highest,
                Contrast::Text,
            ),
            (
                "microphone on surface_container_highest",
                c.microphone,
                c.surface_container_highest,
                Contrast::Text,
            ),
            (
                "recording on surface_container_highest",
                c.recording,
                c.surface_container_highest,
                Contrast::Text,
            ),
            (
                "success on surface_container_highest",
                c.success,
                c.surface_container_highest,
                Contrast::Text,
            ),
            ("surface on on_surface (toast)", c.surface, c.on_surface, Contrast::Text),
            ("outline on surface", c.outline, c.surface, Contrast::Graphic),
            (
                "outline on surface_container_highest",
                c.outline,
                c.surface_container_highest,
                Contrast::Graphic,
            ),
        ];
        for (name, fg, bg, need) in pairs {
            let ratio = contrast_ratio(over(fg, bg), bg);
            assert!(need.holds(fg, bg), "{mode:?}: {name} is {ratio:.2}:1");
        }
    }
}

#[test]
fn a_core_follows_the_primary_it_is_drawn_with() {
    let fallback = colors(Mode::Dark);
    let live = super::Colors {
        primary: Color::from_rgb8(0x3a, 0x7b, 0xd5),
        ..fallback
    };

    assert_eq!(
        derived(&fallback, tokens::PRIMARY_CORE).into_rgba8(),
        [0xff, 0xdb, 0xbc, 0xff]
    );
    assert_eq!(derived(&live, tokens::PRIMARY_CORE).into_rgba8(), [0x9d, 0xbd, 0xea, 0xff]);
}
