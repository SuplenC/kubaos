use super::Corner;
use super::concentric;
use super::nested;
use super::radius;
use super::ring_radius;
use crate::tokens::Radius;
use crate::tokens::Space;

fn px(corner: Corner) -> Option<u16> {
    corner.px()
}

#[test]
fn a_shape_filling_a_surface_takes_the_inset_off_its_radius() {
    assert_eq!(concentric(Radius::Surface.corner(), Space::S8).map(px), Some(Some(16)));
    assert_eq!(concentric(Radius::Surface.corner(), Space::S12).map(px), Some(Some(12)));
}

#[test]
fn a_surface_twelve_in_gives_the_control_radius() {
    assert_eq!(nested(Radius::Surface.corner(), Space::S12), Radius::Control.corner());
}

#[test]
fn nesting_as_deep_as_the_radius_has_no_concentric_corner() {
    assert_eq!(concentric(Radius::Control.corner(), Space::S16), None);
    assert_eq!(concentric(Radius::Small.corner(), Space::S4), None);
}

#[test]
fn a_pill_in_a_pill_is_a_pill() {
    assert_eq!(
        concentric(Radius::Full.corner(), Space::S8),
        Some(Radius::Full.corner())
    );
}

#[test]
fn nesting_twice_takes_both_insets() {
    const IN_CARD: Corner = nested(nested(Radius::Surface.corner(), Space::S8), Space::S4);

    assert_eq!(IN_CARD.px(), Some(12));
}

#[test]
fn the_focus_ring_is_concentric_with_its_control() {
    assert!((ring_radius(Radius::Control) - 16.0).abs() < f32::EPSILON);
    assert!((ring_radius(Radius::Surface) - 28.0).abs() < f32::EPSILON);
}

#[test]
fn a_pill_is_rounder_than_any_shape() {
    assert!(radius(Radius::Full) > 1000.0);
    assert!(ring_radius(Radius::Full) > 1000.0);
}

#[test]
#[should_panic(expected = "the corners can't be concentric")]
fn a_nesting_that_reaches_the_outer_radius_is_refused() {
    let _ = nested(Radius::Small.corner(), Space::S4);
}
