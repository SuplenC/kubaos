use super::Scale;
use crate::tokens;
use crate::tokens::IconSize;
use crate::tokens::Radius;
use crate::tokens::Shadow;
use crate::tokens::Size;
use crate::tokens::Space;
use crate::tokens::Type;

/// Every length token, named, in logical pixels.
fn lengths() -> Vec<(String, u16)> {
    let mut all: Vec<(String, u16)> = Vec::new();
    all.extend(Space::ALL.map(|s| (format!("{s:?}"), s.px())));
    all.extend(Radius::ALL.iter().filter_map(|r| Some((format!("{r:?}"), r.px()?))));
    all.extend(IconSize::ALL.map(|s| (format!("icon {s:?}"), s.px())));
    all.extend(
        Size::ALL
            .into_iter()
            .filter(|s| !s.off_grid())
            .map(|s| (format!("{s:?}"), s.px())),
    );
    all.extend(Type::ALL.map(|t| (format!("{t:?} line"), t.line())));
    for shadow in [Shadow::Panel, Shadow::Dialog, Shadow::Glow] {
        all.push((format!("{shadow:?} y"), shadow.y()));
        all.push((format!("{shadow:?} blur"), shadow.blur()));
    }
    all.push(("focus offset".into(), tokens::FOCUS_OFFSET));
    all
}

#[test]
fn every_length_is_whole_device_pixels_at_every_supported_scale() {
    for scale in Scale::SUPPORTED {
        for (name, px) in lengths() {
            assert!(scale.is_whole(f32::from(px)), "{name} = {px} at {}", scale.factor());
        }
    }
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "The scales and strokes are the tokens' exact values: any difference is the bug."
)]
fn the_supported_scales_are_the_tokens() {
    assert_eq!(Scale::SUPPORTED.map(Scale::factor), tokens::SCALES);
}

#[test]
fn a_hairline_is_one_device_pixel_at_every_scale() {
    for scale in Scale::SUPPORTED {
        assert!((scale.to_device(scale.hairline()) - 1.0).abs() < 1e-6, "{scale:?}");
    }
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "The scales and strokes are the tokens' exact values: any difference is the bug."
)]
fn the_thin_stroke_is_two_three_three_and_four_device_pixels() {
    let device = Scale::SUPPORTED.map(|scale| scale.to_device(scale.thin()));

    assert_eq!(device, [2.0, 3.0, 3.0, 4.0]);
}

#[test]
fn a_stroke_is_never_under_one_device_pixel() {
    assert!((Scale::X1.to_device(Scale::X1.stroke(0.2)) - 1.0).abs() < 1e-6);
}

#[test]
fn snapping_moves_to_the_nearest_device_pixel() {
    assert!((Scale::X1_25.snap(9.0) - 8.8).abs() < 1e-6);
    assert!((Scale::X1_5.snap(9.0) - 9.333_333).abs() < 1e-4);
    assert!((Scale::X2.snap(9.2) - 9.0).abs() < 1e-6);
}

#[test]
fn snapping_up_ignores_float_error_under_a_pixel() {
    assert!((Scale::X1_25.snap_up(19.999_99) - 20.0).abs() < 1e-6);
    assert!((Scale::X1_25.snap_up(20.1) - 20.8).abs() < 1e-6);
}

#[test]
fn a_scale_never_drops_below_an_eighth() {
    assert!((Scale::new(0.0).factor() - 0.125).abs() < f32::EPSILON);
}
