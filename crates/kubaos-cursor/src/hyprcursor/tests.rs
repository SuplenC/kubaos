use super::*;
use crate::fixture;
use crate::source;
use crate::zip;

fn shapes() -> Vec<Shape> {
    source::load(fixture::bibata().path()).unwrap()
}

#[test]
fn the_manifest_names_the_theme_and_its_cursors_directory() {
    let manifest = manifest("KubaOS-Bibata");

    assert!(manifest.lines().any(|l| l == "name = KubaOS-Bibata"));
    assert!(manifest.lines().any(|l| l == "cursors_directory = hyprcursors"));
}

#[test]
fn a_still_cursors_meta_has_its_hotspot_aliases_and_one_svg() {
    let meta = meta(&shapes()[1]);

    assert_eq!(
        meta,
        "resize_algorithm = bilinear\n\
         hotspot_x = 0.2148\n\
         hotspot_y = 0.0664\n\
         define_override = arrow\n\
         define_override = default\n\
         define_size = 0, left_ptr.svg\n"
    );
}

#[test]
fn an_animated_cursors_meta_lists_every_frame_with_its_delay() {
    let meta = meta(&shapes()[2]);

    assert!(meta.contains("define_size = 0, wait-01.svg, 40\ndefine_size = 0, wait-02.svg, 40\n"));
    assert!(meta.contains("hotspot_x = 0.5000\n"));
}

#[test]
fn the_archive_holds_the_meta_and_the_recoloured_svgs() {
    let wait = &shapes()[2];

    let files = zip::tests::read(&archive(wait, &fixture::DARK));

    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["meta.hl", "wait-01.svg", "wait-02.svg"]);
    let svg = String::from_utf8(files[1].1.clone()).unwrap();
    assert!(svg.contains(r##"fill="#1E0F0E" stroke="#FADBD9""##), "{svg}");
    assert!(svg.contains(r##"fill="#FFB3AE" fill-opacity="0.8""##), "{svg}");
    assert!(!svg.contains("#FF0000") && !svg.contains("#0000FF"));
    assert!(fixture::RAINBOW.iter().all(|colour| !svg.contains(colour)), "{svg}");
}
