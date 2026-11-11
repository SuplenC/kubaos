use std::path::Path;

use super::Part;
use super::Switches;

#[test]
fn each_part_is_on_until_its_flag_file_is_made() {
    let tmp = tempfile::tempdir().unwrap();
    let switches = Switches::new(tmp.path());

    for part in [Part::Bitwarden, Part::Nix, Part::Firmware] {
        assert!(switches.is_on(part), "{part:?}");
    }

    switches.turn_off(Part::Nix).unwrap();
    switches.turn_off(Part::Firmware).unwrap();
    switches.turn_off(Part::Firmware).unwrap();

    assert!(switches.is_on(Part::Bitwarden));
    assert!(!switches.is_on(Part::Nix));
    assert!(!switches.is_on(Part::Firmware));
    assert!(tmp.path().join("kubaos/desktop/update/no-nix").is_file());
    assert!(tmp.path().join("kubaos/desktop/update/no-firmware").is_file());

    switches.turn_on(Part::Firmware).unwrap();
    switches.turn_on(Part::Firmware).unwrap();

    assert!(switches.is_on(Part::Firmware));
    assert!(!switches.is_on(Part::Nix));
}

#[test]
fn each_flag_file_is_named_after_its_part_in_the_update_folder() {
    let switches = Switches::new(Path::new("/home/kuba/.config"));

    assert_eq!(
        switches.path(Part::Bitwarden),
        Path::new("/home/kuba/.config/kubaos/desktop/update/no-bitwarden")
    );
    assert_eq!(
        switches.path(Part::Nix),
        Path::new("/home/kuba/.config/kubaos/desktop/update/no-nix")
    );
    assert_eq!(
        switches.path(Part::Firmware),
        Path::new("/home/kuba/.config/kubaos/desktop/update/no-firmware")
    );
}
