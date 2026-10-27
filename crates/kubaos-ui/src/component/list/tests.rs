use super::ROW_RADIUS;
use super::WELL_INSET;
use super::WELL_RADIUS;
use crate::look::concentric;
use crate::tokens::Radius;

#[test]
fn rows_are_concentric_with_their_well() {
    assert_eq!(concentric(WELL_RADIUS.corner(), WELL_INSET), Some(ROW_RADIUS));
}

#[test]
fn a_rows_corner_is_the_control_radius() {
    assert_eq!(ROW_RADIUS, Radius::Control.corner());
}
