use std::collections::HashSet;
use std::env;
use std::fs;

use super::*;

/// The files `system/50-hyprland.sh` installs, at the root of the repository.
const KEYD_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../system/files/keyd/kubaos.conf");
const XKB_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../system/files/xkb/symbols/kubaos");

#[test]
fn the_five_marks_are_on_e_grave_u_i_and_n_as_on_a_mac() {
    let chords: Vec<&str> = ACCENTS.iter().map(|accent| accent.chord).collect();

    assert_eq!(chords, ["e", "`", "u", "i", "n"]);
}

#[test]
fn no_two_marks_share_a_spare_key() {
    let dead: HashSet<&str> = ACCENTS.iter().map(|accent| accent.dead.keyd).collect();
    let spacing: HashSet<(&str, bool)> = ACCENTS
        .iter()
        .map(|accent| (accent.spacing.keyd, accent.spacing.shifted))
        .collect();

    assert_eq!(dead.len(), ACCENTS.len());
    assert_eq!(spacing.len(), ACCENTS.len());
    assert!(spacing.iter().all(|(key, _)| !dead.contains(key)));
}

#[test]
fn left_alt_is_alt_but_for_the_five_chords() {
    let config = keyd_config();

    assert!(config.contains("[main]\nleftalt = layer(macalt)\n"), "{config}");
    assert!(
        config.contains(
            "[macalt:A]\ne = oneshot(mac-acute)\n` = oneshot(mac-grave)\n\
             u = oneshot(mac-diaeresis)\ni = oneshot(mac-circumflex)\nn = oneshot(mac-tilde)\n"
        ),
        "{config}"
    );
    assert!(!config.contains("rightalt"), "{config}");
}

#[test]
fn a_letter_after_a_chord_is_the_dead_key_then_the_same_key() {
    let config = keyd_config();

    assert!(
        config.contains("[mac-acute]\nspace = setup\na = macro(move a)\n"),
        "{config}"
    );
    assert!(config.contains("z = macro(move z)\n"), "{config}");
    assert!(config.contains("[mac-grave]\nspace = S-setup\n"), "{config}");
    assert!(
        config.contains("[mac-tilde]\nspace = closecd\na = macro(iso a)\n"),
        "{config}"
    );
    let letters = config.lines().filter(|line| line.contains(" = macro(")).count();
    assert_eq!(letters, 26 * ACCENTS.len());
}

#[test]
fn the_dead_and_spacing_keys_are_xkb_symbols_on_the_spare_keycodes() {
    let symbols = xkb_symbols();

    assert!(symbols.contains("xkb_symbols \"mac-accents\" {"), "{symbols}");
    assert!(symbols.contains("key <I183> { [ dead_acute ] };"), "{symbols}");
    assert!(symbols.contains("key <I178> { [ dead_tilde ] };"), "{symbols}");
    assert!(symbols.contains("key <I149> { [ acute, grave ] };"), "{symbols}");
    assert!(symbols.contains("key <I154> { [ diaeresis, U02C6 ] };"), "{symbols}");
    assert!(symbols.contains("key <I168> { [ U02DC ] };"), "{symbols}");
}

#[test]
fn the_installed_keyd_config_is_the_generated_one() {
    same_as_generated(KEYD_FILE, &keyd_config());
}

#[test]
fn the_installed_xkb_symbols_are_the_generated_ones() {
    same_as_generated(XKB_FILE, &xkb_symbols());
}

/// Compares the file at `path` with `generated`. With `KUBAOS_WRITE_ACCENTS` set it writes the
/// file first, after a change to the table.
fn same_as_generated(path: &str, generated: &str) {
    if env::var_os("KUBAOS_WRITE_ACCENTS").is_some() {
        fs::write(path, generated).expect("write the generated file");
    }

    let installed = fs::read_to_string(path).expect("read the installed file");

    assert!(
        installed == generated,
        "{path} isn't the table's: KUBAOS_WRITE_ACCENTS=1 cargo test -p kubaos-core accents writes it"
    );
}
