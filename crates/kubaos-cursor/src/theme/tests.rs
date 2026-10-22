use super::*;
use crate::fixture;
use crate::source;
use crate::xcursor::tests::decode;
use crate::zip;

struct Setup {
    _bibata: tempfile::TempDir,
    icons: tempfile::TempDir,
    shapes: Vec<Shape>,
    theme: Theme,
    stamp: String,
}

fn setup() -> Setup {
    let bibata = fixture::bibata();
    let icons = tempfile::tempdir().unwrap();
    let shapes = source::load(bibata.path()).unwrap();
    let theme = Theme::new(icons.path(), "KubaOS-Bibata");
    let stamp = stamp(Style::Dark, Some(&fixture::DARK), bibata.path());
    Setup { _bibata: bibata, icons, shapes, theme, stamp }
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> =
        fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    names
}

#[test]
fn the_hyprcursor_theme_has_its_manifest_index_and_one_archive_per_cursor() {
    let s = setup();

    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();

    let dir = s.theme.dir();
    assert_eq!(entries(&dir), [".stamp", "hyprcursors", "index.theme", "manifest.hl"]);
    assert_eq!(entries(&dir.join("hyprcursors")), ["hand2.hlc", "left_ptr.hlc", "wait.hlc"]);
    assert!(fs::read_to_string(dir.join("index.theme")).unwrap().contains("\nInherits=Bibata-Modern-Classic\n"));
    assert_eq!(s.theme.stamp().as_deref(), Some(s.stamp.as_str()));
    let files = zip::tests::read(&fs::read(dir.join("hyprcursors/left_ptr.hlc")).unwrap());
    assert!(String::from_utf8_lossy(&files[1].1).contains("#1E0F0E"));
}

#[test]
fn nothing_but_the_theme_is_left_in_the_icons_directory() {
    let s = setup();

    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();
    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();
    s.theme.write_xcursor(&s.shapes, &fixture::DARK, 24, &s.stamp, 2).unwrap();

    assert_eq!(entries(s.icons.path()), ["KubaOS-Bibata"]);
}

#[test]
fn the_xcursor_theme_has_every_size_and_frame_and_links_the_aliases() {
    let s = setup();
    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();

    let outcome = s.theme.write_xcursor(&s.shapes, &fixture::DARK, 24, &s.stamp, 2).unwrap();

    assert_eq!(outcome, XcursorOutcome::Written);
    let cursors = s.theme.dir().join("cursors");
    let arrow = decode(&fs::read(cursors.join("left_ptr")).unwrap());
    assert_eq!(arrow.iter().map(|i| i.nominal).collect::<Vec<_>>(), [24, 36, 48]);
    assert_eq!((arrow[0].xhot, arrow[0].yhot), (5, 2));
    assert_eq!(arrow[0].pixels[12 * 24 + 12], 0xff1e_0f0e);
    let wait = decode(&fs::read(cursors.join("watch")).unwrap());
    assert_eq!(wait.len(), 3 * 2);
    assert!(wait.iter().all(|i| i.delay_ms == 40));
    assert_eq!(fs::read_link(cursors.join("default")).unwrap(), Path::new("left_ptr"));
    assert_eq!(s.theme.xcursor_stamp(), Some(xcursor_stamp(&s.stamp, 24)));
}

#[test]
fn a_render_for_a_theme_that_changed_meanwhile_is_dropped() {
    let s = setup();
    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();

    let outcome = s.theme.write_xcursor(&s.shapes, &fixture::DARK, 24, "an older stamp\n", 2).unwrap();

    assert_eq!(outcome, XcursorOutcome::Superseded);
    assert!(!s.theme.dir().join("cursors").exists());
    assert_eq!(entries(s.icons.path()), ["KubaOS-Bibata"]);
}

#[test]
fn a_new_hyprcursor_theme_keeps_the_old_xcursor_part_until_it_is_rendered_again() {
    let s = setup();
    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();
    s.theme.write_xcursor(&s.shapes, &fixture::DARK, 24, &s.stamp, 1).unwrap();
    let newer = stamp(Style::Accent, Some(&fixture::DARK), Path::new("/elsewhere"));

    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &newer).unwrap();

    assert_eq!(s.theme.stamp(), Some(newer.clone()));
    assert_eq!(s.theme.xcursor_stamp(), Some(xcursor_stamp(&s.stamp, 24)));
    assert_ne!(s.theme.xcursor_stamp(), Some(xcursor_stamp(&newer, 24)));
}

#[test]
fn the_classic_stub_only_inherits_bibata_modern_classic() {
    let s = setup();
    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();
    let classic = stamp(Style::Classic, None, Path::new("/unused"));

    s.theme.write_classic(&classic).unwrap();

    assert_eq!(entries(&s.theme.dir()), [".stamp", "index.theme"]);
    assert_eq!(classic, "kubaos-cursor 1\nstyle classic\n");
}

#[test]
fn the_xcursor_sizes_are_the_size_and_its_hidpi_multiples() {
    assert_eq!(xcursor_sizes(24), [24, 36, 48]);
    assert_eq!(xcursor_sizes(32), [32, 48, 64]);
}

#[test]
fn a_broken_svg_names_its_cursor() {
    let mut s = setup();
    s.shapes[0].frames[0] = "not an svg".to_owned();
    s.theme.write_hyprcursor(&s.shapes, &fixture::DARK, &s.stamp).unwrap();

    let err = s.theme.write_xcursor(&s.shapes, &fixture::DARK, 24, &s.stamp, 2).unwrap_err();

    assert!(matches!(err, ThemeError::Render(name, _) if name == "hand2"));
    assert!(!s.theme.dir().join("cursors").exists());
}
