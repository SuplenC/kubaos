use super::GRID;
use super::Icon;
use super::WRITTEN_STROKE;
use super::device_stroke;
use super::drawing;
use crate::scale::Scale;
use crate::tokens::IconSize;

#[test]
fn every_drawing_has_the_stroke_the_renderer_replaces() {
    for glyph in Icon::ALL {
        assert_eq!(glyph.svg().matches(WRITTEN_STROKE).count(), 1, "{}", glyph.name());
    }
}

#[test]
fn every_drawing_is_on_the_24_unit_grid() {
    for glyph in Icon::ALL {
        assert!(glyph.svg().contains("viewBox=\"0 0 24 24\""), "{}", glyph.name());
    }
}

#[test]
fn the_stroke_is_whole_device_pixels_at_every_size_and_scale() {
    for scale in Scale::SUPPORTED {
        for size in IconSize::ALL {
            let device = device_stroke(size, scale);
            assert!(
                (device - device.round()).abs() < 1e-4 && device >= 1.0,
                "{size:?} at {scale:?}"
            );
        }
    }
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "The strokes are whole device pixels by design, so they are exact: any difference is the bug."
)]
fn the_stroke_is_tablers_at_24_and_rounds_at_16_and_20() {
    let at = |size| Scale::SUPPORTED.map(|scale| device_stroke(size, scale));

    assert_eq!(at(IconSize::S24), [2.0, 3.0, 3.0, 4.0]);
    assert_eq!(at(IconSize::S20), [2.0, 2.0, 3.0, 3.0]);
    assert_eq!(at(IconSize::S16), [1.0, 2.0, 2.0, 3.0]);
}

#[test]
fn a_drawing_takes_the_stroke_it_is_given() {
    let svg = drawing(Icon::Check, 1.5);

    assert!(svg.contains("stroke-width=\"1.5\""));
    assert!(!svg.contains(WRITTEN_STROKE));
}

#[test]
fn our_drawings_are_the_four_tabler_lacks() {
    let ours: Vec<&str> = Icon::ALL
        .into_iter()
        .filter(|glyph| glyph.tabler().is_none())
        .map(Icon::name)
        .collect();

    assert_eq!(ours, ["bluetooth_searching", "hard_drive", "wifi_find", "wifi_lock"]);
    assert!((GRID - 24.0).abs() < f32::EPSILON);
}
