use super::glyphs;

#[test]
fn the_sheet_has_forty_glyphs_in_every_set() {
    let glyphs = glyphs();

    assert_eq!(glyphs.len(), 40);
    assert!(glyphs.iter().all(|g| !g.chars.contains(&'\u{fffd}')));
}
