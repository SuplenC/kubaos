use super::Tick;

#[test]
fn a_set_reads_full_partial_or_empty_from_its_picks() {
    assert_eq!(Tick::of(0, 5), Tick::Empty);
    assert_eq!(Tick::of(3, 5), Tick::Partial);
    assert_eq!(Tick::of(5, 5), Tick::Full);
    assert_eq!(Tick::of(0, 0), Tick::Empty);
}
