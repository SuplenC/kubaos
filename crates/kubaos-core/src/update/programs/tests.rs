use super::Switches;
use crate::update::event::Program;

#[test]
fn each_program_is_on_until_its_flag_file_is_made() {
    let tmp = tempfile::tempdir().unwrap();
    let switches = Switches::new(tmp.path());

    assert!(switches.is_on(Program::Bitwarden));
    assert!(switches.is_on(Program::Nix));

    switches.turn_off(Program::Nix).unwrap();

    assert!(switches.is_on(Program::Bitwarden));
    assert!(!switches.is_on(Program::Nix));
    assert!(tmp.path().join("kubaos/desktop/update/no-nix").is_file());

    switches.turn_on(Program::Nix).unwrap();
    switches.turn_on(Program::Nix).unwrap();

    assert!(switches.is_on(Program::Nix));
}
