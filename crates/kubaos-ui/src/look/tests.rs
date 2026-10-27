use super::concentric;
use super::nested;
use super::radius;
use super::ring_radius;
use crate::tokens::Radius;
use crate::tokens::Space;

#[test]
fn a_shape_nested_four_in_takes_four_off_its_parents_radius() {
    assert_eq!(concentric(Radius::R16, Space::S4), Some(Radius::R12));
    assert_eq!(concentric(Radius::R24, Space::S8), Some(Radius::R16));
}

#[test]
fn nesting_deeper_than_the_radius_has_no_concentric_corner() {
    assert_eq!(concentric(Radius::R12, Space::S16), None);
    assert_eq!(concentric(Radius::R4, Space::S4), None);
}

#[test]
fn a_pill_in_a_pill_is_a_pill() {
    assert_eq!(concentric(Radius::Full, Space::S8), Some(Radius::Full));
}

#[test]
fn the_focus_ring_is_concentric_with_its_control() {
    assert!((ring_radius(Radius::R12) - 16.0).abs() < f32::EPSILON);
    assert!((ring_radius(Radius::R16) - 20.0).abs() < f32::EPSILON);
}

#[test]
fn a_pill_is_rounder_than_any_shape() {
    assert!(radius(Radius::Full) > 1000.0);
    assert!(ring_radius(Radius::Full) > 1000.0);
}

#[test]
#[should_panic(expected = "off the radius scale")]
fn a_nesting_off_the_radius_scale_is_refused() {
    let _ = nested(Radius::R4, Space::S4);
}

#[test]
fn a_nesting_on_the_scale_gives_the_concentric_radius() {
    const INNER: Radius = nested(Radius::R16, Space::S4);

    assert_eq!(INNER, Radius::R12);
}
