use super::WELL_INSET;
use super::WELL_RADIUS;
use crate::look::concentric;
use crate::tokens::Radius;

#[test]
fn rows_are_concentric_with_their_well() {
    assert_eq!(concentric(WELL_RADIUS, WELL_INSET), Some(Radius::R12));
}
