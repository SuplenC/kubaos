use std::borrow::Cow;

use iced::advanced::graphics::text::font_system;
use kubaos_ui::Mode;
use kubaos_ui::Scale;

use super::*;

fn load_fonts() {
    let mut system = font_system().write().expect("font system lock poisoned");
    for font in kubaos_ui::text::FONTS {
        system.load_font(Cow::Borrowed(font));
    }
}

fn whole(x: f32) -> bool {
    (x - x.round()).abs() < 1e-3
}

#[test]
fn the_error_function_matches_its_table() {
    for (x, want) in [
        (0.0, 0.0),
        (0.5, 0.520_499_9),
        (1.0, 0.842_700_8),
        (2.0, 0.995_322_3),
        (-1.0, -0.842_700_8),
    ] {
        assert!((erf(x) - want).abs() < 1e-6, "erf({x}) = {}", erf(x));
    }
}

#[test]
fn a_glow_is_brightest_on_its_tube_and_gone_three_blurs_out() {
    let (half, sigma) = (12.0, 10.0);

    assert!(band(0.0, half, sigma) > band(half, half, sigma));
    assert!(band(half + 3.0 * sigma, half, sigma) < 0.01);
}

#[test]
fn the_landed_hanko_sits_on_whole_device_pixels_at_every_scale() {
    load_fonts();
    for scale in Scale::SUPPORTED {
        let look = Look::new(Mode::Dark, scale);

        let art = art(&look, true).expect("baked");

        let b = art.hanko.bounds;
        let s = scale.factor();
        for edge in [b.x, b.y, b.width, b.height] {
            assert!(whole(edge * s), "{edge} at {s}");
        }
    }
}

#[test]
fn the_hanko_holds_its_box_with_room_for_its_rough_edge() {
    load_fonts();
    let art = art(&Look::new(Mode::Dark, Scale::X1), true).expect("baked");
    let rest = hanko_box();

    assert!(art.stamp.bounds.contains(rest.center()));
    assert!(
        art.stamp.bounds.width >= rest.width && art.stamp.bounds.width <= rest.width + 4.0 * f32::from(INK_ROOM.px())
    );
}

#[test]
fn each_stroke_has_a_glow() {
    load_fonts();
    let art = art(&Look::new(Mode::Light, Scale::X1_5), true).expect("baked");

    assert_eq!(art.glows.len(), mark::COUNT);
}
